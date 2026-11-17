//! The app's MCP server (part M): JSON-RPC 2.0 over stdin and stdout, the
//! only transport. The server runs only while an AI app runs it, speaks over
//! that app's stdin and stdout, opens no port and makes no network call, so
//! the Loopback Server Guard has nothing to guard here. Where the app's work
//! lives behind its own loopback server, a tool calls that server like any
//! other client (a blocking `std::net::TcpStream` request is enough), and the
//! server keeps its guard and the AI gate.
//!
//! The kit owns this server rather than using an SDK (kit v8): it is small,
//! it needs only `serde`, `serde_json` and `schemars`, and it has no async
//! runtime. Each `tools/call` runs on its own thread, so a slow tool never
//! holds up the others.
//!
//! It speaks both lifecycles a client may use:
//! - the `initialize` handshake (2024-11-05 to 2025-11-25): the server echoes
//!   a version it knows and otherwise answers with the newest one;
//! - per-request `_meta` (2026-07-28, which Claude Code 2.1.326 uses):
//!   `server/discover`, then each request names its protocol version, client
//!   and capabilities, and each result says `"resultType": "complete"`.
//!
//! Its methods: `initialize`, `notifications/initialized`, `server/discover`,
//! `ping`, `tools/list`, `tools/call`, and empty `resources/list`,
//! `resources/templates/list` and `prompts/list`. Anything else is
//! method-not-found (-32601). The wire format matches what rmcp 3.10 sent
//! (kit v7), byte for byte: the harness tests it against recordings.
//!
//! The app builds a [`Server`] with one [`Tool`] per operation (read tools
//! [`Annotations::read`], write tools [`Annotations::write`]) and runs it
//! with [`Server::serve_stdio`]. The kit's test harness (`tests/mcp.rs`
//! beside the reference copy) is a complete example, and [`LocalClient`]
//! is the client an app's own tests use.

use std::io::{BufRead, Write};
use std::sync::Mutex;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};

use super::error::{ErrorCode, ErrorCodeList, Result};
use super::events::{EventLog, LogEvent, LogLevel};
use super::gate::Caller;
use super::now_ms;

/// The rules every app's server tells the model, after its own description.
pub const SHARED_RULES: &str = "\
Rules:
- You may read anything at any time through the read tools. For \"what went wrong\", start with diagnose.
- Change nothing on your own initiative. Plan or apply a change only when the user has asked for that change \
in so many words.
- To change something: call the plan tool, show the user the plan's steps, then call the apply tool with the \
planId and the user's request in their own words. A plan expires after 10 minutes and is used once.
- If a change is refused with ai.changes_off, the user has not allowed changes from AI tools in the app's \
Settings. Tell them; don't look for another way.
- Some actions are never tools: anything that sends, posts, pays, publishes, deploys or can't be undone stays a \
click in the app's window.
- Every error has a stable code, a message, a hint and whether retrying can work. Follow the hint.";

/// The app's description followed by [`SHARED_RULES`].
pub fn instructions(about_the_app: &str) -> String {
    format!("{}\n\n{SHARED_RULES}", about_the_app.trim_end())
}

// Protocol versions and JSON-RPC error codes

/// Every protocol version the server speaks, oldest first.
pub const PROTOCOL_VERSIONS: [&str; 5] = ["2024-11-05", "2025-03-26", "2025-06-18", "2025-11-25", "2026-07-28"];
/// The first version that replaced the `initialize` handshake with
/// per-request `_meta`.
pub const NO_INITIALIZE: &str = "2026-07-28";
/// The newest version with an `initialize` handshake: the answer to a client
/// asking for one the server doesn't know, or for a later one.
pub const LATEST_WITH_INITIALIZE: &str = "2025-11-25";

pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;
/// MCP's code for a per-request protocol version the server doesn't speak.
pub const UNSUPPORTED_PROTOCOL_VERSION: i64 = -32022;

const META_VERSION: &str = "io.modelcontextprotocol/protocolVersion";
const META_CLIENT: &str = "io.modelcontextprotocol/clientInfo";
const META_CAPABILITIES: &str = "io.modelcontextprotocol/clientCapabilities";

// Tools

/// What a tool says about itself. Every field is sent, as rmcp's `#[tool]`
/// did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub idempotent_hint: bool,
    pub open_world_hint: bool,
}

impl Annotations {
    /// A tool that changes nothing (a read, or a plan). `idempotent`: calling
    /// it twice answers the same. `open_world`: it reaches past the app's own
    /// data (a remote service).
    pub const fn read(idempotent: bool, open_world: bool) -> Self {
        Self { read_only_hint: true, destructive_hint: false, idempotent_hint: idempotent, open_world_hint: open_world }
    }

    /// A tool that writes: destructive, never idempotent.
    pub const fn write(open_world: bool) -> Self {
        Self { read_only_hint: false, destructive_hint: true, idempotent_hint: false, open_world_hint: open_world }
    }
}

/// A tool's answer: the value as structured content (and as text, for
/// clients that read only text), or an error as text marked `isError`.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
    pub text: String,
    pub structured: Option<Value>,
    pub is_error: bool,
}

impl ToolResult {
    /// A success: `value` as structured content and as compact JSON text.
    pub fn structured(value: Value) -> Self {
        Self { text: value.to_string(), structured: Some(value), is_error: false }
    }

    /// A failure the model reads as text.
    pub fn error(text: impl Into<String>) -> Self {
        Self { text: text.into(), structured: None, is_error: true }
    }
}

/// A tool's result: the value as structured content, or, on failure, the
/// same `ErrorReport` the command line prints, as text marked `isError`, so
/// the model can act on it and it never has to match the success schema.
pub fn tool_result<T: Serialize>(r: Result<T>, schema_version: u32) -> ToolResult {
    match r {
        Ok(v) => ToolResult::structured(serde_json::to_value(v).expect("tool results serialize")),
        Err(e) => ToolResult::error(serde_json::to_string_pretty(&e.report(schema_version)).expect("errors serialize")),
    }
}

/// The result of the `list_error_codes` tool every app offers.
pub fn error_codes(schema_version: u32, all: &[ErrorCode]) -> ToolResult {
    tool_result(Ok(ErrorCodeList::of(schema_version, all)), schema_version)
}

/// One tool call, as the tool sees it.
#[derive(Debug, Clone)]
pub struct Call {
    caller: Caller,
}

impl Call {
    /// The caller, named by the AI app's own `clientInfo`.
    pub fn caller(&self) -> &Caller {
        &self.caller
    }
}

type Handler = Box<dyn Fn(Map<String, Value>, &Call) -> ToolResult + Send + Sync>;

/// One tool: its name, description, input schema, annotations and handler.
pub struct Tool {
    name: String,
    description: String,
    input_schema: Map<String, Value>,
    annotations: Annotations,
    run: Handler,
}

impl Tool {
    /// A tool with no arguments.
    pub fn new<F>(name: &str, description: &str, annotations: Annotations, run: F) -> Self
    where
        F: Fn(&Call) -> ToolResult + Send + Sync + 'static,
    {
        let schema = json!({ "type": "object", "properties": {} });
        Self::build(name, description, annotations, object(schema), Box::new(move |_, call| run(call)))
    }

    /// A tool whose arguments are `A`, with `A`'s JSON Schema (2020-12, from
    /// `schemars`) as its input schema. Arguments that don't match `A` are
    /// answered as an `isError` result, so the model can correct them.
    pub fn with_args<A, F>(name: &str, description: &str, annotations: Annotations, run: F) -> Self
    where
        A: DeserializeOwned + schemars::JsonSchema,
        F: Fn(A, &Call) -> ToolResult + Send + Sync + 'static,
    {
        Self::build(
            name,
            description,
            annotations,
            input_schema::<A>(),
            Box::new(move |args, call| match serde_json::from_value::<A>(Value::Object(args)) {
                Ok(args) => run(args, call),
                Err(e) => ToolResult::error(format!("failed to deserialize parameters: {e}")),
            }),
        )
    }

    fn build(
        name: &str,
        description: &str,
        annotations: Annotations,
        input_schema: Map<String, Value>,
        run: Handler,
    ) -> Self {
        Self { name: name.into(), description: description.into(), input_schema, annotations, run }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// `A`'s input schema as rmcp 3.10 wrote it: draft 2020-12, the root an
/// object, its own `title` and `description` (the type's name and doc)
/// removed.
pub fn input_schema<A: schemars::JsonSchema>() -> Map<String, Value> {
    let schema = schemars::generate::SchemaSettings::draft2020_12().into_generator().into_root_schema_for::<A>();
    let mut schema = object(serde_json::to_value(schema).expect("schemas serialize"));
    assert_eq!(schema.get("type"), Some(&json!("object")), "a tool's arguments must be an object");
    schema.remove("title");
    schema.remove("description");
    schema
}

fn object(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}

// The server

/// An MCP server: its identity, instructions and tools.
pub struct Server {
    name: String,
    version: String,
    instructions: String,
    /// Sorted by name, as `tools/list` returns them.
    tools: Vec<Tool>,
    clients: Option<EventLog>,
}

impl Server {
    /// A server with no tools yet. `instructions` is usually
    /// [`instructions`]`(about_the_app)`.
    pub fn new(name: &str, version: &str, instructions: String) -> Self {
        Self { name: name.into(), version: version.into(), instructions, tools: Vec::new(), clients: None }
    }

    /// Adds a tool. Panics on a second tool with the same name: that is a
    /// mistake in the app, found by its first test.
    pub fn tool(mut self, tool: Tool) -> Self {
        match self.tools.binary_search_by(|t| t.name.as_str().cmp(&tool.name)) {
            Ok(_) => panic!("two MCP tools are named {}", tool.name),
            Err(at) => self.tools.insert(at, tool),
        }
        self
    }

    /// Records each AI app that connects (its `clientInfo` name and version)
    /// in `log`, so Settings can show "Last used by Claude Code 2.1.326 at
    /// 22:51": proof from the AI app itself, which costs no tokens. Leave it
    /// out in tests, which must not write to the owner's data folder.
    pub fn recording_clients_in(mut self, log: EventLog) -> Self {
        self.clients = Some(log);
        self
    }

    pub fn records_clients(&self) -> bool {
        self.clients.is_some()
    }

    pub fn tools(&self) -> impl Iterator<Item = &Tool> {
        self.tools.iter()
    }

    /// Serves MCP on stdin and stdout until the client closes stdin.
    pub fn serve_stdio(&self) -> std::result::Result<(), String> {
        self.serve(std::io::stdin().lock(), &mut std::io::stdout())
    }

    /// Serves MCP on `input` and `output`, one JSON message per line, until
    /// `input` ends. A tool call still running then is answered before this
    /// returns. [`Server::serve_stdio`] is this on stdin and stdout; tests
    /// pass bytes.
    pub fn serve<W: Write + Send>(&self, mut input: impl BufRead, output: &mut W) -> std::result::Result<(), String> {
        let session = Session::default();
        let output = Mutex::new(output);
        let send = |line: String| {
            let mut out = output.lock().unwrap_or_else(|e| e.into_inner());
            // A client that has gone away can't be told; reading stops at EOF.
            let _ = out.write_all(line.as_bytes()).and_then(|()| out.write_all(b"\n")).and_then(|()| out.flush());
        };
        std::thread::scope(|scope| {
            let mut bytes = Vec::new();
            loop {
                bytes.clear();
                match input.read_until(b'\n', &mut bytes) {
                    Ok(0) => return Ok(()),
                    Ok(_) => {}
                    Err(e) => return Err(format!("The MCP server couldn't read its input: {e}")),
                }
                let line = String::from_utf8_lossy(&bytes);
                if line.trim().is_empty() {
                    continue;
                }
                match self.handle(&session, &line) {
                    Reply::None => {}
                    Reply::Now(line) => send(line),
                    Reply::Later(work) => {
                        let send = &send;
                        scope.spawn(move || send(work()));
                    }
                }
            }
        })
    }

    fn handle<'s>(&'s self, session: &Session, line: &str) -> Reply<'s> {
        let message: Value = match serde_json::from_str(line) {
            Ok(m) => m,
            Err(_) => return Reply::Now(error_line(&Value::Null, PARSE_ERROR, "Parse error", None)),
        };
        let Value::Object(message) = message else {
            return Reply::Now(error_line(&Value::Null, INVALID_REQUEST, "Invalid Request", None));
        };
        let Some(method) = message.get("method") else {
            // A response to a request the server never sends: nothing to do.
            return Reply::None;
        };
        let id = match message.get("id") {
            Some(id @ (Value::String(_) | Value::Number(_))) => id.clone(),
            Some(_) => return Reply::Now(error_line(&Value::Null, INVALID_REQUEST, "Invalid Request", None)),
            None => {
                if method == "notifications/initialized" {
                    self.record(session.client());
                }
                return Reply::None;
            }
        };
        let Some(method) = method.as_str() else {
            return Reply::Now(error_line(&id, INVALID_REQUEST, "Invalid Request", None));
        };
        let params = message.get("params").and_then(Value::as_object).cloned().unwrap_or_default();
        let meta = params.get("_meta").and_then(Value::as_object);
        let inline = meta.is_some_and(|m| m.contains_key(META_VERSION));

        // The request's lifecycle: its own `_meta`, or the handshake.
        let (stateless, client) = if inline {
            let meta = meta.expect("checked above");
            let version = meta.get(META_VERSION).and_then(Value::as_str);
            let mut missing = Vec::new();
            if version.is_none() {
                missing.push(META_VERSION);
            }
            if !meta.get(META_CAPABILITIES).is_some_and(Value::is_object) {
                missing.push(META_CAPABILITIES);
            }
            if !missing.is_empty() {
                return Reply::Now(missing_meta(&id, &missing));
            }
            let version = version.expect("checked above");
            if !PROTOCOL_VERSIONS.contains(&version) {
                let data = json!({ "requested": version, "supported": PROTOCOL_VERSIONS });
                return Reply::Now(error_line(
                    &id,
                    UNSUPPORTED_PROTOCOL_VERSION,
                    "Unsupported protocol version",
                    Some(data),
                ));
            }
            let client = ClientInfo::from(meta.get(META_CLIENT));
            if session.inline_client_changed(&client) {
                self.record(Some(client.clone()));
            }
            (version >= NO_INITIALIZE, Some(client))
        } else {
            match method {
                "ping" => return Reply::Now(result_line(&id, &Empty {})),
                "initialize" => return Reply::Now(self.initialize(session, &id, &params)),
                _ if !session.initialized() => {
                    return Reply::Now(missing_meta(&id, &[META_VERSION, META_CAPABILITIES]));
                }
                _ => (false, session.client()),
            }
        };
        let stamp = stateless.then_some("complete");

        match method {
            "ping" => Reply::Now(result_line(&id, &Empty {})),
            "initialize" => Reply::Now(self.initialize(session, &id, &params)),
            "server/discover" => Reply::Now(result_line(
                &id,
                &Discover {
                    result_type: "complete",
                    supported_versions: &PROTOCOL_VERSIONS,
                    capabilities: Capabilities::default(),
                    instructions: &self.instructions,
                    ttl_ms: 0,
                    cache_scope: "private",
                    meta: DiscoverMeta { server_info: ServerInfo { name: &self.name, version: &self.version } },
                },
            )),
            "tools/list" => Reply::Now(result_line(
                &id,
                &ToolList {
                    result_type: stamp,
                    ttl_ms: stamp.map(|_| 0),
                    cache_scope: stamp.map(|_| "public"),
                    tools: self.tools.iter().map(ToolDef::of).collect(),
                },
            )),
            "tools/call" => {
                let Some(tool) = params.get("name").and_then(Value::as_str) else {
                    return Reply::Now(error_line(&id, INVALID_PARAMS, "tool name is missing", None));
                };
                let Ok(at) = self.tools.binary_search_by(|t| t.name.as_str().cmp(tool)) else {
                    return Reply::Now(error_line(&id, INVALID_PARAMS, "tool not found", None));
                };
                let tool = &self.tools[at];
                let args = params.get("arguments").and_then(Value::as_object).cloned().unwrap_or_default();
                let call = Call { caller: Caller::Mcp { client: client.map(|c| c.name).unwrap_or_default() } }
                    .named_or_unknown();
                Reply::Later(Box::new(move || {
                    let run = std::panic::AssertUnwindSafe(|| (tool.run)(args, &call));
                    match std::panic::catch_unwind(run) {
                        Ok(result) => result_line(&id, &CallResult::of(&result, stamp)),
                        Err(_) => error_line(&id, INTERNAL_ERROR, &format!("The tool {} stopped.", tool.name), None),
                    }
                }))
            }
            "resources/list" => {
                Reply::Now(result_line(&id, &Listing { result_type: stamp, resources: Some([]), ..Listing::EMPTY }))
            }
            "resources/templates/list" => Reply::Now(result_line(
                &id,
                &Listing { result_type: stamp, resource_templates: Some([]), ..Listing::EMPTY },
            )),
            "prompts/list" => {
                Reply::Now(result_line(&id, &Listing { result_type: stamp, prompts: Some([]), ..Listing::EMPTY }))
            }
            other => Reply::Now(error_line(&id, METHOD_NOT_FOUND, other, None)),
        }
    }

    fn initialize(&self, session: &Session, id: &Value, params: &Map<String, Value>) -> String {
        let asked = params.get("protocolVersion").and_then(Value::as_str).unwrap_or_default();
        let version =
            if asked < NO_INITIALIZE && PROTOCOL_VERSIONS.contains(&asked) { asked } else { LATEST_WITH_INITIALIZE };
        session.initialize(ClientInfo::from(params.get("clientInfo")));
        result_line(
            id,
            &Initialize {
                protocol_version: version,
                capabilities: Capabilities::default(),
                server_info: ServerInfo { name: &self.name, version: &self.version },
                instructions: &self.instructions,
            },
        )
    }

    fn record(&self, client: Option<ClientInfo>) {
        if let Some(log) = &self.clients {
            let client = client.unwrap_or_default();
            let caller = Call { caller: Caller::Mcp { client: client.name } }.named_or_unknown().caller;
            let _ = log.append(&client_event(&caller, &client.version, now_ms()));
        }
    }

    /// A client for the app's own tests, in this process: it sends real
    /// JSON-RPC lines through the server's own dispatch, after an
    /// `initialize` naming `client_name`.
    pub fn connect(&self, client_name: &str) -> LocalClient<'_> {
        let mut client = LocalClient {
            server: self,
            session: Session::default(),
            next_id: Mutex::new(0),
            instructions: String::new(),
        };
        let init = client.request(
            "initialize",
            json!({
                "protocolVersion": LATEST_WITH_INITIALIZE,
                "capabilities": {},
                "clientInfo": { "name": client_name, "version": "0" },
            }),
        );
        client.notify("notifications/initialized");
        client.instructions = init.expect("initialize succeeds")["instructions"].as_str().unwrap_or_default().into();
        client
    }
}

impl Call {
    fn named_or_unknown(self) -> Self {
        match self.caller {
            Caller::Mcp { client } if client.is_empty() => Call { caller: Caller::Mcp { client: "unknown".into() } },
            caller => Call { caller },
        }
    }
}

enum Reply<'s> {
    None,
    Now(String),
    Later(Box<dyn FnOnce() -> String + Send + 's>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ClientInfo {
    name: String,
    version: String,
}

impl From<Option<&Value>> for ClientInfo {
    fn from(info: Option<&Value>) -> Self {
        let field = |key| info.and_then(|i| i.get(key)).and_then(Value::as_str).unwrap_or_default().to_string();
        Self { name: field("name"), version: field("version") }
    }
}

/// What one connection knows: the handshake's client, and the last client
/// a per-request `_meta` named (recorded again only when it changes).
#[derive(Default)]
struct Session {
    handshake: Mutex<Option<ClientInfo>>,
    inline: Mutex<Option<ClientInfo>>,
}

impl Session {
    fn initialize(&self, client: ClientInfo) {
        *self.handshake.lock().unwrap_or_else(|e| e.into_inner()) = Some(client);
    }

    fn initialized(&self) -> bool {
        self.handshake.lock().unwrap_or_else(|e| e.into_inner()).is_some()
    }

    fn client(&self) -> Option<ClientInfo> {
        self.handshake.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn inline_client_changed(&self, client: &ClientInfo) -> bool {
        let mut last = self.inline.lock().unwrap_or_else(|e| e.into_inner());
        if last.as_ref() == Some(client) {
            return false;
        }
        *last = Some(client.clone());
        true
    }
}

// The wire format, field order as rmcp 3.10 wrote it

#[derive(Serialize, Default)]
struct Empty {}

#[derive(Serialize, Default)]
struct Capabilities {
    tools: Empty,
}

#[derive(Serialize)]
struct ServerInfo<'a> {
    name: &'a str,
    version: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Initialize<'a> {
    protocol_version: &'a str,
    capabilities: Capabilities,
    server_info: ServerInfo<'a>,
    instructions: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Discover<'a> {
    result_type: &'a str,
    supported_versions: &'a [&'a str],
    capabilities: Capabilities,
    instructions: &'a str,
    ttl_ms: u64,
    cache_scope: &'a str,
    #[serde(rename = "_meta")]
    meta: DiscoverMeta<'a>,
}

#[derive(Serialize)]
struct DiscoverMeta<'a> {
    #[serde(rename = "io.modelcontextprotocol/serverInfo")]
    server_info: ServerInfo<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolList<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    result_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_scope: Option<&'a str>,
    tools: Vec<ToolDef<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolDef<'a> {
    name: &'a str,
    description: &'a str,
    input_schema: &'a Map<String, Value>,
    annotations: Annotations,
}

impl<'a> ToolDef<'a> {
    fn of(tool: &'a Tool) -> Self {
        Self {
            name: &tool.name,
            description: &tool.description,
            input_schema: &tool.input_schema,
            annotations: tool.annotations,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CallResult<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    result_type: Option<&'a str>,
    content: [TextBlock<'a>; 1],
    #[serde(skip_serializing_if = "Option::is_none")]
    structured_content: Option<&'a Value>,
    is_error: bool,
}

impl<'a> CallResult<'a> {
    fn of(result: &'a ToolResult, result_type: Option<&'a str>) -> Self {
        Self {
            result_type,
            content: [TextBlock { kind: "text", text: &result.text }],
            structured_content: result.structured.as_ref(),
            is_error: result.is_error,
        }
    }
}

#[derive(Serialize)]
struct TextBlock<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    text: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Listing<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    result_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resources: Option<[(); 0]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_templates: Option<[(); 0]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompts: Option<[(); 0]>,
}

impl Listing<'_> {
    const EMPTY: Self = Self { result_type: None, resources: None, resource_templates: None, prompts: None };
}

#[derive(Serialize)]
struct RpcError<'a> {
    code: i64,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Serialize)]
struct Response<'a, R: Serialize> {
    jsonrpc: &'static str,
    id: &'a Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<&'a R>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError<'a>>,
}

fn result_line<R: Serialize>(id: &Value, result: &R) -> String {
    serde_json::to_string(&Response { jsonrpc: "2.0", id, result: Some(result), error: None })
        .expect("responses serialize")
}

fn error_line(id: &Value, code: i64, message: &str, data: Option<Value>) -> String {
    let error = RpcError { code, message, data };
    serde_json::to_string(&Response::<Empty> { jsonrpc: "2.0", id, result: None, error: Some(error) })
        .expect("responses serialize")
}

fn missing_meta(id: &Value, missing: &[&str]) -> String {
    let message = format!("request _meta is missing or has malformed required fields: {}", missing.join(", "));
    error_line(id, INVALID_PARAMS, &message, None)
}

// The test client

/// An in-process MCP client for an app's tests: see [`Server::connect`].
pub struct LocalClient<'s> {
    server: &'s Server,
    session: Session,
    next_id: Mutex<u64>,
    instructions: String,
}

impl LocalClient<'_> {
    /// Sends one request and returns its `result`, or its `error` object.
    pub fn request(&self, method: &str, params: Value) -> std::result::Result<Value, Value> {
        let id = {
            let mut next = self.next_id.lock().unwrap_or_else(|e| e.into_inner());
            *next += 1;
            *next
        };
        let line = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }).to_string();
        let answer = match self.server.handle(&self.session, &line) {
            Reply::None => panic!("no answer to {method}"),
            Reply::Now(answer) => answer,
            Reply::Later(work) => work(),
        };
        let mut answer: Value = serde_json::from_str(&answer).expect("answers are JSON");
        assert_eq!(answer["id"], json!(id), "the answer names its request");
        match answer.get_mut("error") {
            Some(error) => Err(error.take()),
            None => Ok(answer["result"].take()),
        }
    }

    /// Sends one notification.
    pub fn notify(&self, method: &str) {
        let line = json!({ "jsonrpc": "2.0", "method": method }).to_string();
        assert!(matches!(self.server.handle(&self.session, &line), Reply::None), "a notification is never answered");
    }

    /// `tools/list`'s tools.
    pub fn list_tools(&self) -> Vec<Value> {
        match self.request("tools/list", json!({})).expect("tools/list succeeds")["tools"].take() {
            Value::Array(tools) => tools,
            other => panic!("tools/list answered {other}"),
        }
    }

    /// `tools/call`'s result for one tool.
    pub fn call(&self, tool: &str, arguments: Value) -> std::result::Result<Value, Value> {
        self.request("tools/call", json!({ "name": tool, "arguments": arguments }))
    }

    /// The server's instructions, as `initialize` gave them.
    pub fn instructions(&self) -> &str {
        &self.instructions
    }
}

// The last client seen

/// The event name the server logs for each AI app that connects.
pub const CLIENT_SEEN: &str = "mcp.client_seen";

fn client_event(caller: &Caller, version: &str, ts_ms: u64) -> LogEvent {
    let version: String =
        version.chars().filter(|c| c.is_ascii_alphanumeric() || ".-+".contains(*c)).take(32).collect();
    LogEvent {
        ts_ms,
        level: LogLevel::Info,
        event: CLIENT_SEEN.into(),
        caller: caller.label(),
        target: None,
        subject: None,
        plan: None,
        message: version,
        error: None,
    }
}

/// The AI app that last connected, as [`Server::recording_clients_in`]
/// logged it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastClient {
    /// The client's own name, such as `claude-code`.
    pub client: String,
    pub version: String,
    pub ts_ms: u64,
}

/// The newest client-seen entry among the last `scan` events.
pub fn last_client(log: &EventLog, scan: usize) -> Option<LastClient> {
    log.recent(scan).into_iter().rev().find(|e| e.event == CLIENT_SEEN).map(|e| LastClient {
        client: e.caller.strip_prefix("mcp:").unwrap_or(&e.caller).to_string(),
        version: e.message,
        ts_ms: e.ts_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::super::error::{AgentError, codes};
    use super::*;

    #[test]
    fn failures_are_error_reports_and_successes_are_structured() {
        let ok = tool_result(Ok(json!({ "a": 1 })), 4);
        assert!(!ok.is_error);
        assert_eq!(ok.structured.unwrap()["a"], 1);
        assert_eq!(ok.text, r#"{"a":1}"#);
        let err = tool_result::<()>(Err(AgentError::new(codes::AI_CHANGES_OFF, "off")), 4);
        assert!(err.is_error);
        assert!(err.structured.is_none());
        assert!(err.text.contains("ai.changes_off") && err.text.contains("schemaVersion"));
        assert!(instructions("About.\n").starts_with("About.\n\nRules:"));
    }

    #[test]
    fn the_last_client_seen_is_read_back_from_the_event_log() {
        let tmp = super::super::TestDir::new("last-client");
        let log = EventLog::new(tmp.0.join("logs"), "app");
        assert_eq!(last_client(&log, 50), None);
        let claude = Caller::Mcp { client: "claude-code".into() };
        log.append(&client_event(&claude, "2.1.325", 1_000)).unwrap();
        log.append(&client_event(&Caller::Mcp { client: "codex".into() }, "0.1\n<x>", 2_000)).unwrap();
        let last = last_client(&log, 50).unwrap();
        assert_eq!((last.client.as_str(), last.version.as_str(), last.ts_ms), ("codex", "0.1x", 2_000));
    }

    fn echo() -> Server {
        Server::new("echo", "1", instructions("Echo.")).tool(Tool::new(
            "hi",
            "Says hi.",
            Annotations::read(true, false),
            |call| ToolResult::structured(json!({ "caller": call.caller().label() })),
        ))
    }

    fn one(server: &Server, session: &Session, line: &str) -> Option<Value> {
        match server.handle(session, line) {
            Reply::None => None,
            Reply::Now(l) => Some(serde_json::from_str(&l).unwrap()),
            Reply::Later(work) => Some(serde_json::from_str(&work()).unwrap()),
        }
    }

    #[test]
    fn json_rpc_errors_have_their_codes() {
        let server = echo();
        let s = Session::default();
        let code = |line: &str| one(&server, &s, line).unwrap()["error"]["code"].as_i64().unwrap();
        assert_eq!(code("not json"), PARSE_ERROR);
        assert_eq!(code("[]"), INVALID_REQUEST);
        assert_eq!(code(r#"{"jsonrpc":"2.0","id":{},"method":"ping"}"#), INVALID_REQUEST);
        assert_eq!(code(r#"{"jsonrpc":"2.0","id":1,"method":7}"#), INVALID_REQUEST);
        assert_eq!(code(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#), INVALID_PARAMS, "no lifecycle yet");
        assert_eq!(one(&server, &s, r#"{"jsonrpc":"2.0","method":"notifications/cancelled"}"#), None);
        assert_eq!(one(&server, &s, r#"{"jsonrpc":"2.0","id":1,"result":{}}"#), None);
        let init = r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}"#;
        assert_eq!(one(&server, &s, init).unwrap()["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(code(r#"{"jsonrpc":"2.0","id":3,"method":"nope"}"#), METHOD_NOT_FOUND);
        assert_eq!(code(r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{}}"#), INVALID_PARAMS);
        let hi =
            one(&server, &s, r#"{"jsonrpc":"2.0","id":"x","method":"tools/call","params":{"name":"hi"}}"#).unwrap();
        assert_eq!(hi["id"], "x");
        assert_eq!(hi["result"]["structuredContent"]["caller"], "mcp:unknown", "a client without a name");
    }

    #[test]
    fn a_tool_that_panics_is_an_internal_error_not_a_crash() {
        let server =
            Server::new("p", "1", String::new())
                .tool(Tool::new("boom", "", Annotations::write(false), |_| panic!("boom")));
        let client = server.connect("t");
        assert_eq!(client.call("boom", json!({})).unwrap_err()["code"], INTERNAL_ERROR);
        assert!(client.call("boom", json!({})).is_err(), "the server keeps serving");
    }

    #[test]
    #[should_panic(expected = "two MCP tools are named hi")]
    fn two_tools_with_one_name_are_refused() {
        let _ = echo().tool(Tool::new("hi", "", Annotations::read(true, false), |_| ToolResult::error("")));
    }
}

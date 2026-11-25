//! Keeplight's MCP server, `keeplight-cli mcp`: the engine's operations as
//! tools, over stdio only (no port, no network). Shaped like the kit's
//! example server (jamarq-agent-kit `rust/tests/mcp.rs`): read tools and the
//! plan tool are read-only, `apply_plan` is destructive, and failures are
//! `ErrorReport`s marked `isError`. The server is the kit's own (kit 8): no
//! async runtime, and each tool call runs on its own thread.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::agent::{ALL_CODES, Change, Engine, SCHEMA_VERSION};
use crate::agent_kit::mcp::{self, Annotations, Server, Tool};

const ABOUT: &str = "Keeplight keeps the lights in this PC the way the user set them. OpenRGB does the hardware \
work; Keeplight keeps a setup per device and puts it back at sign-in, after sleep and when OpenRGB's devices change.

- get_devices lists every device with an id, its mode and color, whether Keeplight keeps it (kept, differs, \
changed or notKept), what it was kept as, and what probably lights it.
- A change is one action on one device: keep (what it shows now), keepDark (only when every light is off), putBack, \
stopKeeping (a Direct device is handed back to its own Onboard lighting), keepAsBefore (goes back to the setup \
before the last stopKeeping or setColor; on a kept device the two swap) or setColor with a color as #rrggbb, and optionally a zone id from get_devices to color only that zone (keeps \
the device's effect in the new color, or switches to Static or Direct; the color is kept). For a whole-PC color, \
plan one setColor per device.
- What get_devices reads comes from OpenRGB's record. After sleep, RAM can show its own rainbow while the record \
still says the kept color; putBack fixes that.";

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ApplyArgs {
    /// The planId from plan_change.
    plan_id: String,
    /// What the user asked for, in their own words.
    request: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct LogArgs {
    /// How many of the newest events to return; 50 when left out, at most 500.
    limit: Option<usize>,
}

/// Keeplight's tools on the kit's server: the same names, descriptions,
/// schemas and annotations as the rmcp server they replace.
pub fn server(engine: Engine) -> Server {
    let engine = Arc::new(engine);
    let read = Annotations::read(true, false);
    let (devices, logs, plan, apply) = (engine.clone(), engine.clone(), engine.clone(), engine.clone());
    Server::new("keeplight", env!("CARGO_PKG_VERSION"), mcp::instructions(ABOUT))
        .tool(Tool::new(
            "list_error_codes",
            "Every error code: what it means, what to do, whether retrying can work.",
            read,
            |_| mcp::error_codes(SCHEMA_VERSION, &ALL_CODES),
        ))
        .tool(Tool::new(
            "get_devices",
            "Every lighting device OpenRGB reports: id, name, mode, color, whether Keeplight keeps it and \
             as what, and what probably lights it. Also whether AI changes are allowed.",
            read,
            move |_| mcp::tool_result(devices.devices(), SCHEMA_VERSION),
        ))
        .tool(Tool::with_args(
            "get_logs",
            "The newest events: every change, who made it (gui, cli or mcp:<app>), and every failure with \
             its code.",
            read,
            move |a: LogArgs, _| {
                let events = logs.recent_events(a.limit.unwrap_or(50).min(500));
                mcp::tool_result(
                    Ok(serde_json::json!({ "schemaVersion": SCHEMA_VERSION, "events": events })),
                    SCHEMA_VERSION,
                )
            },
        ))
        .tool(Tool::new(
            "diagnose",
            "Everything for working out a problem at once, redacted: OpenRGB, the devices, the setup in \
             counts, the settings, the keeper's last runs and the newest events. Start here for \"what \
             went wrong\".",
            read,
            move |_| mcp::tool_result(Ok(engine.diagnose(serde_json::json!({ "frontEnd": "mcp" }))), SCHEMA_VERSION),
        ))
        .tool(Tool::with_args(
            "plan_change",
            "Checks one change the user asked for (an action on a device) and returns a plan: its steps \
             in plain words and a planId. Changes nothing else.",
            Annotations::read(false, false),
            move |change: Change, call| mcp::tool_result(plan.plan(call.caller(), change), SCHEMA_VERSION),
        ))
        .tool(Tool::with_args(
            "apply_plan",
            "Applies a plan from plan_change, once, if nothing it depends on changed since. Only when the \
             user asked for this change; pass their request. Refused (ai.changes_off) unless the user \
             allows changes from AI tools in Keeplight's Settings.",
            Annotations::write(false),
            move |a: ApplyArgs, call| {
                mcp::tool_result(apply.apply(call.caller(), &a.plan_id, Some(&a.request)), SCHEMA_VERSION)
            },
        ))
}

/// Serves MCP on stdin and stdout until the client disconnects.
pub fn serve(engine: Engine) -> Result<(), String> {
    server(engine).serve_stdio()
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::agent_kit::journal;
    use crate::paths::Paths;
    use crate::testing::{self, FakeServer};

    fn error_code(r: &Value) -> String {
        let text = r["content"][0]["text"].as_str().unwrap();
        serde_json::from_str::<Value>(text).unwrap()["error"]["code"].as_str().unwrap().to_string()
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let data = std::env::temp_dir().join(format!("keeplight-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&data);
        data
    }

    /// The server moved from rmcp to the kit's own (kit 8). Claude Code
    /// 2.1.326's own `server/discover` and `tools/list`, then an `initialize`
    /// session with tool calls, are answered byte for byte as the rmcp build
    /// answered them (recorded from main before the change, two sessions in
    /// one file). One difference on purpose: rmcp refused a `ping` without
    /// `_meta` once a client used per-request `_meta`; the kit answers it.
    /// The error-code list has gained `scratch.refused` since the recording.
    #[test]
    fn clients_see_the_same_server_as_under_rmcp() {
        let data = scratch("mcp-v7");
        let engine = || Engine::new(Paths { data: data.clone() }, "127.0.0.1:9");
        let requests: Vec<&str> = include_str!("fixtures/mcp-v7.requests.jsonl").lines().collect();
        let expected = include_str!("fixtures/mcp-v7.answers.jsonl")
            .replace(r#""version":"0.2.0""#, &format!(r#""version":"{}""#, env!("CARGO_PKG_VERSION")));
        let mut out = Vec::new();
        for session in [&requests[..3], &requests[3..]] {
            let input = session.join("\n") + "\n";
            server(engine()).serve(input.as_bytes(), &mut out).unwrap();
        }
        let out = String::from_utf8(out).unwrap();
        // Each tool call runs on its own thread, so their answers can come in
        // any order: compare the lines, not their order.
        let mut got: Vec<&str> = out.lines().collect();
        let mut want: Vec<&str> = expected.lines().collect();
        assert_eq!(got.len(), want.len(), "{out}");
        assert!(want[2].contains(r#""id":0,"error""#), "the stateless ping rmcp refused: {}", want[2]);
        want[2] = r#"{"jsonrpc":"2.0","id":0,"result":{}}"#;
        got.sort_unstable();
        want.sort_unstable();
        for (got, want) in got.iter().zip(&want) {
            assert_eq!(got, want);
        }
        let _ = std::fs::remove_dir_all(&data);
    }

    /// A client closing stdin ends the session cleanly.
    #[test]
    fn a_client_disconnecting_ends_the_session_cleanly() {
        let data = scratch("mcp-end");
        let engine = Engine::new(Paths { data: data.clone() }, "127.0.0.1:9");
        let mut out = Vec::new();
        assert_eq!(server(engine).serve(&b""[..], &mut out), Ok(()));
        assert!(out.is_empty());
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn tools_are_annotated_and_ai_changes_need_the_switch_and_a_request() {
        let data = scratch("mcp");
        let mut rainbow = testing::ram("SMBus 0x71");
        rainbow.active_mode = Some(1);
        let fake = FakeServer::start(vec![rainbow]);
        let address = fake.address.clone();
        let engine = || Engine::new(Paths { data: data.clone() }, address.clone());
        // Keep the stick as red Chase Fade from the window, then let it lose
        // power: OpenRGB's record goes back to its rainbow.
        fake.state.lock().unwrap().devices[0].active_mode = Some(2);
        engine()
            .change_now(&crate::agent_kit::gate::Caller::Window, Change::new(crate::agent::Action::Keep, "ram-stick"))
            .unwrap();
        fake.state.lock().unwrap().devices[0].active_mode = Some(1);

        for ai_changes_on in [false, true] {
            engine().settings();
            if ai_changes_on {
                engine().set_ai_changes(true).unwrap();
            }
            let mcp_server = server(engine());
            let client = mcp_server.connect("test");

            let tools = client.list_tools();
            let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
            assert_eq!(names.len(), 6, "{names:?}");
            for t in &tools {
                let writes = t["name"] == "apply_plan";
                assert_eq!(t["annotations"]["readOnlyHint"], json!(!writes), "{}", t["name"]);
                assert_eq!(t["annotations"]["destructiveHint"], json!(writes), "{}", t["name"]);
            }
            assert!(client.instructions().starts_with("Keeplight keeps the lights"));
            assert!(client.instructions().contains("Change nothing on your own initiative"));

            let call = |name: &str, args: Value| client.call(name, args).unwrap();
            let codes = call("list_error_codes", json!({}));
            assert_eq!(codes["structuredContent"]["codes"].as_array().unwrap().len(), ALL_CODES.len());

            let devices = call("get_devices", json!({}));
            let devices = &devices["structuredContent"];
            assert_eq!(devices["devices"][0]["id"], "ram-stick");
            assert_eq!(devices["devices"][0]["kept"], "differs");
            assert_eq!(devices["aiChanges"], ai_changes_on);

            let missing = call("plan_change", json!({ "action": "putBack", "device": "zzz" }));
            assert_eq!(error_code(&missing), "device.not_found");

            let plan = call("plan_change", json!({ "action": "putBack", "device": "RAM stick" }));
            let plan_id = plan["structuredContent"]["planId"].as_str().unwrap().to_string();
            let apply = |request: &str| call("apply_plan", json!({ "planId": plan_id.clone(), "request": request }));

            if !ai_changes_on {
                let refused = apply("put the RAM back");
                assert_eq!(refused["isError"], true);
                assert_eq!(error_code(&refused), "ai.changes_off");
                assert_eq!(fake.device(0).active_mode_name(), Some("Rainbow"), "nothing was sent");
            } else {
                assert_eq!(error_code(&apply("   ")), "ai.request_missing");
                let done = apply("put the RAM back");
                assert_eq!(done["isError"], false, "{done}");
                assert_eq!(fake.device(0).active_mode_name(), Some("Chase Fade"));
                assert_eq!(error_code(&apply("put the RAM back")), "plan.not_found");
                let entries = journal::read(&data);
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].caller, "mcp:test");
                assert_eq!(entries[0].request.as_deref(), Some("put the RAM back"));
                let logs = call("get_logs", json!({ "limit": 20 }));
                assert!(
                    logs["structuredContent"]["events"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|e| e["event"] == "device.put_back")
                );
                let diagnosis = call("diagnose", json!({}));
                assert_eq!(diagnosis["structuredContent"]["sections"]["setup"]["kept"], 1);
            }
        }
        let _ = std::fs::remove_dir_all(&data);
    }
}

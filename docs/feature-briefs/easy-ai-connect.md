# Keeplight: easy AI connect (Feature session brief)

Status: carded 2026-11-18 as Next Up item C in `kl-project-state.md`. Queued after B by the default order the owner took on 2026-11-18. Not started.

**Title line:**

```
Keeplight: a Connect button for AI apps in Settings
```

**Paste as the first message:**

```
Replace Keeplight's pasted `mcp add` lines with a Connect button for each AI app, copied from Vaexil Mod Manager's tested version. Read kl-project-state.md, docs/ai.md and docs/safety-model.md first.

Facts, checked 2026-11-18:
- Today Settings (and docs/ai.md) show three lines to paste: `claude mcp add --scope user keeplight -- "<path>\keeplight-cli.exe" mcp`, and the Gemini CLI and Codex equivalents.
- On 2026-11-14 that failed on JASON-PC: there is no `claude` on PATH, only the desktop app's bundled claude.exe, whose path changes with each app update. The session connected it by hand with that copy (user scope; the entry lives in the user's .claude.json), and a Code session then read the lights correctly.
- Vaexil Mod Manager built the design first: jmars319/vaexil-mod-manager#29 (merged 2026-11-14, 1fb9b1d, "Connect AI apps from Settings, check them, and list requests to try"), in crates/engine/src/ai_apps.rs, crates/app/src/gui/ai_apps.rs and ui/settings.slint: find the AI app wherever it is installed, run its own `mcp add` after the owner confirms, check with `mcp get`, show "last used by".
- Keeplight already carries the kit's connect module (crates/core/src/agent_kit/connect.rs, kit v9, KIT_VERSION 9), which finds the desktop app's bundled claude.exe. Keeplight's MCP server does not record clients today (`recording_clients_in` is not used), so "last used by" needs that turned on.
- AI changes stay behind `aiChanges` (only the window turns it on), a plan first, and a request.

Build it in one PR from your own worktree, following VMM's shape and the kit rather than inventing a new one. Kit bugs go to the agent-kit session, never patched in the copy. Done when: Connect works for Claude Code on JASON-PC through the bundled claude.exe (tried by the owner, since it changes their AI config), docs/ai.md matches, item C is removed from `## Next Up`, and the state file and plan page are updated.
```

# Keeplight: test OpenRGB 1.1 and add it to the tested list (Feature session brief)

Status: carded 2026-11-18 as Next Up item D in `kl-project-state.md`. Waits on the owner: picking it, and being at JASON-PC for the test (one admin prompt). Not started.

**Title line:**

```
Keeplight: OpenRGB 1.1 joins the tested versions
```

**Paste as the first message:**

```
Test OpenRGB 1.1 on JASON-PC with the owner, then add it to Keeplight's tested list so the window can offer it. Read kl-project-state.md and crates/core/src/installs.rs first.

Facts, checked 2026-11-18:
- `installs::TESTED` pins OpenRGB 1.0.0 (the codeberg MSI, size and SHA-256 pinned, same product code as JASON-PC's install) and PawnIO 2.2.0. Owner rule (2026-11-14): OpenRGB updates are tested versions only; 1.1 joins after a test on JASON-PC with the owner, never automatically.
- Keeplight talks to OpenRGB's SDK on 127.0.0.1:6742 with its own client at protocol 3 (OpenRGB 1.0 answers protocol 6). The installer path is core::installer: System32 curl.exe over https with a size cap, size and SHA-256 checked, run through ShellExecuteExW runas.
- What must still work after the upgrade: every device listed (RAM needs PawnIO), the keeper's put back at sign-in and wake, colors per device and zone, and the mode fields Keeplight sends back unchanged.

Do: pin 1.1's official Windows download (URL, size, SHA-256 from the release, checked against the publisher's own digest), then with the owner at the PC upgrade through Keeplight's own Install… button and run the checks above on the real lights, one device at a time to its current state. If anything fails, roll back to 1.0.0 the same way and record why. Add 1.1 to TESTED with tests, in one PR from your own worktree. Done when: 1.1 runs on JASON-PC with every check passing (or the failure is recorded and 1.0.0 is back), item D is removed from `## Next Up`, and the state file and plan page are updated.
```

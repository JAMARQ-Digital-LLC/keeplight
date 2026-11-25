# Keeplight history

What has been built and when, newest first. The plan page
(https://claude.ai/artifact/PisWk9vuT3sDiHuyRxWktu) shows what is upcoming and
links here; the state file (`kl-project-state.md`) holds current working state.
Plain Markdown on purpose: quick to read on GitHub or in an editor
(`docs/project-plan-artifact.md`, "Where history lives", in agentic-instructions).
Started 2026-11-23 from `templates/project-history.md`.

Moved from the plan page's Done tab on 2026-11-23, as written there; an entry's "now", "still" or "not yet" means as of its own date.

PR references rechecked 2026-11-23 against GitHub: 30 referenced, 30 merged; not merged: none. (29 in this repo, counting each PR in a range such as "#8 to #12", plus agentic-instructions#345.) keeplight#35, "in review" in the 18 Nov entry, has since merged.

## Stages

- **00. Stop the rainbow (24 Oct):** The task asks the running OpenRGB service to load `Red`. Proven through a sleep and a full shutdown. It is still holding after the account move.
- **01. Repo and first window (24 Oct):** A public repo, the Vaexil theme and fonts, an icon, and a read-only window listing every device with its current color (#1 to #4).
- **02. The keeper (13 Nov):** Kept devices go back to your colors at sign-in, on wake and when OpenRGB changes (#8 to #12). Your sleep, restart and keyboard tests passed.
- **03. One-click colors (14 Nov):** Color… on each device (#18): nine colors or any #rrggbb, kept at once. An effect keeps running in the new color; a device on its own lighting switches to Static, or Direct (your calls). A swatch per zone and a color for one zone (#20). Set everything… for one color on the whole PC (#21). Keep it as before undoes any of them. The command line and AI tools can do the same, AI only with its switch on. You tried a color on the real RAM.
- **04. Retire the old pieces (13 Nov):** Armoury Crate's lighting side uninstalled, and the `\JAMARQ\OpenRGB apply Red profile` task removed (its files kept in a `retired-2026-11-13` folder). Only Keeplight touches the lights now.
- **05. OpenRGB installs and update notice (14 Nov):** The window says which OpenRGB and PawnIO you have against the tested versions (#23), and offers to install a tested one when it is missing or older: from its official site, checked against a pinned checksum, with Windows' admin prompt (#24; your calls: download on demand, tested versions only). It says when a newer Keeplight is out (#25). Version 0.2.0's installer first stops a running Keeplight, so an AI session can't block an update (#26). Agent kit v7 copied, and the command line now needs the AI switch and a request like any AI tool (#27).

## Ledger

- 2026-11-25: **Dev builds never see your real lights or files.** A build from source, or from a worktree, is now "Keeplight Dev": its own folder and sign-in entry, so it can't stop or confuse your installed keeper, no update check or downloads, and it won't talk to OpenRGB on this PC unless told to by name. Tests and the click test run in a stricter scratch mode with a fake OpenRGB. Keeplight's folder is now readable only by your account (fix/scratch-safety, agentic-instructions decision 2026-11-24).
- 2026-11-18: **Main builds for Atlas, and releases on request** (Next Up A, keeplight#35, in review). A new Main build workflow, started by hand, builds the release's installer from a main commit, stamped `0.2.0+main.<sha>`. It is kept for 7 days for Atlas's Reinstall and published nowhere. The Release workflow now runs only when you start it, refusing a version that is already tagged. A main build's update notice offers only a real newer release, never a downgrade. Checked on GitHub: a branch run built `0.2.0+main.879d8b5`, and the installer, the command line and the window all read that version back. Nothing was installed; the first Reinstall is yours, from Atlas. The repo is public, so these builds can be downloaded by anyone signed in to GitHub.
- 2026-11-18: **Tier A auto-merge.** Your call: Dependabot minor and patch updates now merge themselves once the required checks pass (keeplight#34); majors still wait for you.
- 2026-11-18: **Repo moved into JAMARQ-Digital-LLC.** Your decision, so the PCs need only one org-owned GitHub token. The repository id, settings, `main` protection and the releases secret all came across, and old links redirect. Clones on the Mac and both PCs point at the new address, and keeplight#33 updates the links in the repo.
- 2026-11-18: **Next Up B to E, and the PC sessions saved.** The four options for after stage 05 became cards with briefs, and what was still true in the Windows sessions (installed build, device readings, sleep and ASUS findings) moved into the state file before those sessions were cleaned up. Agent kit v9 (#30, 17 Nov) is on main but not installed on this PC yet.
- 2026-11-14: **Stage 05 done, and 0.2.0.** OpenRGB and PawnIO status (#23), installs of tested versions (#24), the update notice (#25), version 0.2.0 with an installer that stops a running Keeplight first, tested over a live AI connection (#26), and agent kit v7 with the command line gated like AI tools (#27). Installed from main 20cea42; you published v0.2.0 the same night and the feed names it.
- 2026-11-14: **Set everything (stage 03, part 3).** One color for every device from the window or `set-color all` (#21), tried on the real PC as a same-color run (5 of 5). Installed.
- 2026-11-14: **Claude reads your lights.** Keeplight connected to Claude Code through the desktop app's own copy (there's no `claude` command on this PC), and a new Code session answered from it correctly: two RAM sticks kept red, three devices red from OpenRGB, the remembered blue, and AI changes off. An AI-made change on the real lights is still to try.
- 2026-11-14: **Colors per zone, and kit v3.** A swatch per zone and a zone choice in the picker (#20); tried read-only and same-color on the real motherboard's three zones. Agent kit v3 copied (#19), so Keeplight no longer carries its own fix for the kit's shutdown bug. Both installed on this PC.
- 2026-11-14: **One-click colors, part 1.** Color… on each device sets and keeps a color, and Keep it as before undoes it (#18). Codex found one gap before merging, a color on a device with no known mode could not be undone, now refused. Installed on this PC, and you tried it the same night: RAM stick 1 went to blue Chase Fade and back to red. The agent kit v3 copy followed in #19.
- 2026-11-13: **Stopgap retired (stage 04).** The October task that loaded the Red profile at sign-in and wake is gone; it had run every time since 23 Oct. Its script, log and profiles are kept in `%LOCALAPPDATA%\JAMARQ\openrgb\retired-2026-11-13`. The state file caught up in #17.
- 2026-11-13: **AI tools and a command line (kit pilot).** Keeplight is the fleet's first app on the agent-interface kit (#16): `keeplight-cli.exe` with commands and an MCP server for Claude Code, Gemini CLI and Codex. AI tools read and plan; changes only with "Let AI tools make changes" on, planned first and journaled. Found a real bug in the kit (its server crashed whenever an AI app disconnected), worked around and reported. Codex found two more before merging (a plan could be written after something else changed the setup, or land on the other identical RAM stick if OpenRGB reordered them), both fixed. Merged as f64fc67, installed on this PC, and the friction list sent to the consolidation session.
- 2026-11-13: **Keep it as before, and Lit by.** Stop keeping remembers the setup so one click brings it back; Put back on every kept device; a spectrum swatch for devices on their own lighting (#14). Each row says what probably lights it, from the mode and which lighting apps are running (#15).
- 2026-11-13: **The keeper, built.** Puts kept devices back at sign-in, after the PC wakes and when OpenRGB changes (#9, #10). Tells you when one doesn't take, with an optional tray icon (#11). Stop keeping hands a device back to its own lighting, and keeping one dark asks first (#12). Codex found five real issues across these, all fixed before merging. Installed on this PC; your sleep, restart and keyboard tests all passed the same evening.
- 2026-11-13: **Keep and Put back.** Keeplight's first changes to lights, one device at a time when you press a button, with its own setup file. Auto-merge turned on for Keeplight, Modlist Launcher and Vaexil Mod Manager (#8).
- 2026-11-13: **Armoury Crate's lighting side uninstalled.** You chose this. Removed: the Armoury Crate app and service, ROG Live Service, and the AURA lighting add-ons for the motherboard, RAM (ASUS and Kingston), extension card and peripherals. Kept: ASUS fan control, which is still running. All 216 LEDs read red afterward.
- 2026-11-13: **Plan page refreshed into tabs** after a first-hand check. Claude recorded as a "maybe later" option.
- 2026-11-08: **Moved to the Jason account and installed.** The repo moved to `C:\JAMARQ\Apps\keeplight`. Keeplight 0.1.0 is installed for your account. OpenRGB's profiles were copied over, and the re-apply task now runs as Jason.
- 2026-11-08: **Release path to the shared feed.** Publishing a reviewed draft release copies the installer to `jamarq-releases` (#6), using its own token (#7).
- 2026-11-08: **Two Slint advisories cleared.** `pulldown-cmark` updated, and `pin-utils` ignored until 8 Feb 2027 (#5).
- 2026-10-24: **Mouse-click fix in the theme** copied from Modlist Launcher (#4).
- 2026-10-24: **Icon as SVG and 1024 px.** JAMARQ family; brand set built on the Mac (#3).
- 2026-10-24: **Repo made public and filled out.** Security settings on, community files in place, health 85% (#2). Registered in agentic-instructions #345.
- 2026-10-24: **Read-only first window** with Keeplight's own control-port client (#1).
- 2026-10-23: **Root cause found and stopgap fixed.** The RAM can't store a color, and the task loaded a profile that didn't exist. Armoury Crate taken out of the fight. Built on OpenRGB rather than a fork.
- 2026-09-25: **OpenRGB as a service** with the PawnIO driver, plus the first re-apply task.

## Owner setup completed

- OpenRGB 1.0.0 installed for the whole PC. Its service starts automatically and serves its control port on 127.0.0.1:6742. PawnIO 2.2.0.0 is installed.
- The OpenRGB profiles were copied to the Jason account on 8 November.
- Armoury Crate stopped on 23 October, and its lighting side uninstalled on 13 November.
- The repo has a protected `main` and squash-only merges, like its sibling apps.

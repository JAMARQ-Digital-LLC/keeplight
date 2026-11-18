# Keeplight: Windows builds for Atlas's Reinstall (Feature session brief)

Status: carded 2026-11-18 as Next Up item A in `kl-project-state.md`. Not started. Fleet: main-build, with Vaexbot C, Studio AE, Pulse B, Vaexil Mod Manager A and Modlist Launcher A.

**Title line:**

```
Keeplight: main-build.yml for Atlas's Reinstall on JASON-PC
```

**Paste as the first message:**

```
Add Keeplight's part of Atlas's "Reinstall on JASON-PC" (Atlas Next Up Z, SSH design phase 6, https://claude.ai/artifact/RZCV7mQTtCHf4nGnwbQt2J; Atlas brief docs/feature-briefs/reinstall-on-jason-pc.md, step 2), and make releases go out only when the owner asks. Read kl-project-state.md and that brief first. Vaexil Mod Manager A and Modlist Launcher A are the same task in the same Rust, Slint and Inno Setup shape.

Facts, checked 2026-11-18:
- The owner's goal (2026-11-18): cut how often releases go out, while still being able to update every Windows app from main. The owner's decision the same day: once an app has its main build, an official release goes out only when the owner asks for one (replacing the nightly rule of ADR-0026 as updated in agentic-instructions#459), and main builds become the normal way to update JASON-PC.
- release.yml today: on `push: tags: ['v*']`, on windows-latest, dtolnay/rust-toolchain@stable, Inno Setup from choco, `cargo test --workspace --locked`, `./scripts/package.ps1`, then `gh release create <tag> dist/*.zip dist/*.exe dist/SHA256SUMS.txt --draft`. No schedule. publish-feed.yml runs on `release: published` (the owner publishing the reviewed draft), refuses a tag that isn't v<MAJOR.MINOR.PATCH>, and copies `Keeplight-<version>-setup.exe` to keeplight-v<version> with keeplight-latest/release.json in jamarq-releases. So the feed already moves only on the owner's action.
- scripts/package.ps1 reads the version from `cargo metadata` (the workspace version in Cargo.toml, 0.2.0 on main 2ad814d), builds with `cargo build --release --locked -p keeplight -p keeplight-cli`, and passes it to ISCC as `/DAppVersion=`; packaging/installer.iss uses it for AppVersion and `OutputBaseFilename=Keeplight-{#AppVersion}-setup`. The installer is per user (PrivilegesRequired=lowest), stops a running keeplight-cli.exe and keeplight.exe before installing, and restarts the keeper when its switch is on. Because of that restart, a script that waits on the installer and its child processes hangs; wait on the installer process only (found 2026-11-14 on JASON-PC, when the PC session's reinstall script had to be changed). Say so to Atlas's Reinstall.
- Versions: the feed serves 0.2.0 (keeplight-latest, read 2026-11-18); JASON-PC had 0.2.0 per user on 2026-11-17 (Atlas's design). Releases bump the version in their own PR (#26 for 0.2.0), so main carries the last released version between releases.
- The updater is a notice only (crates/core/src/update.rs): `newer()` uses `installs::compare`, which splits on "." and reads each part as a number, anything else as 0. A +main stamp breaks it: "0.2.3+main.abc1234" reads as 0.2.0.0, so a build after 0.2.3 would be told 0.2.1 is newer (a downgrade). It only works today by luck when the patch is 0.
- JAMARQ-Digital-LLC/keeplight is a public repository, so a workflow artifact is downloadable by any signed-in GitHub user, not private as the Atlas design says. The source is public under GPL-3.0, so the exposure is small; say so in the PR. Standard GitHub-hosted runners are free for public repositories.

Build:
1. The updater first: compare only the part before "+" (in update::newer, or in a helper it uses; leave installs::compare's OpenRGB use alone), with tests: an installed X.Y.Z+main.<sha> is not told about X.Y.Z or anything older, is told about X.Y.(Z+1), and a feed naming a +main version stays refused by parse().
2. .github/workflows/main-build.yml: workflow_dispatch only, on main. The same build as the release, through the same package.ps1 (no forked copy), with the workspace version stamped `<version>+main.<sha7>` before packaging. `--locked` will refuse a changed workspace version until Cargo.lock matches, so refresh only the workspace entries offline (for example `cargo update --workspace --offline`) or pass the stamp another way; never unlock the dependencies. Check what Inno Setup writes for the file version and the uninstall entry's DisplayVersion with a "+" in AppVersion, and give VersionInfoVersion the numbers alone if it needs them. Upload `Keeplight-<version>+main.<sha7>-setup.exe` and SHA256SUMS.txt as an artifact kept 7 days, named so Atlas can find it by commit. No GitHub release and no feed.
3. release.yml: move it from the tag push to workflow_dispatch, which the owner starts; it tags main's head as v<workspace version> and makes the draft as today, refusing a version that already has a tag. publish-feed.yml stays on the owner publishing the draft. Rewrite both header comments to say releases happen only when the owner asks.
4. Read the stamped version back from the built installer and from `keeplight-cli version`.

Done when: no schedule or push trigger is left in release.yml, the feed moves only after the owner's dispatch and publish, a dispatch of main-build.yml on your branch produces the artifact with the stamped version read back, the updater tests cover the +main rule, and the PR is open. Never install on a real PC; the first real Reinstall is the owner's, from Atlas, which installs with /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /CURRENTUSER after backing up %LOCALAPPDATA%\Keeplight.

Follow the repo's verify (CONTRIBUTING.md: cargo fmt, clippy with -D warnings, tests, the release build and cargo deny, plus actionlint on the workflows), in one PR from your own worktree. Don't install on any machine by hand. Remove item A from `## Next Up` in kl-project-state.md and update the plan page https://claude.ai/artifact/PisWk9vuT3sDiHuyRxWktu before finishing.
```

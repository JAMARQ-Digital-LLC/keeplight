# Contributing

Bug reports and pull requests are welcome. Keeplight drives OpenRGB and never talks to hardware itself, so most of the work is in reading what OpenRGB reports carefully, showing it plainly, and changing only the lights you chose.

Keeplight is built for one PC (see the README), so a report about hardware it has not been tested on is useful, but may not be fixed quickly.

## Before you start

- For anything bigger than a fix, open an issue first so we can agree on the approach.
- Read the [safety model](docs/safety-model.md). Changes must keep its promises:
  - send OpenRGB only the requests the safety model lists, and update it in the same pull request as any new one;
  - change only lights the user has set, and never load, save or delete OpenRGB profiles;
  - write only what the safety model lists;
  - connect to nothing but the OpenRGB SDK server;
  - need no admin rights.
- Design decisions are recorded in [docs/adr](docs/adr). The SDK client in `crates/core/src/sdk` is Keeplight's own, written from OpenRGB's protocol description; do not add `openrgb2` or copy code from it, since it is GPL-2.0-only ([ADR 0001](docs/adr/0001-build-on-openrgb.md)).
- The look comes from the Vaexil theme in `crates/app/ui/theme.slint`, which is copied unchanged from Modlist Launcher ([ADR 0002](docs/adr/0002-rust-slint-vaexil-theme.md)). Use its components and `Theme` tokens rather than new colours, and do not edit that file here; change it in Modlist Launcher and copy it again.

## Working on it

You need the Rust toolchain (stable, MSVC) and Visual Studio Build Tools. OpenRGB is needed only to try a build on real lights.

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p keeplight
```

CI runs all of these on every pull request, plus `cargo deny check`. `cargo deny` bans HTTP and TLS crates, tokio, and the GPL-2.0-only OpenRGB crates, and rejects yanked or vulnerable ones. Both CI checks must pass before a pull request can merge. A pull request that changes only root Markdown, Markdown under `docs/` or `LICENSE` skips both checks, which then pass as skipped; a `.github/` change always runs them.

### Dev builds and scratch mode

A debug build, or any build from a git worktree, is the dev app (`crates/core/build.rs`, `crates/core/src/scratch.rs`). Only a release build from a full checkout, such as the installer's, is the release app. The dev app has its own data folder (`%LOCALAPPDATA%\Keeplight Dev`) and startup entry (`Keeplight Dev`), so its keeper's mutex and stop event never meet the installed keeper's. It never checks for updates or downloads an installer, and it refuses the real OpenRGB (port 6742, or another computer) unless `KEEPLIGHT_NETWORK=live` is set by name. It never opens `%LOCALAPPDATA%\Keeplight` or uses the startup entry `Keeplight`.

`KEEPLIGHT_SCRATCH=1` makes those refusals strict, for any build. `.cargo/config.toml` sets it for every `cargo test` and `cargo run`, and the smoke test sets it too. A scratch run refuses to start unless:

| Variable | In scratch |
| --- | --- |
| `KEEPLIGHT_DATA_DIR` | A full path to a throwaway folder, not under `%APPDATA%` or `%LOCALAPPDATA%` except the temp folder. `KEEPLIGHT_DATA` is its older name and still works. |
| `KEEPLIGHT_OPENRGB` | A fake OpenRGB on a loopback port the test owns, never 6742. `cargo run -p keeplight-core --features testing --example fake-openrgb` prints one. |
| `KEEPLIGHT_STARTUP_NAME` | A test name such as `Keeplight-smoke-1`, never `Keeplight` or `Keeplight Dev`. |
| `KEEPLIGHT_NETWORK` | Unset or `offline`; `live` is refused. |
| `KEEPLIGHT_SECRETS` | Unset or `memory`. Keeplight keeps no secrets; the name is the fleet's. |

To try a dev build on the real lights, by hand: `$env:KEEPLIGHT_SCRATCH = '0'; $env:KEEPLIGHT_NETWORK = 'live'; cargo run -p keeplight`. A test that needs the real thing is `#[ignore]` and named `live_*`. Every front end limits its data folder to the current user when it starts: one access entry, inherited, nothing from the parent folder.

- Keep each pull request to one change.
- Add tests for behaviour that can be tested. Platform-neutral logic belongs in `crates/core`, where it is unit tested; the SDK client is tested against a fake server.
- For UI changes:
  - check dark mode, and light mode once the setting exists;
  - check keyboard use: Tab, Space/Enter;
  - check that controls and device rows have screen-reader labels.
- Update the README or docs when behaviour changes.

## License

By contributing you agree that your work is licensed under [GPL-3.0-only](LICENSE), like the rest of the project.

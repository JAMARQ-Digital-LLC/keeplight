# Keeplight for agents

Keeplight drives real lights through OpenRGB. Read `kl-project-state.md` first, then CONTRIBUTING.md.

## Danger list

- On JASON-PC the installed Keeplight runs all the time (`keeplight.exe --keep`, and `keeplight-cli.exe mcp` for AI apps) and drives the owner's lights through OpenRGB on 127.0.0.1:6742. Never stop, signal or talk to it, never connect to port 6742 from a session, never write the HKCU Run value `Keeplight`, and never open `%LOCALAPPDATA%\Keeplight`.
- Never install a build from a session; Atlas's Reinstall is the owner's. Never start `release.yml`.
- Real-light tests only with the owner: one device at a time, to its current state, with a throwaway setup folder. Never load an OpenRGB profile.

## Scratch recipe

A debug or worktree build is the dev app (`Keeplight Dev`), which already refuses the real OpenRGB, folder and startup entry. Every test and smoke run is also scratch; `cargo test` gets `KEEPLIGHT_SCRATCH=1` from `.cargo/config.toml`. Anything run by hand sets:

```powershell
$env:KEEPLIGHT_SCRATCH = '1'
$env:KEEPLIGHT_NETWORK = 'offline'
$env:KEEPLIGHT_SECRETS = 'memory'
$env:KEEPLIGHT_DATA_DIR = "$env:TEMP\keeplight-<task>"         # under the temp folder
$env:KEEPLIGHT_OPENRGB = '127.0.0.1:<port>'                    # the fake: cargo run -p keeplight-core --features testing --example fake-openrgb
$env:KEEPLIGHT_STARTUP_NAME = 'Keeplight-<task>'               # never Keeplight or Keeplight Dev
```

`scripts/smoke-mouse-clicks.ps1` sets all of these and refuses an installed `keeplight.exe`. It needs the interactive desktop: over SSH, run it from a one-off `schtasks /IT` task and delete the task afterwards. A test that needs the real thing is `#[ignore]` and named `live_*`.

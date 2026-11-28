# Keeplight

Plan artifact: https://claude.ai/artifact/PisWk9vuT3sDiHuyRxWktu

Keeplight shows every light in the PC, lets you set each one with a click, and puts your colors back on its own after a restart, a sleep or a power cut.

It runs on top of [OpenRGB](https://openrgb.org), which does all the hardware work, and talks to it over OpenRGB's SDK port on `127.0.0.1`. See [ADR 0001](docs/adr/0001-build-on-openrgb.md).

Keeplight is an independent project. It is not affiliated with OpenRGB or any hardware vendor. It is built and tested on one PC: an ASUS TUF Gaming Z590-Plus WiFi with Mushkin Redline Lumina RAM, an EVGA RTX 3080 Ti FTW3 Ultra and a SteelSeries Apex Pro TKL keyboard. Other hardware works as far as OpenRGB supports it, but is untested.

## Status

v0.2.0 is released. Stages 02 to 05 are done: the keeper, one-click colors, the old stopgap retired, and OpenRGB installs and the update notice. Stage 06, the case map, is next. Keeplight can:

- **Lists every device OpenRGB reports**, by plain name ("RAM stick 2", "Graphics card"), with the product name, light count, current effect and zones.
- **Shows each device's current color** as a swatch, and each zone's when it has several, read back from the device. A device running a lighting mode of its own, such as the keyboard's Onboard mode, shows a spectrum instead, because OpenRGB can't see the colors another app or the device itself is setting.
- **Says what probably lights each device** ("Lit by: its own memory (Onboard)", "probably SteelSeries GG, which is running", "Keeplight, which keeps it"), from the mode OpenRGB reports and which lighting apps are running, and warns when another app is running that may change a device Keeplight keeps.
- **Says which OpenRGB and PawnIO you have** and whether Keeplight is tested with those versions, and whether the OpenRGB service runs. When either is missing or older, **Install…** downloads the tested version from its official site, checks it, and runs its own installer, after Windows asks for admin rights.
- **Flags devices that cannot store a color themselves**, which go back to their own lighting whenever they lose power.
- **Color…** on a device opens nine common colors and a `#rrggbb` field. Picking one sets the device to it and keeps it. A device running an effect, like the RAM's Chase Fade, keeps the effect in the new color; one on its own lighting switches to Static, or Direct. **Keep it as before** then goes back to the color it had. A device with several zones, such as the motherboard's headers, shows a swatch per zone, and the picker can color one zone on its own.
- **Keep this** saves what a device shows now into Keeplight's own setup file, for a color set somewhere else, such as OpenRGB.
- **Put back** is on every kept device, highlighted when it shows something else, and sends that one device its kept mode and colors. After a power cut the lights can differ while OpenRGB's record still matches, so it's always there.

- **A newer Keeplight** is announced in the window, with a link to its installer, when one is published (checked when the window opens and once a day; **Tell me when a new Keeplight is out** turns it off). Keeplight never updates itself.
- **Put kept lights back when you sign in** starts the keeper, `keeplight --keep`, now and at every sign-in. It has no window. It puts every kept device back when it starts, after the PC wakes, when OpenRGB's device list changes and when OpenRGB comes back after a restart, and logs each run to `%LOCALAPPDATA%\Keeplight\keeper.log`. Off until you turn it on; turning it off stops the keeper.

- **A notification** tells you when the keeper couldn't put a kept device back, once per new problem. **Show the keeper's icon in the notification area** (off by default) keeps an icon there whose menu can open Keeplight or put kept lights back now.

- **Keep it as before** brings back what a device was kept as before you stopped keeping it or gave it a new color, and puts it back.
- **Stop keeping** removes a device from the setup (remembering it for Keep it as before) and, if it's in Direct mode and has a built-in "Onboard" mode, hands it back to that, so it isn't left dark.
- **Keep it dark…** on a device whose lights are all off asks once before keeping it that way.

Colors work per device, per zone (the zone picker above) and for everything at once (**Set everything…** in the window header). See the plan page for what comes next.

## AI tools and the command line

`keeplight-cli.exe`, installed beside the app, is Keeplight's command line (`keeplight-cli devices`, `plan`, `apply`, `keep`, `put-back`, `stop-keeping`, `keep-as-before`, `set-color` (or `set-color all`), `logs`, `diagnose`, `error-codes`, with `--json`) and its MCP server for AI apps such as Claude Code, Gemini CLI and Codex CLI (`keeplight-cli mcp`). AI tools can read and plan; they change nothing unless you turn on "Let AI tools make changes" in the window, and then only what you ask for, planned first and journaled. See [docs/ai.md](docs/ai.md).

## Build from source

You need the Rust toolchain (stable, MSVC), Visual Studio Build Tools, and OpenRGB running with its SDK server on (the OpenRGB service has it on).

```powershell
cargo test --workspace
cargo run -p keeplight
```

A build from source is the dev app, apart from a release build from a full checkout (the installer's): it keeps its files in `%LOCALAPPDATA%\Keeplight Dev`, uses the startup entry `Keeplight Dev`, never checks for updates or downloads, and refuses the real OpenRGB on 127.0.0.1:6742 unless `KEEPLIGHT_NETWORK=live` is set. `cargo test` and `cargo run` also start in scratch mode (`.cargo/config.toml`), which needs a fake OpenRGB and a throwaway folder; see [CONTRIBUTING.md](CONTRIBUTING.md#dev-builds-and-scratch-mode).

`cargo run -p keeplight-cli -- devices` runs the command line from source. `cargo run -p keeplight-core --features testing --example fake-openrgb` starts a fake SDK server with two RAM sticks and a keyboard and prints its address. `scripts/smoke-mouse-clicks.ps1` clicks the real window's buttons against it (CI runs it too).

`scripts/package.ps1` builds the portable zip and, when Inno Setup 6 is installed, the per-user installer into `dist/`. `scripts/generate-icon.ps1` redraws the icon in `assets/`.

Releases go out only when the owner asks: the Release workflow (`.github/workflows/release.yml`), started by hand on `main`, tags its head `v<version>` and builds those into a draft GitHub release, refusing a version already tagged. Publishing the draft also copies the installer to [jamarq-releases](https://github.com/JAMARQ-Digital-LLC/jamarq-releases) as `keeplight-<version>-x64-setup.exe`, where `keeplight-latest/release.json` names the newest one and its SHA-256 (`.github/workflows/publish-feed.yml`).

Between releases, the Main build workflow (`.github/workflows/main-build.yml`, started by hand) builds the same installer from a main commit, stamped `<version>+main.<sha7>`, as a workflow artifact named `keeplight-main-<commit>` kept for 7 days. Nothing is published; it is how Atlas reinstalls Keeplight on the owner's PC. A main build's update notice offers only a release newer than the version it carries.

## Safety

[docs/safety-model.md](docs/safety-model.md) lists every request Keeplight sends to OpenRGB and every file it writes. It changes a light only on a device you kept, colored or stopped keeping, or for every kept device when the keeper runs or you choose the keeper icon's **Put kept lights back now**; the same changes come from the command line, and from AI tools only with **Let AI tools make changes** on, planned first and journaled. The safety model is the full list.

## License

GPL-3.0-only. The embedded Geist fonts are under the SIL Open Font License 1.1 (`assets/fonts/OFL.txt`).

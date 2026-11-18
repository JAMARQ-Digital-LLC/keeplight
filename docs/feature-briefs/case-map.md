# Keeplight: stage 06, the case map (Feature session brief)

Status: carded 2026-11-18 as Next Up item B in `kl-project-state.md`. Waits on the owner: picking it as the stage after 05 (it is the plan page's recommendation, 2026-11-14; the owner has not chosen). Not started.

**Title line:**

```
Keeplight: stage 06, the case map
```

**Paste as the first message:**

```
Build Keeplight's stage 06, the case map. Read kl-project-state.md, docs/safety-model.md and the plan page (https://claude.ai/artifact/PisWk9vuT3sDiHuyRxWktu, Roadmap tab) first.

Facts, checked 2026-11-18:
- Stages 00 to 05 are done; v0.2.0 is the published release (2026-11-14). JASON-PC runs main 20cea42 (0.2.0 plus kit v7), installed 2026-11-14.
- The idea (plan page): the owner photographs the open case once; Keeplight blinks one light at a time and the owner drops a pin where it lit; the same pass counts the real LEDs on each RGB header, replacing today's placeholder of 40 + 40 on the motherboard's two headers; the keyboard draws itself from its own key layout; afterwards a click on the photo sets that light. About 20 minutes of the owner at the PC, once.
- Devices on JASON-PC (OpenRGB, 2026-11-13/14): two Mushkin ENE RAM sticks (8 LEDs each, SMBus 0x71 and 0x73, kept as red Chase Fade), EVGA RTX 3080 Ti FTW3 Ultra (4 zones), ASUS TUF Z590-Plus WiFi (3 zones: Aura Mainboard 4, header 1 40, header 2 40), SteelSeries Apex Pro TKL Gen 3 (112 keys). 216 LEDs counted with the 40 + 40 guess.
- Rules that apply (state file): one device at a time, put back to its current state, throwaway setup file for tests; never SAVEMODE or a profile; a header count set too high does no harm, too low leaves LEDs unlit. The owner prefers every option visible and easy to click (2026-11-13).

Plan first: propose to the owner how the photo is stored (local data folder only), how the blink pass works device by device and how the counted header sizes are saved (OpenRGB's resize-zone request or Keeplight's own record), and how the map is shown. Get the owner's OK, then build in PRs from your own worktree. The pinning pass on real hardware needs the owner at the PC.

Done when: the owner has mapped JASON-PC once, the header counts are real, clicking a mapped light sets it, item B is removed from `## Next Up`, and the state file and plan page are updated.
```

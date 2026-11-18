# Keeplight: stage 07, effects (Feature session brief)

Status: carded 2026-11-18 as Next Up item E in `kl-project-state.md`. Waits on the owner: picking it. The plan has effects after the case map ("solid colors first", settled 2026-10-24). Not started.

**Title line:**

```
Keeplight: stage 07, effects kept like colors
```

**Paste as the first message:**

```
Build Keeplight's stage 07: choose a device's own hardware effects (breathing, rainbow and the like) from the window, and keep them through restarts, sleep and power loss the way colors are kept. Read kl-project-state.md, docs/safety-model.md and the plan page (https://claude.ai/artifact/PisWk9vuT3sDiHuyRxWktu) first.

Facts, checked 2026-11-18:
- Keeplight already keeps a device's whole mode, not only its colors: `device::Mode` carries every field so modes go back unchanged, and `fit` compares speed, brightness, direction and color mode. A new color keeps the device's effect when its mode takes colors (owner, 2026-11-13: red Chase Fade becomes blue Chase Fade), else Static, then Direct.
- The RAM sticks (Mushkin ENE) are kept as red Chase Fade today and report color mode per-LED. ENE RAM falls back to rainbow on power loss without OpenRGB noticing, so the keeper re-applies on wake regardless.
- Rules: send a device only its own modes; never SAVEMODE or a profile; real-light tests one device at a time, put back to its current state, with a throwaway setup file. AI tools and the command line go through the same engine (`Engine::change_now` / plan and apply), behind `aiChanges` for AI.

Plan first: show the owner which modes each real device offers (from get_devices) and how picking one, with its speed, brightness and direction, would look in the window, and get the owner's OK. Then build in PRs from your own worktree. Done when: the owner has picked and kept an effect on a real device and it came back after a restart, item E is removed from `## Next Up`, and the state file and plan page are updated.
```

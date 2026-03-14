# Build Notes — Tutorial Quest System Web Port

**Date:** 2026-03-14
**Task:** Port TUI quest/tutorial system to web UI

## What was done

### New file: `crates/groundwork-web/src/ui/quests.ts`
- Ported all quest definitions from `groundwork-tui/src/quest.rs` to TypeScript
- 17 quests across 8 chapters (reduced from 20/9 — removed TUI-specific quests)
- Pure TypeScript action tracker (no WASM bridge changes needed)
- Completion checks run against the voxel grid directly via zero-copy typed array
- HTML/CSS quest panel rendered as DOM overlay (top-left, below status)
- Notification toast system (top center, fades after ~3 seconds)
- Panel collapse/expand via click or M key

### Adaptations from TUI
- **Removed "Switch to 3D"** — web is already 3D; replaced with "Orbit the camera" (drag)
- **Removed "Open Inspect" / "Inspect Soil"** — no inspect panel in web yet
- **Removed "Use range tool"** — no range tool in web yet
- **Merged chapters 4-5 (Read the Land + Going Underground)** into a single "Going Underground" chapter
- **Control hints updated**: 1-5 for tools, Q/E for depth/species, Space for auto-tick, T for manual tick
- **Focus = last clicked voxel** (TUI uses cursor position; web uses click target)

### Integration points
- `main.ts`: Quest log created alongside HUD, action recording on pan/orbit/depth/tick keys, quest checks after auto-tick and tool placement
- `controls.ts`: Accepts `QuestLog`, records species cycling on Q/E
- Tool placement in `main.ts onToolPlaced` callback records tool use + click position, then checks quests

## What's NOT done yet
- **Inspect panel quests** — need inspect panel first (hover tooltip or panel showing material/water/light/nutrients)
- **Range tool quest** — need range tool UI in web
- **Quest persistence** — quest state resets on page reload (no save/load yet)
- **Grid scanning optimization** — trunk/tree checks scan full grid; could optimize with counters on the Rust side

## Testing notes
- TypeScript compiles clean (no new errors beyond pre-existing missing `three` types)
- Vite transforms all 27 modules including new quests.ts
- WASM not built in this environment so full runtime test not possible; quest logic is self-contained TypeScript

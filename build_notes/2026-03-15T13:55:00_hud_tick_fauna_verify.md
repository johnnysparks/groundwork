# Build Notes: HUD Tick Counter + Fauna Pipeline Verification
**Date:** 2026-03-15T13:55:00
**Dev:** Claude (dev role, sprint 1)

## What Was Built

### SIM-03: HUD tick counter wiring
Added `getTick` import to `main.ts` and wired `hud.setTickCount()` to all four tick paths:
1. Initial warmup (after WASM loads)
2. Manual tick (`T` key)
3. Auto-tick loop (spacebar)
4. AgentAPI tick (Playwright/headless)

Also added `setTickCount` callback to the `AgentAPIConfig` interface so headless ticks update the HUD.

### SIM-02: Fauna rendering pipeline verification
The entire fauna pipeline was already implemented but never verified visually:
- Sim: `fauna_spawn` runs every 20 ticks, spawns bees/butterflies/birds/worms/beetles
- WASM: `fauna_ptr()`, `fauna_len()`, `fauna_count()` exports exist
- Bridge: `getFaunaCount()`, `getFaunaView()`, `readFauna()` all implemented
- Renderer: `FaunaRenderer` with billboard shaders, wing flutter, type-specific shapes/colors

**Verified:** After 200 ticks with flowers planted, `fauna=5` creatures spawn. Billboard sprites are visible in close-up screenshots near canopy areas.

### Screenshot script improvements
- Now ticks sim 200 times before capturing (was 0 — only saw initial state)
- Plants wildflowers, daisies, clover, moss near center for fauna spawning
- Reports `tick=N, fauna=N` count during capture
- Added `getFaunaCount()` to agentAPI

## Files Changed
- `crates/groundwork-web/src/main.ts` — getTick import, setTickCount calls at all tick sites
- `crates/groundwork-web/src/agent-api.ts` — setTickCount config, getFaunaCount export
- `crates/groundwork-web/screenshot.sh` — sim advancement, flower planting, fauna count logging

## Test Results
- Workspace compiles clean
- Screenshot captures show tick=205, fauna=5
- HUD displays correct tick count

## What's Next
- SIM-01: Nitrogen handshake (clover → oak growth boost)
- Fauna sprites may need larger sizes to be more visible at default zoom
- Consider adding a fauna count to the HUD status line

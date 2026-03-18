# Build Notes — Restore Starter Garden (Sprint 339)

**Date:** 2026-03-18T13:00:00
**Builder:** Dev agent (Claude)
**Branch:** main

## Sprint 339 — Restore starter garden for first-load experience

### Problem
After the WASM health fix (Sprint 338), playtest revealed the default garden is completely empty. `create_world_with_garden()` was stripped of starter seeds, leaving new players with bare terrain + pond + gnome. The "alive garden" fantasy was absent on first load.

### Fix
Added 17 starter seeds to `create_world_with_garden()` in `lib.rs`:
- **9 groundcover** (3 moss, 3 grass, 3 clover) at y=20-26 near pond edge
- **4 flowers** (2 wildflower, 2 daisy) among groundcover at y=22-27
- **4 trees** (2 oak, 2 birch) set back from pond at y=28-34

Seeds are placed after 50 ticks of water flow (so soil is moist), then 200 additional ticks run to grow the garden. This gives seedlings time to germinate, groundcover time to spread, and trees to reach sapling/young stage with visible leaves.

### Key Details
- Seeds placed at surface_height + 1 (air cell above soil)
- Each seed registered in SeedSpeciesMap with correct species_id
- Total pre-tick: 250 (50 water flow + 200 growth)
- Tick counter reset to 0 after pre-simulation

### Verified
- All 116 Rust tests pass (111 sim + 5 integration)
- TypeScript compiles cleanly
- WASM rebuilt and deployed to public/wasm/
- Hero screenshot shows: trees with canopy, green groundcover, rain, active garden

### Impact
First-load experience now shows a living garden with visible plants, trees, and groundcover instead of empty terrain.

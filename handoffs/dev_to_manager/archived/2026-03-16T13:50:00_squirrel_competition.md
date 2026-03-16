# Dev → Manager Handoff: Squirrel + Competition Tuning
**Date:** 2026-03-16T13:50:00
**Status:** 13 sprints complete in simulation enhancement workstream

## What Shipped

### Squirrel Fauna (FaunaType::Squirrel = 5)
- Spawns near oaks/berry bushes (10+ leaf voxels of species 0 or 5)
- Erratic scurry movement with dart-and-pause behavior
- Caches oak acorns at random locations → "gift" oak seedlings appear
- From CLAUDE.md spec: "spawn near oak/berry, cache acorns that sprout"

### Competition Balance Fix (Critical)
- Reduced partial health recovery from +0.005 to +0.002 per tick
- Previously shade penalty (~0.006) was nearly cancelled by recovery (+0.005), net = -0.001/tick
- Now net = -0.004/tick → trees die from shade in ~250 ticks (was ~1000)
- This makes crowded tree clusters actually thin over time

### Acceptance Test: crowded_oak_cluster_thins_naturally
- 10 oaks in a 3×4 grid, 3 voxels apart, with limited water
- After 400 ticks: at least 2 die from competition
- Validates the critical sim review's core requirement

## Important Note for Visual Style Team
The partial recovery reduction means stressed trees will show yellow/brown foliage more often. This is intentional — it creates visible competition. The health data on leaf voxels (water_level byte) should now show more variation, making the stress coloring system more useful.

## Final Stats
- **18 interaction chains** (added squirrel acorn caching)
- **6 fauna types**: bee, butterfly, bird, worm, beetle, squirrel
- **98 tests** (93 unit + 5 integration)

# Build Notes: Fauna Surface Height Sweep (Sprint 347)

**Date:** 2026-03-19T09:30:00
**Sprint:** 347
**Commit:** 6606e13

## What Changed

### fauna.rs — fauna_spawn
- All sample points now compute `surface = VoxelGrid::surface_height(sx, sy)` instead of using fixed `GROUND_LEVEL`
- Pollinator flower detection: scans `surface..=(surface+3)` instead of `GROUND_LEVEL..=(GROUND_LEVEL+3)`
- Worm underground detection: uses `surface.saturating_sub(3)` instead of `GROUND_LEVEL.saturating_sub(3)`
- Squirrel oak/berry detection: scans `surface..=(surface+6)` instead of `GROUND_LEVEL..=(GROUND_LEVEL+6)`
- Squirrel spawn height: `surface + 1.5` instead of `GROUND_LEVEL + 1.5`

### fauna.rs — fauna_update (movement)
- Pollinators: floor clamp uses `surface_height(f.x, f.y) + 1` instead of `GROUND_LEVEL + 1`
- Birds: swoop floor uses `surface_height + 3` instead of `GROUND_LEVEL + 3`
- Worms: z clamp uses `surface_height - 0.5` (max) and `surface_height - 8.0` (min)
- Beetles: floor clamp uses `surface_height` instead of `GROUND_LEVEL`
- Squirrels: all 5 GROUND_LEVEL references in movement (bobbing, acting, idle, clamp) now use per-position surface_height

### fauna_integration.rs — test fix
- `fauna_positions_are_valid` test now checks worm/pollinator z against `surface_height` at their position instead of fixed `GROUND_LEVEL`

## Impact

- Pollinators can now detect flowers growing on elevated terrain (the slope)
- Squirrels can find oaks/berry bushes on slopes for acorn caching
- All fauna move at correct heights relative to sloped terrain
- Worms stay underground even on elevated portions of the map
- Combined with Sprint 346 (weather/deadwood surface fix), the entire sim now uses surface_height consistently

## GROUND_LEVEL Audit
The `GROUND_LEVEL` import was removed from fauna.rs entirely — zero remaining hardcoded GROUND_LEVEL references in fauna behavior. gnome.rs already uses `surface_z()` which calls `surface_height()`.

## Test Results
- All 116 tests pass (111 unit + 5 integration), no regressions

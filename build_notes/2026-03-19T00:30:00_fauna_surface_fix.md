# Build Notes: Fauna Surface Height Fix + Ecological Audit

**Date:** 2026-03-19T00:30:00
**Sprint:** 345
**Commit:** 80e249b

## What Changed

### Fauna Surface Height Fix (fauna.rs)
Same bug as pioneer succession (Sprint 341): bird seed-dropping, squirrel acorn caching, and bird droppings used fixed `GROUND_LEVEL + 1` instead of `VoxelGrid::surface_height()`. Seeds landed underground on elevated terrain.

Fixed in three locations:
1. Bird seed-drop: `let tz = VoxelGrid::surface_height(tx, ty) + 1`
2. Bird droppings: `let ground_z = VoxelGrid::surface_height(cx, cy)`
3. Squirrel acorn: `let tz = VoxelGrid::surface_height(cx, cy) + 1`

### Ecological Interactions Audit
Audited all 6 canonical interactions:
- Nitrogen Handshake: Working (1.5× boost with 3+ groundcover)
- Pollinator Bridge: Working (+0.005 health/tick per pollinator)
- Root War: Working (water divided by competitor count)
- Bird Express: Working (5% chance to drop species-specific seed 10-20 voxels away) — was incorrectly flagged as missing by initial audit
- Pioneer Succession: Working (fixed in Sprint 341)
- Canopy Effect: Working (1.5× boost for shade-tolerant species in moderate shade)

All 6 interactions are implemented and functional. The surface_height bug was silently breaking bird/squirrel seed placement on sloped terrain.

## Test Results
- All 116 tests pass, no regressions

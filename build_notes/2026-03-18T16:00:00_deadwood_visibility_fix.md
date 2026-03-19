# Build Notes: Deadwood Visibility Fix

**Date:** 2026-03-18T16:00:00
**Sprint:** 343
**Commit:** 5bb6765

## What Changed

### Deadwood Visibility Fix (systems.rs)
Living trees' Leaf/Branch voxels could overwrite DeadWood material in two code paths:
1. `tree_grow_visual` — gradual canopy growth (pending_voxels drain)
2. `tree_rasterize` — template path for Seedling/Sapling/Dead stages

Removed `Material::DeadWood` from the `can_place` match arm for Leaf/Branch in both locations. Now dead wood persists visually even when a nearby living tree's canopy grows into the same space.

### Pine Starter Seed Moved (lib.rs)
Moved pine from (40,14) → (40,8) to escape the pond area. Pine thrives at (40,8) with full sun — no dead tree drama in the starter garden. Dead trees will appear naturally during gameplay when competition, drought, or shade kills a tree. The visibility fix ensures they'll be visible when that happens.

## Test Results
- 116 tests pass (111 unit + 5 integration, 1 ignored)
- No regressions

## Current State
- Starter garden: ~49K leaves, 9 trees, 15 fauna, all thriving
- No dead trees in starter garden (pine thrives)
- DeadWood visibility fix is forward-looking — dead trees will be visible during gameplay
- WASM rebuilt and synced

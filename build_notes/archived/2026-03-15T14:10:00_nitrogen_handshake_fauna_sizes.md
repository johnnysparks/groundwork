# Build Notes: Nitrogen Handshake + Fauna Sprite Sizes
**Date:** 2026-03-15T14:10:00
**Dev:** Claude (dev role, sprint 2)

## What Was Built

### SIM-01: Nitrogen handshake (clover/groundcover → tree boost)
Added species interaction to `tree_growth` in `systems.rs`:
- Trees (PlantType::Tree) scan for Leaf voxels at ground level (z = GROUND_LEVEL to +2) within 5 voxels of their root zone
- If >= 3 groundcover leaf voxels found, nutrient accumulation gets 1.5x multiplier
- Applies to both `accumulated_water` and `accumulated_light`
- Works for any groundcover near any tree (clover, moss, grass all count)

Regression test: `nitrogen_handshake_clover_boosts_oak` — plants two oaks (one near clover, one isolated), ticks 200, asserts clover-adjacent oak has more accumulated resources.

### Fauna sprite sizes increased 3x
Bee: 0.5→1.5, Butterfly: 0.8→2.0, Bird: 1.2→3.0, Worm: 0.4→1.0, Beetle: 0.45→1.2

## Test Results
- 88 tests pass (83 unit + 5 integration), 0 failures
- New test `nitrogen_handshake_clover_boosts_oak` passes
- WASM rebuilt, screenshots captured

## Files Changed
- `crates/groundwork-sim/src/systems.rs` — PlantType import, nitrogen boost in tree_growth, regression test
- `crates/groundwork-web/src/rendering/fauna.ts` — sprite sizes 3x

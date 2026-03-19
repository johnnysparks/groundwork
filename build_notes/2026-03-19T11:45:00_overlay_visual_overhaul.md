# Build Notes: Overlay Visual Overhaul (Sprint 349)

**Date:** 2026-03-19T11:45:00
**Sprint:** 349

## What Changed

### overlay.ts — Data accuracy fixes
- Water/Nutrient overlays now scan for Soil/Water material cells specifically instead of reading any non-Air cell. Leaf/Trunk voxels store species_id in nutrient_level byte, not actual nutrient data — reading those produced garbage values.
- Light overlay reads any surface cell (light data is valid everywhere).
- Removed zero-value skip (`if (dataValue === 0) continue`) so dry/dark/depleted areas are visible.

### overlay.ts — Color ramp overhaul
Previous ramps were too similar — all converged to warm amber mid-tones indistinguishable from terrain.

New ramps with maximally separated palettes:
- **Water**: pure red (dry) → pure blue (wet) — classic thermal palette
- **Light**: deep violet (shade) → vivid yellow (bright) — dramatic shade map
- **Nutrient**: dark charcoal (depleted) → vivid emerald green (rich) — fertility map

### overlay.ts — Rendering fixes
- Opacity boosted from 0.6 → 0.85 so overlays read through post-processing
- depthTest disabled so overlays render on top of foliage
- renderOrder set to 100 for consistent layering

## Impact
- All three surface overlays are now visually distinct from each other AND from terrain
- Water overlay clearly shows pond/spring moisture vs dry areas
- Light overlay reveals canopy shadow patterns as purple zones
- Nutrient overlay shows fertility gradient from charcoal to green

## Test Results
- TypeScript compiles cleanly
- Overlay screenshots captured and verified

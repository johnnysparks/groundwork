# Build Notes: Surface Height Sweep

**Date:** 2026-03-19T01:00:00
**Sprint:** 346
**Commit:** 8e54331

## What Changed

### Weather System (systems.rs)
- Rain now falls at `surface_height(rx, ry) + 1` instead of `GROUND_LEVEL + 1`
- Soil moistening uses `surface_height()` instead of fixed `GROUND_LEVEL`
- Drought evaporation and shallow soil drying use per-column surface height

### Deadwood Decay (systems.rs)
- Underground/above-ground classification now uses `surface_height(x, y)` per cell
- Dead wood on slopes correctly identified as underground or above-ground

### Nitrogen Boost (systems.rs)
- Groundcover detection scans z range `GROUND_LEVEL..=GROUND_LEVEL+7` (was +2)
- Covers full slope variation (surface can be GROUND_LEVEL to GROUND_LEVEL+5 plus 2 above)

## Impact — DRAMATIC

The rain fix alone transformed the ecosystem. Plants on elevated terrain now get rain:

| Metric | Before Fix | After Fix |
|--------|-----------|-----------|
| Tree entities | 10 | 26 |
| Leaf voxels | 82K | 139K |
| Fauna count | 14 | 29 |
| Dead/dying leaves | 13K | 5.7K |
| Stressed leaves | 0 | 52K |
| Stage 5 (Dead) entities | 0 | 6+ |

The garden went from "9 trees near the pond" to "26 plants across the full terrain with active competition." Multiple dead moss entities, a dying oak, shade-stressed birch. This is real ecological drama across the entire terrain, not just the pond area.

## Key Insight
The `GROUND_LEVEL` hardcoding was the most impactful systematic bug in the sim. Rain was ONLY landing on flat terrain — the entire sloped portion (majority of the map) was in permanent drought except for water flow from the pond. Fixing rain distribution unlocked the full ecosystem potential.

## Test Results
- All 116 tests pass, no regressions

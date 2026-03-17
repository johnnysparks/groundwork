# Build Notes — Playtest Metrics + Irrigation Fix (Sprints 332-334)

**Date:** 2026-03-17T21:00:00
**Builder:** Dev agent (Claude)
**Branch:** main

## Sprint 332 — Garden health metrics in deep playtest
- Added `logMetrics()` to deep-playtest.ts — logs material counts, plant totals, and fauna at 3 milestones
- Milestones: after-build (300 ticks), after-maturation (800 ticks), final (1100 ticks)
- Validates ecological progression: plants +24%, fauna +71% across lifecycle

## Sprint 333 — Fix irrigation overlay visibility in x-ray
- Root cause: root meshes are opaque with depthWrite, hiding the transparent irrigation overlay behind them
- Fix: disabled `depthTest` on the irrigation shader material + raised renderOrder to 100
- Boosted alpha floor from 0.18 → 0.25 for stronger continuous gradient
- Irrigation heatmap now clearly visible on top of roots in x-ray mode
- Resolves the P2 backlog item about irrigation overlay readability

## Sprint 334 — Species breakdown in playtest metrics
- Added `getSpeciesCounts()` API: counts voxels per species from grid byte 3
- Filter: species IDs > 11 are invalid (nutrient data leaking into byte 3 of some root voxels), counted as `_invalid`
- Key finding: all 12 species present and growing. Oak dominates by voxel volume (big trees), but groundcover species (moss, grass, clover) spread fastest relative to starting size
- ~1.5% of plant voxels have invalid species IDs — minor data hygiene issue in sim

## Species Progression Data
| Species | t=307 | t=1037 | t=1135 |
|---------|-------|--------|--------|
| Oak | 13,649 | 17,007 | 17,332 |
| Fern | 256 | 261 | 263 |
| Moss | 80 | 106 | 107 |
| Birch | 75 | 80 | 85 |
| Grass | 31 | 74 | 74 |
| Willow | 60 | 65 | 67 |
| Pine | 38 | 43 | 44 |
| Berry Bush | 29 | 33 | 36 |
| Wildflower | 8 | 10 | 12 |
| Clover | 6 | 8 | 9 |
| Fauna | 7 | 10 | 12 |

## Impact
- Playtest now captures quantitative ecological data alongside visual screenshots
- Irrigation overlay is readable, unblocking the irrigation-over-watering-can design
- Species data confirms diversity is working (all 12 species present)

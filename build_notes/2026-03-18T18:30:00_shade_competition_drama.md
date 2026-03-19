# Build Notes: Shade Competition Drama

**Date:** 2026-03-18T18:30:00
**Sprint:** 344 (continued)
**Commits:** 1e1895f, 19e82ea

## What Changed

### Sprint 344: Shade Competition System
Two changes make ecological competition visible:

1. **Lateral shade in light_propagation**: Air/seed cells under canopy shadow spread 20% shade to cardinal neighbors. Only cells with light < 180 spread (requires at least one leaf layer above). Creates penumbra around tall tree canopies.

2. **Average light health threshold**: Replaced broken raw-sum check with per-voxel average. Old system: 50 voxels × 200 light = 10,000, always exceeding threshold 10-50. New system: avg_light vs threshold range 15-100 scaled by shade_tolerance.

3. **650 pre-ticks** (was 300): Extra time for shade competition to produce visible stress. Pine at (40,8) reaches 8.6% health on first load — visibly dying.

## Results at First Load
- Pine (species=3): health=22/255 (8.6%) — dying from shade stress
- Moss at one position: health=160/255 (63%) — mild stress
- 13,195 leaves in "dying" category (red/brown stress tinting)
- 69,125 leaves still thriving (green canopy)
- Birch, oak, wildflower: all healthy at 255
- 14 fauna active
- 0 deadwood (pine hasn't died yet — player watches it happen)

## Key Technical Insight
The old light_ok check was **never a factor in tree health**. It compared raw sum of light across all voxels (typically 5,000-10,000) against thresholds of 10-50 — always true. Shade competition was broken by design since day 1. This fix makes light the primary driver of ecological drama.

## Test Results
- All 116 tests pass, no regressions

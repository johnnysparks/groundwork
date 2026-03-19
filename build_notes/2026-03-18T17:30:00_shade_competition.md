# Build Notes: Shade Competition

**Date:** 2026-03-18T17:30:00
**Sprint:** 344
**Commit:** 1e1895f

## What Changed

### Lateral Shade in light_propagation (systems.rs)
After the vertical top-down light pass, a second pass spreads shade laterally:
- Only Air/Seed cells under canopy shadow (light < 180) spread shade
- Spreads 20% of shade depth to 4 cardinal neighbors
- Only darkens neighbors that are significantly brighter (prevents over-darkening)
- Creates realistic penumbra zones around tall tree canopies

### Average Light Threshold for Health (systems.rs)
Replaced raw sum-based light_ok check with per-voxel average:
- Old: `light_intake` (sum across all voxels) vs threshold 10-50 → always true
- New: `avg_light` (average per above-ground voxel) vs threshold 15-100
- Sun-loving species (pine, shade_tolerance=180) need avg_light > ~85
- Shade-tolerant species (moss, shade_tolerance=20) need avg_light > ~22

### Canopy Boost Updated
Shade-tolerant species' growth boost now checks `avg_light` in range 30-100 (was raw sum 5-30).

## Results
- Pine at (40,8): health dropped to 143/255 (56%) — shade stress visible
- Birch: mild stress at 247/255
- 18,794 leaves in "ok" category (was 0 — all were "thriving")
- 57,820 leaves still thriving
- 0 deadwood yet — but pine is on a decline trajectory

## Test Results
- All 116 tests pass, no regressions
- WASM rebuilt and synced

## Key Insight
The old light threshold system was **broken by design**: it compared a raw sum (typically 5,000-10,000) against thresholds of 10-50. Light was never a factor in tree health. This fix makes shade competition the primary driver of ecological drama.

# Build Notes — Sprints 290-296: Material & Atmosphere Polish

**Date:** 2026-03-18T12:00:00
**Sprints:** 290-296
**Status:** Shipped

## What Changed

### Sprint 290: Bark Texture Variation
**`crates/groundwork-web/src/rendering/terrain.ts`**
- Trunk and branch voxels get ±8% per-voxel brightness noise
- Makes each tree feel unique even within the same species

### Sprint 291: Leaf Voxel Color Noise
**`crates/groundwork-web/src/rendering/terrain.ts`**
- Leaf voxels in terrain mesh get ±6% per-voxel noise
- Creates dappled canopy look (complements foliage billboard variation)

### Sprint 292: Time-of-Day Fog Density
**`crates/groundwork-web/src/main.ts`**
- Fog thicker at dawn (0.0035) and dusk, clearer at midday (0.0018)
- Rain makes fog thicker (min 0.0035)
- Smooth transitions with 0.3× lerp rate

### Sprint 293: Cool Blue Cloud Shadows
**`crates/groundwork-web/src/rendering/cloudshadow.ts`**
- Shadow color shifted from pure black to cool blue (0.02, 0.04, 0.10)
- More painterly sun/shade contrast

### Sprint 294: Post-Rain Petrichor Glow
**`crates/groundwork-web/src/main.ts`**
- 15-second bloom boost after rain stops
- Fades over last 5 seconds (+0.12 bloom max)
- World feels fresh and clean post-rain

### Sprint 295: Post-Rain Mist Boost
**`crates/groundwork-web/src/rendering/mist.ts`**
- `setRainBoost()` adds 0.8 density after rain
- Decays over 60 seconds
- Next dawn after rain is extra misty

### Sprint 296: Stone Mineral Variety
**`crates/groundwork-web/src/rendering/terrain.ts`**
- Stone voxels tint per-voxel: warm sandstone, neutral gray, cool blue-gray
- 1/3 distribution based on voxelNoise hash
- Breaks visual monotony of underground stone layers

## Test Results
- TypeScript type-check clean (all 7 sprints)
- `cargo check --workspace` passes
- No sim test regressions (JS-only changes)

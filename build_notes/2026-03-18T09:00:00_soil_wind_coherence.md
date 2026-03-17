# Build Notes — Sprints 265-272: Soil Nutrients, Wind Coherence

**Date:** 2026-03-18T09:00:00
**Sprints:** 265-272
**Status:** Shipped

## What Changed

### Sprint 265: Rainbow Discovery Message
**`crates/groundwork-web/src/main.ts`**
- First post-rain rainbow triggers HUD event "A rainbow arcs across the sky!"
- One-time notification with `_firstRainbowNotified` flag
- Resets on New Garden

### Sprint 266: Nutrient-Rich Soil Warm Tint
**`crates/groundwork-web/src/mesher/greedy.ts`**
- New `readNutrientLevel()` / `nutrientBucket()` — reads byte 3 for soil voxels
- Nutrient bucket (0=poor, 1=medium, 2=rich) packed into bits 6-7 of greedy mask
- Prevents greedy merging across nutrient boundaries

**`crates/groundwork-web/src/rendering/terrain.ts`**
- New `SOIL_RICH` (warm golden earth) and `SOIL_GRASS_RICH` (vibrant green) colors
- Soil vertex colors blend toward rich tint: 50% for rich, 20% for medium
- Makes nitrogen handshake visible at terrain level (clover-enriched soil glows golden)

### Sprint 267: Seed Glow Pulse
**`crates/groundwork-web/src/rendering/seeds.ts`**
- New `update(elapsed)` method — seeds pulse emissive intensity (dual-sine: 1.8Hz + 0.7Hz)
- Range 0.1–0.6 emissive intensity — gentle breathing rhythm
- Wired into render loop near foliage update

### Sprint 268: Directional Foliage Wind Lean
**`crates/groundwork-web/src/rendering/foliage.ts`**
- New `uWindDir` vec2 uniform in foliage vertex shader
- Directional lean: `uWindDir * uWindStrength² * heightFactor * 0.6`
- Quadratic falloff means gentle breeze → random sway, strong gust → directional lean
- New `setWindDirection(angle)` method

### Sprint 269: Directional Terrain Wind Lean
**`crates/groundwork-web/src/rendering/terrain.ts`**
- New `uWindDir` vec2 uniform in `onBeforeCompile` solid material shader
- Trunks and branches lean in wind direction: `uWindDir * height * 0.005 * strength²`
- `updateTerrainWind()` now accepts optional `windAngle` parameter
- New `windDirUniform` shared across all solid meshes

### Sprint 270: Directional Falling Leaves
**`crates/groundwork-web/src/rendering/leaves.ts`**
- `setWind()` now accepts `windAngle` parameter
- Falling leaves drift in `(windDirX, windDirZ)` direction instead of hardcoded +X
- Consistent with foliage and terrain lean

### Sprint 271: Wind-Driven Rain Angle
**`crates/groundwork-web/src/rendering/rain.ts`**
- New `setWind(windAngle, strength)` method
- Rain droplets drift horizontally: `windDir * windStrength * 15 * dt`
- Gusty rain falls at a visible angle; calm rain falls straight

### Sprint 272: Wind-Driven Mist Drift
**`crates/groundwork-web/src/rendering/mist.ts`**
- New `setWind(windAngle, strength)` method
- Dawn mist wisps bias drift toward wind direction (0.4× strength)
- Random wobble still present for organic feel

## Coherent Wind System (Sprints 268-272)
All visual elements now respond to the same drifting `windAngle`:
- Foliage billboards lean (quadratic strength, height-scaled)
- Trunk/branch voxels lean (quadratic strength, height-scaled)
- Falling leaves drift (linear, 0.5× strength)
- Rain droplets drift (linear, 15× strength for visible angle)
- Mist wisps drift (linear, 0.4× strength)
- Wind streak particles (already directional from sprint 188)

During gusts, the entire garden bends and flows together.

## Test Results
- TypeScript type-check clean (all 8 sprints)
- Workspace `cargo check` passes
- No sim test regressions (JS-only changes)

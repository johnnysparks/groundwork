# Playtest Feedback — Fauna Surface Height Sweep (Sprint 347)

**Date:** 2026-03-19T11:00:00
**Build:** 6606e13

## Visual Assessment

Hero shot looks consistent with Sprint 346 — dense garden, warm golden lighting, multiple tree species, particles active. 17 fauna spawned in the screenshot garden (300 ticks). This is a mechanical fix, so visual impact shows in the default starter garden over time as more fauna correctly spawn on the slope.

## What Changed (mechanical)
- Pollinators now detect flowers on elevated terrain — previously missed flowers growing above GROUND_LEVEL
- Squirrels now find oaks/berry bushes on the slope for acorn caching
- All fauna z-positions correctly track the sloped terrain surface
- Worms stay underground even on elevated terrain (test was failing before)

## What's Working
1. Dense garden with active ecological competition
2. Day/night cycle creates distinct moods
3. Particles (rain, sparkle, growth) visible
4. X-ray mode reveals underground networks
5. All 6 canonical ecological interactions verified working across terrain

## Remaining Observations
1. **Leaf billboards are very large** relative to terrain — creates a soft/dreamy look but doesn't match the voxel aesthetic. Not blocking.
2. **Ground surface mostly bare** — root/irrigation patterns visible but little groundcover greenery. More moss/grass would help.
3. **All screenshots have similar warm golden tone** — less visual variety across angles than expected.
4. **Screenshot script overlay capture crashes** — the full 14-shot tour crashes on overlay #10 (context destroyed). Needs investigation.

## Status
- P0: none
- P1: none — all mechanical issues from surface_height sweep resolved
- P2: screenshot script crash on overlays, mobile drag-to-zone, etc.

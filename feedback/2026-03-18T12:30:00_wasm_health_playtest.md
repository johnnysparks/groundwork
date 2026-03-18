# Playtest Feedback — Post-WASM Health Fix (Sprint 338)

**Date:** 2026-03-18T12:30:00
**Build:** main @ 992b5b1
**Test type:** Deep playtest + stress test via Playwright

## Health Tinting: VERIFIED WORKING

The P0 bug is fixed. Leaf voxels now carry correct health data:
- Before fix: ALL 1,011 leaves had water_level=0 (health=0%)
- After fix: 9,800 leaves with water_level=255 (health=100%)
- Stress test: 98,097 leaves — 98,001 thriving, 96 dead (die-offs beginning!)

Health tinting is now functional in the renderer. Stressed/dying trees will show brown/yellow foliage.

## P0: Default Garden Is Empty

The **biggest issue** post-fix: the default garden is nearly empty. `create_world_with_garden()` no longer includes a starter garden (removed in commit f952378). A fresh load shows:
- Bare terrain with pond
- Forest ring (decorative)
- Gnome standing idle
- Quest panel

The player must manually plant seeds via quest progression to see ANY plants. This is a terrible first impression — the game looks like an empty sandbox with nothing happening.

**Impact:** New players see a dead-looking world. The "alive garden" fantasy is completely absent on first load. The 50-tick pre-simulation produces nothing because there are no seeds to grow.

## Stress Test: Dense Planting Works Beautifully

When seeds ARE planted densely (stress test scenario), the garden comes alive:
- Massive canopy forms from competing trees
- Die-offs begin at ~96 dead leaves (crowding stress!)
- Health differentiation visible — thriving vs stressed trees
- This is the ecological drama the game needs

## Observations

1. **Sparse default garden** — needs a starter seed bed or pre-planted garden
2. **Rain works** — visible rain particles during playtest
3. **Gnome present** — standing on terrain, quest panel shows
4. **X-ray mode** — underground view functional
5. **Health data flows end-to-end** — Rust sim → WASM → JS → renderer pipeline confirmed

## Recommendations

1. **P0: Restore starter garden** — Plant seeds in `create_world_with_garden()` so the default world has life on first load. Even 10-20 seeds near the pond would transform the experience.
2. **P1: Increase pre-tick count** — 50 pre-ticks isn't enough for trees to mature. Consider 200+ so the starter garden has visible growth.
3. **P1: Verify health tinting is visually distinct** — The data is correct, but need to confirm the renderer actually colors stressed foliage differently.

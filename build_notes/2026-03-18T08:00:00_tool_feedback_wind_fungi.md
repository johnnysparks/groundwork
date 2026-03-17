# Build Notes — Sprints 260-264: Tool Feedback, Wind Sway, Bioluminescence

**Date:** 2026-03-18T08:00:00
**Sprints:** 260-264
**Status:** Shipped

## What Changed

### Sprint 260: Dig Spray Particles
**`crates/groundwork-web/src/rendering/particles.ts`**
- New `emitDigSpray()` — 8 earthy brown particles spray upward from shovel actions
- Bigger burst and more upward velocity than generic growth particles

**`crates/groundwork-web/src/main.ts`**
- Player click-to-dig and gnome task completion both trigger dig spray

### Sprint 261: Seed Scatter Particles
**`crates/groundwork-web/src/rendering/particles.ts`**
- New `emitSeedScatter()` — 6 golden particles tumble outward from planting point
- Warm gold color matches existing seed sparkle palette

### Sprint 262: Soil/Stone Place Particles
**`crates/groundwork-web/src/rendering/particles.ts`**
- New `emitSoilPlace(isStone)` — 4 earthy/gray particles settle from placement
- All 4 tool types now have distinct particle effects

### Sprint 263: Trunk/Branch Wind Sway
**`crates/groundwork-web/src/rendering/terrain.ts`**
- Wind vertex displacement injected via `onBeforeCompile` into solid material
- Above-ground voxels (Y > GROUND_LEVEL + 1) sway with sine-based displacement
- Amplitude scales with height for natural top-heavy tree motion
- Syncs with foliage wind strength (weather-driven, gust-responsive)

### Sprint 264: Bioluminescent Fungi at Night
**`crates/groundwork-web/src/rendering/ecology.ts`**
- New `bioluminescent` color palette (soft teal-green)
- Night-only scan: DeadWood voxels emit slow ethereal glow particles
- Brightness scales with night intensity (uNightAmount > 0.3)
- `emitFromInteractions` and `update` now accept `nightAmount` parameter

## Test Results
- All unit tests pass (111 + 1 ignored)
- All 5 integration tests pass
- TypeScript type-check clean

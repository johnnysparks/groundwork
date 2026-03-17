# Build Notes — Foliage Personality + Atmosphere Polish (Sprints 300-307)

**Date:** 2026-03-18T14:00:00
**Builder:** Dev agent (Claude)
**Branch:** main

## Sprint 300 — Flower day/night bloom cycle + species-specific wind
- Flowers (wildflower, daisy) scale 55%→100% with day cycle: closed at night, open at dawn
- Each tree species has distinct wind personality: oak slow/heavy (0.7x time, 1.3x amp), birch quick/fluttery (1.5x), willow deep swoops (0.5x, 1.6x), pine stiff (0.4x amp)
- New `uDayAmount` uniform on foliage shader, computed from day cycle time

## Sprint 301 — Groundcover breathing
- Moss, grass, clover sprites pulse gently (±4-6% scale) with dual-sine offset by world position
- Independent of wind — forest floor breathes even in still air
- Creates a baseline "alive" micro-movement for groundcover

## Sprint 302 — Willow droop animation
- Willow foliage droops downward proportional to height from ground
- Slow pendulum swing (0.4Hz) intensifies with wind strength
- Creates distinctive weeping silhouette — visually unique from all other species

## Sprint 303 — Canopy shimmer
- Per-instance hashed shimmer phase makes each leaf sparkle independently
- `sin(time * 0.8 + shimmerPhase + worldPos.x * 1.3)` creates shifting highlights
- Stronger in upper canopy, during daytime (reads `vHeight` and `uDayAmount`)
- Simulates sunlight catching individual leaves as camera orbits

## Sprint 304 — Light-responsive foliage tinting
- Reads sim `light_level` byte (index 2) per leaf voxel during foliage rebuild
- Sunlit canopy: warm highlights (+0.06 red), 15% brighter
- Shaded understory: cool blue shift (+0.04 blue), 15% dimmer
- Makes canopy shade structure visible in normal view (not just x-ray particles)

## Sprint 305 — Water canopy reflection
- Water surface picks up subtle green tint proportional to foliage count
- Ripple-distorted noise prevents flat color wash
- Fades at night, scales with `uFoliageDensity` uniform (0→1, normalized to 2000 foliage)
- New export: `updateWaterFoliage(density)`

## Sprint 306 — Zoom-responsive depth of field
- Tilt-shift DOF now adjusts with camera zoom level each frame
- Zoomed in (4x): narrow focus band (0.12), strong blur (6.0px) — intimate miniature view
- Zoomed out (0.35x): wide focus (0.35), weak blur (2.5px) — diorama overview
- New method: `postProcessing.setZoomDOF(zoom)`

## Sprint 307 — Night atmosphere
- Vignette deepens 0.35→0.55 at night, framing the firefly-lit garden center
- Color grade shifts cooler as night amount increases (warmth reduced by 80%)
- Creates cozy blue-moonlit atmosphere vs daytime warmth

## Technical
- All 8 sprints pass `npx tsc --noEmit` on first attempt
- No new dependencies
- Key files: foliage.ts, water.ts, effects.ts, main.ts

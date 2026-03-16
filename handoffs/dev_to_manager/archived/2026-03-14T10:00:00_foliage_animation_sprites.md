# Dev → Manager Handoff: Foliage Animation Sprites

**Date:** 2026-03-14
**Tasks:** WEB-07 (Foliage rendering), WEB-08 (Wind sway), WEB-09 (Growth particles)
**Status:** Complete — ready for visual review

## Summary

Implemented the three foliage/animation features from the P2 backlog:

1. **Billboard foliage sprites** replace blocky Leaf voxel cubes with soft, camera-facing sprites. Per-instance color variation and slight scale randomization create an organic, lush canopy look.

2. **Wind sway animation** via custom vertex shader — all foliage gently sways with sine-wave displacement. Sway amplitude increases with height above ground. Parameters tuned for cozy, not frantic.

3. **Growth particle bursts** — green/golden sparkle particles emit when new vegetation appears. Particles rise, spread, and fade with additive blending. Detects growth by diffing vegetation positions between ticks.

## What Changed
- Greedy mesher now skips Leaf material (rendered as sprites, not cubes)
- New `rendering/foliage.ts` — InstancedMesh billboard system with wind shader
- New `rendering/particles.ts` — pooled particle burst system
- `main.ts` — integrated both systems into render loop
- Mock grid expanded: 7 trees + 3 shrubs for visual testing

## Validation
- TypeScript: clean compile
- Rust workspace: clean compile
- Visual review needed via `npm run dev`

## Recommendations
- These are P2 features and self-contained. No P0/P1 blockers introduced.
- When WASM bridge (WEB-01) lands, foliage/particles need one-line hookup in the tick handler.
- Species-specific foliage shapes could be a follow-up (oak=round, pine=conical, etc.)

## Build Notes
See: `build_notes/2026-03-14T10:00:00_foliage_animation_sprites.md`

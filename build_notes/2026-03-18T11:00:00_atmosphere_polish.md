# Build Notes — Sprints 285-289: Atmosphere & Polish

**Date:** 2026-03-18T11:00:00
**Sprints:** 285-289
**Status:** Shipped

## What Changed

### Sprint 285: Colored Star Variety
**`crates/groundwork-web/src/lighting/sky.ts`**
- Stars now have color temperature variation based on per-star hash
- 30% warm golden (1.0, 0.85, 0.6), 30% soft white, 40% cool blue-white (0.8, 0.9, 1.0)
- Night sky feels more natural and varied

### Sprint 286: Cirrus Wisps
**`crates/groundwork-web/src/lighting/sky.ts`**
- Third cloud layer: thin high-altitude streaky clouds
- Stretched UVs (0.8x horizontal, 2.5x vertical) create elongated shapes
- Visible at mid-elevation (0.25-0.6), drift faster than main clouds
- 25% opacity blend — subtle depth without competing with cumulus

### Sprint 287: Warm AO Tint
**`crates/groundwork-web/src/rendering/terrain.ts`**
- AO shadows now shift toward warm brown (0.25, 0.18, 0.10) instead of pure black
- `aoBlend * 0.3` additive — subtle warmth in corners and crevices
- Makes the garden feel cozier and more inviting

### Sprint 288: Layered Sunrise/Sunset
**`crates/groundwork-web/src/lighting/sky.ts`**
- Three Gaussian band layers replace single horizon glow:
  - Deep orange (tight at horizon, h²×18)
  - Warm amber (slightly above, centered at h=0.08)
  - Pink-violet (highest, centered at h=0.18)
- Opposite-sun counter-glow: faint blue-pink atmospheric scattering
- More photographic sunrise/sunset feel

### Sprint 289: Night Horizon Glow
**`crates/groundwork-web/src/lighting/sky.ts`**
- Faint blue-purple atmospheric glow at horizon during night
- `exp(-h²×6)` Gaussian band, only above horizon
- Creates depth and prevents the night sky from being flat black

## Test Results
- TypeScript type-check clean (all 5 sprints)
- Workspace `cargo check` passes
- No sim test regressions (JS-only changes)

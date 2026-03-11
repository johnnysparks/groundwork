# Dev → Manager Handoff: Seed Growth Loop

**Date:** 2026-03-11T14:00:00
**Sprint:** Seed Growth Loop
**Status:** All 3 P0 tasks shipped

## What shipped

1. **SIM-01: Material placement state bleed** — Fixed. Placing any material now resets water/light/nutrient to 0 (Water gets water_level=255).
2. **SIM-02: Light attenuation through soil** — Fixed. Opaque materials attenuate before assignment. Surface soil ~215, drops ~40/layer, stone blocks completely.
3. **GAME-01: Seed material + growth system** — Shipped. `place seed X Y Z` works. Seeds grow into roots when watered and lit (40 ticks). Three new tests cover growth, no-growth, and light attenuation.

## Acceptance criteria status

- [x] `place seed 30 30 16` works
- [x] `view --z 16` shows `s` for seeds
- [x] After enough ticks near water + light, seed becomes `*` (root)
- [x] `cargo test -p groundwork-sim` passes (15/15)
- [x] `cargo check --workspace` clean

## Tradeoffs made

- Seeds are transparent to light (no attenuation). This means a column of stacked seeds wouldn't block light. Acceptable for MVP since players place seeds on surfaces.
- Growth uses own water_level OR adjacent neighbor water. This is generous — seeds near water grow even without direct contact. Feels right for a cozy garden game.
- 40-tick growth time is a constant, not configurable. Easy to tune later.

## What's next (suggested)

- **Player testing:** Seeds need player feedback. Is 40 ticks too fast/slow? Is it satisfying to watch `s` become `*`?
- **Root spreading:** Currently seeds just convert to a single Root voxel. Roots that spread underground would make the "one more seed" loop more interesting.
- **Seed placement UX:** Player currently must place seeds via CLI. TUI could benefit from a placement mode.
- **Visual feedback:** The TUI seed color (yellow-green) and root color (green) are distinguishable but could use polish.

## Build notes

See `build_notes/2026-03-11T14:00:00_seed_growth_system.md` for technical details.

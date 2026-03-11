# Build Notes: VIS-02 Growth Visibility + Inspect Diagnostics

**Date:** 2026-03-11T17:00:00
**Sprint:** Sprint 3 — Trust & Readability

## What shipped

### VIS-02 Part A: Seed growth stage display (s → S → *)

- **Files changed:**
  - `crates/groundwork-tui/src/cli.rs` — `voxel_char()` now takes `nutrient_level` param; seeds with `nutrient_level >= 100` display as `S`
  - `crates/groundwork-tui/src/render.rs` — `voxel_style()` now takes `nutrient_level` param; seeds with `nutrient_level >= 100` render as `S` in green-shifted color (140,200,60 vs 200,180,60)
- **Display stages:**
  - `s` = seed, nutrient_level 0–99 (yellow-green)
  - `S` = growing seed, nutrient_level 100–199 (greener, signaling life)
  - `*` = root, after nutrient_level hits 200 (material converts)
- **Legend updated** to include `S growing seed`

### VIS-02 Part B: Inspect growth diagnostics

- **File changed:** `crates/groundwork-tui/src/cli.rs` — `cmd_inspect()`
- **When inspecting a Seed voxel, new output includes:**
  - `growth: 120/200 (60%)` — progress bar
  - `water: YES (neighbor below: 98/255)` or `water: NO — need adjacent water_level >= 30`
  - `light: YES (229/255)` or `light: NO — need light_level >= 30`
  - `status: growing (+5/tick, ~16 ticks to root)` or `status: dormant — no water nearby, no light`
- **Neighbor check:** Checks 6 cardinal neighbors (x±1, y±1, z±1) for water_level >= 30, matching the sim's seed_growth system logic. Reports the first found neighbor with direction label.

## Tests added (1)

1. `seed_growth_stages_visible` — Verifies a seed's nutrient_level passes through the <100 and >=100 thresholds during growth, confirming the display character transition is reachable.

## Validation

- `cargo test -p groundwork-sim`: 24/24 pass (was 23, +1 new)
- `cargo check --workspace`: clean (zero warnings)

## Design notes

- The `S` threshold of 100 is halfway to root conversion (200). This gives ~20 ticks of small-seed display and ~20 ticks of growing-seed display, which feels balanced.
- Growing seed color shifts from yellow (200,180,60) toward green (140,200,60) to signal "becoming a plant."
- Inspect diagnostics mirror the exact conditions checked in `seed_growth()` system: water_level >= 30 (own or 6-neighbor) AND light_level >= 30.
- Ticks-to-root calculation uses ceiling division: `(remaining + 4) / 5` to avoid showing "0 ticks" when 1-4 growth points remain.

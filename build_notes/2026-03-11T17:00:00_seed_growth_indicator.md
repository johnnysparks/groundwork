# Build Notes: Seed Growth Indicator (VIS-02)

**Date:** 2026-03-11T17:00:00
**Sprint:** Sprint 3 — Trust & Readability
**Task:** VIS-02 (P0) — Seed growth progress indicator

## What shipped

### VIS-02: Seed growth progress indicator

**Part A — ASCII view (cli.rs + render.rs):**
- Seeds with `nutrient_level` 0–99 display as `s` (unchanged)
- Seeds with `nutrient_level` 100–199 display as `S` (new)
- At 200+: already converts to `*` Root (unchanged)
- Legend updated: `s seed  S seed (growing)`

**Part B — TUI render (render.rs):**
- `voxel_style()` now accepts `nutrient_level` parameter
- Seeds with `nutrient_level >= 100` render as `S` with a slightly brighter green tint
- Seeds below threshold render as `s` (unchanged)

**Part C — Inspect growth diagnostics (cli.rs):**
- When inspecting a Seed, shows: `growth: 120/200 (60%)`
- Water condition check with source: `water: YES (neighbor -z: 98/255)` or `water: NO — need adjacent water_level >= 30`
- Light condition check: `light: YES (229/255)` or `light: NO — need light_level >= 30`
- Overall status: `status: growing (+5/tick, ~16 ticks to root)` or `status: dormant — no water nearby, no light`

## Files changed

- `crates/groundwork-tui/src/cli.rs` — `voxel_char()` gains `nutrient_level` param + `S` branch; `cmd_view()` passes it; `cmd_inspect()` gains seed diagnostics block; legend updated
- `crates/groundwork-tui/src/render.rs` — `voxel_style()` gains `nutrient_level` param + `S` branch with brighter tint; call site updated

## Validation

- `cargo test -p groundwork-sim`: 23/23 pass (no sim changes)
- `cargo check --workspace`: clean (zero warnings)

## Design notes

- Growth threshold of 100 for `S` is at 50% — halfway to conversion. This gives a clear visual "something is happening" signal during the 40-tick wait.
- Inspect diagnostics check all 6 neighbors for water (matching the sim's seed_growth logic exactly), reports the first one found with direction label.
- No sim changes — this is pure display/diagnostics work on the TUI crate.
- The `nutrient_level` field already serves as growth counter (set in Sprint 2). No new fields needed.

## Risks

- **None.** Display-only changes, no sim modifications. All existing tests pass unchanged.

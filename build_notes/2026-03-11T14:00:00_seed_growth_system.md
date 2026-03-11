# Build Notes: Seed Growth System

**Date:** 2026-03-11T14:00:00
**Sprint:** Seed Growth Loop (from manager handoff 2026-03-11T13:00:00)

## What shipped

### SIM-01: Fix material placement state bleed
- **File:** `crates/groundwork-tui/src/cli.rs` (cmd_place)
- **Problem:** Placing a new material preserved old water/light/nutrient levels. Stone placed on wet soil would permanently show water_level=62.
- **Fix:** Reset all levels to 0 when material changes, except Water which gets water_level=255.

### SIM-02: Fix light attenuation through soil
- **File:** `crates/groundwork-sim/src/systems.rs` (light_propagation)
- **Problem:** Light was assigned *before* attenuating for opaque materials, so surface soil got sky-level light (~253). Underground was unrealistically bright, breaking trust in the simulation.
- **Fix:** Opaque materials (Soil, Root, Stone) now attenuate *before* assignment. Transparent materials (Air, Water) attenuate *after*. Surface soil now gets ~215 light, drops ~40 per layer. Stone blocks completely.
- **Attenuation values:** Soil -40, Root -40, Stone → 0, Air -2, Water -15.

### GAME-01: Seed material + growth system
- **Files changed:**
  - `crates/groundwork-sim/src/voxel.rs` — Added `Material::Seed = 5` with full enum support
  - `crates/groundwork-sim/src/systems.rs` — New `seed_growth` system + 3 tests
  - `crates/groundwork-sim/src/lib.rs` — Registered seed_growth after soil_absorption
  - `crates/groundwork-tui/src/cli.rs` — Seed in voxel_char ('s'), help, legend, material counts
  - `crates/groundwork-tui/src/render.rs` — Seed style (yellow-green, 's')
- **Growth mechanic:**
  - Each tick, seeds with water_level >= 30 (own or adjacent neighbor) AND light_level >= 30 gain +5 nutrient_level
  - At nutrient_level >= 200 (40 ticks), seed converts to Root
  - nutrient_level is repurposed as growth counter (no new fields needed)
  - Uses snapshot buffer for neighbor checks, consistent with water_flow/soil_absorption patterns

## System order

`water_flow → soil_absorption → light_propagation → seed_growth → tick_counter`

Seed growth runs after light so seeds get accurate light values on the same tick.

## Tests added (3)

1. `seed_grows_into_root_on_wet_lit_soil` — seed with water_level=100 above ground converts to Root in 50 ticks
2. `seed_does_not_grow_without_water` — seed far from water stays Seed after 50 ticks
3. `light_attenuates_through_soil` — verifies surface soil < sky light, deep soil < surface soil

## Validation

- `cargo test -p groundwork-sim`: 15/15 pass
- `cargo check --workspace`: clean (zero warnings)

## Design notes

- Seeds are transparent to light (no attenuation) since they're small objects sitting on the surface
- water_flow skips seeds (not Air/Water), so seed's own water_level is stable — won't drain
- Seed growth checks *both* own water_level and 6-neighbor water_levels, so seeds work next to water sources even if they have no water on them directly
- Growth rate (5/tick) and threshold (200) are constants, not config — per handoff instructions

## Risks

- **Low:** Save format includes Seed (material byte = 5). Old save files with byte 5 would fail to load since from_u8(5) previously returned None. No existing saves should contain byte 5 since it wasn't a valid material before.
- **Low:** The 40-tick growth time feels good for testing but may need tuning after player sessions. Constant is easy to find in seed_growth().

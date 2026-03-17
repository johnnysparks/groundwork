# Build Notes — Sprint 231: Condition-Based Species Emergence

**Date:** 2026-03-18T02:00:00
**Sprint:** 231
**Status:** Shipped

## What Changed

**P0 gap closed: density-not-species now functional.**

When the player paints density zones (species=255), `seed_growth()` now calls `pick_species_from_conditions()` instead of defaulting all seeds to Oak via `unwrap_or(0)`.

### `crates/groundwork-sim/src/systems.rs`

**New function `pick_species_from_conditions()`:**
- Scores all 12 species against local water level, light level, nutrient level
- Water fitness: matches species `water_need` (Low/Medium/High) against actual moisture
- Light fitness: uses `shade_tolerance` to determine sun-loving vs shade-tolerant
- Nutrient fitness: plant type determines soil richness requirements (groundcover tolerates poor soil, trees need rich)
- Maturity gating: counts existing Tree entities by plant type
  - Groundcover: 40x multiplier (always available as pioneers)
  - Flowers: 20x with 3+ plants, 8x otherwise
  - Shrubs: 20x with 5+ groundcover + 10+ total, scaled down otherwise
  - Trees: 15x with 10+ groundcover + 20+ total, nearly zero in empty gardens
- Temporal bias: early ticks (< 200) add bonus for fast growers
- Deterministic weighted random via `tree_hash` — same conditions produce same species

**Modified `seed_growth()` signature:**
- Added `species_table: Res<SpeciesTable>` and `existing_trees: Query<&Tree>` params
- bevy_ecs auto-injects these via system param extraction

**New tests:**
- `condition_based_species_emergence_produces_variety` — 6 condition sets × 5 positions, asserts ≥4 distinct species and groundcover presence
- `mature_garden_enables_trees` — rich conditions + 30 plants → trees can emerge

## Test Results

110 unit tests + 5 fauna integration tests pass. Zero failures.

## Discovery Arc (Expected Gameplay)

1. Player's first garden: mostly moss, grass, clover emerge → "everything is groundcover"
2. After 3+ plants: wildflowers and daisies start appearing → "flowers grew!"
3. After 10+ plants with groundcover: ferns and berry bushes → "shrubs now too"
4. After 20+ plants, 10+ groundcover: first tree emerges → "a tree appeared! I earned that."

## Remaining P2

- Density influence (dense vs sparse sowing affects species mix)
- Neighbor influence (local plant proximity shifts probabilities)

# Species Diversity — Build Notes

**Date:** 2026-03-13
**Task:** GAME-06: Expand species diversity (shrubs, groundcover, flowers)

## What was built

### 12 species across 4 plant types

| Type | Species | Height | Water | Light | Notes |
|------|---------|--------|-------|-------|-------|
| Tree | Oak | 8m | Med | Med | Round crown, baseline |
| Tree | Birch | 7m | Low | Med | Narrow crown, fast |
| Tree | Willow | 5m | High | Low | Wide crown, shade-tolerant |
| Tree | Pine | 9m | Low | High | Conical crown, tallest |
| Shrub | Fern | 1m | High | Low | Shade-loving, fast spread |
| Shrub | Berry Bush | 1.5m | Med | Med | Round bushy shape |
| Shrub | Holly | 2m | Low | Low | Slow, very hardy |
| Flower | Wildflower | 0.5m | Med | High | Fast growth (2.0x) |
| Flower | Daisy | 0.5m | Low | High | Fastest growth (2.5x) |
| Groundcover | Moss | 0.5m | High | Low | Flat disc, quick spread |
| Groundcover | Grass | 0.5m | Low | High | Flat disc, fastest spread |
| Groundcover | Clover | 0.5m | Med | Med | Flat disc, moderate |

### PlantType enum
`Tree | Shrub | Groundcover | Flower` — controls growth behavior:
- Trees: space colonization branching at YoungTree+ stages
- All others: template-only growth at every stage (no skeleton)

### Template shapes
- **Shrub**: Branch material for woody base + crown with species shape
- **Flower**: Trunk stem + single Leaf bloom at top
- **Groundcover**: Flat disc of Leaf voxels at z=0, ~75% coverage

### Species-specific dispersal
- `dispersal_distance_m`: controls how far seeds fall from parent
- `dispersal_period`: ticks between dispersal attempts (lower = faster spread)
- Groundcover has shortest period (20-25 ticks), trees longest (80-120)

### Species selection
- **CLI**: Species names work as tool names: `place fern 60 60 40`
- **TUI**: `[`/`]` cycles species when seed bag is selected
- `SeedSpeciesMap` registers species ID at seed landing position
- `species_name_to_id()` normalizes names (case-insensitive, ignores hyphens/spaces/underscores)

## Files changed

- `crates/groundwork-sim/src/tree.rs` — PlantType enum, 8 new species, template generators, species_name_to_id()
- `crates/groundwork-sim/src/systems.rs` — PlantType-aware growth/branching/pruning/dispersal
- `crates/groundwork-tui/src/app.rs` — apply_tool returns landing pos, selected_species, parse_tool_and_species()
- `crates/groundwork-tui/src/cli.rs` — Species names in place/fill/tool-start, SeedSpeciesMap registration
- `crates/groundwork-tui/src/input.rs` — [ / ] for species cycling
- `crates/groundwork-tui/src/render.rs` — Species name in tool display, [ / ] in controls
- `CLAUDE.md` — Updated with 12 species docs

## Tests
- 76 sim tests pass (7 new: species_table_has_12_species, plant_types_correct, shrub/flower/groundcover template tests, non_tree_species_no_skeleton, all_species_have_valid_templates_at_all_stages)
- 10 integration tests pass
- CLI verified: place fern/moss/wildflower/berry-bush, tick 150, all grow correctly

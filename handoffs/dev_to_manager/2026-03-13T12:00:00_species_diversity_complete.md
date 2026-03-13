# Dev → Manager: Species Diversity Complete

**Date:** 2026-03-13

## Implemented
- **12 species** across 4 plant types (Tree/Shrub/Groundcover/Flower), up from 4 trees
- **PlantType enum** controlling growth behavior: trees use space colonization, all others template-only
- **Species-specific templates**: shrubs have bushy crowns, flowers have stems+blooms, groundcover spreads flat
- **Species-specific dispersal**: each species has its own distance and period
- **Species selection in CLI**: `place fern 60 60 40` plants a fern. All 12 species names work as tool names.
- **Species selection in TUI**: `[`/`]` cycles species when seed bag is selected. Display shows species name.
- **Interface parity**: both CLI and TUI support species selection (parity rule satisfied)
- 7 new tests (83 total passing)
- CLAUDE.md updated with species documentation

## Not implemented
- Save/load does not persist `SeedSpeciesMap` — seeds placed but not yet grown will lose their species on reload (will become oak). Low risk since seeds grow within ~40 ticks.
- `tool-start`/`tool-end` doesn't store species in `ToolState` — range operations with tool-start default to oak for seeds. Direct `place` and TUI tool operations work correctly.

## Tradeoffs made
- Groundcover uses ~75% coverage disc (hash-based gaps) rather than 100% — looks more natural
- Non-tree plants skip space colonization entirely rather than using a simplified version — keeps code simple, templates look good
- Species names normalize spaces/hyphens/underscores for CLI friendliness (`berry-bush`, `berrybush`, `berry_bush` all work)

## Risks / regressions
- None identified. All 86 tests pass (76 sim + 10 integration).
- Growth balance may need tuning — fast-growing species (daisy 2.5x, wildflower 2.0x) might dominate through dispersal

## Recommended next task
- **Player validation**: have player test planting each species type near water, observe growth patterns
- **GAME-04 (Water depth visual)** or **CLI-13 (Cross-section view)** would help visualize the now-richer garden
- Consider persisting SeedSpeciesMap in save format (v4) if seed loss on reload becomes a problem

## Build validation
```
cargo test --workspace  →  86 tests pass
cargo check --workspace →  clean
CLI: place fern/moss/wildflower/berry-bush → all plant, grow through stages
```

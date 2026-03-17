# Density, Not Species — Player Plants Zones, Garden Picks Species

**Date:** 2026-03-17
**Status:** Accepted (executive mandate)
**Author:** Player feedback → Manager

## Context

The species picker contradicts the game's core discovery loop. When the player selects "Oak" from a menu and places it, there's no surprise — they already know what will grow. The inspect panel, x-ray lenses, and ecological events lose their purpose because the player chose the outcome before it happened.

The game is about *building conditions and discovering what emerges*. The player is a gardener, not a botanist. They shape soil, water, density, and light. The garden decides what grows.

## Decision

**Remove the species picker entirely.** The player paints *density zones* with the seed tool. The sim decides what species emerge based on environmental conditions. Species are *discovered* through the inspect panel after they appear.

### What the player controls
- **Where** to sow (zone painting)
- **How dense** to sow (click = sparse, drag = dense zone)
- **Water flow** (irrigation channels via shovel)
- **Terrain shape** (soil, stone placement)

### What the sim decides
- **Which species** emerge based on local conditions:
  - Moisture level
  - Light availability
  - Soil composition
  - Neighboring plants
  - Sowing density
  - Time / garden maturity

### Discovery arc
- Early gardens (basic conditions) → groundcover (moss, grass, clover)
- Richer conditions (good water, light) → flowers emerge
- Mature gardens (established ecosystem, fauna present) → shrubs and trees appear
- The player learns: "dense sowing near water = different results than sparse sowing on dry hill"

## Known Gap: Condition-Based Species Emergence

**The sim does not yet implement condition-based species selection.** Currently, when a seed is placed with species index 255 (sim picks), the sim falls back to a basic/random selection. The full vision requires:

1. **Environmental fitness scoring** — each species has preferred ranges for moisture, light, soil pH, nutrient levels. The sim scores all candidate species against local conditions and picks probabilistically.

2. **Density influence** — dense sowing favors fast-growing groundcover. Sparse sowing in rich conditions allows slower, larger species (shrubs, trees) to establish.

3. **Neighbor influence** — existing nearby plants shift probabilities. Nitrogen-fixing clover nearby boosts tree emergence. Shade from canopy favors shade-tolerant species.

4. **Maturity gating** — trees should only emerge in gardens with established groundcover and active fauna, matching the old milestone tier system but driven by actual conditions rather than global flags.

5. **Temporal emergence** — early ticks favor fast growers (groundcover). Only after the garden matures do slow growers (trees) have a chance, even in ideal conditions.

This is a **P0 gap** for making the density-not-species design actually work. Until implemented, the sim uses its existing species selection logic as a placeholder.

## Changes Made

### Removed
- Species picker panel from HUD (`#species-panel`, all species buttons, group headers)
- `selectSpecies()` and `cycleSpecies()` methods from Hud class
- `activeSpeciesIndex` from HudState
- Z/C keyboard shortcuts for species cycling
- `setSelectedSpecies` import in main.ts (deprecated in bridge.ts)
- `_playerPlantedSpecies` tracking (every species is now a discovery)
- `_companionSuggested` / `COMPANION_TIPS` (species-specific tips don't apply when player doesn't choose)
- `updateMilestones()` method (species unlock tiers no longer needed in HUD)
- Species panel CSS (`.species-btn`, `.species-group-header`, `#species-panel`, etc.)

### Changed
- Seed placement now passes species=255 (sim picks) instead of player-selected index
- Seed zone spacing is uniform (r=4, spacing=3) instead of species-type-aware
- Event detection treats all new species as discoveries (no "player planted" vs "wild" distinction)
- Help text updated: removed "Z/C: species" reference

### Kept
- `SPECIES` array in bridge.ts (still needed for names in inspect panel, events, tooltips)
- `setSelectedSpecies()` in bridge.ts (deprecated, kept for WASM API compat)
- Species discovery/milestone WASM exports (may be useful for future condition-based gating)
- Inspect panel (now the *primary* way players learn species — more important than ever)

## Quest Implications

Chapters 0-3 are unchanged — they don't reference species selection.

Future chapters (4+) should follow the density/discovery model:
- Chapter 4: "Notice something grew that you didn't plant" (dispersal discovery)
- Chapter 5: "Sow dense near water vs. sparse on dry ground" (density shapes species mix)
- Chapter 6: "A tree appeared" (reward for sustained rich conditions)
- Chapter 7: "The garden sustains itself" (self-sustaining loop, no planting needed)

No chapter should ever say "plant an oak." The player *earns* oaks by building the right conditions.

## Impact

- **hud.ts**: Species picker removed, milestone unlocks removed
- **main.ts**: Species-aware spacing → uniform density, companion tips removed
- **controls.ts**: Z/C keybinds removed
- **quests.ts**: `cycledSpecies` tracking removed
- **bridge.ts**: `setSelectedSpecies` deprecated
- **CLAUDE.md**: Updated to reflect density-not-species design

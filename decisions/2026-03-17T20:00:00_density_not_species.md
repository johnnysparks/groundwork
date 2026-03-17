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

**Sprint 231: Core implementation complete.** The sim now implements condition-based species selection via `pick_species_from_conditions()` in `systems.rs`. When a seed is placed with species index 255 (sim picks), the sim scores all 12 species against local conditions and picks probabilistically using deterministic hashing.

**Implemented:**

1. **Environmental fitness scoring** — each species scored on water match (Low/Medium/High need vs. local level), light match (shade_tolerance vs. local light), and nutrient match (plant type determines soil richness requirements). Weighted random selection via deterministic `tree_hash`.

2. **Maturity gating** — groundcover dominates early gardens (40x multiplier). Flowers need 3+ existing plants. Shrubs need 5+ groundcover and 10+ total. Trees need 10+ groundcover and 20+ total plants. Prevents trees from appearing before the ecosystem can support them.

3. **Temporal emergence** — early ticks (< 200) add a speed bonus proportional to species `growth_rate`, favoring fast-growing groundcover and flowers over slow trees.

**Sprint 233: Neighbor influence implemented.**

4. **Neighbor influence** — scans 8-voxel radius around germinating seeds for existing plants. Clover nearby boosts tree scores +40 (nitrogen handshake). Groundcover boosts flower scores +25 (succession). Tree canopy boosts shade-tolerant species +25 (canopy effect). Same species nearby reduces score -15 (diversity pressure).

**Sprint 242: Density influence implemented.**

5. **Density influence** — dense sowing (5+ seeds within radius 5) boosts groundcover +30, penalizes slower species -10. Moderate density (3+) gives +15 groundcover bonus. Sparse sowing lets conditions decide freely.

**All 5 items complete.**

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

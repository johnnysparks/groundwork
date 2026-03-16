# Discovery & Milestones

The progression system ensures players experience the ecology in the right order.

## Species Discovery

Players don't see all 12 species in a menu. Species are **discovered** through ecological processes.

**Always discovered:** Moss (9), Grass (10), Clover (11)

**Discovery mechanisms:**
| Process | What Gets Discovered |
|---------|---------------------|
| Pioneer succession | Moss, Grass, Wildflower |
| Seed dispersal | Parent tree's species |
| Bird Express | Species of nearby tree |
| Squirrel acorn | Oak |
| Player planting | Whatever they plant |

**Implementation:** `DiscoveredSpecies` resource — u32 bitfield (bit N = species N discovered). Scanned every 20 ticks. One-way: once discovered, stays discovered.

## Milestone Tiers

Plant types gate behind ecological milestones.

| Tier | Unlocks | Condition |
|------|---------|-----------|
| 0 | Groundcover (moss, grass, clover) | Always |
| 1 | Flowers (wildflower, daisy) | 10+ groundcover leaf voxels |
| 2 | Shrubs (fern, berry bush, holly) | 2+ pollinators present |
| 3 | Trees (oak, birch, willow, pine) | 4+ fauna AND 3+ species diversity |

**Implementation:** `EcoMilestones` resource. Tiers are one-way. Raw progress counts exported for UI.

## The UI Model

The species picker shows **plant types** (Groundcover, Flower, Shrub, Tree), not individual species. Within each unlocked type, only discovered species appear. This means:

1. Player starts → sees only "Groundcover" with moss/grass/clover
2. Plants groundcover → milestone tier 1 → "Flower" appears
3. Flowers attract bees → milestone tier 2 → "Shrub" appears
4. Ecosystem develops → milestone tier 3 → "Tree" appears

**Within each type:** `pick_discovered_species(plant_type, rng)` returns a random discovered species of that type. The player picks a *category*, the sim picks the *species*.

## WASM Exports
- `milestone_tier1_flowers()`, `milestone_tier2_shrubs()`, `milestone_tier3_trees()`: tier states
- `milestone_groundcover_count()`, `milestone_pollinator_count()`, `milestone_fauna_count()`, `milestone_species_diversity()`: raw progress
- `discovered_species()`: u32 bitfield
- `is_species_discovered(id)`: per-species check
- `pick_discovered_species(type, rng)`: random pick from discovered species of a type

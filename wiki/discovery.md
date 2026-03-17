# Discovery & Species Emergence

The player doesn't choose species. They paint **density zones** with the seed tool, and the sim decides what grows. Species are *discovered* through the inspect panel after they appear.

## Condition-Based Emergence

When a seed germinates, `pick_species_from_conditions()` (in `systems.rs`) scores all 12 species against local conditions and picks probabilistically using deterministic hashing:

1. **Water fitness** — matches species water_need (Low/Medium/High) against local water level
2. **Light fitness** — matches species shade_tolerance against local light level
3. **Nutrient fitness** — plant type determines soil richness requirements (trees need rich soil, groundcover thrives anywhere)
4. **Neighbor influence** — scans 8-voxel radius for nearby species (trunk/leaf voxels with species_id):
   - Clover (id 11) nearby → +40 score for trees (nitrogen fixing boosts tree emergence)
   - Any groundcover nearby → +25 for flowers (succession: groundcover enables flowers)
   - Any tree nearby → +25 for shade-tolerant species (canopy effect: shade_tolerance < 80)
   - Same species nearby → -15 score (diversity pressure prevents monoculture)
5. **Density influence** — counts seeds within 5-voxel radius:
   - Dense (5+ seeds): groundcover +30, flowers -5, shrubs/trees -10 (pioneers win in crowds)
   - Moderate (3-4 seeds): groundcover +15
   - Sparse (0-2 seeds): no modifier — let conditions decide
6. **Maturity gating** — garden development stage controls which plant types can emerge:
   - Groundcover: always (4.0x multiplier — pioneer species)
   - Flowers: need 3+ existing plants (2.0x)
   - Shrubs: need 5+ groundcover and 10+ total plants (2.0x)
   - Trees: need 10+ groundcover and 20+ total plants (1.5x)
7. **Temporal bias** — early ticks (< 200) favor fast growers via growth_rate bonus

**Result:** Same conditions at the same tick/position always produce the same species (deterministic). Different environmental conditions produce different species mixes.

## Natural Succession

Beyond player-planted density zones, species also appear through ecological processes:

| Process | What Appears | Trigger |
|---------|-------------|---------|
| Pioneer succession | Moss → Grass → Wildflower | Bare moist soil, sampled every 50 ticks |
| Seed dispersal | Parent tree's species | Mature trees drop seeds nearby (80-120 tick interval) |
| OldGrowth seed rain | Parent tree's species | 2x dispersal frequency |
| Bird Express | Nearby tree species | Birds carry seeds 10-20 voxels away |
| Squirrel acorn cache | Oak | Squirrels cache acorns that sprout (~30% chance) |
| Wind drift | Inherited from seed | +1 voxel lateral shift every 3 ticks at altitude |

## Species Discovery (Inspect Panel)

The inspect panel is the primary way players learn what species appeared and why. When the player inspects a grown plant, they see:
- Species name and type
- Current health and growth stage
- Environmental conditions at that location

Every new species that appears generates a discovery event in the event feed.

## WASM Exports

- `discovered_species()`: u32 bitfield (bit N = species N seen)
- `is_species_discovered(id)`: per-species check
- `milestone_groundcover_count()`, `milestone_pollinator_count()`, `milestone_fauna_count()`, `milestone_species_diversity()`: raw progress counts

## Design Philosophy

The density-not-species model supports the core discovery loop: the player shapes conditions, observes what emerges, and learns to create the right environment for desired species. "How do I get an oak?" is answered by sustained rich conditions and a mature ecosystem — not by clicking a menu item.

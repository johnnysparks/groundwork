# Build Notes: Interaction Systems Sprint
**Date:** 2026-03-16T12:30:00
**Dev:** Claude (simulation enhancement workstream)
**Theme:** Discoverable ecological interaction chains

## What Shipped (2 sprints)

### Sprint 3: Pollinator Bridge + Allelopathy

**Pollinator Bridge (SIM-12):**
- tree_growth now reads FaunaList to detect nearby pollinators
- Each bee/butterfly within 10 voxels of a tree gives +0.005 health recovery (max +0.02 from 4 pollinators)
- Creates discovery: flowers → bees arrive → nearby trees recover health faster
- Test: `pollinator_boosts_tree_health`

**Pine Allelopathy (SIM-16):**
- soil_evolution detects pine roots (species_id=3) adjacent to soil cells
- Pine roots lower pH 5× faster than organic decay
- seed_growth checks soil pH: acidic soil (pH < 40) halves growth rate for non-tolerant species
- Pine, fern, moss are acid-tolerant (immune to penalty)
- Creates discovery: "Nothing grows near my pines — the soil is too acidic!"
- Test: `pine_roots_acidify_soil`

### Sprint 4: Bird Express Enhancement

**Species-specific seed transport:**
- Birds now read species_id from nearby leaf/trunk voxels
- Dropped seeds are registered in SeedSpeciesMap with correct species
- Previously all bird seeds defaulted to oak (species 0)

**Soil enrichment (droppings):**
- Birds deposit +3 nutrients and +2 organic matter per effect tick
- Creates nutrient hotspots under bird perching areas

**Berry bush affinity:**
- Birds spawn at lower threshold near berry bush leaf clusters (12 leaves vs 8 trunk/branch)
- Creates feedback loop: berry bush → birds → berry seeds spread → more berry bushes

## Interaction Chains Now in the Game

1. **Nitrogen Handshake**: clover near oak → 1.5× growth boost
2. **Pollinator Bridge**: flowers → bees → tree health recovery
3. **Pine Territory**: pine → acidic soil → suppresses non-tolerant neighbors
4. **Bird Express**: berry bush → birds → species-specific seeds spread + soil enrichment
5. **Canopy Effect**: tall trees → shade → fern/moss grow 1.5× faster
6. **Pioneer Succession**: bare soil → moss → grass → wildflower (autonomous)
7. **Decomposition Cycle**: dead tree → DeadWood → beetles → nutrient-rich soil
8. **Root Competition**: overlapping roots → shared water pool → weaker trees stressed

## Test Results
- 93 tests pass (88 unit + 5 integration)
- 5 new tests for interaction systems

## Architecture Notes
- tree_growth now takes `Res<FaunaList>` parameter for pollinator check
- fauna_effects now takes `ResMut<SeedSpeciesMap>` for bird seed registration
- soil_evolution snapshot extended to 3-tuple (material, water, nutrient) for allelopathy

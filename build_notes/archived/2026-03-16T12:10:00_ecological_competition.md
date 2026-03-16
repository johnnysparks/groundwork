# Build Notes: Ecological Competition System
**Date:** 2026-03-16T12:10:00
**Dev:** Claude (simulation enhancement workstream)
**Theme:** Simulation enhancement — bring the garden to life through competition

## What Shipped

4 interlocking changes that create ecological competition where none existed:

### 1. Doubled Leaf Light Attenuation
- `scale_attenuation(50)` → `scale_attenuation(100)` for leaves
- At 5cm/voxel: 11 light blocked per leaf → 22 per leaf
- A canopy with 5 leaf layers now blocks 110/255 light (was 55)
- Creates real shade zones under mature trees

### 2. Territorial Seed Suppression
- Seeds within 6 voxels (~30cm) of an existing Trunk voxel won't germinate
- Prevents trees from growing on top of each other
- The radius is intentionally small — competition kicks in at the canopy level
- New test: `territorial_suppression_prevents_crowded_germination`

### 3. Youth Vulnerability (3-4× faster stress death)
- Seedlings take 4× health damage from stress
- Saplings take 3×, young trees 2×, mature 1×
- Seedlings die at age 20 (was 50 for all stages)
- Creates natural thinning — only well-placed seedlings survive
- New test: `crowded_seedlings_die_from_shade`

### 4. Root Water Competition
- Pre-computes how many root voxels neighbor each soil cell
- Water transfer divided by competitor count: 2 roots → each gets half
- Makes overlapping root zones a real disadvantage
- Trees in crowded conditions get less water per root

### 5. Species-Aware Seed Spacing (JS)
- Trees: 16 voxel spacing, 16 voxel radius → 1-4 seeds per click
- Shrubs: 8/8
- Flowers: 4/6
- Groundcover: 3/6
- Prevents the "dump 16 seeds in a 4-voxel radius" problem

## Test Results
- 90 tests pass (85 unit + 5 integration)
- 2 new tests for competition mechanics
- TypeScript compiles clean
- Workspace compiles clean

## What This Enables
- Trees should now have distinct territories
- Seedlings under canopies should die, creating undergrowth niches
- Root overlap creates water scarcity for competing trees
- The garden should show "survival of the fittest" dynamics after 200+ ticks

## Next Priorities for Sim Enhancement
1. **Dead tree cleanup / decomposition** — dead trees should attract decomposers (beetles), enrich soil, and eventually crumble
2. **Canopy effect on undergrowth** — shade-tolerant species (fern, moss) should *benefit* from canopy shade (reduced competition from sun-loving plants)
3. **Water stress visual indicator** — the health→water_level on leaves is already exported; the renderer needs to use it for yellow/brown coloring
4. **Pollinator bridge strengthening** — flower clusters should attract more pollinators, which should visibly boost nearby plant health

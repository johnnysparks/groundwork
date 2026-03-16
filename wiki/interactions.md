# Interaction Chains

18 discoverable ecological relationships. Each creates a "why did that happen?" moment the player can trace backward.

## The Chains

### 1. Nitrogen Handshake
`clover/groundcover near tree roots -> 1.5x tree growth`

- **Mechanism:** 3+ Leaf voxels (groundcover) within 5 voxels of tree root_pos at ground level
- **Multiplier:** 1.5x on accumulated_water and accumulated_light
- **Only for:** Tree plant types
- **Discovery:** "Clover near my oak's roots makes it grow faster"
- **Cascade:** Faster oak -> larger canopy -> shade environment -> fern thrives below

### 2. Pollinator Bridge
`flowers -> bees/butterflies -> tree health recovery`

- **Mechanism:** Each bee/butterfly within 10 voxels of tree root_pos gives +0.005 health (max +0.02 from 4 pollinators)
- **Spawn trigger:** 6+ Leaf voxels in radius 8, doubled if 5+ flower-species leaves
- **Discovery:** "I planted flowers near my oak, bees came, the oak recovered"
- **Cascade:** Healthier tree -> more seeds -> more trees -> more pollinators

### 3. Pine Allelopathy
`pine roots -> acidic soil -> suppressed neighbors`

- **Mechanism:** Pine roots (species_id=3) lower adjacent soil pH by 5/cycle in soil_evolution
- **Effect:** Seeds in pH<40 soil grow at half speed
- **Immune:** Pine, fern, moss (species 3, 4, 9)
- **Discovery:** "My seeds won't grow near the pine... the soil is too acidic!"
- **Counter:** Plant fern/moss under pine — they're immune and thrive in shade

### 4. Bird Express
`berry bush -> birds -> species-specific seeds + soil enrichment`

- **Mechanism:** Birds read species_id from nearby leaf/trunk voxels, drop seeds 10-20 voxels away with correct species_id. Also +3 nutrients and +2 organic at perch site.
- **Spawn:** trunk+branch >= 8 OR leaf_count >= 12 (berry bushes lower threshold)
- **Discovery:** "A berry bush appeared far from where I planted — a bird must have carried the seed"

### 5. Canopy Effect
`tall tree shade -> shade-tolerant species thrive`

- **Mechanism:** Species with shade_tolerance < 60 get 1.5x growth in moderate shade (light 5-30)
- **Species:** Fern (30), Moss (20), Holly (40)
- **Discovery:** "Ferns grow faster under the oak canopy — they love the shade"
- **Creates:** Layered forest structure: oak (2.5m) > fern (0.4m) > moss (0.05m)

### 6. Pioneer Succession
`bare moist soil -> moss -> grass -> wildflower`

- **Mechanism:** Every 50 ticks, samples 20 random surface positions. Places seeds based on what's already nearby (no moss = place moss, moss present = place grass, grass present = place wildflower).
- **Moisture threshold:** 20 (or 5 near DeadWood)
- **Discovery:** "The bare patch filled itself with moss, then grass, then wildflowers"
- **Significance:** The garden has agency — it recovers without player intervention

### 7. Decomposition Cycle
`dead tree -> DeadWood -> beetles -> nutrient-rich soil`

- **Passive:** DeadWood gains +2 decay/20 ticks. At 250: converts to Soil (nutrient=60, organic+40, bacteria+20)
- **Beetle acceleration:** +3 nutrient/tick per adjacent DeadWood. At 200: immediate soil conversion
- **Discovery:** "The dead tree disappeared — the soil underneath is really fertile now"

### 8. Root Competition
`overlapping root zones -> shared water -> weaker trees stressed`

- **Mechanism:** Root water absorption divided by number of competing root voxels neighboring each soil cell
- **Effect:** 4 roots sharing one soil cell each get 1/4 the water
- **Result:** Crowded trees get less water -> health declines -> natural thinning
- **Discovery:** "Most of my crowded oaks died, but the survivors exploded with growth"

### 9. Drought Recovery
`wet roots -> dead tree revives`

- **Mechanism:** Dead trees with root water_level > 100 recover health at +0.006/tick. At health 0.3: revive as Sapling with fresh branches.
- **Discovery:** "My dead tree came back to life after I added water nearby!"
- **Design intent:** Mistakes don't punish — the garden recovers. Experimentation is safe.

### 10. Willow Water Affinity
`water access -> 2x willow growth`

- **Mechanism:** Willow (species_id=2) gets 2.0x growth multiplier when water_intake > 50
- **Discovery:** "My willow by the stream is growing twice as fast"
- **Strategic:** Plant willows near water sources as ecosystem anchors

### 11. Pioneer Birch
`open ground -> 1.5x birch growth`

- **Mechanism:** Birch (species_id=1) gets 1.5x boost when no Trunk voxels within 8 voxels at trunk height
- **Discovery:** "The birch shot up fast in the clearing, but slowed once the oak grew tall"
- **Ecological role:** First colonizer, makes way for slower-growing oaks

### 12. Berry-Bird Symbiosis
`birds near berry bush -> 1.5x berry growth`

- **Mechanism:** Berry bush (species_id=5) gets 1.5x growth when Bird fauna within 12 voxels
- **Loop:** Berry bush attracts birds -> birds boost berry bush -> more berries -> more birds
- **Discovery:** "My berry bush is thriving — oh, the birds are helping it!"

### 13. Nurse Log Effect
`DeadWood -> 2x seed germination nearby`

- **Mechanism:** Seeds adjacent to DeadWood voxels get 2x growth rate. Pioneer succession moisture threshold lowered from 20 to 5 near DeadWood.
- **Discovery:** "Seedlings keep sprouting near that dead tree!"
- **Creates:** Forest rotation cycle: grow -> compete -> die -> nurse -> regrow

### 14. Carrying Capacity
`dense roots -> soil bacteria decline -> weakened trees`

- **Mechanism:** Soil cells with 4+ adjacent root voxels lose -3 bacteria/cycle
- **Effect:** Lower bacteria -> lower nutrient generation -> slower growth
- **Discovery:** "The soil quality dropped under my dense forest!"
- **Self-regulation:** Prevents infinite forest growth. Creates natural openings.

### 15. Flower Meadows
`flower clusters -> pollinator swarms`

- **Mechanism:** 5+ flower-species Leaf voxels (wildflower=7, daisy=8) double pollinator spawn probability and increase cap from 3 to 5
- **Discovery:** "My flower patch is swarming with bees!"
- **Strategic:** Concentrate flowers for maximum pollinator effect

### 16. Mycorrhizal Network
`same-species roots nearby -> health sharing`

- **Mechanism:** Trees of same species with root voxels within 3 voxels: healthier tree transfers +0.005 health/cycle to weaker neighbor
- **Discovery:** "My oaks are supporting each other through their roots"
- **Strategic:** Plant same-species groves for mutual support

### 17. Wind Seed Drift
`seeds in air -> directional drift`

- **Mechanism:** Seeds 8+ voxels above surface drift 1 voxel per 3 ticks in wind direction. Wind rotates through 4 cardinal directions every 500 ticks.
- **Discovery:** "Seeds always drift east — they create a spread pattern over time"

### 18. Squirrel Acorn Caching
`squirrel -> buried oak seeds`

- **Mechanism:** Squirrels (spawn near 10+ oak/berry leaf voxels) cache oak seeds (30% chance when Acting) at random locations
- **Discovery:** "An oak seedling appeared in the clearing — the squirrel buried an acorn there!"

---

## Feedback Loops

### Self-Reinforcing (Positive)
- Flowers -> pollinators -> healthier trees -> more seeds -> more flowers
- Berry bush -> birds -> seed spread -> more berry bushes -> more birds
- Clover -> nitrogen -> faster oak -> more shade -> more fern/moss -> more nitrogen

### Self-Regulating (Negative)
- Dense trees -> root competition -> shared water -> natural thinning
- Dense roots -> bacteria decline -> lower nutrients -> slower growth
- Canopy shade -> understory death -> soil decline -> canopy weakening

### Recovery Cycles
- Death -> DeadWood -> nurse log -> new seedlings -> new growth
- Drought -> stress -> death -> rain returns -> drought recovery -> regrowth
- Competition -> thinning -> survivors accelerate -> new equilibrium

---

## Interaction Depth

The chains layer on each other. The longest discoverable chain:

```
Plant clover (groundcover)
  -> Nitrogen handshake boosts nearby oak (1.5x)
    -> Oak grows large canopy
      -> Canopy shades ground below
        -> Fern thrives in shade (canopy effect 1.5x)
          -> Fern's moisture-holding creates wetter soil
            -> Moss colonizes nearby (pioneer succession)
              -> Groundcover density triggers milestone tier 1
                -> Flowers unlocked
                  -> Wildflowers attract bees (pollinator bridge)
                    -> Bees boost oak health (+0.02)
                      -> Oak reaches OldGrowth
                        -> Seed rain creates offspring
                          -> Mycorrhizal network supports young oaks
```

13 steps. This is the "twentieth hour" discovery.

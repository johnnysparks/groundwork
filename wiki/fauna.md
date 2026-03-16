# Fauna

6 fauna types. Max 128 active simultaneously. Each serves a distinct ecological role — fauna are the connective tissue between plant species.

## Fauna Types

### Bee (id: 0)
**Pollinator. Connects flowers to trees.**

| Property | Value |
|----------|-------|
| Spawn trigger | 6+ Leaf voxels in radius 8 (doubled by flower clusters) |
| Max per area | 3 (5 in flower meadows) |
| Lifetime | 200-400 ticks |
| Movement | Figure-8 drift, speed 0.12/tick |
| Effect | +5 nutrient to nearby Seeds (pollination boost) |
| Health boost | +0.005 per bee near a tree (pollinator bridge) |

**Behavior:** Idle hovering with figure-8 drift -> seeks flower (Leaf voxel) every 30 ticks -> hovers near flower (Acting) -> returns to Idle.

### Butterfly (id: 1)
**Alternative pollinator. Visual variety.**

Same as Bee but slower (speed 0.08/tick) and spawns as 1/3 of pollinator spawns. Identical ecological effects.

### Bird (id: 2)
**Seed disperser. Creates "gift" plantings.**

| Property | Value |
|----------|-------|
| Spawn trigger | trunk+branch >= 8, OR leaf >= 12 (berry bushes lower threshold) |
| Max per area | 2 |
| Lifetime | 300-600 ticks |
| Movement | Circular flight at canopy height, radius 5-10, speed 0.03 rad/tick |
| Seed drop | ~5% chance/effect tick, 10-20 voxels away, carries parent species |
| Soil enrichment | +3 nutrient, +2 organic at perch location |

**Behavior:** Circles above canopy -> occasionally swoops (every 80 ticks) -> drops seeds when near trees -> enriches soil with droppings.

### Worm (id: 3)
**Underground decomposer. Enriches soil.**

| Property | Value |
|----------|-------|
| Spawn trigger | Underground soil (z=GL-3), water>50, organic>20 |
| Max per area | 2 |
| Lifetime | 400-600 ticks |
| Movement | Sinusoidal underground path, speed 0.03/tick |
| Effect | +1 organic, +2 bacteria, +2 nutrient per tick in adjacent soil |

**Behavior:** Slow sinusoidal drift through soil. Stays underground (clamped between GL-8 and GL-0.5).

### Beetle (id: 4)
**Surface decomposer. Accelerates deadwood decay.**

| Property | Value |
|----------|-------|
| Spawn trigger | 2+ DeadWood voxels within 6 |
| Max per area | 2 |
| Lifetime | 250-400 ticks |
| Movement | Surface crawl near dead wood, speed 0.04/tick |
| Effect | +3 nutrient per adjacent DeadWood/tick. At 200: converts to Soil |

**Behavior:** Crawls on surfaces near dead wood. Stays near ground level.

### Squirrel (id: 5)
**Companion. Caches acorns that sprout.**

| Property | Value |
|----------|-------|
| Spawn trigger | 10+ oak/berry leaf/trunk voxels within 8 |
| Max per area | 2 |
| Lifetime | 500-800 ticks (longest-lived) |
| Movement | Dart-and-pause, speed 0.15/tick (fastest ground fauna) |
| Effect | 30% chance to cache oak seed (species 0) when Acting |

**Behavior:** Idle twitching -> seeks random nearby target every 20 ticks -> darts to target at high speed with bobbing motion -> brief "digging" animation -> sometimes caches acorn -> returns to Idle.

**Future:** Domesticable by the gnome (builds trust over time, carries out tasks).

---

## Spawn System

Checked every 20 ticks. Samples 9 evenly-spaced points across the garden:
```
(1/4, 1/4)  (1/2, 1/4)  (3/4, 1/4)
(1/4, 1/2)  (1/2, 1/2)  (3/4, 1/2)
(1/4, 3/4)  (1/2, 3/4)  (3/4, 3/4)
```

At each point, checks conditions for all fauna types. Spawn probability scales with ecological density. Capped at MAX_FAUNA=128 total.

## Design Intent

Fauna are the **visible proof** that the ecosystem is working. A garden without fauna is just geometry. A garden with bees drifting between flowers, birds circling above canopy, and a squirrel darting between oaks is *alive*. Each fauna type rewards a specific planting strategy — the player learns "if I plant flowers, bees come; if I plant berries, birds come."

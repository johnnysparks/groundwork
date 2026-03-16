# Succession & Recovery

The garden recovers from mistakes. This is core to the "cozy" promise — experimentation should feel safe.

## Pioneer Succession
Runs every 50 ticks, samples 20 random surface positions.

**Stages (each requires the previous nearby):**
1. **Bare moist soil** (water >= 20, or >= 5 near DeadWood) -> **Moss** (species 9)
2. **Moss present** within 4 voxels -> **Grass** (species 10)
3. **Grass present** within 4 voxels -> **Wildflower** (species 7)

Each stage has a probability gate (hash-based, ~25% moss, ~20% grass, ~12% wildflower).

**Nurse log bonus:** DeadWood within 3 voxels lowers moisture threshold from 20 to 5, enabling succession in drier conditions.

## Deadwood Decomposition
- **Passive:** +2 decay per 20 ticks (~2550 ticks to full decomposition)
- **Moisture bonus:** +0.5-3 extra if adjacent soil wet
- **Beetle acceleration:** +3 nutrient per adjacent DeadWood per 10-tick effect cycle
- **Conversion:** At decay >= 250 (underground) -> Soil with nutrient=60, organic+40, bacteria+20
- **Above ground:** Converts to Air (wood crumbles away)

## Drought Recovery
Dead trees with wet roots (root water_level > 100 total):
- Health recovers at +0.006/tick
- At health 0.3: revives as Sapling with fresh skeleton
- Dead trees don't re-rasterize during recovery (preserves roots)
- Takes ~50 ticks of sustained water to revive

**Discovery:** "My tree came back to life after I added water nearby!"

## The Forest Rotation Cycle
```
Trees grow
  -> Compete for light and water
  -> Weaker trees die -> DeadWood
  -> DeadWood nurtures seedlings (nurse log 2x germination)
  -> Pioneer succession fills gaps (moss -> grass -> wildflower)
  -> Beetles decompose deadwood -> nutrient-rich soil
  -> New trees grow in enriched soil
  -> Cycle repeats
```

This runs autonomously. A player who stops clicking sees the garden rotate through generations.

## Weather Recovery
- **Drought stress:** surface water evaporates, shallow soil dries
- **Rain relief:** 40% chance drought ends with rain
- **Deep-rooted trees survive:** roots reach sustained water below drought-affected surface layers
- **Strategic implication:** plant near water or plant deep-rooting species for drought resilience

## Design Intent
Recovery is a *feature*, not a failure state. When the player floods, over-digs, or crowds, the response should be visible organic recovery. This teaches the player that experimentation is safe, making them bolder and more creative.

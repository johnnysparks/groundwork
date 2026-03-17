# Competition

Competition is what makes the garden an *ecosystem* instead of a toy. Without it, planting is just clicking.

## Shade Competition
- Leaf voxels attenuate ~22 light each (scaled from base 100)
- Dense canopy (10+ leaf layers) creates near-darkness below
- Trees below light threshold lose health (shade penalty * youth multiplier)
- Sun-loving species (pine, daisy, grass) suffer more in shade
- Shade-tolerant species (moss, fern, holly) actually *benefit* (canopy effect 1.5x)

## Root Water Competition
- Each root absorbs water from adjacent soil, but transfer is **divided by competitor count**
- 4 roots sharing one soil cell: each gets 1/4 the water
- Trees with more roots still get more total water (they tap more cells)
- Effect: crowded root zones create water scarcity for everyone

## Root Water Decay
- Roots without adjacent wet soil (water_level > 20) lose -2 water/tick
- Isolated roots dry out in ~100 ticks
- This enforces real water dependency — removing water kills plants eventually
- Before this fix, roots held water forever and drought was cosmetic

## Territorial Seed Suppression
- Seeds within 6 voxels of existing Trunk voxels **won't germinate**
- Prevents trees from growing directly on top of each other
- Player must space plantings intentionally

## Crowding Death
- Young plants (seedling/sapling) take 3-4x more health damage from stress
- Death threshold: health < 0.1 for 20 ticks (young) or 50 ticks (mature)
- This creates **natural thinning** — only the best-placed seedlings survive
- Acceptance test: 10 oaks in tight cluster -> at least 2 die after 400 ticks

## Carrying Capacity
- Dense root zones (4+ adjacent root voxels) suppress soil bacteria (-3/cycle)
- Lower bacteria -> lower nutrient generation -> slower growth
- Self-regulating: too many trees -> poor soil -> weakened trees -> natural thinning

## Pine Allelopathy
- Pine roots lower adjacent soil pH by 5 per soil_evolution cycle
- Seeds in pH < 40 soil grow at half speed (except pine/fern/moss)
- Creates species-specific territory around pines

## The Competition Cycle
```
Trees planted close together
  -> Canopies overlap -> shade stress
  -> Roots compete -> water scarcity
  -> Young trees die first (4x vulnerability)
  -> Survivors get more light + water (less competition)
  -> Survivors grow faster -> reach Mature
  -> Natural thinning creates spacing
```

## Balance Notes
- **Partial recovery (+0.002/tick)** must be lower than shade penalty (~0.006) for thinning to work
- Previously recovery was +0.005 which nearly cancelled shade -> trees never died -> no drama
- The pond ensures at least *some* trees near water always survive -> player feels safe to experiment

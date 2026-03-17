# Seeds & Germination

Seeds are the transition from player action to living plant.

## Germination Requirements
- Water: water_level >= 30 (own cell or any adjacent neighbor)
- Light: light_level >= 30
- NOT compacted soil (clay > 200 && organic < 30 in adjacent soil)
- NOT within 6 voxels of an existing Trunk (territorial suppression)

## Growth Rate
- **Base:** +12 nutrient_level per tick (germination at 200 -> ~17 ticks)
- **Soil bonus:** +(best_adjacent_nutrient * 5 / 255) extra per tick
- **Nurse log:** x2 if adjacent to DeadWood
- **Allelopathy:** /2 if soil pH < 40 and species not acid-tolerant (pine/fern/moss)

## Germination Process
At nutrient_level >= 200:
1. Seed voxel -> Trunk voxel
2. Place Root in soil below (up to 2 voxels deep)
3. Spawn Tree entity with:
   - stage = Seedling
   - health = 1.0
   - accumulated_water = 40 (head start for fast first leaf)
   - accumulated_light = 40

## Territorial Suppression
Seeds within 6 voxels (~30cm) of any Trunk voxel are suppressed — they don't accumulate growth. This prevents trees from growing on top of each other. The radius is small enough that strategic placement still works, but random scatter gets thinned.

## Seed Sources
| Source | Species | Distance | Frequency |
|--------|---------|----------|-----------|
| Player density zone | Sim-selected (based on conditions) | At zone | On demand |
| Seed dispersal | Parent tree | 0.5-1.5m | Every 80-120 ticks (mature) |
| OldGrowth rain | Parent tree | 0.5-1.5m | Every 40-60 ticks (2x frequency) |
| Bird Express | Nearby tree | 10-20 voxels | ~5%/effect tick |
| Squirrel cache | Oak (always) | At squirrel pos | ~30%/Acting tick |
| Pioneer succession | Stage-dependent | Same position | Every 50 ticks |
| Wind drift | Inherited | +1 voxel lateral | Every 3 ticks (high altitude only) |

## Design Intent
The 17-tick germination time was tuned to competitive benchmarks: industry standard for cozy games is 1-5 seconds for first visual feedback. Our trunk appears at ~1.7s, first leaf at ~3s. Previously this was 4+ seconds for trunk and 13+ seconds for leaf — a retention killer.

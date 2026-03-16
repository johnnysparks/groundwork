# Light System

Light propagates top-down through each column. Canopy shade is the primary competitive mechanism between trees.

## Propagation
- Each (x, y) column starts at light=255 (full sun) at the top
- Attenuates through each voxel based on material type
- All attenuation values scaled by `sqrt(VOXEL_SIZE_M)` = 0.224

| Material | Base Attenuation | Scaled (per voxel) |
|----------|------------------|--------------------|
| Leaf | 100 | ~22 |
| Trunk/Soil/Root | 30 | ~7 |
| Branch | 20 | ~4 |
| DeadWood | 10 | ~2 |
| Water | 15 | ~3 |
| Air/Seed | 2 | ~1 |
| Stone | total | 0 (blocks all) |

**Example:** A canopy with 5 layers of Leaf voxels blocks ~110 light (22 per layer), leaving ~145 below. Dense canopies with 10+ layers create near-darkness.

## Day-Night Cycle
`DayPhase` resource cycles 0-99 each tick:

| Phase | Range | Growth Multiplier |
|-------|-------|-------------------|
| Dawn | 0-24 | 0.75x |
| Day | 25-49 | 1.0x |
| Dusk | 50-74 | 0.75x |
| Night | 75-99 | 0.5x |

Full day = 100 ticks = 10 seconds at default speed.

## Shade Thresholds
Each species has a `shade_tolerance` (0-255). Lower = more shade-tolerant.

Light threshold formula: `10 + (1 - shade_tolerance/255) * 40`

| Species | shade_tolerance | Light Threshold | Niche |
|---------|----------------|-----------------|-------|
| Moss | 20 | ~13 | Deep shade pioneer |
| Fern | 30 | ~15 | Understory |
| Holly | 40 | ~17 | Shade survivor |
| Willow | 60 | ~21 | Moderate shade |
| Oak | 80 | ~24 | Needs some light |
| Pine | 100 | ~28 | Moderate sun-lover |
| Birch | 120 | ~32 | Moderate sun-lover |
| Grass | 140 | ~37 | Sun-loving |
| Wildflower | 150 | ~40 | Sun-loving |
| Daisy | 160 | ~43 | Strong sun-lover |

**Canopy Effect:** Species with shade_tolerance < 60 get 1.5x growth boost in moderate shade (light_intake 5-30).

# World & Scale

## Grid Dimensions

| Property | Value | Real-world |
|----------|-------|------------|
| Width (X) | 80 voxels | 4.0m |
| Depth (Y) | 80 voxels | 4.0m |
| Height (Z) | 100 voxels | 5.0m |
| Voxel size | 0.05m | 5cm cubes |
| Ground level | Z=40 | 2.0m from bottom |
| Surface area | 6,400 voxels | ~16 m^2 |
| Total volume | 640,000 voxels | 2.56 MB raw |

The world is a **miniature terrarium** — a 4m x 4m garden bed viewed as a diorama. This scale means individual plants are significant. A single oak dominates a quarter of the garden.

## Terrain

The surface is not flat — it has **rolling hills** generated from overlapping sine waves (amplitude +-0.3m / +-6 voxels around GROUND_LEVEL). This creates:
- Natural water pooling in valleys
- Elevated dry spots on hilltops
- Visual depth even before planting

**Fixed features:**
- **Central spring:** 4x4 water pool at grid center, refilled to 255 every tick
- **Stream bed:** channel from center toward SE edge, 2-3 voxels wide
- **Stone outcrops:** 3-4 rocky boulders near edges (decorative, block roots)

## Voxel Materials

Each voxel is 4 bytes: `[material: u8, water_level: u8, light_level: u8, nutrient_level: u8]`

| Material | ID | Solid | Notes |
|----------|----|-------|-------|
| Air | 0 | No | Empty space |
| Soil | 1 | Yes | Holds water, supports roots, stores nutrients |
| Stone | 2 | Yes | Blocks everything (water, light, roots) |
| Water | 3 | No | Flows down + laterally, evaporates in drought |
| Root | 4 | Yes | Underground water/nutrient absorption |
| Seed | 5 | No | Plant embryo, accumulates growth counter |
| Trunk | 6 | Yes | Main woody structure, moderate light blockage |
| Branch | 7 | Yes | Smaller woody connector |
| Leaf | 8 | No | Foliage, strongest light blockage |
| DeadWood | 9 | Yes | Decomposing wood, becomes nutrient-rich soil |

### Byte Repurposing

The 4-byte voxel format is tight. Some bytes serve double duty:

- **Trunk/Leaf/Branch/Root voxels:** `nutrient_level` stores **species_id** (0-11) for renderer color differentiation
- **Leaf/Branch voxels:** `water_level` stores **tree health** (0-255) for visual stress coloring
- **DeadWood voxels:** `water_level` stores **decay progress** (0-255, converts to soil at 250+)
- **Seed voxels:** `nutrient_level` stores **germination progress** (0-200)

## Design Intent

The small scale is deliberate: every plant matters, every interaction is visible, and the player can trace cause-and-effect across the entire garden. A 4m terrarium forces spatial decisions — you can't plant everything everywhere.

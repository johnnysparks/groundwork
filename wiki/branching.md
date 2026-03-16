# Branching (Space Colonization)

Trees (PlantType::Tree only) use space colonization for organic branching. Other plant types use static templates at all stages.

## Algorithm
1. **Attraction points** scattered in the crown envelope (species-specific shape: Round, Narrow, Wide, Conical)
2. **Branch tips** grow toward nearest attraction points
3. **Phototropism** biases growth toward brightest neighbors (species.phototropism: 0.0-0.6)
4. **Kill distance** (0.3x crown_radius) consumes attraction points near new nodes
5. **Regeneration** adds new points when count drops below 10 (prevents stalling)

## Key Parameters
- Grows every 3 ticks for trees with active skeletons
- Max 6 new nodes per tick
- Influence distance: 2x crown_radius
- Kill distance: 0.3x crown_radius

## Self-Pruning
- Branches accumulate shade_stress when light < species.shade_tolerance
- At shade_stress >= species.prune_threshold: branch dies -> DeadWood
- Dead branches cascade: children of dead branches also die
- Fully decayed branches (shade_stress >= 2x threshold) are removed

## Phototropism Values
| Species | Value | Behavior |
|---------|-------|----------|
| Pine | 0.6 | Strong upward/light-seeking |
| Birch | 0.5 | Moderate |
| Oak | 0.3 | Mild |
| Willow | 0.2 | Weak (follows water instead) |

## Leaf Placement
- Dense spherical leaf shells around alive branch tips
- Radius: Seedling=1, YoungTree=2, Mature/OldGrowth=3
- Leaf voxels store species_id (nutrient_level) and health (water_level)
- **Tuning note:** If canopies look too sparse ("brown sticks with green blobs"), increase leaf_r or the number of attraction points per stage. Current: 60 points (YoungTree), 120 (Mature).

## Rasterization
tree_rasterize converts skeleton -> voxels every 30 ticks when dirty. Clears old footprint, writes new geometry. Trunk inflated to species trunk_radius (tapered with height). Roots tapered with depth.

**Clearing phase:** When re-rasterizing, old footprint voxels are reverted to Soil (underground) or Air (above ground). The surface boundary uses `VoxelGrid::surface_height(x, y)` per-column — not the constant `GROUND_LEVEL` — because terrain has rolling hills. This prevents gaps that would fill with water and cause floating trees.

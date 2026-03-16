# Branching (Space Colonization)

Trees (PlantType::Tree only) use space colonization for organic branching. Other plant types use static templates at all stages.

## Algorithm
1. **Attraction points** scattered in the crown envelope (species-specific shape: Round, Narrow, Wide, Conical). The crown starts well below the trunk top (15-25% of trunk height) so the canopy wraps nearly the full trunk rather than sitting on top like a cap.
2. **Branch stubs** are placed at multiple heights along the trunk during skeleton init, giving space colonization starting points throughout the crown zone — not just the trunk tip.
3. **Branch tips** grow toward nearest attraction points
4. **Phototropism** biases growth toward brightest neighbors (species.phototropism: 0.0-0.6)
5. **Kill distance** (0.3x crown_radius) consumes attraction points near new nodes
6. **Regeneration** adds new points when count drops below 10 (prevents stalling)

## Key Parameters
- Grows every 3 ticks for trees with active skeletons
- Max 10 new nodes per tick
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
- Dense spherical leaf shells around **all branch nodes** (not just tips)
- **Tips** get large spheres: YoungTree = `(crown_r / 3).clamp(4, 8)`, Mature/OldGrowth = `(crown_r / 3).clamp(5, 10)`, Seedling = 1
- **Interior nodes** above crown_start get smaller spheres (2/3 of tip radius, min 2) — fills gaps around the trunk so canopy isn't just green caps at branch ends
- Crown start for interior leaves: YoungTree at 25% trunk height, Mature at 15%
- Oak/willow get lush crowns, birch stays slim — each species has a distinct canopy size
- Leaf voxels store species_id (nutrient_level) and health (water_level)

### Incremental Canopy Growth
When `branch_growth` creates a new branch tip via space colonization, it **immediately generates a leaf sphere** around that tip and pushes the voxels to `pending_voxels`. This means:
- New branches become visible within a few ticks (via `tree_grow_visual` drain) instead of waiting for the next stage transition
- Canopies fill continuously as branches grow, producing dense multi-height foliage
- Only places leaves in Air cells (respects existing geometry)

## Rasterization
tree_rasterize runs when `tree.dirty = true`. Two modes:

1. **Stage change** (`stage_changed = true`): Clears old footprint, writes new geometry from skeleton/template. Trunk inflated to species trunk_radius (tapered with height). Roots tapered with depth.
2. **Health-only** (`stage_changed = false`): Skips footprint clear. Just updates `water_level` (health) and `nutrient_level` (species_id) on existing Leaf/Branch voxels. Prevents visual shape "snapping."

**Material priority:** Leaf and Branch voxels cannot overwrite Trunk voxels. This ensures trunks remain visible even when canopy leaf spheres overlap the trunk column. Leaves can overwrite Air, other Leaves, Branches, and DeadWood.

**Clearing phase (stage change only):** Old footprint voxels are reverted to Soil (underground) or Air (above ground). The surface boundary uses `VoxelGrid::surface_height(x, y)` per-column — not the constant `GROUND_LEVEL` — because terrain has rolling hills. This prevents gaps that would fill with water and cause floating trees.

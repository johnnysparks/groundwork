# Tree Growth

## Growth Stages

| Stage | Threshold | Visual | Vulnerability |
|-------|-----------|--------|---------------|
| Seedling | start | Tiny trunk + root, no leaves | 4x stress damage |
| Sapling | water >= 80, light >= 80 | Starter skeleton: 1/3 height trunk + 3 branch tips + leaf shell (first green!) | 3x stress damage |
| YoungTree | water >= 500, light >= 500 | 2/3 trunk, skeleton branches, early canopy | 2x stress damage |
| Mature | water >= 3000, light >= 3000 | Full height, full crown, seed dispersal | 1x (baseline) |
| OldGrowth | age >= 1200 | Same as Mature but 2x seed dispersal | 1x |
| Dead | health = 0 | DeadWood conversion | Can recover if roots wet |

## Resource Accumulation

Each tick, tree_growth computes:
```
water_intake = sum(root_voxel.water_level for all roots in footprint)
light_intake = sum(above_ground_voxel.light_level for trunk/leaf/branch)

total_boost = nitrogen * canopy * water_affinity * pioneer * bird_symbiosis * seasonal

accumulated_water += sqrt(water_intake) * species.growth_rate * total_boost
accumulated_light += sqrt(light_intake) * species.growth_rate * total_boost
```

The `sqrt()` gives diminishing returns: 100 input -> +10, 10000 -> +100.

## Growth Multipliers (all multiplicative)

| Boost | Condition | Multiplier |
|-------|-----------|------------|
| Nitrogen handshake | 3+ groundcover leaves within 5 voxels of root_pos | 1.5x |
| Canopy effect | shade_tolerance < 60 AND light_intake 5-30 | 1.5x |
| Water affinity | Willow (id=2) AND water_intake > 50 | 2.0x |
| Pioneer vigor | Birch (id=1) AND no trunks within 8 voxels | 1.5x |
| Bird symbiosis | Berry Bush (id=5) AND birds within 12 voxels | 1.5x |
| Seasonal | Day phase: dawn 0.75, day 1.0, dusk 0.75, night 0.5 | 0.5-1.0x |

## Health Dynamics

| Condition | Rate | Notes |
|-----------|------|-------|
| Both water + light OK | +0.02/tick | Fast recovery |
| One resource OK | +0.002/tick | Slow partial recovery |
| Water missing only | -0.005/tick * youth_mult | |
| Light missing only | -0.003 to -0.008/tick * youth_mult | Shade penalty varies by species |
| Both missing | -0.015/tick * youth_mult | Severe stress |
| Pollinator bonus | +0.005/pollinator (max +0.02) | Per nearby bee/butterfly |

**Youth multiplier:** Seedling=4x, Sapling=3x, YoungTree=2x, Mature/OldGrowth=1x

**Death:** health < 0.1 for 20 ticks (seedling/sapling) or 50 ticks (mature) -> Dead stage

## Drought Recovery
Dead trees with root water > 100: health recovers at +0.006/tick. At health 0.3: revives as Sapling with fresh skeleton. Dead trees don't re-rasterize during recovery (preserves root footprint).

## Re-rasterization

Two modes, controlled by the `stage_changed` flag on each Tree:

- **Stage change** (`stage_changed = true`): Full footprint clear + regenerate skeleton. Voxels are queued via `pending_voxels` for gradual placement (see below). Happens on Seedling→Sapling, Sapling→YoungTree, etc.
- **Health-only update** (`stage_changed = false`): Just refreshes `water_level` (health) and `nutrient_level` (species_id) on existing Leaf/Branch voxels. No footprint clear, no shape change. Runs every 30 ticks.

## Smooth Trunk Growth

Stage transitions use **gradual voxel placement** instead of snapping. When a tree transitions to a new stage, the rasterizer:

1. **Roots** are placed immediately (underground, no visual snap needed)
2. **Trunk/branch voxels** are queued in `pending_voxels`, sorted bottom→up
3. **Leaf voxels** are queued after trunk, sorted top→down
4. The `tree_grow_visual` system drains the queue at an adaptive rate: `(queue_len / 4).clamp(3, 200)` voxels per tick

This means trunk grows upward from the ground, then leaves fill in from the crown downward. Small trees (≤12 pending voxels) place 3 voxels/tick. Dense canopies with thousands of queued voxels drain up to 200/tick so they fully appear within the 100-tick pre-tick window at world creation.

**Incremental canopy:** Beyond stage transitions, `branch_growth` also pushes leaf spheres to `pending_voxels` whenever a new branch tip forms. This means canopies fill continuously between stages, not just on transition.

**Template path:** Seedling, Sapling, and Dead stages still use static templates (small enough that gradual growth isn't needed). The smooth growth only applies to skeleton-based stages (YoungTree, Mature, OldGrowth).

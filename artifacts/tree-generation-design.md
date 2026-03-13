# Procedural Tree Generation Design

> GROUNDWORK's core gameplay payoff: plant a seed, shape the environment, watch a tree grow.

## Architecture: Two-Layer Model

Trees exist as **ECS entities** with parametric components (the truth) and are **rasterized into the VoxelGrid** (the view). The grid is the rendering target; tree entities are the simulation state.

```
TreeEntity (bevy_ecs)          VoxelGrid (rendering)
┌──────────────────┐           ┌──────────────────┐
│ species: Oak     │           │ . . . . . . . .  │
│ age: 340 ticks   │  ──────>  │ . . &&&&& . . .  │
│ stage: Mature    │ rasterize │ . . .&|&. . . .  │
│ health: 0.9      │           │ . . . | . . . .  │
│ root_pos: (20,20)│           │ ######|#######.  │
│ rng_seed: 42     │           │ . . .**. . . . . │
│ dirty: false     │           │ . . * . * . . .  │
└──────────────────┘           └──────────────────┘
```

**Why this split:**
- Tree logic is O(num_trees) per tick, not O(108K voxels)
- Re-rasterize only when growth stage changes (every 40-300 ticks)
- Save/load stores ~64 bytes per tree entity, not thousands of voxels
- Enables species breeding, mutation, parametric variation
- Clean separation: sim systems read/write entities, a rasterizer stamps voxels

## New Materials

Add 4 new `Material` variants to `voxel.rs`:

```rust
#[repr(u8)]
pub enum Material {
    Air = 0,
    Soil = 1,
    Stone = 2,
    Water = 3,
    Root = 4,
    Seed = 5,
    Trunk = 6,    // NEW: vertical wood (main stem)
    Branch = 7,   // NEW: lateral wood
    Leaf = 8,     // NEW: canopy foliage (blocks light)
    DeadWood = 9, // NEW: decaying wood
}
```

Stays within u8. The 4-byte voxel struct is unchanged. Later: Flower(10), Fruit(11) for P2.

**Light attenuation values:**
| Material | Attenuation | Rationale |
|----------|------------|-----------|
| Leaf     | 50         | Primary light blocker; creates shade gameplay |
| Trunk    | 30         | Opaque but narrow |
| Branch   | 20         | Thinner than trunk |
| DeadWood | 10         | Sparse, decaying |

**ASCII rendering:**
| Material | 2-char | Color |
|----------|--------|-------|
| Trunk    | `\|\|` | RGB(139, 90, 43) brown |
| Branch   | `/\\` or `--` | RGB(120, 80, 40) lighter brown |
| Leaf     | `&&`   | RGB(60, 160, 40) green |
| DeadWood | `XX`   | RGB(100, 80, 60) gray-brown |

**Connection mask for trunk/branch:** Store 6-bit face connectivity in `nutrient_level` byte of Trunk/Branch voxels. Bit 0-5 = -X, +X, -Y, +Y, -Z, +Z connections. The renderer selects glyphs based on which faces connect: `||` for vertical-only, `--` for horizontal, `+/` for trunk-with-branch-right, etc. This makes branching readable without querying neighbors at render time.

**Growth energy model:** Rather than exactly 1 voxel/tick, trees accumulate growth energy from water + light. Well-fed trees grow in satisfying bursts (2-3 voxels at once). Stressed trees grow slowly. The player sees a direct cause-and-effect: water a tree → watch it suddenly extend.

**Z-slice cross-sections** (scanning top to bottom reveals 3D structure):
```
Z=25 (crown):     . . && . .
Z=24 (upper):     . &&&&&& .
Z=23 (mid crown): &&&&&&&&&&
Z=22 (low crown): &&//||\\&&
Z=21 (branches):  . //||\\ .
Z=20 (trunk):     . . || . .
Z=16 (surface):   . . || . .
Z=15 (roots):     ##**##**##
Z=14 (deep root): ####**####
```

## Tree Entity Components

```rust
/// Species parameters table — defines what a tree CAN become.
#[derive(Clone, Debug)]
pub struct Species {
    pub name: &'static str,
    pub max_height: u8,           // 3-14 voxels above ground
    pub canopy_shape: CanopyShape, // Sphere, Cone, Flat, Columnar, Weeping
    pub trunk_style: TrunkStyle,   // Straight, Bent, Forked
    pub branch_angle: f32,         // 15-80 degrees from vertical
    pub branch_density: u8,        // 1-6 branches per tier
    pub leaf_density: f32,         // 0.3-1.0 fill probability
    pub canopy_radius: u8,         // 1-6 voxels
    pub root_depth: u8,            // 1-8 voxels below ground
    pub root_spread: u8,           // 1-6 voxels lateral
    pub water_need: ResourceNeed,  // Low/Medium/High
    pub light_need: ResourceNeed,  // Low/Medium/High (shade tolerance)
    pub growth_rate: f32,          // 0.5x-2.0x multiplier
    pub seed_dispersal: u8,        // 1-8 voxel radius
}

#[derive(Clone, Copy, Debug)]
pub enum CanopyShape { Sphere, Cone, Flat, Columnar, Weeping, Spreading }

#[derive(Clone, Copy, Debug)]
pub enum TrunkStyle { Straight, Bent, Forked, MultiStem }

#[derive(Clone, Copy, Debug)]
pub enum ResourceNeed { Low, Medium, High }

/// Per-tree instance state — what a tree IS right now.
#[derive(Component)]
pub struct Tree {
    pub species_id: usize,         // index into species table
    pub root_pos: (usize, usize, usize), // where trunk enters ground
    pub age: u32,                  // ticks since planting
    pub stage: GrowthStage,
    pub health: f32,               // 0.0-1.0
    pub accumulated_water: f32,
    pub accumulated_light: f32,
    pub rng_seed: u64,             // deterministic randomness
    pub dirty: bool,               // needs re-rasterization
    pub voxel_footprint: Vec<(usize, usize, usize)>, // cached positions to clear
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrowthStage {
    Seedling,   // 1-2 voxels tall
    Sapling,    // 3-5 tall, first leaves
    YoungTree,  // 5-10 tall, small canopy
    Mature,     // species max, full canopy, produces seeds
    OldGrowth,  // slow decline, DeadWood appears
    Dead,       // only DeadWood remains
}
```

## Growth Stages

| Stage | Voxel Footprint | Ticks | What Happens |
|-------|----------------|-------|-------------|
| **Seed** (existing) | 1 voxel `()` | 40 | nutrient_level counts up. Already implemented. |
| **Seedling** | 1 Trunk + 1-2 Root below | 60-80 | First visible growth. Fragile — can die if dewatered. |
| **Sapling** | 3-5 Trunk, 2-4 Leaf clusters | 100-150 | Species shape hints visible. Roots 2-3 voxels. |
| **Young Tree** | 5-10 tall, small canopy | 200-300 | Shape recognizable. Starts blocking meaningful light. |
| **Mature** | Full species height + canopy | Sustaining | Peak resource use. Produces seeds. The payoff. |
| **Old Growth** | Same + irregular | Slow decline | DeadWood appears. Canopy thins. |

**Real-time pacing** (at 2 ticks/second auto-play):
- Seed to seedling: ~20 seconds
- Seedling to sapling: ~30-40 seconds
- Sapling to young tree: ~50-75 seconds
- Young tree to mature: ~2-3 minutes
- **Total seed-to-mature: ~3-5 minutes** of watching

Stage transitions are gated on **accumulated resources**, not just age. A tree in shade or drought stalls — this IS the gameplay.

## Generation Algorithm

### MVP: Template-Based with Parameterized Distortion

Each growth stage is a small deterministic template stamped from species parameters + rng_seed.

**Trunk generation** — 3D Bresenham line from root_pos upward:
```
fn generate_trunk(species, stage, rng) -> Vec<(offset, Material)>:
    height = stage_height(species, stage)
    for z in 0..height:
        style_offset = trunk_wobble(species.trunk_style, z, rng)
        emit (style_offset.x, style_offset.y, z) as Trunk
```

**Branch generation** — Bresenham lines from trunk at tier heights:
```
fn generate_branches(species, stage, trunk_points, rng) -> Vec<(offset, Material)>:
    for tier_z in branch_tiers(species, stage):
        for i in 0..species.branch_density:
            angle = base_angle(i, species) + noise(rng)
            length = branch_length(species, tier_z, stage)
            walk Bresenham3D from trunk at tier_z outward at angle
            emit each step as Branch
```

**Canopy generation** — layered discs or spheres at branch endpoints:
```
fn generate_canopy(species, branch_endpoints, rng) -> Vec<(offset, Material)>:
    match species.canopy_shape:
        Sphere => sphere_fill(center, radius, species.leaf_density, rng)
        Cone => stacked_discs(top, bottom, taper_rate, density, rng)
        Flat => disc_fill(center, radius, 1-2 layers, density, rng)
        Weeping => sphere_fill + downward tendrils
```

**Root generation** — mirror of branches, growing downward through soil:
```
fn generate_roots(species, stage, root_pos, grid, rng) -> Vec<(offset, Material)>:
    for i in 0..root_count(species, stage):
        direction = downward + lateral_angle(i, rng)
        walk through grid:
            if voxel is Soil → emit as Root
            if voxel is Stone → deflect (thigmotropism)
            if voxel is Water → attracted (hydrotropism)
```

### Upgrade Path: Meristem-Driven Space Colonization (Hybrid)

Combine **space colonization** for natural geometry with **meristem tracking** for tick-by-tick growth and ecological response. Each growing tip is a meristem entity:

```rust
struct Meristem {
    plant: Entity,             // which tree this belongs to
    pos: (usize, usize, usize),
    direction: (i8, i8, i8),   // discretized to -1/0/1 per axis
    vigor: u8,                 // determines growth rate
    is_apical: bool,           // main shoot tip vs lateral branch
    above_ground: bool,
}
```

Each tick, each meristem:
1. Checks vigor (from root water + canopy light). Skip if dormant.
2. Computes growth direction: current direction + phototropism bias + gravitropism bias + space-colonization bias (toward nearest attractor in crown envelope).
3. Extends 1 voxel if target is Air (above) or Soil (below).
4. May spawn lateral meristems at branch probability (inversely weighted by apical dominance).
5. Over-shaded meristems lose vigor → die → branch sheds.

**Why this hybrid**: Space colonization produces natural branching patterns. Meristems give each tip agency to sense light/water. The existing `light_propagation` and `root_water_absorption` systems drive growth decisions directly.

### Long-term: Self-Organizing Growth (Palubicki)

The full ecological model where bud fate is driven by:
- Light availability (from the existing light_propagation system)
- Resource flow through tree structure (pipe model)
- Apical dominance (auxin signaling)

Trees would grow meristem-by-meristem each tick, responding dynamically to environment. This is the "SpeedTree in voxel space" endgame.

## Sim Integration

### New Systems (added to schedule)

```rust
pub fn create_schedule() -> Schedule {
    schedule.add_systems((
        water_flow,
        soil_absorption,
        root_water_absorption,
        light_propagation,     // existing — add Leaf/Trunk/Branch attenuation
        seed_growth,           // existing — modified to spawn TreeEntity
        tree_growth,           // NEW: accumulate resources, advance stages
        tree_rasterize,        // NEW: stamp dirty trees into VoxelGrid
        tick_counter,
    ).chain());
}
```

### `seed_growth` modification

When nutrient_level hits 200, instead of converting to Root:
1. Spawn a new `Tree` entity (Seedling stage)
2. Set the seed voxel to Trunk
3. Place 1-2 Root voxels below

### `tree_growth` system

```rust
fn tree_growth(
    mut trees: Query<&mut Tree>,
    grid: Res<VoxelGrid>,
    species_table: Res<SpeciesTable>,
) {
    for mut tree in trees.iter_mut() {
        let species = &species_table[tree.species_id];
        tree.age += 1;

        // Accumulate resources from root voxels
        let water_intake = count_root_water(&grid, &tree);
        let light_intake = measure_canopy_light(&grid, &tree);
        tree.accumulated_water += water_intake * species.growth_rate;
        tree.accumulated_light += light_intake * species.growth_rate;

        // Check stage transition thresholds
        let next_stage = check_stage_transition(tree, species);
        if next_stage != tree.stage {
            tree.stage = next_stage;
            tree.dirty = true;
        }

        // Health declines without resources
        if water_intake < species.water_need.threshold() {
            tree.health -= 0.01;
        }
        if light_intake < species.light_need.threshold() {
            tree.health -= 0.005;
        }
        tree.health = tree.health.clamp(0.0, 1.0);
    }
}
```

### `tree_rasterize` system

```rust
fn tree_rasterize(
    mut trees: Query<&mut Tree>,
    mut grid: ResMut<VoxelGrid>,
    species_table: Res<SpeciesTable>,
) {
    for mut tree in trees.iter_mut() {
        if !tree.dirty { continue; }

        // Clear old footprint
        for &(x, y, z) in &tree.voxel_footprint {
            if let Some(cell) = grid.get_mut(x, y, z) {
                cell.set_material(Material::Air);
            }
        }

        // Generate new footprint from parameters
        let species = &species_table[tree.species_id];
        let new_voxels = generate_tree(species, &tree, &grid);

        // Stamp into grid
        let mut footprint = Vec::new();
        for (pos, material) in new_voxels {
            let (x, y, z) = pos;
            if let Some(cell) = grid.get_mut(x, y, z) {
                if cell.material == Material::Air || cell.material == Material::Soil {
                    cell.set_material(material);
                    footprint.push(pos);
                }
            }
        }
        tree.voxel_footprint = footprint;
        tree.dirty = false;
    }
}
```

### Light interaction

In `light_propagation`, add attenuation for tree materials:
```rust
Material::Leaf => { light = light.saturating_sub(50); }
Material::Trunk => { light = light.saturating_sub(30); }
Material::Branch => { light = light.saturating_sub(20); }
Material::DeadWood => { light = light.saturating_sub(10); }
```

This naturally creates **shade beneath canopies** — the core competition mechanic.

## Species Archetypes (12 starter species)

| # | Name | Height | Canopy | Trunk | Light | Water | Signature |
|---|------|--------|--------|-------|-------|-------|-----------|
| 1 | Oak | 8 | Sphere | Straight | Medium | Medium | Classic round crown |
| 2 | Pine | 10 | Cone | Straight | High | Low | Tall conical evergreen |
| 3 | Birch | 10 | Columnar | Straight | High | Medium | Thin, tall, airy |
| 4 | Willow | 7 | Weeping | Bent | Medium | High | Drooping near water |
| 5 | Acacia | 6 | Flat | Bent | High | Low | Wide flat canopy |
| 6 | Maple | 9 | Sphere | Forked | Medium | Medium | Dense round, forks |
| 7 | Spruce | 11 | Cone | Straight | Low | Medium | Narrow, shade-tolerant |
| 8 | Elm | 10 | Spreading | Straight | High | Medium | Very wide crown |
| 9 | Shrub | 3 | Sphere | MultiStem | Low | Medium | Dense undergrowth |
| 10 | Fern | 2 | Flat | MultiStem | Low | High | Ground cover, wet |
| 11 | Cactus | 4 | Columnar | Straight | High | Low | Minimal canopy |
| 12 | Vine | 5 | Spreading | Bent | Low | Medium | Climbs stone/trunk |

**Shade tolerance creates layered ecosystems:**
- Canopy layer: Oak, Pine, Elm (need full light)
- Understory: Spruce, Maple (tolerate some shade)
- Ground layer: Shrub, Fern, Vine (thrive in shade)

## Tropisms as Gameplay

The player shapes tree growth by controlling the environment:

| Tropism | Mechanism | Player Action |
|---------|-----------|---------------|
| **Phototropism** | Branches grow toward highest light | Place shade to redirect growth |
| **Gravitropism** | Roots grow down, trunk grows up | Natural behavior |
| **Hydrotropism** | Roots grow toward water | Water strategically to guide roots |
| **Thigmotropism** | Roots deflect around stone, vines climb | Place stone to shape root paths |

## Combined Tropism Scoring

All tropisms combine into a single scoring function per growth point. Each species sets different weights:

```rust
fn score_growth_candidate(
    candidate: (usize, usize, usize),
    direction: Direction,
    grid: &VoxelGrid,
    species: &Species,
    rng: &mut Rng,
) -> f32 {
    let light = grid.get(candidate).map_or(0, |v| v.light_level) as f32;
    let water = grid.get(candidate).map_or(0, |v| v.water_level) as f32;
    let gravity = match direction { Down => 10.0, Lateral => 3.0, Up => 0.0 };
    let surface = count_adjacent_solid(grid, candidate) as f32;

    species.phototropism * light / 255.0
        + species.gravitropism * gravity
        + species.hydrotropism * water / 255.0
        + species.thigmotropism * surface
        + rng.gen_range(0.0..species.randomness)
}
```

An oak has high gravitropism + phototropism. A willow has high hydrotropism. A vine has high thigmotropism. One function, species-specific weights, all growth diversity.

## Competition & Ecology

**Light competition:** Tall trees shade shorter ones. Shade-intolerant species stall or die under canopy. Shade-tolerant species thrive underneath. The player can literally see shadow by pressing J/K to move between z-levels: bright `&&` at canopy height, dim ground below.

**Water competition:** Root systems overlap → trees drain shared water. The existing `root_water_absorption` system handles this naturally — two Root voxels adjacent to the same Soil voxel both absorb, draining it twice as fast. No special competition code needed.

**Nurse trees (emergent):** Birch grows fast, creates moderate shade, has shallow roots. Plant birch first, then oak underneath — the birch shelters the oak through its vulnerable early stages. Remove birch when oak matures. This strategy is never taught; players discover it.

**Seed dispersal:** Mature trees drop seeds within dispersal radius. Seeds land, germinate if conditions are right. Garden becomes self-sustaining — the ultimate payoff.

**Nutrient cycling (P2):** Dead trees → DeadWood → slowly converts to nutrient-rich soil. Creates fertile ground for new growth.

## Ecological Progression

The garden evolves through distinct phases:

| Phase | Ticks | What Happens | Player Feeling |
|-------|-------|-------------|----------------|
| **Colonization** | 0-200 | First seeds sprout. Grass/clover spread. Patchy green. | "Things are growing!" |
| **Establishment** | 200-600 | Pioneer trees (birch) reach sapling. Root systems visible underground. Water dynamics start mattering. | "I see structure forming" |
| **Canopy Formation** | 600-1500 | First trees reach canopy height. Shade appears. Shade-intolerant ground cover dies (dramatic!). Understory species emerge. | "It's sorting itself out" |
| **Climax** | 1500+ | Slow-growing oaks mature. Full vertical structure: canopy → understory → ground cover → roots. Self-sustaining. | "It's alive on its own" |

## Failure Design (Cozy)

**Principle: every death tells its story within a 1-voxel inspection radius.**

| Failure | Visible Cause | Recovery |
|---------|--------------|----------|
| Overwatering | `%%` saturated soil around roots | Stop watering; soil drains naturally |
| Overcrowding | Dark ground level, dry soil between roots | Self-thins; weakest trees die first |
| Light starvation | light_level=0, canopy visible above | Remove obstruction; light returns instantly |
| Drought | Dry `##` soil, no water nearby | Add water; roots resume absorbing |

**No failure is permanent.** Death is slow (warning period via color change). Dead material enriches soil (nutrient boost). Recovery is faster than initial growth (pre-conditioned soil). The garden self-corrects through natural thinning.

## Performance

**At 108K voxels, performance is not a concern.**

- Tree growth: O(num_trees) per tick — check ~50 entities
- Rasterization: O(voxels_per_tree) only when dirty — ~50-200 voxels, ~every 100 ticks
- Light propagation: already O(108K) — adding Leaf attenuation is free
- Water flow: already O(108K) — unchanged

Re-rasterization amortized cost: ~200 voxel writes per tree, once every 100+ ticks. For 20 trees, that's 4000 writes spread over 100 ticks = 40 writes/tick. Negligible.

## Implementation Plan

### Phase 1: Materials & Seedling (P0)
1. Add Trunk, Branch, Leaf, DeadWood to Material enum
2. Update light_propagation with new attenuation values
3. Add ASCII rendering for new materials
4. Modify seed_growth to spawn Tree entity at nutrient_level=200
5. Implement Seedling stage (1 Trunk + 1-2 Roots)
6. Add tree_growth system (resource accumulation only)

### Phase 2: Growth Stages (P0)
1. Implement template-based trunk/branch/canopy generation
2. Wire up stage transitions (Seedling → Sapling → Young → Mature)
3. Add tree_rasterize system with dirty flag
4. First 3-4 species (Oak, Pine, Shrub, Willow)

### Phase 3: Ecology (P1)
1. Root generation with tropisms (hydro, thigmo)
2. Shade competition (trees stalling under canopy)
3. Seed dispersal from mature trees
4. Remaining species
5. Old Growth → Dead → DeadWood decay cycle

### Phase 4: Advanced Generation (P2)
1. Upgrade branching to Space Colonization Algorithm
2. Phototropism (branches respond to light)
3. Self-pruning (shaded branches die)
4. Wind response (directional branch bias)

## Screenshot Moments

The emergent beauty that makes players stop and share:

- **First canopy closure** — two trees' canopies merge into continuous `&&&&&&` with a dark cathedral space below
- **The underground reveal** — player descends below ground for the first time and discovers a complex root network they didn't consciously create
- **Seasonal cascade** — deciduous trees change from green → gold → brown → bare skeleton over 50 ticks, staggered by water stress
- **Vine-covered stone** — plant a vine at a stone wall's base, watch it climb via thigmotropism over 200 ticks
- **Self-sustaining test** — player stops interacting; garden continues thriving for 500+ ticks. "I built something alive."

## Key Sources

- Runions et al. 2007 — [Space Colonization Algorithm](https://algorithmicbotany.org/papers/colonization.egwnp2007.large.pdf)
- Palubicki et al. 2009 — [Self-Organizing Tree Models](https://algorithmicbotany.org/papers/selforg.sig2009.html) (SIGGRAPH)
- Prusinkiewicz & Lindenmayer 1990 — [The Algorithmic Beauty of Plants](https://algorithmicbotany.org/papers/abop/abop.pdf)
- Miguel Cepero / Procedural World — [Voxel Tree Pipeline](http://procworld.blogspot.com/2010/12/voxel-tree.html)
- Dynamic Trees Mod — [Growth pacing and competition](https://modrinth.com/mod/dynamictrees)
- Nauber & Mader 2025 — [Voxel shadow propagation](https://onlinelibrary.wiley.com/doi/10.1111/cgf.15268)
- CRootBox — [Root system simulation with tropisms](https://academic.oup.com/aob/article/121/5/1033/4844040)
- Nick McDonald — [Transport-Oriented Growth](https://nickmcd.me/2020/10/19/transport-oriented-growth-and-procedural-trees/)
- Equilinox, Eco, Idu — ecological game references
- Earthcomputer — [Minecraft tree algorithms](https://gist.github.com/Earthcomputer/41addf80c12d001dfa4391c3a0d03be8)

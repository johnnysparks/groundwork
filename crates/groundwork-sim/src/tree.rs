use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::voxel::Material;

/// Maps seed voxel positions to their parent's species ID.
/// Written by seed_dispersal, read/consumed by seed_growth.
#[derive(Resource, Default)]
pub struct SeedSpeciesMap {
    pub map: HashMap<(usize, usize, usize), usize>,
}

/// Simple deterministic hash for per-tree variation.
pub fn tree_hash(seed: u64, step: u64) -> u64 {
    let mut x = seed.wrapping_add(step).wrapping_mul(6364136223846793005);
    x = x.wrapping_add(1442695040888963407);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Growth stage of a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrowthStage {
    Seedling,
    Sapling,
    YoungTree,
    Mature,
    OldGrowth,
    Dead,
}

impl GrowthStage {
    /// Check if this stage should transition, returning the next stage if so.
    pub fn next_stage(
        &self,
        age: u32,
        accumulated_water: f32,
        accumulated_light: f32,
        health: f32,
    ) -> Option<GrowthStage> {
        if health <= 0.0 && *self != GrowthStage::Dead {
            return Some(GrowthStage::Dead);
        }
        match self {
            GrowthStage::Seedling => {
                // Fast first transition: seedling → sapling in ~8 ticks after germination.
                // The sapling template has visible leaves — this is the "first sprout" moment.
                // Previously 200 → took ~20 ticks. At 80, takes ~8 ticks.
                if accumulated_water >= 80.0 && accumulated_light >= 80.0 {
                    Some(GrowthStage::Sapling)
                } else {
                    None
                }
            }
            GrowthStage::Sapling => {
                // Sapling → YoungTree: first significant canopy. Target: ~40 ticks after sapling.
                // Previously 800. At 500, takes ~35-40 ticks with typical resource rates.
                if accumulated_water >= 500.0 && accumulated_light >= 500.0 {
                    Some(GrowthStage::YoungTree)
                } else {
                    None
                }
            }
            GrowthStage::YoungTree => {
                if accumulated_water >= 3000.0 && accumulated_light >= 3000.0 {
                    Some(GrowthStage::Mature)
                } else {
                    None
                }
            }
            GrowthStage::Mature => {
                if age >= 1200 {
                    Some(GrowthStage::OldGrowth)
                } else {
                    None
                }
            }
            GrowthStage::OldGrowth | GrowthStage::Dead => None,
        }
    }
}

/// Resource need level for a species parameter.
#[derive(Clone, Copy, Debug)]
pub enum ResourceNeed {
    Low,
    Medium,
    High,
}

impl ResourceNeed {
    pub fn threshold(self) -> f32 {
        match self {
            Self::Low => 10.0,
            Self::Medium => 30.0,
            Self::High => 60.0,
        }
    }
}

/// Crown shape determines canopy generation pattern.
#[derive(Clone, Copy, Debug)]
pub enum CrownShape {
    /// Wide in middle, tapers top and bottom (Oak).
    Round,
    /// Slim column of leaves (Birch).
    Narrow,
    /// Wider than tall, drooping (Willow).
    Wide,
    /// Wide at bottom, tapers to a point (Pine).
    Conical,
}

/// What kind of plant this species is. Controls growth behavior:
/// - Trees use space colonization branching at YoungTree+ stages.
/// - All other types use templates at every stage (no skeleton).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlantType {
    /// Tall woody plant with trunk and canopy. Uses space colonization.
    Tree,
    /// Woody plant, 1-3m. Bushy shape, no tall trunk. Template-only.
    Shrub,
    /// Low-growing plant, <0.5m. Spreads laterally. Template-only.
    Groundcover,
    /// Herbaceous plant, 0.5-1.5m. Thin stem + bloom. Template-only.
    Flower,
}

impl PlantType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Tree => "Tree",
            Self::Shrub => "Shrub",
            Self::Groundcover => "Ground",
            Self::Flower => "Flower",
        }
    }
}

/// Species parameters — defines what a plant CAN become.
/// Dimensions are in meters; use accessor methods for voxel units.
#[derive(Clone, Debug)]
pub struct Species {
    pub name: &'static str,
    pub plant_type: PlantType,
    pub max_height_m: f64,
    pub root_depth_m: f64,
    pub crown_radius_m: f64,
    /// Trunk radius in meters. Trees get multi-voxel-wide trunks.
    pub trunk_radius_m: f64,
    pub crown_shape: CrownShape,
    pub water_need: ResourceNeed,
    pub light_need: ResourceNeed,
    pub growth_rate: f32,
    /// How strongly branches bend toward light (0.0 = none, 1.0 = strong).
    /// Only used for Tree plant types.
    pub phototropism: f32,
    /// Light level below which branches accumulate shade stress (0-255).
    pub shade_tolerance: u8,
    /// Shade stress ticks before a branch dies and converts to DeadWood.
    pub prune_threshold: u16,
    /// Dispersal distance in meters. Trees disperse far, groundcover spreads nearby.
    pub dispersal_distance_m: f64,
    /// Ticks between seed dispersal attempts. Lower = faster spread.
    pub dispersal_period: u32,
}

impl Species {
    /// Max height in voxel units.
    pub fn max_height(&self) -> u8 {
        crate::scale::meters_to_voxels(self.max_height_m) as u8
    }
    /// Root depth in voxel units.
    pub fn root_depth(&self) -> u8 {
        crate::scale::meters_to_voxels(self.root_depth_m) as u8
    }
    /// Crown radius in voxel units.
    pub fn crown_radius(&self) -> u8 {
        crate::scale::meters_to_voxels(self.crown_radius_m) as u8
    }
    /// Trunk radius in voxel units.
    pub fn trunk_radius(&self) -> u8 {
        crate::scale::meters_to_voxels(self.trunk_radius_m) as u8
    }
    /// Whether this species uses space colonization branching.
    pub fn uses_skeleton(&self) -> bool {
        self.plant_type == PlantType::Tree
    }
}

/// Look up a species by name (case-insensitive, ignores spaces/hyphens/underscores).
/// Returns the species index in the default SpeciesTable.
pub fn species_name_to_id(name: &str) -> Option<usize> {
    let norm = |s: &str| -> String { s.to_ascii_lowercase().replace([' ', '-', '_'], "") };
    let needle = norm(name);
    let table = SpeciesTable::default();
    table.species.iter().position(|s| norm(s.name) == needle)
}

/// Table of all species, stored as an ECS resource.
#[derive(Resource)]
pub struct SpeciesTable {
    pub species: Vec<Species>,
}

impl Default for SpeciesTable {
    fn default() -> Self {
        Self {
            species: vec![
                // --- Trees (indices 0-3) ---
                // Glen scale (4m×4m×5m): trees fill 40-50 of 60 voxels above ground.
                // At 0.05m/voxel, oak trunk_radius 0.15m = 3 voxels → 7-voxel-wide trunk.
                Species {
                    name: "Oak",
                    plant_type: PlantType::Tree,
                    max_height_m: 2.5,
                    root_depth_m: 1.5,
                    crown_radius_m: 1.2,
                    trunk_radius_m: 0.15,
                    crown_shape: CrownShape::Round,
                    water_need: ResourceNeed::Medium,
                    light_need: ResourceNeed::Medium,
                    growth_rate: 1.0,
                    phototropism: 0.3,
                    shade_tolerance: 80,
                    prune_threshold: 200,
                    dispersal_distance_m: 1.5,
                    dispersal_period: 100,
                },
                Species {
                    name: "Birch",
                    plant_type: PlantType::Tree,
                    max_height_m: 2.0,
                    root_depth_m: 1.0,
                    crown_radius_m: 0.8,
                    trunk_radius_m: 0.08,
                    crown_shape: CrownShape::Narrow,
                    water_need: ResourceNeed::Low,
                    light_need: ResourceNeed::Medium,
                    growth_rate: 1.5,
                    phototropism: 0.5,
                    shade_tolerance: 120,
                    prune_threshold: 100,
                    dispersal_distance_m: 1.5,
                    dispersal_period: 80,
                },
                Species {
                    name: "Willow",
                    plant_type: PlantType::Tree,
                    max_height_m: 1.8,
                    root_depth_m: 1.2,
                    crown_radius_m: 1.5,
                    trunk_radius_m: 0.1,
                    crown_shape: CrownShape::Wide,
                    water_need: ResourceNeed::High,
                    light_need: ResourceNeed::Low,
                    growth_rate: 0.8,
                    phototropism: 0.2,
                    shade_tolerance: 60,
                    prune_threshold: 300,
                    dispersal_distance_m: 1.0,
                    dispersal_period: 120,
                },
                Species {
                    name: "Pine",
                    plant_type: PlantType::Tree,
                    max_height_m: 2.8,
                    root_depth_m: 1.5,
                    crown_radius_m: 0.8,
                    trunk_radius_m: 0.1,
                    crown_shape: CrownShape::Conical,
                    water_need: ResourceNeed::Low,
                    light_need: ResourceNeed::High,
                    growth_rate: 0.7,
                    phototropism: 0.6,
                    shade_tolerance: 100,
                    prune_threshold: 150,
                    dispersal_distance_m: 1.5,
                    dispersal_period: 90,
                },
                // --- Shrubs (indices 4-6) ---
                Species {
                    name: "Fern",
                    plant_type: PlantType::Shrub,
                    max_height_m: 0.4,
                    root_depth_m: 0.3,
                    crown_radius_m: 0.3,
                    trunk_radius_m: 0.03,
                    crown_shape: CrownShape::Wide,
                    water_need: ResourceNeed::High,
                    light_need: ResourceNeed::Low,
                    growth_rate: 1.5,
                    phototropism: 0.0,
                    shade_tolerance: 30,
                    prune_threshold: 500,
                    dispersal_distance_m: 0.5,
                    dispersal_period: 60,
                },
                Species {
                    name: "Berry Bush",
                    plant_type: PlantType::Shrub,
                    max_height_m: 0.6,
                    root_depth_m: 0.4,
                    crown_radius_m: 0.4,
                    trunk_radius_m: 0.04,
                    crown_shape: CrownShape::Round,
                    water_need: ResourceNeed::Medium,
                    light_need: ResourceNeed::Medium,
                    growth_rate: 1.2,
                    phototropism: 0.0,
                    shade_tolerance: 80,
                    prune_threshold: 300,
                    dispersal_distance_m: 0.8,
                    dispersal_period: 80,
                },
                Species {
                    name: "Holly",
                    plant_type: PlantType::Shrub,
                    max_height_m: 0.8,
                    root_depth_m: 0.5,
                    crown_radius_m: 0.4,
                    trunk_radius_m: 0.05,
                    crown_shape: CrownShape::Conical,
                    water_need: ResourceNeed::Low,
                    light_need: ResourceNeed::Low,
                    growth_rate: 0.6,
                    phototropism: 0.0,
                    shade_tolerance: 40,
                    prune_threshold: 400,
                    dispersal_distance_m: 0.6,
                    dispersal_period: 100,
                },
                // --- Flowers (indices 7-8) ---
                Species {
                    name: "Wildflower",
                    plant_type: PlantType::Flower,
                    max_height_m: 0.25,
                    root_depth_m: 0.15,
                    crown_radius_m: 0.15,
                    trunk_radius_m: 0.02,
                    crown_shape: CrownShape::Round,
                    water_need: ResourceNeed::Medium,
                    light_need: ResourceNeed::High,
                    growth_rate: 2.0,
                    phototropism: 0.0,
                    shade_tolerance: 150,
                    prune_threshold: 100,
                    dispersal_distance_m: 0.5,
                    dispersal_period: 40,
                },
                Species {
                    name: "Daisy",
                    plant_type: PlantType::Flower,
                    max_height_m: 0.15,
                    root_depth_m: 0.1,
                    crown_radius_m: 0.15,
                    trunk_radius_m: 0.02,
                    crown_shape: CrownShape::Narrow,
                    water_need: ResourceNeed::Low,
                    light_need: ResourceNeed::High,
                    growth_rate: 2.5,
                    phototropism: 0.0,
                    shade_tolerance: 160,
                    prune_threshold: 80,
                    dispersal_distance_m: 0.4,
                    dispersal_period: 30,
                },
                // --- Groundcover (indices 9-11) ---
                Species {
                    name: "Moss",
                    plant_type: PlantType::Groundcover,
                    max_height_m: 0.05,
                    root_depth_m: 0.1,
                    crown_radius_m: 0.3,
                    trunk_radius_m: 0.0,
                    crown_shape: CrownShape::Wide,
                    water_need: ResourceNeed::High,
                    light_need: ResourceNeed::Low,
                    growth_rate: 1.0,
                    phototropism: 0.0,
                    shade_tolerance: 20,
                    prune_threshold: 500,
                    dispersal_distance_m: 0.3,
                    dispersal_period: 25,
                },
                Species {
                    name: "Grass",
                    plant_type: PlantType::Groundcover,
                    max_height_m: 0.1,
                    root_depth_m: 0.1,
                    crown_radius_m: 0.2,
                    trunk_radius_m: 0.0,
                    crown_shape: CrownShape::Narrow,
                    water_need: ResourceNeed::Low,
                    light_need: ResourceNeed::High,
                    growth_rate: 2.0,
                    phototropism: 0.0,
                    shade_tolerance: 140,
                    prune_threshold: 100,
                    dispersal_distance_m: 0.3,
                    dispersal_period: 20,
                },
                Species {
                    name: "Clover",
                    plant_type: PlantType::Groundcover,
                    max_height_m: 0.1,
                    root_depth_m: 0.1,
                    crown_radius_m: 0.2,
                    trunk_radius_m: 0.0,
                    crown_shape: CrownShape::Round,
                    water_need: ResourceNeed::Medium,
                    light_need: ResourceNeed::Medium,
                    growth_rate: 1.8,
                    phototropism: 0.0,
                    shade_tolerance: 100,
                    prune_threshold: 150,
                    dispersal_distance_m: 0.3,
                    dispersal_period: 30,
                },
            ],
        }
    }
}

/// A single node in the branch skeleton for space colonization.
/// Stored in Tree::branches as a flat Vec indexed by u16.
#[derive(Clone, Debug)]
pub struct BranchNode {
    /// Voxel position relative to tree root_pos.
    pub pos: (isize, isize, isize),
    /// Index of parent node in the Vec, or u16::MAX for the trunk base.
    pub parent: u16,
    /// Material to rasterize: Trunk, Branch, or Leaf.
    pub material: Material,
    /// Accumulated low-light ticks for self-pruning.
    pub shade_stress: u16,
    /// Whether this node is alive (false = pruned, rendered as DeadWood then removed).
    pub alive: bool,
}

/// Per-tree instance state — what a tree IS right now.
#[derive(Component)]
pub struct Tree {
    pub species_id: usize,
    pub root_pos: (usize, usize, usize),
    pub age: u32,
    pub stage: GrowthStage,
    pub health: f32,
    pub accumulated_water: f32,
    pub accumulated_light: f32,
    pub rng_seed: u64,
    pub dirty: bool,
    pub voxel_footprint: Vec<(usize, usize, usize)>,
    /// Branch skeleton for space colonization (used for YoungTree+ stages).
    pub branches: Vec<BranchNode>,
    /// Attraction points that guide branch growth (relative coords from root_pos).
    pub attraction_points: Vec<(isize, isize, isize)>,
    /// Whether the skeleton has been initialized for the current stage.
    pub skeleton_initialized: bool,
}

/// Generated template: list of (dx, dy, dz, Material) offsets from root_pos.
pub struct TreeTemplate {
    pub voxels: Vec<(isize, isize, isize, Material)>,
}

impl TreeTemplate {
    /// Generate a voxel template for the given species and growth stage.
    pub fn generate(species: &Species, stage: &GrowthStage, rng_seed: u64) -> Self {
        match species.plant_type {
            PlantType::Tree => match stage {
                GrowthStage::Seedling => Self::seedling(species),
                GrowthStage::Sapling => Self::sapling(species, rng_seed),
                GrowthStage::YoungTree => Self::young_tree(species, rng_seed),
                GrowthStage::Mature | GrowthStage::OldGrowth => Self::mature(species, rng_seed),
                GrowthStage::Dead => Self::dead(species, rng_seed),
            },
            PlantType::Shrub => match stage {
                GrowthStage::Seedling => Self::seedling(species),
                GrowthStage::Dead => Self::dead_small(species),
                _ => Self::shrub(species, stage, rng_seed),
            },
            PlantType::Flower => match stage {
                GrowthStage::Seedling => Self::seedling(species),
                GrowthStage::Dead => Self::dead_small(species),
                _ => Self::flower(species, stage, rng_seed),
            },
            PlantType::Groundcover => match stage {
                GrowthStage::Seedling => Self::seedling_ground(),
                GrowthStage::Dead => Self::dead_small(species),
                _ => Self::groundcover(species, stage, rng_seed),
            },
        }
    }

    fn seedling(_species: &Species) -> Self {
        let voxels = vec![(0, 0, 0, Material::Trunk), (0, 0, -1, Material::Root)];
        Self { voxels }
    }

    /// Fill a cylindrical trunk column of the given radius.
    fn add_trunk_column(
        voxels: &mut Vec<(isize, isize, isize, Material)>,
        z_start: isize,
        z_end: isize,
        radius: u8,
        material: Material,
    ) {
        let r = radius as isize;
        let r_sq = r * r;
        for z in z_start..z_end {
            if r == 0 {
                voxels.push((0, 0, z, material));
            } else {
                for dx in -r..=r {
                    for dy in -r..=r {
                        if dx * dx + dy * dy <= r_sq {
                            voxels.push((dx, dy, z, material));
                        }
                    }
                }
            }
        }
    }

    /// Fill a root column with lateral spread at depth.
    fn add_root_column(
        voxels: &mut Vec<(isize, isize, isize, Material)>,
        depth: isize,
        trunk_r: u8,
        rng_seed: u64,
    ) {
        let r = trunk_r.max(1) as isize;
        let r_sq = r * r;
        for z in 1..=depth {
            // Central root column at trunk radius (tapers with depth)
            let taper = (r - z / 4).max(0);
            let t_sq = taper * taper;
            for dx in -taper..=taper {
                for dy in -taper..=taper {
                    if dx * dx + dy * dy <= t_sq {
                        voxels.push((dx, dy, -z, Material::Root));
                    }
                }
            }
            // Lateral root spread at deeper levels
            if z >= r_sq.max(2) {
                for step in 0..3u64 {
                    let h = tree_hash(rng_seed, z as u64 * 10 + step);
                    let spread = (z / 2).max(1);
                    let dx = (h % (spread as u64 * 2 + 1)) as isize - spread;
                    let dy = ((h >> 8) % (spread as u64 * 2 + 1)) as isize - spread;
                    if dx != 0 || dy != 0 {
                        voxels.push((dx, dy, -z, Material::Root));
                    }
                }
            }
        }
    }

    fn sapling(species: &Species, rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let trunk_h = (species.max_height() / 3).max(2) as isize;
        // Sapling trunk is half adult radius (at least 1)
        let trunk_r = (species.trunk_radius() / 2).max(1);

        // Trunk
        Self::add_trunk_column(&mut voxels, 0, trunk_h, trunk_r, Material::Trunk);

        // Small leaf cap
        let leaf_r = (species.crown_radius() / 3).max(1);
        Self::add_leaf_disc(&mut voxels, 0, 0, trunk_h, leaf_r, rng_seed, 0);

        // Roots
        let root_d = (species.root_depth() / 2).max(1) as isize;
        Self::add_root_column(&mut voxels, root_d, trunk_r, rng_seed);

        Self { voxels }
    }

    fn young_tree(species: &Species, rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let trunk_h = (species.max_height() * 2 / 3).max(3) as isize;
        let crown_r = species.crown_radius().div_ceil(2).max(1);
        // Young tree trunk is 2/3 adult radius
        let trunk_r = (species.trunk_radius() * 2 / 3).max(1);

        // Trunk
        Self::add_trunk_column(&mut voxels, 0, trunk_h, trunk_r, Material::Trunk);

        // Crown (leaves first so branches can overwrite where they overlap)
        Self::add_crown(
            &mut voxels,
            trunk_h,
            crown_r,
            &species.crown_shape,
            rng_seed,
        );

        // Branches near the top — reach scales with crown radius
        let branch_z = trunk_h - 2;
        let branch_reach = (crown_r / 2).max(1) as isize;
        if branch_z > 0 {
            Self::add_branches(&mut voxels, branch_z, branch_reach, rng_seed, 0);
        }

        // Roots
        let root_d = (species.root_depth() * 2 / 3).max(2) as isize;
        Self::add_root_column(&mut voxels, root_d, trunk_r, rng_seed);

        Self { voxels }
    }

    fn mature(species: &Species, rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let trunk_h = species.max_height() as isize;
        let crown_r = species.crown_radius();
        let trunk_r = species.trunk_radius().max(1);

        // Trunk — full width
        Self::add_trunk_column(&mut voxels, 0, trunk_h, trunk_r, Material::Trunk);

        // Crown (leaves first)
        Self::add_crown(
            &mut voxels,
            trunk_h,
            crown_r,
            &species.crown_shape,
            rng_seed,
        );

        // Branches at multiple levels — reach scales with crown
        let branch_start = trunk_h / 2;
        let branch_spacing = (trunk_h / 6).max(2);
        for i in 0..3 {
            let bz = branch_start + i * branch_spacing;
            if bz < trunk_h {
                let reach = if i == 1 {
                    (crown_r / 2).max(2) as isize
                } else {
                    (crown_r / 3).max(1) as isize
                };
                Self::add_branches(&mut voxels, bz, reach, rng_seed, i as u64);
            }
        }

        // Full roots with lateral spread
        let root_d = species.root_depth() as isize;
        Self::add_root_column(&mut voxels, root_d, trunk_r, rng_seed);

        Self { voxels }
    }

    fn dead(species: &Species, rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let trunk_h = (species.max_height() / 2).max(1) as isize;
        let trunk_r = (species.trunk_radius() * 2 / 3).max(1);

        Self::add_trunk_column(&mut voxels, 0, trunk_h, trunk_r, Material::DeadWood);

        // A dead branch or two
        if trunk_h > 2 {
            let h = tree_hash(rng_seed, 0);
            let dx: isize = if h.is_multiple_of(2) { 1 } else { -1 };
            let reach = (species.crown_radius() / 3).max(1) as isize;
            for r in 1..=reach {
                voxels.push((dx * r, 0, trunk_h - 1, Material::DeadWood));
            }
        }

        // Remaining roots
        let root_d = (species.root_depth() / 2).max(1) as isize;
        Self::add_root_column(&mut voxels, root_d, trunk_r, rng_seed);

        Self { voxels }
    }

    // --- Small plant templates ---

    /// Groundcover seedling: just a single leaf at ground level.
    fn seedling_ground() -> Self {
        Self {
            voxels: vec![(0, 0, 0, Material::Leaf), (0, 0, -1, Material::Root)],
        }
    }

    /// Dead small plant: single deadwood voxel.
    fn dead_small(species: &Species) -> Self {
        let mut voxels = vec![(0, 0, 0, Material::DeadWood)];
        if species.root_depth() >= 1 {
            voxels.push((0, 0, -1, Material::Root));
        }
        Self { voxels }
    }

    /// Shrub template: short woody base with bushy leaf crown.
    fn shrub(species: &Species, stage: &GrowthStage, rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let max_h = species.max_height().max(1) as isize;
        let crown_r = species.crown_radius().max(1);

        // Scale by growth stage
        let (trunk_h, cr, root_d) = match stage {
            GrowthStage::Sapling => (1isize, 1u8, 1isize),
            GrowthStage::YoungTree => ((max_h / 2).max(1), (crown_r / 2).max(1), 1),
            _ => (max_h.max(1), crown_r, species.root_depth().max(1) as isize),
        };

        // Short woody trunk (Branch material for shrubs)
        for z in 0..trunk_h {
            voxels.push((0, 0, z, Material::Branch));
        }

        // Bushy crown — use the species crown shape
        if cr > 0 {
            Self::add_crown(&mut voxels, trunk_h, cr, &species.crown_shape, rng_seed);
        }

        // Shallow roots
        for z in 1..=root_d {
            voxels.push((0, 0, -z, Material::Root));
        }

        Self { voxels }
    }

    /// Flower template: thin stem with leaf bloom at top.
    fn flower(species: &Species, stage: &GrowthStage, _rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let max_h = species.max_height().max(1) as isize;

        let (stem_h, has_bloom) = match stage {
            GrowthStage::Sapling => (1isize, false),
            GrowthStage::YoungTree => ((max_h / 2).max(1), true),
            _ => (max_h, true),
        };

        // Thin stem (trunk)
        for z in 0..stem_h {
            voxels.push((0, 0, z, Material::Trunk));
        }

        // Bloom at top (leaf voxel)
        if has_bloom {
            voxels.push((0, 0, stem_h, Material::Leaf));
        }

        // Shallow root
        voxels.push((0, 0, -1, Material::Root));

        Self { voxels }
    }

    /// Groundcover template: flat spread of leaves at surface level.
    fn groundcover(species: &Species, stage: &GrowthStage, rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let max_r = species.crown_radius().max(1) as isize;

        let r = match stage {
            GrowthStage::Sapling => 0isize,
            GrowthStage::YoungTree => (max_r / 2).max(0),
            _ => max_r,
        };

        // Flat carpet of leaves at z=0
        if r == 0 {
            voxels.push((0, 0, 0, Material::Leaf));
        } else {
            let r_sq = r * r;
            for dx in -r..=r {
                for dy in -r..=r {
                    if dx * dx + dy * dy <= r_sq {
                        // Use hash to create natural-looking gaps
                        let h = tree_hash(rng_seed, (dx + 100) as u64 * 200 + (dy + 100) as u64);
                        if !h.is_multiple_of(4) {
                            // ~75% coverage for natural look
                            voxels.push((dx, dy, 0, Material::Leaf));
                        }
                    }
                }
            }
        }

        // Single root below center
        voxels.push((0, 0, -1, Material::Root));

        Self { voxels }
    }

    /// Add a filled disc of leaf voxels at the given center and radius.
    fn add_leaf_disc(
        voxels: &mut Vec<(isize, isize, isize, Material)>,
        cx: isize,
        cy: isize,
        z: isize,
        radius: u8,
        _rng_seed: u64,
        _layer: u64,
    ) {
        let r = radius as isize;
        let r_sq = r * r;
        for dx in -r..=r {
            for dy in -r..=r {
                if dx * dx + dy * dy <= r_sq {
                    voxels.push((cx + dx, cy + dy, z, Material::Leaf));
                }
            }
        }
    }

    /// Add branches extending from the trunk at height z in 2 cardinal directions.
    fn add_branches(
        voxels: &mut Vec<(isize, isize, isize, Material)>,
        z: isize,
        reach: isize,
        rng_seed: u64,
        step: u64,
    ) {
        let h = tree_hash(rng_seed, step * 100 + z as u64);
        let dirs: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let d1 = (h % 4) as usize;
        let d2 = ((h >> 4) % 4) as usize;

        for r in 1..=reach {
            let (dx, dy) = dirs[d1];
            voxels.push((dx * r, dy * r, z, Material::Branch));
        }
        if d2 != d1 {
            for r in 1..=reach {
                let (dx, dy) = dirs[d2];
                voxels.push((dx * r, dy * r, z, Material::Branch));
            }
        }
    }

    /// Generate a multi-layer crown based on species shape.
    fn add_crown(
        voxels: &mut Vec<(isize, isize, isize, Material)>,
        trunk_top: isize,
        max_radius: u8,
        shape: &CrownShape,
        rng_seed: u64,
    ) {
        let mr = max_radius as isize;
        match shape {
            CrownShape::Round => {
                let layers = mr + 1;
                let mid = layers / 2;
                for i in 0..layers {
                    let z = trunk_top + i;
                    let dist = (i - mid).abs();
                    let r = (mr - dist).max(1) as u8;
                    Self::add_leaf_disc(voxels, 0, 0, z, r, rng_seed, i as u64);
                }
            }
            CrownShape::Narrow => {
                let layers = mr + 2;
                for i in 0..layers {
                    let z = trunk_top + i;
                    let r = (mr / 2).max(1) as u8;
                    Self::add_leaf_disc(voxels, 0, 0, z, r, rng_seed, i as u64);
                }
            }
            CrownShape::Wide => {
                let layers = (mr / 2 + 1).max(2);
                for i in 0..layers {
                    let z = trunk_top + i;
                    let r = if i == 0 || i == layers - 1 {
                        (mr - 1).max(1) as u8
                    } else {
                        mr as u8
                    };
                    Self::add_leaf_disc(voxels, 0, 0, z, r, rng_seed, i as u64);
                }
            }
            CrownShape::Conical => {
                let layers = mr + 1;
                let denom = if layers > 1 { layers - 1 } else { 1 };
                for i in 0..layers {
                    let z = trunk_top + i;
                    let r = (mr * (denom - i) / denom).max(1) as u8;
                    Self::add_leaf_disc(voxels, 0, 0, z, r, rng_seed, i as u64);
                }
            }
        }
    }
}

/// Generate attraction points within a species-shaped crown envelope.
/// Points are in relative coordinates from root_pos.
pub fn generate_attraction_points(
    species: &Species,
    stage: &GrowthStage,
    rng_seed: u64,
) -> Vec<(isize, isize, isize)> {
    let trunk_h = match stage {
        GrowthStage::YoungTree => (species.max_height() * 2 / 3).max(3) as isize,
        _ => species.max_height() as isize,
    };
    let cr = match stage {
        GrowthStage::YoungTree => species.crown_radius().div_ceil(2).max(1) as isize,
        _ => species.crown_radius() as isize,
    };

    // Number of attraction points scales with crown volume.
    // Dense sampling fills the crown envelope so branches reach all regions.
    let num_points = match stage {
        GrowthStage::YoungTree => 60,
        GrowthStage::Mature => 120,
        GrowthStage::OldGrowth => 160,
        _ => return vec![],
    };

    let mut points = Vec::with_capacity(num_points);
    for i in 0..num_points * 3 {
        // Over-sample and filter to shape
        let h = tree_hash(rng_seed, 5000 + i as u64);
        let h2 = tree_hash(rng_seed, 6000 + i as u64);
        let h3 = tree_hash(rng_seed, 7000 + i as u64);

        // Random offset within bounding box
        let dx = (h % (cr as u64 * 2 + 1)) as isize - cr;
        let dy = (h2 % (cr as u64 * 2 + 1)) as isize - cr;
        let crown_height = match species.crown_shape {
            CrownShape::Round => cr + 1,
            CrownShape::Narrow => cr + 2,
            CrownShape::Wide => (cr / 2 + 1).max(2),
            CrownShape::Conical => cr + 1,
        };
        let dz = (h3 % crown_height as u64) as isize;

        let z = trunk_h + dz;

        // Filter by crown shape envelope
        let inside = match species.crown_shape {
            CrownShape::Round => {
                let mid = crown_height / 2;
                let dist_from_mid = (dz - mid).abs();
                let allowed_r = (cr - dist_from_mid).max(0);
                dx * dx + dy * dy <= allowed_r * allowed_r
            }
            CrownShape::Narrow => {
                let nr = (cr / 2).max(1);
                dx * dx + dy * dy <= nr * nr
            }
            CrownShape::Wide => dx * dx + dy * dy <= cr * cr,
            CrownShape::Conical => {
                let denom = if crown_height > 1 {
                    crown_height - 1
                } else {
                    1
                };
                let allowed_r = (cr * (denom - dz) / denom).max(0);
                dx * dx + dy * dy <= allowed_r * allowed_r
            }
        };

        if inside && z > 0 {
            points.push((dx, dy, z));
            if points.len() >= num_points {
                break;
            }
        }
    }
    points
}

/// Initialize a branch skeleton with trunk nodes for a tree transitioning to
/// a branching stage. Returns (branches, attraction_points).
pub fn init_skeleton(
    species: &Species,
    stage: &GrowthStage,
    rng_seed: u64,
) -> (Vec<BranchNode>, Vec<(isize, isize, isize)>) {
    let trunk_h = match stage {
        GrowthStage::YoungTree => (species.max_height() * 2 / 3).max(3) as isize,
        _ => species.max_height() as isize,
    };

    let mut branches = Vec::new();

    // Trunk nodes
    for z in 0..trunk_h {
        branches.push(BranchNode {
            pos: (0, 0, z),
            parent: if z == 0 { u16::MAX } else { (z - 1) as u16 },
            material: Material::Trunk,
            shade_stress: 0,
            alive: true,
        });
    }

    // Roots
    let root_d = match stage {
        GrowthStage::YoungTree => (species.root_depth() * 2 / 3).max(2) as isize,
        _ => species.root_depth() as isize,
    };
    for z in 1..=root_d {
        branches.push(BranchNode {
            pos: (0, 0, -z),
            parent: 0,
            material: Material::Root,
            shade_stress: 0,
            alive: true,
        });
    }

    let points = generate_attraction_points(species, stage, rng_seed);
    (branches, points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seedling_template_has_trunk_and_root() {
        let species = &SpeciesTable::default().species[0]; // Oak
        let t = TreeTemplate::generate(species, &GrowthStage::Seedling, 42);
        assert!(t.voxels.iter().any(|v| v.3 == Material::Trunk));
        assert!(t.voxels.iter().any(|v| v.3 == Material::Root));
    }

    #[test]
    fn sapling_taller_than_seedling() {
        let species = &SpeciesTable::default().species[0];
        let seedling = TreeTemplate::generate(species, &GrowthStage::Seedling, 42);
        let sapling = TreeTemplate::generate(species, &GrowthStage::Sapling, 42);
        let max_z = |t: &TreeTemplate| t.voxels.iter().map(|v| v.2).max().unwrap();
        assert!(max_z(&sapling) > max_z(&seedling));
    }

    #[test]
    fn mature_has_leaves_and_branches() {
        let species = &SpeciesTable::default().species[0];
        let t = TreeTemplate::generate(species, &GrowthStage::Mature, 42);
        assert!(t.voxels.iter().any(|v| v.3 == Material::Leaf));
        assert!(t.voxels.iter().any(|v| v.3 == Material::Branch));
    }

    #[test]
    fn dead_uses_deadwood() {
        let species = &SpeciesTable::default().species[0];
        let t = TreeTemplate::generate(species, &GrowthStage::Dead, 42);
        assert!(t.voxels.iter().any(|v| v.3 == Material::DeadWood));
        assert!(!t.voxels.iter().any(|v| v.3 == Material::Leaf));
    }

    #[test]
    fn stage_transitions_in_order() {
        // Seedling → Sapling (threshold: 80)
        assert_eq!(
            GrowthStage::Seedling.next_stage(10, 80.0, 80.0, 1.0),
            Some(GrowthStage::Sapling)
        );
        // Not enough resources
        assert_eq!(
            GrowthStage::Seedling.next_stage(10, 50.0, 80.0, 1.0),
            None
        );
        // Sapling → YoungTree (threshold: 500)
        assert_eq!(
            GrowthStage::Sapling.next_stage(100, 500.0, 500.0, 1.0),
            Some(GrowthStage::YoungTree)
        );
        // YoungTree → Mature
        assert_eq!(
            GrowthStage::YoungTree.next_stage(500, 6000.0, 6000.0, 1.0),
            Some(GrowthStage::Mature)
        );
        // Mature → OldGrowth
        assert_eq!(
            GrowthStage::Mature.next_stage(2000, 10000.0, 10000.0, 1.0),
            Some(GrowthStage::OldGrowth)
        );
        // Death from low health
        assert_eq!(
            GrowthStage::Sapling.next_stage(100, 500.0, 500.0, 0.0),
            Some(GrowthStage::Dead)
        );
    }

    #[test]
    fn conical_crown_wider_at_bottom() {
        let species = &SpeciesTable::default().species[3]; // Pine
        let t = TreeTemplate::generate(species, &GrowthStage::Mature, 42);
        let leaves: Vec<_> = t.voxels.iter().filter(|v| v.3 == Material::Leaf).collect();
        let trunk_top = species.max_height() as isize;
        // Bottom layer leaves
        let bottom: Vec<_> = leaves.iter().filter(|v| v.2 == trunk_top).collect();
        // Top layer leaves
        let top_z = leaves.iter().map(|v| v.2).max().unwrap();
        let top: Vec<_> = leaves.iter().filter(|v| v.2 == top_z).collect();
        assert!(
            bottom.len() >= top.len(),
            "Conical crown should be wider at bottom ({}) than top ({})",
            bottom.len(),
            top.len()
        );
    }

    #[test]
    fn different_species_produce_different_templates() {
        let table = SpeciesTable::default();
        let oak = TreeTemplate::generate(&table.species[0], &GrowthStage::Mature, 42);
        let pine = TreeTemplate::generate(&table.species[3], &GrowthStage::Mature, 42);
        assert_ne!(oak.voxels.len(), pine.voxels.len());
    }

    #[test]
    fn attraction_points_within_crown_envelope() {
        let table = SpeciesTable::default();
        for species in &table.species {
            if !species.uses_skeleton() {
                // Non-tree types don't use attraction points
                continue;
            }
            let points = generate_attraction_points(species, &GrowthStage::Mature, 42);
            assert!(
                !points.is_empty(),
                "{} should have attraction points",
                species.name
            );
            // All points should be above trunk base (z > 0)
            for &(_dx, _dy, z) in &points {
                assert!(
                    z > 0,
                    "{}: attraction point at z={} should be above ground",
                    species.name,
                    z
                );
            }
        }
    }

    #[test]
    fn init_skeleton_has_trunk_and_roots() {
        let table = SpeciesTable::default();
        let species = &table.species[0]; // Oak
        let (branches, points) = init_skeleton(species, &GrowthStage::YoungTree, 42);

        let has_trunk = branches.iter().any(|b| b.material == Material::Trunk);
        let has_root = branches.iter().any(|b| b.material == Material::Root);
        assert!(has_trunk, "Skeleton should have trunk nodes");
        assert!(has_root, "Skeleton should have root nodes");
        assert!(!points.is_empty(), "Should have attraction points");

        // Trunk should go from z=0 to trunk_h-1
        let trunk_h = (species.max_height() * 2 / 3).max(3) as isize;
        let trunk_positions: Vec<_> = branches
            .iter()
            .filter(|b| b.material == Material::Trunk)
            .map(|b| b.pos.2)
            .collect();
        assert_eq!(*trunk_positions.iter().min().unwrap(), 0);
        assert_eq!(*trunk_positions.iter().max().unwrap(), trunk_h - 1);
    }

    #[test]
    fn deterministic_skeleton() {
        let table = SpeciesTable::default();
        let species = &table.species[0];
        let (b1, p1) = init_skeleton(species, &GrowthStage::YoungTree, 42);
        let (b2, p2) = init_skeleton(species, &GrowthStage::YoungTree, 42);
        assert_eq!(b1.len(), b2.len());
        assert_eq!(p1.len(), p2.len());
        for (a, b) in b1.iter().zip(b2.iter()) {
            assert_eq!(a.pos, b.pos);
        }
    }

    #[test]
    fn different_seeds_different_points() {
        let table = SpeciesTable::default();
        let species = &table.species[0];
        let p1 = generate_attraction_points(species, &GrowthStage::Mature, 1);
        let p2 = generate_attraction_points(species, &GrowthStage::Mature, 999);
        // Different seeds should produce different point distributions
        assert_ne!(p1, p2);
    }

    #[test]
    fn species_table_has_12_species() {
        let table = SpeciesTable::default();
        assert_eq!(table.species.len(), 12);
    }

    #[test]
    fn plant_types_correct() {
        let table = SpeciesTable::default();
        // 4 trees
        assert_eq!(
            table
                .species
                .iter()
                .filter(|s| s.plant_type == PlantType::Tree)
                .count(),
            4
        );
        // 3 shrubs
        assert_eq!(
            table
                .species
                .iter()
                .filter(|s| s.plant_type == PlantType::Shrub)
                .count(),
            3
        );
        // 2 flowers
        assert_eq!(
            table
                .species
                .iter()
                .filter(|s| s.plant_type == PlantType::Flower)
                .count(),
            2
        );
        // 3 groundcover
        assert_eq!(
            table
                .species
                .iter()
                .filter(|s| s.plant_type == PlantType::Groundcover)
                .count(),
            3
        );
    }

    #[test]
    fn shrub_template_has_branch_and_leaf() {
        let table = SpeciesTable::default();
        // Fern = index 4
        let fern = &table.species[4];
        assert_eq!(fern.plant_type, PlantType::Shrub);
        let template = TreeTemplate::generate(fern, &GrowthStage::Mature, 42);
        let has_branch = template
            .voxels
            .iter()
            .any(|(_, _, _, m)| *m == Material::Branch);
        let has_leaf = template
            .voxels
            .iter()
            .any(|(_, _, _, m)| *m == Material::Leaf);
        let has_root = template
            .voxels
            .iter()
            .any(|(_, _, _, m)| *m == Material::Root);
        assert!(has_branch, "Shrub should have branch voxels");
        assert!(has_leaf, "Shrub should have leaf voxels");
        assert!(has_root, "Shrub should have root voxels");
    }

    #[test]
    fn flower_template_has_stem_and_bloom() {
        let table = SpeciesTable::default();
        // Wildflower = index 7
        let wildflower = &table.species[7];
        assert_eq!(wildflower.plant_type, PlantType::Flower);
        let template = TreeTemplate::generate(wildflower, &GrowthStage::Mature, 42);
        let has_trunk = template
            .voxels
            .iter()
            .any(|(_, _, _, m)| *m == Material::Trunk);
        let has_leaf = template
            .voxels
            .iter()
            .any(|(_, _, _, m)| *m == Material::Leaf);
        assert!(has_trunk, "Flower should have stem (trunk)");
        assert!(has_leaf, "Flower should have bloom (leaf)");
        // Flowers should be short
        let max_z = template.voxels.iter().map(|(_, _, z, _)| *z).max().unwrap();
        let max_height = wildflower.max_height() as isize;
        assert!(
            max_z <= max_height,
            "Flower should be within species height, max_z={} max_height={}",
            max_z,
            max_height
        );
    }

    #[test]
    fn groundcover_template_is_flat() {
        let table = SpeciesTable::default();
        // Moss = index 9
        let moss = &table.species[9];
        assert_eq!(moss.plant_type, PlantType::Groundcover);
        let template = TreeTemplate::generate(moss, &GrowthStage::Mature, 42);
        let has_leaf = template
            .voxels
            .iter()
            .any(|(_, _, _, m)| *m == Material::Leaf);
        assert!(has_leaf, "Groundcover should have leaf voxels");
        // All above-ground voxels should be at z=0
        let above_ground: Vec<_> = template
            .voxels
            .iter()
            .filter(|(_, _, z, m)| *z >= 0 && *m != Material::Root)
            .collect();
        assert!(!above_ground.is_empty());
        for (_, _, z, _) in &above_ground {
            assert_eq!(*z, 0, "Groundcover should be flat at z=0");
        }
    }

    #[test]
    fn non_tree_species_no_skeleton() {
        let table = SpeciesTable::default();
        for species in &table.species {
            if species.plant_type != PlantType::Tree {
                assert!(
                    !species.uses_skeleton(),
                    "{} should not use skeleton",
                    species.name
                );
            }
        }
    }

    #[test]
    fn all_species_have_valid_templates_at_all_stages() {
        let table = SpeciesTable::default();
        let stages = [
            GrowthStage::Seedling,
            GrowthStage::Sapling,
            GrowthStage::YoungTree,
            GrowthStage::Mature,
            GrowthStage::Dead,
        ];
        for species in &table.species {
            for stage in &stages {
                let template = TreeTemplate::generate(species, stage, 42);
                assert!(
                    !template.voxels.is_empty(),
                    "{} at {:?} should produce voxels",
                    species.name,
                    stage
                );
            }
        }
    }
}

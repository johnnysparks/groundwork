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
                if accumulated_water >= 200.0 && accumulated_light >= 200.0 {
                    Some(GrowthStage::Sapling)
                } else {
                    None
                }
            }
            GrowthStage::Sapling => {
                if accumulated_water >= 1500.0 && accumulated_light >= 1500.0 {
                    Some(GrowthStage::YoungTree)
                } else {
                    None
                }
            }
            GrowthStage::YoungTree => {
                if accumulated_water >= 6000.0 && accumulated_light >= 6000.0 {
                    Some(GrowthStage::Mature)
                } else {
                    None
                }
            }
            GrowthStage::Mature => {
                if age >= 2000 {
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

/// Species parameters — defines what a tree CAN become.
#[derive(Clone, Debug)]
pub struct Species {
    pub name: &'static str,
    pub max_height: u8,
    pub root_depth: u8,
    pub crown_radius: u8,
    pub crown_shape: CrownShape,
    pub water_need: ResourceNeed,
    pub light_need: ResourceNeed,
    pub growth_rate: f32,
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
                Species {
                    name: "Oak",
                    max_height: 8,
                    root_depth: 4,
                    crown_radius: 3,
                    crown_shape: CrownShape::Round,
                    water_need: ResourceNeed::Medium,
                    light_need: ResourceNeed::Medium,
                    growth_rate: 1.0,
                },
                Species {
                    name: "Birch",
                    max_height: 7,
                    root_depth: 3,
                    crown_radius: 2,
                    crown_shape: CrownShape::Narrow,
                    water_need: ResourceNeed::Low,
                    light_need: ResourceNeed::Medium,
                    growth_rate: 1.5,
                },
                Species {
                    name: "Willow",
                    max_height: 5,
                    root_depth: 3,
                    crown_radius: 4,
                    crown_shape: CrownShape::Wide,
                    water_need: ResourceNeed::High,
                    light_need: ResourceNeed::Low,
                    growth_rate: 0.8,
                },
                Species {
                    name: "Pine",
                    max_height: 9,
                    root_depth: 5,
                    crown_radius: 3,
                    crown_shape: CrownShape::Conical,
                    water_need: ResourceNeed::Low,
                    light_need: ResourceNeed::High,
                    growth_rate: 0.7,
                },
            ],
        }
    }
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
}

/// Generated template: list of (dx, dy, dz, Material) offsets from root_pos.
pub struct TreeTemplate {
    pub voxels: Vec<(isize, isize, isize, Material)>,
}

impl TreeTemplate {
    /// Generate a voxel template for the given species and growth stage.
    pub fn generate(species: &Species, stage: &GrowthStage, rng_seed: u64) -> Self {
        match stage {
            GrowthStage::Seedling => Self::seedling(species),
            GrowthStage::Sapling => Self::sapling(species, rng_seed),
            GrowthStage::YoungTree => Self::young_tree(species, rng_seed),
            GrowthStage::Mature | GrowthStage::OldGrowth => Self::mature(species, rng_seed),
            GrowthStage::Dead => Self::dead(species, rng_seed),
        }
    }

    fn seedling(_species: &Species) -> Self {
        let voxels = vec![
            (0, 0, 0, Material::Trunk),
            (0, 0, -1, Material::Root),
        ];
        Self { voxels }
    }

    fn sapling(species: &Species, rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let trunk_h = (species.max_height / 3).max(2) as isize;

        // Trunk
        for z in 0..trunk_h {
            voxels.push((0, 0, z, Material::Trunk));
        }

        // Small leaf cap
        Self::add_leaf_disc(&mut voxels, 0, 0, trunk_h, 1, rng_seed, 0);

        // Roots
        let root_d = (species.root_depth / 2).max(1) as isize;
        for z in 1..=root_d {
            voxels.push((0, 0, -z, Material::Root));
        }

        Self { voxels }
    }

    fn young_tree(species: &Species, rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let trunk_h = (species.max_height * 2 / 3).max(3) as isize;
        let crown_r = ((species.crown_radius + 1) / 2).max(1);

        // Trunk
        for z in 0..trunk_h {
            voxels.push((0, 0, z, Material::Trunk));
        }

        // Crown (leaves first so branches can overwrite where they overlap)
        Self::add_crown(&mut voxels, trunk_h, crown_r, &species.crown_shape, rng_seed);

        // Branches near the top
        let branch_z = trunk_h - 2;
        if branch_z > 0 {
            Self::add_branches(&mut voxels, branch_z, 1, rng_seed, 0);
        }

        // Roots
        let root_d = (species.root_depth * 2 / 3).max(2) as isize;
        for z in 1..=root_d {
            voxels.push((0, 0, -z, Material::Root));
            if z >= 2 {
                let h = tree_hash(rng_seed, z as u64);
                let dx = (h % 3) as isize - 1;
                let dy = ((h >> 8) % 3) as isize - 1;
                if dx != 0 || dy != 0 {
                    voxels.push((dx, dy, -z, Material::Root));
                }
            }
        }

        Self { voxels }
    }

    fn mature(species: &Species, rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let trunk_h = species.max_height as isize;
        let crown_r = species.crown_radius;

        // Trunk
        for z in 0..trunk_h {
            voxels.push((0, 0, z, Material::Trunk));
        }

        // Crown (leaves first)
        Self::add_crown(&mut voxels, trunk_h, crown_r, &species.crown_shape, rng_seed);

        // Branches at multiple levels (after leaves so they show through)
        let branch_start = trunk_h / 2;
        for i in 0..3 {
            let bz = branch_start + i * 2;
            if bz < trunk_h {
                let reach = if i == 1 { 2 } else { 1 };
                Self::add_branches(&mut voxels, bz, reach, rng_seed, i as u64);
            }
        }

        // Full roots with lateral spread
        let root_d = species.root_depth as isize;
        for z in 1..=root_d {
            voxels.push((0, 0, -z, Material::Root));
            if z >= 2 {
                for step in 0..2u64 {
                    let h = tree_hash(rng_seed, z as u64 * 10 + step);
                    let dx = (h % 3) as isize - 1;
                    let dy = ((h >> 8) % 3) as isize - 1;
                    if dx != 0 || dy != 0 {
                        voxels.push((dx, dy, -z, Material::Root));
                    }
                }
            }
        }

        Self { voxels }
    }

    fn dead(species: &Species, rng_seed: u64) -> Self {
        let mut voxels = Vec::new();
        let trunk_h = (species.max_height / 2).max(1) as isize;

        for z in 0..trunk_h {
            voxels.push((0, 0, z, Material::DeadWood));
        }

        // A dead branch or two
        if trunk_h > 2 {
            let h = tree_hash(rng_seed, 0);
            let dx: isize = if h % 2 == 0 { 1 } else { -1 };
            voxels.push((dx, 0, trunk_h - 1, Material::DeadWood));
        }

        // Remaining roots
        for z in 1..=(species.root_depth as isize / 2).max(1) {
            voxels.push((0, 0, -z, Material::Root));
        }

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
        // Seedling → Sapling
        assert_eq!(
            GrowthStage::Seedling.next_stage(10, 200.0, 200.0, 1.0),
            Some(GrowthStage::Sapling)
        );
        // Not enough resources
        assert_eq!(
            GrowthStage::Seedling.next_stage(10, 100.0, 200.0, 1.0),
            None
        );
        // Sapling → YoungTree
        assert_eq!(
            GrowthStage::Sapling.next_stage(100, 1500.0, 1500.0, 1.0),
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
        let trunk_top = species.max_height as isize;
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
}

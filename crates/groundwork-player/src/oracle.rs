//! Oracle-view state snapshots — privileged ground truth.
//!
//! Evaluators use oracle snapshots to check whether the sim reached the desired state.
//! The actor never sees these; they exist only for scoring and debugging.

use serde::{Deserialize, Serialize};

use groundwork_sim::grid::VoxelGrid;
use groundwork_sim::voxel::Material;
use groundwork_sim::Tick;

use bevy_ecs::prelude::*;

/// Privileged snapshot of simulation state for evaluators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleSnapshot {
    pub tick: u64,
    pub material_counts: MaterialCounts,
    /// Sampled voxel values at specific coordinates (set by the scenario).
    pub probes: Vec<VoxelProbe>,
}

/// Counts of each material type in the grid.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialCounts {
    pub air: u64,
    pub soil: u64,
    pub stone: u64,
    pub water: u64,
    pub root: u64,
    pub seed: u64,
    pub trunk: u64,
    pub branch: u64,
    pub leaf: u64,
    pub deadwood: u64,
    pub wet_soil: u64,
}

impl MaterialCounts {
    /// Total living plant material (seed + root + trunk + branch + leaf).
    pub fn total_plant(&self) -> u64 {
        self.seed + self.root + self.trunk + self.branch + self.leaf
    }

    /// Total tree structure (trunk + branch + leaf).
    pub fn total_tree(&self) -> u64 {
        self.trunk + self.branch + self.leaf
    }
}

/// A single voxel's state at a specific coordinate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelProbe {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub material: String,
    pub water_level: u8,
    pub light_level: u8,
    pub nutrient_level: u8,
}

/// Take an oracle snapshot of the current world state.
pub fn snapshot(world: &World) -> OracleSnapshot {
    snapshot_with_probes(world, &[])
}

/// Take an oracle snapshot with specific voxel probes.
pub fn snapshot_with_probes(world: &World, probe_coords: &[(usize, usize, usize)]) -> OracleSnapshot {
    let grid = world.resource::<VoxelGrid>();
    let tick = world.resource::<Tick>().0;

    let mut counts = MaterialCounts::default();
    for v in grid.cells() {
        match v.material {
            Material::Air => counts.air += 1,
            Material::Soil => {
                counts.soil += 1;
                if v.water_level > 50 {
                    counts.wet_soil += 1;
                }
            }
            Material::Stone => counts.stone += 1,
            Material::Water => counts.water += 1,
            Material::Root => counts.root += 1,
            Material::Seed => counts.seed += 1,
            Material::Trunk => counts.trunk += 1,
            Material::Branch => counts.branch += 1,
            Material::Leaf => counts.leaf += 1,
            Material::DeadWood => counts.deadwood += 1,
        }
    }

    let probes = probe_coords
        .iter()
        .filter_map(|&(x, y, z)| {
            grid.get(x, y, z).map(|v| VoxelProbe {
                x,
                y,
                z,
                material: v.material.name().to_string(),
                water_level: v.water_level,
                light_level: v.light_level,
                nutrient_level: v.nutrient_level,
            })
        })
        .collect();

    OracleSnapshot {
        tick,
        material_counts: counts,
        probes,
    }
}

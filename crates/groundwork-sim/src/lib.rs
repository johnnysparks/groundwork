pub mod grid;
pub mod save;
pub mod scale;
pub mod soil;
pub mod systems;
pub mod tree;
pub mod voxel;

#[cfg(target_arch = "wasm32")]
pub mod wasm_bridge;

use bevy_ecs::prelude::*;

use grid::{VoxelGrid, GROUND_LEVEL};
use soil::SoilGrid;
use systems::{branch_growth, light_propagation, root_growth, root_water_absorption, seed_dispersal, seed_growth, self_pruning, soil_absorption, soil_evolution, tree_growth, tree_rasterize, water_flow, water_spring};
use tree::{SeedSpeciesMap, SpeciesTable};
use voxel::Material;

/// Tick counter resource.
#[derive(Resource, Default)]
pub struct Tick(pub u64);

/// Focus/cursor position in the world. Shared by CLI and TUI.
#[derive(Resource, Debug, Clone)]
pub struct FocusState {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    /// Active tool operation: material being placed and the start position.
    pub tool: Option<ToolState>,
}

#[derive(Debug, Clone)]
pub struct ToolState {
    pub material: Material,
    pub start_x: usize,
    pub start_y: usize,
    pub start_z: usize,
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            x: scale::grid_center_x(),
            y: scale::grid_center_y(),
            z: GROUND_LEVEL + 1,
            tool: None,
        }
    }
}

/// Create a new simulation world with default terrain.
pub fn create_world() -> World {
    let mut world = World::new();
    world.insert_resource(VoxelGrid::new());
    world.insert_resource(SoilGrid::new());
    world.insert_resource(Tick::default());
    world.insert_resource(FocusState::default());
    world.insert_resource(SpeciesTable::default());
    world.insert_resource(SeedSpeciesMap::default());
    world
}

/// Create the simulation schedule with all systems in order.
pub fn create_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems((water_spring, water_flow, soil_absorption, root_water_absorption, soil_evolution, light_propagation, seed_growth, ApplyDeferred, tree_growth, branch_growth, self_pruning, tree_rasterize, root_growth, seed_dispersal, tick_counter).chain());
    schedule
}

/// Advance the simulation by one step.
pub fn tick(world: &mut World, schedule: &mut Schedule) {
    schedule.run(world);
}

fn tick_counter(mut t: ResMut<Tick>) {
    t.0 += 1;
}

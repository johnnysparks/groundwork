pub mod grid;
pub mod save;
pub mod systems;
pub mod voxel;

use bevy_ecs::prelude::*;

use grid::VoxelGrid;
use systems::{light_propagation, root_water_absorption, seed_growth, soil_absorption, water_flow};

/// Tick counter resource.
#[derive(Resource, Default)]
pub struct Tick(pub u64);

/// Create a new simulation world with default terrain.
pub fn create_world() -> World {
    let mut world = World::new();
    world.insert_resource(VoxelGrid::new());
    world.insert_resource(Tick::default());
    world
}

/// Create the simulation schedule with all systems in order.
pub fn create_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems((water_flow, soil_absorption, root_water_absorption, light_propagation, seed_growth, tick_counter).chain());
    schedule
}

/// Advance the simulation by one step.
pub fn tick(world: &mut World, schedule: &mut Schedule) {
    schedule.run(world);
}

fn tick_counter(mut t: ResMut<Tick>) {
    t.0 += 1;
}

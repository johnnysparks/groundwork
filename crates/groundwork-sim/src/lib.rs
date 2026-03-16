pub mod fauna;
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

use fauna::{fauna_effects, fauna_spawn, fauna_update, FaunaList};
use grid::{VoxelGrid, GROUND_LEVEL};
use soil::SoilGrid;
use systems::{
    branch_growth, light_propagation, pioneer_succession, root_growth, root_water_absorption,
    seed_dispersal, seed_growth, self_pruning, soil_absorption, soil_evolution, tree_growth,
    tree_rasterize, water_flow, water_spring,
};
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

/// Create a new simulation world with default terrain (bare, for tests).
pub fn create_world() -> World {
    let mut world = World::new();
    world.insert_resource(VoxelGrid::new());
    world.insert_resource(SoilGrid::new());
    world.insert_resource(Tick::default());
    world.insert_resource(FocusState::default());
    world.insert_resource(SpeciesTable::default());
    world.insert_resource(SeedSpeciesMap::default());
    world.insert_resource(FaunaList::default());
    world
}

/// Create a world with the full glen dressing: peripheral forest ring and starter garden.
/// Called by the WASM init and TUI new-world paths.
pub fn create_world_with_garden() -> World {
    let mut world = create_world();
    plant_starter_garden(&mut world);

    // Pre-tick so the world has visible growth and wet soil from tick 0.
    // Seeds start sprouting, water flows into soil, the oak begins branching.
    let mut schedule = create_schedule();
    for _ in 0..40 {
        tick(&mut world, &mut schedule);
    }
    // Reset tick counter so the player starts at tick 0
    world.resource_mut::<Tick>().0 = 0;

    world
}

/// Plant a small starter garden near the spring so the world feels alive from tick 0.
/// Places a young oak, a few seeds of varied species, and some groundcover.
fn plant_starter_garden(world: &mut World) {
    use tree::{GrowthStage, Tree, TreeTemplate};

    let cx = grid::GRID_X / 2;
    let cy = grid::GRID_Y / 2;
    let species_table = world.resource::<SpeciesTable>().species.clone();

    // -- Young oak a few voxels from the spring --
    let oak_x = cx + 8;
    let oak_y = cy + 4;
    let oak_z = VoxelGrid::surface_height(oak_x, oak_y) + 1;
    let oak_species = &species_table[0]; // Oak
    let oak_template = TreeTemplate::generate(oak_species, &GrowthStage::YoungTree, 42);
    let mut oak_footprint = Vec::new();
    {
        let grid = world.resource_mut::<VoxelGrid>().into_inner();
        for &(dx, dy, dz, mat) in &oak_template.voxels {
            let ax = oak_x as isize + dx;
            let ay = oak_y as isize + dy;
            let az = oak_z as isize + dz;
            if ax < 0 || ay < 0 || az < 0 {
                continue;
            }
            let (ax, ay, az) = (ax as usize, ay as usize, az as usize);
            if !VoxelGrid::in_bounds(ax, ay, az) {
                continue;
            }
            if let Some(cell) = grid.get_mut(ax, ay, az) {
                let can_place = match mat {
                    Material::Root => cell.material == Material::Soil,
                    _ => cell.material == Material::Air,
                };
                if can_place {
                    cell.set_material(mat);
                    oak_footprint.push((ax, ay, az));
                }
            }
        }
    }
    world.spawn(Tree {
        species_id: 0,
        root_pos: (oak_x, oak_y, oak_z),
        age: 80,
        stage: GrowthStage::YoungTree,
        health: 1.0,
        accumulated_water: 1500.0,
        accumulated_light: 1500.0,
        rng_seed: 42,
        dirty: true,
        voxel_footprint: oak_footprint,
        branches: Vec::new(),
        attraction_points: Vec::new(),
        skeleton_initialized: false,
    });

    // -- Scatter seeds of varied species around the spring --
    // Each (species_id, dx_from_center, dy_from_center)
    let seed_spots: &[(usize, isize, isize)] = &[
        (1, -6, 3),  // birch
        (4, 3, -5),  // fern
        (7, -4, -3), // wildflower
        (9, 5, 6),   // moss
        (10, -2, 5), // grass
        (11, 4, -7), // clover
        (5, -7, -6), // berry bush
    ];

    // Collect which seeds landed, then update the species map separately
    let mut planted_seeds: Vec<(usize, usize, usize, usize)> = Vec::new();
    {
        let grid = world.resource_mut::<VoxelGrid>().into_inner();
        for &(species_id, dx, dy) in seed_spots {
            let sx = (cx as isize + dx) as usize;
            let sy = (cy as isize + dy) as usize;
            let sz = VoxelGrid::surface_height(sx, sy) + 1;
            if VoxelGrid::in_bounds(sx, sy, sz) {
                if let Some(cell) = grid.get_mut(sx, sy, sz) {
                    if cell.material == Material::Air {
                        cell.set_material(Material::Seed);
                        planted_seeds.push((sx, sy, sz, species_id));
                    }
                }
            }
        }
    }
    {
        let mut seed_map = world.resource_mut::<SeedSpeciesMap>();
        for (sx, sy, sz, species_id) in planted_seeds {
            seed_map.map.insert((sx, sy, sz), species_id);
        }
    }
}

/// Create the simulation schedule with all systems in order.
pub fn create_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            water_spring,
            water_flow,
            soil_absorption,
            root_water_absorption,
            soil_evolution,
            light_propagation,
            seed_growth,
            ApplyDeferred,
            tree_growth,
            branch_growth,
            self_pruning,
            tree_rasterize,
            root_growth,
            seed_dispersal,
            pioneer_succession,
            fauna_spawn,
            fauna_update,
            fauna_effects,
            tick_counter,
        )
            .chain(),
    );
    schedule
}

/// Advance the simulation by one step.
pub fn tick(world: &mut World, schedule: &mut Schedule) {
    schedule.run(world);
}

fn tick_counter(mut t: ResMut<Tick>) {
    t.0 += 1;
}

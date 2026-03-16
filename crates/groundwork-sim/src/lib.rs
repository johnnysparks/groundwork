pub mod fauna;
pub mod gnome;
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
use gnome::{
    gnome_export, gnome_fauna_interact, gnome_move, gnome_needs, gnome_plan, gnome_work, GnomeData,
};
use grid::{VoxelGrid, GROUND_LEVEL};
use soil::SoilGrid;
use systems::{
    branch_growth, deadwood_decay, light_propagation, milestone_tracker, mycorrhizal_network,
    pioneer_succession, root_growth, root_water_absorption, seed_dispersal, seed_growth,
    self_pruning, soil_absorption, soil_evolution, tree_grow_visual, tree_growth, tree_rasterize,
    water_flow, water_spring, weather_system, wind_seed_drift,
};
use tree::{SeedSpeciesMap, SpeciesTable};
use voxel::Material;

/// Tick counter resource.
#[derive(Resource, Default)]
pub struct Tick(pub u64);

/// Day phase resource: cycles 0→99, representing time-of-day.
/// 0-24 = dawn, 25-49 = day, 50-74 = dusk, 75-99 = night.
/// Growth rates scale with phase: day=1.0×, dawn/dusk=0.75×, night=0.5×.
/// Advances 1 per tick, wraps at 100. JS can sync this with the visual day cycle.
#[derive(Resource)]
pub struct DayPhase(pub u8);

impl Default for DayPhase {
    fn default() -> Self {
        Self(30) // start at midday
    }
}

impl DayPhase {
    /// Growth multiplier based on time-of-day. Full sun at day, reduced at night.
    pub fn growth_multiplier(&self) -> f32 {
        match self.0 {
            25..=49 => 1.0,  // day: full growth
            0..=24 => 0.75,  // dawn: moderate
            50..=74 => 0.75, // dusk: moderate
            _ => 0.5,        // night: half growth
        }
    }

    /// Advance the phase by 1, wrapping at 100.
    pub fn advance(&mut self) {
        self.0 = (self.0 + 1) % 100;
    }
}

/// Ecological milestone tracking for species unlock progression.
///
/// The game vision describes a learning arc (groundcover → flowers → shrubs → trees).
/// This resource tracks sim-state conditions that JS reads to gate species availability.
/// Updated every 20 ticks by the `milestone_tracker` system.
///
/// Milestones are one-way (once reached, they stay reached) so the player
/// never loses access to unlocked species.
#[derive(Resource, Debug, Clone, Default)]
pub struct EcoMilestones {
    /// Tier 0: always available (moss, grass, clover)
    /// Tier 1: groundcover established (10+ groundcover leaf voxels)
    pub tier1_flowers: bool,
    /// Tier 2: pollinators attracted (2+ bees/butterflies present)
    pub tier2_shrubs: bool,
    /// Tier 3: fauna ecosystem active (4+ total fauna, 3+ species diversity)
    pub tier3_trees: bool,
    /// Raw counts for JS to display progress toward next tier
    pub groundcover_count: u16,
    pub pollinator_count: u16,
    pub fauna_count: u16,
    pub species_diversity: u8,
}

/// Species discovery tracking. Players don't see a full species list — they
/// discover species through ecological processes (pioneer succession, bird seeds,
/// squirrel acorns, seed dispersal). The UI shows "plant groundcover" / "plant tree"
/// and the sim picks from discovered species of that type.
///
/// Discovery is one-way: once a species appears in the garden, it's discovered forever.
/// Updated by the `milestone_tracker` system alongside EcoMilestones.
#[derive(Resource, Debug, Clone)]
pub struct DiscoveredSpecies {
    /// Bitfield: species_id N is discovered if bit N is set.
    /// Supports up to 32 species (u32). Currently 12 species (0-11).
    pub discovered: u32,
}

impl Default for DiscoveredSpecies {
    fn default() -> Self {
        // Moss (9), grass (10), clover (11) are always discovered — the player
        // starts knowing these basic groundcover species.
        let mut d = 0u32;
        d |= 1 << 9; // moss
        d |= 1 << 10; // grass
        d |= 1 << 11; // clover
        Self { discovered: d }
    }
}

impl DiscoveredSpecies {
    pub fn is_discovered(&self, species_id: usize) -> bool {
        species_id < 32 && (self.discovered & (1 << species_id)) != 0
    }

    pub fn discover(&mut self, species_id: usize) {
        if species_id < 32 {
            self.discovered |= 1 << species_id;
        }
    }

    pub fn count(&self) -> u32 {
        self.discovered.count_ones()
    }
}

/// Weather system: periodic rain bursts and drought periods.
///
/// Creates dramatic garden-wide events that test ecosystem resilience.
/// Rain floods the surface with water; drought accelerates evaporation.
/// Events trigger every ~200-400 ticks using deterministic RNG.
#[derive(Resource, Debug, Clone)]
pub struct Weather {
    /// Current weather state
    pub state: WeatherState,
    /// Ticks remaining in current state
    pub duration: u32,
    /// RNG counter for deterministic weather sequence
    pub sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherState {
    Clear,
    Rain,
    Drought,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            state: WeatherState::Clear,
            duration: 500, // start clear for 500 ticks so early game isn't disrupted
            sequence: 0,
        }
    }
}

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
    world.insert_resource(DayPhase::default());
    world.insert_resource(EcoMilestones::default());
    world.insert_resource(Weather::default());
    world.insert_resource(DiscoveredSpecies::default());
    world.insert_resource(FocusState::default());
    world.insert_resource(SpeciesTable::default());
    world.insert_resource(SeedSpeciesMap::default());
    world.insert_resource(FaunaList::default());
    world.insert_resource(GnomeData::default());
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
    use tree::{init_skeleton, GrowthStage, Tree};

    let cx = grid::GRID_X / 2;
    let cy = grid::GRID_Y / 2;
    let species_table = world.resource::<SpeciesTable>().species.clone();

    // -- Young oak a few voxels from the spring --
    let oak_x = cx + 8;
    let oak_y = cy + 4;
    let oak_z = VoxelGrid::surface_height(oak_x, oak_y) + 1;
    let oak_species = &species_table[0]; // Oak
                                         // Initialize the skeleton so branch_growth and tree_rasterize work correctly.
                                         // Without this, the tree has empty branches and the skeleton path never fires,
                                         // causing a floating tree when it transitions to Mature during pre-ticks.
    let (branches, attraction_points) = init_skeleton(oak_species, &GrowthStage::YoungTree, 42);
    world.spawn(Tree {
        species_id: 0,
        root_pos: (oak_x, oak_y, oak_z),
        age: 80,
        stage: GrowthStage::YoungTree,
        health: 1.0,
        accumulated_water: 2900.0,
        accumulated_light: 2900.0,
        rng_seed: 42,
        dirty: true,
        voxel_footprint: Vec::new(),
        branches,
        attraction_points,
        skeleton_initialized: true,
        stage_changed: true,
        pending_voxels: Vec::new(),
        revealed_z: 0,
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
/// Split into two groups to stay within bevy_ecs tuple size limits.
pub fn create_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    // Group 1: physics, resources, growth
    schedule.add_systems(
        (
            weather_system,
            water_spring,
            water_flow,
            soil_absorption,
            root_water_absorption,
            soil_evolution,
            light_propagation,
            seed_growth,
            ApplyDeferred,
            tree_growth,
            mycorrhizal_network,
            branch_growth,
        )
            .chain(),
    );
    // Group 2: rasterize, ecology, fauna — runs after group 1
    schedule.add_systems(
        (
            self_pruning,
            tree_rasterize,
            tree_grow_visual,
            root_growth,
            deadwood_decay,
            wind_seed_drift,
            seed_dispersal,
            pioneer_succession,
            fauna_spawn,
            fauna_update,
            gnome_plan,
            gnome_move,
            gnome_work,
            gnome_needs,
            gnome_fauna_interact,
            gnome_export,
            fauna_effects,
            milestone_tracker,
            tick_counter,
        )
            .chain()
            .after(mycorrhizal_network),
    );
    schedule
}

/// Advance the simulation by one step.
pub fn tick(world: &mut World, schedule: &mut Schedule) {
    schedule.run(world);
}

fn tick_counter(mut t: ResMut<Tick>, mut phase: ResMut<DayPhase>) {
    t.0 += 1;
    phase.advance();
}

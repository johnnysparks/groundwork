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

/// Create a world with terrain ready to play: gentle slope with a pond.
/// Called by the WASM init and TUI new-world paths.
/// Plants a starter garden near the pond and pre-ticks so plants are visible on first load.
pub fn create_world_with_garden() -> World {
    let mut world = create_world();

    // Pre-tick 50 ticks so water flows from the pond into surrounding soil.
    let mut schedule = create_schedule();
    for _ in 0..50 {
        tick(&mut world, &mut schedule);
    }

    // Plant starter seeds near the pond where soil is now moist.
    // Pond center is at (POND_X=40, POND_Y=16). Moist soil extends south (higher y).
    // Species: 0=Oak, 1=Birch, 7=Wildflower, 8=Daisy, 9=Moss, 10=Grass, 11=Clover
    let starter_seeds: &[(usize, usize, usize)] = &[
        // Groundcover ring near pond edge (y=20-26) — pioneers that establish quickly
        (36, 21, 9),  // moss
        (40, 20, 9),  // moss
        (44, 22, 9),  // moss
        (38, 24, 10), // grass
        (42, 23, 10), // grass
        (46, 24, 10), // grass
        (35, 25, 11), // clover
        (40, 26, 11), // clover
        (45, 25, 11), // clover
        // Flowers among the groundcover (y=22-28)
        (37, 23, 7),  // wildflower
        (43, 22, 7),  // wildflower
        (39, 27, 8),  // daisy
        (41, 26, 8),  // daisy
        // Trees set back from pond (y=28-35) — will grow into canopy
        (38, 30, 0),  // oak
        (44, 32, 0),  // oak
        (36, 34, 1),  // birch
        (42, 28, 1),  // birch
        // Additional trees spread around the wet zone for a fuller garden.
        // Spaced 10+ voxels apart to avoid territorial suppression (6-voxel radius).
        // Drama (shade competition, crowding) emerges naturally during gameplay
        // as trees mature and canopies overlap — not during pre-tick.
        (30, 20, 0),  // oak — west of pond
        (50, 20, 1),  // birch — east of pond
        (40, 14, 3),  // pine — north, near pond edge
        // Scattered groundcover across broader terrain for green carpet.
        // These fill in the bare soil south and east of the pond.
        // Pioneer succession will spread from these anchor points.
        (30, 18, 9),  // moss — west of pond
        (50, 18, 10), // grass — east of pond
        (35, 20, 10), // grass — southwest
        (45, 20, 9),  // moss — southeast
        (32, 22, 11), // clover — west mid
        (48, 22, 11), // clover — east mid
        (34, 18, 10), // grass — near pond west
        (46, 18, 9),  // moss — near pond east
    ];

    {
        let grid = world.resource::<VoxelGrid>();
        // Find surface z for each seed position and collect placements
        let placements: Vec<(usize, usize, usize, usize)> = starter_seeds
            .iter()
            .filter_map(|&(x, y, species_id)| {
                // Find the air cell just above the surface
                let surface_z = VoxelGrid::surface_height(x, y);
                let seed_z = surface_z + 1;
                if seed_z < grid::GRID_Z {
                    if let Some(v) = grid.get(x, y, seed_z) {
                        if v.material == Material::Air {
                            return Some((x, y, seed_z, species_id));
                        }
                    }
                }
                None
            })
            .collect();

        // Release the immutable borrow before mutating
        let _ = grid;

        // Place seeds and register species
        let mut grid = world.resource_mut::<VoxelGrid>();
        for &(x, y, z, _) in &placements {
            if let Some(voxel) = grid.get_mut(x, y, z) {
                voxel.set_material(Material::Seed);
            }
        }
        drop(grid);

        let mut seed_map = world.resource_mut::<SeedSpeciesMap>();
        for &(x, y, z, species_id) in &placements {
            seed_map.map.insert((x, y, z), species_id);
        }
    }

    // Pre-tick 300 more ticks so seeds germinate, trees grow, and the
    // competing grove has time to develop visible stress from crowding.
    for _ in 0..300 {
        tick(&mut world, &mut schedule);
    }

    // Reset tick counter so the player starts at tick 0
    world.resource_mut::<Tick>().0 = 0;

    world
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

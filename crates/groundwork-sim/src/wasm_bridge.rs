//! WASM bridge: exports sim functions for the Three.js web renderer.
//!
//! This module only compiles on `wasm32`. It exposes a thin API:
//! - `init()` / `tick(n)` — lifecycle
//! - `grid_ptr()` / `grid_len()` / `soil_ptr()` / `soil_len()` — zero-copy grid access
//! - `place_tool(tool, x, y, z)` / `fill_tool(...)` — player actions
//! - `get_tick()` / `grid_dimensions()` / `get_focus_*()` / `set_focus()` — queries
//!
//! The JS side reads grid data directly from WASM linear memory via typed array views.

use wasm_bindgen::prelude::*;

use crate::fauna::FaunaList;
use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};
use crate::soil::SoilGrid;
use crate::tree::{SeedSpeciesMap, SpeciesTable};
use crate::voxel::Material;
use crate::{FocusState, Tick};

use bevy_ecs::prelude::*;

use std::cell::RefCell;

thread_local! {
    static SIM: RefCell<Option<SimState>> = RefCell::new(None);
}

struct SimState {
    world: World,
    schedule: Schedule,
}

fn with_sim<R>(f: impl FnOnce(&mut SimState) -> R) -> R {
    SIM.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let sim = borrow
            .as_mut()
            .expect("sim not initialized — call init() first");
        f(sim)
    })
}

/// Initialize the simulation. Must be called before any other function.
#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
    let world = crate::create_world_with_garden();
    let schedule = crate::create_schedule();
    SIM.with(|cell| {
        *cell.borrow_mut() = Some(SimState { world, schedule });
    });
}

/// Advance the simulation by `n` ticks.
#[wasm_bindgen]
pub fn tick(n: u32) {
    with_sim(|sim| {
        for _ in 0..n {
            crate::tick(&mut sim.world, &mut sim.schedule);
        }
    });
}

/// Pointer to the raw voxel grid data in WASM linear memory.
/// JS can wrap this as `new Uint8Array(wasm.memory.buffer, ptr, len)`.
/// Each voxel is 4 bytes: [material, water_level, light_level, nutrient_level].
#[wasm_bindgen]
pub fn grid_ptr() -> *const u8 {
    with_sim(|sim| {
        let grid = sim.world.resource::<VoxelGrid>();
        grid.cells().as_ptr() as *const u8
    })
}

/// Length of the voxel grid in bytes.
#[wasm_bindgen]
pub fn grid_len() -> usize {
    with_sim(|sim| {
        let grid = sim.world.resource::<VoxelGrid>();
        grid.cells().len() * 4 // 4 bytes per Voxel
    })
}

/// Pointer to the soil composition grid data.
/// Each cell is 6 bytes: [sand, clay, organic, rock, ph, bacteria].
#[wasm_bindgen]
pub fn soil_ptr() -> *const u8 {
    with_sim(|sim| {
        let soil = sim.world.resource::<SoilGrid>();
        soil.cells().as_ptr() as *const u8
    })
}

/// Length of the soil grid in bytes.
#[wasm_bindgen]
pub fn soil_len() -> usize {
    with_sim(|sim| {
        let soil = sim.world.resource::<SoilGrid>();
        soil.cells().len() * 6 // 6 bytes per SoilComposition
    })
}

/// Grid dimensions as a packed u32: x | (y << 16). Z is returned separately.
#[wasm_bindgen]
pub fn grid_width() -> usize {
    GRID_X
}

#[wasm_bindgen]
pub fn grid_height() -> usize {
    GRID_Y
}

#[wasm_bindgen]
pub fn grid_depth() -> usize {
    GRID_Z
}

/// Current tick count.
#[wasm_bindgen]
pub fn get_tick() -> u64 {
    with_sim(|sim| sim.world.resource::<Tick>().0)
}

/// Current day phase (0-99). Dawn=0-24, Day=25-49, Dusk=50-74, Night=75-99.
/// JS can use this to sync the visual day cycle with the sim's growth rhythm.
#[wasm_bindgen]
pub fn get_day_phase() -> u8 {
    with_sim(|sim| sim.world.resource::<crate::DayPhase>().0)
}

/// Set the day phase explicitly (for syncing JS day cycle to sim).
#[wasm_bindgen]
pub fn set_day_phase(phase: u8) {
    with_sim(|sim| {
        sim.world.resource_mut::<crate::DayPhase>().0 = phase % 100;
    });
}

/// Current weather state: 0=Clear, 1=Rain, 2=Drought.
/// JS can use this for visual effects (rain particles, dry palette shift).
#[wasm_bindgen]
pub fn get_weather_state() -> u8 {
    with_sim(|sim| sim.world.resource::<crate::Weather>().state as u8)
}

/// Ticks remaining in current weather state.
#[wasm_bindgen]
pub fn get_weather_duration() -> u32 {
    with_sim(|sim| sim.world.resource::<crate::Weather>().duration)
}

/// Place a tool at (x, y, z). Tool codes: 0=shovel, 1=seed, 2=water, 3=soil, 4=stone.
/// Returns the landing z coordinate, or -1 if the action had no effect.
#[wasm_bindgen]
pub fn place_tool(tool: u8, x: usize, y: usize, z: usize) -> i32 {
    with_sim(|sim| {
        let mat = match tool {
            0 => Material::Air, // shovel
            1 => Material::Seed,
            2 => Material::Water,
            3 => Material::Soil,
            4 => Material::Stone,
            _ => return -1,
        };

        let grid = sim.world.resource_mut::<VoxelGrid>();
        let grid: &mut VoxelGrid = grid.into_inner();

        if mat == Material::Air {
            // Shovel: dig
            if let Some(voxel) = grid.get_mut(x, y, z) {
                if voxel.material == Material::Air {
                    return -1;
                }
                voxel.set_material(Material::Air);
                return z as i32;
            }
            return -1;
        }

        // Check target is Air
        if let Some(voxel) = grid.get(x, y, z) {
            if voxel.material != Material::Air {
                return -1;
            }
        } else {
            return -1;
        }

        // Gravity for non-stone tools
        let landing_z = match mat {
            Material::Stone => z,
            _ => grid.find_landing_z(x, y, z),
        };

        if let Some(voxel) = grid.get(x, y, landing_z) {
            if voxel.material != Material::Air {
                return -1;
            }
        } else {
            return -1;
        }

        if let Some(voxel) = grid.get_mut(x, y, landing_z) {
            voxel.set_material(mat);

            // Register seed species so the engine grows the right plant
            if mat == Material::Seed {
                let species_idx = SELECTED_SPECIES.with(|cell| *cell.borrow());
                let mut seed_map = sim.world.resource_mut::<SeedSpeciesMap>();
                seed_map.map.insert((x, y, landing_z), species_idx);
            }

            landing_z as i32
        } else {
            -1
        }
    })
}

/// Fill a rectangular region with a tool.
/// Skips seeds and roots to protect living plants (matching CLI behavior).
#[wasm_bindgen]
pub fn fill_tool(tool: u8, x1: usize, y1: usize, z1: usize, x2: usize, y2: usize, z2: usize) {
    let (x_lo, x_hi) = (x1.min(x2), x1.max(x2));
    let (y_lo, y_hi) = (y1.min(y2), y1.max(y2));
    let (z_lo, z_hi) = (z1.min(z2), z1.max(z2));

    // Shovel (tool=0) fill needs explicit protection for seeds/roots.
    // Non-shovel tools already skip non-Air cells via place_tool.
    let is_shovel = tool == 0;

    for z in z_lo..=z_hi {
        for y in y_lo..=y_hi {
            for x in x_lo..=x_hi {
                if is_shovel {
                    // Skip seeds and roots during fill-dig
                    let should_skip = with_sim(|sim| {
                        let grid = sim.world.resource::<VoxelGrid>();
                        if let Some(v) = grid.get(x, y, z) {
                            matches!(v.material, Material::Seed | Material::Root)
                        } else {
                            false
                        }
                    });
                    if should_skip {
                        continue;
                    }
                }
                place_tool(tool, x, y, z);
            }
        }
    }
}

/// Get focus position.
#[wasm_bindgen]
pub fn get_focus_x() -> usize {
    with_sim(|sim| sim.world.resource::<FocusState>().x)
}

#[wasm_bindgen]
pub fn get_focus_y() -> usize {
    with_sim(|sim| sim.world.resource::<FocusState>().y)
}

#[wasm_bindgen]
pub fn get_focus_z() -> usize {
    with_sim(|sim| sim.world.resource::<FocusState>().z)
}

/// Set focus position.
#[wasm_bindgen]
pub fn set_focus(x: usize, y: usize, z: usize) {
    with_sim(|sim| {
        let mut focus = sim.world.resource_mut::<FocusState>();
        focus.x = x;
        focus.y = y;
        focus.z = z;
    });
}

// --- Grid constants ---

/// Ground level Z coordinate (surface height baseline).
#[wasm_bindgen]
pub fn ground_level() -> usize {
    GROUND_LEVEL
}

// --- Material metadata ---

/// Total number of material types.
#[wasm_bindgen]
pub fn material_count() -> u8 {
    Material::COUNT
}

/// Name of material at the given index, or empty string if invalid.
#[wasm_bindgen]
pub fn material_name(idx: u8) -> String {
    Material::from_u8(idx).map_or(String::new(), |m| m.name().to_string())
}

/// Whether a material generates solid mesh geometry.
#[wasm_bindgen]
pub fn material_is_solid(idx: u8) -> bool {
    Material::from_u8(idx).map_or(false, |m| m.is_solid())
}

/// Whether a material is foliage (rendered as billboard sprites).
#[wasm_bindgen]
pub fn material_is_foliage(idx: u8) -> bool {
    Material::from_u8(idx).map_or(false, |m| m.is_foliage())
}

/// Whether a material is a seed (rendered as small sprites).
#[wasm_bindgen]
pub fn material_is_seed(idx: u8) -> bool {
    Material::from_u8(idx).map_or(false, |m| m.is_seed())
}

// --- Tool metadata ---

/// Number of available tools.
#[wasm_bindgen]
pub fn tool_count() -> u8 {
    5
}

/// Name of tool at the given index.
#[wasm_bindgen]
pub fn tool_name(idx: u8) -> String {
    match idx {
        0 => "Shovel".to_string(),
        1 => "Seed".to_string(),
        2 => "Water".to_string(),
        3 => "Soil".to_string(),
        4 => "Stone".to_string(),
        _ => String::new(),
    }
}

// --- Species metadata ---

/// Number of species in the species table.
#[wasm_bindgen]
pub fn species_count() -> usize {
    with_sim(|sim| sim.world.resource::<SpeciesTable>().species.len())
}

/// Name of species at the given index.
#[wasm_bindgen]
pub fn species_name(idx: usize) -> String {
    with_sim(|sim| {
        let table = sim.world.resource::<SpeciesTable>();
        table
            .species
            .get(idx)
            .map_or(String::new(), |s| s.name.to_string())
    })
}

/// Plant type name of species at the given index (Tree, Shrub, Ground, Flower).
#[wasm_bindgen]
pub fn species_plant_type(idx: usize) -> String {
    with_sim(|sim| {
        let table = sim.world.resource::<SpeciesTable>();
        table
            .species
            .get(idx)
            .map_or(String::new(), |s| s.plant_type.name().to_string())
    })
}

// --- Species selection for seed placement ---

thread_local! {
    static SELECTED_SPECIES: RefCell<usize> = RefCell::new(0);
}

/// Set the species index used when placing seeds.
#[wasm_bindgen]
pub fn set_selected_species(idx: usize) {
    SELECTED_SPECIES.with(|cell| *cell.borrow_mut() = idx);
}

/// Get the currently selected species index.
#[wasm_bindgen]
pub fn get_selected_species() -> usize {
    SELECTED_SPECIES.with(|cell| *cell.borrow())
}

// --- Fauna data ---

/// Number of active fauna creatures.
#[wasm_bindgen]
pub fn fauna_count() -> usize {
    with_sim(|sim| sim.world.resource::<FaunaList>().count())
}

/// Pointer to packed fauna data in WASM linear memory.
/// Each fauna is 16 bytes: [type: u8, state: u8, _pad: u8, _pad: u8, x: f32, y: f32, z: f32].
#[wasm_bindgen]
pub fn fauna_ptr() -> *const u8 {
    with_sim(|sim| sim.world.resource::<FaunaList>().export_ptr())
}

/// Length of fauna data in bytes.
#[wasm_bindgen]
pub fn fauna_len() -> usize {
    with_sim(|sim| sim.world.resource::<FaunaList>().export_len())
}

// --- Ecological Milestones ---

/// Whether flowers are unlocked (tier 1: groundcover established).
#[wasm_bindgen]
pub fn milestone_tier1_flowers() -> bool {
    with_sim(|sim| sim.world.resource::<crate::EcoMilestones>().tier1_flowers)
}

/// Whether shrubs are unlocked (tier 2: pollinators attracted).
#[wasm_bindgen]
pub fn milestone_tier2_shrubs() -> bool {
    with_sim(|sim| sim.world.resource::<crate::EcoMilestones>().tier2_shrubs)
}

/// Whether trees are unlocked (tier 3: fauna ecosystem active).
#[wasm_bindgen]
pub fn milestone_tier3_trees() -> bool {
    with_sim(|sim| sim.world.resource::<crate::EcoMilestones>().tier3_trees)
}

/// Groundcover leaf voxel count (progress toward tier 1).
#[wasm_bindgen]
pub fn milestone_groundcover_count() -> u16 {
    with_sim(|sim| {
        sim.world
            .resource::<crate::EcoMilestones>()
            .groundcover_count
    })
}

/// Active pollinator count (progress toward tier 2).
#[wasm_bindgen]
pub fn milestone_pollinator_count() -> u16 {
    with_sim(|sim| {
        sim.world
            .resource::<crate::EcoMilestones>()
            .pollinator_count
    })
}

/// Total active fauna count (progress toward tier 3).
#[wasm_bindgen]
pub fn milestone_fauna_count() -> u16 {
    with_sim(|sim| sim.world.resource::<crate::EcoMilestones>().fauna_count)
}

/// Plant species diversity count (progress toward tier 3).
#[wasm_bindgen]
pub fn milestone_species_diversity() -> u8 {
    with_sim(|sim| {
        sim.world
            .resource::<crate::EcoMilestones>()
            .species_diversity
    })
}

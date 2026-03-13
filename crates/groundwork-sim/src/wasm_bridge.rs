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

use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z};
use crate::soil::SoilGrid;
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
        let sim = borrow.as_mut().expect("sim not initialized — call init() first");
        f(sim)
    })
}

/// Initialize the simulation. Must be called before any other function.
#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
    let world = crate::create_world();
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

/// Place a tool at (x, y, z). Tool codes: 0=shovel, 1=seed, 2=water, 3=soil, 4=stone.
/// Returns the landing z coordinate, or -1 if the action had no effect.
#[wasm_bindgen]
pub fn place_tool(tool: u8, x: usize, y: usize, z: usize) -> i32 {
    with_sim(|sim| {
        let mat = match tool {
            0 => Material::Air,   // shovel
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
            landing_z as i32
        } else {
            -1
        }
    })
}

/// Fill a rectangular region with a tool.
#[wasm_bindgen]
pub fn fill_tool(tool: u8, x1: usize, y1: usize, z1: usize, x2: usize, y2: usize, z2: usize) {
    let (x_lo, x_hi) = (x1.min(x2), x1.max(x2));
    let (y_lo, y_hi) = (y1.min(y2), y1.max(y2));
    let (z_lo, z_hi) = (z1.min(z2), z1.max(z2));
    for z in z_lo..=z_hi {
        for y in y_lo..=y_hi {
            for x in x_lo..=x_hi {
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

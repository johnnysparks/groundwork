//! Garden gnome: the player's agent in the world.
//!
//! The gnome is a sim-side entity that carries out the player's gardening
//! commands. Instead of instant voxel mutation, the player paints zones and
//! the gnome walks over and does the work. This creates natural pacing,
//! character delight, and idle-time reward.
//!
//! Follows the `fauna.rs` pattern: lightweight struct in a Resource, packed
//! export buffer for zero-copy WASM transfer.

use std::collections::VecDeque;

use bevy_ecs::prelude::*;

use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GROUND_LEVEL};
use crate::tree::SeedSpeciesMap;
use crate::voxel::Material;
use crate::Tick;

// --- Tuning constants ---

/// Sim ticks the gnome spends working at each task voxel.
const TICKS_PER_TASK: u8 = 3;

/// Base walk speed in voxels per tick.
const WALK_SPEED: f32 = 1.5;

/// Maximum number of queued tasks.
pub const MAX_QUEUE: usize = 200;

/// Maximum ghost zones exported to the renderer.
pub const MAX_GHOSTS: usize = 200;

// --- State machine ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GnomeState {
    Idle = 0,
    Walking = 1,
    Working = 2,
    // Future phases:
    // Eating = 3,
    // Resting = 4,
    // Reacting = 5,
}

// --- Task ---

#[derive(Clone, Debug)]
pub struct GnomeTask {
    /// Tool code: 0=shovel, 1=seed, 2=water, 3=soil, 4=stone
    pub tool: u8,
    pub x: usize,
    pub y: usize,
    pub z: usize,
    /// Species id for seed tasks; 255 = not applicable
    pub species: u8,
}

// --- Gnome struct ---

#[derive(Clone, Debug)]
pub struct Gnome {
    /// Current position (fractional for smooth interpolation)
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Target position (current task location)
    pub target_x: f32,
    pub target_y: f32,
    pub target_z: f32,
    pub state: GnomeState,
    /// 0 = full, 255 = starving (Phase 2)
    pub hunger: u8,
    /// 255 = full, 0 = exhausted (Phase 2)
    pub energy: u8,
    /// Tool the gnome is currently carrying (matches task tool code)
    pub active_tool: u8,
    /// Progress on current work action (counts up to TICKS_PER_TASK)
    pub work_progress: u8,
}

impl Default for Gnome {
    fn default() -> Self {
        let cx = GRID_X as f32 / 2.0;
        let cy = GRID_Y as f32 / 2.0;
        let gz = GROUND_LEVEL as f32 + 1.0;
        Self {
            x: cx,
            y: cy,
            z: gz,
            target_x: cx,
            target_y: cy,
            target_z: gz,
            state: GnomeState::Idle,
            hunger: 0,
            energy: 255,
            active_tool: 255, // no tool
            work_progress: 0,
        }
    }
}

// --- Resource ---

/// Export format: 32 bytes per gnome.
/// [state: u8, active_tool: u8, hunger: u8, energy: u8,
///  x: f32, y: f32, z: f32,
///  target_x: f32, target_y: f32, target_z: f32,
///  queue_len: u16le, _pad: u16]
pub const GNOME_EXPORT_BYTES: usize = 32;

/// Ghost zone export: 8 bytes each.
/// [x: u16le, y: u16le, z: u16le, tool: u8, species: u8]
pub const GHOST_EXPORT_BYTES: usize = 8;

#[derive(Resource)]
pub struct GnomeData {
    pub gnome: Gnome,
    pub tasks: VecDeque<GnomeTask>,
    export_buf: Vec<u8>,
    ghost_buf: Vec<u8>,
}

impl Default for GnomeData {
    fn default() -> Self {
        Self {
            gnome: Gnome::default(),
            tasks: VecDeque::with_capacity(MAX_QUEUE),
            export_buf: vec![0u8; GNOME_EXPORT_BYTES],
            ghost_buf: vec![0u8; MAX_GHOSTS * GHOST_EXPORT_BYTES],
        }
    }
}

impl GnomeData {
    /// Push a task. Returns false if queue is full.
    pub fn queue_task(&mut self, task: GnomeTask) -> bool {
        if self.tasks.len() >= MAX_QUEUE {
            return false;
        }
        self.tasks.push_back(task);
        true
    }

    /// Remove all tasks at a given position.
    pub fn cancel_at(&mut self, x: usize, y: usize, z: usize) {
        self.tasks.retain(|t| !(t.x == x && t.y == y && t.z == z));
    }

    /// Clear all pending tasks.
    pub fn cancel_all(&mut self) {
        self.tasks.clear();
    }

    /// Pack gnome state into export buffer for WASM bridge.
    pub fn pack_export(&mut self) {
        let g = &self.gnome;
        let buf = &mut self.export_buf;
        buf[0] = g.state as u8;
        buf[1] = g.active_tool;
        buf[2] = g.hunger;
        buf[3] = g.energy;
        buf[4..8].copy_from_slice(&g.x.to_le_bytes());
        buf[8..12].copy_from_slice(&g.y.to_le_bytes());
        buf[12..16].copy_from_slice(&g.z.to_le_bytes());
        buf[16..20].copy_from_slice(&g.target_x.to_le_bytes());
        buf[20..24].copy_from_slice(&g.target_y.to_le_bytes());
        buf[24..28].copy_from_slice(&g.target_z.to_le_bytes());
        let qlen = self.tasks.len() as u16;
        buf[28..30].copy_from_slice(&qlen.to_le_bytes());
        buf[30] = 0;
        buf[31] = 0;
    }

    /// Pack ghost zones into export buffer.
    pub fn pack_ghosts(&mut self) {
        let count = self.tasks.len().min(MAX_GHOSTS);
        for (i, task) in self.tasks.iter().take(count).enumerate() {
            let off = i * GHOST_EXPORT_BYTES;
            self.ghost_buf[off..off + 2].copy_from_slice(&(task.x as u16).to_le_bytes());
            self.ghost_buf[off + 2..off + 4].copy_from_slice(&(task.y as u16).to_le_bytes());
            self.ghost_buf[off + 4..off + 6].copy_from_slice(&(task.z as u16).to_le_bytes());
            self.ghost_buf[off + 6] = task.tool;
            self.ghost_buf[off + 7] = task.species;
        }
    }

    pub fn export_ptr(&self) -> *const u8 {
        self.export_buf.as_ptr()
    }

    pub fn export_len(&self) -> usize {
        GNOME_EXPORT_BYTES
    }

    pub fn ghost_ptr(&self) -> *const u8 {
        self.ghost_buf.as_ptr()
    }

    /// Length of ghost data in bytes (only active ghosts).
    pub fn ghost_len(&self) -> usize {
        self.tasks.len().min(MAX_GHOSTS) * GHOST_EXPORT_BYTES
    }

    pub fn queue_len(&self) -> usize {
        self.tasks.len()
    }
}

// --- Systems ---

/// Pick the next task from the queue and set target. Transitions Idle → Walking.
pub fn gnome_plan(mut gd: ResMut<GnomeData>) {
    let state = gd.gnome.state;
    if state != GnomeState::Idle {
        return;
    }
    if gd.tasks.is_empty() {
        return;
    }

    // Copy task fields before mutating gnome (borrow rules)
    let tx = gd.tasks[0].x;
    let ty = gd.tasks[0].y;
    let tool = gd.tasks[0].tool;
    gd.gnome.target_x = tx as f32 + 0.5;
    gd.gnome.target_y = ty as f32 + 0.5;
    gd.gnome.target_z = surface_z(tx, ty);
    gd.gnome.active_tool = tool;
    gd.gnome.state = GnomeState::Walking;
    gd.gnome.work_progress = 0;
}

/// Move gnome toward target. Transitions Walking → Working when close enough.
pub fn gnome_move(mut gd: ResMut<GnomeData>) {
    if gd.gnome.state != GnomeState::Walking {
        return;
    }

    let g = &mut gd.gnome;
    let dx = g.target_x - g.x;
    let dy = g.target_y - g.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < 1.0 {
        // Arrived at task location
        g.x = g.target_x;
        g.y = g.target_y;
        g.state = GnomeState::Working;
        g.work_progress = 0;
        return;
    }

    // Walk toward target along surface
    let speed = WALK_SPEED;
    let step = speed.min(dist);
    g.x += dx / dist * step;
    g.y += dy / dist * step;
    // Stay on surface
    g.z = surface_z(g.x as usize, g.y as usize);
}

/// Execute the current task. Applies the tool to the voxel grid after
/// TICKS_PER_TASK ticks of work. Transitions Working → Idle.
pub fn gnome_work(
    mut gd: ResMut<GnomeData>,
    mut grid: ResMut<VoxelGrid>,
    mut seed_map: ResMut<SeedSpeciesMap>,
    _tick: Res<Tick>,
) {
    if gd.gnome.state != GnomeState::Working {
        return;
    }

    gd.gnome.work_progress += 1;
    if gd.gnome.work_progress < TICKS_PER_TASK {
        return;
    }

    // Execute the task
    let task = match gd.tasks.pop_front() {
        Some(t) => t,
        None => {
            gd.gnome.state = GnomeState::Idle;
            return;
        }
    };

    apply_tool(&task, &mut grid, &mut seed_map);

    // Done — return to idle for next plan cycle
    gd.gnome.state = GnomeState::Idle;
    gd.gnome.work_progress = 0;
    gd.gnome.active_tool = 255;
}

/// Pack gnome + ghost data for WASM export. Runs after gnome systems.
pub fn gnome_export(mut gd: ResMut<GnomeData>) {
    gd.pack_export();
    gd.pack_ghosts();
}

// --- Helpers ---

/// Get the surface z coordinate at (x, y) — one voxel above the highest solid.
fn surface_z(x: usize, y: usize) -> f32 {
    // VoxelGrid::surface_height returns the z of the topmost solid voxel at
    // the default terrain level. The gnome walks one voxel above that.
    let z = VoxelGrid::surface_height(x.min(GRID_X - 1), y.min(GRID_Y - 1));
    (z + 1) as f32
}

/// Apply a gardening tool at the task position, same logic as wasm_bridge::place_tool.
fn apply_tool(task: &GnomeTask, grid: &mut VoxelGrid, seed_map: &mut SeedSpeciesMap) {
    let (x, y, z) = (task.x, task.y, task.z);

    let mat = match task.tool {
        0 => Material::Air, // shovel
        1 => Material::Seed,
        2 => Material::Water,
        3 => Material::Soil,
        4 => Material::Stone,
        _ => return,
    };

    if mat == Material::Air {
        // Shovel: dig
        if let Some(voxel) = grid.get_mut(x, y, z) {
            if voxel.material != Material::Air {
                voxel.set_material(Material::Air);
            }
        }
        return;
    }

    // Target must be air
    if let Some(voxel) = grid.get(x, y, z) {
        if voxel.material != Material::Air {
            return;
        }
    } else {
        return;
    }

    // Gravity for non-stone tools
    let landing_z = match mat {
        Material::Stone => z,
        _ => grid.find_landing_z(x, y, z),
    };

    if let Some(voxel) = grid.get(x, y, landing_z) {
        if voxel.material != Material::Air {
            return;
        }
    } else {
        return;
    }

    if let Some(voxel) = grid.get_mut(x, y, landing_z) {
        voxel.set_material(mat);
        if mat == Material::Seed && task.species != 255 {
            seed_map
                .map
                .insert((x, y, landing_z), task.species as usize);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gnome_defaults_to_center() {
        let gd = GnomeData::default();
        assert_eq!(gd.gnome.state, GnomeState::Idle);
        assert!(gd.gnome.x > 0.0);
        assert_eq!(gd.gnome.hunger, 0);
        assert_eq!(gd.gnome.energy, 255);
        assert_eq!(gd.tasks.len(), 0);
    }

    #[test]
    fn queue_and_cancel() {
        let mut gd = GnomeData::default();
        assert!(gd.queue_task(GnomeTask {
            tool: 1,
            x: 10,
            y: 20,
            z: 31,
            species: 0,
        }));
        assert_eq!(gd.queue_len(), 1);

        gd.cancel_at(10, 20, 31);
        assert_eq!(gd.queue_len(), 0);
    }

    #[test]
    fn queue_max_capacity() {
        let mut gd = GnomeData::default();
        for i in 0..MAX_QUEUE {
            assert!(gd.queue_task(GnomeTask {
                tool: 1,
                x: i,
                y: 0,
                z: 31,
                species: 0,
            }));
        }
        assert!(!gd.queue_task(GnomeTask {
            tool: 1,
            x: 999,
            y: 0,
            z: 31,
            species: 0,
        }));
        assert_eq!(gd.queue_len(), MAX_QUEUE);
    }

    #[test]
    fn export_packs_state() {
        let mut gd = GnomeData::default();
        gd.gnome.state = GnomeState::Walking;
        gd.gnome.active_tool = 2;
        gd.gnome.x = 50.0;
        gd.gnome.y = 60.0;
        gd.gnome.z = 31.0;
        gd.queue_task(GnomeTask {
            tool: 1,
            x: 10,
            y: 20,
            z: 31,
            species: 0,
        });
        gd.pack_export();

        let buf = &gd.export_buf;
        assert_eq!(buf[0], GnomeState::Walking as u8);
        assert_eq!(buf[1], 2); // active_tool

        let x = f32::from_le_bytes(buf[4..8].try_into().unwrap());
        assert!((x - 50.0).abs() < 0.001);

        let qlen = u16::from_le_bytes(buf[28..30].try_into().unwrap());
        assert_eq!(qlen, 1);
    }

    #[test]
    fn ghost_export() {
        let mut gd = GnomeData::default();
        gd.queue_task(GnomeTask {
            tool: 1,
            x: 10,
            y: 20,
            z: 31,
            species: 3,
        });
        gd.pack_ghosts();

        assert_eq!(gd.ghost_len(), GHOST_EXPORT_BYTES);
        let x = u16::from_le_bytes(gd.ghost_buf[0..2].try_into().unwrap());
        let y = u16::from_le_bytes(gd.ghost_buf[2..4].try_into().unwrap());
        let z = u16::from_le_bytes(gd.ghost_buf[4..6].try_into().unwrap());
        assert_eq!(x, 10);
        assert_eq!(y, 20);
        assert_eq!(z, 31);
        assert_eq!(gd.ghost_buf[6], 1); // tool = seed
        assert_eq!(gd.ghost_buf[7], 3); // species
    }

    /// Integration test: gnome walks to a task and executes it.
    #[test]
    fn gnome_walks_and_works() {
        let mut world = crate::create_world();
        // Put gnome near center
        let mut gd = GnomeData::default();
        let task_x = (GRID_X / 2) + 3;
        let task_y = (GRID_Y / 2) + 3;
        let task_z = GROUND_LEVEL + 1;

        gd.queue_task(GnomeTask {
            tool: 3, // soil
            x: task_x,
            y: task_y,
            z: task_z,
            species: 255,
        });
        world.insert_resource(gd);

        // Run gnome systems manually
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems((gnome_plan, gnome_move, gnome_work, gnome_export).chain());

        // Run enough ticks for gnome to walk + work
        for _ in 0..30 {
            schedule.run(&mut world);
        }

        let gd = world.resource::<GnomeData>();
        // Task should be consumed
        assert_eq!(gd.tasks.len(), 0);
        assert_eq!(gd.gnome.state, GnomeState::Idle);

        // Soil should be placed (may have fallen via gravity)
        let grid = world.resource::<VoxelGrid>();
        // Check the task position or below it
        let mut found = false;
        for z in 0..=task_z {
            if let Some(v) = grid.get(task_x, task_y, z) {
                if v.material == Material::Soil && z > GROUND_LEVEL - 5 {
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "gnome should have placed soil");
    }
}

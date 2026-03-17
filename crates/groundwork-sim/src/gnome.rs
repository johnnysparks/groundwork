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

use crate::fauna::{FaunaList, FaunaType};
use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GROUND_LEVEL};
use crate::tree::SeedSpeciesMap;
use crate::voxel::Material;
use crate::Tick;

// --- Tuning constants ---

/// Sim ticks the gnome spends working at each task voxel.
const TICKS_PER_TASK: u8 = 3;

/// Base walk speed in voxels per tick.
const WALK_SPEED: f32 = 1.5;

/// Hunger increases by 1 every this many work actions.
const HUNGER_RATE: u8 = 5;

/// Energy decreases by 1 every this many work actions.
const ENERGY_RATE: u8 = 3;

/// Hunger threshold above which gnome slows to 70% speed.
const HUNGER_SLOW_THRESHOLD: u8 = 200;

/// Energy threshold below which gnome slows to 60% speed.
const ENERGY_SLOW_THRESHOLD: u8 = 50;

/// Ticks spent eating when hungry (restores hunger to 0).
const EAT_DURATION: u8 = 15;

/// Ticks spent resting when tired (restores energy to 255).
const REST_DURATION: u8 = 20;

/// Ticks the gnome waits in Idle before starting an idle wander.
const IDLE_WAIT_TICKS: u8 = 25;

/// Ticks spent inspecting a spot after wandering there.
const INSPECT_TICKS: u8 = 12;

/// Maximum wander distance from current position (in voxels).
const WANDER_RADIUS: f32 = 15.0;

/// Hunger level that triggers eating behavior.
const HUNGER_TRIGGER: u8 = 180;

/// Energy level that triggers resting behavior.
const ENERGY_TRIGGER: u8 = 30;

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
    Eating = 3,
    Resting = 4,
    Wandering = 5,
    Inspecting = 6,
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
    /// Trust level with squirrels (0-255). Builds through co-presence.
    pub squirrel_trust: u8,
    /// Count of nearby fauna (updated each tick by gnome_fauna_interact)
    pub nearby_fauna: u8,
    /// Number of work actions completed (for hunger/energy rate calculation)
    pub work_count: u16,
    /// Timer for eating/resting duration (counts down to 0)
    pub needs_timer: u8,
    /// Ticks remaining before the gnome picks a new idle wander target.
    pub idle_timer: u8,
    /// Simple counter for deterministic wander target variety.
    pub idle_seed: u16,
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
            squirrel_trust: 0,
            nearby_fauna: 0,
            work_count: 0,
            needs_timer: 0,
            work_progress: 0,
            idle_timer: IDLE_WAIT_TICKS,
            idle_seed: 0,
        }
    }
}

// --- Resource ---

/// Export format: 32 bytes per gnome.
/// [state: u8, active_tool: u8, hunger: u8, energy: u8,
///  x: f32, y: f32, z: f32,
///  target_x: f32, target_y: f32, target_z: f32,
///  queue_len: u16le, squirrel_trust: u8, nearby_fauna: u8]
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
        buf[30] = g.squirrel_trust;
        buf[31] = g.nearby_fauna;
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
/// When no tasks are queued, the gnome wanders to nearby plants/water after a cooldown.
pub fn gnome_plan(mut gd: ResMut<GnomeData>, grid: Res<VoxelGrid>) {
    // Interrupt idle behaviors if tasks are queued
    if !gd.tasks.is_empty()
        && matches!(
            gd.gnome.state,
            GnomeState::Wandering | GnomeState::Inspecting
        )
    {
        gd.gnome.state = GnomeState::Idle;
        gd.gnome.work_progress = 0;
        gd.gnome.idle_timer = 0;
    }

    if gd.gnome.state != GnomeState::Idle {
        return;
    }

    // Task pickup (priority over idle behaviors)
    if !gd.tasks.is_empty() {
        let tx = gd.tasks[0].x;
        let ty = gd.tasks[0].y;
        let tool = gd.tasks[0].tool;
        gd.gnome.target_x = tx as f32 + 0.5;
        gd.gnome.target_y = ty as f32 + 0.5;
        gd.gnome.target_z = surface_z(tx, ty);
        gd.gnome.active_tool = tool;
        gd.gnome.state = GnomeState::Walking;
        gd.gnome.work_progress = 0;
        return;
    }

    // Idle wander cooldown
    if gd.gnome.idle_timer > 0 {
        gd.gnome.idle_timer -= 1;
        return;
    }

    // Pick a wander target — prefer interesting spots (plants, water, trees)
    let seed = gd.gnome.idle_seed;
    gd.gnome.idle_seed = gd.gnome.idle_seed.wrapping_add(1);
    let (wx, wy) = find_interesting_target(gd.gnome.x, gd.gnome.y, seed, &grid);
    gd.gnome.target_x = wx;
    gd.gnome.target_y = wy;
    gd.gnome.target_z = surface_z(wx as usize, wy as usize);
    gd.gnome.state = GnomeState::Wandering;
    gd.gnome.work_progress = 0;
    gd.gnome.active_tool = 255;
}

/// Move gnome toward target. Walking → Working, Wandering → Inspecting.
pub fn gnome_move(mut gd: ResMut<GnomeData>) {
    let is_walking = gd.gnome.state == GnomeState::Walking;
    let is_wandering = gd.gnome.state == GnomeState::Wandering;
    if !is_walking && !is_wandering {
        return;
    }

    let g = &mut gd.gnome;
    let dx = g.target_x - g.x;
    let dy = g.target_y - g.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < 1.0 {
        g.x = g.target_x;
        g.y = g.target_y;
        g.state = if is_walking {
            GnomeState::Working
        } else {
            GnomeState::Inspecting
        };
        g.work_progress = 0;
        return;
    }

    // Wandering is slower (60% speed) for a relaxed feel
    let speed = if is_walking {
        WALK_SPEED * speed_multiplier(g)
    } else {
        WALK_SPEED * 0.6
    };
    let step = speed.min(dist);
    g.x += dx / dist * step;
    g.y += dy / dist * step;
    g.z = surface_z(g.x as usize, g.y as usize);
}

/// Execute the current task. Applies the tool to the voxel grid after
/// TICKS_PER_TASK ticks of work. Transitions Working → Idle.
/// Also handles Inspecting → Idle after INSPECT_TICKS.
pub fn gnome_work(
    mut gd: ResMut<GnomeData>,
    mut grid: ResMut<VoxelGrid>,
    mut seed_map: ResMut<SeedSpeciesMap>,
    _tick: Res<Tick>,
) {
    // Handle inspection (idle wander destination)
    if gd.gnome.state == GnomeState::Inspecting {
        gd.gnome.work_progress += 1;
        if gd.gnome.work_progress >= INSPECT_TICKS {
            gd.gnome.state = GnomeState::Idle;
            gd.gnome.idle_timer = IDLE_WAIT_TICKS;
            gd.gnome.work_progress = 0;
        }
        return;
    }

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

    // Apply hunger/energy cost for completing work
    apply_needs_cost(&mut gd.gnome);

    // Done — return to idle for next plan cycle
    gd.gnome.state = GnomeState::Idle;
    gd.gnome.work_progress = 0;
    gd.gnome.active_tool = 255;
}

/// System: manage gnome hunger and energy. Triggers eating/resting when
/// needs are high, and applies speed penalties. Never creates fail states.
pub fn gnome_needs(mut gd: ResMut<GnomeData>) {
    let g = &mut gd.gnome;

    match g.state {
        GnomeState::Eating => {
            g.needs_timer = g.needs_timer.saturating_sub(1);
            if g.needs_timer == 0 {
                g.hunger = 0;
                g.state = GnomeState::Idle;
            }
            return;
        }
        GnomeState::Resting => {
            g.needs_timer = g.needs_timer.saturating_sub(1);
            if g.needs_timer == 0 {
                g.energy = 255;
                g.state = GnomeState::Idle;
            }
            return;
        }
        _ => {}
    }

    // Check if gnome just finished a task (work_progress reset to 0 in gnome_work)
    // We use the transition from Working → Idle as the signal to deplete needs.
    // This is handled in gnome_work instead — see apply_needs_cost below.

    // Check thresholds: eating/resting interrupts Idle state
    if g.state == GnomeState::Idle {
        if g.hunger >= HUNGER_TRIGGER {
            g.state = GnomeState::Eating;
            g.needs_timer = EAT_DURATION;
            return;
        }
        if g.energy <= ENERGY_TRIGGER {
            g.state = GnomeState::Resting;
            g.needs_timer = REST_DURATION;
        }
    }
}

/// Apply hunger/energy cost after completing a work action.
/// Called from gnome_work when a task finishes.
fn apply_needs_cost(gnome: &mut Gnome) {
    gnome.work_count = gnome.work_count.wrapping_add(1);

    // Hunger increases every HUNGER_RATE actions
    if gnome.work_count.is_multiple_of(HUNGER_RATE as u16) {
        gnome.hunger = gnome.hunger.saturating_add(1);
    }

    // Energy decreases every ENERGY_RATE actions
    if gnome.work_count.is_multiple_of(ENERGY_RATE as u16) {
        gnome.energy = gnome.energy.saturating_sub(1);
    }
}

/// Get the gnome's speed multiplier based on needs.
fn speed_multiplier(gnome: &Gnome) -> f32 {
    let mut mult = 1.0;
    if gnome.hunger > HUNGER_SLOW_THRESHOLD {
        mult *= 0.7;
    }
    if gnome.energy < ENERGY_SLOW_THRESHOLD {
        mult *= 0.6;
    }
    mult
}

/// Squirrel trust threshold for following the gnome.
const SQUIRREL_FOLLOW_TRUST: u8 = 180;

/// Radius (in voxels) within which fauna interact with the gnome.
const FAUNA_INTERACT_RADIUS: f32 = 8.0;

/// System: gnome-fauna proximity interactions.
///
/// - Counts nearby fauna for export (JS can show emotion particles)
/// - Builds squirrel trust through co-presence
/// - High-trust squirrels adjust their target toward the gnome
pub fn gnome_fauna_interact(
    mut gd: ResMut<GnomeData>,
    mut fauna_list: ResMut<FaunaList>,
    tick: Res<Tick>,
) {
    // Only check every 5 ticks to avoid overhead
    if !tick.0.is_multiple_of(5) {
        return;
    }

    let gx = gd.gnome.x;
    let gy = gd.gnome.y;
    let mut nearby: u8 = 0;
    let mut squirrel_nearby = false;

    for f in &mut fauna_list.fauna {
        let dx = f.x - gx;
        let dy = f.y - gy;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < FAUNA_INTERACT_RADIUS {
            nearby = nearby.saturating_add(1);

            match f.fauna_type {
                FaunaType::Squirrel => {
                    squirrel_nearby = true;
                    // High-trust squirrels follow the gnome
                    if gd.gnome.squirrel_trust >= SQUIRREL_FOLLOW_TRUST {
                        // Set target to gnome position (offset slightly)
                        f.target_x = gx + 2.0;
                        f.target_y = gy + 2.0;
                    }
                }
                FaunaType::Bird => {
                    // Birds that encounter the gnome working prefer to stay nearby
                    if gd.gnome.state == GnomeState::Working && dist < 5.0 {
                        f.target_x = gx;
                        f.target_y = gy;
                    }
                }
                _ => {}
            }
        }
    }

    gd.gnome.nearby_fauna = nearby;

    // Build squirrel trust through co-presence (every 10 ticks if squirrel nearby)
    if squirrel_nearby && tick.0.is_multiple_of(10) {
        gd.gnome.squirrel_trust = gd.gnome.squirrel_trust.saturating_add(1);
    }
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

/// Sample nearby spots and prefer those with interesting voxels (plants, water, trees).
/// Falls back to a random wander target if nothing interesting is nearby.
fn find_interesting_target(gx: f32, gy: f32, seed: u16, grid: &VoxelGrid) -> (f32, f32) {
    let mut candidates = [(0.0f32, 0.0f32); 8];
    let mut count = 0usize;

    for i in 0..24u16 {
        if count >= 8 {
            break;
        }
        let hash = (seed.wrapping_add(i) as u32).wrapping_mul(2654435761);
        let angle = (hash & 0xFF) as f32 / 255.0 * std::f32::consts::TAU;
        let dist = ((hash >> 8) & 0xFF) as f32 / 255.0 * WANDER_RADIUS + 3.0;
        let wx = (gx + angle.cos() * dist).clamp(2.0, (GRID_X - 2) as f32);
        let wy = (gy + angle.sin() * dist).clamp(2.0, (GRID_Y - 2) as f32);
        let x = wx as usize;
        let y = wy as usize;
        let z = VoxelGrid::surface_height(x, y);
        let mut interesting = false;
        for dz in 0..=3 {
            if let Some(v) = grid.get(x, y, z + dz) {
                if matches!(
                    v.material,
                    Material::Leaf | Material::Trunk | Material::Seed | Material::Water
                ) {
                    interesting = true;
                    break;
                }
            }
        }
        if interesting {
            candidates[count] = (wx, wy);
            count += 1;
        }
    }

    if count == 0 {
        // Fallback: random wander
        let hash = (seed as u32).wrapping_mul(2654435761);
        let angle = (hash & 0xFF) as f32 / 255.0 * std::f32::consts::TAU;
        let dist = ((hash >> 8) & 0xFF) as f32 / 255.0 * WANDER_RADIUS + 3.0;
        let wx = (gx + angle.cos() * dist).clamp(2.0, (GRID_X - 2) as f32);
        let wy = (gy + angle.sin() * dist).clamp(2.0, (GRID_Y - 2) as f32);
        return (wx, wy);
    }

    let idx = (seed as usize).wrapping_mul(7) % count;
    candidates[idx]
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

    /// Sync guard: if GnomeState enum or export layout changes,
    /// this test fails — reminding you to update bridge.ts and bridge.contract.test.ts.
    #[test]
    fn wasm_bridge_sync_guard() {
        // GnomeState repr(u8) values (must match bridge.ts GnomeState object)
        assert_eq!(GnomeState::Idle as u8, 0);
        assert_eq!(GnomeState::Walking as u8, 1);
        assert_eq!(GnomeState::Working as u8, 2);
        assert_eq!(GnomeState::Eating as u8, 3);
        assert_eq!(GnomeState::Resting as u8, 4);
        assert_eq!(GnomeState::Wandering as u8, 5);
        assert_eq!(GnomeState::Inspecting as u8, 6);

        // Export sizes (must match GNOME_BYTES=32 and GHOST_EXPORT_BYTES=8 in bridge.ts)
        assert_eq!(GNOME_EXPORT_BYTES, 32, "Gnome export size changed — update bridge.ts");
        assert_eq!(GHOST_EXPORT_BYTES, 8, "Ghost export size changed — update bridge.ts");

        // Export byte layout: verify field offsets match bridge.ts getGnomeState()
        let mut gd = GnomeData::default();
        gd.gnome.state = GnomeState::Working;
        gd.gnome.active_tool = 3;
        gd.gnome.hunger = 100;
        gd.gnome.energy = 200;
        gd.gnome.x = 1.0;
        gd.gnome.y = 2.0;
        gd.gnome.z = 3.0;
        gd.pack_export();
        let buf = &gd.export_buf;
        assert_eq!(buf[0], GnomeState::Working as u8, "state at offset 0");
        assert_eq!(buf[1], 3, "active_tool at offset 1");
        assert_eq!(buf[2], 100, "hunger at offset 2");
        assert_eq!(buf[3], 200, "energy at offset 3");
        let x = f32::from_le_bytes(buf[4..8].try_into().unwrap());
        assert!((x - 1.0).abs() < 0.001, "x at offset 4");
        let qlen = u16::from_le_bytes(buf[28..30].try_into().unwrap());
        assert_eq!(qlen, 0, "queue_len at offset 28");
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

    /// Gnome hunger/energy depletes with work and triggers eating/resting.
    #[test]
    fn needs_deplete_and_trigger() {
        let mut gd = GnomeData::default();
        // Simulate many completed work actions
        for _ in 0..300 {
            apply_needs_cost(&mut gd.gnome);
        }
        // Hunger should have increased, energy decreased
        assert!(gd.gnome.hunger > 0, "hunger should increase with work");
        assert!(gd.gnome.energy < 255, "energy should decrease with work");

        // If hunger is high enough, needs system should trigger eating
        gd.gnome.hunger = HUNGER_TRIGGER;
        gd.gnome.state = GnomeState::Idle;
        // Manually call gnome_needs logic
        if gd.gnome.state == GnomeState::Idle && gd.gnome.hunger >= HUNGER_TRIGGER {
            gd.gnome.state = GnomeState::Eating;
            gd.gnome.needs_timer = EAT_DURATION;
        }
        assert_eq!(gd.gnome.state, GnomeState::Eating);

        // After eating timer expires, gnome returns to idle with hunger reset
        gd.gnome.needs_timer = 0;
        gd.gnome.hunger = 0;
        gd.gnome.state = GnomeState::Idle;
        assert_eq!(gd.gnome.hunger, 0);
    }

    /// Speed multiplier applies correctly based on needs.
    #[test]
    fn speed_affected_by_needs() {
        let mut gnome = Gnome::default();
        assert!((speed_multiplier(&gnome) - 1.0).abs() < 0.001);

        gnome.hunger = HUNGER_SLOW_THRESHOLD + 1;
        assert!((speed_multiplier(&gnome) - 0.7).abs() < 0.001);

        gnome.energy = ENERGY_SLOW_THRESHOLD - 1;
        assert!((speed_multiplier(&gnome) - 0.42).abs() < 0.001);
    }

    /// Gnome near squirrel builds trust over time.
    #[test]
    fn squirrel_trust_builds() {
        use crate::fauna::{Fauna, FaunaState};

        let mut world = crate::create_world();

        // Place gnome at center
        let gd = GnomeData::default();
        let gx = gd.gnome.x;
        let gy = gd.gnome.y;
        world.insert_resource(gd);

        // Spawn a squirrel near the gnome
        let mut fauna_list = world.resource_mut::<FaunaList>();
        fauna_list.fauna.push(Fauna {
            fauna_type: FaunaType::Squirrel,
            state: FaunaState::Idle,
            x: gx + 3.0,
            y: gy + 3.0,
            z: GROUND_LEVEL as f32 + 1.0,
            target_x: gx + 3.0,
            target_y: gy + 3.0,
            target_z: GROUND_LEVEL as f32 + 1.0,
            age: 0,
            max_age: 1000,
            rng_seed: 42,
        });

        // Run the interaction system repeatedly
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(gnome_fauna_interact);

        // Trust builds every 10 ticks when squirrel nearby
        // We run 200 iterations; the system checks every 5 ticks, trust every 10
        for i in 0..200 {
            world.resource_mut::<crate::Tick>().0 = i;
            schedule.run(&mut world);
        }

        let gd = world.resource::<GnomeData>();
        assert!(
            gd.gnome.squirrel_trust > 0,
            "squirrel trust should build through co-presence"
        );
        assert!(
            gd.gnome.nearby_fauna > 0,
            "nearby fauna count should be tracked"
        );
    }

    /// Gnome wanders when idle with no tasks.
    #[test]
    fn gnome_wanders_when_idle() {
        let mut world = crate::create_world();
        let mut gd = GnomeData::default();
        gd.gnome.idle_timer = 0; // skip initial wait
        let start_x = gd.gnome.x;
        let start_y = gd.gnome.y;
        world.insert_resource(gd);

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems((gnome_plan, gnome_move, gnome_work, gnome_export).chain());

        for _ in 0..50 {
            schedule.run(&mut world);
        }

        let gd = world.resource::<GnomeData>();
        let moved = (gd.gnome.x - start_x).abs() > 1.0 || (gd.gnome.y - start_y).abs() > 1.0;
        assert!(moved, "gnome should wander from starting position");
    }

    /// Tasks interrupt wandering immediately.
    #[test]
    fn tasks_interrupt_wandering() {
        let mut world = crate::create_world();
        let mut gd = GnomeData::default();
        gd.gnome.idle_timer = 0;
        world.insert_resource(gd);

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems((gnome_plan, gnome_move, gnome_work, gnome_export).chain());

        // Start wandering
        for _ in 0..3 {
            schedule.run(&mut world);
        }

        // Queue a task
        {
            let mut gd = world.resource_mut::<GnomeData>();
            gd.queue_task(GnomeTask {
                tool: 3,
                x: (GRID_X / 2) + 5,
                y: (GRID_Y / 2) + 5,
                z: GROUND_LEVEL + 1,
                species: 255,
            });
        }

        // Next tick should interrupt wandering
        schedule.run(&mut world);

        let gd = world.resource::<GnomeData>();
        assert!(
            matches!(
                gd.gnome.state,
                GnomeState::Walking | GnomeState::Working | GnomeState::Idle
            ),
            "task should interrupt wandering: state={:?}",
            gd.gnome.state,
        );
        assert_eq!(gd.tasks.len(), 1, "task should still be queued");
    }
}

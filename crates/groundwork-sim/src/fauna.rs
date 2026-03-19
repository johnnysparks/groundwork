//! Fauna system: visible ecological agents that connect the garden.
//!
//! Fauna types and their ecological roles:
//! - **Bee/Butterfly** (pollinators): spawn near flower clusters, drift between
//!   flowers, boost seed production of visited plants.
//! - **Bird**: spawn near berry bushes, carry seeds to distant locations,
//!   creating "gift" plantings the player didn't plan.
//! - **Worm**: spawn in moist soil with organic matter, enrich soil nutrients,
//!   leave visible trails underground.
//! - **Beetle** (decomposer): spawn near dead wood, accelerate decomposition
//!   into nutrient-rich soil.
//!
//! Fauna are NOT full ECS entities — they're lightweight structs stored in a
//! flat Vec (like VoxelGrid). The web renderer reads them via zero-copy typed
//! array views through the WASM bridge.

use bevy_ecs::prelude::*;

use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z};
use crate::soil::SoilGrid;
use crate::tree::tree_hash;
use crate::voxel::Material;
use crate::Tick;

/// Fauna species type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FaunaType {
    Bee = 0,
    Butterfly = 1,
    Bird = 2,
    Worm = 3,
    Beetle = 4,
    Squirrel = 5,
}

/// Movement behavior state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FaunaState {
    /// Wandering around a home area
    Idle = 0,
    /// Moving toward a target (flower, dead wood, etc.)
    Seeking = 1,
    /// Performing ecological action (pollinating, decomposing)
    Acting = 2,
    /// Departing the garden (lifetime ending)
    Leaving = 3,
}

/// A single fauna creature. Packed for zero-copy WASM transfer.
///
/// Layout: 16 bytes per fauna (see FAUNA_BYTES).
/// [type: u8, state: u8, pad: u8, pad: u8,
///  x: f32, y: f32, z: f32]
///
/// We use a simpler flat struct for internal sim, then pack for export.
#[derive(Clone, Debug)]
pub struct Fauna {
    pub fauna_type: FaunaType,
    pub state: FaunaState,
    /// Position in voxel coordinates (fractional for smooth movement)
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Target position (for seeking behavior)
    pub target_x: f32,
    pub target_y: f32,
    pub target_z: f32,
    /// Ticks alive
    pub age: u32,
    /// Maximum lifetime in ticks
    pub max_age: u32,
    /// RNG seed for deterministic behavior
    pub rng_seed: u64,
}

/// Packed fauna data for WASM export. 16 bytes per fauna.
/// [type: u8, state: u8, _pad: u8, _pad: u8, x: f32, y: f32, z: f32]
pub const FAUNA_EXPORT_BYTES: usize = 16;

/// Maximum number of fauna alive at once.
pub const MAX_FAUNA: usize = 128;

/// Resource holding all active fauna.
#[derive(Resource)]
pub struct FaunaList {
    pub fauna: Vec<Fauna>,
    /// Packed export buffer for zero-copy WASM access
    export_buf: Vec<u8>,
    /// Counter for deterministic RNG
    spawn_counter: u64,
}

impl Default for FaunaList {
    fn default() -> Self {
        Self {
            fauna: Vec::with_capacity(MAX_FAUNA),
            export_buf: vec![0u8; MAX_FAUNA * FAUNA_EXPORT_BYTES],
            spawn_counter: 0,
        }
    }
}

impl FaunaList {
    /// Pack fauna data into the export buffer for WASM bridge.
    pub fn pack_export(&mut self) {
        for (i, f) in self.fauna.iter().enumerate() {
            if i >= MAX_FAUNA {
                break;
            }
            let off = i * FAUNA_EXPORT_BYTES;
            self.export_buf[off] = f.fauna_type as u8;
            self.export_buf[off + 1] = f.state as u8;
            self.export_buf[off + 2] = 0; // pad
            self.export_buf[off + 3] = 0; // pad
            let x_bytes = f.x.to_le_bytes();
            let y_bytes = f.y.to_le_bytes();
            let z_bytes = f.z.to_le_bytes();
            self.export_buf[off + 4..off + 8].copy_from_slice(&x_bytes);
            self.export_buf[off + 8..off + 12].copy_from_slice(&y_bytes);
            self.export_buf[off + 12..off + 16].copy_from_slice(&z_bytes);
        }
    }

    /// Pointer to export buffer.
    pub fn export_ptr(&self) -> *const u8 {
        self.export_buf.as_ptr()
    }

    /// Length of export data in bytes (only active fauna).
    pub fn export_len(&self) -> usize {
        self.fauna.len().min(MAX_FAUNA) * FAUNA_EXPORT_BYTES
    }

    /// Number of active fauna.
    pub fn count(&self) -> usize {
        self.fauna.len()
    }

    fn next_seed(&mut self) -> u64 {
        self.spawn_counter += 1;
        self.spawn_counter
    }
}

/// Count occurrences of a material in a neighborhood around (cx, cy, cz).
fn count_material_nearby(
    grid: &VoxelGrid,
    cx: usize,
    cy: usize,
    cz: usize,
    radius: usize,
    mat: Material,
) -> usize {
    let mut count = 0;
    let x_lo = cx.saturating_sub(radius);
    let x_hi = (cx + radius).min(GRID_X - 1);
    let y_lo = cy.saturating_sub(radius);
    let y_hi = (cy + radius).min(GRID_Y - 1);
    let z_lo = cz.saturating_sub(radius);
    let z_hi = (cz + radius).min(GRID_Z - 1);
    for z in z_lo..=z_hi {
        for y in y_lo..=y_hi {
            for x in x_lo..=x_hi {
                if let Some(v) = grid.get(x, y, z) {
                    if v.material == mat {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

/// Find a random position of a given material in a radius. Returns None if none found.
fn find_material_nearby(
    grid: &VoxelGrid,
    cx: usize,
    cy: usize,
    cz: usize,
    radius: usize,
    mat: Material,
    rng: u64,
) -> Option<(usize, usize, usize)> {
    let mut candidates: Vec<(usize, usize, usize)> = Vec::new();
    let x_lo = cx.saturating_sub(radius);
    let x_hi = (cx + radius).min(GRID_X - 1);
    let y_lo = cy.saturating_sub(radius);
    let y_hi = (cy + radius).min(GRID_Y - 1);
    let z_lo = cz.saturating_sub(radius);
    let z_hi = (cz + radius).min(GRID_Z - 1);
    for z in z_lo..=z_hi {
        for y in y_lo..=y_hi {
            for x in x_lo..=x_hi {
                if let Some(v) = grid.get(x, y, z) {
                    if v.material == mat {
                        candidates.push((x, y, z));
                    }
                }
            }
        }
    }
    if candidates.is_empty() {
        None
    } else {
        let idx = (tree_hash(rng, 42) as usize) % candidates.len();
        Some(candidates[idx])
    }
}

/// System: spawn fauna based on ecological conditions.
///
/// Runs every 20 ticks to avoid overhead. Checks for:
/// - Flower density → spawn pollinators (bees, butterflies)
/// - Berry/fruit bushes → spawn birds
/// - Moist soil with organic matter → spawn worms
/// - Dead wood → spawn beetles
pub fn fauna_spawn(
    grid: Res<VoxelGrid>,
    soil: Res<SoilGrid>,
    tick: Res<Tick>,
    mut fauna_list: ResMut<FaunaList>,
) {
    // Only check every 20 ticks
    if !tick.0.is_multiple_of(20) {
        return;
    }

    if fauna_list.fauna.len() >= MAX_FAUNA {
        return;
    }

    let t = tick.0;

    // Sample several spots across the garden for ecological conditions
    let sample_points: [(usize, usize); 9] = [
        (GRID_X / 4, GRID_Y / 4),
        (GRID_X / 2, GRID_Y / 4),
        (3 * GRID_X / 4, GRID_Y / 4),
        (GRID_X / 4, GRID_Y / 2),
        (GRID_X / 2, GRID_Y / 2),
        (3 * GRID_X / 4, GRID_Y / 2),
        (GRID_X / 4, 3 * GRID_Y / 4),
        (GRID_X / 2, 3 * GRID_Y / 4),
        (3 * GRID_X / 4, 3 * GRID_Y / 4),
    ];

    for (si, &(sx, sy)) in sample_points.iter().enumerate() {
        if fauna_list.fauna.len() >= MAX_FAUNA {
            break;
        }

        let surface = VoxelGrid::surface_height(sx, sy);
        let sz = surface + 2;
        let h = tree_hash(t + si as u64, 999);

        // --- Pollinators: spawn near leaf clusters (flowers/foliage) ---
        // Flower species (wildflower=7, daisy=8) double the spawn probability,
        // creating pollinator "meadows" when flowers cluster together.
        // Discovery: "My flower patch is swarming with bees!"
        let leaf_count = count_material_nearby(&grid, sx, sy, sz, 8, Material::Leaf);
        let flower_bonus = {
            let mut flower_leaves = 0u64;
            let r = 6_usize;
            for fz in surface..=(surface + 3).min(GRID_Z - 1) {
                for fy in sy.saturating_sub(r)..=(sy + r).min(GRID_Y - 1) {
                    for fx in sx.saturating_sub(r)..=(sx + r).min(GRID_X - 1) {
                        if let Some(v) = grid.get(fx, fy, fz) {
                            if v.material == Material::Leaf
                                && (v.nutrient_level == 7 || v.nutrient_level == 8)
                            {
                                flower_leaves += 1;
                            }
                        }
                    }
                }
            }
            if flower_leaves >= 5 {
                2u64
            } else {
                1
            }
        };
        if leaf_count >= 6 {
            // Probability increases with more flowers; flower clusters double it
            let prob = (leaf_count as u64).min(30) * flower_bonus;
            let max_pollinators = if flower_bonus > 1 { 5 } else { 3 }; // meadows support more
            if h % 100 < prob
                && count_type_nearby(&fauna_list, sx as f32, sy as f32, FaunaType::Bee)
                    < max_pollinators
            {
                let seed = fauna_list.next_seed();
                let fauna_type = if tree_hash(seed, 1).is_multiple_of(3) {
                    FaunaType::Butterfly
                } else {
                    FaunaType::Bee
                };
                fauna_list.fauna.push(Fauna {
                    fauna_type,
                    state: FaunaState::Idle,
                    x: sx as f32 + 0.5,
                    y: sy as f32 + 0.5,
                    z: sz as f32 + 2.0,
                    target_x: sx as f32,
                    target_y: sy as f32,
                    target_z: sz as f32 + 1.0,
                    age: 0,
                    max_age: 200 + (tree_hash(seed, 2) % 200) as u32,
                    rng_seed: seed,
                });
            }
        }

        // --- Birds: spawn near trunk/branch clusters or berry bushes ---
        // Berry bushes (species_id=5) attract birds at a lower threshold.
        let trunk_count = count_material_nearby(&grid, sx, sy, sz + 5, 10, Material::Trunk);
        let branch_count = count_material_nearby(&grid, sx, sy, sz + 5, 10, Material::Branch);
        let leaf_count_birds = count_material_nearby(&grid, sx, sy, sz + 2, 6, Material::Leaf);
        // Berry bushes make it easier for birds to spawn (lower threshold)
        let bird_threshold_met = trunk_count + branch_count >= 8 || leaf_count_birds >= 12;
        if bird_threshold_met {
            let h2 = tree_hash(t + si as u64, 1001);
            if h2 % 150 < 10
                && count_type_nearby(&fauna_list, sx as f32, sy as f32, FaunaType::Bird) < 2
            {
                let seed = fauna_list.next_seed();
                // Birds spawn higher up, near canopy
                let bz = sz as f32 + 8.0 + (tree_hash(seed, 3) % 5) as f32;
                fauna_list.fauna.push(Fauna {
                    fauna_type: FaunaType::Bird,
                    state: FaunaState::Idle,
                    x: sx as f32 + 0.5,
                    y: sy as f32 + 0.5,
                    z: bz,
                    target_x: sx as f32,
                    target_y: sy as f32,
                    target_z: bz,
                    age: 0,
                    max_age: 300 + (tree_hash(seed, 4) % 300) as u32,
                    rng_seed: seed,
                });
            }
        }

        // --- Worms: spawn in moist soil underground ---
        let underground_z = surface.saturating_sub(3);
        if let Some(v) = grid.get(sx, sy, underground_z) {
            if v.material == Material::Soil && v.water_level > 50 {
                if let Some(sc) = soil.get(sx, sy, underground_z) {
                    if sc.organic > 20 {
                        let h3 = tree_hash(t + si as u64, 1003);
                        if h3 % 200 < 15
                            && count_type_nearby(&fauna_list, sx as f32, sy as f32, FaunaType::Worm)
                                < 2
                        {
                            let seed = fauna_list.next_seed();
                            fauna_list.fauna.push(Fauna {
                                fauna_type: FaunaType::Worm,
                                state: FaunaState::Idle,
                                x: sx as f32 + 0.5,
                                y: sy as f32 + 0.5,
                                z: underground_z as f32 + 0.5,
                                target_x: sx as f32,
                                target_y: sy as f32,
                                target_z: underground_z as f32,
                                age: 0,
                                max_age: 400 + (tree_hash(seed, 5) % 200) as u32,
                                rng_seed: seed,
                            });
                        }
                    }
                }
            }
        }

        // --- Beetles: spawn near dead wood ---
        let deadwood_count = count_material_nearby(&grid, sx, sy, sz, 6, Material::DeadWood);
        if deadwood_count >= 2 {
            let h4 = tree_hash(t + si as u64, 1005);
            if h4 % 120 < 15
                && count_type_nearby(&fauna_list, sx as f32, sy as f32, FaunaType::Beetle) < 2
            {
                let seed = fauna_list.next_seed();
                // Find a dead wood voxel to spawn near
                if let Some((dx, dy, dz)) =
                    find_material_nearby(&grid, sx, sy, sz, 6, Material::DeadWood, seed)
                {
                    fauna_list.fauna.push(Fauna {
                        fauna_type: FaunaType::Beetle,
                        state: FaunaState::Idle,
                        x: dx as f32 + 0.5,
                        y: dy as f32 + 0.5,
                        z: dz as f32 + 1.0,
                        target_x: dx as f32,
                        target_y: dy as f32,
                        target_z: dz as f32,
                        age: 0,
                        max_age: 250 + (tree_hash(seed, 6) % 150) as u32,
                        rng_seed: seed,
                    });
                }
            }
        }

        // --- Squirrels: spawn near oaks and berry bushes ---
        // Squirrels are companion fauna: they cache acorns that sprout into
        // new oaks, creating "gift" plantings the player didn't plan.
        // Check for oak/berry leaf voxels (species_id 0 or 5 in nutrient_level).
        {
            let mut oak_berry_count = 0u32;
            let sqr = 8_usize;
            for sqz in surface..=(surface + 6).min(GRID_Z - 1) {
                for sqy in sy.saturating_sub(sqr)..=(sy + sqr).min(GRID_Y - 1) {
                    for sqx in sx.saturating_sub(sqr)..=(sx + sqr).min(GRID_X - 1) {
                        if let Some(v) = grid.get(sqx, sqy, sqz) {
                            if (v.material == Material::Trunk || v.material == Material::Leaf)
                                && (v.nutrient_level == 0 || v.nutrient_level == 5)
                            {
                                oak_berry_count += 1;
                            }
                        }
                    }
                }
            }
            if oak_berry_count >= 10 {
                let h5 = tree_hash(t + si as u64, 1007);
                if h5 % 200 < 15
                    && count_type_nearby(&fauna_list, sx as f32, sy as f32, FaunaType::Squirrel) < 2
                {
                    let seed = fauna_list.next_seed();
                    fauna_list.fauna.push(Fauna {
                        fauna_type: FaunaType::Squirrel,
                        state: FaunaState::Idle,
                        x: sx as f32 + 0.5,
                        y: sy as f32 + 0.5,
                        z: surface as f32 + 1.5,
                        target_x: sx as f32,
                        target_y: sy as f32,
                        target_z: surface as f32 + 1.0,
                        age: 0,
                        max_age: 500 + (tree_hash(seed, 7) % 300) as u32,
                        rng_seed: seed,
                    });
                }
            }
        }
    }
}

/// Count how many fauna of a given type are near a position.
fn count_type_nearby(fauna_list: &FaunaList, x: f32, y: f32, fauna_type: FaunaType) -> usize {
    let radius = 15.0f32;
    fauna_list
        .fauna
        .iter()
        .filter(|f| {
            f.fauna_type == fauna_type && (f.x - x).abs() < radius && (f.y - y).abs() < radius
        })
        .count()
}

/// System: update fauna movement and behavior.
///
/// - Pollinators drift between flowers in gentle sine-wave paths
/// - Birds circle above the canopy, occasionally swooping
/// - Worms move slowly underground through soil
/// - Beetles crawl on dead wood surfaces
pub fn fauna_update(grid: Res<VoxelGrid>, mut fauna_list: ResMut<FaunaList>, tick: Res<Tick>) {
    let t = tick.0;

    // Update each fauna's position and state
    for i in 0..fauna_list.fauna.len() {
        let f = &mut fauna_list.fauna[i];
        f.age += 1;

        // Check lifetime
        if f.age >= f.max_age {
            f.state = FaunaState::Leaving;
        }

        let h = tree_hash(f.rng_seed, f.age as u64);

        match f.fauna_type {
            FaunaType::Bee | FaunaType::Butterfly => {
                // Gentle drifting flight pattern with sine-wave offsets
                let speed = if f.fauna_type == FaunaType::Butterfly {
                    0.08
                } else {
                    0.12
                };
                let t_f = t as f32 + f.rng_seed as f32 * 0.1;

                match f.state {
                    FaunaState::Idle => {
                        // Gentle hovering with figure-8 pattern
                        let phase = f.rng_seed as f32 * 0.37;
                        f.x += (t_f * 0.05 + phase).sin() * speed;
                        f.y += (t_f * 0.04 + phase * 1.3).cos() * speed;
                        f.z += (t_f * 0.08 + phase).sin() * 0.02;

                        // Every ~30 ticks, pick a new flower target
                        if f.age.is_multiple_of(30) {
                            if let Some((tx, ty, tz)) = find_material_nearby(
                                &grid,
                                f.x as usize,
                                f.y as usize,
                                f.z as usize,
                                10,
                                Material::Leaf,
                                h,
                            ) {
                                f.target_x = tx as f32 + 0.5;
                                f.target_y = ty as f32 + 0.5;
                                f.target_z = tz as f32 + 1.5;
                                f.state = FaunaState::Seeking;
                            }
                        }
                    }
                    FaunaState::Seeking => {
                        // Move toward target with gentle curve
                        let dx = f.target_x - f.x;
                        let dy = f.target_y - f.y;
                        let dz = f.target_z - f.z;
                        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                        if dist < 1.5 {
                            f.state = FaunaState::Acting;
                        } else {
                            let move_speed = speed * 1.5;
                            f.x += dx / dist * move_speed;
                            f.y += dy / dist * move_speed;
                            f.z += dz / dist * move_speed;
                            // Add gentle wobble
                            f.x += (t_f * 0.1).sin() * 0.03;
                            f.y += (t_f * 0.12).cos() * 0.03;
                        }
                    }
                    FaunaState::Acting => {
                        // Hover near flower for a few ticks, then return to idle
                        f.x += (t_f * 0.15).sin() * 0.02;
                        f.y += (t_f * 0.12).cos() * 0.02;
                        if f.age.is_multiple_of(15) {
                            f.state = FaunaState::Idle;
                        }
                    }
                    FaunaState::Leaving => {
                        // Rise and drift away
                        f.z += 0.15;
                        f.x += (h % 3) as f32 * 0.1 - 0.1;
                        f.y += ((h >> 2) % 3) as f32 * 0.1 - 0.1;
                    }
                }

                // Keep pollinators above ground
                let surface = VoxelGrid::surface_height(f.x as usize, f.y as usize);
                let min_z = surface as f32 + 1.0;
                if f.z < min_z {
                    f.z = min_z;
                }
            }
            FaunaType::Bird => {
                // Circling flight pattern above canopy
                let circle_speed = 0.03;
                let circle_radius = 5.0 + (f.rng_seed % 5) as f32;
                let phase = f.rng_seed as f32 * 0.5;
                let angle = t as f32 * circle_speed + phase;

                match f.state {
                    FaunaState::Idle | FaunaState::Seeking => {
                        // Circle pattern
                        let center_x = f.target_x;
                        let center_y = f.target_y;
                        f.x = center_x + angle.cos() * circle_radius;
                        f.y = center_y + angle.sin() * circle_radius;
                        // Gentle bobbing
                        f.z = f.target_z + (angle * 2.0).sin() * 1.0;
                    }
                    FaunaState::Acting => {
                        // Swooping down briefly
                        f.z -= 0.2;
                        let surface = VoxelGrid::surface_height(f.x as usize, f.y as usize);
                        if f.z < surface as f32 + 3.0 {
                            f.state = FaunaState::Idle;
                            f.z = f.target_z;
                        }
                    }
                    FaunaState::Leaving => {
                        f.z += 0.2;
                        f.x += 0.15;
                    }
                }

                // Occasionally swoop
                if f.age.is_multiple_of(80) && f.state == FaunaState::Idle {
                    f.state = FaunaState::Acting;
                }
            }
            FaunaType::Worm => {
                // Slow underground movement through soil
                let speed = 0.03;
                let phase = f.rng_seed as f32 * 0.7;

                match f.state {
                    FaunaState::Idle | FaunaState::Seeking => {
                        // Sinusoidal path through soil
                        let t_f = t as f32 * 0.02 + phase;
                        f.x += t_f.sin() * speed;
                        f.y += (t_f * 1.3).cos() * speed;
                        // Occasionally change depth slightly
                        if h.is_multiple_of(50) {
                            f.z += if h.is_multiple_of(2) { 0.1 } else { -0.1 };
                        }
                    }
                    _ => {}
                }

                // Keep underground
                let surface = VoxelGrid::surface_height(f.x as usize, f.y as usize);
                let max_z = surface as f32 - 0.5;
                let min_z = (surface as f32 - 8.0).max(1.0);
                f.z = f.z.clamp(min_z, max_z);
            }
            FaunaType::Beetle => {
                // Crawl on surfaces near dead wood
                let speed = 0.04;
                let phase = f.rng_seed as f32 * 1.1;
                let t_f = t as f32 * 0.03 + phase;

                f.x += t_f.sin() * speed;
                f.y += (t_f * 0.8).cos() * speed;

                // Keep near surface
                let surface = VoxelGrid::surface_height(f.x as usize, f.y as usize);
                let min_z = surface as f32;
                if f.z < min_z {
                    f.z = min_z + 0.5;
                }
            }
            FaunaType::Squirrel => {
                // Squirrels scurry along the ground, darting between trees.
                // Fast erratic movement with pauses (Idle → Seeking → Acting cycle).
                let surface = VoxelGrid::surface_height(f.x as usize, f.y as usize);
                let speed = 0.15;
                let phase = f.rng_seed as f32 * 0.9;
                let t_f = t as f32 + phase;

                match f.state {
                    FaunaState::Idle => {
                        // Brief pause, twitch in place
                        f.x += (t_f * 0.3).sin() * 0.01;
                        f.y += (t_f * 0.4).cos() * 0.01;
                        // Start seeking a new tree every ~20 ticks
                        if f.age.is_multiple_of(20) {
                            // Pick a random nearby target
                            let h2 = tree_hash(f.rng_seed, f.age as u64);
                            let dist = 5.0 + (h2 % 10) as f32;
                            let angle = (h2 >> 8) as f32 * 0.1;
                            f.target_x = f.x + angle.cos() * dist;
                            f.target_y = f.y + angle.sin() * dist;
                            f.target_x = f.target_x.clamp(2.0, (GRID_X - 3) as f32);
                            f.target_y = f.target_y.clamp(2.0, (GRID_Y - 3) as f32);
                            f.state = FaunaState::Seeking;
                        }
                    }
                    FaunaState::Seeking => {
                        // Dart toward target
                        let dx = f.target_x - f.x;
                        let dy = f.target_y - f.y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < 1.0 {
                            f.state = FaunaState::Acting; // arrived — cache an acorn
                        } else {
                            f.x += dx / dist * speed;
                            f.y += dy / dist * speed;
                            // Bobbing motion
                            f.z = surface as f32 + 1.0 + (t_f * 0.4).sin().abs() * 0.3;
                        }
                    }
                    FaunaState::Acting => {
                        // Brief digging animation, then return to idle
                        f.z = surface as f32 + 0.8;
                        if f.age.is_multiple_of(5) {
                            f.state = FaunaState::Idle;
                            f.z = surface as f32 + 1.0;
                        }
                    }
                    FaunaState::Leaving => {
                        f.x += 0.2;
                        f.y += 0.1;
                    }
                }

                // Keep on ground surface
                f.z =
                    f.z.clamp(surface as f32 + 0.5, surface as f32 + 2.0);
            }
        }

        // Clamp to grid bounds
        f.x = f.x.clamp(1.0, (GRID_X - 2) as f32);
        f.y = f.y.clamp(1.0, (GRID_Y - 2) as f32);
        f.z = f.z.clamp(1.0, (GRID_Z - 2) as f32);
    }

    // Remove fauna that have left or exceeded lifetime
    fauna_list.fauna.retain(|f| {
        if f.state == FaunaState::Leaving {
            f.z > (GRID_Z - 3) as f32 || f.age < f.max_age + 50
        } else {
            true
        }
    });
    // Final removal for truly gone fauna
    fauna_list
        .fauna
        .retain(|f| !(f.state == FaunaState::Leaving && f.age >= f.max_age + 50));

    // Pack for WASM export
    fauna_list.pack_export();
}

/// System: apply fauna ecological effects to the simulation.
///
/// - Pollinators near flowers: boost nearby seed nutrient levels
/// - Worms in soil: increase soil organic content and bacteria
/// - Beetles near dead wood: accelerate decomposition
/// - Birds: carry species-specific seeds, enrich soil with droppings
pub fn fauna_effects(
    mut grid: ResMut<VoxelGrid>,
    mut soil: ResMut<SoilGrid>,
    mut seed_map: ResMut<crate::tree::SeedSpeciesMap>,
    fauna_list: Res<FaunaList>,
    tick: Res<Tick>,
) {
    // Only apply effects every 10 ticks to reduce overhead
    if !tick.0.is_multiple_of(10) {
        return;
    }

    for f in &fauna_list.fauna {
        match f.fauna_type {
            FaunaType::Bee | FaunaType::Butterfly => {
                // Pollinators boost seed nutrient levels of nearby seeds
                if f.state == FaunaState::Acting || f.state == FaunaState::Idle {
                    let cx = f.x as usize;
                    let cy = f.y as usize;
                    let cz = f.z as usize;
                    // Boost nutrient level of nearby seeds (simulates pollination)
                    for dz in 0..3 {
                        for dy in 0..3_usize {
                            for dx in 0..3_usize {
                                let nx = cx.wrapping_add(dx).wrapping_sub(1);
                                let ny = cy.wrapping_add(dy).wrapping_sub(1);
                                let nz = cz.wrapping_add(dz).wrapping_sub(1);
                                if let Some(v) = grid.get_mut(nx, ny, nz) {
                                    if v.material == Material::Seed {
                                        v.nutrient_level = v.nutrient_level.saturating_add(5);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            FaunaType::Worm => {
                // Worms enrich soil: increase organic content and bacteria
                let cx = f.x as usize;
                let cy = f.y as usize;
                let cz = f.z as usize;
                if let Some(sc) = soil.get_mut(cx, cy, cz) {
                    sc.organic = sc.organic.saturating_add(1);
                    sc.bacteria = sc.bacteria.saturating_add(2);
                }
                // Also boost adjacent soil nutrients
                if let Some(v) = grid.get_mut(cx, cy, cz) {
                    if v.material == Material::Soil {
                        v.nutrient_level = v.nutrient_level.saturating_add(2);
                    }
                }
            }
            FaunaType::Beetle => {
                // Beetles accelerate dead wood decomposition
                let cx = f.x as usize;
                let cy = f.y as usize;
                let cz = f.z as usize;
                for dz in 0..3 {
                    for dy in 0..3_usize {
                        for dx in 0..3_usize {
                            let nx = cx.wrapping_add(dx).wrapping_sub(1);
                            let ny = cy.wrapping_add(dy).wrapping_sub(1);
                            let nz = cz.wrapping_add(dz).wrapping_sub(1);
                            if let Some(v) = grid.get_mut(nx, ny, nz) {
                                if v.material == Material::DeadWood {
                                    // Accelerate decomposition: increase nutrient on dead wood
                                    v.nutrient_level = v.nutrient_level.saturating_add(3);
                                    // When nutrient_level is high enough, convert to soil
                                    if v.nutrient_level > 200 {
                                        v.set_material(Material::Soil);
                                        v.nutrient_level = 80;
                                        // Enrich soil composition
                                        if let Some(sc) = soil.get_mut(nx, ny, nz) {
                                            sc.organic = sc.organic.saturating_add(30);
                                            sc.bacteria = sc.bacteria.saturating_add(20);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            FaunaType::Bird => {
                // Bird Express: birds carry species-specific seeds to distant locations.
                // Birds also enrich soil with droppings (nitrogen) near where they perch.
                // Discovery: "Berry bushes attract birds → birds spread berry seeds →
                //             bird droppings fertilize soil → better growth nearby."
                if f.state == FaunaState::Acting || f.state == FaunaState::Idle {
                    let cx = f.x as usize;
                    let cy = f.y as usize;
                    let cz = f.z as usize;

                    // Find nearby plant voxels and determine species
                    let mut near_tree = false;
                    let mut nearby_species: Option<u8> = None;
                    for dz in 0..5_usize {
                        for dy in 0..3_usize {
                            for dx in 0..3_usize {
                                let nx = cx.wrapping_add(dx).wrapping_sub(1);
                                let ny = cy.wrapping_add(dy).wrapping_sub(1);
                                let nz = cz.wrapping_sub(dz);
                                if let Some(v) = grid.get(nx, ny, nz) {
                                    if v.material == Material::Leaf || v.material == Material::Trunk
                                    {
                                        near_tree = true;
                                        // nutrient_level stores species_id on plant voxels
                                        if nearby_species.is_none() {
                                            nearby_species = Some(v.nutrient_level);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Bird droppings: enrich soil below the bird's position
                    // Creates nutrient hotspots that benefit nearby plants
                    let ground_z = VoxelGrid::surface_height(cx, cy);
                    if let Some(soil_cell) = grid.get_mut(cx, cy, ground_z) {
                        if soil_cell.material == Material::Soil {
                            soil_cell.nutrient_level = soil_cell.nutrient_level.saturating_add(3);
                        }
                    }
                    // Also boost soil composition (organic matter from droppings)
                    if let Some(sc) = soil.get_mut(cx, cy, ground_z) {
                        sc.organic = sc.organic.saturating_add(2);
                    }

                    if near_tree {
                        // ~5% chance per effect tick to drop a seed
                        let h = tree_hash(tick.0 + f.rng_seed, 777);
                        if h.is_multiple_of(20) {
                            // Pick a random landing spot 10-20 voxels away
                            let dist = 10 + (tree_hash(f.rng_seed, 888) % 12) as usize;
                            let angle = tree_hash(f.rng_seed + tick.0, 999) % 360;
                            let rad = (angle as f64) * std::f64::consts::PI / 180.0;
                            let tx = (cx as f64 + rad.cos() * dist as f64) as usize;
                            let ty = (cy as f64 + rad.sin() * dist as f64) as usize;
                            // Find surface at target position (uses actual terrain height)
                            if tx < GRID_X && ty < GRID_Y {
                                let tz = VoxelGrid::surface_height(tx, ty) + 1;
                                if tz < GRID_Z {
                                    if let Some(cell) = grid.get_mut(tx, ty, tz) {
                                        if cell.material == Material::Air {
                                            cell.set_material(Material::Seed);
                                            cell.water_level = 0;
                                            cell.light_level = 0;
                                            cell.nutrient_level = 0;
                                            // Register species so the right plant grows
                                            // Bird carries seeds from the tree it was near
                                            if let Some(sid) = nearby_species {
                                                seed_map.map.insert((tx, ty, tz), sid as usize);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            FaunaType::Squirrel => {
                // Squirrel acorn caching: when Acting (digging), cache an acorn
                // that will sprout into an oak seedling. Creates "gift" plantings
                // in unexpected places — the squirrel forgot where it buried them!
                // Discovery: "An oak seedling appeared in the clearing — the squirrel must have buried an acorn there!"
                if f.state == FaunaState::Acting {
                    let cx = f.x as usize;
                    let cy = f.y as usize;

                    // ~30% chance per Acting tick to cache an acorn
                    let h = tree_hash(tick.0 + f.rng_seed, 1111);
                    if h.is_multiple_of(3) && cx < GRID_X && cy < GRID_Y {
                        let tz = VoxelGrid::surface_height(cx, cy) + 1;
                        if tz < GRID_Z {
                            if let Some(cell) = grid.get_mut(cx, cy, tz) {
                                if cell.material == Material::Air {
                                    cell.set_material(Material::Seed);
                                    // Register as oak seed
                                    seed_map.map.insert((cx, cy, tz), 0); // oak = species 0
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fauna_list_default() {
        let fl = FaunaList::default();
        assert_eq!(fl.fauna.len(), 0);
        assert_eq!(fl.count(), 0);
    }

    #[test]
    fn fauna_export_pack() {
        let mut fl = FaunaList::default();
        fl.fauna.push(Fauna {
            fauna_type: FaunaType::Bee,
            state: FaunaState::Idle,
            x: 10.5,
            y: 20.5,
            z: 42.0,
            target_x: 10.0,
            target_y: 20.0,
            target_z: 42.0,
            age: 0,
            max_age: 200,
            rng_seed: 1,
        });
        fl.pack_export();

        assert_eq!(fl.export_len(), FAUNA_EXPORT_BYTES);
        assert_eq!(fl.export_buf[0], FaunaType::Bee as u8);
        assert_eq!(fl.export_buf[1], FaunaState::Idle as u8);

        // Check packed x coordinate
        let x_bytes: [u8; 4] = fl.export_buf[4..8].try_into().unwrap();
        let x = f32::from_le_bytes(x_bytes);
        assert!((x - 10.5).abs() < 0.001);
    }

    /// Sync guard: if FaunaType/FaunaState enums or export layout changes,
    /// this test fails — reminding you to update bridge.ts and bridge.contract.test.ts.
    #[test]
    fn wasm_bridge_sync_guard() {
        // FaunaType repr(u8) values (must match bridge.ts FaunaType object)
        assert_eq!(FaunaType::Bee as u8, 0);
        assert_eq!(FaunaType::Butterfly as u8, 1);
        assert_eq!(FaunaType::Bird as u8, 2);
        assert_eq!(FaunaType::Worm as u8, 3);
        assert_eq!(FaunaType::Beetle as u8, 4);
        assert_eq!(FaunaType::Squirrel as u8, 5);

        // FaunaState repr(u8) values (must match bridge.ts FaunaState object)
        assert_eq!(FaunaState::Idle as u8, 0);
        assert_eq!(FaunaState::Seeking as u8, 1);
        assert_eq!(FaunaState::Acting as u8, 2);
        assert_eq!(FaunaState::Leaving as u8, 3);

        // Export record size (must match FAUNA_BYTES in bridge.ts)
        assert_eq!(
            FAUNA_EXPORT_BYTES, 16,
            "Fauna export size changed — update FAUNA_BYTES in bridge.ts"
        );

        // Export byte layout: [type: u8, state: u8, pad, pad, x: f32, y: f32, z: f32]
        let mut fl = FaunaList::default();
        fl.fauna.push(Fauna {
            fauna_type: FaunaType::Squirrel,
            state: FaunaState::Acting,
            x: 1.0,
            y: 2.0,
            z: 3.0,
            target_x: 0.0,
            target_y: 0.0,
            target_z: 0.0,
            age: 0,
            max_age: 100,
            rng_seed: 0,
        });
        fl.pack_export();
        assert_eq!(
            fl.export_buf[0],
            FaunaType::Squirrel as u8,
            "fauna_type at offset 0"
        );
        assert_eq!(
            fl.export_buf[1],
            FaunaState::Acting as u8,
            "state at offset 1"
        );
        // x at offset 4 (little-endian f32)
        let x = f32::from_le_bytes(fl.export_buf[4..8].try_into().unwrap());
        assert!((x - 1.0).abs() < 0.001, "x at offset 4");
    }
}

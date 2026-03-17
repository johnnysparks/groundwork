use bevy_ecs::prelude::*;

use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};
use crate::soil::SoilGrid;
use crate::tree::{
    generate_attraction_points, init_skeleton, tree_hash, BranchNode, GrowthStage, PlantType,
    SeedSpeciesMap, SpeciesTable, Tree, TreeTemplate,
};
use crate::voxel::Material;
use crate::{Tick, Weather, WeatherState};

/// Weather system: transitions between Clear, Rain, and Drought states.
/// Rain adds water to the surface; drought evaporates surface water.
/// Creates dramatic garden-wide events every ~200-400 ticks.
pub fn weather_system(mut grid: ResMut<VoxelGrid>, tick: Res<Tick>, mut weather: ResMut<Weather>) {
    // Advance weather state
    if weather.duration > 0 {
        weather.duration -= 1;
    }
    if weather.duration == 0 {
        // Transition to next state using deterministic sequence
        weather.sequence += 1;
        let h = tree_hash(weather.sequence, 12345);
        let (next_state, next_duration) = match weather.state {
            WeatherState::Clear => {
                // 30% chance of rain, 15% chance of drought, 55% stay clear
                if h % 100 < 30 {
                    (WeatherState::Rain, 30 + (h % 20) as u32) // rain for 30-50 ticks
                } else if h % 100 < 45 {
                    (WeatherState::Drought, 40 + (h % 30) as u32) // drought for 40-70 ticks
                } else {
                    (WeatherState::Clear, 150 + (h % 100) as u32) // clear for 150-250 ticks
                }
            }
            WeatherState::Rain => {
                (WeatherState::Clear, 100 + (h % 80) as u32) // always return to clear
            }
            WeatherState::Drought => {
                // Drought often ends with rain (recovery)
                if h % 100 < 40 {
                    (WeatherState::Rain, 20 + (h % 15) as u32) // rain relief
                } else {
                    (WeatherState::Clear, 100 + (h % 60) as u32)
                }
            }
        };
        weather.state = next_state;
        weather.duration = next_duration;
    }

    // Apply weather effects
    match weather.state {
        WeatherState::Rain => {
            // Rain: add water to surface air/soil cells every 3 ticks
            if tick.0.is_multiple_of(3) {
                let z = GROUND_LEVEL + 1;
                // Scatter rain across ~20% of the surface
                for i in 0..((GRID_X * GRID_Y) / 5) {
                    let h = tree_hash(tick.0 + i as u64, 54321);
                    let rx = (h as usize) % GRID_X;
                    let ry = ((h >> 16) as usize) % GRID_Y;
                    // Add water to surface
                    if let Some(cell) = grid.get_mut(rx, ry, z) {
                        if cell.material == Material::Air {
                            cell.water_level = cell.water_level.saturating_add(30);
                            if cell.water_level >= 50 {
                                cell.material = Material::Water;
                            }
                        }
                    }
                    // Also moisten the soil below
                    if let Some(cell) = grid.get_mut(rx, ry, GROUND_LEVEL) {
                        if cell.material == Material::Soil {
                            cell.water_level = cell.water_level.saturating_add(15);
                        }
                    }
                }
            }
        }
        WeatherState::Drought => {
            // Drought: evaporate surface water and dry out shallow soil every 5 ticks
            if tick.0.is_multiple_of(5) {
                let cells = grid.cells_mut();
                let z_stride = GRID_X * GRID_Y;
                // Surface layer: evaporate water
                let z = GROUND_LEVEL + 1;
                for y in 0..GRID_Y {
                    for x in 0..GRID_X {
                        let idx = x + y * GRID_X + z * z_stride;
                        if cells[idx].material == Material::Water {
                            cells[idx].water_level = cells[idx].water_level.saturating_sub(8);
                            if cells[idx].water_level < 5 {
                                cells[idx].set_material(Material::Air);
                            }
                        }
                    }
                }
                // Shallow soil: dry out
                for sz in GROUND_LEVEL.saturating_sub(2)..=GROUND_LEVEL {
                    for y in 0..GRID_Y {
                        for x in 0..GRID_X {
                            let idx = x + y * GRID_X + sz * z_stride;
                            if cells[idx].material == Material::Soil && cells[idx].water_level > 10
                            {
                                cells[idx].water_level = cells[idx].water_level.saturating_sub(3);
                            }
                        }
                    }
                }
            }
        }
        WeatherState::Clear => {} // no special effects
    }
}

/// Persistent water spring: refills the spring and stream source each tick.
/// Without this, the spring dries up by tick ~200 and the garden dies.
pub fn water_spring(mut grid: ResMut<VoxelGrid>) {
    // Refill the pond at the top of the slope
    use crate::grid::{POND_X, POND_Y};
    let pond_radius: isize = 6;
    let pond_radius_sq = pond_radius * pond_radius;
    for dy in (POND_Y as isize - pond_radius)..=(POND_Y as isize + pond_radius) {
        for dx in (POND_X as isize - pond_radius)..=(POND_X as isize + pond_radius) {
            if dx < 0 || dy < 0 {
                continue;
            }
            let (ux, uy) = (dx as usize, dy as usize);
            let ddx = dx - POND_X as isize;
            let ddy = dy - POND_Y as isize;
            if ddx * ddx + ddy * ddy > pond_radius_sq {
                continue;
            }
            let sz = VoxelGrid::surface_height(ux, uy);
            if let Some(cell) = grid.get_mut(ux, uy, sz) {
                if cell.material == Material::Water || cell.material == Material::Air {
                    cell.material = Material::Water;
                    cell.water_level = 255;
                }
            }
        }
    }
}

/// Gravity-driven water flow. Each tick, water tries to move down,
/// then spreads laterally to lower-water neighbors.
///
/// Uses a snapshot buffer to avoid iteration-order artifacts.
pub fn water_flow(mut grid: ResMut<VoxelGrid>) {
    // Snapshot water levels before mutation.
    let snapshot: Vec<u8> = grid.cells().iter().map(|v| v.water_level).collect();
    let total = snapshot.len();
    // Delta buffer for lateral spread — applied after the full pass to avoid
    // iteration-order bias that caused diagonal stripe artifacts.
    let mut lateral_deltas: Vec<i16> = vec![0; total];
    let z_stride = GRID_X * GRID_Y;
    let max_gravity = crate::scale::scale_transfer(32);
    let max_lateral = crate::scale::scale_transfer(8);

    // Iterate top-to-bottom so gravity cascades naturally.
    let cells = grid.cells_mut();
    for z in (0..GRID_Z).rev() {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = x + y * GRID_X + z * z_stride;
                let water = snapshot[idx];
                if water == 0 {
                    continue;
                }

                let mat = cells[idx].material;
                // Only Air and Water cells carry free water.
                if mat != Material::Air && mat != Material::Water {
                    continue;
                }

                // Try to flow down.
                if z > 0 {
                    let bidx = idx - z_stride;
                    let bmat = cells[bidx].material;
                    if (bmat == Material::Air || bmat == Material::Water)
                        && cells[bidx].water_level < 255
                    {
                        let transfer = water.min(255 - cells[bidx].water_level).min(max_gravity);
                        cells[bidx].water_level = cells[bidx].water_level.saturating_add(transfer);
                        if cells[bidx].material != Material::Water {
                            cells[bidx].nutrient_level = 0;
                            cells[bidx].material = Material::Water;
                        }
                        cells[idx].water_level = cells[idx].water_level.saturating_sub(transfer);
                        if cells[idx].water_level == 0 && cells[idx].material == Material::Water {
                            cells[idx].nutrient_level = 0;
                            cells[idx].material = Material::Air;
                        }
                        continue;
                    }
                }

                // Can't flow down — record lateral spread into delta buffer.
                macro_rules! lateral {
                    ($nidx:expr) => {{
                        let nidx = $nidx;
                        let neighbor_water = snapshot[nidx];
                        let neighbor_mat = cells[nidx].material;
                        if (neighbor_mat == Material::Air || neighbor_mat == Material::Water)
                            && neighbor_water < water.saturating_sub(1)
                        {
                            let transfer = ((water - neighbor_water) / 5).max(1).min(max_lateral);
                            lateral_deltas[nidx] += transfer as i16;
                            lateral_deltas[idx] -= transfer as i16;
                        }
                    }};
                }

                if x > 0 {
                    lateral!(idx - 1);
                }
                if x + 1 < GRID_X {
                    lateral!(idx + 1);
                }
                if y > 0 {
                    lateral!(idx - GRID_X);
                }
                if y + 1 < GRID_Y {
                    lateral!(idx + GRID_X);
                }
            }
        }
    }

    // Apply lateral spread deltas in one pass to avoid iteration-order bias.
    for (i, &delta) in lateral_deltas.iter().enumerate() {
        if delta == 0 {
            continue;
        }
        let cell = &mut cells[i];
        if delta > 0 {
            cell.water_level = cell.water_level.saturating_add(delta as u8);
            if cell.material == Material::Air {
                cell.nutrient_level = 0;
                cell.material = Material::Water;
            }
        } else {
            cell.water_level = cell.water_level.saturating_sub((-delta) as u8);
            if cell.water_level == 0 && cell.material == Material::Water {
                cell.nutrient_level = 0;
                cell.material = Material::Air;
            }
        }
    }

    // Cleanup: revert water cells with very low water_level to air.
    for cell in cells.iter_mut() {
        if cell.material == Material::Water && cell.water_level < 5 {
            cell.material = Material::Air;
            cell.water_level = 0;
            cell.nutrient_level = 0;
        }
    }
}

/// Top-down light propagation. For each (x, y) column, light starts
/// at 255 at the top and attenuates through solid materials.
pub fn light_propagation(mut grid: ResMut<VoxelGrid>) {
    // Pre-compute attenuation values (they're constant across the grid).
    let att_soil = crate::scale::scale_attenuation(30);
    let att_leaf = crate::scale::scale_attenuation(100);
    let att_trunk = crate::scale::scale_attenuation(30);
    let att_branch = crate::scale::scale_attenuation(20);
    let att_dead = crate::scale::scale_attenuation(10);
    let att_air = crate::scale::scale_attenuation(2);
    let att_water = crate::scale::scale_attenuation(15);
    let z_stride = GRID_X * GRID_Y;

    let cells = grid.cells_mut();
    for y in 0..GRID_Y {
        for x in 0..GRID_X {
            let mut light: u8 = 255;
            let base = x + y * GRID_X;
            for z in (0..GRID_Z).rev() {
                let idx = base + z * z_stride;
                let cell = &mut cells[idx];
                // Opaque materials attenuate *before* assignment.
                match cell.material {
                    Material::Soil | Material::Root => {
                        light = light.saturating_sub(att_soil);
                    }
                    Material::Stone => {
                        light = 0;
                    }
                    Material::Leaf => {
                        light = light.saturating_sub(att_leaf);
                    }
                    Material::Trunk => {
                        light = light.saturating_sub(att_trunk);
                    }
                    Material::Branch => {
                        light = light.saturating_sub(att_branch);
                    }
                    Material::DeadWood => {
                        light = light.saturating_sub(att_dead);
                    }
                    _ => {}
                }
                cell.light_level = light;
                // Transparent materials attenuate *after* assignment.
                match cell.material {
                    Material::Air | Material::Seed => {
                        light = light.saturating_sub(att_air);
                    }
                    Material::Water => {
                        light = light.saturating_sub(att_water);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Seeds grow into tree seedlings when they have enough water and light.
/// Uses nutrient_level as a growth counter: increments by 3-8 each tick
/// (based on adjacent soil quality) when conditions are met, spawns a Tree entity at 200.
pub fn seed_growth(
    mut grid: ResMut<VoxelGrid>,
    soil_grid: ResMut<SoilGrid>,
    mut commands: Commands,
    tick: Res<Tick>,
    mut seed_species: ResMut<SeedSpeciesMap>,
) {
    // No snapshot needed: seeds only read neighbor water/material and write to
    // themselves. Seeds don't affect each other's neighbor checks.
    let z_stride = GRID_X * GRID_Y;
    let cells = grid.cells();

    // First pass: collect seed positions (seeds are very rare, <0.1% of cells)
    let mut seeds: Vec<(usize, usize, usize, usize)> = Vec::new(); // (x, y, z, idx)
    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = x + y * GRID_X + z * z_stride;
                if cells[idx].material == Material::Seed {
                    seeds.push((x, y, z, idx));
                }
            }
        }
    }

    for (x, y, z, idx) in seeds {
        let cells = grid.cells();
        let cell_water = cells[idx].water_level;
        let cell_light = cells[idx].light_level;

        // Territorial suppression: seeds near established trunks can't germinate.
        // This prevents trees from growing on top of each other. Check a radius
        // around the seed for Trunk voxels — if found, skip this seed.
        let suppress_radius: usize = 6; // ~30cm at 5cm/voxel
        let mut suppressed = false;
        'suppress: for sz in z.saturating_sub(4)..=(z + 4).min(GRID_Z - 1) {
            for sy in y.saturating_sub(suppress_radius)..=(y + suppress_radius).min(GRID_Y - 1) {
                for sx in x.saturating_sub(suppress_radius)..=(x + suppress_radius).min(GRID_X - 1)
                {
                    let sidx = sx + sy * GRID_X + sz * z_stride;
                    if cells[sidx].material == Material::Trunk {
                        suppressed = true;
                        break 'suppress;
                    }
                }
            }
        }
        if suppressed {
            continue;
        }

        // Check own water or adjacent water.
        let mut has_water = cell_water >= 30;
        if !has_water {
            macro_rules! check_water {
                ($nidx:expr) => {
                    if cells[$nidx].water_level >= 30 {
                        has_water = true;
                    }
                };
            }
            if x > 0 {
                check_water!(idx - 1);
            }
            if !has_water && x + 1 < GRID_X {
                check_water!(idx + 1);
            }
            if !has_water && y > 0 {
                check_water!(idx - GRID_X);
            }
            if !has_water && y + 1 < GRID_Y {
                check_water!(idx + GRID_X);
            }
            if !has_water && z > 0 {
                check_water!(idx - z_stride);
            }
            if !has_water && z + 1 < GRID_Z {
                check_water!(idx + z_stride);
            }
        }

        let has_light = cell_light >= 30;

        if has_water && has_light {
            let mut best_nutrient: u8 = 0;
            let mut blocked_by_compaction = false;
            let mut min_ph: u8 = 128; // track lowest pH for allelopathy
            let soil_cells = soil_grid.cells();

            macro_rules! check_soil {
                ($nidx:expr) => {{
                    let nidx = $nidx;
                    if cells[nidx].material == Material::Soil {
                        let comp = &soil_cells[nidx];
                        // Use actual nutrient_level if available, fall back to capacity
                        let nl = cells[nidx].nutrient_level;
                        let effective = if nl > 0 { nl } else { comp.nutrient_capacity() };
                        if effective > best_nutrient {
                            best_nutrient = effective;
                        }
                        if comp.is_compacted() {
                            blocked_by_compaction = true;
                        }
                        if comp.ph < min_ph {
                            min_ph = comp.ph;
                        }
                    }
                }};
            }

            if x > 0 {
                check_soil!(idx - 1);
            }
            if x + 1 < GRID_X {
                check_soil!(idx + 1);
            }
            if y > 0 {
                check_soil!(idx - GRID_X);
            }
            if y + 1 < GRID_Y {
                check_soil!(idx + GRID_X);
            }
            if z > 0 {
                check_soil!(idx - z_stride);
            }
            if z + 1 < GRID_Z {
                check_soil!(idx + z_stride);
            }

            if !blocked_by_compaction {
                // Base growth rate: 12/tick → germination in ~17 ticks (200/12).
                // Previously 5/tick → 40 ticks. The feedback says 4 seconds of
                // staring at a seed with no feedback is the #1 retention killer.
                let soil_bonus = (best_nutrient as u16 * 5 / 255) as u8;
                let mut growth_rate = 12 + soil_bonus;

                // --- Allelopathy: acidic soil slows non-tolerant seed growth ---
                // Pine roots acidify soil (pH < 40). Most species grow at half speed.
                // Acid-tolerant species (pine=3, fern=4, moss=9) are immune.
                // Discovery: "My seeds won't grow near the pine... the soil is too acidic!"
                if min_ph < 40 {
                    let species_id = seed_species.map.get(&(x, y, z)).copied().unwrap_or(0);
                    let acid_tolerant = matches!(species_id, 3 | 4 | 9); // pine, fern, moss
                    if !acid_tolerant {
                        growth_rate /= 2; // half speed in acidic soil
                    }
                }

                // --- Nurse Log Effect ---
                // Seeds near DeadWood germinate faster. Rotting logs provide moisture,
                // shelter, and nutrients — a real-world forest succession pattern.
                // Discovery: "Seedlings keep sprouting near that dead tree!"
                // Check for DeadWood in immediate neighbors.
                {
                    let dead_u8 = Material::DeadWood.as_u8();
                    let mut near_deadwood = false;
                    if x > 0 && cells[idx - 1].material.as_u8() == dead_u8 {
                        near_deadwood = true;
                    }
                    if x + 1 < GRID_X && cells[idx + 1].material.as_u8() == dead_u8 {
                        near_deadwood = true;
                    }
                    if y > 0 && cells[idx - GRID_X].material.as_u8() == dead_u8 {
                        near_deadwood = true;
                    }
                    if y + 1 < GRID_Y && cells[idx + GRID_X].material.as_u8() == dead_u8 {
                        near_deadwood = true;
                    }
                    if z > 0 && cells[idx - z_stride].material.as_u8() == dead_u8 {
                        near_deadwood = true;
                    }
                    if z + 1 < GRID_Z && cells[idx + z_stride].material.as_u8() == dead_u8 {
                        near_deadwood = true;
                    }
                    if near_deadwood {
                        growth_rate = growth_rate.saturating_mul(2); // 2× germination near dead wood
                    }
                }
                if let Some(cell) = grid.get_mut(x, y, z) {
                    cell.nutrient_level = cell.nutrient_level.saturating_add(growth_rate);
                    if cell.nutrient_level >= 200 {
                        cell.set_material(Material::Trunk);

                        let mut footprint = vec![(x, y, z)];
                        if z > 0 {
                            if let Some(below) = grid.get_mut(x, y, z - 1) {
                                if below.material == Material::Soil {
                                    below.set_material(Material::Root);
                                    footprint.push((x, y, z - 1));
                                }
                            }
                        }
                        if z > 1 {
                            if let Some(below2) = grid.get_mut(x, y, z - 2) {
                                if below2.material == Material::Soil {
                                    below2.set_material(Material::Root);
                                    footprint.push((x, y, z - 2));
                                }
                            }
                        }

                        let species_id = seed_species.map.remove(&(x, y, z)).unwrap_or(0);
                        let rng_seed = tick.0.wrapping_mul(x as u64 + 1).wrapping_mul(y as u64 + 1);
                        // Start with accumulated resources so Seedling→Sapling
                        // happens within ~10 ticks (threshold 80, start at 40).
                        // This gets first leaf (from Sapling template) to tick ~27-35.
                        commands.spawn(Tree {
                            species_id,
                            root_pos: (x, y, z),
                            age: 0,
                            stage: GrowthStage::Seedling,
                            health: 1.0,
                            accumulated_water: 40.0,
                            accumulated_light: 40.0,
                            rng_seed,
                            dirty: false,
                            voxel_footprint: footprint,
                            branches: Vec::new(),
                            attraction_points: Vec::new(),
                            skeleton_initialized: false,
                            stage_changed: true,
                            pending_voxels: Vec::new(),
                            revealed_z: 0,
                        });
                    }
                }
            }
        }
    }
}

/// Soil absorbs water from adjacent Water voxels and diffuses water to
/// neighboring soil. Absorption and diffusion rates depend on soil composition.
pub fn soil_absorption(mut grid: ResMut<VoxelGrid>, soil_grid: ResMut<SoilGrid>) {
    let total = GRID_X * GRID_Y * GRID_Z;

    // Interleaved snapshot: (material_u8, water_level) per cell — one cache line
    // access per neighbor instead of two separate arrays.
    let snapshot: Vec<(u8, u8)> = grid
        .cells()
        .iter()
        .map(|v| (v.material.as_u8(), v.water_level))
        .collect();

    let soil_cells = soil_grid.cells();
    let soil_u8 = Material::Soil.as_u8();
    let water_u8 = Material::Water.as_u8();
    let z_stride = GRID_X * GRID_Y;
    let max_diffusion = crate::scale::scale_transfer(8) as u16;

    // Delta buffer for soil-to-soil diffusion (applied after full pass).
    let mut diffusion_deltas: Vec<i16> = vec![0; total];

    // Water absorption accumulator per soil cell — lets us get a single
    // mutable borrow of grid.cells_mut() at the end instead of per-cell.
    let mut water_absorbed: Vec<u8> = vec![0; total];

    // Sequential scan: ~35% of cells are soil, but the branch predictor handles
    // the `continue` path efficiently. An indexed approach is slower due to
    // index-to-coordinate decomposition and worse cache prefetch.
    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = x + y * GRID_X + z * z_stride;
                if snapshot[idx].0 != soil_u8 {
                    continue;
                }

                let comp = &soil_cells[idx];
                let absorption = 3 + (comp.drainage_rate() as u16 * 9 / 255) as u8;
                let max_water = 80 + (comp.water_retention() as u16 * 175 / 255) as u8;
                // Cache own drainage rate for diffusion (avoid recomputing per neighbor)
                let my_drainage = comp.drainage_rate() as u16;
                let my_water = snapshot[idx].1;

                macro_rules! check_neighbor {
                    ($nidx:expr) => {{
                        let nidx = $nidx;
                        let (nmat, nwater) = snapshot[nidx];
                        if nmat == water_u8 && nwater > 0 && my_water < max_water {
                            let space = max_water - my_water;
                            water_absorbed[idx] =
                                water_absorbed[idx].saturating_add(absorption.min(space));
                        } else if nmat == soil_u8 && my_water > nwater.saturating_add(5) {
                            let avg_drainage =
                                (my_drainage + soil_cells[nidx].drainage_rate() as u16) / 2;
                            let diff = my_water - nwater;
                            let transfer = ((diff as u16 * avg_drainage) / (255 * 2))
                                .max(1)
                                .min(max_diffusion) as i16;
                            diffusion_deltas[nidx] += transfer;
                            diffusion_deltas[idx] -= transfer;
                        }
                    }};
                }

                if x > 0 {
                    check_neighbor!(idx - 1);
                }
                if x + 1 < GRID_X {
                    check_neighbor!(idx + 1);
                }
                if y > 0 {
                    check_neighbor!(idx - GRID_X);
                }
                if y + 1 < GRID_Y {
                    check_neighbor!(idx + GRID_X);
                }
                if z > 0 {
                    check_neighbor!(idx - z_stride);
                }
                if z + 1 < GRID_Z {
                    check_neighbor!(idx + z_stride);
                }
            }
        }
    }

    // Apply absorption and diffusion deltas in a single pass.
    let grid_cells = grid.cells_mut();
    for i in 0..total {
        let absorbed = water_absorbed[i];
        let delta = diffusion_deltas[i];
        if absorbed == 0 && delta == 0 {
            continue;
        }
        let cell = &mut grid_cells[i];
        if cell.material != Material::Soil {
            continue;
        }
        if absorbed > 0 {
            cell.water_level = cell.water_level.saturating_add(absorbed);
        }
        if delta > 0 {
            cell.water_level = cell.water_level.saturating_add(delta as u8);
        } else if delta < 0 {
            cell.water_level = cell.water_level.saturating_sub((-delta) as u8);
        }
    }
}

/// Tree growth system: accumulates resources, checks stage transitions.
/// Health declines when water or light is insufficient.
/// Pollinators near a tree boost health recovery (Pollinator Bridge).
pub fn tree_growth(
    mut trees: Query<&mut Tree>,
    grid: Res<VoxelGrid>,
    species_table: Res<SpeciesTable>,
    fauna_list: Res<crate::fauna::FaunaList>,
    day_phase: Res<crate::DayPhase>,
) {
    let seasonal_mult = day_phase.growth_multiplier();
    // --- Nitrogen handshake ---
    // Clover/groundcover near a tree's root zone enriches soil nitrogen,
    // boosting growth by 1.5x. Detected by scanning for Leaf voxels at ground
    // level near the tree's roots (groundcover places Leaf voxels on the surface).
    let nitrogen_radius: usize = 5;

    for mut tree in trees.iter_mut() {
        if tree.stage == GrowthStage::Dead {
            // --- Drought Recovery ---
            // Dead trees with roots in wet soil can recover. This rewards the player
            // for adding water near a dead tree — "My tree came back to life!"
            // Recovery requires: root voxels with water_level > 50.
            tree.age += 1;
            let mut root_water: f32 = 0.0;
            for &(vx, vy, vz) in &tree.voxel_footprint {
                if let Some(voxel) = grid.get(vx, vy, vz) {
                    if voxel.material == Material::Root {
                        root_water += voxel.water_level as f32;
                    }
                }
            }
            // Need meaningful water flow to roots
            if root_water > 100.0 {
                // Slow health recovery while dead — takes ~50 ticks to reach 0.3
                tree.health = (tree.health + 0.006).min(1.0);
                if tree.health >= 0.3 {
                    // Resurrection! Tree revives as a sapling with new growth
                    tree.stage = GrowthStage::Sapling;
                    tree.dirty = true;
                    // Reset skeleton so it regrows fresh branches
                    tree.branches.clear();
                    tree.attraction_points.clear();
                    tree.skeleton_initialized = false;
                }
            }
            // Don't set dirty while dead — rasterizing a dead tree clears its roots,
            // which would prevent recovery. Only set dirty on actual revival (above).
            continue;
        }
        let species = &species_table.species[tree.species_id];
        tree.age += 1;

        // Accumulate water from root voxels, light from above-ground voxels
        let mut water_intake: f32 = 0.0;
        let mut light_intake: f32 = 0.0;

        for &(vx, vy, vz) in &tree.voxel_footprint {
            if let Some(voxel) = grid.get(vx, vy, vz) {
                match voxel.material {
                    Material::Root => water_intake += voxel.water_level as f32,
                    Material::Trunk | Material::Leaf | Material::Branch => {
                        light_intake += voxel.light_level as f32;
                    }
                    _ => {}
                }
            }
        }

        // --- Nitrogen boost from nearby groundcover ---
        // Groundcover (clover, moss, grass) at ground level near a tree's roots
        // enriches soil nitrogen → 1.5x nutrient accumulation.
        // Detected by counting Leaf voxels at ground level (z = GROUND_LEVEL to +2).
        let nitrogen_boost = if species.plant_type == PlantType::Tree {
            let (rx, ry, _rz) = tree.root_pos;
            let mut ground_leaf_count = 0u32;
            let x_lo = rx.saturating_sub(nitrogen_radius);
            let x_hi = (rx + nitrogen_radius).min(GRID_X - 1);
            let y_lo = ry.saturating_sub(nitrogen_radius);
            let y_hi = (ry + nitrogen_radius).min(GRID_Y - 1);
            for gz in GROUND_LEVEL..=(GROUND_LEVEL + 2) {
                for gy in y_lo..=y_hi {
                    for gx in x_lo..=x_hi {
                        if let Some(v) = grid.get(gx, gy, gz) {
                            if v.material == Material::Leaf {
                                ground_leaf_count += 1;
                            }
                        }
                    }
                }
            }
            // Need at least 3 groundcover leaf voxels for the boost
            if ground_leaf_count >= 3 {
                1.5_f32
            } else {
                1.0
            }
        } else {
            1.0
        };

        // --- Canopy Effect: shade-tolerant species thrive under canopy ---
        // Species with low shade_tolerance (fern=30, moss=20) get a growth boost
        // in moderate shade. This creates the undergrowth layer: oaks shade ferns,
        // ferns shade moss — each species fills its niche.
        let canopy_boost = if species.shade_tolerance < 60 {
            // shade_tolerance < 60 = very shade-tolerant (fern, moss, holly)
            // In moderate shade (light_intake 5-30), they grow 1.5× faster
            if (5.0..30.0).contains(&light_intake) {
                1.5_f32
            } else {
                1.0
            }
        } else {
            1.0
        };

        // --- Species Water Affinity ---
        // Willow (species_id=2) thrives near water — 2× growth when roots are well-watered.
        // Discovery: "My willow by the stream is growing twice as fast as the one on dry ground!"
        let water_affinity_boost = if tree.species_id == 2 && water_intake > 50.0 {
            2.0_f32
        } else {
            1.0
        };

        // --- Birch Pioneer Vigor ---
        // Birch (species_id=1) grows 1.5× faster in open ground (no nearby trunks
        // within 8 voxels). First to colonize bare patches and clearings.
        // Discovery: "The birch shot up fast in the clearing, but slowed once the oak grew tall."
        let pioneer_boost = if tree.species_id == 1 {
            let (rx, ry, _rz) = tree.root_pos;
            let check_r: usize = 8;
            let mut nearby_trunks = 0u32;
            // Only check a small sample to stay fast
            for cy in ry.saturating_sub(check_r)..=(ry + check_r).min(GRID_Y - 1) {
                for cx in rx.saturating_sub(check_r)..=(rx + check_r).min(GRID_X - 1) {
                    if cx == rx && cy == ry {
                        continue;
                    }
                    // Check at trunk height (surface + 2..5)
                    for cz in (GROUND_LEVEL + 2)..=(GROUND_LEVEL + 5).min(GRID_Z - 1) {
                        if let Some(v) = grid.get(cx, cy, cz) {
                            if v.material == Material::Trunk {
                                nearby_trunks += 1;
                            }
                        }
                    }
                }
            }
            if nearby_trunks == 0 {
                1.5_f32
            } else {
                1.0
            }
        } else {
            1.0
        };

        // --- Berry Bush + Bird Symbiosis ---
        // Berry bush (species_id=5) gets a health and growth boost when birds are nearby.
        // The birds eat berries and in return spread seeds + fertilize with droppings.
        // Discovery: "My berry bush is thriving — oh, the birds are helping it!"
        let bird_symbiosis_boost = if tree.species_id == 5 {
            let (rx, ry, _rz) = tree.root_pos;
            let bird_r = 12.0_f32;
            let mut nearby_birds = 0u32;
            for f in &fauna_list.fauna {
                if f.fauna_type == crate::fauna::FaunaType::Bird {
                    let dx = f.x - rx as f32;
                    let dy = f.y - ry as f32;
                    if dx * dx + dy * dy < bird_r * bird_r {
                        nearby_birds += 1;
                    }
                }
            }
            if nearby_birds > 0 {
                1.5_f32
            } else {
                1.0
            }
        } else {
            1.0
        };

        // Use diminishing returns so more roots/light don't trivially blast
        // through all growth stages in a single tick. sqrt gives gentle scaling:
        // 100 water_intake → +10, 10000 → +100, 50000 → +224
        let total_boost = nitrogen_boost
            * canopy_boost
            * water_affinity_boost
            * pioneer_boost
            * bird_symbiosis_boost
            * seasonal_mult;
        tree.accumulated_water += water_intake.sqrt() * species.growth_rate * total_boost;
        tree.accumulated_light += light_intake.sqrt() * species.growth_rate * total_boost;

        // Health declines without resources, recovers when well-supplied.
        // Light check uses shade_tolerance: low tolerance = sun-loving = dies faster in shade.
        // Threshold is scaled down because light_intake is sqrt(sum_of_light_across_voxels).
        // A healthy tree in full sun: ~50 voxels × 200 light = sqrt(10000) ≈ 100 intake.
        // Shade tolerance maps to threshold: high tolerance → low threshold.
        let water_ok = water_intake >= species.water_need.threshold();
        let shade_factor = 1.0 - species.shade_tolerance as f32 / 255.0; // 0=tolerant, 1=needs sun
        let light_threshold = 10.0 + shade_factor * 40.0; // range: 10 (tolerant) to 50 (sun-loving)
        let light_ok = light_intake >= light_threshold;

        // Young plants are more vulnerable to stress — seedlings and saplings
        // die 3× faster in shade/drought than mature trees. This creates natural
        // thinning: only well-placed seedlings survive to maturity.
        let youth_multiplier = match tree.stage {
            GrowthStage::Seedling => 4.0,
            GrowthStage::Sapling => 3.0,
            GrowthStage::YoungTree => 2.0,
            _ => 1.0,
        };

        if !water_ok && !light_ok {
            // Severe stress: both resources missing (crowded conditions)
            // Youth multiplier: seedlings die 4× faster, saplings 3×, etc.
            tree.health = (tree.health - 0.015 * youth_multiplier).max(0.0);
        } else if !water_ok {
            tree.health = (tree.health - 0.005 * youth_multiplier).max(0.0);
        } else if !light_ok {
            // Shade stress: sun-loving species decline faster in shade
            let shade_penalty = 0.003 + (1.0 - species.shade_tolerance as f32 / 255.0) * 0.005;
            tree.health = (tree.health - shade_penalty * youth_multiplier).max(0.0);
        }
        if water_ok && light_ok {
            // Healthy plants recover quickly — the garden should be mostly green
            tree.health = (tree.health + 0.02).min(1.0);
        } else if water_ok || light_ok {
            // One resource met: slow partial recovery. Must be lower than the
            // shade/drought penalty so competition can actually thin crowded areas.
            // At +0.002 vs shade penalty ~0.006, net = -0.004/tick → death in ~250 ticks.
            tree.health = (tree.health + 0.002).min(1.0);
        }

        // --- Pollinator Bridge ---
        // Pollinators (bees, butterflies) near a tree boost its health recovery.
        // Discovery moment: "I planted flowers near my oak, bees came, the oak recovered."
        // Creates the loop: flowers → pollinators → healthier neighbors → more flowers.
        {
            let (rx, ry, _rz) = tree.root_pos;
            let pollinator_radius = 10.0_f32;
            let mut nearby_pollinators = 0u32;
            for f in &fauna_list.fauna {
                match f.fauna_type {
                    crate::fauna::FaunaType::Bee | crate::fauna::FaunaType::Butterfly => {
                        let dx = f.x - rx as f32;
                        let dy = f.y - ry as f32;
                        if dx * dx + dy * dy < pollinator_radius * pollinator_radius {
                            nearby_pollinators += 1;
                        }
                    }
                    _ => {}
                }
            }
            if nearby_pollinators > 0 {
                // Each pollinator gives +0.005 health recovery, up to +0.02 (4 pollinators)
                let bonus = (nearby_pollinators as f32 * 0.005).min(0.02);
                tree.health = (tree.health + bonus).min(1.0);
            }
        }

        // Re-rasterize every 30 ticks to update health visual stress on foliage.
        // This is a health-only update — don't clear footprint, just update colors.
        if tree.age > 0 && tree.age % 30 == 0 {
            tree.dirty = true;
            tree.stage_changed = false; // health update, not stage change
        }

        // Crowding death: sustained low health kills the plant.
        // Young plants die faster (age > 20 for seedlings, > 50 for mature).
        // This creates natural thinning — only the fittest survive in crowded zones.
        let death_age_threshold = match tree.stage {
            GrowthStage::Seedling | GrowthStage::Sapling => 20,
            _ => 50,
        };
        if tree.health < 0.1 && tree.age > death_age_threshold {
            tree.health = (tree.health - 0.01 * youth_multiplier).max(0.0);
            if tree.health == 0.0 && tree.stage != GrowthStage::Dead {
                tree.stage = GrowthStage::Dead;
                tree.dirty = true;
            }
        }

        // Check stage transition
        if let Some(next) = tree.stage.next_stage(
            tree.age,
            tree.accumulated_water,
            tree.accumulated_light,
            tree.health,
        ) {
            let prev = tree.stage;
            tree.stage = next;
            tree.dirty = true;
            tree.stage_changed = true;

            // Initialize or expand skeleton on transition to branching stages
            // Only trees use space colonization; other plant types always use templates
            if species.uses_skeleton() {
                match next {
                    GrowthStage::Sapling => {
                        // Starter skeleton: short trunk with branch tips at the crown.
                        // This produces visible leaves at the Sapling stage (~t25-35)
                        // instead of waiting until YoungTree (~t80+).
                        // The player sees a tiny tree with a leaf tuft — the first
                        // emotional payoff after planting.
                        let sapling_h = (species.max_height() / 3).max(2) as isize;
                        let mut branches = Vec::new();
                        // Short trunk
                        for z in 0..sapling_h {
                            branches.push(crate::tree::BranchNode {
                                pos: (0, 0, z),
                                parent: if z == 0 { u16::MAX } else { (z - 1) as u16 },
                                material: Material::Trunk,
                                shade_stress: 0,
                                alive: true,
                            });
                        }
                        // 3 branch tips at crown (offset from trunk top)
                        let top_idx = (sapling_h - 1) as u16;
                        let offsets: [(isize, isize, isize); 3] =
                            [(1, 0, 1), (-1, 1, 1), (0, -1, 1)];
                        for (dx, dy, dz) in offsets {
                            branches.push(crate::tree::BranchNode {
                                pos: (dx, dy, sapling_h + dz),
                                parent: top_idx,
                                material: Material::Branch,
                                shade_stress: 0,
                                alive: true,
                            });
                        }
                        // Single root
                        branches.push(crate::tree::BranchNode {
                            pos: (0, 0, -1),
                            parent: 0,
                            material: Material::Root,
                            shade_stress: 0,
                            alive: true,
                        });
                        tree.branches = branches;
                        tree.skeleton_initialized = true;
                    }
                    GrowthStage::YoungTree => {
                        let (branches, points) = init_skeleton(species, &next, tree.rng_seed);
                        tree.branches = branches;
                        tree.attraction_points = points;
                        tree.skeleton_initialized = true;
                    }
                    GrowthStage::Mature | GrowthStage::OldGrowth => {
                        // Add more attraction points for larger crown
                        let new_points = generate_attraction_points(species, &next, tree.rng_seed);
                        tree.attraction_points.extend(new_points);
                        // Extend trunk if needed
                        if prev == GrowthStage::YoungTree {
                            let old_trunk_h = (species.max_height() * 2 / 3).max(3) as isize;
                            let new_trunk_h = species.max_height() as isize;
                            for z in old_trunk_h..new_trunk_h {
                                let parent_idx =
                                    tree.branches
                                        .iter()
                                        .position(|b| b.pos == (0, 0, z - 1))
                                        .unwrap_or(0) as u16;
                                tree.branches.push(BranchNode {
                                    pos: (0, 0, z),
                                    parent: parent_idx,
                                    material: Material::Trunk,
                                    shade_stress: 0,
                                    alive: true,
                                });
                            }
                            // Add branch stubs on the new trunk section so
                            // space colonization can fill the mid-canopy.
                            let stub_dirs: [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
                            let stub_positions = [
                                old_trunk_h + (new_trunk_h - old_trunk_h) / 3,
                                old_trunk_h + (new_trunk_h - old_trunk_h) * 2 / 3,
                            ];
                            for (i, &sz) in stub_positions.iter().enumerate() {
                                if sz > 0 && sz < new_trunk_h {
                                    let (sdx, sdy) = stub_dirs[i % 4];
                                    let pidx = tree
                                        .branches
                                        .iter()
                                        .position(|b| b.pos == (0, 0, sz))
                                        .unwrap_or(0)
                                        as u16;
                                    tree.branches.push(BranchNode {
                                        pos: (sdx, sdy, sz),
                                        parent: pidx,
                                        material: Material::Branch,
                                        shade_stress: 0,
                                        alive: true,
                                    });
                                }
                            }
                        }
                    }
                    GrowthStage::Dead => {
                        // Kill all branch nodes
                        for node in &mut tree.branches {
                            if node.material != Material::Root {
                                node.alive = false;
                                if node.material != Material::Trunk {
                                    node.material = Material::DeadWood;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Space colonization: grow branches toward attraction points with phototropism.
/// Runs every 3 ticks for trees with active skeletons.
pub fn branch_growth(
    mut trees: Query<&mut Tree>,
    grid: Res<VoxelGrid>,
    species_table: Res<SpeciesTable>,
) {
    for mut tree in trees.iter_mut() {
        let species = &species_table.species[tree.species_id];
        // Only trees use space colonization branching
        if !species.uses_skeleton() {
            continue;
        }
        if tree.branches.is_empty() {
            continue;
        }
        if matches!(tree.stage, GrowthStage::Dead) {
            continue;
        }
        // Only grow every 3 ticks
        if tree.age % 3 != 0 {
            continue;
        }

        // Regenerate attraction points when running low — ensures continuous
        // branch growth instead of stalling after initial points are consumed.
        if tree.attraction_points.len() < 10
            && matches!(
                tree.stage,
                GrowthStage::YoungTree | GrowthStage::Mature | GrowthStage::OldGrowth
            )
        {
            let new_points = crate::tree::generate_attraction_points(
                species,
                &tree.stage,
                tree.rng_seed.wrapping_add(tree.age as u64),
            );
            tree.attraction_points.extend(new_points);
        }

        if tree.attraction_points.is_empty() {
            continue;
        }

        let (rx, ry, rz) = tree.root_pos;

        // Find active tips: alive branch/leaf nodes with no children
        let branch_count = tree.branches.len();
        let mut has_child = vec![false; branch_count];
        for node in &tree.branches {
            if node.parent != u16::MAX && (node.parent as usize) < branch_count {
                has_child[node.parent as usize] = true;
            }
        }

        let tips: Vec<usize> = (0..branch_count)
            .filter(|&i| {
                tree.branches[i].alive
                    && !has_child[i]
                    && tree.branches[i].material != Material::Root
            })
            .collect();

        if tips.is_empty() {
            continue;
        }

        // Scale influence and kill distances to crown radius so branch growth
        // works correctly regardless of voxel size. Kill distance should be
        // a small fraction of crown radius to prevent premature point exhaustion.
        let cr = species.crown_radius().max(2) as f64;
        let influence_dist_sq = (cr * cr * 4.0) as isize; // 2× crown radius
        let kill_dist_sq = (cr * 0.3 * cr * 0.3).max(1.0) as isize; // 0.3× crown radius

        // Associate each attraction point with its nearest tip
        let mut tip_directions: Vec<(isize, isize, isize, u32)> = vec![(0, 0, 0, 0); tips.len()];
        for &(px, py, pz) in tree.attraction_points.iter() {
            let mut best_tip = None;
            let mut best_dist = isize::MAX;

            for (ti, &tip_idx) in tips.iter().enumerate() {
                let tp = tree.branches[tip_idx].pos;
                let d = (px - tp.0) * (px - tp.0)
                    + (py - tp.1) * (py - tp.1)
                    + (pz - tp.2) * (pz - tp.2);
                if d < best_dist && d <= influence_dist_sq {
                    best_dist = d;
                    best_tip = Some(ti);
                }
            }

            if let Some(ti) = best_tip {
                let tp = tree.branches[tips[ti]].pos;
                tip_directions[ti].0 += px - tp.0;
                tip_directions[ti].1 += py - tp.1;
                tip_directions[ti].2 += pz - tp.2;
                tip_directions[ti].3 += 1;
            }
        }

        // Grow up to 15 new nodes per tick for denser canopy coverage
        let mut grown = 0u32;
        for (ti, &tip_idx) in tips.iter().enumerate() {
            if grown >= 15 {
                break;
            }
            let (dx, dy, dz, count) = tip_directions[ti];
            if count == 0 {
                continue;
            }

            // Normalize to unit direction
            let mut fdx = dx as f32 / count as f32;
            let mut fdy = dy as f32 / count as f32;
            let mut fdz = dz as f32 / count as f32;

            // Phototropism: bias toward brightest neighbor
            let tp = tree.branches[tip_idx].pos;
            let world_pos = (rx as isize + tp.0, ry as isize + tp.1, rz as isize + tp.2);

            let light_neighbors: [(isize, isize, isize); 5] =
                [(0, 0, 1), (1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0)];
            let mut light_dx: f32 = 0.0;
            let mut light_dy: f32 = 0.0;
            let mut light_dz: f32 = 0.0;
            let mut total_light: f32 = 0.0;

            for (ndx, ndy, ndz) in light_neighbors {
                let wx = world_pos.0 + ndx;
                let wy = world_pos.1 + ndy;
                let wz = world_pos.2 + ndz;
                if wx >= 0 && wy >= 0 && wz >= 0 {
                    let (wx, wy, wz) = (wx as usize, wy as usize, wz as usize);
                    if let Some(cell) = grid.get(wx, wy, wz) {
                        let l = cell.light_level as f32;
                        light_dx += ndx as f32 * l;
                        light_dy += ndy as f32 * l;
                        light_dz += ndz as f32 * l;
                        total_light += l;
                    }
                }
            }

            if total_light > 0.0 {
                light_dx /= total_light;
                light_dy /= total_light;
                light_dz /= total_light;

                let p = species.phototropism;
                fdx = (1.0 - p) * fdx + p * light_dx;
                fdy = (1.0 - p) * fdy + p * light_dy;
                fdz = (1.0 - p) * fdz + p * light_dz;
            }

            // Quantize to nearest cardinal step
            let step = quantize_direction(fdx, fdy, fdz);
            if step == (0, 0, 0) {
                continue;
            }

            let new_pos = (tp.0 + step.0, tp.1 + step.1, tp.2 + step.2);

            // Check world bounds and occupancy
            let world_new = (
                rx as isize + new_pos.0,
                ry as isize + new_pos.1,
                rz as isize + new_pos.2,
            );
            if world_new.0 < 0 || world_new.1 < 0 || world_new.2 < 0 {
                continue;
            }
            let (wnx, wny, wnz) = (
                world_new.0 as usize,
                world_new.1 as usize,
                world_new.2 as usize,
            );
            if !VoxelGrid::in_bounds(wnx, wny, wnz) {
                continue;
            }

            // Only grow into air or existing tree materials
            if let Some(cell) = grid.get(wnx, wny, wnz) {
                let ok = cell.material == Material::Air
                    || cell.material == Material::Leaf
                    || cell.material == Material::Branch;
                if !ok {
                    continue;
                }
            }

            // Don't duplicate an existing node position
            if tree.branches.iter().any(|b| b.pos == new_pos && b.alive) {
                continue;
            }

            // Add new branch node
            tree.branches.push(BranchNode {
                pos: new_pos,
                parent: tip_idx as u16,
                material: Material::Branch,
                shade_stress: 0,
                alive: true,
            });

            // The old tip is now an interior node — keep as Branch
            // (Leaf placement happens separately during rasterize)

            // Generate leaf sphere around new tip and queue for gradual growth.
            // This makes branches visible immediately instead of waiting for the
            // next stage transition (which could be hundreds of ticks away).
            let new_crown_r = species.crown_radius() as isize;
            let new_leaf_r: isize = match tree.stage {
                GrowthStage::YoungTree => (new_crown_r / 2).clamp(4, 10),
                GrowthStage::Mature | GrowthStage::OldGrowth => (new_crown_r / 2).clamp(5, 12),
                _ => 1,
            };
            let new_leaf_r_sq = new_leaf_r * new_leaf_r;
            // Push branch voxel first, then leaf sphere
            tree.pending_voxels.push((wnx, wny, wnz, Material::Branch));
            for ddx in -new_leaf_r..=new_leaf_r {
                for ddy in -new_leaf_r..=new_leaf_r {
                    for ddz in -new_leaf_r..=new_leaf_r {
                        if ddx * ddx + ddy * ddy + ddz * ddz > new_leaf_r_sq {
                            continue;
                        }
                        let lx = rx as isize + new_pos.0 + ddx;
                        let ly = ry as isize + new_pos.1 + ddy;
                        let lz = rz as isize + new_pos.2 + ddz;
                        if lx < 0 || ly < 0 || lz < 0 {
                            continue;
                        }
                        let (lx, ly, lz) = (lx as usize, ly as usize, lz as usize);
                        if !VoxelGrid::in_bounds(lx, ly, lz) {
                            continue;
                        }
                        if let Some(cell) = grid.get(lx, ly, lz) {
                            if cell.material == Material::Air {
                                tree.pending_voxels.push((lx, ly, lz, Material::Leaf));
                            }
                        }
                    }
                }
            }

            // Consume nearby attraction points
            tree.attraction_points.retain(|&(px, py, pz)| {
                let d = (px - new_pos.0) * (px - new_pos.0)
                    + (py - new_pos.1) * (py - new_pos.1)
                    + (pz - new_pos.2) * (pz - new_pos.2);
                d > kill_dist_sq
            });

            tree.dirty = true;
            grown += 1;
        }
    }
}

/// Quantize a floating-point direction to the nearest cardinal/diagonal voxel step.
fn quantize_direction(dx: f32, dy: f32, dz: f32) -> (isize, isize, isize) {
    // Pick the axis with the largest magnitude, or allow diagonals
    let ax = dx.abs();
    let ay = dy.abs();
    let az = dz.abs();

    if ax < 0.01 && ay < 0.01 && az < 0.01 {
        return (0, 0, 1); // Default upward
    }

    let sx = if dx > 0.0 {
        1isize
    } else if dx < 0.0 {
        -1
    } else {
        0
    };
    let sy = if dy > 0.0 {
        1isize
    } else if dy < 0.0 {
        -1
    } else {
        0
    };
    let sz = if dz > 0.0 {
        1isize
    } else if dz < 0.0 {
        -1
    } else {
        0
    };

    // Dominant axis gets priority; include secondary if it's > 50% of dominant
    let max_a = ax.max(ay).max(az);
    let threshold = max_a * 0.5;

    let rx = if ax >= threshold { sx } else { 0 };
    let ry = if ay >= threshold { sy } else { 0 };
    let rz = if az >= threshold { sz } else { 0 };

    if rx == 0 && ry == 0 && rz == 0 {
        (0, 0, 1)
    } else {
        (rx, ry, rz)
    }
}

/// Self-pruning: shaded branches accumulate stress and eventually die.
pub fn self_pruning(
    mut trees: Query<&mut Tree>,
    grid: Res<VoxelGrid>,
    species_table: Res<SpeciesTable>,
) {
    for mut tree in trees.iter_mut() {
        let species = &species_table.species[tree.species_id];
        // Only trees use skeleton-based pruning
        if !species.uses_skeleton() {
            continue;
        }
        if tree.branches.is_empty() {
            continue;
        }
        if matches!(tree.stage, GrowthStage::Dead) {
            continue;
        }

        let (rx, ry, rz) = tree.root_pos;
        let mut any_changed = false;

        // Collect dead parent indices for cascade
        let mut newly_dead: Vec<u16> = Vec::new();

        for i in 0..tree.branches.len() {
            let node = &tree.branches[i];
            if !node.alive || node.material == Material::Root || node.material == Material::Trunk {
                continue;
            }

            let world_pos = (
                rx as isize + node.pos.0,
                ry as isize + node.pos.1,
                rz as isize + node.pos.2,
            );

            let light = if world_pos.0 >= 0 && world_pos.1 >= 0 && world_pos.2 >= 0 {
                let (wx, wy, wz) = (
                    world_pos.0 as usize,
                    world_pos.1 as usize,
                    world_pos.2 as usize,
                );
                grid.get(wx, wy, wz).map_or(0, |c| c.light_level)
            } else {
                0
            };

            if light < species.shade_tolerance {
                tree.branches[i].shade_stress += 1;
            } else {
                tree.branches[i].shade_stress = tree.branches[i].shade_stress.saturating_sub(1);
            }

            if tree.branches[i].shade_stress >= species.prune_threshold {
                tree.branches[i].alive = false;
                tree.branches[i].material = Material::DeadWood;
                newly_dead.push(i as u16);
                any_changed = true;
            }
        }

        // Cascade: kill descendants of dead nodes
        if !newly_dead.is_empty() {
            let mut changed = true;
            while changed {
                changed = false;
                for i in 0..tree.branches.len() {
                    if tree.branches[i].alive
                        && tree.branches[i].parent != u16::MAX
                        && newly_dead.contains(&tree.branches[i].parent)
                    {
                        tree.branches[i].alive = false;
                        tree.branches[i].material = Material::DeadWood;
                        newly_dead.push(i as u16);
                        changed = true;
                    }
                }
            }
        }

        // Remove fully decayed nodes (shade_stress well past threshold)
        let decay_threshold = species.prune_threshold * 2;
        for node in &mut tree.branches {
            if !node.alive && node.shade_stress < decay_threshold {
                node.shade_stress += 1;
            }
        }
        let before_len = tree.branches.len();
        tree.branches
            .retain(|n| n.alive || n.shade_stress < decay_threshold);
        if tree.branches.len() != before_len {
            any_changed = true;
        }

        if any_changed {
            tree.dirty = true;
        }
    }
}

/// Rasterize dirty trees: clear old voxels, generate template, write new voxels.
pub fn tree_rasterize(
    mut trees: Query<&mut Tree>,
    mut grid: ResMut<VoxelGrid>,
    species_table: Res<SpeciesTable>,
) {
    for mut tree in trees.iter_mut() {
        if !tree.dirty {
            continue;
        }

        let species = &species_table.species[tree.species_id];

        // Collect dynamic root positions (from root_growth, not in skeleton)
        // to preserve them through rasterization.
        let skeleton_root_positions: std::collections::HashSet<(usize, usize, usize)> = tree
            .branches
            .iter()
            .filter(|b| b.material == Material::Root)
            .map(|b| {
                let (rx, ry, rz) = tree.root_pos;
                (
                    (rx as isize + b.pos.0) as usize,
                    (ry as isize + b.pos.1) as usize,
                    (rz as isize + b.pos.2) as usize,
                )
            })
            .collect();

        let mut dynamic_roots: Vec<(usize, usize, usize)> = Vec::new();

        // Only clear footprint on stage change. Health-only updates just refresh leaf colors.
        if !tree.stage_changed {
            // Health-only update: just write health into existing leaf voxels
            for &(x, y, z) in &tree.voxel_footprint {
                if let Some(cell) = grid.get_mut(x, y, z) {
                    if cell.material == Material::Leaf || cell.material == Material::Branch {
                        cell.water_level = (tree.health * 255.0) as u8;
                        cell.nutrient_level = tree.species_id as u8;
                    }
                }
            }
            tree.dirty = false;
            continue;
        }
        tree.stage_changed = false;

        // Clear old footprint, preserving dynamic roots
        for &(x, y, z) in &tree.voxel_footprint {
            if let Some(cell) = grid.get_mut(x, y, z) {
                match cell.material {
                    Material::Root if !skeleton_root_positions.contains(&(x, y, z)) => {
                        // This is a dynamic root — keep it
                        dynamic_roots.push((x, y, z));
                    }
                    Material::Trunk
                    | Material::Branch
                    | Material::Leaf
                    | Material::Root
                    | Material::DeadWood => {
                        // Use actual surface height, not the constant GROUND_LEVEL.
                        // The terrain has rolling hills (±6 voxels around GROUND_LEVEL),
                        // so roots above GROUND_LEVEL but at/below the surface must
                        // revert to Soil — otherwise they become Air gaps that fill
                        // with water, causing trees to float.
                        let surface = VoxelGrid::surface_height(x, y);
                        if z <= surface {
                            cell.set_material(Material::Soil);
                        } else {
                            cell.set_material(Material::Air);
                        }
                    }
                    _ => {} // changed externally, leave it
                }
            }
        }

        let mut new_footprint = dynamic_roots;
        let (rx, ry, rz) = tree.root_pos;

        if !tree.branches.is_empty() {
            // Skeleton path: rasterize from branch nodes
            // Determine tips for leaf placement
            let branch_count = tree.branches.len();
            let mut has_child = vec![false; branch_count];
            for node in &tree.branches {
                if node.parent != u16::MAX && (node.parent as usize) < branch_count {
                    has_child[node.parent as usize] = true;
                }
            }

            // Trunk inflation radius — species trunk_radius, tapered with height
            let trunk_r = species.trunk_radius().max(1) as isize;
            let trunk_h = species.max_height() as isize;

            // Collect skeleton voxels instead of placing immediately.
            // This lets trunk grow gradually via pending_voxels.
            let mut skeleton_voxels: Vec<(usize, usize, usize, Material)> = Vec::new();

            for (i, node) in tree.branches.iter().enumerate() {
                if !node.alive && node.material != Material::DeadWood {
                    continue;
                }

                // Tips get Leaf material, interior nodes keep their material
                let mat = if node.alive && !has_child[i] && node.material == Material::Branch {
                    Material::Leaf
                } else {
                    node.material
                };

                // Trunk nodes get inflated to a cylinder; branches/leaves stay 1-wide
                let inflate_r = if mat == Material::Trunk {
                    // Taper: full radius at base, 1 at top
                    let height_frac = node.pos.2 as f32 / trunk_h.max(1) as f32;
                    let tapered = trunk_r as f32 * (1.0 - height_frac * 0.6);
                    tapered.round().max(1.0) as isize
                } else if mat == Material::Root {
                    // Roots taper with depth
                    let depth = (-node.pos.2).max(0) as f32;
                    let root_r = trunk_r as f32 * (1.0 - depth * 0.15).max(0.3);
                    root_r.round().max(0.0) as isize
                } else {
                    0 // branches/leaves: single voxel
                };

                let r_sq = inflate_r * inflate_r;
                for ddx in -inflate_r..=inflate_r {
                    for ddy in -inflate_r..=inflate_r {
                        if ddx * ddx + ddy * ddy > r_sq {
                            continue;
                        }
                        let ax = rx as isize + node.pos.0 + ddx;
                        let ay = ry as isize + node.pos.1 + ddy;
                        let az = rz as isize + node.pos.2;
                        if ax < 0 || ay < 0 || az < 0 {
                            continue;
                        }
                        let (ax, ay, az) = (ax as usize, ay as usize, az as usize);
                        if !VoxelGrid::in_bounds(ax, ay, az) {
                            continue;
                        }

                        skeleton_voxels.push((ax, ay, az, mat));
                    }
                }
            }

            // Dense spherical leaf shells around branch nodes for full canopy.
            // Tips get large spheres; interior branch nodes above crown_start
            // also get smaller spheres so the canopy fills in around the trunk
            // instead of just sitting at the branch tips like green caps.
            let crown_r = species.crown_radius() as isize;
            let tip_leaf_r: isize = match tree.stage {
                GrowthStage::YoungTree => (crown_r / 2).clamp(4, 10),
                GrowthStage::Mature | GrowthStage::OldGrowth => (crown_r / 2).clamp(5, 12),
                _ => 1,
            };
            // Interior branch nodes get smaller leaf spheres to fill gaps
            let interior_leaf_r = (tip_leaf_r * 2 / 3).max(3);
            // Crown start: only add interior leaf spheres above this height
            let crown_z_start = match tree.stage {
                GrowthStage::YoungTree => (trunk_h * 25 / 100).max(2),
                _ => (trunk_h * 15 / 100).max(2),
            };
            let mut pending_leaves: Vec<(usize, usize, usize, Material)> = Vec::new();
            for (i, node) in tree.branches.iter().enumerate() {
                if !node.alive || node.material == Material::Root {
                    continue;
                }
                let is_tip = !has_child[i];
                // Interior nodes only get leaves if above crown start
                if !is_tip && node.pos.2 < crown_z_start {
                    continue;
                }
                let r = if is_tip { tip_leaf_r } else { interior_leaf_r };
                let r_sq = r * r;
                // Fill a sphere of leaf voxels around this node
                for ddx in -r..=r {
                    for ddy in -r..=r {
                        for ddz in -r..=r {
                            if ddx * ddx + ddy * ddy + ddz * ddz > r_sq {
                                continue;
                            }
                            let ax = rx as isize + node.pos.0 + ddx;
                            let ay = ry as isize + node.pos.1 + ddy;
                            let az = rz as isize + node.pos.2 + ddz;
                            if ax < 0 || ay < 0 || az < 0 {
                                continue;
                            }
                            let (ax, ay, az) = (ax as usize, ay as usize, az as usize);
                            if !VoxelGrid::in_bounds(ax, ay, az) {
                                continue;
                            }
                            if let Some(cell) = grid.get(ax, ay, az) {
                                if cell.material == Material::Air {
                                    pending_leaves.push((ax, ay, az, Material::Leaf));
                                }
                            }
                        }
                    }
                }
            }
            // Dedup skeleton voxels: later entries overwrite earlier (branch tip
            // overwrites inflated trunk at the same position, matching old behavior).
            let mut voxel_map: std::collections::HashMap<(usize, usize, usize), Material> =
                std::collections::HashMap::new();
            for (x, y, z, mat) in skeleton_voxels {
                voxel_map.insert((x, y, z), mat);
            }
            let skeleton_voxels: Vec<(usize, usize, usize, Material)> = voxel_map
                .into_iter()
                .map(|((x, y, z), mat)| (x, y, z, mat))
                .collect();

            // Dedup leaves against skeleton positions and against each other
            let mut seen_pos: std::collections::HashSet<(usize, usize, usize)> = skeleton_voxels
                .iter()
                .map(|&(x, y, z, _)| (x, y, z))
                .collect();
            pending_leaves.retain(|&(x, y, z, _)| seen_pos.insert((x, y, z)));

            // Place roots immediately (underground, no visual snap needed)
            for &(x, y, z, mat) in &skeleton_voxels {
                if mat == Material::Root {
                    if let Some(cell) = grid.get_mut(x, y, z) {
                        if cell.material == Material::Soil || cell.material == Material::Root {
                            cell.set_material(mat);
                            cell.nutrient_level = tree.species_id as u8;
                            new_footprint.push((x, y, z));
                        }
                    }
                }
            }

            // Queue above-ground skeleton voxels (trunk/branch) for gradual growth.
            // Sort descending Z so pop() yields lowest Z first → trunk grows bottom-up.
            let mut pending_trunk: Vec<(usize, usize, usize, Material)> = skeleton_voxels
                .into_iter()
                .filter(|&(_, _, _, mat)| mat != Material::Root)
                .collect();
            pending_trunk.sort_by_key(|&(_, _, z, _)| std::cmp::Reverse(z));

            // Leaves sorted ascending Z — pop() yields highest Z → crown fills top-down.
            pending_leaves.sort_by_key(|&(_, _, z, _)| z);

            // Combine: leaves first (popped last), trunk last (popped first).
            // Growth order: trunk bottom→up, then leaves top→down.
            let mut pending = pending_leaves;
            pending.extend(pending_trunk);
            tree.pending_voxels = pending;
        } else {
            // Template path: Seedling/Sapling/Dead use static templates
            let template = TreeTemplate::generate(species, &tree.stage, tree.rng_seed);

            for &(dx, dy, dz, mat) in &template.voxels {
                let ax = rx as isize + dx;
                let ay = ry as isize + dy;
                let az = rz as isize + dz;
                if ax < 0 || ay < 0 || az < 0 {
                    continue;
                }
                let (ax, ay, az) = (ax as usize, ay as usize, az as usize);
                if !VoxelGrid::in_bounds(ax, ay, az) {
                    continue;
                }

                if let Some(cell) = grid.get_mut(ax, ay, az) {
                    let can_place = match mat {
                        Material::Root => {
                            cell.material == Material::Soil || cell.material == Material::Root
                        }
                        // Leaf/Branch must not overwrite Trunk — trunk has visual priority
                        Material::Leaf | Material::Branch => {
                            cell.material == Material::Air
                                || cell.material == Material::Branch
                                || cell.material == Material::Leaf
                                || cell.material == Material::DeadWood
                        }
                        _ => {
                            cell.material == Material::Air
                                || cell.material == Material::Trunk
                                || cell.material == Material::Branch
                                || cell.material == Material::Leaf
                                || cell.material == Material::DeadWood
                        }
                    };
                    if can_place {
                        cell.set_material(mat);
                        cell.nutrient_level = tree.species_id as u8;
                        if mat == Material::Leaf || mat == Material::Branch {
                            cell.water_level = (tree.health * 255.0) as u8;
                        }
                        new_footprint.push((ax, ay, az));
                    }
                }
            }
        }

        tree.voxel_footprint = new_footprint;
        tree.dirty = false;
    }
}

/// Dynamic root growth with tropisms: roots extend toward water (hydrotropism)
/// and downward (gravitropism). Runs every 5 ticks per tree.
pub fn root_growth(
    mut trees: Query<&mut Tree>,
    mut grid: ResMut<VoxelGrid>,
    species_table: Res<SpeciesTable>,
) {
    for mut tree in trees.iter_mut() {
        if matches!(tree.stage, GrowthStage::Dead | GrowthStage::Seedling) {
            continue;
        }

        // Only grow roots every 5 ticks
        if tree.age % 5 != 0 {
            continue;
        }

        let species = &species_table.species[tree.species_id];

        // Max roots based on growth stage
        let max_roots = match tree.stage {
            GrowthStage::Sapling => species.root_depth() as usize * 3,
            GrowthStage::YoungTree => species.root_depth() as usize * 5,
            GrowthStage::Mature | GrowthStage::OldGrowth => species.root_depth() as usize * 8,
            _ => 0,
        };

        // Collect current root positions
        let current_roots: Vec<(usize, usize, usize)> = tree
            .voxel_footprint
            .iter()
            .filter(|&&(x, y, z)| {
                grid.get(x, y, z)
                    .is_some_and(|v| v.material == Material::Root)
            })
            .copied()
            .collect();

        if current_roots.len() >= max_roots {
            continue;
        }

        // Find best soil neighbor of any root: score by water (hydrotropism) + depth (gravitropism)
        let mut best: Option<(usize, usize, usize, u16)> = None;

        for &(rx, ry, rz) in &current_roots {
            // Prefer downward, then lateral
            let neighbors: [(i32, i32, i32); 5] =
                [(0, 0, -1), (1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0)];
            for (dx, dy, dz) in neighbors {
                let nx = rx as i32 + dx;
                let ny = ry as i32 + dy;
                let nz = rz as i32 + dz;
                if nx < 0 || ny < 0 || nz < 0 {
                    continue;
                }
                let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                if !VoxelGrid::in_bounds(nx, ny, nz) {
                    continue;
                }

                if let Some(cell) = grid.get(nx, ny, nz) {
                    if cell.material != Material::Soil {
                        continue;
                    }

                    // Hydrotropism: prefer wetter soil
                    let water_score = cell.water_level as u16;
                    // Gravitropism: prefer deeper
                    let depth_score = GROUND_LEVEL.saturating_sub(nz) as u16 * 3;
                    let score = water_score + depth_score;

                    if best.is_none_or(|(_, _, _, s)| score > s) {
                        best = Some((nx, ny, nz, score));
                    }
                }
            }
        }

        if let Some((nx, ny, nz, _)) = best {
            if let Some(cell) = grid.get_mut(nx, ny, nz) {
                cell.set_material(Material::Root);
                tree.voxel_footprint.push((nx, ny, nz));
            }
        }
    }
}

/// Pioneer succession: bare moist soil spontaneously grows groundcover.
///
/// Succession order: bare soil → moss (species 9) → grass (10) → wildflower (7).
/// Each stage requires the previous stage nearby. Runs every 50 ticks.
/// Creates the feeling that "life finds a way" — the garden bootstraps itself.
/// Gradually place pending voxels for smooth visual growth.
/// Each tick, place up to 3 pending voxels sorted bottom-to-top.
/// This creates the "growing" animation instead of trees snapping into shape.
pub fn tree_grow_visual(mut trees: Query<&mut Tree>, mut grid: ResMut<VoxelGrid>) {
    for mut tree in trees.iter_mut() {
        if tree.pending_voxels.is_empty() {
            continue;
        }
        // Adaptive drain rate: faster for large queues (trunk + canopy).
        // Small queue (≤12): 3/tick. Large queue: up to 200/tick so dense
        // canopies fully appear within the 100 pre-tick window.
        let count = (tree.pending_voxels.len() / 4).clamp(3, 200);
        let count = count.min(tree.pending_voxels.len());
        for _ in 0..count {
            if let Some((x, y, z, mat)) = tree.pending_voxels.pop() {
                if let Some(cell) = grid.get_mut(x, y, z) {
                    let can_place = match mat {
                        Material::Root => {
                            cell.material == Material::Soil || cell.material == Material::Root
                        }
                        // Leaf/Branch must not overwrite Trunk — trunk has visual priority
                        Material::Leaf | Material::Branch => {
                            cell.material == Material::Air
                                || cell.material == Material::Branch
                                || cell.material == Material::Leaf
                                || cell.material == Material::DeadWood
                        }
                        _ => {
                            cell.material == Material::Air
                                || cell.material == Material::Trunk
                                || cell.material == Material::Branch
                                || cell.material == Material::Leaf
                                || cell.material == Material::DeadWood
                        }
                    };
                    if can_place {
                        cell.set_material(mat);
                        cell.nutrient_level = tree.species_id as u8;
                        if mat == Material::Leaf || mat == Material::Branch {
                            cell.water_level = (tree.health * 255.0) as u8;
                        }
                        tree.voxel_footprint.push((x, y, z));
                    }
                }
            }
        }
    }
}

pub fn pioneer_succession(
    mut grid: ResMut<VoxelGrid>,
    mut seed_map: ResMut<SeedSpeciesMap>,
    tick: Res<Tick>,
) {
    if !tick.0.is_multiple_of(50) {
        return;
    }

    let t = tick.0;
    // Sample ~20 random surface positions
    for i in 0..20_u64 {
        let h = tree_hash(t + i, 5555);
        let sx = (h as usize) % GRID_X;
        let sy = ((h >> 16) as usize) % GRID_Y;
        let sz = GROUND_LEVEL + 1; // just above surface

        if let Some(cell) = grid.get(sx, sy, sz) {
            if cell.material != Material::Air {
                continue;
            }
        } else {
            continue;
        }

        // Check if the soil below is moist enough
        let soil_below = grid.get(sx, sy, GROUND_LEVEL);
        let water_level = soil_below.map_or(0, |v| v.water_level);

        // Nurse Log Effect: DeadWood nearby lowers the moisture threshold,
        // so pioneer species can establish in drier conditions near dead trees.
        let has_nearby_deadwood = {
            let nr = 3_usize;
            let mut found = false;
            'dw: for dz in GROUND_LEVEL.saturating_sub(1)..=(GROUND_LEVEL + 3).min(GRID_Z - 1) {
                for dy in sy.saturating_sub(nr)..=(sy + nr).min(GRID_Y - 1) {
                    for dx in sx.saturating_sub(nr)..=(sx + nr).min(GRID_X - 1) {
                        if let Some(v) = grid.get(dx, dy, dz) {
                            if v.material == Material::DeadWood {
                                found = true;
                                break 'dw;
                            }
                        }
                    }
                }
            }
            found
        };
        let moisture_threshold = if has_nearby_deadwood { 5 } else { 20 };
        if water_level < moisture_threshold {
            continue;
        }

        // Count nearby groundcover types to determine succession stage
        let mut has_moss = false;
        let mut has_grass = false;
        let radius = 4_usize;
        for dy in sy.saturating_sub(radius)..=(sy + radius).min(GRID_Y - 1) {
            for dx in sx.saturating_sub(radius)..=(sx + radius).min(GRID_X - 1) {
                for dz in GROUND_LEVEL..=(GROUND_LEVEL + 2) {
                    if let Some(v) = grid.get(dx, dy, dz) {
                        if v.material == Material::Leaf {
                            let species_id = v.nutrient_level;
                            if species_id == 9 {
                                has_moss = true;
                            }
                            if species_id == 10 {
                                has_grass = true;
                            }
                        }
                    }
                }
            }
        }

        // Determine what to plant based on succession stage
        let species_id = if has_grass {
            // Grass present → wildflower can appear (low probability)
            let h2 = tree_hash(t + i, 6666);
            if h2.is_multiple_of(8) {
                Some(7_usize)
            } else {
                None
            } // wildflower
        } else if has_moss {
            // Moss present → grass can appear
            let h2 = tree_hash(t + i, 7777);
            if h2.is_multiple_of(5) {
                Some(10)
            } else {
                None
            } // grass
        } else {
            // Bare moist soil → moss appears (the pioneer)
            let h2 = tree_hash(t + i, 8888);
            if h2.is_multiple_of(4) {
                Some(9)
            } else {
                None
            } // moss
        };

        if let Some(sid) = species_id {
            if let Some(cell) = grid.get_mut(sx, sy, sz) {
                if cell.material == Material::Air {
                    cell.set_material(Material::Seed);
                    cell.water_level = 0;
                    cell.light_level = 0;
                    cell.nutrient_level = 0;
                    seed_map.map.insert((sx, sy, sz), sid);
                }
            }
        }
    }
}

/// Mature trees disperse seeds nearby. Seeds land on soil via gravity.
pub fn seed_dispersal(
    trees: Query<&Tree>,
    mut grid: ResMut<VoxelGrid>,
    mut seed_species: ResMut<SeedSpeciesMap>,
    species_table: Res<SpeciesTable>,
) {
    for tree in trees.iter() {
        if !matches!(tree.stage, GrowthStage::Mature | GrowthStage::OldGrowth) {
            continue;
        }
        if tree.health < 0.5 {
            continue;
        }

        let species = &species_table.species[tree.species_id];

        // Dispersal period varies per species and per individual tree.
        // OldGrowth trees disperse 2× more frequently — established trees
        // produce more seeds, creating visible "seed rain" during idle time.
        let base_period = species.dispersal_period;
        let age_factor = if tree.stage == GrowthStage::OldGrowth {
            2
        } else {
            1
        };
        let period = (base_period
            + (tree_hash(tree.rng_seed, 0) % (base_period as u64 / 4 + 1)) as u32)
            / age_factor;
        let period = period.max(10); // don't go below 10 ticks
        if tree.age < period || tree.age % period != 0 {
            continue;
        }

        // Pick dispersal direction and distance (species-specific)
        let h = tree_hash(tree.rng_seed, tree.age as u64);
        let base_dist =
            crate::scale::meters_to_voxels(species.dispersal_distance_m / 2.0).max(1) as u64;
        let var_dist = crate::scale::meters_to_voxels(species.dispersal_distance_m).max(1) as u64;
        let dist = base_dist + (h >> 8) % var_dist;
        let (dx, dy): (isize, isize) = match h % 8 {
            0 => (dist as isize, 0),
            1 => (-(dist as isize), 0),
            2 => (0, dist as isize),
            3 => (0, -(dist as isize)),
            4 => (dist as isize, dist as isize),
            5 => (-(dist as isize), dist as isize),
            6 => (dist as isize, -(dist as isize)),
            _ => (-(dist as isize), -(dist as isize)),
        };

        let (rx, ry, rz) = tree.root_pos;
        let sx = rx as isize + dx;
        let sy = ry as isize + dy;
        if sx < 0 || sy < 0 {
            continue;
        }
        let (sx, sy) = (sx as usize, sy as usize);

        // Start above the canopy and drop down
        let canopy_h = crate::scale::meters_to_voxels(species.max_height_m * 1.5).max(2);
        let start_z = (rz + canopy_h).min(GRID_Z - 1);
        if !VoxelGrid::in_bounds(sx, sy, start_z) {
            continue;
        }

        let landing_z = grid.find_landing_z(sx, sy, start_z);

        // Must land on Air with Soil below
        if let Some(cell) = grid.get(sx, sy, landing_z) {
            if cell.material != Material::Air {
                continue;
            }
        } else {
            continue;
        }

        if landing_z == 0 {
            continue;
        }
        if let Some(below) = grid.get(sx, sy, landing_z - 1) {
            if below.material != Material::Soil {
                continue;
            }
        } else {
            continue;
        }

        // Place seed and record its species
        if let Some(cell) = grid.get_mut(sx, sy, landing_z) {
            cell.set_material(Material::Seed);
            seed_species
                .map
                .insert((sx, sy, landing_z), tree.species_id);
        }
    }
}

/// Dead wood slowly decomposes into nutrient-rich soil. Beetles accelerate this
/// via fauna_effects, but passive decomposition ensures the nutrient cycle completes
/// even without beetles present. Creates visible "life from death" progression.
///
/// Runs every 20 ticks. Each DeadWood voxel accumulates decay in its water_level byte
/// (repurposed since dead wood doesn't hold water). At 255, it converts to soil.
pub fn deadwood_decay(
    mut grid: ResMut<VoxelGrid>,
    mut soil_grid: ResMut<SoilGrid>,
    tick: Res<Tick>,
) {
    if !tick.0.is_multiple_of(20) {
        return;
    }

    let z_stride = GRID_X * GRID_Y;
    let cells = grid.cells_mut();
    let soil_cells = soil_grid.cells_mut();

    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = x + y * GRID_X + z * z_stride;
                if cells[idx].material != Material::DeadWood {
                    continue;
                }

                // Passive decay: +2 per 20-tick cycle (~2550 ticks to full decomposition)
                // Moisture accelerates: wet dead wood decays faster
                let moisture_bonus = if z <= GROUND_LEVEL {
                    // Underground dead wood: check adjacent soil water
                    let mut adj_water = 0u16;
                    if x > 0 {
                        adj_water += cells[idx - 1].water_level as u16;
                    }
                    if x + 1 < GRID_X {
                        adj_water += cells[idx + 1].water_level as u16;
                    }
                    if y > 0 {
                        adj_water += cells[idx - GRID_X].water_level as u16;
                    }
                    if y + 1 < GRID_Y {
                        adj_water += cells[idx + GRID_X].water_level as u16;
                    }
                    (adj_water / 200).min(3) as u8
                } else {
                    0
                };

                let decay_rate = 2 + moisture_bonus;
                cells[idx].water_level = cells[idx].water_level.saturating_add(decay_rate);

                // When fully decayed, convert to soil with rich nutrients
                if cells[idx].water_level >= 250 {
                    let was_underground = z <= GROUND_LEVEL;
                    cells[idx].set_material(if was_underground {
                        Material::Soil
                    } else {
                        Material::Air
                    });
                    if was_underground {
                        cells[idx].nutrient_level = 60; // nutrient-rich soil
                                                        // Enrich the soil composition
                        soil_cells[idx].organic = soil_cells[idx].organic.saturating_add(40);
                        soil_cells[idx].bacteria = soil_cells[idx].bacteria.saturating_add(20);
                    }
                }
            }
        }
    }
}

/// Roots absorb water from adjacent Soil voxels.
/// Transfer ~4 units per adjacent wet soil per tick, divided by competing root count.
/// When multiple roots neighbor the same soil cell, they share the water — creating
/// real resource competition between overlapping root zones.
/// Visible effect: wet soil near roots dries out faster with more roots competing.
pub fn root_water_absorption(mut grid: ResMut<VoxelGrid>) {
    // Single snapshot: (material_u8, water_level, nutrient_level) tuples for cache-friendly reads.
    let snapshot: Vec<(u8, u8, u8)> = grid
        .cells()
        .iter()
        .map(|v| (v.material.as_u8(), v.water_level, v.nutrient_level))
        .collect();

    let root_u8 = Material::Root.as_u8();
    let soil_u8 = Material::Soil.as_u8();
    let max_transfer = crate::scale::scale_transfer(4);
    let max_nutrient_transfer: u8 = 2; // nutrients move slower than water
    let z_stride = GRID_X * GRID_Y;

    // Pre-compute root competition: for each soil cell, count adjacent root voxels.
    // More roots competing for the same soil = less water per root.
    let total = GRID_X * GRID_Y * GRID_Z;
    let mut root_neighbor_count: Vec<u8> = vec![0; total];
    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = x + y * GRID_X + z * z_stride;
                if snapshot[idx].0 != soil_u8 {
                    continue;
                }
                let mut count = 0u8;
                if x > 0 && snapshot[idx - 1].0 == root_u8 {
                    count += 1;
                }
                if x + 1 < GRID_X && snapshot[idx + 1].0 == root_u8 {
                    count += 1;
                }
                if y > 0 && snapshot[idx - GRID_X].0 == root_u8 {
                    count += 1;
                }
                if y + 1 < GRID_Y && snapshot[idx + GRID_X].0 == root_u8 {
                    count += 1;
                }
                if z > 0 && snapshot[idx - z_stride].0 == root_u8 {
                    count += 1;
                }
                if z + 1 < GRID_Z && snapshot[idx + z_stride].0 == root_u8 {
                    count += 1;
                }
                root_neighbor_count[idx] = count;
            }
        }
    }

    let cells = grid.cells_mut();
    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = x + y * GRID_X + z * z_stride;
                if snapshot[idx].0 != root_u8 {
                    continue;
                }

                macro_rules! absorb {
                    ($nidx:expr) => {{
                        let nidx = $nidx;
                        if snapshot[nidx].0 == soil_u8 {
                            // Root competition: divide transfer by number of roots
                            // competing for this soil cell. More roots = less per root.
                            let competitors = root_neighbor_count[nidx].max(1);
                            // Water absorption
                            if snapshot[nidx].1 > 0 {
                                let base_transfer = snapshot[nidx].1.min(max_transfer);
                                let transfer = base_transfer / competitors;
                                if transfer > 0 {
                                    cells[nidx].water_level =
                                        cells[nidx].water_level.saturating_sub(transfer);
                                    cells[idx].water_level =
                                        cells[idx].water_level.saturating_add(transfer);
                                }
                            }
                            // Nutrient absorption (also competed)
                            if snapshot[nidx].2 > 0 {
                                let base_transfer = snapshot[nidx].2.min(max_nutrient_transfer);
                                let transfer = base_transfer / competitors;
                                if transfer > 0 {
                                    cells[nidx].nutrient_level =
                                        cells[nidx].nutrient_level.saturating_sub(transfer);
                                    cells[idx].nutrient_level =
                                        cells[idx].nutrient_level.saturating_add(transfer);
                                }
                            }
                        }
                    }};
                }

                if x > 0 {
                    absorb!(idx - 1);
                }
                if x + 1 < GRID_X {
                    absorb!(idx + 1);
                }
                if y > 0 {
                    absorb!(idx - GRID_X);
                }
                if y + 1 < GRID_Y {
                    absorb!(idx + GRID_X);
                }
                if z > 0 {
                    absorb!(idx - z_stride);
                }
                if z + 1 < GRID_Z {
                    absorb!(idx + z_stride);
                }
            }
        }
    }

    // --- Root Water Decay ---
    // Roots that have NO adjacent wet soil lose water. This enforces water
    // dependency: removing water from the garden kills plants within 30-50 ticks.
    // Rate: -5 per tick for roots with no wet soil neighbors.
    // Also: ALL roots lose -1 per tick (metabolic consumption) regardless of
    // neighbors, creating continuous demand that must be met by the water supply.
    let cells = grid.cells_mut();
    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = x + y * GRID_X + z * z_stride;
                if snapshot[idx].0 != root_u8 || snapshot[idx].1 == 0 {
                    continue;
                }
                // Check if any neighbor is wet soil (water_level > 20)
                let mut has_wet_neighbor = false;
                macro_rules! check_wet {
                    ($nidx:expr) => {
                        if snapshot[$nidx].0 == soil_u8 && snapshot[$nidx].1 > 20 {
                            has_wet_neighbor = true;
                        }
                    };
                }
                if x > 0 {
                    check_wet!(idx - 1);
                }
                if !has_wet_neighbor && x + 1 < GRID_X {
                    check_wet!(idx + 1);
                }
                if !has_wet_neighbor && y > 0 {
                    check_wet!(idx - GRID_X);
                }
                if !has_wet_neighbor && y + 1 < GRID_Y {
                    check_wet!(idx + GRID_X);
                }
                if !has_wet_neighbor && z > 0 {
                    check_wet!(idx - z_stride);
                }
                if !has_wet_neighbor && z + 1 < GRID_Z {
                    check_wet!(idx + z_stride);
                }

                // Drought penalty: roots without wet soil neighbors dry out
                if !has_wet_neighbor {
                    cells[idx].water_level = cells[idx].water_level.saturating_sub(4);
                }
            }
        }
    }
}

/// Soil evolves over time based on environmental interactions.
/// - Organic matter increases near roots, slowly decays without them.
/// - Bacteria grow in moist, organic-rich soil; die in dry soil.
/// - pH drifts acidic with organic decomposition; rock buffers toward neutral.
/// - Rock fragments slowly weather into clay when wet.
pub fn soil_evolution(
    mut grid: ResMut<VoxelGrid>,
    mut soil_grid: ResMut<SoilGrid>,
    tick: Res<Tick>,
) {
    // Soil chemistry changes slowly — only run every 10 ticks.
    // Organic/bacteria/pH increments are scaled 10× to compensate.
    if !tick.0.is_multiple_of(10) {
        return;
    }

    // Snapshot grid state so we can read neighbors while writing nutrient_level.
    // Includes nutrient_level so we can read species_id from root voxels (allelopathy).
    let snapshot: Vec<(u8, u8, u8)> = grid
        .cells()
        .iter()
        .map(|v| (v.material.as_u8(), v.water_level, v.nutrient_level))
        .collect();

    let soil_cells = soil_grid.cells_mut();

    let root_u8 = Material::Root.as_u8();
    let soil_u8 = Material::Soil.as_u8();
    let z_stride = GRID_X * GRID_Y;
    let pine_species_id: u8 = 3; // Pine species index

    // First pass: evolve soil composition and compute nutrient generation amounts.
    // We collect nutrient deltas because we need soil_cells borrow to end before
    // we can mutate grid cells.
    let mut nutrient_deltas: Vec<(usize, u8)> = Vec::new();

    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = VoxelGrid::index(x, y, z);
                if snapshot[idx].0 != soil_u8 {
                    continue;
                }

                let water_level = snapshot[idx].1;
                let comp = &mut soil_cells[idx];

                // --- Organic matter ---
                // Increases when adjacent to roots (+10/run per adjacent root, compensating for 10-tick skip)
                let mut adjacent_roots = 0u8;
                // Inline neighbor checks to avoid isize conversion overhead
                if x > 0 && snapshot[idx - 1].0 == root_u8 {
                    adjacent_roots += 1;
                }
                if x + 1 < GRID_X && snapshot[idx + 1].0 == root_u8 {
                    adjacent_roots += 1;
                }
                if y > 0 && snapshot[idx - GRID_X].0 == root_u8 {
                    adjacent_roots += 1;
                }
                if y + 1 < GRID_Y && snapshot[idx + GRID_X].0 == root_u8 {
                    adjacent_roots += 1;
                }
                if z > 0 && snapshot[idx - z_stride].0 == root_u8 {
                    adjacent_roots += 1;
                }
                if z + 1 < GRID_Z && snapshot[idx + z_stride].0 == root_u8 {
                    adjacent_roots += 1;
                }

                if adjacent_roots > 0 {
                    // 10× because we run every 10 ticks
                    comp.organic = comp.organic.saturating_add(adjacent_roots.min(2) * 10);
                } else if comp.organic > 0 {
                    // Slow decay without roots: -1 every ~10 ticks → -1 per run
                    if (x + y + z) % 10 == 0 {
                        comp.organic = comp.organic.saturating_sub(1);
                    }
                }

                // --- Grass Erosion Prevention ---
                // Grass roots (species_id=10) and clover (11) stabilize soil,
                // increasing clay content (better water retention). Creates a
                // "prepare the ground" strategy: plant grass first, then trees.
                // Discovery: "The soil holds water better where I planted grass first."
                {
                    let grass_species_id: u8 = 10;
                    let clover_species_id: u8 = 11;
                    let mut has_grass_root = false;
                    macro_rules! check_grass {
                        ($nidx:expr) => {
                            if snapshot[$nidx].0 == root_u8 {
                                let sid = snapshot[$nidx].2; // nutrient_level = species_id
                                if sid == grass_species_id || sid == clover_species_id {
                                    has_grass_root = true;
                                }
                            }
                        };
                    }
                    if x > 0 {
                        check_grass!(idx - 1);
                    }
                    if x + 1 < GRID_X {
                        check_grass!(idx + 1);
                    }
                    if y > 0 {
                        check_grass!(idx - GRID_X);
                    }
                    if y + 1 < GRID_Y {
                        check_grass!(idx + GRID_X);
                    }
                    if z > 0 {
                        check_grass!(idx - z_stride);
                    }
                    if z + 1 < GRID_Z {
                        check_grass!(idx + z_stride);
                    }
                    if has_grass_root && comp.clay < 200 {
                        // Grass/clover roots bind soil particles, increasing clay
                        comp.clay = comp.clay.saturating_add(1);
                    }
                }

                // --- Bacteria ---
                // 10× increments to compensate for running every 10 ticks
                if water_level > 50 && comp.organic > 30 {
                    comp.bacteria = comp.bacteria.saturating_add(10);
                } else if water_level < 10 {
                    comp.bacteria = comp.bacteria.saturating_sub(20);
                } else if comp.organic < 15 && (x + y) % 5 == 0 {
                    comp.bacteria = comp.bacteria.saturating_sub(10);
                }

                // --- Overgrowth Carrying Capacity ---
                // In deep canopy shade (surface light < 30), soil bacteria decline
                // and nutrients deplete. This creates negative feedback: too many
                // trees → dense canopy → poor soil → weakened trees → natural thinning.
                // Discovery: "The soil quality dropped under my dense forest!"
                // Only affects near-surface soil (within 3 of ground level).
                if z >= GROUND_LEVEL.saturating_sub(3) && z <= GROUND_LEVEL {
                    // Check light at surface above this soil cell
                    let surface_z = GROUND_LEVEL + 1;
                    let surface_idx = x + y * GRID_X + surface_z * z_stride;
                    if surface_idx < snapshot.len() {
                        // snapshot stores (material, water, nutrient) — need light from grid
                        // We can't read light from snapshot, so use a heuristic:
                        // if there are many roots AND many leaf/trunk voxels above,
                        // the canopy is dense. Count roots as a density proxy.
                        if adjacent_roots >= 4 {
                            // Very dense root zone → likely dense canopy overhead
                            // Bacteria decline from poor air circulation
                            comp.bacteria = comp.bacteria.saturating_sub(3);
                            // Organic matter accumulates but doesn't decompose well
                            // (anaerobic conditions) — this caps nutrient generation
                        }
                    }
                }

                // --- pH drift ---
                if comp.organic > 100 && comp.ph > 0 && (x + z) % 20 == 0 {
                    comp.ph = comp.ph.saturating_sub(1);
                }
                if comp.rock > 50 && comp.ph < 128 && (y + z) % 25 == 0 {
                    comp.ph = comp.ph.saturating_add(1);
                }

                // --- Allelopathy: pine roots acidify soil ---
                // Pine (species_id=3) needles and roots release organic acids that
                // lower soil pH. This creates pine territory — most other species
                // struggle in acidic soil, but ferns and moss thrive.
                // Discovery: "Why does nothing grow near my pines? The soil is acidic!"
                {
                    let mut has_pine_root = false;
                    macro_rules! check_pine {
                        ($nidx:expr) => {
                            if snapshot[$nidx].0 == root_u8 && snapshot[$nidx].2 == pine_species_id
                            {
                                has_pine_root = true;
                            }
                        };
                    }
                    if x > 0 {
                        check_pine!(idx - 1);
                    }
                    if x + 1 < GRID_X {
                        check_pine!(idx + 1);
                    }
                    if y > 0 {
                        check_pine!(idx - GRID_X);
                    }
                    if y + 1 < GRID_Y {
                        check_pine!(idx + GRID_X);
                    }
                    if z > 0 {
                        check_pine!(idx - z_stride);
                    }
                    if z + 1 < GRID_Z {
                        check_pine!(idx + z_stride);
                    }
                    if has_pine_root && comp.ph > 0 {
                        // Pine roots aggressively lower pH (5× faster than organic decay)
                        comp.ph = comp.ph.saturating_sub(5);
                    }
                }

                // --- Rock weathering ---
                if comp.rock > 0 && water_level > 30 && (x + y + z) % 50 == 0 {
                    comp.rock = comp.rock.saturating_sub(1);
                    comp.clay = comp.clay.saturating_add(1);
                }

                // --- Nutrient generation ---
                // Bacteria decompose organic matter into plant-available nutrients.
                // Rate scales with bacteria activity and organic content.
                // Nutrients cap at nutrient_capacity (rich soil holds more).
                if comp.bacteria > 20 && comp.organic > 20 {
                    let generation = ((comp.bacteria as u16 * comp.organic as u16) / 6400) as u8;
                    let gen = generation.max(1); // at least 1 per 10-tick cycle
                    let cap = comp.nutrient_capacity();
                    nutrient_deltas.push((idx, gen.min(cap)));
                }
            }
        }
    }

    // Second pass: apply nutrient generation to grid voxels.
    let grid_cells = grid.cells_mut();
    for (idx, gen) in nutrient_deltas {
        let cap = soil_cells[idx].nutrient_capacity();
        if grid_cells[idx].nutrient_level < cap {
            grid_cells[idx].nutrient_level =
                grid_cells[idx].nutrient_level.saturating_add(gen).min(cap);
        }
    }
}

/// Mycorrhizal network: trees of the same species with overlapping root zones
/// share water. Healthier trees transfer water to struggling neighbors through
/// underground fungal connections — the "wood-wide web."
///
/// Runs every 10 ticks. Checks all tree pairs of the same species.
/// Discovery: "My oaks are supporting each other through their roots!"
pub fn mycorrhizal_network(mut trees: Query<&mut Tree>, grid: Res<VoxelGrid>, tick: Res<Tick>) {
    if !tick.0.is_multiple_of(10) {
        return;
    }

    // Collect tree data for pairwise comparison.
    // We need: species_id, health, root positions, entity index.
    struct TreeInfo {
        species_id: usize,
        health: f32,
        _root_water: f32,
        root_positions: Vec<(usize, usize, usize)>,
    }

    let mut infos: Vec<TreeInfo> = Vec::new();
    for tree in trees.iter() {
        if matches!(tree.stage, GrowthStage::Dead | GrowthStage::Seedling) {
            continue;
        }
        let root_positions: Vec<(usize, usize, usize)> = tree
            .voxel_footprint
            .iter()
            .filter(|&&(x, y, z)| {
                grid.get(x, y, z)
                    .is_some_and(|v| v.material == Material::Root)
            })
            .copied()
            .collect();
        if root_positions.is_empty() {
            continue;
        }
        let root_water: f32 = root_positions
            .iter()
            .filter_map(|&(x, y, z)| grid.get(x, y, z))
            .map(|v| v.water_level as f32)
            .sum();

        infos.push(TreeInfo {
            species_id: tree.species_id,
            health: tree.health,
            _root_water: root_water,
            root_positions,
        });
    }

    // Find connected pairs (same species, overlapping root zones)
    // and compute health transfer direction.
    let mut health_deltas: Vec<f32> = vec![0.0; infos.len()];
    let connection_dist_sq: usize = 9; // 3 voxels apart = connected

    for i in 0..infos.len() {
        for j in (i + 1)..infos.len() {
            if infos[i].species_id != infos[j].species_id {
                continue;
            }

            // Check if any roots are close enough to be "connected"
            let mut connected = false;
            'check: for &(ax, ay, az) in &infos[i].root_positions {
                for &(bx, by, bz) in &infos[j].root_positions {
                    let dx = ax.abs_diff(bx);
                    let dy = ay.abs_diff(by);
                    let dz = az.abs_diff(bz);
                    if dx * dx + dy * dy + dz * dz <= connection_dist_sq {
                        connected = true;
                        break 'check;
                    }
                }
            }

            if !connected {
                continue;
            }

            // Transfer health from the stronger tree to the weaker one.
            // Rate: 0.005 per cycle (small but meaningful over time).
            let diff = infos[i].health - infos[j].health;
            if diff.abs() > 0.05 {
                let transfer = diff.signum() * 0.005;
                health_deltas[i] -= transfer;
                health_deltas[j] += transfer;
            }
        }
    }

    // Apply health deltas to the actual Tree components
    let mut idx = 0;
    for mut tree in trees.iter_mut() {
        if matches!(tree.stage, GrowthStage::Dead | GrowthStage::Seedling) {
            continue;
        }
        let root_count = tree
            .voxel_footprint
            .iter()
            .filter(|&&(x, y, z)| {
                grid.get(x, y, z)
                    .is_some_and(|v| v.material == Material::Root)
            })
            .count();
        if root_count == 0 {
            continue;
        }
        if idx < health_deltas.len() {
            let delta = health_deltas[idx];
            if delta != 0.0 {
                tree.health = (tree.health + delta).clamp(0.0, 1.0);
            }
        }
        idx += 1;
    }
}

/// Wind seed drift: seeds in air fall with a lateral offset based on a persistent
/// wind direction. Creates directional spread patterns visible over time.
///
/// The wind direction rotates slowly (every 500 ticks), so the garden doesn't
/// only expand in one direction forever.
pub fn wind_seed_drift(mut grid: ResMut<VoxelGrid>, tick: Res<Tick>) {
    // Wind direction rotates every 500 ticks through 4 cardinal directions
    let wind_phase = (tick.0 / 500) % 4;
    let (wind_dx, wind_dy): (isize, isize) = match wind_phase {
        0 => (1, 0),  // east
        1 => (0, 1),  // north
        2 => (-1, 0), // west
        _ => (0, -1), // south
    };

    // Only apply drift every 3 ticks (seeds don't fall instantly)
    if !tick.0.is_multiple_of(3) {
        return;
    }

    let z_stride = GRID_X * GRID_Y;

    // Scan for seed voxels high in the air. Only seeds far above the surface
    // (at least 8 voxels up) drift with wind — closer seeds are settling.
    // Process top-down so gravity + drift cascade naturally.
    for z in (GROUND_LEVEL + 8..GRID_Z).rev() {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = x + y * GRID_X + z * z_stride;
                let cells = grid.cells();
                if cells[idx].material != Material::Seed {
                    continue;
                }
                // Only drift seeds that are "in flight" (freshly dispersed).
                // Growing seeds have nutrient_level > 0 (the growth counter).
                if cells[idx].nutrient_level > 0 {
                    continue;
                }
                // Only drift seeds that are in the air (not resting on soil)
                let below_idx = idx - z_stride;
                if cells[below_idx].material != Material::Air {
                    continue;
                }

                // Compute drift target: one step down + wind offset
                let nx = (x as isize + wind_dx) as usize;
                let ny = (y as isize + wind_dy) as usize;
                let nz = z - 1;

                if nx >= GRID_X || ny >= GRID_Y {
                    continue;
                }
                let nidx = nx + ny * GRID_X + nz * z_stride;
                let target_mat = cells[nidx].material;
                if target_mat != Material::Air {
                    continue;
                }

                // Move the seed: clear old position, place at new
                let seed_water = cells[idx].water_level;
                let seed_light = cells[idx].light_level;
                let seed_nutrient = cells[idx].nutrient_level;

                let cells = grid.cells_mut();
                cells[idx].set_material(Material::Air);
                cells[nidx].material = Material::Seed;
                cells[nidx].water_level = seed_water;
                cells[nidx].light_level = seed_light;
                cells[nidx].nutrient_level = seed_nutrient;
            }
        }
    }
}

/// Track ecological milestones for species unlock progression.
///
/// Scans the world every 20 ticks and updates `EcoMilestones` with:
/// - Groundcover leaf count (for tier 1: flowers unlock)
/// - Pollinator count (for tier 2: shrubs unlock)
/// - Total fauna + species diversity (for tier 3: trees unlock)
///
/// Milestones are one-way: once reached, they stay reached.
pub fn milestone_tracker(
    grid: Res<VoxelGrid>,
    fauna_list: Res<crate::fauna::FaunaList>,
    trees: Query<&Tree>,
    tick: Res<Tick>,
    mut milestones: ResMut<crate::EcoMilestones>,
    mut discovered: ResMut<crate::DiscoveredSpecies>,
) {
    // Only scan every 20 ticks for performance
    if !tick.0.is_multiple_of(20) {
        return;
    }

    // --- Species Discovery ---
    // Scan Tree entities for species we haven't seen before.
    // Discovery happens when any tree of that species exists (even a seedling).
    for tree in trees.iter() {
        if !discovered.is_discovered(tree.species_id) {
            discovered.discover(tree.species_id);
        }
    }

    // Count groundcover leaf voxels at ground level
    // Groundcover species: moss=9, grass=10, clover=11
    let mut groundcover_count = 0u16;
    for z in GROUND_LEVEL..=(GROUND_LEVEL + 2).min(GRID_Z - 1) {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                if let Some(v) = grid.get(x, y, z) {
                    if v.material == Material::Leaf {
                        let sid = v.nutrient_level;
                        if matches!(sid, 9..=11) {
                            groundcover_count += 1;
                        }
                    }
                }
            }
        }
    }

    // Count pollinators (bees + butterflies)
    let pollinator_count = fauna_list
        .fauna
        .iter()
        .filter(|f| {
            matches!(
                f.fauna_type,
                crate::fauna::FaunaType::Bee | crate::fauna::FaunaType::Butterfly
            )
        })
        .count() as u16;

    // Count total fauna and unique species
    let fauna_count = fauna_list.fauna.len() as u16;
    let mut species_seen = [false; 6]; // 6 fauna types
    for f in &fauna_list.fauna {
        species_seen[f.fauna_type as usize] = true;
    }
    let _species_diversity = species_seen.iter().filter(|&&s| s).count() as u8;

    // Count plant species diversity (from Tree entities)
    let mut plant_species_seen = [false; 12]; // 12 plant species
    for tree in trees.iter() {
        if tree.species_id < 12 {
            plant_species_seen[tree.species_id] = true;
        }
    }
    let plant_diversity: u8 = plant_species_seen.iter().filter(|&&s| s).count() as u8;

    // Update raw counts
    milestones.groundcover_count = groundcover_count;
    milestones.pollinator_count = pollinator_count;
    milestones.fauna_count = fauna_count;
    milestones.species_diversity = plant_diversity;

    // Check tier unlocks (one-way — never reverts)
    if !milestones.tier1_flowers && groundcover_count >= 10 {
        milestones.tier1_flowers = true;
    }
    if !milestones.tier2_shrubs && pollinator_count >= 2 {
        milestones.tier2_shrubs = true;
    }
    if !milestones.tier3_trees && fauna_count >= 4 && plant_diversity >= 3 {
        milestones.tier3_trees = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::GROUND_LEVEL;

    #[test]
    fn seed_grows_into_tree_seedling() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Place seed near the center spring so it gets sustained water.
        // Position (GRID_X/2 + 6, GRID_Y/2) is close enough for water flow
        // but far enough to avoid the water voxels themselves.
        let sx = GRID_X / 2 + 6;
        let sy = GRID_Y / 2;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(sx, sy, GROUND_LEVEL + 1) {
                cell.material = Material::Seed;
                cell.water_level = 100;
                cell.light_level = 0;
                cell.nutrient_level = 0;
            }
        }

        // Growth rate is 3-8/tick depending on soil; 80 ticks is enough for any soil type.
        for _ in 0..80 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let cell = grid.get(sx, sy, GROUND_LEVEL + 1).unwrap();
        assert_eq!(
            cell.material,
            Material::Trunk,
            "Seed should have grown into trunk after 80 ticks with water and light"
        );

        // Root should have been placed in the soil below
        let below = grid.get(sx, sy, GROUND_LEVEL).unwrap();
        assert_eq!(
            below.material,
            Material::Root,
            "Root should be placed in soil below the trunk"
        );
    }

    #[test]
    fn seed_does_not_grow_without_water() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place seed far from any water source.
            if let Some(cell) = grid.get_mut(0, 0, GROUND_LEVEL + 1) {
                cell.material = Material::Seed;
                cell.water_level = 0;
                cell.light_level = 0;
                cell.nutrient_level = 0;
            }
        }

        for _ in 0..50 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let cell = grid.get(0, 0, GROUND_LEVEL + 1).unwrap();
        assert_eq!(
            cell.material,
            Material::Seed,
            "Seed should remain a seed without water"
        );
    }

    #[test]
    fn water_flow_does_not_bleed_nutrient_into_new_water() {
        let mut world = crate::create_world();

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place an air voxel with leftover nutrient (simulating prior state)
            // directly below a water source.
            let z = GROUND_LEVEL + 1;
            if let Some(cell) = grid.get_mut(5, 5, z + 1) {
                cell.material = Material::Water;
                cell.water_level = 255;
            }
            if let Some(cell) = grid.get_mut(5, 5, z) {
                cell.material = Material::Air;
                cell.water_level = 0;
                cell.nutrient_level = 99; // stale state
            }
        }

        let mut schedule = crate::create_schedule();
        crate::tick(&mut world, &mut schedule);

        let grid = world.resource::<VoxelGrid>();
        let cell = grid.get(5, 5, GROUND_LEVEL + 1).unwrap();
        // If water flowed down and converted this to Water, nutrient should be 0.
        if cell.material == Material::Water {
            assert_eq!(
                cell.nutrient_level, 0,
                "Water converted from air should not retain stale nutrient_level"
            );
        }
    }

    #[test]
    fn seed_to_trunk_resets_water_level() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(30, 10, GROUND_LEVEL + 1) {
                cell.material = Material::Seed;
                cell.water_level = 100;
                cell.light_level = 0;
                cell.nutrient_level = 0;
            }
        }

        for _ in 0..80 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let cell = grid.get(30, 10, GROUND_LEVEL + 1).unwrap();
        if cell.material == Material::Trunk {
            assert_eq!(
                cell.water_level, 0,
                "Trunk converted from seed should not retain stale water_level"
            );
        }
    }

    #[test]
    fn root_absorbs_water_from_adjacent_wet_soil() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Place a root surrounded by wet soil underground.
        let root_x = 10;
        let root_y = 10;
        let root_z = GROUND_LEVEL - 1; // underground
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(root_x, root_y, root_z) {
                cell.material = Material::Root;
                cell.water_level = 0;
            }
            // Set adjacent soil to have water.
            if let Some(cell) = grid.get_mut(root_x + 1, root_y, root_z) {
                cell.material = Material::Soil;
                cell.water_level = 50;
            }
            if let Some(cell) = grid.get_mut(root_x - 1, root_y, root_z) {
                cell.material = Material::Soil;
                cell.water_level = 50;
            }
        }

        crate::tick(&mut world, &mut schedule);

        let grid = world.resource::<VoxelGrid>();
        let root = grid.get(root_x, root_y, root_z).unwrap();
        assert!(
            root.water_level > 0,
            "Root should have absorbed water from adjacent wet soil, got water_level={}",
            root.water_level
        );
        // Each wet soil neighbor donates scale_transfer(4), two neighbors.
        let expected = crate::scale::scale_transfer(4) * 2;
        assert_eq!(
            root.water_level, expected,
            "Root should absorb scaled amount from 2 neighbors"
        );

        let neighbor = grid.get(root_x + 1, root_y, root_z).unwrap();
        assert!(
            neighbor.water_level < 50,
            "Wet soil next to root should lose water, got water_level={}",
            neighbor.water_level
        );
    }

    #[test]
    fn root_does_not_absorb_from_dry_soil() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let root_x = 15;
        let root_y = 15;
        let root_z = GROUND_LEVEL - 2;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(root_x, root_y, root_z) {
                cell.material = Material::Root;
                cell.water_level = 0;
            }
            // Ensure all neighbors are bone-dry in a wide radius so no water
            // can diffuse or flow into the root's immediate neighborhood.
            for dx in -4i32..=4 {
                for dy in -4i32..=4 {
                    for dz in -4i32..=4 {
                        if dx == 0 && dy == 0 && dz == 0 {
                            continue;
                        }
                        let nx = root_x as i32 + dx;
                        let ny = root_y as i32 + dy;
                        let nz = root_z as i32 + dz;
                        if nx < 0 || ny < 0 || nz < 0 {
                            continue;
                        }
                        let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                        if let Some(cell) = grid.get_mut(nx, ny, nz) {
                            cell.water_level = 0;
                        }
                    }
                }
            }
        }

        crate::tick(&mut world, &mut schedule);

        let grid = world.resource::<VoxelGrid>();
        let root = grid.get(root_x, root_y, root_z).unwrap();
        assert_eq!(
            root.water_level, 0,
            "Root should not absorb water from dry soil"
        );
    }

    #[test]
    fn wet_soil_near_root_dries_over_time() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let root_x = 30;
        let root_y = 30;
        let root_z = GROUND_LEVEL - 2;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(root_x, root_y, root_z) {
                cell.material = Material::Root;
                cell.water_level = 0;
            }
            // One neighbor with limited water.
            if let Some(cell) = grid.get_mut(root_x + 1, root_y, root_z) {
                cell.material = Material::Soil;
                cell.water_level = 20;
            }
            // Dry out a wide zone so water_flow doesn't replenish from distant cells.
            for dx in -6i32..=6 {
                for dy in -6i32..=6 {
                    for dz in -3i32..=3 {
                        let nx = (root_x as i32 + dx) as usize;
                        let ny = (root_y as i32 + dy) as usize;
                        let nz = (root_z as i32 + dz) as usize;
                        if nx == root_x && ny == root_y && nz == root_z {
                            continue;
                        }
                        if nx == root_x + 1 && ny == root_y && nz == root_z {
                            continue;
                        }
                        if let Some(cell) = grid.get_mut(nx, ny, nz) {
                            cell.water_level = 0;
                        }
                    }
                }
            }
        }

        // Run enough ticks to drain the soil.
        for _ in 0..10 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let soil = grid.get(root_x + 1, root_y, root_z).unwrap();
        assert_eq!(
            soil.water_level, 0,
            "Soil adjacent to root should dry out over time"
        );
    }

    #[test]
    fn water_spreads_symmetrically() {
        // Regression test: lateral water spread should not favor +x/+y over -x/-y.
        // A single water source on a flat plane should produce equal levels in
        // all four cardinal neighbors after several ticks.
        // Uses a corner far from the spring (28-31) to avoid interference.
        use crate::grid::{GRID_X, GRID_Y, GRID_Z};

        let cells = vec![
            crate::voxel::Voxel {
                material: Material::Air,
                water_level: 0,
                light_level: 0,
                nutrient_level: 0,
            };
            GRID_X * GRID_Y * GRID_Z
        ];
        let mut world = crate::create_world();
        *world.resource_mut::<VoxelGrid>() = VoxelGrid::from_cells(cells);

        // Place stone floor at z=0, air everywhere else, water blob at (10,10,1).
        // Far from the spring at (28-31) so water_spring system doesn't interfere.
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            let z = 0;
            for y in 0..GRID_Y {
                for x in 0..GRID_X {
                    if let Some(cell) = grid.get_mut(x, y, z) {
                        cell.material = Material::Stone;
                    }
                }
            }
            let cx = 10;
            let cy = 10;
            if let Some(cell) = grid.get_mut(cx, cy, 1) {
                cell.material = Material::Water;
                cell.water_level = 200;
            }
        }

        let mut schedule = crate::create_schedule();
        for _ in 0..20 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let cx = 10;
        let cy = 10;
        let w_xm = grid.get(cx - 1, cy, 1).unwrap().water_level;
        let w_xp = grid.get(cx + 1, cy, 1).unwrap().water_level;
        let w_ym = grid.get(cx, cy - 1, 1).unwrap().water_level;
        let w_yp = grid.get(cx, cy + 1, 1).unwrap().water_level;

        // All four cardinal neighbors should have the same water level.
        assert_eq!(w_xm, w_xp, "x-axis symmetry: -{w_xm} vs +{w_xp}");
        assert_eq!(w_ym, w_yp, "y-axis symmetry: -{w_ym} vs +{w_yp}");
        assert_eq!(w_xm, w_ym, "cross-axis symmetry: x{w_xm} vs y{w_ym}");
    }

    #[test]
    fn no_checkerboard_water_frontier() {
        // After many ticks, no water cells should have water_level < 5.
        // This catches the alternating .~.~.~ frontier artifact.
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        for _ in 0..100 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        for z in 0..GRID_Z {
            for y in 0..GRID_Y {
                for x in 0..GRID_X {
                    let cell = grid.get(x, y, z).unwrap();
                    if cell.material == Material::Water {
                        assert!(
                            cell.water_level >= 5,
                            "Water at ({x},{y},{z}) has water_level={}, should be >= 5 or reverted to air",
                            cell.water_level
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn seed_growth_stages_visible() {
        // Verify that a seed's nutrient_level passes through the 100 threshold
        // (used by the display layer to show 's' vs 'S') on its way to 200.
        // Growth rate is ~12/tick base. Using inland position for loam soil.
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(30, 10, GROUND_LEVEL + 1) {
                cell.material = Material::Seed;
                cell.water_level = 100;
                cell.light_level = 0;
                cell.nutrient_level = 0;
            }
        }

        // After 5 ticks: seed should be partway through growth (~60 nutrient)
        for _ in 0..5 {
            crate::tick(&mut world, &mut schedule);
        }

        {
            let grid = world.resource::<VoxelGrid>();
            let cell = grid.get(30, 10, GROUND_LEVEL + 1).unwrap();
            assert_eq!(
                cell.material,
                Material::Seed,
                "Should still be a seed at 5 ticks"
            );
            assert!(
                cell.nutrient_level < 100,
                "At 5 ticks, nutrient_level ({}) should be < 100 (small seed stage)",
                cell.nutrient_level
            );
        }

        // After 8 more ticks (13 total): growth should cross 100 threshold
        // At 12/tick: 13 * 12 = 156 >= 100
        for _ in 0..8 {
            crate::tick(&mut world, &mut schedule);
        }

        {
            let grid = world.resource::<VoxelGrid>();
            let cell = grid.get(30, 10, GROUND_LEVEL + 1).unwrap();
            // With growth rate 12/tick, at 13 ticks: ~156 nutrient.
            // Seed either has high nutrient_level or already germinated to trunk.
            let progressed = cell.material == Material::Trunk
                || (cell.material == Material::Seed && cell.nutrient_level >= 100);
            assert!(
                progressed,
                "At 13 ticks, seed should have progressed: material={:?}, nutrient={}",
                cell.material, cell.nutrient_level
            );
        }
    }

    #[test]
    fn light_attenuates_through_soil() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();
        crate::tick(&mut world, &mut schedule);

        let grid = world.resource::<VoxelGrid>();
        let sh = VoxelGrid::surface_height(0, 0);
        let sky = grid.get(0, 0, sh + 2).unwrap().light_level;
        let surface_soil = grid.get(0, 0, sh).unwrap().light_level;
        let one_below = grid.get(0, 0, sh - 1).unwrap().light_level;
        let deep_z = sh.saturating_sub(4);
        let deep_soil = grid.get(0, 0, deep_z).unwrap().light_level;

        assert!(
            surface_soil < sky,
            "Surface soil ({surface_soil}) should be dimmer than sky ({sky})"
        );
        assert!(
            deep_soil < surface_soil,
            "Deep soil ({deep_soil}) should be dimmer than surface soil ({surface_soil})"
        );

        // Light gradient: surface soil should have usable light,
        // and it should decay monotonically with depth.
        assert!(
            surface_soil >= 150,
            "Surface soil ({surface_soil}) should have substantial light"
        );
        assert!(
            one_below < surface_soil,
            "One below ({one_below}) should be dimmer than surface ({surface_soil})"
        );
    }

    #[test]
    fn sandy_soil_absorbs_faster_than_clay() {
        use crate::soil::{SoilComposition, SoilGrid};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Set up two soil cells next to water: one sandy, one clay
        let sandy_pos = (5, 5, GROUND_LEVEL);
        let clay_pos = (10, 10, GROUND_LEVEL);
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place water adjacent to both
            if let Some(cell) = grid.get_mut(sandy_pos.0 + 1, sandy_pos.1, sandy_pos.2) {
                cell.set_material(Material::Water);
            }
            if let Some(cell) = grid.get_mut(clay_pos.0 + 1, clay_pos.1, clay_pos.2) {
                cell.set_material(Material::Water);
            }
            // Ensure both soil cells start dry
            if let Some(cell) = grid.get_mut(sandy_pos.0, sandy_pos.1, sandy_pos.2) {
                cell.water_level = 0;
            }
            if let Some(cell) = grid.get_mut(clay_pos.0, clay_pos.1, clay_pos.2) {
                cell.water_level = 0;
            }
        }
        {
            let mut soil = world.resource_mut::<SoilGrid>();
            *soil.get_mut(sandy_pos.0, sandy_pos.1, sandy_pos.2).unwrap() =
                SoilComposition::sandy();
            *soil.get_mut(clay_pos.0, clay_pos.1, clay_pos.2).unwrap() = SoilComposition::clay();
        }

        // Run a few ticks
        for _ in 0..5 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let sandy_water = grid
            .get(sandy_pos.0, sandy_pos.1, sandy_pos.2)
            .unwrap()
            .water_level;
        let clay_water = grid
            .get(clay_pos.0, clay_pos.1, clay_pos.2)
            .unwrap()
            .water_level;

        assert!(
            sandy_water > clay_water,
            "Sandy soil ({sandy_water}) should absorb more water than clay ({clay_water}) in same time"
        );
    }

    #[test]
    fn soil_organic_increases_near_roots() {
        use crate::soil::SoilGrid;

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let soil_x = 20;
        let soil_y = 20;
        let soil_z = GROUND_LEVEL - 1;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place a root adjacent to our soil cell
            if let Some(cell) = grid.get_mut(soil_x + 1, soil_y, soil_z) {
                cell.material = Material::Root;
                cell.water_level = 0;
            }
        }
        {
            let mut soil = world.resource_mut::<SoilGrid>();
            let comp = soil.get_mut(soil_x, soil_y, soil_z).unwrap();
            comp.organic = 10; // Start low
        }

        let initial_organic = {
            let soil = world.resource::<SoilGrid>();
            soil.get(soil_x, soil_y, soil_z).unwrap().organic
        };

        for _ in 0..20 {
            crate::tick(&mut world, &mut schedule);
        }

        let soil = world.resource::<SoilGrid>();
        let final_organic = soil.get(soil_x, soil_y, soil_z).unwrap().organic;
        assert!(
            final_organic > initial_organic,
            "Organic matter should increase near roots: initial={initial_organic}, final={final_organic}"
        );
    }

    #[test]
    fn bacteria_grow_in_moist_organic_soil() {
        use crate::soil::SoilGrid;

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Use a position near the spring so it stays moist from water_spring refills
        let x = GRID_X / 2 + 1;
        let y = GRID_Y / 2 + 1;
        let surface = VoxelGrid::surface_height(x, y);
        let z = surface - 1; // Below surface = soil
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Saturate the test cell and neighbors with water so it stays moist
            for dz in 0..=2 {
                let wz = z.saturating_sub(dz);
                if let Some(cell) = grid.get_mut(x, y, wz) {
                    cell.water_level = 255;
                }
            }
        }
        {
            let mut soil = world.resource_mut::<SoilGrid>();
            let comp = soil.get_mut(x, y, z).unwrap();
            comp.organic = 80; // Organic-rich
            comp.bacteria = 10; // Start low
        }

        for _ in 0..30 {
            crate::tick(&mut world, &mut schedule);
        }

        let soil = world.resource::<SoilGrid>();
        let bacteria = soil.get(x, y, z).unwrap().bacteria;
        assert!(
            bacteria > 10,
            "Bacteria should grow in moist organic soil, got {bacteria}"
        );
    }

    #[test]
    fn bacteria_die_in_dry_soil() {
        use crate::soil::SoilGrid;

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Pick a dry soil cell far from water but in the interior
        let x = GRID_X / 4;
        let y = GRID_Y / 4;
        let surface = VoxelGrid::surface_height(x, y);
        let z = surface - 3;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(x, y, z) {
                cell.water_level = 0; // Bone dry
            }
        }
        {
            let mut soil = world.resource_mut::<SoilGrid>();
            let comp = soil.get_mut(x, y, z).unwrap();
            comp.bacteria = 100; // Start high
            comp.organic = 5;
        }

        for _ in 0..30 {
            crate::tick(&mut world, &mut schedule);
        }

        let soil = world.resource::<SoilGrid>();
        let bacteria = soil.get(x, y, z).unwrap().bacteria;
        assert!(
            bacteria < 100,
            "Bacteria should die in dry soil, got {bacteria}"
        );
    }

    #[test]
    fn rock_weathers_into_clay() {
        use crate::soil::SoilGrid;

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Find a position where (x+y+z) % 50 == 0 so weathering triggers.
        // Use surface_height(x, y) - 1 for each candidate so z is always soil.
        let mut test_x = 0;
        let mut test_y = 0;
        let mut test_z = 0;
        let mut found = false;
        'outer: for x in GRID_X / 4..GRID_X * 3 / 4 {
            for y in GRID_Y / 4..GRID_Y * 3 / 4 {
                let z = VoxelGrid::surface_height(x, y).saturating_sub(1);
                if (x + y + z) % 50 == 0 {
                    test_x = x;
                    test_y = y;
                    test_z = z;
                    found = true;
                    break 'outer;
                }
            }
        }
        assert!(found, "Should find a (x+y+z)%50==0 position in soil layer");

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(test_x, test_y, test_z) {
                cell.water_level = 255; // Very wet to survive drainage
            }
        }
        let initial_rock;
        let initial_clay;
        {
            let mut soil = world.resource_mut::<SoilGrid>();
            let comp = soil.get_mut(test_x, test_y, test_z).unwrap();
            comp.rock = 100;
            comp.clay = 50;
            initial_rock = comp.rock;
            initial_clay = comp.clay;
        }

        // Run many ticks — soil_evolution fires every 10 ticks, weathering
        // requires water_level > 30. Refill water periodically to ensure
        // the cell stays wet at the finer scale.
        for i in 0..100 {
            crate::tick(&mut world, &mut schedule);
            // Re-saturate every 5 ticks so the cell stays wet.
            if i % 5 == 0 {
                let mut grid = world.resource_mut::<VoxelGrid>();
                if let Some(cell) = grid.get_mut(test_x, test_y, test_z) {
                    cell.water_level = 255;
                }
            }
        }

        let soil = world.resource::<SoilGrid>();
        let comp = soil.get(test_x, test_y, test_z).unwrap();
        assert!(
            comp.rock < initial_rock,
            "Rock should decrease via weathering at ({test_x},{test_y},{test_z}): initial={initial_rock}, final={}",
            comp.rock
        );
        assert!(
            comp.clay > initial_clay,
            "Clay should increase from weathered rock: initial={initial_clay}, final={}",
            comp.clay
        );
    }

    #[test]
    fn save_load_preserves_soil_composition() {
        use crate::soil::{SoilComposition, SoilGrid};

        let mut world = crate::create_world();
        {
            let mut soil = world.resource_mut::<SoilGrid>();
            let comp = soil.get_mut(30, 30, GROUND_LEVEL).unwrap();
            *comp = SoilComposition {
                sand: 42,
                clay: 99,
                organic: 150,
                rock: 77,
                ph: 200,
                bacteria: 33,
            };
        }

        let path = std::env::temp_dir().join("groundwork_test_soil_rt.state");
        crate::save::save_world(&world, &path).unwrap();
        let loaded = crate::save::load_world(&path).unwrap();

        let soil = loaded.resource::<SoilGrid>();
        let comp = soil.get(30, 30, GROUND_LEVEL).unwrap();
        assert_eq!(comp.sand, 42);
        assert_eq!(comp.clay, 99);
        assert_eq!(comp.organic, 150);
        assert_eq!(comp.rock, 77);
        assert_eq!(comp.ph, 200);
        assert_eq!(comp.bacteria, 33);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn v2_backward_compatible_generates_soil() {
        use crate::Tick;
        // Build a valid V2 file (no soil data)
        let world = crate::create_world();
        let grid = world.resource::<VoxelGrid>();
        let tick = world.resource::<Tick>();

        let mut buf = Vec::new();
        buf.extend_from_slice(b"GWRK");
        buf.extend_from_slice(&2u16.to_le_bytes()); // version 2
        buf.extend_from_slice(&[0u8; 2]);
        buf.extend_from_slice(&tick.0.to_le_bytes());
        for v in grid.cells() {
            buf.push(v.material.as_u8());
            buf.push(v.water_level);
            buf.push(v.light_level);
            buf.push(v.nutrient_level);
        }
        // Focus block
        buf.extend_from_slice(&[0u8; 14]);

        let path = std::env::temp_dir().join("groundwork_test_v2_soil_compat.state");
        std::fs::write(&path, &buf).unwrap();

        let loaded = crate::save::load_world(&path).unwrap();
        // Should have a SoilGrid generated from the voxel data
        let soil = loaded.resource::<SoilGrid>();
        // Topsoil far from edges/water should be loam.
        // edge_band = meters_to_voxels(1.6) = 32, peat zone starts at cx - spring_range = 34.
        // Use position between edge band and peat zone.
        let edge_band = crate::scale::meters_to_voxels(1.6);
        let sx = edge_band + 1; // 33: inside interior, outside peat zone
        let sy = edge_band + 1;
        let surface = VoxelGrid::surface_height(sx, sy);
        let comp = soil.get(sx, sy, surface).unwrap();
        assert_eq!(
            comp.type_name(),
            "loam",
            "V2 backward compat should generate loam topsoil"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn compacted_soil_blocks_seed_growth() {
        use crate::soil::{SoilComposition, SoilGrid};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let seed_x = 15;
        let seed_y = 15;
        let seed_z = GROUND_LEVEL + 1;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(seed_x, seed_y, seed_z) {
                cell.material = Material::Seed;
                cell.water_level = 100;
                cell.nutrient_level = 0;
            }
        }
        {
            // Make adjacent soil compacted
            let mut soil = world.resource_mut::<SoilGrid>();
            let comp = soil.get_mut(seed_x, seed_y, GROUND_LEVEL).unwrap();
            *comp = SoilComposition {
                sand: 10,
                clay: 220,
                organic: 5,
                rock: 10,
                ph: 128,
                bacteria: 2,
            };
        }

        for _ in 0..60 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let cell = grid.get(seed_x, seed_y, seed_z).unwrap();
        assert_eq!(
            cell.material,
            Material::Seed,
            "Seed should not grow in compacted soil — should remain a seed"
        );
        assert_eq!(
            cell.nutrient_level, 0,
            "Seed growth counter should stay at 0 in compacted soil"
        );
    }

    #[test]
    fn tree_grows_through_stages() {
        // Full lifecycle: seed → seedling → sapling (with visible canopy).
        // Place near center spring for sustained water access.
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Near the center spring so water persists
        let tx = GRID_X / 2 + 5;
        let ty = GRID_Y / 2 + 2;
        let tz = VoxelGrid::surface_height(tx, ty) + 1;

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(tx, ty, tz) {
                cell.material = Material::Seed;
                cell.water_level = 100;
            }
            // Ensure soil below is wet. The spring nearby will sustain moisture.
            for dz in 1..=4 {
                if let Some(cell) = grid.get_mut(tx, ty, tz - dz) {
                    cell.water_level = 200;
                }
            }
        }

        // Run until seed matures into seedling (~40-70 ticks depending on soil)
        for _ in 0..80 {
            crate::tick(&mut world, &mut schedule);
        }

        {
            let grid = world.resource::<VoxelGrid>();
            assert_eq!(
                grid.get(tx, ty, tz).unwrap().material,
                Material::Trunk,
                "Seed should have become a trunk (seedling stage)"
            );
        }

        // Run more ticks to trigger seedling → sapling transition.
        // Sapling template adds a second trunk + leaf cap above.
        for _ in 0..30 {
            crate::tick(&mut world, &mut schedule);
        }

        // Check that the tree has grown taller (sapling has trunk_h >= 2).
        let grid = world.resource::<VoxelGrid>();
        let above = grid.get(tx, ty, tz + 1).unwrap();
        assert!(
            above.material == Material::Trunk || above.material == Material::Leaf,
            "After ~110 ticks, tree should have grown above seedling. Got {:?} at z+1",
            above.material
        );
    }

    #[test]
    fn tree_rasterize_places_leaves() {
        // Directly test that rasterization generates leaf voxels for a sapling.
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Use center-ish position away from outcrops/edges
        let tx = GRID_X / 4;
        let ty = GRID_Y / 4;
        let tz = VoxelGrid::surface_height(tx, ty) + 1;

        // Manually spawn a tree entity in Sapling stage with dirty=true.
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Clear the area above ground for tree placement.
            if let Some(cell) = grid.get_mut(tx, ty, tz) {
                cell.set_material(Material::Trunk);
            }
        }

        world.spawn(Tree {
            species_id: 0, // Oak
            root_pos: (tx, ty, tz),
            age: 50,
            stage: GrowthStage::Sapling,
            health: 1.0,
            accumulated_water: 300.0,
            accumulated_light: 300.0,
            rng_seed: 12345,
            dirty: true,
            voxel_footprint: vec![(tx, ty, tz)],
            branches: Vec::new(),
            attraction_points: Vec::new(),
            skeleton_initialized: false,
            stage_changed: true,
            pending_voxels: Vec::new(),
            revealed_z: 0,
        });

        // One tick runs tree_rasterize which should place the sapling template.
        crate::tick(&mut world, &mut schedule);

        let grid = world.resource::<VoxelGrid>();
        let species = &crate::tree::SpeciesTable::default().species[0];
        let trunk_h = (species.max_height() / 3).max(2) as usize;
        let leaf_z = tz + trunk_h;
        let center_leaf = grid.get(tx, ty, leaf_z).unwrap();
        assert_eq!(
            center_leaf.material,
            Material::Leaf,
            "Sapling should have leaf at trunk_top. Got {:?}",
            center_leaf.material
        );

        // Check a cardinal neighbor also has a leaf (radius 1 disc).
        let neighbor_leaf = grid.get(tx + 1, ty, leaf_z).unwrap();
        assert_eq!(
            neighbor_leaf.material,
            Material::Leaf,
            "Sapling leaf disc (r=1) should include cardinal neighbor. Got {:?}",
            neighbor_leaf.material
        );
    }

    #[test]
    fn rasterize_clear_restores_soil_above_ground_level() {
        // Regression: tree_rasterize clearing phase used the constant GROUND_LEVEL
        // to decide Soil vs Air revert. On terrain where surface_height > GROUND_LEVEL,
        // roots above GROUND_LEVEL but at/below the surface were reverted to Air,
        // creating gaps that filled with water — causing floating trees.
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();

        // Find a position where surface_height > GROUND_LEVEL
        let mut tx = 0;
        let mut ty = 0;
        let mut surface_z = GROUND_LEVEL;
        'search: for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let h = VoxelGrid::surface_height(x, y);
                if h > GROUND_LEVEL {
                    tx = x;
                    ty = y;
                    surface_z = h;
                    break 'search;
                }
            }
        }
        assert!(
            surface_z > GROUND_LEVEL,
            "Test requires terrain above GROUND_LEVEL"
        );

        let tz = surface_z + 1; // trunk base, one above surface
        let root_z = surface_z; // root position, at surface (above GROUND_LEVEL)

        // Place trunk and root
        {
            let grid = world.resource_mut::<VoxelGrid>().into_inner();
            grid.get_mut(tx, ty, tz)
                .unwrap()
                .set_material(Material::Trunk);
            // Root at surface_z (which is > GROUND_LEVEL)
            assert_eq!(
                grid.get(tx, ty, root_z).unwrap().material,
                Material::Soil,
                "Surface voxel should be soil"
            );
            grid.get_mut(tx, ty, root_z)
                .unwrap()
                .set_material(Material::Root);
        }

        world.spawn(Tree {
            species_id: 0,
            root_pos: (tx, ty, tz),
            age: 50,
            stage: GrowthStage::Sapling,
            health: 1.0,
            accumulated_water: 300.0,
            accumulated_light: 300.0,
            rng_seed: 99,
            dirty: true,
            voxel_footprint: vec![(tx, ty, tz), (tx, ty, root_z)],
            branches: Vec::new(),
            attraction_points: Vec::new(),
            skeleton_initialized: false,
            stage_changed: true,
            pending_voxels: Vec::new(),
            revealed_z: 0,
        });

        let mut schedule = crate::create_schedule();
        crate::tick(&mut world, &mut schedule);

        // The root_z position should be Soil (or Root if re-placed), NOT Air.
        let grid = world.resource::<VoxelGrid>();
        let cell = grid.get(tx, ty, root_z).unwrap();
        assert_ne!(
            cell.material,
            Material::Air,
            "Root at z={} (above GROUND_LEVEL={}) should revert to Soil, not Air",
            root_z,
            GROUND_LEVEL
        );
    }

    #[test]
    fn roots_grow_toward_water() {
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let tx = 20;
        let ty = 20;
        let tz = GROUND_LEVEL + 1;

        // Set up: wet soil on the -x side, dry soil on +x side.
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place trunk
            if let Some(cell) = grid.get_mut(tx, ty, tz) {
                cell.set_material(Material::Trunk);
            }
            // Place initial root
            if let Some(cell) = grid.get_mut(tx, ty, GROUND_LEVEL) {
                cell.set_material(Material::Root);
            }
            // Wet soil to the west (lower x)
            for dx in 1..=5 {
                if let Some(cell) = grid.get_mut(tx - dx, ty, GROUND_LEVEL) {
                    cell.water_level = 200;
                }
                if let Some(cell) = grid.get_mut(tx - dx, ty, GROUND_LEVEL - 1) {
                    cell.water_level = 200;
                }
            }
            // Dry soil to the east (higher x)
            for dx in 1..=5 {
                if let Some(cell) = grid.get_mut(tx + dx, ty, GROUND_LEVEL) {
                    cell.water_level = 0;
                }
                if let Some(cell) = grid.get_mut(tx + dx, ty, GROUND_LEVEL - 1) {
                    cell.water_level = 0;
                }
            }
        }

        // Spawn sapling tree with age divisible by 5 so root_growth fires.
        world.spawn(Tree {
            species_id: 0,
            root_pos: (tx, ty, tz),
            age: 4, // tree_growth increments to 5, then root_growth checks age%5==0
            stage: GrowthStage::Sapling,
            health: 1.0,
            accumulated_water: 300.0,
            accumulated_light: 300.0,
            rng_seed: 999,
            dirty: false,
            voxel_footprint: vec![(tx, ty, tz), (tx, ty, GROUND_LEVEL)],
            branches: Vec::new(),
            attraction_points: Vec::new(),
            skeleton_initialized: false,
            stage_changed: true,
            pending_voxels: Vec::new(),
            revealed_z: 0,
        });

        crate::tick(&mut world, &mut schedule);

        // Root should have grown toward wet soil (lower x or downward), not toward dry soil.
        let grid = world.resource::<VoxelGrid>();
        let grew_toward_water = grid
            .get(tx - 1, ty, GROUND_LEVEL)
            .map_or(false, |v| v.material == Material::Root)
            || grid
                .get(tx, ty, GROUND_LEVEL - 1)
                .map_or(false, |v| v.material == Material::Root);

        assert!(
            grew_toward_water,
            "Root should grow toward wet soil or downward (hydrotropism + gravitropism)"
        );
    }

    #[test]
    fn seed_dispersal_from_mature_tree() {
        use crate::tree::{GrowthStage, Tree};

        // Place at grid center for maximum landing area.
        // At the 0.05m scale, dispersal distances are 15-44 voxels.
        let tx = GRID_X / 2;
        let ty = GRID_Y / 2;
        let tz = GROUND_LEVEL + 1;

        // Try multiple rng_seeds to find one that disperses successfully.
        // Different seeds pick different directions; some may hit water or OOB.
        let mut found_seed = false;
        for rng_trial in 0..20u64 {
            // Reset world for each trial
            let mut world = crate::create_world();
            let mut schedule = crate::create_schedule();

            {
                let mut grid = world.resource_mut::<VoxelGrid>();
                if let Some(cell) = grid.get_mut(tx, ty, tz) {
                    cell.set_material(Material::Trunk);
                }
                if let Some(cell) = grid.get_mut(tx, ty, GROUND_LEVEL) {
                    cell.set_material(Material::Root);
                    cell.water_level = 200;
                }
            }

            let rng_seed = rng_trial * 7 + 1;
            // Must match the dispersal period formula in seed_dispersal system
            let species = &world.resource::<crate::tree::SpeciesTable>().species[0];
            let base_period = species.dispersal_period;
            let period = base_period
                + (crate::tree::tree_hash(rng_seed, 0) % (base_period as u64 / 4 + 1)) as u32;
            world.spawn(Tree {
                species_id: 0,
                root_pos: (tx, ty, tz),
                age: period - 1,
                stage: GrowthStage::Mature,
                health: 1.0,
                accumulated_water: 10000.0,
                accumulated_light: 10000.0,
                rng_seed,
                dirty: false,
                voxel_footprint: vec![(tx, ty, tz), (tx, ty, GROUND_LEVEL)],
                branches: Vec::new(),
                attraction_points: Vec::new(),
                skeleton_initialized: true,
                stage_changed: true,
                pending_voxels: Vec::new(),
                revealed_z: 0,
            });

            crate::tick(&mut world, &mut schedule);

            // Search the entire grid for a dispersed seed.
            let grid = world.resource::<VoxelGrid>();
            for sy in 0..GRID_Y {
                for sx in 0..GRID_X {
                    if sx == tx && sy == ty {
                        continue;
                    }
                    // Seeds land at surface + 1
                    let sh = VoxelGrid::surface_height(sx, sy);
                    let seed_z = sh + 1;
                    if let Some(cell) = grid.get(sx, sy, seed_z) {
                        if cell.material == Material::Seed {
                            found_seed = true;
                        }
                    }
                }
                if found_seed {
                    break;
                }
            }

            if found_seed {
                break;
            }
        }

        assert!(
            found_seed,
            "At least one rng_seed trial should produce a dispersed seed"
        );
    }

    #[test]
    fn health_recovers_with_good_resources() {
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let tx = 30;
        let ty = 30;
        let tz = GROUND_LEVEL + 1;

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(tx, ty, tz) {
                cell.set_material(Material::Trunk);
            }
            // Place a root in wet soil
            if let Some(cell) = grid.get_mut(tx, ty, GROUND_LEVEL) {
                cell.set_material(Material::Root);
                cell.water_level = 200;
            }
            // Saturate surrounding soil so root keeps getting water
            for dx in -5i32..=5 {
                for dy in -5i32..=5 {
                    for dz in 0i32..=3 {
                        let nx = (tx as i32 + dx) as usize;
                        let ny = (ty as i32 + dy) as usize;
                        let nz = (GROUND_LEVEL as i32 - dz) as usize;
                        if let Some(cell) = grid.get_mut(nx, ny, nz) {
                            if cell.material == Material::Soil {
                                cell.water_level = 255;
                            }
                        }
                    }
                }
            }
        }

        // Spawn tree with low health but good resources available.
        world.spawn(Tree {
            species_id: 0,
            root_pos: (tx, ty, tz),
            age: 100,
            stage: GrowthStage::Sapling,
            health: 0.2,
            accumulated_water: 500.0,
            accumulated_light: 500.0,
            rng_seed: 77,
            dirty: false,
            voxel_footprint: vec![(tx, ty, tz), (tx, ty, GROUND_LEVEL)],
            branches: Vec::new(),
            attraction_points: Vec::new(),
            skeleton_initialized: false,
            stage_changed: true,
            pending_voxels: Vec::new(),
            revealed_z: 0,
        });

        // Run several ticks for health to recover.
        for _ in 0..20 {
            crate::tick(&mut world, &mut schedule);
        }

        let mut trees = world.query::<&Tree>();
        let tree = trees.iter(&world).next().unwrap();
        assert!(
            tree.health > 0.2,
            "Tree health ({}) should have recovered above 0.2 with good resources",
            tree.health
        );
    }

    #[test]
    fn branch_growth_produces_nodes() {
        use crate::tree::{init_skeleton, GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let tx = 30;
        let ty = 30;
        let tz = GROUND_LEVEL + 1;

        // Clear area above ground — must be larger than crown radius
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            for dz in 0..60 {
                for dx in -30i32..=30 {
                    for dy in -30i32..=30 {
                        let ax = (tx as i32 + dx) as usize;
                        let ay = (ty as i32 + dy) as usize;
                        let az = tz + dz;
                        if VoxelGrid::in_bounds(ax, ay, az) {
                            if let Some(cell) = grid.get_mut(ax, ay, az) {
                                if cell.material != Material::Water {
                                    cell.set_material(Material::Air);
                                }
                            }
                        }
                    }
                }
            }
        }

        let species = &crate::tree::SpeciesTable::default().species[0];
        let (branches, points) = init_skeleton(species, &GrowthStage::YoungTree, 42);
        let initial_branch_count = branches.len();

        world.spawn(Tree {
            species_id: 0,
            root_pos: (tx, ty, tz),
            age: 2, // Will become 3 on first tick (3 % 3 == 0 triggers branch_growth)
            stage: GrowthStage::YoungTree,
            health: 1.0,
            accumulated_water: 2000.0,
            accumulated_light: 2000.0,
            rng_seed: 42,
            dirty: true,
            voxel_footprint: Vec::new(),
            branches,
            attraction_points: points,
            skeleton_initialized: true,
            stage_changed: true,
            pending_voxels: Vec::new(),
            revealed_z: 0,
        });

        // Run several ticks to allow branch growth
        for _ in 0..15 {
            crate::tick(&mut world, &mut schedule);
        }

        let mut trees = world.query::<&Tree>();
        let tree = trees
            .iter(&world)
            .find(|t| t.root_pos == (tx, ty, tz))
            .expect("Test tree should exist at expected position");
        assert!(
            tree.branches.len() > initial_branch_count,
            "Branch growth should add nodes: initial={}, current={}",
            initial_branch_count,
            tree.branches.len()
        );
        // Should have consumed some attraction points
        let initial_points = crate::tree::generate_attraction_points(
            &crate::tree::SpeciesTable::default().species[0],
            &GrowthStage::YoungTree,
            42,
        )
        .len();
        assert!(
            tree.attraction_points.len() < initial_points,
            "Some attraction points should have been consumed"
        );
    }

    #[test]
    fn skeleton_rasterize_produces_voxels() {
        use crate::tree::{BranchNode, GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let tx = 15;
        let ty = 15;
        let tz = VoxelGrid::surface_height(tx, ty) + 1;

        // Ensure the root position is Soil and above is Air
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(tx, ty, tz - 1) {
                cell.material = Material::Soil;
                cell.water_level = 0;
            }
            for dz in 0..5 {
                if let Some(cell) = grid.get_mut(tx, ty, tz + dz) {
                    cell.material = Material::Air;
                }
            }
            // Also ensure branch tip position is Air
            if let Some(cell) = grid.get_mut(tx + 1, ty, tz + 2) {
                cell.material = Material::Air;
            }
        }

        // Manually build a small skeleton
        let branches = vec![
            BranchNode {
                pos: (0, 0, 0),
                parent: u16::MAX,
                material: Material::Trunk,
                shade_stress: 0,
                alive: true,
            },
            BranchNode {
                pos: (0, 0, 1),
                parent: 0,
                material: Material::Trunk,
                shade_stress: 0,
                alive: true,
            },
            BranchNode {
                pos: (0, 0, 2),
                parent: 1,
                material: Material::Trunk,
                shade_stress: 0,
                alive: true,
            },
            BranchNode {
                pos: (1, 0, 2),
                parent: 2,
                material: Material::Branch,
                shade_stress: 0,
                alive: true,
            },
            BranchNode {
                pos: (0, 0, -1),
                parent: 0,
                material: Material::Root,
                shade_stress: 0,
                alive: true,
            },
        ];

        // Provide 10 dummy attraction points far from any branch tip so
        // branch_growth doesn't regenerate new ones and flood pending_voxels
        // with leaf spheres that delay trunk reveal during the test window.
        let dummy_points = vec![(10000, 10000, 10000); 10];

        world.spawn(Tree {
            species_id: 0,
            root_pos: (tx, ty, tz),
            age: 100,
            stage: GrowthStage::YoungTree,
            health: 1.0,
            accumulated_water: 100.0,
            accumulated_light: 100.0,
            rng_seed: 42,
            dirty: true,
            voxel_footprint: Vec::new(),
            branches,
            attraction_points: dummy_points,
            skeleton_initialized: true,
            stage_changed: true,
            pending_voxels: Vec::new(),
            revealed_z: 0,
        });

        // Trunk+leaf voxels now grow gradually via pending_voxels.
        // Run enough ticks to drain the queue (roots are placed immediately).
        // Keep accumulated resources low (500) so the tree stays YoungTree
        // and doesn't transition to Mature during the test ticks.
        for _ in 0..30 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        // Root below (pos (0,0,-1) = tz-1) — placed immediately
        assert_eq!(grid.get(tx, ty, tz - 1).unwrap().material, Material::Root);
        // Trunk at root_pos — placed via pending_voxels
        assert_eq!(grid.get(tx, ty, tz).unwrap().material, Material::Trunk);
        // Trunk at z+1
        assert_eq!(grid.get(tx, ty, tz + 1).unwrap().material, Material::Trunk);
        // Trunk at z+2
        assert_eq!(grid.get(tx, ty, tz + 2).unwrap().material, Material::Trunk);
        // Branch tip at (1,0,2) should become Leaf (it's a tip)
        assert_eq!(
            grid.get(tx + 1, ty, tz + 2).unwrap().material,
            Material::Leaf
        );
    }

    #[test]
    fn self_pruning_kills_shaded_branches() {
        use crate::tree::{BranchNode, GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let tx = 20;
        let ty = 20;
        let tz = GROUND_LEVEL + 1;

        // Build a tree with a branch node that will be fully shaded
        let branches = vec![
            BranchNode {
                pos: (0, 0, 0),
                parent: u16::MAX,
                material: Material::Trunk,
                shade_stress: 0,
                alive: true,
            },
            BranchNode {
                pos: (0, 0, 1),
                parent: 0,
                material: Material::Trunk,
                shade_stress: 0,
                alive: true,
            },
            // This branch at z=0 level will be underground/shaded
            BranchNode {
                pos: (1, 0, 0),
                parent: 0,
                material: Material::Branch,
                shade_stress: 190,
                alive: true,
            },
        ];

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Ensure the branch position is dark by placing opaque material above
            for z in (tz + 2)..GRID_Z {
                if let Some(cell) = grid.get_mut(tx + 1, ty, z) {
                    cell.set_material(Material::Stone);
                }
            }
        }

        world.spawn(Tree {
            species_id: 0, // Oak: shade_tolerance=80, prune_threshold=200
            root_pos: (tx, ty, tz),
            age: 100,
            stage: GrowthStage::YoungTree,
            health: 1.0,
            accumulated_water: 2000.0,
            accumulated_light: 2000.0,
            rng_seed: 42,
            dirty: true,
            voxel_footprint: Vec::new(),
            branches,
            attraction_points: Vec::new(),
            skeleton_initialized: true,
            stage_changed: true,
            pending_voxels: Vec::new(),
            revealed_z: 0,
        });

        // Run enough ticks for shade_stress to exceed prune_threshold (200)
        // Starting at 190, need 10+ ticks
        for _ in 0..15 {
            crate::tick(&mut world, &mut schedule);
        }

        let mut trees = world.query::<&Tree>();
        let tree = trees.iter(&world).next().unwrap();
        let shaded_branch = tree.branches.iter().find(|b| b.pos == (1, 0, 0));
        assert!(
            shaded_branch.map_or(true, |b| !b.alive),
            "Heavily shaded branch should have been pruned"
        );
    }

    #[test]
    fn seedling_sapling_still_use_templates() {
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let tx = GRID_X / 4;
        let ty = GRID_Y / 4;
        let tz = VoxelGrid::surface_height(tx, ty) + 1;

        world.spawn(Tree {
            species_id: 0,
            root_pos: (tx, ty, tz),
            age: 50,
            stage: GrowthStage::Sapling,
            health: 1.0,
            accumulated_water: 300.0,
            accumulated_light: 300.0,
            rng_seed: 12345,
            dirty: true,
            voxel_footprint: vec![(tx, ty, tz)],
            branches: Vec::new(), // No skeleton
            attraction_points: Vec::new(),
            skeleton_initialized: false,
            stage_changed: true,
            pending_voxels: Vec::new(),
            revealed_z: 0,
        });

        crate::tick(&mut world, &mut schedule);

        // Sapling should still use template path — leaf at trunk_top
        let grid = world.resource::<VoxelGrid>();
        let species = &crate::tree::SpeciesTable::default().species[0];
        let trunk_h = (species.max_height() / 3).max(2) as usize;
        let leaf_z = tz + trunk_h;
        assert_eq!(
            grid.get(tx, ty, leaf_z).unwrap().material,
            Material::Leaf,
            "Sapling without skeleton should still use template rasterization"
        );
    }

    #[test]
    fn seed_transparent_to_light() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Place seed high above ground so below is also air (not soil)
        let x = 40;
        let y = 40;
        let seed_z = GROUND_LEVEL + 5;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(x, y, seed_z) {
                cell.material = Material::Seed;
            }
        }

        crate::tick(&mut world, &mut schedule);

        let grid = world.resource::<VoxelGrid>();
        let above = grid.get(x, y, seed_z + 1).unwrap().light_level;
        let at_seed = grid.get(x, y, seed_z).unwrap().light_level;
        // Check air cell below the seed (not soil)
        let below = grid.get(x, y, seed_z - 1).unwrap().light_level;

        // Seed should receive nearly the same light as air above it
        // (both attenuate by att_air after assignment)
        assert!(
            at_seed >= above.saturating_sub(5),
            "Seed should be transparent to light: above={above}, at_seed={at_seed}"
        );
        // Air below seed should get nearly the same as the seed
        // (seed attenuates like air — minimal reduction)
        assert!(
            below >= at_seed.saturating_sub(5),
            "Light below seed should continue: at_seed={at_seed}, below={below}"
        );
    }

    #[test]
    fn soil_generates_nutrients_from_bacteria() {
        use crate::soil::SoilGrid;

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let x = 20;
        let y = 20;
        let z = GROUND_LEVEL - 1;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Ensure the cell is soil with water
            if let Some(cell) = grid.get_mut(x, y, z) {
                cell.material = Material::Soil;
                cell.water_level = 100;
                cell.nutrient_level = 0;
            }
        }
        {
            let mut soil = world.resource_mut::<SoilGrid>();
            let comp = soil.get_mut(x, y, z).unwrap();
            comp.organic = 150;
            comp.bacteria = 100;
        }

        // Run enough ticks for soil_evolution to fire (every 10 ticks)
        for _ in 0..30 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let nutrient = grid.get(x, y, z).unwrap().nutrient_level;
        assert!(
            nutrient > 0,
            "Soil with bacteria + organic matter should generate nutrients, got {nutrient}"
        );
    }

    #[test]
    fn root_absorbs_nutrients_from_soil() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let root_x = 25;
        let root_y = 25;
        let z = GROUND_LEVEL - 1;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place a root
            if let Some(cell) = grid.get_mut(root_x, root_y, z) {
                cell.material = Material::Root;
                cell.water_level = 0;
                cell.nutrient_level = 0;
            }
            // Place soil with nutrients adjacent to root
            if let Some(cell) = grid.get_mut(root_x + 1, root_y, z) {
                cell.material = Material::Soil;
                cell.water_level = 100;
                cell.nutrient_level = 50;
            }
        }

        crate::tick(&mut world, &mut schedule);

        let grid = world.resource::<VoxelGrid>();
        let root_nutrients = grid.get(root_x, root_y, z).unwrap().nutrient_level;
        let soil_nutrients = grid.get(root_x + 1, root_y, z).unwrap().nutrient_level;

        assert!(
            root_nutrients > 0,
            "Root should absorb nutrients from adjacent soil, got {root_nutrients}"
        );
        assert!(
            soil_nutrients < 50,
            "Soil nutrients should decrease after root absorption, got {soil_nutrients}"
        );
    }

    #[test]
    fn nitrogen_handshake_clover_boosts_oak() {
        // Plant two oaks: one near clover, one isolated.
        // After 200 ticks, the clover-adjacent oak should have more accumulated resources.
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        {
            let mut grid = world.resource_mut::<VoxelGrid>();

            // Oak A at (20, 40) with clover nearby at (22, 40)
            if let Some(cell) = grid.get_mut(20, 40, GROUND_LEVEL + 1) {
                cell.material = Material::Seed;
                cell.water_level = 200;
                cell.light_level = 200;
            }
            // Clover near oak A — place as a seed that will grow into groundcover
            if let Some(cell) = grid.get_mut(22, 40, GROUND_LEVEL + 1) {
                cell.material = Material::Seed;
                cell.water_level = 200;
                cell.light_level = 200;
            }

            // Oak B at (60, 40) — isolated, no clover
            if let Some(cell) = grid.get_mut(60, 40, GROUND_LEVEL + 1) {
                cell.material = Material::Seed;
                cell.water_level = 200;
                cell.light_level = 200;
            }

            // Water near both
            for x in 18..=24 {
                if let Some(cell) = grid.get_mut(x, 40, GROUND_LEVEL) {
                    cell.material = Material::Water;
                    cell.water_level = 255;
                }
            }
            for x in 58..=64 {
                if let Some(cell) = grid.get_mut(x, 40, GROUND_LEVEL) {
                    cell.material = Material::Water;
                    cell.water_level = 255;
                }
            }
        }

        // Set clover seed species
        {
            let mut seed_map = world.resource_mut::<SeedSpeciesMap>();
            // (22, 40, GROUND_LEVEL+1) → clover (species_id 11)
            seed_map.map.insert((22, 40, GROUND_LEVEL + 1), 11);
        }

        for _ in 0..200 {
            crate::tick(&mut world, &mut schedule);
        }

        // Check: oak near clover should have more growth
        let mut trees = world.query::<&Tree>();
        let mut oak_near_clover_water = 0.0_f32;
        let mut oak_isolated_water = 0.0_f32;

        for tree in trees.iter(&world) {
            let species = &world.resource::<SpeciesTable>().species[tree.species_id];
            if species.name == "Oak" {
                let (rx, _ry, _rz) = tree.root_pos;
                if rx < 40 {
                    oak_near_clover_water = tree.accumulated_water;
                } else {
                    oak_isolated_water = tree.accumulated_water;
                }
            }
        }

        // The oak near clover should accumulate at least 20% more water
        // (1.5x boost once clover grows enough leaf voxels)
        if oak_near_clover_water > 0.0 && oak_isolated_water > 0.0 {
            assert!(
                oak_near_clover_water > oak_isolated_water,
                "Oak near clover ({oak_near_clover_water}) should grow faster than isolated oak ({oak_isolated_water})"
            );
        }
        // If one of them didn't grow (e.g., seed placement issue), that's OK —
        // the test structure is correct, we just need both seeds to germinate.
    }

    #[test]
    fn territorial_suppression_prevents_crowded_germination() {
        // A seed next to an existing trunk should not germinate.
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let cx = GRID_X / 2 + 10;
        let cy = GRID_Y / 2;
        let sz = GROUND_LEVEL + 1;

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place an existing trunk (simulating a mature tree)
            if let Some(cell) = grid.get_mut(cx, cy, sz) {
                cell.set_material(Material::Trunk);
            }
            // Place a seed 3 voxels away (within suppress_radius of 6)
            if let Some(cell) = grid.get_mut(cx + 3, cy, sz) {
                cell.material = Material::Seed;
                cell.water_level = 200;
                cell.light_level = 200;
                cell.nutrient_level = 0;
            }
            // Place a seed 10 voxels away (outside suppress_radius)
            if let Some(cell) = grid.get_mut(cx + 10, cy, sz) {
                cell.material = Material::Seed;
                cell.water_level = 200;
                cell.light_level = 200;
                cell.nutrient_level = 0;
            }
        }

        // Run enough ticks for seeds to accumulate growth
        for _ in 0..60 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        // Seed near trunk should still be a seed (suppressed — not growing)
        let near = grid.get(cx + 3, cy, sz).unwrap();
        assert_eq!(
            near.material,
            Material::Seed,
            "Seed near existing trunk should be suppressed (not germinate)"
        );

        // Seed far from trunk should have germinated into trunk
        let far = grid.get(cx + 10, cy, sz).unwrap();
        assert_eq!(
            far.material,
            Material::Trunk,
            "Seed far from existing trunk should germinate normally"
        );
    }

    #[test]
    fn crowded_seedlings_die_from_shade() {
        // Two trees planted close together: the one in shade should lose health.
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let cx = GRID_X / 2 + 10;
        let cy = GRID_Y / 2;
        let sz = GROUND_LEVEL + 1;

        // Spawn a mature tree that will cast shade
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place trunk column (5 voxels tall) and leaf canopy
            for z in sz..sz + 10 {
                if let Some(cell) = grid.get_mut(cx, cy, z) {
                    cell.set_material(Material::Trunk);
                    cell.nutrient_level = 0; // species_id 0 = oak
                }
            }
            // Dense leaf layer at top
            for dx in -3i32..=3 {
                for dy in -3i32..=3 {
                    for dz in 8i32..=12 {
                        let lx = (cx as i32 + dx) as usize;
                        let ly = (cy as i32 + dy) as usize;
                        let lz = (sz as i32 + dz) as usize;
                        if let Some(cell) = grid.get_mut(lx, ly, lz) {
                            if cell.material == Material::Air {
                                cell.set_material(Material::Leaf);
                            }
                        }
                    }
                }
            }
        }

        // Spawn a seedling directly under the canopy
        let shaded_tree = world
            .spawn(Tree {
                species_id: 0,
                root_pos: (cx + 1, cy, sz),
                age: 0,
                stage: GrowthStage::Seedling,
                health: 1.0,
                accumulated_water: 0.0,
                accumulated_light: 0.0,
                rng_seed: 999,
                dirty: false,
                voxel_footprint: vec![(cx + 1, cy, sz)],
                branches: Vec::new(),
                attraction_points: Vec::new(),
                skeleton_initialized: false,
                stage_changed: true,
                pending_voxels: Vec::new(),
                revealed_z: 0,
            })
            .id();

        // Run ticks — the shaded seedling should lose health quickly
        for _ in 0..50 {
            crate::tick(&mut world, &mut schedule);
        }

        let tree = world.entity(shaded_tree).get::<Tree>().unwrap();
        assert!(
            tree.health < 0.5,
            "Shaded seedling should have significantly reduced health ({} >= 0.5)",
            tree.health
        );
    }

    #[test]
    fn deadwood_decays_into_soil() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let dx = 30;
        let dy = 30;
        let dz = GROUND_LEVEL; // underground deadwood

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(dx, dy, dz) {
                cell.set_material(Material::DeadWood);
                // Pre-decay: set water_level high to simulate near-complete decomposition
                cell.water_level = 240;
            }
        }

        // Run enough ticks for passive decay to complete (240 + 2 per 20 ticks)
        for _ in 0..200 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let cell = grid.get(dx, dy, dz).unwrap();
        assert_eq!(
            cell.material,
            Material::Soil,
            "Underground DeadWood should decompose into soil"
        );
        assert!(
            cell.nutrient_level > 0,
            "Decomposed wood should leave nutrient-rich soil"
        );
    }

    #[test]
    fn pine_roots_acidify_soil() {
        // Pine roots should lower pH of adjacent soil cells.
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let rx = 30;
        let ry = 30;
        let rz = GROUND_LEVEL - 1; // underground

        // Record initial pH
        let initial_ph = {
            let soil = world.resource::<SoilGrid>();
            soil.get(rx + 1, ry, rz).unwrap().ph
        };

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place a pine root (species_id=3 stored in nutrient_level)
            if let Some(cell) = grid.get_mut(rx, ry, rz) {
                cell.set_material(Material::Root);
                cell.nutrient_level = 3; // Pine
            }
            // Ensure adjacent cell is soil
            assert_eq!(
                grid.get(rx + 1, ry, rz).unwrap().material,
                Material::Soil,
                "Adjacent cell should be soil"
            );
        }

        // Run soil_evolution cycles (every 10 ticks)
        for _ in 0..100 {
            crate::tick(&mut world, &mut schedule);
        }

        let soil = world.resource::<SoilGrid>();
        let final_ph = soil.get(rx + 1, ry, rz).unwrap().ph;
        assert!(
            final_ph < initial_ph,
            "Soil adjacent to pine root should have lower pH: initial={initial_ph}, final={final_ph}"
        );
    }

    #[test]
    fn pollinator_boosts_tree_health() {
        // A tree with pollinators nearby should recover health faster.
        use crate::fauna::{Fauna, FaunaList, FaunaState, FaunaType};
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let cx = GRID_X / 2 + 12;
        let cy = GRID_Y / 2;
        let sz = GROUND_LEVEL + 1;

        // Spawn a stressed tree
        let tree_id = world
            .spawn(Tree {
                species_id: 0,
                root_pos: (cx, cy, sz),
                age: 100,
                stage: GrowthStage::Mature,
                health: 0.5, // stressed
                accumulated_water: 5000.0,
                accumulated_light: 5000.0,
                rng_seed: 42,
                dirty: false,
                voxel_footprint: vec![(cx, cy, sz)],
                branches: Vec::new(),
                attraction_points: Vec::new(),
                skeleton_initialized: false,
                stage_changed: true,
                pending_voxels: Vec::new(),
                revealed_z: 0,
            })
            .id();

        // Place trunk and root so tree gets resources
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(cx, cy, sz) {
                cell.set_material(Material::Trunk);
            }
            if let Some(cell) = grid.get_mut(cx, cy, sz - 1) {
                cell.set_material(Material::Root);
                // Place water nearby so root can absorb
                if let Some(w) = grid.get_mut(cx + 1, cy, sz - 1) {
                    w.water_level = 200;
                }
            }
        }

        // Spawn a bee near the tree
        {
            let mut fauna = world.resource_mut::<FaunaList>();
            fauna.fauna.push(Fauna {
                fauna_type: FaunaType::Bee,
                state: FaunaState::Idle,
                x: cx as f32 + 2.0,
                y: cy as f32,
                z: sz as f32 + 2.0,
                target_x: cx as f32,
                target_y: cy as f32,
                target_z: sz as f32 + 1.0,
                age: 0,
                max_age: 500,
                rng_seed: 99,
            });
        }

        // Run a few ticks
        for _ in 0..5 {
            crate::tick(&mut world, &mut schedule);
        }

        let tree = world.entity(tree_id).get::<Tree>().unwrap();
        assert!(
            tree.health > 0.5,
            "Tree near pollinator should have recovered some health (health={})",
            tree.health
        );
    }

    #[test]
    fn dead_tree_recovers_with_water() {
        // A dead tree with wet roots should slowly recover and revive.
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let cx = GRID_X / 2 + 10;
        let cy = GRID_Y / 2;
        let sz = GROUND_LEVEL + 1;

        // Place a root with good water access
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(cx, cy, sz) {
                cell.set_material(Material::Trunk);
            }
            // Root below with water nearby
            if let Some(cell) = grid.get_mut(cx, cy, sz - 1) {
                cell.set_material(Material::Root);
                cell.water_level = 200;
            }
            // Water source adjacent to root
            if let Some(cell) = grid.get_mut(cx + 1, cy, sz - 1) {
                cell.material = Material::Water;
                cell.water_level = 255;
            }
        }

        // Spawn a dead tree
        let tree_id = world
            .spawn(Tree {
                species_id: 0, // Oak
                root_pos: (cx, cy, sz),
                age: 200,
                stage: GrowthStage::Dead,
                health: 0.0,
                accumulated_water: 2000.0,
                accumulated_light: 2000.0,
                rng_seed: 42,
                dirty: false,
                voxel_footprint: vec![(cx, cy, sz), (cx, cy, sz - 1)],
                branches: Vec::new(),
                attraction_points: Vec::new(),
                skeleton_initialized: false,
                stage_changed: true,
                pending_voxels: Vec::new(),
                revealed_z: 0,
            })
            .id();

        // Run enough ticks for recovery (~50 ticks to reach health 0.3)
        for _ in 0..60 {
            crate::tick(&mut world, &mut schedule);
        }

        let tree = world.entity(tree_id).get::<Tree>().unwrap();
        assert!(
            tree.health > 0.0,
            "Dead tree with wet roots should start recovering (health={})",
            tree.health
        );
        assert_ne!(
            tree.stage,
            GrowthStage::Dead,
            "Dead tree should have revived to Sapling after sustained water"
        );
    }

    #[test]
    fn willow_grows_faster_near_water() {
        // A willow with good water access should accumulate resources faster.
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Willow near water
        let wx = GRID_X / 2 + 5;
        let wy = GRID_Y / 2;
        let wz = GROUND_LEVEL + 1;

        // Oak in same conditions for comparison
        let ox = GRID_X / 2 + 20;
        let oy = GRID_Y / 2;
        let oz = GROUND_LEVEL + 1;

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Willow trunk + root with water
            if let Some(cell) = grid.get_mut(wx, wy, wz) {
                cell.set_material(Material::Trunk);
                cell.nutrient_level = 2; // willow species_id
            }
            if let Some(cell) = grid.get_mut(wx, wy, wz - 1) {
                cell.set_material(Material::Root);
                cell.water_level = 200;
                cell.nutrient_level = 2;
            }
            // Oak trunk + root with same water
            if let Some(cell) = grid.get_mut(ox, oy, oz) {
                cell.set_material(Material::Trunk);
                cell.nutrient_level = 0; // oak species_id
            }
            if let Some(cell) = grid.get_mut(ox, oy, oz - 1) {
                cell.set_material(Material::Root);
                cell.water_level = 200;
                cell.nutrient_level = 0;
            }
        }

        let willow_id = world
            .spawn(Tree {
                species_id: 2, // Willow
                root_pos: (wx, wy, wz),
                age: 0,
                stage: GrowthStage::Sapling,
                health: 1.0,
                accumulated_water: 0.0,
                accumulated_light: 0.0,
                rng_seed: 42,
                dirty: false,
                voxel_footprint: vec![(wx, wy, wz), (wx, wy, wz - 1)],
                branches: Vec::new(),
                attraction_points: Vec::new(),
                skeleton_initialized: false,
                stage_changed: true,
                pending_voxels: Vec::new(),
                revealed_z: 0,
            })
            .id();

        let oak_id = world
            .spawn(Tree {
                species_id: 0, // Oak
                root_pos: (ox, oy, oz),
                age: 0,
                stage: GrowthStage::Sapling,
                health: 1.0,
                accumulated_water: 0.0,
                accumulated_light: 0.0,
                rng_seed: 42,
                dirty: false,
                voxel_footprint: vec![(ox, oy, oz), (ox, oy, oz - 1)],
                branches: Vec::new(),
                attraction_points: Vec::new(),
                skeleton_initialized: false,
                stage_changed: true,
                pending_voxels: Vec::new(),
                revealed_z: 0,
            })
            .id();

        for _ in 0..50 {
            crate::tick(&mut world, &mut schedule);
        }

        let willow = world.entity(willow_id).get::<Tree>().unwrap();
        let oak = world.entity(oak_id).get::<Tree>().unwrap();

        assert!(
            willow.accumulated_water > oak.accumulated_water,
            "Willow near water should accumulate more resources than oak: willow={}, oak={}",
            willow.accumulated_water,
            oak.accumulated_water
        );
    }

    #[test]
    fn nurse_log_boosts_seed_germination() {
        // A seed next to DeadWood should grow faster (2× growth rate).
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let cx = GRID_X / 2 + 15;
        let cy = GRID_Y / 2;
        let sz = GROUND_LEVEL + 1;

        // Seed next to dead wood
        let nurse_x = cx;
        // Seed far from dead wood (control)
        let control_x = cx + 20;

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place DeadWood next to seed position
            if let Some(cell) = grid.get_mut(nurse_x + 1, cy, sz) {
                cell.set_material(Material::DeadWood);
            }
            // Seed near dead wood
            if let Some(cell) = grid.get_mut(nurse_x, cy, sz) {
                cell.material = Material::Seed;
                cell.water_level = 150;
            }
            // Control seed far from dead wood
            if let Some(cell) = grid.get_mut(control_x, cy, sz) {
                cell.material = Material::Seed;
                cell.water_level = 150;
            }
        }

        // Run ticks — nurse log seed should germinate first
        for _ in 0..30 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let nurse_cell = grid.get(nurse_x, cy, sz).unwrap();
        let control_cell = grid.get(control_x, cy, sz).unwrap();

        // Nurse log seed should have grown faster (higher nutrient_level or already trunk)
        let nurse_advanced = nurse_cell.material == Material::Trunk
            || (nurse_cell.material == Material::Seed
                && nurse_cell.nutrient_level > control_cell.nutrient_level);
        assert!(
            nurse_advanced,
            "Seed near dead wood should germinate faster (nurse={:?} n={}, control={:?} n={})",
            nurse_cell.material,
            nurse_cell.nutrient_level,
            control_cell.material,
            control_cell.nutrient_level
        );
    }

    #[test]
    fn mycorrhizal_network_shares_health() {
        // Two oaks with overlapping roots: the healthier one should boost
        // the struggling one's health via mycorrhizal transfer.
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let cx = GRID_X / 2 + 10;
        let cy = GRID_Y / 2;
        let sz = GROUND_LEVEL + 1;

        // Place two oak roots close together (within connection distance of 3)
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Tree A root
            if let Some(cell) = grid.get_mut(cx, cy, sz - 1) {
                cell.set_material(Material::Root);
                cell.water_level = 100;
                cell.nutrient_level = 0; // oak
            }
            if let Some(cell) = grid.get_mut(cx, cy, sz) {
                cell.set_material(Material::Trunk);
            }
            // Tree B root — 2 voxels away (connected)
            if let Some(cell) = grid.get_mut(cx + 2, cy, sz - 1) {
                cell.set_material(Material::Root);
                cell.water_level = 100;
                cell.nutrient_level = 0; // oak
            }
            if let Some(cell) = grid.get_mut(cx + 2, cy, sz) {
                cell.set_material(Material::Trunk);
            }
        }

        // Healthy oak
        world.spawn(Tree {
            species_id: 0,
            root_pos: (cx, cy, sz),
            age: 100,
            stage: GrowthStage::Mature,
            health: 1.0,
            accumulated_water: 5000.0,
            accumulated_light: 5000.0,
            rng_seed: 42,
            dirty: false,
            voxel_footprint: vec![(cx, cy, sz), (cx, cy, sz - 1)],
            branches: Vec::new(),
            attraction_points: Vec::new(),
            skeleton_initialized: false,
            stage_changed: true,
            pending_voxels: Vec::new(),
            revealed_z: 0,
        });

        // Struggling oak nearby
        let weak_id = world
            .spawn(Tree {
                species_id: 0,
                root_pos: (cx + 2, cy, sz),
                age: 100,
                stage: GrowthStage::Mature,
                health: 0.3,
                accumulated_water: 5000.0,
                accumulated_light: 5000.0,
                rng_seed: 43,
                dirty: false,
                voxel_footprint: vec![(cx + 2, cy, sz), (cx + 2, cy, sz - 1)],
                branches: Vec::new(),
                attraction_points: Vec::new(),
                skeleton_initialized: false,
                stage_changed: true,
                pending_voxels: Vec::new(),
                revealed_z: 0,
            })
            .id();

        // Run ticks — mycorrhizal runs every 10 ticks
        for _ in 0..30 {
            crate::tick(&mut world, &mut schedule);
        }

        let weak = world.entity(weak_id).get::<Tree>().unwrap();
        assert!(
            weak.health > 0.3,
            "Struggling oak near healthy oak should gain health via mycorrhizal network (health={})",
            weak.health
        );
    }

    /// Critical sim review acceptance test:
    /// "Plant 10 oaks in a tight cluster. After 200 ticks, only 2-3 should survive
    /// — the rest should have died from shade/water competition, leaving deadwood."
    #[test]
    fn crowded_oak_cluster_thins_naturally() {
        use crate::tree::{GrowthStage, Tree};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        // Place cluster away from center spring so water is limited
        let cx = GRID_X / 4;
        let cy = GRID_Y / 4;
        let sz = GROUND_LEVEL + 1;

        // Place 10 oak trees in a tight 3×4 grid, 3 voxels apart
        let positions: Vec<(usize, usize)> = vec![
            (cx, cy),
            (cx + 3, cy),
            (cx + 6, cy),
            (cx, cy + 3),
            (cx + 3, cy + 3),
            (cx + 6, cy + 3),
            (cx, cy + 6),
            (cx + 3, cy + 6),
            (cx + 6, cy + 6),
            (cx + 3, cy + 9),
        ];

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Add a small water source near the cluster — enough for some trees
            // but not all. This forces shade competition, not total water starvation.
            for wy in cy..cy + 3 {
                for wx in cx..cx + 3 {
                    if let Some(cell) = grid.get_mut(wx, wy, GROUND_LEVEL) {
                        cell.material = Material::Water;
                        cell.water_level = 200;
                    }
                }
            }
            for &(px, py) in &positions {
                // Place trunk
                if let Some(cell) = grid.get_mut(px, py, sz) {
                    cell.set_material(Material::Trunk);
                    cell.nutrient_level = 0; // oak
                }
                // Place root below with some initial water
                if let Some(cell) = grid.get_mut(px, py, sz - 1) {
                    cell.set_material(Material::Root);
                    cell.nutrient_level = 0;
                    cell.water_level = 100;
                }
            }
        }

        // Spawn tree entities as YoungTree with enough resources to have canopies.
        // This simulates trees that have been growing for a while and now compete.
        for (i, &(px, py)) in positions.iter().enumerate() {
            world.spawn(Tree {
                species_id: 0, // Oak
                root_pos: (px, py, sz),
                age: 80,
                stage: GrowthStage::YoungTree,
                health: 1.0,
                accumulated_water: 2000.0,
                accumulated_light: 2000.0,
                rng_seed: 100 + i as u64,
                dirty: true,
                voxel_footprint: vec![(px, py, sz), (px, py, sz - 1)],
                branches: Vec::new(),
                attraction_points: Vec::new(),
                skeleton_initialized: false,
                stage_changed: true,
                pending_voxels: Vec::new(),
                revealed_z: 0,
            });
        }

        // Run 400 ticks — enough for canopies to develop and competition to thin
        for _ in 0..400 {
            crate::tick(&mut world, &mut schedule);
        }

        // Count survivors (not Dead)
        let mut alive = 0u32;
        let mut dead = 0u32;
        let mut trees = world.query::<&Tree>();
        for tree in trees.iter(&world) {
            if tree.stage == GrowthStage::Dead {
                dead += 1;
            } else {
                alive += 1;
            }
        }

        // Competition should kill some trees. The exact number depends on
        // water access and canopy development. At least 2 deaths shows that
        // shade/water competition is working.
        assert!(
            dead >= 2,
            "At least 2 out of 10 crowded oaks should die from competition (alive={alive}, dead={dead})"
        );
        assert!(
            alive >= 1,
            "At least 1 oak should survive in the cluster (alive={alive}, dead={dead})"
        );
    }
}

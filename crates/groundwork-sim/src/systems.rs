use bevy_ecs::prelude::*;

use crate::grid::{VoxelGrid, GROUND_LEVEL, GRID_X, GRID_Y, GRID_Z};
use crate::soil::SoilGrid;
use crate::tree::{tree_hash, init_skeleton, generate_attraction_points, BranchNode, GrowthStage, SeedSpeciesMap, SpeciesTable, Tree, TreeTemplate};
use crate::voxel::Material;
use crate::Tick;

/// Persistent water spring: refills the spring and stream source each tick.
/// Without this, the spring dries up by tick ~200 and the garden dies.
pub fn water_spring(mut grid: ResMut<VoxelGrid>) {
    // Refill the 4x4 spring at center
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let pool_half = crate::scale::meters_to_voxels(0.2);
    for dy in (cy.saturating_sub(pool_half))..=(cy + pool_half - 1) {
        for dx in (cx.saturating_sub(pool_half))..=(cx + pool_half - 1) {
            let sz = VoxelGrid::surface_height(dx, dy);
            let wz = sz; // Spring sits at surface level
            if let Some(cell) = grid.get_mut(dx, dy, wz) {
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

                if x > 0 { lateral!(idx - 1); }
                if x + 1 < GRID_X { lateral!(idx + 1); }
                if y > 0 { lateral!(idx - GRID_X); }
                if y + 1 < GRID_Y { lateral!(idx + GRID_X); }
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
    let att_leaf = crate::scale::scale_attenuation(50);
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
pub fn seed_growth(mut grid: ResMut<VoxelGrid>, soil_grid: ResMut<SoilGrid>, mut commands: Commands, tick: Res<Tick>, mut seed_species: ResMut<SeedSpeciesMap>) {
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

        // Check own water or adjacent water.
        let mut has_water = cell_water >= 30;
        if !has_water {
            macro_rules! check_water {
                ($nidx:expr) => {
                    if cells[$nidx].water_level >= 30 { has_water = true; }
                };
            }
            if x > 0 { check_water!(idx - 1); }
            if !has_water && x + 1 < GRID_X { check_water!(idx + 1); }
            if !has_water && y > 0 { check_water!(idx - GRID_X); }
            if !has_water && y + 1 < GRID_Y { check_water!(idx + GRID_X); }
            if !has_water && z > 0 { check_water!(idx - z_stride); }
            if !has_water && z + 1 < GRID_Z { check_water!(idx + z_stride); }
        }

        let has_light = cell_light >= 30;

        if has_water && has_light {
            let mut best_nutrient: u8 = 0;
            let mut blocked_by_compaction = false;
            let soil_cells = soil_grid.cells();

            macro_rules! check_soil {
                ($nidx:expr) => {{
                    let nidx = $nidx;
                    if cells[nidx].material == Material::Soil {
                        let comp = &soil_cells[nidx];
                        // Use actual nutrient_level if available, fall back to capacity
                        let nl = cells[nidx].nutrient_level;
                        let effective = if nl > 0 { nl } else { comp.nutrient_capacity() };
                        if effective > best_nutrient { best_nutrient = effective; }
                        if comp.is_compacted() { blocked_by_compaction = true; }
                    }
                }};
            }

            if x > 0 { check_soil!(idx - 1); }
            if x + 1 < GRID_X { check_soil!(idx + 1); }
            if y > 0 { check_soil!(idx - GRID_X); }
            if y + 1 < GRID_Y { check_soil!(idx + GRID_X); }
            if z > 0 { check_soil!(idx - z_stride); }
            if z + 1 < GRID_Z { check_soil!(idx + z_stride); }

            if !blocked_by_compaction {
                let soil_bonus = (best_nutrient as u16 * 5 / 255) as u8;
                let growth_rate = 3 + soil_bonus;
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
                        let rng_seed = (tick.0 as u64)
                            .wrapping_mul(x as u64 + 1)
                            .wrapping_mul(y as u64 + 1);
                        commands.spawn(Tree {
                            species_id,
                            root_pos: (x, y, z),
                            age: 0,
                            stage: GrowthStage::Seedling,
                            health: 1.0,
                            accumulated_water: 0.0,
                            accumulated_light: 0.0,
                            rng_seed,
                            dirty: false,
                            voxel_footprint: footprint,
                            branches: Vec::new(),
                            attraction_points: Vec::new(),
                            skeleton_initialized: false,
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
                            water_absorbed[idx] = water_absorbed[idx].saturating_add(absorption.min(space));
                        } else if nmat == soil_u8 && my_water > nwater.saturating_add(5) {
                            let avg_drainage = (my_drainage + soil_cells[nidx].drainage_rate() as u16) / 2;
                            let diff = my_water - nwater;
                            let transfer = ((diff as u16 * avg_drainage) / (255 * 2)).max(1).min(max_diffusion) as i16;
                            diffusion_deltas[nidx] += transfer;
                            diffusion_deltas[idx] -= transfer;
                        }
                    }};
                }

                if x > 0 { check_neighbor!(idx - 1); }
                if x + 1 < GRID_X { check_neighbor!(idx + 1); }
                if y > 0 { check_neighbor!(idx - GRID_X); }
                if y + 1 < GRID_Y { check_neighbor!(idx + GRID_X); }
                if z > 0 { check_neighbor!(idx - z_stride); }
                if z + 1 < GRID_Z { check_neighbor!(idx + z_stride); }
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
pub fn tree_growth(
    mut trees: Query<&mut Tree>,
    grid: Res<VoxelGrid>,
    species_table: Res<SpeciesTable>,
) {
    for mut tree in trees.iter_mut() {
        if tree.stage == GrowthStage::Dead {
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

        // Use diminishing returns so more roots/light don't trivially blast
        // through all growth stages in a single tick. sqrt gives gentle scaling:
        // 100 water_intake → +10, 10000 → +100, 50000 → +224
        tree.accumulated_water += water_intake.sqrt() * species.growth_rate;
        tree.accumulated_light += light_intake.sqrt() * species.growth_rate;

        // Health declines without resources, recovers when well-supplied
        let water_ok = water_intake >= species.water_need.threshold();
        let light_ok = light_intake >= species.light_need.threshold();
        if !water_ok {
            tree.health = (tree.health - 0.01).max(0.0);
        }
        if !light_ok {
            tree.health = (tree.health - 0.005).max(0.0);
        }
        if water_ok && light_ok {
            tree.health = (tree.health + 0.005).min(1.0);
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

            // Initialize or expand skeleton on transition to branching stages
            // Only trees use space colonization; other plant types always use templates
            if species.uses_skeleton() {
                match next {
                    GrowthStage::YoungTree => {
                        let (branches, points) =
                            init_skeleton(species, &next, tree.rng_seed);
                        tree.branches = branches;
                        tree.attraction_points = points;
                        tree.skeleton_initialized = true;
                    }
                    GrowthStage::Mature | GrowthStage::OldGrowth => {
                        // Add more attraction points for larger crown
                        let new_points =
                            generate_attraction_points(species, &next, tree.rng_seed);
                        tree.attraction_points.extend(new_points);
                        // Extend trunk if needed
                        if prev == GrowthStage::YoungTree {
                            let old_trunk_h = (species.max_height() * 2 / 3).max(3) as isize;
                            let new_trunk_h = species.max_height() as isize;
                            for z in old_trunk_h..new_trunk_h {
                                let parent_idx = tree.branches.iter()
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
        if tree.attraction_points.len() < 10 && matches!(
            tree.stage,
            GrowthStage::YoungTree | GrowthStage::Mature | GrowthStage::OldGrowth
        ) {
            let new_points = crate::tree::generate_attraction_points(
                species, &tree.stage, tree.rng_seed.wrapping_add(tree.age as u64),
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

        // Grow up to 5 new nodes per tick for visible canopy development
        let mut grown = 0u32;
        for (ti, &tip_idx) in tips.iter().enumerate() {
            if grown >= 5 {
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
            let world_pos = (
                rx as isize + tp.0,
                ry as isize + tp.1,
                rz as isize + tp.2,
            );

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
            let (wnx, wny, wnz) = (world_new.0 as usize, world_new.1 as usize, world_new.2 as usize);
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

    let sx = if dx > 0.0 { 1isize } else if dx < 0.0 { -1 } else { 0 };
    let sy = if dy > 0.0 { 1isize } else if dy < 0.0 { -1 } else { 0 };
    let sz = if dz > 0.0 { 1isize } else if dz < 0.0 { -1 } else { 0 };

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
                let (wx, wy, wz) = (world_pos.0 as usize, world_pos.1 as usize, world_pos.2 as usize);
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
        tree.branches.retain(|n| n.alive || n.shade_stress < decay_threshold);
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
        let skeleton_root_positions: std::collections::HashSet<(usize, usize, usize)> =
            tree.branches.iter()
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

        // Clear old footprint, preserving dynamic roots
        for &(x, y, z) in &tree.voxel_footprint {
            if let Some(cell) = grid.get_mut(x, y, z) {
                match cell.material {
                    Material::Root if !skeleton_root_positions.contains(&(x, y, z)) => {
                        // This is a dynamic root — keep it
                        dynamic_roots.push((x, y, z));
                    }
                    Material::Trunk | Material::Branch | Material::Leaf
                    | Material::Root | Material::DeadWood => {
                        if z <= GROUND_LEVEL {
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

            for (i, node) in tree.branches.iter().enumerate() {
                if !node.alive && node.material != Material::DeadWood {
                    continue;
                }

                let ax = rx as isize + node.pos.0;
                let ay = ry as isize + node.pos.1;
                let az = rz as isize + node.pos.2;
                if ax < 0 || ay < 0 || az < 0 {
                    continue;
                }
                let (ax, ay, az) = (ax as usize, ay as usize, az as usize);
                if !VoxelGrid::in_bounds(ax, ay, az) {
                    continue;
                }

                // Tips get Leaf material, interior nodes keep their material
                let mat = if node.alive && !has_child[i] && node.material == Material::Branch {
                    Material::Leaf
                } else {
                    node.material
                };

                if let Some(cell) = grid.get_mut(ax, ay, az) {
                    let can_place = match mat {
                        Material::Root => {
                            cell.material == Material::Soil || cell.material == Material::Root
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
                        new_footprint.push((ax, ay, az));
                    }
                }
            }

            // Add leaf shells around alive tips for fuller canopy
            for (i, node) in tree.branches.iter().enumerate() {
                if !node.alive || has_child[i] || node.material == Material::Root {
                    continue;
                }
                // Place leaves in cardinal neighbors of tips
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let ax = rx as isize + node.pos.0 + dx;
                    let ay = ry as isize + node.pos.1 + dy;
                    let az = rz as isize + node.pos.2;
                    if ax < 0 || ay < 0 || az < 0 {
                        continue;
                    }
                    let (ax, ay, az) = (ax as usize, ay as usize, az as usize);
                    if !VoxelGrid::in_bounds(ax, ay, az) {
                        continue;
                    }
                    if let Some(cell) = grid.get_mut(ax, ay, az) {
                        if cell.material == Material::Air {
                            cell.set_material(Material::Leaf);
                            new_footprint.push((ax, ay, az));
                        }
                    }
                }
            }
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
                    .map_or(false, |v| v.material == Material::Root)
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

                    if best.map_or(true, |(_, _, _, s)| score > s) {
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

/// Mature trees disperse seeds nearby. Seeds land on soil via gravity.
pub fn seed_dispersal(
    trees: Query<&Tree>,
    mut grid: ResMut<VoxelGrid>,
    mut seed_species: ResMut<SeedSpeciesMap>,
    species_table: Res<SpeciesTable>,
) {
    for tree in trees.iter() {
        if !matches!(
            tree.stage,
            GrowthStage::Mature | GrowthStage::OldGrowth
        ) {
            continue;
        }
        if tree.health < 0.5 {
            continue;
        }

        let species = &species_table.species[tree.species_id];

        // Dispersal period varies per species and per individual tree
        let base_period = species.dispersal_period;
        let period = base_period + (tree_hash(tree.rng_seed, 0) % (base_period as u64 / 4 + 1)) as u32;
        if tree.age < period || tree.age % period != 0 {
            continue;
        }

        // Pick dispersal direction and distance (species-specific)
        let h = tree_hash(tree.rng_seed, tree.age as u64);
        let base_dist = crate::scale::meters_to_voxels(species.dispersal_distance_m / 2.0).max(1) as u64;
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
            seed_species.map.insert((sx, sy, landing_z), tree.species_id);
        }
    }
}

/// Roots absorb water from adjacent Soil voxels.
/// Transfer ~4 units per adjacent wet soil per tick. Root's water_level increases.
/// Visible effect: wet soil near roots dries out over time.
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
                            // Water absorption
                            if snapshot[nidx].1 > 0 {
                                let transfer = snapshot[nidx].1.min(max_transfer);
                                cells[nidx].water_level = cells[nidx].water_level.saturating_sub(transfer);
                                cells[idx].water_level = cells[idx].water_level.saturating_add(transfer);
                            }
                            // Nutrient absorption
                            if snapshot[nidx].2 > 0 {
                                let transfer = snapshot[nidx].2.min(max_nutrient_transfer);
                                cells[nidx].nutrient_level = cells[nidx].nutrient_level.saturating_sub(transfer);
                                // Root stores absorbed nutrients (visible in inspect)
                                cells[idx].nutrient_level = cells[idx].nutrient_level.saturating_add(transfer);
                            }
                        }
                    }};
                }

                if x > 0 { absorb!(idx - 1); }
                if x + 1 < GRID_X { absorb!(idx + 1); }
                if y > 0 { absorb!(idx - GRID_X); }
                if y + 1 < GRID_Y { absorb!(idx + GRID_X); }
                if z > 0 { absorb!(idx - z_stride); }
                if z + 1 < GRID_Z { absorb!(idx + z_stride); }
            }
        }
    }
}

/// Soil evolves over time based on environmental interactions.
/// - Organic matter increases near roots, slowly decays without them.
/// - Bacteria grow in moist, organic-rich soil; die in dry soil.
/// - pH drifts acidic with organic decomposition; rock buffers toward neutral.
/// - Rock fragments slowly weather into clay when wet.
pub fn soil_evolution(mut grid: ResMut<VoxelGrid>, mut soil_grid: ResMut<SoilGrid>, tick: Res<Tick>) {
    // Soil chemistry changes slowly — only run every 10 ticks.
    // Organic/bacteria/pH increments are scaled 10× to compensate.
    if tick.0 % 10 != 0 {
        return;
    }

    // Snapshot grid state so we can read neighbors while writing nutrient_level.
    let snapshot: Vec<(u8, u8)> = grid
        .cells()
        .iter()
        .map(|v| (v.material.as_u8(), v.water_level))
        .collect();

    let soil_cells = soil_grid.cells_mut();

    let root_u8 = Material::Root.as_u8();
    let soil_u8 = Material::Soil.as_u8();
    let z_stride = GRID_X * GRID_Y;

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
                if x > 0 && snapshot[idx - 1].0 == root_u8 { adjacent_roots += 1; }
                if x + 1 < GRID_X && snapshot[idx + 1].0 == root_u8 { adjacent_roots += 1; }
                if y > 0 && snapshot[idx - GRID_X].0 == root_u8 { adjacent_roots += 1; }
                if y + 1 < GRID_Y && snapshot[idx + GRID_X].0 == root_u8 { adjacent_roots += 1; }
                if z > 0 && snapshot[idx - z_stride].0 == root_u8 { adjacent_roots += 1; }
                if z + 1 < GRID_Z && snapshot[idx + z_stride].0 == root_u8 { adjacent_roots += 1; }

                if adjacent_roots > 0 {
                    // 10× because we run every 10 ticks
                    comp.organic = comp.organic.saturating_add(adjacent_roots.min(2) * 10);
                } else if comp.organic > 0 {
                    // Slow decay without roots: -1 every ~10 ticks → -1 per run
                    if (x + y + z) % 10 == 0 {
                        comp.organic = comp.organic.saturating_sub(1);
                    }
                }

                // --- Bacteria ---
                // 10× increments to compensate for running every 10 ticks
                if water_level > 50 && comp.organic > 30 {
                    comp.bacteria = comp.bacteria.saturating_add(10);
                } else if water_level < 10 {
                    comp.bacteria = comp.bacteria.saturating_sub(20);
                } else if comp.organic < 15 {
                    if (x + y) % 5 == 0 {
                        comp.bacteria = comp.bacteria.saturating_sub(10);
                    }
                }

                // --- pH drift ---
                if comp.organic > 100 && comp.ph > 0 {
                    if (x + z) % 20 == 0 {
                        comp.ph = comp.ph.saturating_sub(1);
                    }
                }
                if comp.rock > 50 && comp.ph < 128 {
                    if (y + z) % 25 == 0 {
                        comp.ph = comp.ph.saturating_add(1);
                    }
                }

                // --- Rock weathering ---
                if comp.rock > 0 && water_level > 30 {
                    if (x + y + z) % 50 == 0 {
                        comp.rock = comp.rock.saturating_sub(1);
                        comp.clay = comp.clay.saturating_add(1);
                    }
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
            grid_cells[idx].nutrient_level = grid_cells[idx].nutrient_level.saturating_add(gen).min(cap);
        }
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

        // Use position away from grid edges so soil is loam (higher nutrient capacity).
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(30, 10, GROUND_LEVEL + 1) {
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
        let cell = grid.get(30, 10, GROUND_LEVEL + 1).unwrap();
        assert_eq!(
            cell.material,
            Material::Trunk,
            "Seed should have grown into trunk after 80 ticks with water and light"
        );

        // Root should have been placed in the soil below
        let below = grid.get(30, 10, GROUND_LEVEL).unwrap();
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
        assert_eq!(root.water_level, expected, "Root should absorb scaled amount from 2 neighbors");

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
                        if dx == 0 && dy == 0 && dz == 0 { continue; }
                        let nx = root_x as i32 + dx;
                        let ny = root_y as i32 + dy;
                        let nz = root_z as i32 + dz;
                        if nx < 0 || ny < 0 || nz < 0 { continue; }
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
                        if nx == root_x && ny == root_y && nz == root_z { continue; }
                        if nx == root_x + 1 && ny == root_y && nz == root_z { continue; }
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
        // Growth rate is soil-dependent (3-8/tick). Using inland position for loam soil (~5/tick).
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

        // After 10 ticks with soil-aware growth (3-8/tick): at most 80, should be < 100.
        for _ in 0..10 {
            crate::tick(&mut world, &mut schedule);
        }

        {
            let grid = world.resource::<VoxelGrid>();
            let cell = grid.get(30, 10, GROUND_LEVEL + 1).unwrap();
            assert_eq!(cell.material, Material::Seed, "Should still be a seed at 10 ticks");
            assert!(
                cell.nutrient_level < 100,
                "At 10 ticks, nutrient_level ({}) should be < 100 (small seed stage)",
                cell.nutrient_level
            );
        }

        // After 25 more ticks (35 total): growth should cross 100 threshold
        // Even at minimum rate 3/tick: 34 growing ticks * 3 = 102 >= 100
        for _ in 0..25 {
            crate::tick(&mut world, &mut schedule);
        }

        {
            let grid = world.resource::<VoxelGrid>();
            let cell = grid.get(30, 10, GROUND_LEVEL + 1).unwrap();
            assert_eq!(cell.material, Material::Seed, "Should still be a seed at 35 ticks");
            assert!(
                cell.nutrient_level >= 100,
                "At 35 ticks, nutrient_level ({}) should be >= 100 (growing seed stage 'S')",
                cell.nutrient_level
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
            *soil.get_mut(sandy_pos.0, sandy_pos.1, sandy_pos.2).unwrap() = SoilComposition::sandy();
            *soil.get_mut(clay_pos.0, clay_pos.1, clay_pos.2).unwrap() = SoilComposition::clay();
        }

        // Run a few ticks
        for _ in 0..5 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let sandy_water = grid.get(sandy_pos.0, sandy_pos.1, sandy_pos.2).unwrap().water_level;
        let clay_water = grid.get(clay_pos.0, clay_pos.1, clay_pos.2).unwrap().water_level;

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
                sand: 42, clay: 99, organic: 150, rock: 77, ph: 200, bacteria: 33,
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
        assert_eq!(comp.type_name(), "loam", "V2 backward compat should generate loam topsoil");

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
            *comp = SoilComposition { sand: 10, clay: 220, organic: 5, rock: 10, ph: 128, bacteria: 2 };
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
        // Position near spring for good water access and loam soil.
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let tx = 25;
        let ty = 25;
        let tz = VoxelGrid::surface_height(tx, ty) + 1;

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(tx, ty, tz) {
                cell.material = Material::Seed;
                cell.water_level = 100;
            }
            // Ensure soil below is wet so roots can accumulate water.
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
            let period = base_period + (crate::tree::tree_hash(rng_seed, 0) % (base_period as u64 / 4 + 1)) as u32;
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
            });

            crate::tick(&mut world, &mut schedule);

            // Search the entire grid for a dispersed seed.
            let grid = world.resource::<VoxelGrid>();
            for sy in 0..GRID_Y {
                for sx in 0..GRID_X {
                    if sx == tx && sy == ty { continue; }
                    // Seeds land at surface + 1
                    let sh = VoxelGrid::surface_height(sx, sy);
                    let seed_z = sh + 1;
                    if let Some(cell) = grid.get(sx, sy, seed_z) {
                        if cell.material == Material::Seed {
                            found_seed = true;
                        }
                    }
                }
                if found_seed { break; }
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
        use crate::tree::{GrowthStage, Tree, init_skeleton};

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
        });

        // Run several ticks to allow branch growth
        for _ in 0..15 {
            crate::tick(&mut world, &mut schedule);
        }

        let mut trees = world.query::<&Tree>();
        let tree = trees.iter(&world).next().unwrap();
        assert!(
            tree.branches.len() > initial_branch_count,
            "Branch growth should add nodes: initial={}, current={}",
            initial_branch_count,
            tree.branches.len()
        );
        // Should have consumed some attraction points (started with 60 for YoungTree)
        assert!(
            tree.attraction_points.len() < 60,
            "Some attraction points should have been consumed: current={}",
            tree.attraction_points.len()
        );
    }

    #[test]
    fn skeleton_rasterize_produces_voxels() {
        use crate::tree::{GrowthStage, Tree, BranchNode};

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
            BranchNode { pos: (0, 0, 0), parent: u16::MAX, material: Material::Trunk, shade_stress: 0, alive: true },
            BranchNode { pos: (0, 0, 1), parent: 0, material: Material::Trunk, shade_stress: 0, alive: true },
            BranchNode { pos: (0, 0, 2), parent: 1, material: Material::Trunk, shade_stress: 0, alive: true },
            BranchNode { pos: (1, 0, 2), parent: 2, material: Material::Branch, shade_stress: 0, alive: true },
            BranchNode { pos: (0, 0, -1), parent: 0, material: Material::Root, shade_stress: 0, alive: true },
        ];

        world.spawn(Tree {
            species_id: 0,
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
        });

        crate::tick(&mut world, &mut schedule);

        let grid = world.resource::<VoxelGrid>();
        // Trunk at root_pos
        assert_eq!(grid.get(tx, ty, tz).unwrap().material, Material::Trunk);
        // Trunk at z+1
        assert_eq!(grid.get(tx, ty, tz + 1).unwrap().material, Material::Trunk);
        // Trunk at z+2
        assert_eq!(grid.get(tx, ty, tz + 2).unwrap().material, Material::Trunk);
        // Branch tip at (1,0,2) should become Leaf (it's a tip)
        assert_eq!(grid.get(tx + 1, ty, tz + 2).unwrap().material, Material::Leaf);
        // Root below (pos (0,0,-1) = tz-1)
        assert_eq!(grid.get(tx, ty, tz - 1).unwrap().material, Material::Root);
    }

    #[test]
    fn self_pruning_kills_shaded_branches() {
        use crate::tree::{GrowthStage, Tree, BranchNode};

        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        let tx = 20;
        let ty = 20;
        let tz = GROUND_LEVEL + 1;

        // Build a tree with a branch node that will be fully shaded
        let branches = vec![
            BranchNode { pos: (0, 0, 0), parent: u16::MAX, material: Material::Trunk, shade_stress: 0, alive: true },
            BranchNode { pos: (0, 0, 1), parent: 0, material: Material::Trunk, shade_stress: 0, alive: true },
            // This branch at z=0 level will be underground/shaded
            BranchNode { pos: (1, 0, 0), parent: 0, material: Material::Branch, shade_stress: 190, alive: true },
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
}

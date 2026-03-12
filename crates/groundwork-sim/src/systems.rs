use bevy_ecs::system::ResMut;

use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z};
use crate::soil::SoilGrid;
use crate::voxel::Material;

/// Gravity-driven water flow. Each tick, water tries to move down,
/// then spreads laterally to lower-water neighbors.
///
/// Uses a snapshot buffer to avoid iteration-order artifacts.
pub fn water_flow(mut grid: ResMut<VoxelGrid>) {
    // Snapshot water levels before mutation.
    let snapshot: Vec<u8> = grid.cells().iter().map(|v| v.water_level).collect();
    // Delta buffer for lateral spread — applied after the full pass to avoid
    // iteration-order bias that caused diagonal stripe artifacts.
    let mut lateral_deltas: Vec<i16> = vec![0; snapshot.len()];

    // Iterate top-to-bottom so gravity cascades naturally.
    for z in (0..GRID_Z).rev() {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = VoxelGrid::index(x, y, z);
                let water = snapshot[idx];
                if water == 0 {
                    continue;
                }

                let mat = grid.cells()[idx].material;
                // Only Air and Water cells carry free water.
                if mat != Material::Air && mat != Material::Water {
                    continue;
                }

                // Try to flow down.
                if z > 0 {
                    let below = grid.get(x, y, z - 1).copied();
                    if let Some(bv) = below {
                        if (bv.material == Material::Air || bv.material == Material::Water)
                            && bv.water_level < 255
                        {
                            let transfer = water.min(255 - bv.water_level).min(32);
                            if let Some(cell) = grid.get_mut(x, y, z - 1) {
                                cell.water_level = cell.water_level.saturating_add(transfer);
                                if cell.material != Material::Water {
                                    cell.nutrient_level = 0;
                                    cell.material = Material::Water;
                                }
                            }
                            if let Some(cell) = grid.get_mut(x, y, z) {
                                cell.water_level = cell.water_level.saturating_sub(transfer);
                                if cell.water_level == 0 && cell.material == Material::Water {
                                    cell.nutrient_level = 0;
                                    cell.material = Material::Air;
                                }
                            }
                            continue;
                        }
                    }
                }

                // Can't flow down — record lateral spread into delta buffer.
                let neighbors: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                for (dx, dy) in neighbors {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx < 0 || ny < 0 {
                        continue;
                    }
                    let (nx, ny) = (nx as usize, ny as usize);
                    if !VoxelGrid::in_bounds(nx, ny, z) {
                        continue;
                    }
                    let nidx = VoxelGrid::index(nx, ny, z);
                    let neighbor_water = snapshot[nidx];
                    let neighbor_mat = grid.cells()[nidx].material;

                    if (neighbor_mat == Material::Air || neighbor_mat == Material::Water)
                        && neighbor_water < water.saturating_sub(1)
                    {
                        let transfer = ((water - neighbor_water) / 5).max(1).min(8);
                        lateral_deltas[nidx] += transfer as i16;
                        lateral_deltas[idx] -= transfer as i16;
                    }
                }
            }
        }
    }

    // Apply lateral spread deltas in one pass to avoid iteration-order bias.
    for (i, &delta) in lateral_deltas.iter().enumerate() {
        if delta == 0 {
            continue;
        }
        let cell = &mut grid.cells_mut()[i];
        if delta > 0 {
            cell.water_level = cell.water_level.saturating_add(delta as u8);
            if cell.material != Material::Water
                && (cell.material == Material::Air)
            {
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
    // This prevents the checkerboard frontier artifact where tiny amounts
    // of water oscillate between adjacent cells.
    for cell in grid.cells_mut().iter_mut() {
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
    for y in 0..GRID_Y {
        for x in 0..GRID_X {
            let mut light: u8 = 255;
            for z in (0..GRID_Z).rev() {
                if let Some(cell) = grid.get_mut(x, y, z) {
                    // Opaque materials attenuate *before* assignment so
                    // the first soil layer below open sky isn't full brightness.
                    match cell.material {
                        Material::Soil => {
                            light = light.saturating_sub(30);
                        }
                        Material::Root => {
                            light = light.saturating_sub(30);
                        }
                        Material::Stone => {
                            light = 0;
                        }
                        _ => {}
                    }
                    cell.light_level = light;
                    // Transparent materials attenuate *after* assignment.
                    match cell.material {
                        Material::Air => {
                            light = light.saturating_sub(2);
                        }
                        Material::Water => {
                            light = light.saturating_sub(15);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Seeds grow into roots when they have enough water and light.
/// Uses nutrient_level as a growth counter: increments by 3-8 each tick
/// (based on adjacent soil quality) when conditions are met, converts to Root at 200.
pub fn seed_growth(mut grid: ResMut<VoxelGrid>, soil_grid: ResMut<SoilGrid>) {
    // Snapshot to check neighbors without order artifacts.
    let snapshot: Vec<(Material, u8)> = grid
        .cells()
        .iter()
        .map(|v| (v.material, v.water_level))
        .collect();

    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = VoxelGrid::index(x, y, z);
                if snapshot[idx].0 != Material::Seed {
                    continue;
                }

                let cell_water = grid.cells()[idx].water_level;
                let cell_light = grid.cells()[idx].light_level;

                // Check own water or adjacent water.
                let mut has_water = cell_water >= 30;
                if !has_water {
                    let neighbors: [(isize, isize, isize); 6] = [
                        (-1, 0, 0),
                        (1, 0, 0),
                        (0, -1, 0),
                        (0, 1, 0),
                        (0, 0, -1),
                        (0, 0, 1),
                    ];
                    for (dx, dy, dz) in neighbors {
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;
                        let nz = z as isize + dz;
                        if nx < 0 || ny < 0 || nz < 0 {
                            continue;
                        }
                        let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                        if !VoxelGrid::in_bounds(nx, ny, nz) {
                            continue;
                        }
                        let nidx = VoxelGrid::index(nx, ny, nz);
                        if snapshot[nidx].1 >= 30 {
                            has_water = true;
                            break;
                        }
                    }
                }

                let has_light = cell_light >= 30;

                if has_water && has_light {
                    // Check soil quality of adjacent soil for growth rate bonus.
                    // Base growth: 3/tick. Nutrient-rich soil adds up to +5.
                    let mut best_nutrient_cap: u8 = 0;
                    let mut blocked_by_compaction = false;
                    let soil_neighbors: [(isize, isize, isize); 6] = [
                        (-1, 0, 0), (1, 0, 0),
                        (0, -1, 0), (0, 1, 0),
                        (0, 0, -1), (0, 0, 1),
                    ];
                    for (dx, dy, dz) in soil_neighbors {
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;
                        let nz = z as isize + dz;
                        if nx < 0 || ny < 0 || nz < 0 { continue; }
                        let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                        if !VoxelGrid::in_bounds(nx, ny, nz) { continue; }
                        let nidx = VoxelGrid::index(nx, ny, nz);
                        if snapshot[nidx].0 == Material::Soil {
                            if let Some(comp) = soil_grid.get(nx, ny, nz) {
                                let nc = comp.nutrient_capacity();
                                if nc > best_nutrient_cap {
                                    best_nutrient_cap = nc;
                                }
                                if comp.is_compacted() {
                                    blocked_by_compaction = true;
                                }
                            }
                        }
                    }

                    // Compacted soil blocks growth entirely
                    if !blocked_by_compaction {
                        // Growth rate: 3 base + up to 5 from soil nutrients
                        let soil_bonus = (best_nutrient_cap as u16 * 5 / 255) as u8;
                        let growth_rate = 3 + soil_bonus;
                        if let Some(cell) = grid.get_mut(x, y, z) {
                            cell.nutrient_level = cell.nutrient_level.saturating_add(growth_rate);
                            if cell.nutrient_level >= 200 {
                                cell.set_material(Material::Root);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Soil absorbs water from adjacent Water voxels and diffuses water to
/// neighboring soil. Absorption and diffusion rates depend on soil composition.
pub fn soil_absorption(mut grid: ResMut<VoxelGrid>, soil_grid: ResMut<SoilGrid>) {
    let snapshot: Vec<(Material, u8)> = grid
        .cells()
        .iter()
        .map(|v| (v.material, v.water_level))
        .collect();

    // Delta buffer for soil-to-soil diffusion (applied after full pass).
    let mut diffusion_deltas: Vec<i16> = vec![0; snapshot.len()];

    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = VoxelGrid::index(x, y, z);
                if snapshot[idx].0 != Material::Soil {
                    continue;
                }

                let comp = soil_grid.get(x, y, z).copied().unwrap_or_default();
                // Base absorption rate from composition: sandy absorbs fast,
                // clay absorbs slow. Range: 1-4 per adjacent water per tick.
                let absorption = 1 + (comp.drainage_rate() as u16 * 3 / 255) as u8;
                // Water retention caps how much water soil will hold.
                // Range: 80-255 based on retention.
                let max_water = 80 + (comp.water_retention() as u16 * 175 / 255) as u8;

                let neighbors: [(isize, isize, isize); 6] = [
                    (-1, 0, 0),
                    (1, 0, 0),
                    (0, -1, 0),
                    (0, 1, 0),
                    (0, 0, -1),
                    (0, 0, 1),
                ];
                for (dx, dy, dz) in neighbors {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    let nz = z as isize + dz;
                    if nx < 0 || ny < 0 || nz < 0 {
                        continue;
                    }
                    let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                    if !VoxelGrid::in_bounds(nx, ny, nz) {
                        continue;
                    }
                    let nidx = VoxelGrid::index(nx, ny, nz);

                    // Absorb from adjacent water
                    if snapshot[nidx].0 == Material::Water && snapshot[nidx].1 > 0 {
                        let current = snapshot[idx].1;
                        if current < max_water {
                            let space = max_water.saturating_sub(current);
                            let transfer = absorption.min(space);
                            if let Some(soil) = grid.get_mut(x, y, z) {
                                soil.water_level = soil.water_level.saturating_add(transfer);
                            }
                        }
                    }

                    // Soil-to-soil diffusion: water moves from wetter to drier soil
                    if snapshot[nidx].0 == Material::Soil {
                        let my_water = snapshot[idx].1;
                        let their_water = snapshot[nidx].1;
                        if my_water > their_water.saturating_add(5) {
                            // Diffusion rate: avg of both cells' drainage rates
                            let n_comp = soil_grid.get(nx, ny, nz).copied().unwrap_or_default();
                            let avg_drainage = (comp.drainage_rate() as u16 + n_comp.drainage_rate() as u16) / 2;
                            let diff = my_water - their_water;
                            let transfer = ((diff as u16 * avg_drainage) / (255 * 4)).max(1).min(4) as i16;
                            diffusion_deltas[nidx] += transfer;
                            diffusion_deltas[idx] -= transfer;
                        }
                    }
                }
            }
        }
    }

    // Apply soil-to-soil diffusion deltas
    for (i, &delta) in diffusion_deltas.iter().enumerate() {
        if delta == 0 {
            continue;
        }
        let cell = &mut grid.cells_mut()[i];
        if cell.material != Material::Soil {
            continue;
        }
        if delta > 0 {
            cell.water_level = cell.water_level.saturating_add(delta as u8);
        } else {
            cell.water_level = cell.water_level.saturating_sub((-delta) as u8);
        }
    }
}

/// Roots absorb water from adjacent Soil voxels.
/// Transfer ~4 units per adjacent wet soil per tick. Root's water_level increases.
/// Visible effect: wet soil near roots dries out over time.
pub fn root_water_absorption(mut grid: ResMut<VoxelGrid>) {
    let snapshot: Vec<(Material, u8)> = grid
        .cells()
        .iter()
        .map(|v| (v.material, v.water_level))
        .collect();

    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = VoxelGrid::index(x, y, z);
                if snapshot[idx].0 != Material::Root {
                    continue;
                }

                let neighbors: [(isize, isize, isize); 6] = [
                    (-1, 0, 0),
                    (1, 0, 0),
                    (0, -1, 0),
                    (0, 1, 0),
                    (0, 0, -1),
                    (0, 0, 1),
                ];
                for (dx, dy, dz) in neighbors {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    let nz = z as isize + dz;
                    if nx < 0 || ny < 0 || nz < 0 {
                        continue;
                    }
                    let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                    if !VoxelGrid::in_bounds(nx, ny, nz) {
                        continue;
                    }
                    let nidx = VoxelGrid::index(nx, ny, nz);
                    if snapshot[nidx].0 == Material::Soil && snapshot[nidx].1 > 0 {
                        let transfer = snapshot[nidx].1.min(4);
                        if let Some(soil) = grid.get_mut(nx, ny, nz) {
                            soil.water_level = soil.water_level.saturating_sub(transfer);
                        }
                        if let Some(root) = grid.get_mut(x, y, z) {
                            root.water_level = root.water_level.saturating_add(transfer);
                        }
                    }
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
pub fn soil_evolution(grid: ResMut<VoxelGrid>, mut soil_grid: ResMut<SoilGrid>) {
    let tick_mod = {
        // We don't have access to the Tick resource here, so we use a simple
        // approach: slow processes happen probabilistically based on field values
        // rather than tick counting. This avoids needing an extra system parameter.
        // The system runs every tick, but slow changes use saturating math and
        // small increments.
        true
    };
    if !tick_mod {
        return;
    }

    let snapshot: Vec<(Material, u8)> = grid
        .cells()
        .iter()
        .map(|v| (v.material, v.water_level))
        .collect();

    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = VoxelGrid::index(x, y, z);
                if snapshot[idx].0 != Material::Soil {
                    continue;
                }

                let water_level = grid.cells()[idx].water_level;
                let comp = &mut soil_grid.cells_mut()[idx];

                // --- Organic matter ---
                // Increases when adjacent to roots (+1/tick per adjacent root)
                let neighbors: [(isize, isize, isize); 6] = [
                    (-1, 0, 0), (1, 0, 0),
                    (0, -1, 0), (0, 1, 0),
                    (0, 0, -1), (0, 0, 1),
                ];
                let mut adjacent_roots = 0u8;
                for (dx, dy, dz) in neighbors {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    let nz = z as isize + dz;
                    if nx < 0 || ny < 0 || nz < 0 { continue; }
                    let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                    if !VoxelGrid::in_bounds(nx, ny, nz) { continue; }
                    let nidx = VoxelGrid::index(nx, ny, nz);
                    if snapshot[nidx].0 == Material::Root {
                        adjacent_roots += 1;
                    }
                }

                if adjacent_roots > 0 {
                    comp.organic = comp.organic.saturating_add(adjacent_roots.min(2));
                } else if comp.organic > 0 {
                    // Slow decay without roots: -1 every ~10 ticks
                    // Use a simple deterministic approach based on position
                    if (x + y + z) % 10 == 0 {
                        comp.organic = comp.organic.saturating_sub(1);
                    }
                }

                // --- Bacteria ---
                // Grow when moist + organic-rich; die when dry
                if water_level > 50 && comp.organic > 30 {
                    comp.bacteria = comp.bacteria.saturating_add(1);
                } else if water_level < 10 {
                    // Dry soil kills bacteria faster
                    comp.bacteria = comp.bacteria.saturating_sub(2);
                } else if comp.organic < 15 {
                    // Low organic = bacteria starve slowly
                    if (x + y) % 5 == 0 {
                        comp.bacteria = comp.bacteria.saturating_sub(1);
                    }
                }

                // --- pH drift ---
                // Organic decomposition makes soil more acidic
                if comp.organic > 100 && comp.ph > 0 {
                    if (x + z) % 20 == 0 {
                        comp.ph = comp.ph.saturating_sub(1);
                    }
                }
                // Rock fragments buffer toward neutral (pH ~6.0 = byte 128)
                if comp.rock > 50 && comp.ph < 128 {
                    if (y + z) % 25 == 0 {
                        comp.ph = comp.ph.saturating_add(1);
                    }
                }

                // --- Rock weathering ---
                // Wet rock fragments slowly break down into clay
                if comp.rock > 0 && water_level > 30 {
                    if (x + y + z) % 50 == 0 {
                        comp.rock = comp.rock.saturating_sub(1);
                        comp.clay = comp.clay.saturating_add(1);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::GROUND_LEVEL;

    #[test]
    fn seed_grows_into_root_on_wet_lit_soil() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            // Place seed above ground with enough water on it.
            // water_flow skips non-Air/Water materials, so seed keeps its water.
            if let Some(cell) = grid.get_mut(10, 10, GROUND_LEVEL + 1) {
                cell.material = Material::Seed;
                cell.water_level = 100;
                cell.light_level = 0;
                cell.nutrient_level = 0;
            }
        }

        // 200 / 5 = 40 ticks minimum, extra for light to propagate first tick.
        for _ in 0..50 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let cell = grid.get(10, 10, GROUND_LEVEL + 1).unwrap();
        assert_eq!(
            cell.material,
            Material::Root,
            "Seed should have grown into root after 50 ticks with water and light"
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
    fn seed_to_root_resets_water_level() {
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(10, 10, GROUND_LEVEL + 1) {
                cell.material = Material::Seed;
                cell.water_level = 100;
                cell.light_level = 0;
                cell.nutrient_level = 0;
            }
        }

        for _ in 0..50 {
            crate::tick(&mut world, &mut schedule);
        }

        let grid = world.resource::<VoxelGrid>();
        let cell = grid.get(10, 10, GROUND_LEVEL + 1).unwrap();
        if cell.material == Material::Root {
            assert_eq!(
                cell.water_level, 0,
                "Root converted from seed should not retain stale water_level"
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
        // Each wet soil neighbor donates 4, two neighbors = 8.
        assert_eq!(root.water_level, 8, "Root should absorb 4 from each of 2 wet soil neighbors");

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

        let root_x = 5;
        let root_y = 5;
        let root_z = GROUND_LEVEL - 1;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(root_x, root_y, root_z) {
                cell.material = Material::Root;
                cell.water_level = 0;
            }
            // Ensure adjacent soil is dry.
            for (dx, dy, dz) in [(-1i32,0,0),(1,0,0),(0,-1,0),(0,1,0),(0,0,-1),(0,0,1)] {
                let nx = (root_x as i32 + dx) as usize;
                let ny = (root_y as i32 + dy) as usize;
                let nz = (root_z as i32 + dz) as usize;
                if let Some(cell) = grid.get_mut(nx, ny, nz) {
                    if cell.material == Material::Soil {
                        cell.water_level = 0;
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

        let root_x = 15;
        let root_y = 15;
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
            // Dry out other soil neighbors to isolate the test.
            for (dx, dy, dz) in [(-1i32,0,0),(0,-1,0),(0,1,0),(0,0,-1),(0,0,1)] {
                let nx = (root_x as i32 + dx) as usize;
                let ny = (root_y as i32 + dy) as usize;
                let nz = (root_z as i32 + dz) as usize;
                if let Some(cell) = grid.get_mut(nx, ny, nz) {
                    if cell.material == Material::Soil {
                        cell.water_level = 0;
                    }
                }
            }
        }

        // Run enough ticks to drain the soil (20 water / 4 per tick = 5 ticks).
        for _ in 0..5 {
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

        // Place stone floor at z=0, air everywhere else, water blob at center z=1.
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
            let cx = GRID_X / 2;
            let cy = GRID_Y / 2;
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
        let cx = GRID_X / 2;
        let cy = GRID_Y / 2;
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
        let mut world = crate::create_world();
        let mut schedule = crate::create_schedule();

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(10, 10, GROUND_LEVEL + 1) {
                cell.material = Material::Seed;
                cell.water_level = 100;
                cell.light_level = 0;
                cell.nutrient_level = 0;
            }
        }

        // After 15 ticks: growth = 15*5 = 75 (first tick may not grow due to light propagation)
        // Should still be 's' stage (< 100)
        for _ in 0..15 {
            crate::tick(&mut world, &mut schedule);
        }

        {
            let grid = world.resource::<VoxelGrid>();
            let cell = grid.get(10, 10, GROUND_LEVEL + 1).unwrap();
            assert_eq!(cell.material, Material::Seed, "Should still be a seed at 15 ticks");
            // First tick has no light yet, so ~14 growth ticks = 70
            assert!(
                cell.nutrient_level < 100,
                "At 15 ticks, nutrient_level ({}) should be < 100 (small seed stage)",
                cell.nutrient_level
            );
        }

        // After 10 more ticks (25 total): growth should cross 100 threshold
        for _ in 0..10 {
            crate::tick(&mut world, &mut schedule);
        }

        {
            let grid = world.resource::<VoxelGrid>();
            let cell = grid.get(10, 10, GROUND_LEVEL + 1).unwrap();
            assert_eq!(cell.material, Material::Seed, "Should still be a seed at 25 ticks");
            assert!(
                cell.nutrient_level >= 100,
                "At 25 ticks, nutrient_level ({}) should be >= 100 (growing seed stage 'S')",
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
        let sky = grid.get(0, 0, GROUND_LEVEL + 2).unwrap().light_level;
        let surface_soil = grid.get(0, 0, GROUND_LEVEL).unwrap().light_level;
        let one_below = grid.get(0, 0, GROUND_LEVEL - 1).unwrap().light_level;
        let two_below = grid.get(0, 0, GROUND_LEVEL - 2).unwrap().light_level;
        let deep_soil = grid.get(0, 0, GROUND_LEVEL - 3).unwrap().light_level;

        assert!(
            surface_soil < sky,
            "Surface soil ({surface_soil}) should be dimmer than sky ({sky})"
        );
        assert!(
            deep_soil < surface_soil,
            "Deep soil ({deep_soil}) should be dimmer than surface soil ({surface_soil})"
        );

        // SIM-02 acceptance: surface soil ~200, gradient through layers,
        // at least 3-4 layers of soil have usable light (>=30 for seed growth).
        assert!(
            surface_soil >= 180 && surface_soil <= 220,
            "Surface soil ({surface_soil}) should be ~200"
        );
        assert!(
            one_below >= 140 && one_below <= 190,
            "One layer below ({one_below}) should be ~140-170"
        );
        assert!(
            two_below >= 80 && two_below <= 160,
            "Two layers below ({two_below}) should be ~80-140"
        );
        assert!(
            deep_soil >= 30,
            "Three layers deep ({deep_soil}) should still have usable light (>=30)"
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

        let x = 25;
        let y = 25;
        let z = GROUND_LEVEL;
        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(x, y, z) {
                cell.water_level = 100; // Moist
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

        // Pick a dry soil cell far from water
        let x = 2;
        let y = 2;
        let z = GROUND_LEVEL - 3;
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

        // Use a position where (x+y+z) % 50 == 0 so weathering triggers
        // Find such a position in the soil layer
        let z = GROUND_LEVEL - 1;
        let mut test_x = 0;
        let mut test_y = 0;
        for x in 0..GRID_X {
            for y in 0..GRID_Y {
                if (x + y + z) % 50 == 0 {
                    test_x = x;
                    test_y = y;
                    break;
                }
            }
            if test_x != 0 || test_y != 0 || (0 + 0 + z) % 50 == 0 {
                break;
            }
        }

        {
            let mut grid = world.resource_mut::<VoxelGrid>();
            if let Some(cell) = grid.get_mut(test_x, test_y, z) {
                cell.water_level = 100; // Wet enough for weathering
            }
        }
        let initial_rock;
        let initial_clay;
        {
            let mut soil = world.resource_mut::<SoilGrid>();
            let comp = soil.get_mut(test_x, test_y, z).unwrap();
            comp.rock = 100;
            comp.clay = 50;
            initial_rock = comp.rock;
            initial_clay = comp.clay;
        }

        // Run many ticks - weathering happens every tick at matching positions
        for _ in 0..100 {
            crate::tick(&mut world, &mut schedule);
        }

        let soil = world.resource::<SoilGrid>();
        let comp = soil.get(test_x, test_y, z).unwrap();
        assert!(
            comp.rock < initial_rock,
            "Rock should decrease via weathering at ({test_x},{test_y},{z}): initial={initial_rock}, final={}",
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
        // Topsoil should be loam-ish (generated from depth)
        let comp = soil.get(30, 10, GROUND_LEVEL).unwrap();
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
}

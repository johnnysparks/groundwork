use bevy_ecs::system::ResMut;

use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z};
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
/// Uses nutrient_level as a growth counter: increments by 5 each tick
/// when conditions are met, converts to Root at 200.
pub fn seed_growth(mut grid: ResMut<VoxelGrid>) {
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
                    if let Some(cell) = grid.get_mut(x, y, z) {
                        cell.nutrient_level = cell.nutrient_level.saturating_add(5);
                        if cell.nutrient_level >= 200 {
                            cell.set_material(Material::Root);
                        }
                    }
                }
            }
        }
    }
}

/// Soil absorbs water from adjacent Water voxels.
pub fn soil_absorption(mut grid: ResMut<VoxelGrid>) {
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

                // Check neighbors for water.
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
                    if snapshot[nidx].0 == Material::Water && snapshot[nidx].1 > 0 {
                        let transfer = 2u8;
                        if let Some(soil) = grid.get_mut(x, y, z) {
                            soil.water_level = soil.water_level.saturating_add(transfer);
                        }
                    }
                }
            }
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
}

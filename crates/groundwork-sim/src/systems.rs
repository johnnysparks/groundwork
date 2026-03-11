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
                                cell.material = Material::Water;
                            }
                            if let Some(cell) = grid.get_mut(x, y, z) {
                                cell.water_level = cell.water_level.saturating_sub(transfer);
                                if cell.water_level == 0 && cell.material == Material::Water {
                                    cell.material = Material::Air;
                                }
                            }
                            continue;
                        }
                    }
                }

                // Can't flow down — spread laterally.
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
                        if let Some(cell) = grid.get_mut(nx, ny, z) {
                            cell.water_level = cell.water_level.saturating_add(transfer);
                            cell.material = Material::Water;
                        }
                        if let Some(cell) = grid.get_mut(x, y, z) {
                            cell.water_level = cell.water_level.saturating_sub(transfer);
                            if cell.water_level == 0 && cell.material == Material::Water {
                                cell.material = Material::Air;
                            }
                        }
                    }
                }
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
                    cell.light_level = light;
                    match cell.material {
                        Material::Air => {
                            // Slight attenuation through air.
                            light = light.saturating_sub(2);
                        }
                        Material::Water => {
                            light = light.saturating_sub(15);
                        }
                        Material::Soil => {
                            light = light.saturating_sub(60);
                        }
                        Material::Root => {
                            light = light.saturating_sub(40);
                        }
                        Material::Stone => {
                            light = 0;
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

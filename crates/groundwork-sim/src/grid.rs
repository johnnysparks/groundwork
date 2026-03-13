use bevy_ecs::prelude::Resource;

use crate::voxel::{Material, Voxel};

pub const GRID_X: usize = 120;
pub const GRID_Y: usize = 120;
pub const GRID_Z: usize = 60;

/// Z level where underground meets the surface (~15m real depth).
pub const GROUND_LEVEL: usize = 30;

/// The voxel grid is the central data structure of the simulation.
/// Flat array indexed by x + y*GRID_X + z*GRID_X*GRID_Y.
/// Z=0 is deepest underground, Z=GROUND_LEVEL is the surface, Z=GRID_Z-1 is sky.
#[derive(Resource)]
pub struct VoxelGrid {
    cells: Vec<Voxel>,
}

impl VoxelGrid {
    /// Create a new grid with default terrain: stone at the bottom,
    /// soil in the middle, air above ground, and a small water spring.
    pub fn from_cells(cells: Vec<Voxel>) -> Self {
        assert_eq!(cells.len(), GRID_X * GRID_Y * GRID_Z);
        Self { cells }
    }

    /// Height map value for position (x, y). Returns a surface elevation
    /// ranging from ~13 to ~17, creating rolling hills.
    pub fn surface_height(x: usize, y: usize) -> usize {
        // Two overlapping sine waves for gentle, natural-looking hills.
        // Frequencies normalized to grid size; amplitudes in meters converted to voxels.
        use crate::scale::VOXEL_SIZE_M;
        let fx = x as f64;
        let fy = y as f64;
        let freq_x1 = 7.2 / GRID_X as f64;
        let freq_y1 = 6.0 / GRID_Y as f64;
        let freq_x2 = 4.2 / GRID_X as f64;
        let freq_y2 = 5.4 / GRID_Y as f64;
        let phase_x = 20.0 / 60.0 * GRID_X as f64;
        let phase_y = 15.0 / 60.0 * GRID_Y as f64;
        // Amplitudes: 1.5m and 0.8m of terrain variation
        let amp1 = 1.5 / VOXEL_SIZE_M;
        let amp2 = 0.8 / VOXEL_SIZE_M;
        let h1 = (fx * freq_x1).sin() * (fy * freq_y1).sin() * amp1;
        let h2 = ((fx + phase_x) * freq_x2).cos() * ((fy + phase_y) * freq_y2).cos() * amp2;
        let base = GROUND_LEVEL as f64 + h1 + h2;
        // Clamp to ±2m from GROUND_LEVEL
        let range = (2.0 / VOXEL_SIZE_M) as usize;
        (base.round() as usize).clamp(GROUND_LEVEL - range, GROUND_LEVEL + range)
    }

    /// Whether (x, y) is part of the stream bed. The stream runs from the
    /// spring (center) toward the southeast edge, ~2-3 voxels wide.
    pub fn is_stream(x: usize, y: usize) -> bool {
        // Stream flows from grid center toward the SE edge.
        let cx = GRID_X / 2;
        let cy = GRID_Y / 2;
        if x < cx || y < cy {
            return false;
        }
        let dx = x as f64 - cx as f64;
        let dy = y as f64 - cy as f64;
        let dist = (dx - dy).abs() / 1.414;
        // Stream width ~1.2m, exclusion zone ~2m (in voxel units)
        let width = 1.2 / crate::scale::VOXEL_SIZE_M;
        let exclusion = 2.0 / crate::scale::VOXEL_SIZE_M;
        dist <= width && (dx + dy) > exclusion
    }

    /// Whether (x, y, z) is a stone outcrop. Creates 3-4 rocky clusters
    /// that poke through the surface near edges.
    fn is_stone_outcrop(x: usize, y: usize, z: usize) -> bool {
        let surface = Self::surface_height(x, y);
        let spread = crate::scale::meters_to_voxels(1.0).max(1);
        if z > surface + spread || z + spread < surface {
            return false;
        }
        // Cluster positions as fractions of grid dimensions.
        // (8/60, 12/60), (50/60, 8/60), (12/60, 48/60)
        // Radii in meters-squared, converted to voxels-squared
        let r_scale = 1.0 / (crate::scale::VOXEL_SIZE_M * crate::scale::VOXEL_SIZE_M);
        let clusters: [(f64, f64, f64); 3] = [
            (8.0 / 60.0, 12.0 / 60.0, 10.0 * r_scale),
            (50.0 / 60.0, 8.0 / 60.0, 8.0 * r_scale),
            (12.0 / 60.0, 48.0 / 60.0, 12.0 * r_scale),
        ];
        for (fx, fy, r_sq) in clusters {
            let cx = (fx * GRID_X as f64).round() as isize;
            let cy = (fy * GRID_Y as f64).round() as isize;
            let d = ((x as isize - cx) * (x as isize - cx)
                   + (y as isize - cy) * (y as isize - cy)) as f64;
            if d < r_sq { return true; }
        }
        false
    }

    pub fn new() -> Self {
        let mut cells = vec![Voxel::default(); GRID_X * GRID_Y * GRID_Z];

        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let surface = Self::surface_height(x, y);

                for z in 0..GRID_Z {
                    let idx = Self::index(x, y, z);

                    let stone_top = crate::scale::meters_to_voxels(5.0);
                    if z < stone_top {
                        cells[idx].material = Material::Stone;
                    } else if z <= surface {
                        cells[idx].material = Material::Soil;
                    } else {
                        cells[idx].material = Material::Air;
                    }
                }

                // Stone outcrops: replace soil/air with stone
                let out_spread = crate::scale::meters_to_voxels(1.0).max(1);
                for z in (surface.saturating_sub(out_spread))..=(surface + out_spread).min(GRID_Z - 1) {
                    if Self::is_stone_outcrop(x, y, z) {
                        let idx = Self::index(x, y, z);
                        cells[idx].material = Material::Stone;
                    }
                }

                // Stream bed: carve 1 level below surface, fill with water
                if Self::is_stream(x, y) {
                    // Carve the stream bed 1 below surface
                    let stream_z = surface;
                    let idx = Self::index(x, y, stream_z);
                    cells[idx].material = Material::Water;
                    cells[idx].water_level = 200;
                    // Air above the stream
                    if stream_z + 1 < GRID_Z {
                        let idx_above = Self::index(x, y, stream_z + 1);
                        cells[idx_above].material = Material::Air;
                    }
                }
            }
        }

        // Water spring at center — a ~4m×4m pool at surface level
        let cx = GRID_X / 2;
        let cy = GRID_Y / 2;
        let pool_half = crate::scale::meters_to_voxels(2.0);
        let spring_z = Self::surface_height(cx, cy);
        for dy in (cy.saturating_sub(pool_half))..=(cy + pool_half - 1) {
            for dx in (cx.saturating_sub(pool_half))..=(cx + pool_half - 1) {
                let sz = Self::surface_height(dx, dy).max(spring_z);
                // Water sits at the spring level
                let wz = sz;
                if wz < GRID_Z {
                    let idx = Self::index(dx, dy, wz);
                    cells[idx].material = Material::Water;
                    cells[idx].water_level = 255;
                    // Ensure air above
                    if wz + 1 < GRID_Z {
                        let idx_above = Self::index(dx, dy, wz + 1);
                        cells[idx_above].material = Material::Air;
                    }
                }
            }
        }

        Self { cells }
    }

    #[inline]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * GRID_X + z * GRID_X * GRID_Y
    }

    #[inline]
    pub fn in_bounds(x: usize, y: usize, z: usize) -> bool {
        x < GRID_X && y < GRID_Y && z < GRID_Z
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&Voxel> {
        if Self::in_bounds(x, y, z) {
            Some(&self.cells[Self::index(x, y, z)])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut Voxel> {
        if Self::in_bounds(x, y, z) {
            let idx = Self::index(x, y, z);
            Some(&mut self.cells[idx])
        } else {
            None
        }
    }

    /// Find the lowest Air cell at (x, y) starting from z, dropping through
    /// Air until resting above something solid (or the grid floor).
    /// Returns z unchanged if (x, y, z) is already solid or at the bottom.
    pub fn find_landing_z(&self, x: usize, y: usize, z: usize) -> usize {
        let mut z = z;
        while z > 0 {
            if let Some(below) = self.get(x, y, z - 1) {
                if below.material == Material::Air {
                    z -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        z
    }

    /// Raw access to the backing storage (useful for double-buffer swaps).
    pub fn cells(&self) -> &[Voxel] {
        &self.cells
    }

    pub fn cells_mut(&mut self) -> &mut [Voxel] {
        &mut self.cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_cells_correct_size() {
        let cells = vec![Voxel::default(); GRID_X * GRID_Y * GRID_Z];
        let grid = VoxelGrid::from_cells(cells);
        assert_eq!(grid.cells().len(), GRID_X * GRID_Y * GRID_Z);
    }

    #[test]
    #[should_panic]
    fn from_cells_wrong_size_panics() {
        let cells = vec![Voxel::default(); 10];
        let _ = VoxelGrid::from_cells(cells);
    }

    #[test]
    fn default_terrain_layers() {
        let grid = VoxelGrid::new();
        // Z=0 is stone
        assert_eq!(grid.get(0, 0, 0).unwrap().material, Material::Stone);
        // Mid-depth is soil (z = GROUND_LEVEL/2, always below any surface)
        let mid = GROUND_LEVEL / 2;
        assert_eq!(grid.get(0, 0, mid).unwrap().material, Material::Soil);
        // Surface height varies — check that surface is soil and above is air
        let sh = VoxelGrid::surface_height(0, 0);
        assert_eq!(grid.get(0, 0, sh).unwrap().material, Material::Soil);
        // Well above surface should be air
        let above = (sh + 4).min(GRID_Z - 1);
        assert_eq!(grid.get(0, 0, above).unwrap().material, Material::Air);
        // Water spring at center sits at surface height
        let cx = GRID_X / 2;
        let cy = GRID_Y / 2;
        let spring_z = VoxelGrid::surface_height(cx, cy);
        assert_eq!(grid.get(cx, cy, spring_z).unwrap().material, Material::Water);
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let grid = VoxelGrid::new();
        assert!(grid.get(GRID_X, 0, 0).is_none());
        assert!(grid.get(0, GRID_Y, 0).is_none());
        assert!(grid.get(0, 0, GRID_Z).is_none());
    }
}

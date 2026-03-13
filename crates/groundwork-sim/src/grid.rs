use bevy_ecs::prelude::Resource;

use crate::voxel::{Material, Voxel};

pub const GRID_X: usize = 60;
pub const GRID_Y: usize = 60;
pub const GRID_Z: usize = 30;

/// Z level where underground meets the surface.
pub const GROUND_LEVEL: usize = 15;

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
        // Frequencies normalized to grid size so terrain scales with grid dimensions.
        let fx = x as f64;
        let fy = y as f64;
        let freq_x1 = 7.2 / GRID_X as f64;  // 0.12 at GRID_X=60
        let freq_y1 = 6.0 / GRID_Y as f64;  // 0.10 at GRID_Y=60
        let freq_x2 = 4.2 / GRID_X as f64;  // 0.07 at GRID_X=60
        let freq_y2 = 5.4 / GRID_Y as f64;  // 0.09 at GRID_Y=60
        let phase_x = 20.0 / 60.0 * GRID_X as f64;
        let phase_y = 15.0 / 60.0 * GRID_Y as f64;
        let h1 = (fx * freq_x1).sin() * (fy * freq_y1).sin() * 1.5;
        let h2 = ((fx + phase_x) * freq_x2).cos() * ((fy + phase_y) * freq_y2).cos() * 0.8;
        let base = GROUND_LEVEL as f64 + h1 + h2;
        (base.round() as usize).clamp(GROUND_LEVEL - 2, GROUND_LEVEL + 2)
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
        dist <= 1.2 && (dx + dy) > 2.0
    }

    /// Whether (x, y, z) is a stone outcrop. Creates 3-4 rocky clusters
    /// that poke through the surface near edges.
    fn is_stone_outcrop(x: usize, y: usize, z: usize) -> bool {
        let surface = Self::surface_height(x, y);
        if z > surface + 1 || z < surface - 1 {
            return false;
        }
        // Cluster positions as fractions of grid dimensions.
        // (8/60, 12/60), (50/60, 8/60), (12/60, 48/60)
        let clusters: [(f64, f64, f64); 3] = [
            (8.0 / 60.0, 12.0 / 60.0, 10.0),
            (50.0 / 60.0, 8.0 / 60.0, 8.0),
            (12.0 / 60.0, 48.0 / 60.0, 12.0),
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

                    if z < 5 {
                        cells[idx].material = Material::Stone;
                    } else if z <= surface {
                        cells[idx].material = Material::Soil;
                    } else {
                        cells[idx].material = Material::Air;
                    }
                }

                // Stone outcrops: replace soil/air with stone
                for z in (surface.saturating_sub(1))..=(surface + 1).min(GRID_Z - 1) {
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

        // Water spring at center — a 4x4 pool at surface level
        let cx = GRID_X / 2;
        let cy = GRID_Y / 2;
        let spring_z = Self::surface_height(cx, cy);
        for dy in (cy - 2)..=(cy + 1) {
            for dx in (cx - 2)..=(cx + 1) {
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
        // Z=10 is soil (always below any surface height)
        assert_eq!(grid.get(0, 0, 10).unwrap().material, Material::Soil);
        // Surface height varies — check that surface is soil and above is air
        let sh = VoxelGrid::surface_height(0, 0);
        assert_eq!(grid.get(0, 0, sh).unwrap().material, Material::Soil);
        assert_eq!(grid.get(0, 0, sh + 2).unwrap().material, Material::Air);
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

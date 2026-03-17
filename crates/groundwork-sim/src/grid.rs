use bevy_ecs::prelude::Resource;

use crate::voxel::{Material, Voxel};

pub const GRID_X: usize = 80;
pub const GRID_Y: usize = 80;
pub const GRID_Z: usize = 100;

/// Z level where underground meets the surface (~2m real depth).
/// 60 voxels above ground = 3m of sky and canopy.
pub const GROUND_LEVEL: usize = 40;

/// Pond center position — spring-fed pool at the top of the slope.
/// Water spring system replenishes water here every tick.
pub const POND_X: usize = GRID_X / 2;
pub const POND_Y: usize = GRID_Y / 5; // near the high (north) end of the slope

/// The voxel grid is the central data structure of the simulation.
/// Flat array indexed by x + y*GRID_X + z*GRID_X*GRID_Y.
/// Z=0 is deepest underground, Z=GROUND_LEVEL is the surface, Z=GRID_Z-1 is sky.
#[derive(Resource)]
pub struct VoxelGrid {
    cells: Vec<Voxel>,
}

impl Default for VoxelGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl VoxelGrid {
    /// Create a new grid with default terrain: stone at the bottom,
    /// soil in the middle, air above ground, and a small water spring.
    pub fn from_cells(cells: Vec<Voxel>) -> Self {
        assert_eq!(cells.len(), GRID_X * GRID_Y * GRID_Z);
        Self { cells }
    }

    /// Height map value for position (x, y). Returns a surface elevation
    /// creating a gentle slope — high at the north (y=0) end, low at the south.
    /// The pond sits at the high end; water flows downhill naturally.
    pub fn surface_height(x: usize, y: usize) -> usize {
        // Gentle linear slope: ~4 voxels drop from y=0 to y=GRID_Y.
        // Slight x-axis waviness for visual interest (not perfectly flat side-to-side).
        let slope = 4.0 * (1.0 - y as f64 / GRID_Y as f64);
        let waviness = 0.5 * (x as f64 * std::f64::consts::PI / GRID_X as f64).sin();
        let base = GROUND_LEVEL as f64 + slope + waviness;
        (base.round() as usize).clamp(GROUND_LEVEL, GROUND_LEVEL + 5)
    }

    /// Whether (x, y) is part of the pond. Roughly circular pool at the
    /// high end of the slope, radius ~6 voxels.
    fn is_pond(x: usize, y: usize) -> bool {
        let dx = x as f64 - POND_X as f64;
        let dy = y as f64 - POND_Y as f64;
        let pond_radius_sq = 6.0_f64 * 6.0; // ~6 voxels radius = 0.3m
        (dx * dx + dy * dy) <= pond_radius_sq
    }

    pub fn new() -> Self {
        let mut cells = vec![Voxel::default(); GRID_X * GRID_Y * GRID_Z];

        // Find the pond surface level (lowest surface_height within the pond)
        // so water sits at a flat level and the basin is carved into the slope.
        let pond_z = {
            let mut min_z = GRID_Z;
            for py in 0..GRID_Y {
                for px in 0..GRID_X {
                    if Self::is_pond(px, py) {
                        min_z = min_z.min(Self::surface_height(px, py));
                    }
                }
            }
            min_z.saturating_sub(1) // pond floor is 1 below the lowest surface in the basin
        };

        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let surface = Self::surface_height(x, y);

                for z in 0..GRID_Z {
                    let idx = Self::index(x, y, z);

                    let stone_top = crate::scale::meters_to_voxels(1.0);
                    if z < stone_top {
                        cells[idx].material = Material::Stone;
                    } else if z <= surface {
                        cells[idx].material = Material::Soil;
                    } else {
                        cells[idx].material = Material::Air;
                    }
                }

                // Pond: carve basin and fill with water
                if Self::is_pond(x, y) {
                    // Carve down to pond floor, fill with water at pond_z
                    for z in pond_z..=surface {
                        let idx = Self::index(x, y, z);
                        cells[idx].material = Material::Water;
                        cells[idx].water_level = 255;
                    }
                    // Ensure air above the pond surface
                    if surface + 1 < GRID_Z {
                        let idx_above = Self::index(x, y, surface + 1);
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
        // Midway between stone top and surface is soil
        let stone_top = crate::scale::meters_to_voxels(1.0);
        let soil_mid = (stone_top + GROUND_LEVEL) / 2;
        assert_eq!(grid.get(0, 0, soil_mid).unwrap().material, Material::Soil);
        // Surface height varies — check that surface is soil and above is air
        let sh = VoxelGrid::surface_height(0, 0);
        assert_eq!(grid.get(0, 0, sh).unwrap().material, Material::Soil);
        // Well above surface should be air
        let above = (sh + 4).min(GRID_Z - 1);
        assert_eq!(grid.get(0, 0, above).unwrap().material, Material::Air);
        // Pond at POND_X, POND_Y sits at surface height
        let pond_z = VoxelGrid::surface_height(POND_X, POND_Y);
        assert_eq!(
            grid.get(POND_X, POND_Y, pond_z).unwrap().material,
            Material::Water
        );
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let grid = VoxelGrid::new();
        assert!(grid.get(GRID_X, 0, 0).is_none());
        assert!(grid.get(0, GRID_Y, 0).is_none());
        assert!(grid.get(0, 0, GRID_Z).is_none());
    }
}

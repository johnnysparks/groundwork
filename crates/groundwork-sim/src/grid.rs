use bevy_ecs::prelude::Resource;

use crate::voxel::{Material, Voxel};

pub const GRID_X: usize = 60;
pub const GRID_Y: usize = 60;
pub const GRID_Z: usize = 30;

/// Z level where underground meets the surface.
pub const GROUND_LEVEL: usize = 15;

/// Surface elevation at (x, y). Returns the highest Z that is solid ground.
/// Uses layered sine waves to create gentle hills and shallow depressions
/// centred around GROUND_LEVEL (range roughly 13–17).
fn surface_height(x: usize, y: usize) -> usize {
    let fx = x as f64;
    let fy = y as f64;
    // Two octaves of sine waves at different frequencies and axes.
    let h = (fx * 0.10).sin() * 1.5
        + (fy * 0.13).sin() * 1.5
        + (fx * 0.22 + fy * 0.18).sin() * 0.8;
    let raw = GROUND_LEVEL as f64 + h;
    // Clamp to keep at least 2 stone layers below and 2 air layers above.
    (raw.round() as usize).clamp(4, GRID_Z - 3)
}

/// Top of the stone layer at (x, y). Returns the highest Z that is stone.
/// Usually sits around z=4 but sharp peaks can reach z=13+ to create
/// stone outcrops visible at the surface.
fn stone_height(x: usize, y: usize) -> usize {
    let fx = x as f64;
    let fy = y as f64;
    // Broad base around z=4.
    let base = (fx * 0.25 + 1.0).sin() * 1.2 + (fy * 0.30 + 2.0).sin() * 1.0;
    // Narrow spike that pushes stone up through soil in a few spots.
    let spike_val = ((fx - 15.0) * 0.35).cos() * ((fy - 40.0) * 0.35).cos();
    let spike = if spike_val > 0.7 {
        (spike_val - 0.7) * 30.0 // sharp bump up to ~9 extra layers
    } else {
        0.0
    };
    let raw = 4.0 + base + spike;
    (raw.round() as usize).clamp(2, GRID_Z - 3)
}

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

    pub fn new() -> Self {
        let mut cells = vec![Voxel::default(); GRID_X * GRID_Y * GRID_Z];

        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let surface = surface_height(x, y);
                let stone_top = stone_height(x, y);

                for z in 0..GRID_Z {
                    let idx = Self::index(x, y, z);
                    cells[idx].material = if z <= stone_top {
                        Material::Stone
                    } else if z <= surface {
                        Material::Soil
                    } else {
                        Material::Air
                    };
                }
            }
        }

        // Place a small water spring near the center, one layer above the
        // local surface. This gives something to watch flow on the first tick.
        for dy in 28..=31 {
            for dx in 28..=31 {
                let sz = surface_height(dx, dy) + 1;
                if sz < GRID_Z {
                    let idx = Self::index(dx, dy, sz);
                    cells[idx].material = Material::Water;
                    cells[idx].water_level = 255;
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
        // Z=0 is always stone (well below any stone_height minimum)
        assert_eq!(grid.get(0, 0, 0).unwrap().material, Material::Stone);
        // Z=GRID_Z-1 is always air (well above any surface)
        assert_eq!(
            grid.get(0, 0, GRID_Z - 1).unwrap().material,
            Material::Air
        );
        // Water spring present near center, one above local surface
        let spring_z = surface_height(30, 30) + 1;
        assert_eq!(
            grid.get(30, 30, spring_z).unwrap().material,
            Material::Water
        );
    }

    #[test]
    fn terrain_has_elevation_variation() {
        // Surface height should not be identical everywhere.
        let mut heights = std::collections::HashSet::new();
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                heights.insert(surface_height(x, y));
            }
        }
        assert!(
            heights.len() > 1,
            "terrain should have varied elevation, got a single height"
        );
    }

    #[test]
    fn stone_outcrops_exist() {
        // At least one column should have stone reaching the surface.
        let grid = VoxelGrid::new();
        let mut found = false;
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let sh = stone_height(x, y);
                let surf = surface_height(x, y);
                if sh >= surf {
                    // The surface voxel itself should be stone.
                    assert_eq!(grid.get(x, y, surf).unwrap().material, Material::Stone);
                    found = true;
                }
            }
        }
        assert!(found, "expected at least one stone outcrop");
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let grid = VoxelGrid::new();
        assert!(grid.get(GRID_X, 0, 0).is_none());
        assert!(grid.get(0, GRID_Y, 0).is_none());
        assert!(grid.get(0, 0, GRID_Z).is_none());
    }
}

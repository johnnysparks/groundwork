use bevy_ecs::prelude::Resource;

use crate::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z};
use crate::scale::meters_to_voxels;
use crate::voxel::Material;

/// Per-cell soil composition data. Stored in a parallel grid alongside VoxelGrid.
/// Only meaningful for cells where `material == Material::Soil`.
///
/// All fields are 0-255 proportions. Together they determine derived properties
/// like drainage rate, water retention, and nutrient capacity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct SoilComposition {
    /// Coarse particles. High sand = fast drainage, low nutrient retention.
    pub sand: u8,
    /// Fine particles. High clay = slow drainage, high water retention.
    pub clay: u8,
    /// Decomposed organic material. Increases near roots/decay over time.
    pub organic: u8,
    /// Stone fragments. Inherited from nearby stone, decreases via weathering.
    pub rock: u8,
    /// Soil pH mapped to 3.0-9.0: 0=pH3.0, 128=pH6.0, 255=pH9.0.
    pub ph: u8,
    /// Microbial activity. Grows with moisture + organic matter.
    pub bacteria: u8,
}

impl SoilComposition {
    /// Drainage rate: how quickly water passes through (0-255).
    /// Sandy/rocky soil drains fast; clay holds water.
    pub fn drainage_rate(&self) -> u8 {
        // Weighted: sand dominates, rock contributes, clay reduces
        let raw = (self.sand as u16 * 2 + self.rock as u16) / 3;
        raw.min(255) as u8
    }

    /// Water retention: how much water soil holds against gravity (0-255).
    /// Clay and organic matter hold water.
    pub fn water_retention(&self) -> u8 {
        let raw = (self.clay as u16 * 2 + self.organic as u16) / 3;
        raw.min(255) as u8
    }

    /// Nutrient capacity: how well soil supports plant growth (0-255).
    /// Clay, organic matter, and bacteria all contribute.
    pub fn nutrient_capacity(&self) -> u8 {
        let raw = (self.clay as u16 + self.organic as u16 * 2 + self.bacteria as u16) / 4;
        raw.min(255) as u8
    }

    /// Whether soil is compacted (high clay, low organic). Blocks root growth.
    pub fn is_compacted(&self) -> bool {
        self.clay > 200 && self.organic < 30
    }

    /// pH as a float in range 3.0 - 9.0.
    pub fn ph_value(&self) -> f32 {
        3.0 + (self.ph as f32 / 255.0) * 6.0
    }

    /// Human-readable soil type name based on dominant composition.
    pub fn type_name(&self) -> &'static str {
        if self.rock > 150 {
            "rocky"
        } else if self.organic > 150 {
            "peaty"
        } else if self.sand > 150 {
            "sandy"
        } else if self.clay > 150 {
            "clay"
        } else {
            "loam"
        }
    }

    // --- Preset constructors ---

    pub fn rocky() -> Self {
        Self {
            sand: 40,
            clay: 20,
            organic: 10,
            rock: 200,
            ph: 128,
            bacteria: 5,
        }
    }

    pub fn clay() -> Self {
        Self {
            sand: 30,
            clay: 200,
            organic: 20,
            rock: 30,
            ph: 110,
            bacteria: 20,
        }
    }

    pub fn sandy() -> Self {
        Self {
            sand: 200,
            clay: 30,
            organic: 15,
            rock: 40,
            ph: 135,
            bacteria: 15,
        }
    }

    pub fn loam() -> Self {
        Self {
            sand: 100,
            clay: 80,
            organic: 80,
            rock: 30,
            ph: 128,
            bacteria: 60,
        }
    }

    pub fn peat() -> Self {
        Self {
            sand: 40,
            clay: 40,
            organic: 220,
            rock: 10,
            ph: 80,
            bacteria: 100,
        }
    }
}

/// Parallel grid storing soil composition for every cell.
/// Same dimensions and indexing as VoxelGrid.
/// Data is only meaningful where the corresponding voxel is `Material::Soil`.
#[derive(Resource)]
pub struct SoilGrid {
    cells: Vec<SoilComposition>,
}

impl Default for SoilGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl SoilGrid {
    /// Create soil grid with depth-based composition matching dynamic terrain.
    /// Uses VoxelGrid::surface_height() for per-column ground level.
    /// - Deep (near stone): rocky soil
    /// - Subsoil: clay-heavy
    /// - Transition: mixed clay/loam
    /// - Topsoil: loam
    /// - Near water spring/stream: peat patches
    /// - Grid edges: sandy
    pub fn new() -> Self {
        let mut cells = vec![SoilComposition::default(); GRID_X * GRID_Y * GRID_Z];

        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let surface = VoxelGrid::surface_height(x, y);

                for z in 0..GRID_Z {
                    let idx = VoxelGrid::index(x, y, z);
                    // Only populate soil layers (z=stone_top through surface)
                    let stone_top = meters_to_voxels(1.0);
                    if z < stone_top || z > surface {
                        continue;
                    }

                    // Depth below surface determines composition
                    let depth_below = surface.saturating_sub(z);
                    let deep_thresh = meters_to_voxels(1.4);
                    let comp =
                        if depth_below >= surface.saturating_sub(deep_thresh) || z <= deep_thresh {
                            // Deep soil near stone: rocky
                            SoilComposition::rocky()
                        } else if depth_below >= meters_to_voxels(0.5) {
                            // Subsoil: clay-heavy
                            SoilComposition::clay()
                        } else if depth_below >= meters_to_voxels(0.3) {
                            // Transition: blend of clay and loam
                            SoilComposition {
                                sand: 70,
                                clay: 130,
                                organic: 50,
                                rock: 30,
                                ph: 120,
                                bacteria: 35,
                            }
                        } else {
                            // Topsoil: loam
                            SoilComposition::loam()
                        };

                    cells[idx] = comp;
                }
            }
        }

        // Peat patches near the water spring (28-31, 28-31) and along stream
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let surface = VoxelGrid::surface_height(x, y);
                // Near spring or stream
                let cx = GRID_X / 2;
                let cy = GRID_Y / 2;
                let spring_range = meters_to_voxels(0.3);
                let near_spring = x >= cx.saturating_sub(spring_range)
                    && x < cx + spring_range
                    && y >= cy.saturating_sub(spring_range)
                    && y < cy + spring_range;
                let near_stream = VoxelGrid::is_stream(x, y)
                    || (x > 0 && VoxelGrid::is_stream(x - 1, y))
                    || (y > 0 && VoxelGrid::is_stream(x, y - 1))
                    || VoxelGrid::is_stream(x + 1, y)
                    || VoxelGrid::is_stream(x, y + 1);

                if !near_spring && !near_stream {
                    continue;
                }

                for z in meters_to_voxels(1.0)..=surface {
                    let idx = VoxelGrid::index(x, y, z);
                    let depth_below = surface.saturating_sub(z);
                    let peat_strength = if depth_below <= meters_to_voxels(0.4) {
                        200u16
                    } else if depth_below <= meters_to_voxels(0.5) {
                        100
                    } else {
                        50
                    };
                    // Weaker peat along stream than at spring
                    let peat_strength = if near_spring {
                        peat_strength
                    } else {
                        peat_strength / 2
                    };
                    let base = &cells[idx];
                    cells[idx] = SoilComposition {
                        sand: ((base.sand as u16 * (255 - peat_strength) + 40 * peat_strength)
                            / 255) as u8,
                        clay: ((base.clay as u16 * (255 - peat_strength) + 40 * peat_strength)
                            / 255) as u8,
                        organic: ((base.organic as u16 * (255 - peat_strength)
                            + 220 * peat_strength)
                            / 255) as u8,
                        rock: ((base.rock as u16 * (255 - peat_strength) + 10 * peat_strength)
                            / 255) as u8,
                        ph: ((base.ph as u16 * (255 - peat_strength) + 80 * peat_strength) / 255)
                            as u8,
                        bacteria: ((base.bacteria as u16 * (255 - peat_strength)
                            + 100 * peat_strength)
                            / 255) as u8,
                    };
                }
            }
        }

        // Sandy patches at grid edges
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let surface = VoxelGrid::surface_height(x, y);
                let edge_dist = x.min(y).min(GRID_X - 1 - x).min(GRID_Y - 1 - y);
                let edge_band = meters_to_voxels(0.4);
                if edge_dist >= edge_band {
                    continue;
                }
                for z in surface.saturating_sub(meters_to_voxels(0.4))..=surface {
                    if z < meters_to_voxels(1.0) {
                        continue;
                    }
                    let idx = VoxelGrid::index(x, y, z);
                    let sand_strength = ((edge_band - edge_dist) as u16 * 32).min(255);
                    let base = &cells[idx];
                    cells[idx] = SoilComposition {
                        sand: ((base.sand as u16 * (255 - sand_strength) + 200 * sand_strength)
                            / 255) as u8,
                        clay: ((base.clay as u16 * (255 - sand_strength) + 30 * sand_strength)
                            / 255) as u8,
                        organic: ((base.organic as u16 * (255 - sand_strength)
                            + 15 * sand_strength)
                            / 255) as u8,
                        rock: ((base.rock as u16 * (255 - sand_strength) + 40 * sand_strength)
                            / 255) as u8,
                        ph: ((base.ph as u16 * (255 - sand_strength) + 135 * sand_strength) / 255)
                            as u8,
                        bacteria: ((base.bacteria as u16 * (255 - sand_strength)
                            + 15 * sand_strength)
                            / 255) as u8,
                    };
                }
            }
        }

        Self { cells }
    }

    /// Create soil grid from raw cells (for save/load).
    pub fn from_cells(cells: Vec<SoilComposition>) -> Self {
        assert_eq!(cells.len(), GRID_X * GRID_Y * GRID_Z);
        Self { cells }
    }

    /// Generate default soil composition based on a voxel grid's current materials.
    /// Used when loading old save files that don't have soil data.
    pub fn from_voxel_grid(grid: &VoxelGrid) -> Self {
        let mut soil = Self::new();
        // For non-soil voxels, zero out the soil data
        for z in 0..GRID_Z {
            for y in 0..GRID_Y {
                for x in 0..GRID_X {
                    let idx = VoxelGrid::index(x, y, z);
                    if let Some(v) = grid.get(x, y, z) {
                        if v.material != Material::Soil {
                            soil.cells[idx] = SoilComposition::default();
                        }
                    }
                }
            }
        }
        soil
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&SoilComposition> {
        if VoxelGrid::in_bounds(x, y, z) {
            Some(&self.cells[VoxelGrid::index(x, y, z)])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut SoilComposition> {
        if VoxelGrid::in_bounds(x, y, z) {
            let idx = VoxelGrid::index(x, y, z);
            Some(&mut self.cells[idx])
        } else {
            None
        }
    }

    pub fn cells(&self) -> &[SoilComposition] {
        &self.cells
    }

    pub fn cells_mut(&mut self) -> &mut [SoilComposition] {
        &mut self.cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_drainage_rates() {
        // Sandy soil should drain fastest
        assert!(SoilComposition::sandy().drainage_rate() > SoilComposition::clay().drainage_rate());
        // Rocky soil drains fast too
        assert!(SoilComposition::rocky().drainage_rate() > SoilComposition::loam().drainage_rate());
    }

    #[test]
    fn preset_water_retention() {
        // Clay retains most water
        assert!(
            SoilComposition::clay().water_retention() > SoilComposition::sandy().water_retention()
        );
        // Peat also retains well
        assert!(
            SoilComposition::peat().water_retention() > SoilComposition::sandy().water_retention()
        );
    }

    #[test]
    fn preset_nutrient_capacity() {
        // Peat has highest nutrients (organic + bacteria)
        assert!(
            SoilComposition::peat().nutrient_capacity()
                > SoilComposition::rocky().nutrient_capacity()
        );
        // Loam is well-balanced
        assert!(
            SoilComposition::loam().nutrient_capacity()
                > SoilComposition::sandy().nutrient_capacity()
        );
    }

    #[test]
    fn compaction_detection() {
        assert!(SoilComposition {
            sand: 20,
            clay: 210,
            organic: 10,
            rock: 20,
            ph: 128,
            bacteria: 5
        }
        .is_compacted());
        assert!(!SoilComposition::loam().is_compacted());
        assert!(!SoilComposition::peat().is_compacted());
    }

    #[test]
    fn ph_value_range() {
        assert!(
            (SoilComposition {
                ph: 0,
                ..Default::default()
            }
            .ph_value()
                - 3.0)
                .abs()
                < 0.01
        );
        assert!(
            (SoilComposition {
                ph: 128,
                ..Default::default()
            }
            .ph_value()
                - 6.0)
                .abs()
                < 0.1
        );
        assert!(
            (SoilComposition {
                ph: 255,
                ..Default::default()
            }
            .ph_value()
                - 9.0)
                .abs()
                < 0.01
        );
    }

    #[test]
    fn type_names() {
        assert_eq!(SoilComposition::rocky().type_name(), "rocky");
        assert_eq!(SoilComposition::clay().type_name(), "clay");
        assert_eq!(SoilComposition::sandy().type_name(), "sandy");
        assert_eq!(SoilComposition::peat().type_name(), "peaty");
        assert_eq!(SoilComposition::loam().type_name(), "loam");
    }

    #[test]
    fn soil_grid_new_has_depth_layers() {
        let soil = SoilGrid::new();
        // Position must be far from edges (edge_band = 8), outside peat zone
        // (spring_range = 6 from center=40, so peat is [34..45]),
        // and outside stream (requires x>=40 AND y>=40).
        // (15, 15) works: edge_dist=15 >= 8, not in peat zone, not in stream.
        let tx = 15;
        let ty = 15;
        let surface = VoxelGrid::surface_height(tx, ty);
        // Deep layer (near stone) should be rocky
        let deep_z = meters_to_voxels(1.0) + 1; // just above stone
        let deep = soil.get(tx, ty, deep_z).unwrap();
        assert_eq!(deep.type_name(), "rocky");
        // Subsoil should be clay (0.5+ meters below surface)
        let sub_z = surface.saturating_sub(meters_to_voxels(0.6));
        if sub_z > meters_to_voxels(1.4) {
            // only test if there's room
            let sub = soil.get(tx, ty, sub_z).unwrap();
            assert_eq!(sub.type_name(), "clay");
        }
        // Topsoil (top layers) should be loam
        let top = soil.get(tx, ty, surface).unwrap();
        assert_eq!(top.type_name(), "loam");
    }

    #[test]
    fn soil_grid_peat_near_spring() {
        let soil = SoilGrid::new();
        // Near water spring at surface should have high organic
        let sx = GRID_X / 2 - 1;
        let sy = GRID_Y / 2 - 1;
        let surface = VoxelGrid::surface_height(sx, sy);
        let near_spring = soil.get(sx, sy, surface).unwrap();
        assert!(
            near_spring.organic > 100,
            "Near-spring topsoil should be organic-rich, got {}",
            near_spring.organic
        );
    }

    #[test]
    fn soil_grid_sandy_edges() {
        let soil = SoilGrid::new();
        // Corner of grid at surface should be sandy
        let surface = VoxelGrid::surface_height(0, 0);
        let edge = soil.get(0, 0, surface).unwrap();
        assert!(
            edge.sand > 100,
            "Edge topsoil should be sandy, got sand={}",
            edge.sand
        );
    }
}

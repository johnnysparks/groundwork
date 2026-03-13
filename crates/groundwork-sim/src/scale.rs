/// Side length of one voxel in meters. Default 1.0m preserves current behavior.
/// Changing this scales all physics rates and species dimensions proportionally.
pub const VOXEL_SIZE_M: f64 = 1.0;

pub use crate::grid::{GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};

/// Convert meters to voxel count (rounds to nearest integer).
#[inline]
pub fn meters_to_voxels(meters: f64) -> usize {
    (meters / VOXEL_SIZE_M).round().max(0.0) as usize
}

/// Convert meters to voxel count as f64 (for fractional values).
#[inline]
pub fn meters_to_voxels_f64(meters: f64) -> f64 {
    meters / VOXEL_SIZE_M
}

/// Convert voxel count to meters.
#[inline]
pub fn voxels_to_meters(voxels: usize) -> f64 {
    voxels as f64 * VOXEL_SIZE_M
}

/// Convert a fraction (0.0..1.0) of GRID_X to an absolute X voxel coordinate.
#[inline]
pub fn grid_frac_x(frac: f64) -> usize {
    ((frac * GRID_X as f64).round() as usize).min(GRID_X - 1)
}

/// Convert a fraction (0.0..1.0) of GRID_Y to an absolute Y voxel coordinate.
#[inline]
pub fn grid_frac_y(frac: f64) -> usize {
    ((frac * GRID_Y as f64).round() as usize).min(GRID_Y - 1)
}

/// Center X of the grid in voxel coordinates.
#[inline]
pub fn grid_center_x() -> usize {
    GRID_X / 2
}

/// Center Y of the grid in voxel coordinates.
#[inline]
pub fn grid_center_y() -> usize {
    GRID_Y / 2
}

/// Scale factor for light attenuation per voxel layer.
/// Smaller voxels = each layer attenuates less.
#[inline]
pub fn scale_attenuation(base: u8) -> u8 {
    let scaled = base as f64 * VOXEL_SIZE_M;
    (scaled.round() as u8).max(if base > 0 { 1 } else { 0 })
}

/// Scale a u8 transfer amount by voxel size.
#[inline]
pub fn scale_transfer(base: u8) -> u8 {
    let scaled = base as f64 * VOXEL_SIZE_M;
    (scaled.round() as u8).max(if base > 0 { 1 } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_at_default_scale() {
        assert_eq!(meters_to_voxels(1.0), 1);
        assert_eq!(meters_to_voxels(8.0), 8);
        assert_eq!(meters_to_voxels(0.0), 0);
        assert_eq!(voxels_to_meters(1), 1.0);
        assert_eq!(voxels_to_meters(5), 5.0);
    }

    #[test]
    fn grid_center() {
        assert_eq!(grid_center_x(), GRID_X / 2);
        assert_eq!(grid_center_y(), GRID_Y / 2);
    }

    #[test]
    fn grid_frac_center() {
        assert_eq!(grid_frac_x(0.5), GRID_X / 2);
        assert_eq!(grid_frac_y(0.5), GRID_Y / 2);
    }

    #[test]
    fn scale_noop_at_1m() {
        assert_eq!(scale_attenuation(30), 30);
        assert_eq!(scale_transfer(32), 32);
        assert_eq!(scale_attenuation(0), 0);
    }

    #[test]
    fn round_trip() {
        assert_eq!(meters_to_voxels(voxels_to_meters(5)), 5);
        assert_eq!(meters_to_voxels(voxels_to_meters(0)), 0);
    }
}

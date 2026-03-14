//! Scenario: camera exploration — above and below ground.
//!
//! Tests the signature feature: continuous above/below ground camera.
//! Verifies that exploring underground reveals roots, soil, and water
//! that aren't visible from the surface.

use groundwork_sim::grid::{GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};

use crate::evaluator::{CameraOrbited, CameraWentUnderground, Custom, NoCrash, Verdict};
use crate::scenario::Scenario;

/// Plant a tree, then explore it from multiple camera angles and depths.
/// Verifies the player can see roots underground and canopy above.
pub fn explore_above_and_below() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;
    let gl = GROUND_LEVEL as f64;

    Scenario::new("explore_above_and_below")
        .description("Plant a tree, orbit around it, cut underground to see roots")
        .camera_reset()
        .checkpoint("surface_view")
        // Plant near the spring
        .plant("oak", cx - 2, cy, seed_z)
        .tick(150)
        .checkpoint("tree_grown")
        // Orbit to see the tree from different angles
        .orbit(0.0, 60.0)
        .checkpoint("front_view")
        .orbit(90.0, 60.0)
        .checkpoint("side_view")
        .orbit(180.0, 45.0)
        .checkpoint("back_low")
        .orbit(270.0, 80.0)
        .checkpoint("top_down")
        // Zoom in to see detail
        .zoom(2.5)
        .pan(cx as f64 - 2.0, cy as f64, gl)
        .checkpoint("zoomed_in")
        // Go underground — the signature feature
        .cutaway(gl)
        .checkpoint("at_surface")
        .cutaway(gl - 5.0)
        .checkpoint("shallow_underground")
        .cutaway(gl - 10.0)
        .checkpoint("deep_underground")
        // Look at roots from below
        .orbit(45.0, 30.0)
        .checkpoint("underground_angled")
        // Come back up
        .cutaway(GRID_Z as f64)
        .camera_reset()
        .checkpoint("back_to_surface")
        .status()
        .eval(NoCrash)
        .eval(CameraOrbited)
        .eval(CameraWentUnderground::new())
        .eval(CameraWentUnderground::below(gl - 5.0))
        // Verify the camera traversed a meaningful range
        .eval(Custom {
            name: "camera_full_journey".into(),
            f: Box::new(|trace| {
                let cutaway_values: Vec<f64> = trace
                    .steps
                    .iter()
                    .map(|s| s.oracle.camera.cutaway_z)
                    .collect();
                let min = cutaway_values.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = cutaway_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let range = max - min;
                let passed = range >= 15.0; // Should span at least 15 Z-levels
                Verdict {
                    evaluator: "camera_full_journey".into(),
                    passed,
                    reason: format!("cutaway range: {min:.0}..{max:.0} ({range:.0} levels)"),
                    score: Some((range / 40.0).min(1.0)),
                }
            }),
        })
        .build()
}

/// Test that the camera works smoothly at extreme positions.
pub fn camera_edge_cases() -> Scenario {
    Scenario::new("camera_edge_cases")
        .description("Test camera at extreme zoom, angles, and positions")
        .zoom(0.3)
        .checkpoint("zoomed_out")
        .zoom(4.0)
        .checkpoint("zoomed_in")
        .orbit(0.0, 85.0)
        .checkpoint("top_down")
        .orbit(0.0, 11.0)
        .checkpoint("near_horizontal")
        .pan(0.0, 0.0, GROUND_LEVEL as f64)
        .checkpoint("corner")
        .pan(GRID_X as f64, GRID_Y as f64, GROUND_LEVEL as f64)
        .checkpoint("far_corner")
        .cutaway(0.0)
        .checkpoint("deepest")
        .camera_reset()
        .checkpoint("reset")
        .eval(NoCrash)
        .eval(CameraWentUnderground::below(5.0))
        .build()
}

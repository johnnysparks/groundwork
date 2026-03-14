//! Task setter — generate investigation scenarios from failure patterns.
//!
//! When clustering reveals a failure pattern, the task setter generates
//! targeted scenarios that probe the failure boundary, helping devs
//! pinpoint exactly where the bug is.

use crate::clustering::FailureCluster;
use crate::evaluator::{MaterialGrew, MaterialMinimum, NoCrash};
use crate::scenario::Scenario;

/// Generate investigation scenarios from failure clusters.
///
/// Each cluster produces one or more targeted scenarios that probe
/// the specific failure. Results tell the dev exactly where the bug is.
pub fn generate_investigation_scenarios(clusters: &[FailureCluster]) -> Vec<Scenario> {
    let mut scenarios = Vec::new();

    for cluster in clusters {
        let sig = &cluster.signature;

        if sig.contains("no_growth") {
            scenarios.extend(investigate_no_growth());
        } else if sig.contains("insufficient_water") {
            scenarios.extend(investigate_water());
        } else if sig.contains("camera_issue") {
            scenarios.extend(investigate_camera());
        } else if sig.contains("material_did_not_grow") {
            scenarios.extend(investigate_material_growth(sig));
        }
    }

    scenarios
}

/// Generate scenarios probing seed growth at various distances from water.
fn investigate_no_growth() -> Vec<Scenario> {
    let cx = 40usize;
    let cy = 40usize;
    let seed_z = 50usize;
    let water_z = 45usize;

    vec![
        // Test: seed directly adjacent to water
        Scenario::new("investigate_seed_adjacent_water")
            .description("Seed placed 1 voxel from water — should grow if water proximity works")
            .fill("water", cx, cy, water_z, cx, cy, water_z)
            .plant("oak", cx + 1, cy, seed_z)
            .tick(150)
            .status()
            .eval(NoCrash)
            .eval(MaterialGrew::new("plant"))
            .build(),
        // Test: seed far from water
        Scenario::new("investigate_seed_far_from_water")
            .description("Seed placed 20 voxels from water — may not grow if out of range")
            .fill("water", cx, cy, water_z, cx, cy, water_z)
            .plant("oak", cx + 20, cy, seed_z)
            .tick(150)
            .status()
            .eval(NoCrash)
            .build(),
        // Test: seed on different species
        Scenario::new("investigate_species_growth_rates")
            .description("Plant all species types near water and see which grow")
            .fill("water", cx - 3, cy - 3, water_z, cx + 3, cy + 3, water_z)
            .plant("oak", cx - 5, cy, seed_z)
            .plant("fern", cx + 5, cy, seed_z)
            .plant("moss", cx, cy - 5, seed_z)
            .plant("wildflower", cx, cy + 5, seed_z)
            .plant("grass", cx - 5, cy - 5, seed_z)
            .plant("pine", cx + 5, cy + 5, seed_z)
            .tick(200)
            .status()
            .eval(NoCrash)
            .eval(MaterialMinimum::new("plant", 1))
            .build(),
        // Test: many ticks
        Scenario::new("investigate_long_growth")
            .description("Seed with water, 500 ticks — does enough time help?")
            .fill("water", cx - 3, cy - 3, water_z, cx + 3, cy + 3, water_z)
            .plant("oak", cx - 4, cy, seed_z)
            .tick(500)
            .status()
            .eval(NoCrash)
            .eval(MaterialMinimum::new("plant", 1))
            .build(),
    ]
}

/// Generate scenarios probing water persistence.
fn investigate_water() -> Vec<Scenario> {
    let cx = 40usize;
    let cy = 40usize;
    let water_z = 45usize;

    vec![
        // Small water amount
        Scenario::new("investigate_water_small")
            .description("Place 1 water voxel and tick — does it persist?")
            .place("water", cx, cy, water_z)
            .tick(50)
            .status()
            .eval(NoCrash)
            .eval(MaterialMinimum::new("water", 1))
            .build(),
        // Larger water area
        Scenario::new("investigate_water_basin")
            .description("Fill 5x5 water area and tick 100 — does it persist?")
            .fill("water", cx - 2, cy - 2, water_z, cx + 2, cy + 2, water_z)
            .tick(100)
            .status()
            .eval(NoCrash)
            .eval(MaterialMinimum::new("water", 5))
            .build(),
    ]
}

/// Generate scenarios probing camera behavior.
fn investigate_camera() -> Vec<Scenario> {
    vec![
        Scenario::new("investigate_camera_orbit")
            .description("Orbit through all quadrants")
            .action(crate::action::Action::CameraOrbit {
                theta_deg: 0.0,
                phi_deg: 45.0,
            })
            .action(crate::action::Action::CameraOrbit {
                theta_deg: 90.0,
                phi_deg: 45.0,
            })
            .action(crate::action::Action::CameraOrbit {
                theta_deg: 180.0,
                phi_deg: 45.0,
            })
            .action(crate::action::Action::CameraOrbit {
                theta_deg: 270.0,
                phi_deg: 45.0,
            })
            .eval(NoCrash)
            .eval(crate::evaluator::CameraOrbited)
            .build(),
        Scenario::new("investigate_camera_underground")
            .description("Cutaway progressively deeper")
            .action(crate::action::Action::CameraCutaway { z: 35.0 })
            .action(crate::action::Action::CameraCutaway { z: 25.0 })
            .action(crate::action::Action::CameraCutaway { z: 10.0 })
            .eval(NoCrash)
            .eval(crate::evaluator::CameraWentUnderground::new())
            .build(),
    ]
}

/// Generate scenarios for specific material growth failures.
fn investigate_material_growth(signature: &str) -> Vec<Scenario> {
    let cx = 40usize;
    let cy = 40usize;
    let seed_z = 50usize;
    let water_z = 45usize;

    // Extract material name from signature like "material_did_not_grow:trunk grew"
    let material = signature
        .split(':')
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .unwrap_or("plant");

    vec![Scenario::new(&format!("investigate_{material}_growth"))
        .description(&format!(
            "Targeted test for {material} growth with ideal conditions"
        ))
        .fill(
            "water",
            cx - 5,
            cy - 5,
            water_z,
            cx + 5,
            cy + 5,
            water_z,
        )
        .plant("oak", cx, cy, seed_z)
        .tick(300)
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new(material, 1))
        .build()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner;

    #[test]
    fn generates_scenarios_for_no_growth() {
        let cluster = FailureCluster {
            signature: "no_growth_after_100_ticks".into(),
            count: 1,
            affected_scenarios: vec!["test".into()],
            common_conditions: "no growth".into(),
            suggested_investigation: "check seeds".into(),
        };

        let scenarios = generate_investigation_scenarios(&[cluster]);
        assert!(scenarios.len() >= 3);

        // All should be runnable
        for scenario in &scenarios {
            let result = runner::run(scenario);
            // NoCrash should always pass
            assert!(
                result.verdicts.iter().any(|v| v.evaluator == "no_crash" && v.passed),
                "Scenario '{}' crashed",
                scenario.name
            );
        }
    }

    #[test]
    fn generates_scenarios_for_water_issues() {
        let cluster = FailureCluster {
            signature: "insufficient_water_after_50_ticks".into(),
            count: 1,
            affected_scenarios: vec!["test".into()],
            common_conditions: "water gone".into(),
            suggested_investigation: "check springs".into(),
        };

        let scenarios = generate_investigation_scenarios(&[cluster]);
        assert!(!scenarios.is_empty());
    }
}

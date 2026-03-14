//! Scenario: multi-species ecosystem interactions.
//!
//! Tests that planting multiple species near each other produces
//! emergent ecological behavior (e.g., water competition, light shading).

use groundwork_sim::grid::{GRID_X, GRID_Y, GROUND_LEVEL};

use crate::evaluator::{Custom, MaterialMinimum, NoCrash, Verdict};
use crate::scenario::Scenario;

/// Plant a small garden with diverse species and verify ecosystem health.
pub fn diverse_garden() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("diverse_garden")
        .description("Plant a diverse garden near water and verify ecosystem emerges")
        .checkpoint("start")
        // Add extra water for a larger growing area
        .fill("water", cx - 5, cy - 5, GROUND_LEVEL + 5, cx + 5, cy + 5, GROUND_LEVEL + 5)
        // Plant a variety of species
        .plant("oak", cx - 4, cy - 2, seed_z)
        .plant("birch", cx + 4, cy - 2, seed_z)
        .plant("fern", cx - 2, cy + 2, seed_z)
        .plant("moss", cx, cy, seed_z)
        .plant("grass", cx + 2, cy + 2, seed_z)
        .plant("wildflower", cx - 2, cy - 2, seed_z)
        // Let the ecosystem develop
        .tick(300)
        .checkpoint("end")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("plant", 3))
        // Custom: verify the garden isn't dead
        .eval(Custom {
            name: "garden_not_dead".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "garden_not_dead".into(),
                        passed: false,
                        reason: "no trace".into(),
                        score: Some(0.0),
                    };
                };
                let counts = &oracle.material_counts;
                let has_life = counts.total_plant() > 0;
                let has_water = counts.water > 0 || counts.wet_soil > 0;
                let passed = has_life || has_water;
                Verdict {
                    evaluator: "garden_not_dead".into(),
                    passed,
                    reason: format!(
                        "plant={}, water={}, wet_soil={}",
                        counts.total_plant(),
                        counts.water,
                        counts.wet_soil
                    ),
                    score: Some(if passed { 1.0 } else { 0.0 }),
                }
            }),
        })
        .build()
}

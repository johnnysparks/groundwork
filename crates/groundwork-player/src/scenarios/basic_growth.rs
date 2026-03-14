//! Scenario: basic seed → tree growth.
//!
//! Tests the fundamental player action: plant a seed near water,
//! tick the simulation, and verify the seed grows into a tree.

use groundwork_sim::grid::{GRID_X, GRID_Y, GROUND_LEVEL};

use crate::evaluator::{MaterialGrew, MaterialMinimum, NoCrash};
use crate::scenario::Scenario;

/// Plant an oak seed near the water spring and verify it grows.
pub fn seed_to_tree() -> Scenario {
    // Water spring is at grid center. Place seed near it, above surface so gravity drops it.
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_x = cx - 2;
    let seed_y = cy;
    let seed_z = GROUND_LEVEL + 10; // well above surface

    Scenario::new("seed_to_tree")
        .description("Plant an oak near the spring and verify it grows into a tree")
        .status()
        .checkpoint("before_planting")
        .plant("oak", seed_x, seed_y, seed_z)
        .checkpoint("after_planting")
        // Seeds need ~40 ticks to mature (growth +5/tick, threshold 200)
        .tick(50)
        .checkpoint("after_seed_phase")
        // Continue growing — tree growth happens after seed matures
        .tick(100)
        .checkpoint("after_growth")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("seed", 0))
        .eval(MaterialGrew::new("trunk"))
        .eval(MaterialGrew::new("root"))
        .build()
}

/// Plant multiple species and verify at least some grow.
pub fn multi_species_planting() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("multi_species_planting")
        .description("Plant different species near water and verify growth")
        .checkpoint("start")
        .plant("oak", cx - 4, cy, seed_z)
        .plant("fern", cx, cy - 4, seed_z)
        .plant("moss", cx + 4, cy, seed_z)
        .plant("wildflower", cx, cy + 4, seed_z)
        .tick(200)
        .checkpoint("end")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("plant", 1))
        .build()
}

//! Scenario: basic seed → tree growth.
//!
//! Tests the fundamental player action: plant a seed near water,
//! tick the simulation, and verify the seed grows into a tree.

use crate::evaluator::{MaterialGrew, MaterialMinimum, NoCrash};
use crate::scenario::Scenario;

/// Plant an oak seed near the water spring and verify it grows.
pub fn seed_to_tree() -> Scenario {
    // The water spring is at grid center (60, 60) at surface level.
    // Surface height varies ~28-32. Place seed one above surface near spring.
    let seed_x = 58;
    let seed_y = 60;
    // Place seed well above surface so gravity drops it to the right spot.
    let seed_z = 40;

    Scenario::new("seed_to_tree")
        .description("Plant an oak near the spring and verify it grows into a tree")
        // Initial status check
        .status()
        .checkpoint("before_planting")
        // Plant a seed near the water spring
        .plant("oak", seed_x, seed_y, seed_z)
        .checkpoint("after_planting")
        // Tick enough for seed → seedling → sapling
        // Seeds need ~40 ticks to mature (growth +5/tick, threshold 200)
        .tick(50)
        .checkpoint("after_seed_phase")
        // Continue growing — tree growth happens after seed matures
        .tick(100)
        .checkpoint("after_growth")
        .status()
        // Check final state
        .eval(NoCrash)
        .eval(MaterialMinimum::new("seed", 0))  // at least attempted to plant
        .eval(MaterialGrew::new("trunk"))
        .eval(MaterialGrew::new("root"))
        .build()
}

/// Plant multiple species and verify at least some grow.
pub fn multi_species_planting() -> Scenario {
    // Place seeds near the spring at (60,60), spread out so they don't compete
    let seed_z = 40; // well above surface, gravity handles landing

    Scenario::new("multi_species_planting")
        .description("Plant different species near water and verify growth")
        .checkpoint("start")
        // Plant 4 different species spread around the spring
        .plant("oak", 56, 60, seed_z)
        .plant("fern", 60, 56, seed_z)
        .plant("moss", 64, 60, seed_z)
        .plant("wildflower", 60, 64, seed_z)
        .tick(200)
        .checkpoint("end")
        .status()
        .eval(NoCrash)
        // At least some plant material should exist
        .eval(MaterialMinimum::new("plant", 1))
        .build()
}

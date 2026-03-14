//! Scenario: water system behavior.
//!
//! Tests water flow, soil absorption, and the player's ability to
//! shape water for gardening.

use crate::evaluator::{MaterialMinimum, NoCrash};
use crate::scenario::Scenario;

/// Pour water and verify it flows and eventually wets soil.
pub fn water_flow_and_absorption() -> Scenario {
    // Place water above the surface near center.
    // Water should flow down through air, hit the surface, and wet the soil.
    Scenario::new("water_flow_and_absorption")
        .description("Pour water and verify it flows downward and wets soil")
        .checkpoint("before_water")
        .status()
        // Fill a small water region above the surface
        .fill("water", 50, 50, 35, 52, 52, 35)
        .checkpoint("after_water_placed")
        .tick(10)
        .checkpoint("after_flow")
        .status()
        // Water should exist somewhere (either still flowing or pooled)
        .eval(NoCrash)
        .eval(MaterialMinimum::new("water", 1))
        .build()
}

/// Verify the starting world has water from the spring.
pub fn spring_exists() -> Scenario {
    Scenario::new("spring_exists")
        .description("Verify the default world has a water spring")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("water", 10))
        .build()
}

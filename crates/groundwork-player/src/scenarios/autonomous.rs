//! Autonomous session scenarios — test planner-driven gameplay.
//!
//! These scenarios use the planner interface rather than scripted action lists.
//! The scripted planner runs deterministically in CI; the LLM planner runs
//! manually with `cargo test -- --ignored`.

use crate::evaluator::{
    CameraOrbited, CameraWentUnderground, Evaluator, MaterialMinimum, NoCrash,
};

/// Configuration for an autonomous scenario.
pub struct AutonomousScenario {
    /// Scenario name.
    pub name: String,
    /// Maximum steps before stopping.
    pub max_steps: usize,
    /// Voxel coordinates to probe in oracle snapshots.
    pub probes: Vec<(usize, usize, usize)>,
    /// Evaluators to judge the trace.
    pub evaluators: Vec<Box<dyn Evaluator>>,
}

/// A scripted planner session: plants a garden, explores camera, ticks for growth.
pub fn scripted_garden_session() -> AutonomousScenario {
    AutonomousScenario {
        name: "scripted_garden_session".into(),
        max_steps: 50,
        probes: vec![(40, 40, 41), (39, 40, 41), (41, 40, 41), (40, 40, 38)],
        evaluators: vec![
            Box::new(NoCrash),
            Box::new(MaterialMinimum::new("plant", 1)),
            Box::new(CameraOrbited),
            Box::new(CameraWentUnderground::new()),
        ],
    }
}

/// A longer session with growth expectations.
pub fn scripted_growth_session() -> AutonomousScenario {
    AutonomousScenario {
        name: "scripted_growth_session".into(),
        max_steps: 100,
        probes: vec![(39, 40, 41), (40, 40, 38)],
        evaluators: vec![
            Box::new(NoCrash),
            Box::new(MaterialMinimum::new("plant", 1)),
            Box::new(MaterialMinimum::new("water", 1)),
        ],
    }
}

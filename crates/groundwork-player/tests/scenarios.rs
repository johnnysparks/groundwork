//! Integration tests: run all built-in scenarios and assert they pass.
//!
//! These tests are deterministic — same code → same results every time.
//! Failures here indicate a regression in the simulation or a scenario
//! that needs updating.

use groundwork_player::evaluator::{
    CameraOrbited, CameraWentUnderground, MaterialMinimum, NoCrash,
};
use groundwork_player::planner_scripted::ScriptedPlanner;
use groundwork_player::runner;
use groundwork_player::scenarios::{basic_growth, camera_exploration, ecosystem, water_system};

fn run_and_assert(scenario_fn: fn() -> groundwork_player::scenario::Scenario) {
    let scenario = scenario_fn();
    let name = scenario.name.clone();
    let result = runner::run(&scenario);
    eprintln!("{}", result.report());
    assert!(
        result.all_passed(),
        "Scenario '{name}' failed:\n{}",
        result
            .verdicts
            .iter()
            .filter(|v| !v.passed)
            .map(|v| format!("  {v}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn scenario_seed_to_tree() {
    run_and_assert(basic_growth::seed_to_tree);
}

#[test]
fn scenario_multi_species_planting() {
    run_and_assert(basic_growth::multi_species_planting);
}

#[test]
fn scenario_spring_exists() {
    run_and_assert(water_system::spring_exists);
}

#[test]
fn scenario_water_flow_and_absorption() {
    run_and_assert(water_system::water_flow_and_absorption);
}

#[test]
fn scenario_diverse_garden() {
    run_and_assert(ecosystem::diverse_garden);
}

#[test]
fn scenario_explore_above_and_below() {
    run_and_assert(camera_exploration::explore_above_and_below);
}

#[test]
fn scenario_camera_edge_cases() {
    run_and_assert(camera_exploration::camera_edge_cases);
}

/// Verify that traces are serializable (important for artifact storage).
#[test]
fn trace_serialization() {
    let scenario = water_system::spring_exists();
    let result = runner::run(&scenario);
    let json = result.trace.to_json();
    assert!(json.contains("spring_exists"));
    assert!(json.contains("material_counts"));
    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("trace should be valid JSON");
    assert!(parsed.is_object());
}

// ---------------------------------------------------------------------------
// Autonomous runner + scripted planner tests
// ---------------------------------------------------------------------------

/// Scripted planner can play a full session and produce a valid trace.
#[test]
fn autonomous_scripted_planner_session() {
    let mut planner = ScriptedPlanner::default();
    let cx = groundwork_sim::grid::GRID_X / 2;
    let cy = groundwork_sim::grid::GRID_Y / 2;
    let gl = groundwork_sim::grid::GROUND_LEVEL;

    let probes: Vec<(usize, usize, usize)> = vec![
        (cx, cy, gl),         // center surface
        (cx - 5, cy, gl),     // oak planting spot
        (cx, cy, gl - 3),     // underground
    ];

    let evaluators: Vec<Box<dyn groundwork_player::evaluator::Evaluator>> = vec![
        Box::new(NoCrash),
        Box::new(MaterialMinimum::new("plant", 1)),
        Box::new(CameraOrbited),
        Box::new(CameraWentUnderground::new()),
    ];

    let result = runner::run_autonomous(&mut planner, &probes, &evaluators, 100);

    eprintln!("{}", result.report());
    assert!(!result.trace.steps.is_empty(), "trace should have steps");
    assert!(
        result.trace.steps.len() > 10,
        "scripted planner should produce multiple steps, got {}",
        result.trace.steps.len()
    );

    // Verify all evaluators pass
    for v in &result.verdicts {
        assert!(v.passed, "Evaluator failed: {v}");
    }
}

/// Scripted planner respects max_steps limit.
#[test]
fn autonomous_max_steps_limit() {
    let mut planner = ScriptedPlanner::new(3);
    let result = runner::run_autonomous(&mut planner, &[], &[], 5);
    assert!(
        result.trace.steps.len() <= 5,
        "should respect max_steps, got {} steps",
        result.trace.steps.len()
    );
}

/// Scripted planner is deterministic — two runs produce identical oracle states.
#[test]
fn autonomous_scripted_deterministic() {
    let probes = vec![(40, 40, 40)];

    let mut p1 = ScriptedPlanner::default();
    let mut p2 = ScriptedPlanner::default();

    let r1 = runner::run_autonomous(&mut p1, &probes, &[], 100);
    let r2 = runner::run_autonomous(&mut p2, &probes, &[], 100);

    assert_eq!(r1.trace.steps.len(), r2.trace.steps.len());
    for (i, (s1, s2)) in r1.trace.steps.iter().zip(r2.trace.steps.iter()).enumerate() {
        let o1 = &s1.oracle.material_counts;
        let o2 = &s2.oracle.material_counts;
        assert_eq!(o1.trunk, o2.trunk, "step {i}: trunk diverged");
        assert_eq!(o1.seed, o2.seed, "step {i}: seed diverged");
        assert_eq!(o1.water, o2.water, "step {i}: water diverged");
    }
}

/// Autonomous trace serializes to valid JSON.
#[test]
fn autonomous_trace_serialization() {
    let mut planner = ScriptedPlanner::new(5);
    let result = runner::run_autonomous(&mut planner, &[], &[], 20);
    let json = result.trace.to_json();
    assert!(json.contains("autonomous_scripted"));
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("trace should be valid JSON");
    assert!(parsed.is_object());
    assert!(parsed["steps"].is_array());
}

/// Verify deterministic replay — running the same scenario twice gives identical oracle state.
#[test]
fn deterministic_replay() {
    let s1 = basic_growth::seed_to_tree();
    let s2 = basic_growth::seed_to_tree();
    let r1 = runner::run(&s1);
    let r2 = runner::run(&s2);

    assert_eq!(r1.trace.steps.len(), r2.trace.steps.len());

    for (i, (step1, step2)) in r1.trace.steps.iter().zip(r2.trace.steps.iter()).enumerate() {
        let o1 = &step1.oracle.material_counts;
        let o2 = &step2.oracle.material_counts;
        assert_eq!(o1.trunk, o2.trunk, "step {i}: trunk count diverged");
        assert_eq!(o1.root, o2.root, "step {i}: root count diverged");
        assert_eq!(o1.leaf, o2.leaf, "step {i}: leaf count diverged");
        assert_eq!(o1.water, o2.water, "step {i}: water count diverged");
        assert_eq!(o1.seed, o2.seed, "step {i}: seed count diverged");
    }
}

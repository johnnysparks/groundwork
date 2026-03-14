//! Integration tests: run all built-in scenarios and assert they pass.
//!
//! These tests are deterministic — same code → same results every time.
//! Failures here indicate a regression in the simulation or a scenario
//! that needs updating.

use groundwork_player::runner;
use groundwork_player::scenarios::{basic_growth, ecosystem, water_system};

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

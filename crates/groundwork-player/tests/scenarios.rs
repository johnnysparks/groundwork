//! Integration tests: run all built-in scenarios and assert they pass.
//!
//! These tests are deterministic — same code → same results every time.
//! Failures here indicate a regression in the simulation or a scenario
//! that needs updating.

use groundwork_player::planner_scripted::ScriptedPlanner;
use groundwork_player::runner;
use groundwork_player::scenarios::{
    autonomous, basic_growth, camera_exploration, ecosystem, gameplay, water_system,
};

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

// ---------------------------------------------------------------------------
// Gameplay scenarios — core delight moments
// ---------------------------------------------------------------------------

#[test]
fn scenario_water_table_sculpting() {
    run_and_assert(gameplay::water_table_sculpting);
}

#[test]
fn scenario_first_sprout() {
    run_and_assert(gameplay::first_sprout);
}

#[test]
fn scenario_underground_reveal() {
    run_and_assert(gameplay::underground_reveal);
}

#[test]
fn scenario_root_competition() {
    run_and_assert(gameplay::root_competition);
}

#[test]
fn scenario_seed_dispersal_surprise() {
    run_and_assert(gameplay::seed_dispersal_surprise);
}

#[test]
fn scenario_canopy_shade_garden() {
    run_and_assert(gameplay::canopy_shade_garden);
}

#[test]
#[ignore]
fn scenario_self_pruning_discovery() {
    run_and_assert(gameplay::self_pruning_discovery);
}

#[test]
fn scenario_groundcover_spread() {
    run_and_assert(gameplay::groundcover_spread);
}

#[test]
fn scenario_recovery_after_destruction() {
    run_and_assert(gameplay::recovery_after_destruction);
}

#[test]
fn scenario_idle_garden_changes() {
    run_and_assert(gameplay::idle_garden_changes);
}

#[test]
fn scenario_willow_loves_water() {
    run_and_assert(gameplay::willow_loves_water);
}

// Aspirational scenarios — expected to fail until features are built.
// Run with: cargo test -p groundwork-player -- --ignored

#[test]
fn scenario_nitrogen_handshake() {
    run_and_assert(gameplay::nitrogen_handshake);
}

#[test]
fn scenario_pioneer_succession() {
    run_and_assert(gameplay::pioneer_succession);
}

#[test]
fn scenario_fauna_near_flowers() {
    run_and_assert(gameplay::fauna_near_flowers);
}

#[test]
fn scenario_crowding_thins_forest() {
    run_and_assert(gameplay::crowding_thins_forest);
}

#[test]
fn scenario_diversity_beats_monoculture() {
    run_and_assert(gameplay::diversity_beats_monoculture);
}

#[test]
fn scenario_player_journey_pacing() {
    run_and_assert(gameplay::player_journey_pacing);
}

#[test]
fn scenario_growth_timeline() {
    run_and_assert(gameplay::growth_timeline);
}

#[test]
fn scenario_interaction_chain_depth() {
    run_and_assert(gameplay::interaction_chain_depth);
}

#[test]
fn scenario_water_scarcity_response() {
    run_and_assert(gameplay::water_scarcity_response);
}

#[test]
fn scenario_observation_reward_density() {
    run_and_assert(gameplay::observation_reward_density);
}

#[test]
fn scenario_visual_growth_stages() {
    run_and_assert(gameplay::visual_growth_stages);
}

#[test]
fn scenario_species_feel_different() {
    run_and_assert(gameplay::species_feel_different);
}

// ---------------------------------------------------------------------------

/// Verify that traces are serializable (important for artifact storage).
#[test]
fn trace_serialization() {
    let scenario = water_system::spring_exists();
    let result = runner::run(&scenario);
    let json = result.trace.to_json();
    assert!(json.contains("spring_exists"));
    assert!(json.contains("material_counts"));
    // Verify it's valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("trace should be valid JSON");
    assert!(parsed.is_object());
}

// ---------------------------------------------------------------------------
// Autonomous planner scenarios
// ---------------------------------------------------------------------------

fn run_autonomous_and_assert(scenario_fn: fn() -> autonomous::AutonomousScenario) {
    let scenario = scenario_fn();
    let name = scenario.name.clone();
    let mut planner = ScriptedPlanner::new();
    let result = runner::run_autonomous(
        &mut planner,
        &scenario.probes,
        &scenario.evaluators,
        scenario.max_steps,
    );
    eprintln!("{}", result.report());
    assert!(
        result.all_passed(),
        "Autonomous scenario '{name}' failed:\n{}",
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
fn autonomous_scripted_garden_session() {
    run_autonomous_and_assert(autonomous::scripted_garden_session);
}

#[test]
fn autonomous_scripted_growth_session() {
    run_autonomous_and_assert(autonomous::scripted_growth_session);
}

/// Verify that autonomous traces serialize to valid JSON.
#[test]
fn autonomous_trace_serialization() {
    let scenario = autonomous::scripted_garden_session();
    let mut planner = ScriptedPlanner::new();
    let result = runner::run_autonomous(
        &mut planner,
        &scenario.probes,
        &scenario.evaluators,
        scenario.max_steps,
    );
    let json = result.trace.to_json();
    assert!(json.contains("autonomous"));
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("autonomous trace should be valid JSON");
    assert!(parsed.is_object());
}

/// Verify scripted planner produces identical traces on replay.
#[test]
fn autonomous_deterministic_replay() {
    let scenario = autonomous::scripted_garden_session();
    let mut p1 = ScriptedPlanner::new();
    let mut p2 = ScriptedPlanner::new();

    let r1 = runner::run_autonomous(
        &mut p1,
        &scenario.probes,
        &scenario.evaluators,
        scenario.max_steps,
    );
    let r2 = runner::run_autonomous(
        &mut p2,
        &scenario.probes,
        &scenario.evaluators,
        scenario.max_steps,
    );

    assert_eq!(r1.trace.steps.len(), r2.trace.steps.len());
    for (i, (s1, s2)) in r1.trace.steps.iter().zip(r2.trace.steps.iter()).enumerate() {
        let o1 = &s1.oracle.material_counts;
        let o2 = &s2.oracle.material_counts;
        assert_eq!(o1.seed, o2.seed, "step {i}: seed count diverged");
        assert_eq!(o1.water, o2.water, "step {i}: water count diverged");
        assert_eq!(o1.trunk, o2.trunk, "step {i}: trunk count diverged");
    }
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

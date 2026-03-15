//! Regression generator — turn failed traces into permanent test scenarios.
//!
//! When a failure is found, the trace captures the exact action sequence.
//! This module converts that into a `Scenario` that replays those actions
//! with evaluators that should pass once the bug is fixed.

use std::path::Path;

use crate::action::Action;
use crate::evaluator::Verdict;
use crate::scenario::Scenario;
use crate::trace::Trace;

/// Generate a deterministic scenario from a failed trace.
///
/// Replays the exact same actions and adds evaluators for the expected outcome.
/// The generated scenario fails when the bug reproduces and passes once it's fixed.
pub fn trace_to_regression(
    trace: &Trace,
    failed_verdicts: &[Verdict],
    name: &str,
) -> Scenario {
    let mut builder = Scenario::new(name).description(&format!(
        "Regression test from failed run of '{}'. Failures: {}",
        trace.scenario_name,
        failed_verdicts
            .iter()
            .map(|v| format!("{}: {}", v.evaluator, v.reason))
            .collect::<Vec<_>>()
            .join("; ")
    ));

    // Replay exact action sequence
    for step in &trace.steps {
        builder = builder.action(step.action.clone());
    }

    // Add evaluators that match the original failures.
    // These use MaterialMinimum for growth-related failures and NoCrash as baseline.
    builder = builder.eval(crate::evaluator::NoCrash);

    for verdict in failed_verdicts {
        builder = add_evaluator_from_verdict(builder, verdict);
    }

    builder.build()
}

/// Add an evaluator to the builder based on a failed verdict.
fn add_evaluator_from_verdict(
    builder: crate::scenario::ScenarioBuilder,
    verdict: &Verdict,
) -> crate::scenario::ScenarioBuilder {
    let evaluator = &verdict.evaluator;

    // Parse "material >= N" patterns
    if evaluator.contains(">=") {
        let parts: Vec<&str> = evaluator.split_whitespace().collect();
        if parts.len() >= 3 {
            let material = parts[0];
            if let Ok(min) = parts[2].parse::<u64>() {
                return builder.eval(crate::evaluator::MaterialMinimum::new(material, min));
            }
        }
    }

    // Parse "material grew" patterns
    if evaluator.contains("grew") {
        let material = evaluator.split_whitespace().next().unwrap_or("plant");
        return builder.eval(crate::evaluator::MaterialGrew::new(material));
    }

    // Camera evaluators
    if evaluator.contains("camera_orbited") {
        return builder.eval(crate::evaluator::CameraOrbited);
    }
    if evaluator.contains("camera_went_underground") || evaluator.contains("camera_below") {
        return builder.eval(crate::evaluator::CameraWentUnderground::new());
    }

    builder
}

/// Write a regression scenario as a Rust source file.
///
/// The generated file defines a function that returns a `Scenario` and can
/// be added to the test suite.
pub fn write_regression_file(
    scenario: &Scenario,
    output_path: &Path,
) -> std::io::Result<()> {
    let mut code = String::new();

    code.push_str("//! Auto-generated regression test.\n");
    code.push_str("//!\n");
    code.push_str(&format!(
        "//! Generated from failed run of '{}'.\n",
        scenario.description
    ));
    code.push_str("//! Do not edit — regenerate if the scenario changes.\n\n");

    code.push_str("use crate::evaluator::{NoCrash, MaterialMinimum};\n");
    code.push_str("use crate::scenario::Scenario;\n\n");

    let fn_name = scenario
        .name
        .replace('-', "_")
        .replace(' ', "_")
        .to_ascii_lowercase();

    code.push_str(&format!(
        "/// Regression: {}\n",
        scenario.description.lines().next().unwrap_or("")
    ));
    code.push_str(&format!("pub fn {fn_name}() -> Scenario {{\n"));
    code.push_str(&format!(
        "    Scenario::new({:?})\n",
        scenario.name
    ));
    code.push_str(&format!(
        "        .description({:?})\n",
        scenario.description
    ));

    // Write actions
    for action in &scenario.actions {
        code.push_str(&format_action_builder(action));
    }

    // Always include NoCrash
    code.push_str("        .eval(NoCrash)\n");

    code.push_str("        .build()\n");
    code.push_str("}\n");

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_path, code)
}

fn format_action_builder(action: &Action) -> String {
    match action {
        Action::Tick { n } => format!("        .tick({n})\n"),
        Action::Place {
            tool,
            x,
            y,
            z,
            species,
        } => {
            if let Some(sp) = species {
                format!("        .plant({sp:?}, {x}, {y}, {z})\n")
            } else {
                format!("        .place({tool:?}, {x}, {y}, {z})\n")
            }
        }
        Action::Fill {
            tool,
            x1,
            y1,
            z1,
            x2,
            y2,
            z2,
        } => {
            format!("        .fill({tool:?}, {x1}, {y1}, {z1}, {x2}, {y2}, {z2})\n")
        }
        Action::CameraOrbit { theta_deg, phi_deg } => {
            format!("        .orbit({theta_deg:.1}, {phi_deg:.1})\n")
        }
        Action::CameraPan { x, y, z } => {
            format!("        .action(crate::action::Action::CameraPan {{ x: {x:.1}, y: {y:.1}, z: {z:.1} }})\n")
        }
        Action::CameraZoom { level } => {
            format!("        .zoom({level:.1})\n")
        }
        Action::CameraCutaway { z } => {
            format!("        .cutaway({z:.1})\n")
        }
        Action::CameraReset => "        .camera_reset()\n".into(),
        Action::Inspect { x, y, z } => {
            format!("        .inspect({x}, {y}, {z})\n")
        }
        Action::Status => "        .status()\n".into(),
        Action::View { z } => {
            format!("        .action(crate::action::Action::View {{ z: {z} }})\n")
        }
        Action::Checkpoint { label } => {
            format!("        .checkpoint({label:?})\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner;
    use crate::scenarios::water_system;

    #[test]
    fn trace_to_regression_replays_actions() {
        let scenario = water_system::spring_exists();
        let result = runner::run(&scenario);

        // Create a synthetic failed verdict
        let failed = vec![Verdict {
            evaluator: "plant >= 5".into(),
            passed: false,
            reason: "plant: 0 (need >= 5)".into(),
            score: Some(0.0),
        }];

        let regression = trace_to_regression(&result.trace, &failed, "regression_spring");

        // Same number of actions as original trace steps
        assert_eq!(regression.actions.len(), result.trace.steps.len());
        assert_eq!(regression.name, "regression_spring");
        // Has evaluators (NoCrash + parsed from verdict)
        assert!(regression.evaluators.len() >= 2);
    }

    #[test]
    fn regression_scenario_runs() {
        let scenario = water_system::spring_exists();
        let result = runner::run(&scenario);

        let failed = vec![Verdict {
            evaluator: "water >= 1".into(),
            passed: false,
            reason: "water: 0 (need >= 1)".into(),
            score: Some(0.0),
        }];

        let regression = trace_to_regression(&result.trace, &failed, "regression_water");

        // The regression should actually pass because water >= 1 is satisfied
        let reg_result = runner::run(&regression);
        assert!(reg_result.all_passed());
    }

    #[test]
    fn write_regression_file_creates_valid_rust() {
        let scenario = water_system::spring_exists();
        let result = runner::run(&scenario);

        let failed = vec![Verdict {
            evaluator: "plant >= 1".into(),
            passed: false,
            reason: "plant: 0".into(),
            score: Some(0.0),
        }];

        let regression = trace_to_regression(&result.trace, &failed, "test_regression");

        let tmp = std::env::temp_dir().join("groundwork_regression_test.rs");
        write_regression_file(&regression, &tmp).unwrap();

        let content = std::fs::read_to_string(&tmp).unwrap();
        assert!(content.contains("pub fn test_regression"));
        assert!(content.contains("Scenario::new"));
        assert!(content.contains("NoCrash"));
        std::fs::remove_file(tmp).ok();
    }
}

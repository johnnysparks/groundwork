//! Trace analysis — extract summaries and metrics from run results.
//!
//! Operates on `Trace` and `RunResult` data to produce `RunSummary` structs
//! used by clustering, regression generation, and reporting.

use std::collections::HashMap;

use crate::evaluator::Verdict;
use crate::oracle::MaterialCounts;
use crate::runner::RunResult;
use crate::trace::Trace;

/// Summary of a run for reporting and clustering.
#[derive(Debug, Clone)]
pub struct RunSummary {
    /// Scenario name.
    pub scenario_name: String,
    /// Whether all evaluators passed.
    pub passed: bool,
    /// Individual verdicts.
    pub verdicts: Vec<Verdict>,
    /// Average of all verdict scores (0.0–1.0).
    pub aggregate_score: f64,
    /// Total action count.
    pub step_count: usize,
    /// Final material counts.
    pub final_material_counts: MaterialCounts,
    /// Camera exploration coverage.
    pub camera_coverage: CameraCoverage,
    /// How many times each action type was used.
    pub action_histogram: HashMap<String, usize>,
    /// Plant count at each tick snapshot: (tick, total_plant).
    pub growth_timeline: Vec<(u64, u64)>,
    /// Machine-readable failure descriptors.
    pub failure_signatures: Vec<String>,
}

/// How thoroughly the camera explored the world.
#[derive(Debug, Clone)]
pub struct CameraCoverage {
    /// Min/max azimuth angle.
    pub angle_range: (f64, f64),
    /// Min/max elevation angle.
    pub elevation_range: (f64, f64),
    /// Min/max zoom level.
    pub zoom_range: (f64, f64),
    /// Min/max cutaway Z.
    pub cutaway_range: (f64, f64),
    /// Whether camera went below ground level.
    pub went_underground: bool,
    /// Total number of camera-related actions.
    pub total_camera_actions: usize,
}

/// Summarize a run result into a compact analysis struct.
pub fn summarize(result: &RunResult) -> RunSummary {
    let trace = &result.trace;

    // Aggregate score
    let scores: Vec<f64> = result.verdicts.iter().filter_map(|v| v.score).collect();
    let aggregate_score = if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f64>() / scores.len() as f64
    };

    // Final material counts
    let final_material_counts = trace
        .final_oracle()
        .map(|o| o.material_counts.clone())
        .unwrap_or_default();

    // Action histogram
    let mut action_histogram = HashMap::new();
    for step in &trace.steps {
        let key = action_type_name(&step.action);
        *action_histogram.entry(key).or_insert(0) += 1;
    }

    // Growth timeline
    let growth_timeline = growth_timeline(trace);

    // Camera coverage
    let camera_coverage = compute_camera_coverage(trace);

    // Failure signatures
    let failure_signatures: Vec<String> = result
        .verdicts
        .iter()
        .filter(|v| !v.passed)
        .map(|v| failure_signature(v, trace))
        .collect();

    RunSummary {
        scenario_name: trace.scenario_name.clone(),
        passed: result.all_passed(),
        verdicts: result.verdicts.clone(),
        aggregate_score,
        step_count: trace.steps.len(),
        final_material_counts,
        camera_coverage,
        action_histogram,
        growth_timeline,
        failure_signatures,
    }
}

/// Track plant count over simulation ticks.
pub fn growth_timeline(trace: &Trace) -> Vec<(u64, u64)> {
    let mut timeline = Vec::new();
    let mut last_tick = u64::MAX;

    for step in &trace.steps {
        let tick = step.oracle.tick;
        if tick != last_tick {
            let plant_count = step.oracle.material_counts.total_plant();
            timeline.push((tick, plant_count));
            last_tick = tick;
        }
    }

    timeline
}

/// Generate a machine-readable failure signature for clustering.
pub fn failure_signature(verdict: &Verdict, trace: &Trace) -> String {
    let total_ticks = trace.final_oracle().map(|o| o.tick).unwrap_or(0);
    let final_plants = trace
        .final_oracle()
        .map(|o| o.material_counts.total_plant())
        .unwrap_or(0);

    // Parse evaluator name to determine failure category
    let evaluator = &verdict.evaluator;

    if evaluator.contains(">=") && verdict.reason.contains("plant") && final_plants == 0 {
        return format!("no_growth_after_{total_ticks}_ticks");
    }
    if evaluator.contains(">=") && verdict.reason.contains("water") {
        return format!("insufficient_water_after_{total_ticks}_ticks");
    }
    if evaluator.contains("grew") && !verdict.passed {
        return format!("material_did_not_grow:{evaluator}");
    }
    if evaluator.contains("camera") {
        return format!("camera_issue:{evaluator}");
    }
    if evaluator == "no_crash" {
        return "crash_during_run".to_string();
    }

    // Generic fallback
    format!("failed:{evaluator}")
}

fn compute_camera_coverage(trace: &Trace) -> CameraCoverage {
    let ground_level = groundwork_sim::grid::GROUND_LEVEL as f64;

    if trace.steps.is_empty() {
        return CameraCoverage {
            angle_range: (0.0, 0.0),
            elevation_range: (0.0, 0.0),
            zoom_range: (1.0, 1.0),
            cutaway_range: (0.0, 0.0),
            went_underground: false,
            total_camera_actions: 0,
        };
    }

    let mut min_theta = f64::MAX;
    let mut max_theta = f64::MIN;
    let mut min_phi = f64::MAX;
    let mut max_phi = f64::MIN;
    let mut min_zoom = f64::MAX;
    let mut max_zoom = f64::MIN;
    let mut min_cutaway = f64::MAX;
    let mut max_cutaway = f64::MIN;
    let mut went_underground = false;
    let mut camera_actions = 0usize;

    for step in &trace.steps {
        let cam = &step.oracle.camera;
        min_theta = min_theta.min(cam.theta_deg);
        max_theta = max_theta.max(cam.theta_deg);
        min_phi = min_phi.min(cam.phi_deg);
        max_phi = max_phi.max(cam.phi_deg);
        min_zoom = min_zoom.min(cam.zoom);
        max_zoom = max_zoom.max(cam.zoom);
        min_cutaway = min_cutaway.min(cam.cutaway_z);
        max_cutaway = max_cutaway.max(cam.cutaway_z);

        if cam.cutaway_z <= ground_level {
            went_underground = true;
        }

        if is_camera_action(&step.action) {
            camera_actions += 1;
        }
    }

    CameraCoverage {
        angle_range: (min_theta, max_theta),
        elevation_range: (min_phi, max_phi),
        zoom_range: (min_zoom, max_zoom),
        cutaway_range: (min_cutaway, max_cutaway),
        went_underground,
        total_camera_actions: camera_actions,
    }
}

fn action_type_name(action: &crate::action::Action) -> String {
    use crate::action::Action;
    match action {
        Action::Tick { .. } => "tick".into(),
        Action::Place { .. } => "place".into(),
        Action::Fill { .. } => "fill".into(),
        Action::CameraOrbit { .. } => "camera_orbit".into(),
        Action::CameraPan { .. } => "camera_pan".into(),
        Action::CameraZoom { .. } => "camera_zoom".into(),
        Action::CameraCutaway { .. } => "camera_cutaway".into(),
        Action::CameraReset => "camera_reset".into(),
        Action::Inspect { .. } => "inspect".into(),
        Action::Status => "status".into(),
        Action::View { .. } => "view".into(),
        Action::Screenshot { .. } => "screenshot".into(),
        Action::Checkpoint { .. } => "checkpoint".into(),
    }
}

fn is_camera_action(action: &crate::action::Action) -> bool {
    use crate::action::Action;
    matches!(
        action,
        Action::CameraOrbit { .. }
            | Action::CameraPan { .. }
            | Action::CameraZoom { .. }
            | Action::CameraCutaway { .. }
            | Action::CameraReset
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner;
    use crate::scenarios::water_system;

    #[test]
    fn summarize_produces_valid_summary() {
        let scenario = water_system::spring_exists();
        let result = runner::run(&scenario);
        let summary = summarize(&result);

        assert_eq!(summary.scenario_name, "spring_exists");
        assert!(summary.passed);
        assert!(summary.aggregate_score > 0.0);
        assert!(summary.step_count > 0);
        assert!(summary.failure_signatures.is_empty());
    }

    #[test]
    fn growth_timeline_tracks_changes() {
        let scenario = water_system::spring_exists();
        let result = runner::run(&scenario);
        let timeline = growth_timeline(&result.trace);
        // At minimum we get one entry per unique tick
        assert!(!timeline.is_empty());
    }

    #[test]
    fn failure_signature_categorizes() {
        let verdict = Verdict {
            evaluator: "plant >= 1".into(),
            passed: false,
            reason: "plant: 0 (need >= 1)".into(),
            score: Some(0.0),
        };
        // Create a minimal trace for signature generation
        let scenario = water_system::spring_exists();
        let result = runner::run(&scenario);
        let sig = failure_signature(&verdict, &result.trace);
        assert!(sig.contains("no_growth"));
    }
}

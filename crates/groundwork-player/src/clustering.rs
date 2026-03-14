//! Failure clustering — group similar failures across multiple runs.
//!
//! Identifies patterns in failures so that batch diagnosis can focus
//! on root causes rather than individual symptoms.

use std::collections::HashMap;

use crate::analysis::RunSummary;

/// A cluster of similar failures across runs.
#[derive(Debug, Clone)]
pub struct FailureCluster {
    /// Machine-readable signature (e.g. "no_growth_after_200_ticks").
    pub signature: String,
    /// How many runs exhibited this failure.
    pub count: usize,
    /// Scenario names that produced this failure.
    pub affected_scenarios: Vec<String>,
    /// What these failures have in common.
    pub common_conditions: String,
    /// Suggested investigation direction.
    pub suggested_investigation: String,
}

/// Cluster failures from multiple run summaries.
pub fn cluster_failures(summaries: &[RunSummary]) -> Vec<FailureCluster> {
    // Group by failure signature
    let mut groups: HashMap<String, Vec<&RunSummary>> = HashMap::new();

    for summary in summaries {
        for sig in &summary.failure_signatures {
            groups
                .entry(sig.clone())
                .or_default()
                .push(summary);
        }
    }

    let mut clusters: Vec<FailureCluster> = groups
        .into_iter()
        .map(|(signature, members)| {
            let affected_scenarios: Vec<String> = members
                .iter()
                .map(|s| s.scenario_name.clone())
                .collect();
            let count = members.len();
            let common_conditions = diagnose_common_conditions(&signature, &members);
            let suggested_investigation = suggest_investigation(&signature, &members);

            FailureCluster {
                signature,
                count,
                affected_scenarios,
                common_conditions,
                suggested_investigation,
            }
        })
        .collect();

    // Sort by count descending (most common failures first)
    clusters.sort_by(|a, b| b.count.cmp(&a.count));
    clusters
}

/// Analyze what failing runs have in common.
fn diagnose_common_conditions(signature: &str, members: &[&RunSummary]) -> String {
    if signature.starts_with("no_growth") {
        let has_water: Vec<bool> = members
            .iter()
            .map(|s| s.final_material_counts.water > 0)
            .collect();
        let has_seeds: Vec<bool> = members
            .iter()
            .map(|s| s.final_material_counts.seed > 0)
            .collect();
        let all_no_water = has_water.iter().all(|w| !w);
        let all_no_seeds = has_seeds.iter().all(|s| !s);

        if all_no_water {
            return "No water present in any failing run — seeds may lack moisture".into();
        }
        if all_no_seeds {
            return "No seeds remaining — seeds may have been placed incorrectly or consumed without growing".into();
        }
        return "Seeds and water present but no plant growth occurred".into();
    }

    if signature.starts_with("insufficient_water") {
        return "Water count below expected threshold — possible spring/flow issue".into();
    }

    if signature.starts_with("camera_issue") {
        let camera_actions: Vec<usize> = members
            .iter()
            .map(|s| s.camera_coverage.total_camera_actions)
            .collect();
        if camera_actions.iter().all(|&c| c == 0) {
            return "No camera actions taken in any failing run".into();
        }
        return "Camera actions present but coverage requirement not met".into();
    }

    if signature == "crash_during_run" {
        return "Simulation crashed — zero steps completed".into();
    }

    format!(
        "Failure occurred in {} run(s) across {} scenario(s)",
        members.len(),
        members
            .iter()
            .map(|s| &s.scenario_name)
            .collect::<std::collections::HashSet<_>>()
            .len()
    )
}

/// Suggest what to investigate based on failure pattern.
fn suggest_investigation(signature: &str, members: &[&RunSummary]) -> String {
    if signature.starts_with("no_growth") {
        let avg_ticks: f64 = members
            .iter()
            .filter_map(|s| s.growth_timeline.last().map(|(t, _)| *t as f64))
            .sum::<f64>()
            / members.len().max(1) as f64;

        if avg_ticks < 50.0 {
            return "Try running more ticks — seeds need ~40 ticks to germinate".into();
        }
        return "Check seed placement: seeds must land on soil adjacent to water. Verify with inspect action at seed coordinates.".into();
    }

    if signature.starts_with("insufficient_water") {
        return "Check water spring generation in create_world(). Verify water_flow system preserves water volume.".into();
    }

    if signature.starts_with("camera_issue") {
        return "Verify camera action parameters are within valid ranges (theta: 0-360, phi: 11-85, zoom: 0.3-4.0).".into();
    }

    if signature.starts_with("material_did_not_grow") {
        return "Check the specific material's growth system. Verify preconditions (water, light, nutrients) are met.".into();
    }

    if signature == "crash_during_run" {
        return "Run with RUST_BACKTRACE=1 to find the panic source. Check for out-of-bounds grid access.".into();
    }

    "Review the failing evaluator's logic and the trace for unexpected state.".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::summarize;
    use crate::runner;
    use crate::scenarios::water_system;

    #[test]
    fn cluster_no_failures_returns_empty() {
        let scenario = water_system::spring_exists();
        let result = runner::run(&scenario);
        let summary = summarize(&result);
        let clusters = cluster_failures(&[summary]);
        assert!(clusters.is_empty());
    }

    #[test]
    fn cluster_groups_by_signature() {
        // Create summaries with synthetic failures
        let scenario = water_system::spring_exists();
        let result = runner::run(&scenario);
        let mut s1 = summarize(&result);
        s1.failure_signatures = vec!["no_growth_after_100_ticks".into()];
        s1.passed = false;

        let mut s2 = summarize(&result);
        s2.failure_signatures = vec!["no_growth_after_100_ticks".into()];
        s2.passed = false;

        let mut s3 = summarize(&result);
        s3.failure_signatures = vec!["camera_issue:camera_orbited".into()];
        s3.passed = false;

        let clusters = cluster_failures(&[s1, s2, s3]);
        assert_eq!(clusters.len(), 2);
        // Most common first
        assert_eq!(clusters[0].count, 2);
        assert!(clusters[0].signature.contains("no_growth"));
    }
}

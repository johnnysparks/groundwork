//! Report generator — structured reports from batch runs.
//!
//! Generates player→manager handoffs and dev-actionable bug reports
//! in the exact format defined by AGENTS.md.

use crate::analysis::RunSummary;
use crate::clustering::FailureCluster;

/// Generate a player→manager handoff from batch run results.
///
/// Follows the AGENTS.md format:
/// Observed | Felt | Bugs | Confusions | What kept me playing | What stopped me | Requests
pub fn generate_player_handoff(
    summaries: &[RunSummary],
    clusters: &[FailureCluster],
) -> String {
    let total_runs = summaries.len();
    let passed = summaries.iter().filter(|s| s.passed).count();
    let failed = total_runs - passed;

    let avg_score = if total_runs > 0 {
        summaries.iter().map(|s| s.aggregate_score).sum::<f64>() / total_runs as f64
    } else {
        0.0
    };

    let total_plants: u64 = summaries
        .iter()
        .map(|s| s.final_material_counts.total_plant())
        .sum();

    let underground_sessions = summaries
        .iter()
        .filter(|s| s.camera_coverage.went_underground)
        .count();

    let mut report = String::new();

    // Header
    report.push_str("# Player → Manager Handoff (Automated)\n\n");

    // Observed
    report.push_str("## Observed\n");
    report.push_str(&format!(
        "- Ran {total_runs} scenario(s): {passed} passed, {failed} failed\n"
    ));
    report.push_str(&format!("- Average score: {:.0}%\n", avg_score * 100.0));
    report.push_str(&format!(
        "- Total plant voxels across all runs: {total_plants}\n"
    ));
    report.push_str(&format!(
        "- Underground exploration: {underground_sessions}/{total_runs} sessions\n"
    ));
    report.push('\n');

    // Felt
    report.push_str("## Felt\n");
    if avg_score > 0.8 {
        report.push_str("- Garden systems feel healthy — most scenarios produce living ecosystems.\n");
    } else if avg_score > 0.5 {
        report.push_str("- Mixed results — some gardens thrive, others stall. Growth feels inconsistent.\n");
    } else {
        report.push_str("- Frustrating — many gardens fail to grow. Core loop feels broken.\n");
    }
    report.push('\n');

    // Bugs
    report.push_str("## Bugs\n");
    if clusters.is_empty() {
        report.push_str("- No bugs detected in this batch.\n");
    } else {
        for cluster in clusters {
            report.push_str(&format!(
                "- **{}** ({}x): {}\n",
                cluster.signature, cluster.count, cluster.common_conditions
            ));
        }
    }
    report.push('\n');

    // Confusions
    report.push_str("## Confusions\n");
    let no_growth_clusters: Vec<&FailureCluster> = clusters
        .iter()
        .filter(|c| c.signature.contains("no_growth"))
        .collect();
    if !no_growth_clusters.is_empty() {
        report.push_str("- Seeds placed but nothing grew — unclear why. Are seeds landing on soil? Is water close enough?\n");
    }
    if underground_sessions == 0 && total_runs > 0 {
        report.push_str("- Never explored underground — the signature feature isn't being exercised.\n");
    }
    if no_growth_clusters.is_empty() && underground_sessions > 0 {
        report.push_str("- None — systems behaved as expected.\n");
    }
    report.push('\n');

    // What kept me playing
    report.push_str("## What kept me playing\n");
    if total_plants > 0 {
        report.push_str("- Seeing plants actually grow from seeds is satisfying.\n");
    }
    if underground_sessions > 0 {
        report.push_str("- Underground camera reveals root systems — the dual-view is compelling.\n");
    }
    if passed > 0 {
        report.push_str("- The garden feels alive when water and plants interact.\n");
    }
    report.push('\n');

    // What stopped me
    report.push_str("## What stopped me\n");
    if failed > 0 {
        report.push_str(&format!(
            "- {failed} scenario(s) failed — growth didn't happen as expected.\n"
        ));
    }
    for cluster in clusters {
        report.push_str(&format!("- {} ({}x)\n", cluster.common_conditions, cluster.count));
    }
    if failed == 0 && clusters.is_empty() {
        report.push_str("- Nothing major — all scenarios passed.\n");
    }
    report.push('\n');

    // Requests
    report.push_str("## Requests\n");
    for cluster in clusters {
        report.push_str(&format!("- Investigate: {}\n", cluster.suggested_investigation));
    }
    if clusters.is_empty() {
        report.push_str("- No urgent requests. Consider adding more diverse test scenarios.\n");
    }

    report
}

/// Generate a dev-actionable bug report from a failure cluster.
///
/// Follows the agents/player.md bug report format.
pub fn generate_bug_report(cluster: &FailureCluster) -> String {
    let severity = if cluster.signature.contains("crash") {
        "Blocker"
    } else if cluster.signature.contains("no_growth") {
        "Major"
    } else if cluster.signature.contains("camera") {
        "Minor"
    } else {
        "Major"
    };

    let mut report = String::new();

    report.push_str(&format!("# BUG: {}\n\n", cluster.signature));
    report.push_str(&format!("**Severity:** {severity}\n"));
    report.push_str(&format!("**Frequency:** {}/{} runs\n\n", cluster.count, cluster.affected_scenarios.len()));

    report.push_str("## Steps to reproduce\n");
    report.push_str(&format!(
        "Run scenario(s): {}\n\n",
        cluster.affected_scenarios.join(", ")
    ));

    report.push_str("## Expected\n");
    report.push_str(&format!("{}\n\n", expected_from_signature(&cluster.signature)));

    report.push_str("## Actual\n");
    report.push_str(&format!("{}\n\n", cluster.common_conditions));

    report.push_str("## Investigation\n");
    report.push_str(&format!("{}\n", cluster.suggested_investigation));

    report
}

fn expected_from_signature(signature: &str) -> &str {
    if signature.contains("no_growth") {
        "Seeds should grow into plants after sufficient ticks near water."
    } else if signature.contains("insufficient_water") {
        "Water should persist at expected levels throughout the run."
    } else if signature.contains("camera") {
        "Camera should reach the required exploration targets."
    } else if signature.contains("crash") {
        "Simulation should complete without panicking."
    } else {
        "Evaluator condition should be satisfied at end of run."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::summarize;
    use crate::clustering::cluster_failures;
    use crate::runner;
    use crate::scenarios::water_system;

    #[test]
    fn player_handoff_follows_format() {
        let scenario = water_system::spring_exists();
        let result = runner::run(&scenario);
        let summary = summarize(&result);
        let clusters = cluster_failures(&[summary.clone()]);
        let report = generate_player_handoff(&[summary], &clusters);

        // Verify all required sections present
        assert!(report.contains("## Observed"));
        assert!(report.contains("## Felt"));
        assert!(report.contains("## Bugs"));
        assert!(report.contains("## Confusions"));
        assert!(report.contains("## What kept me playing"));
        assert!(report.contains("## What stopped me"));
        assert!(report.contains("## Requests"));
    }

    #[test]
    fn bug_report_from_cluster() {
        let cluster = FailureCluster {
            signature: "no_growth_after_100_ticks".into(),
            count: 3,
            affected_scenarios: vec!["test_a".into(), "test_b".into()],
            common_conditions: "Seeds present but no plant growth".into(),
            suggested_investigation: "Check seed proximity to water".into(),
        };
        let report = generate_bug_report(&cluster);
        assert!(report.contains("BUG: no_growth_after_100_ticks"));
        assert!(report.contains("Major"));
        assert!(report.contains("test_a, test_b"));
    }
}

//! Batch runner — run multiple scenarios and produce full analysis.
//!
//! Collects traces, summaries, failure clusters, regressions, skills,
//! and a player→manager report in one pass.

use std::path::{Path, PathBuf};

use crate::analysis::{self, RunSummary};
use crate::clustering::{self, FailureCluster};
use crate::regression;
use crate::report;
use crate::runner;
use crate::scenario::Scenario;
use crate::skills::{self, Skill};

/// Complete result of a batch run.
pub struct BatchResult {
    /// Per-run summaries.
    pub summaries: Vec<RunSummary>,
    /// Clustered failures.
    pub clusters: Vec<FailureCluster>,
    /// Paths to generated regression files.
    pub regressions_generated: Vec<PathBuf>,
    /// Skills extracted from successes.
    pub skills_extracted: Vec<Skill>,
    /// Player→manager report text.
    pub report: String,
}

/// Run all scenarios in batch and produce a full analysis.
///
/// - Runs each scenario, saves traces as JSON to `trace_dir`
/// - Summarizes all results
/// - Clusters failures
/// - Generates regressions for new failures
/// - Extracts skills from successes
/// - Generates player→manager report
pub fn run_batch(scenarios: &[Scenario], trace_dir: &Path) -> BatchResult {
    std::fs::create_dir_all(trace_dir).ok();

    let mut summaries = Vec::new();
    let mut traces = Vec::new();
    let mut regressions_generated = Vec::new();

    for scenario in scenarios {
        let result = runner::run(scenario);

        // Save trace JSON
        let trace_path = trace_dir.join(format!("{}.json", scenario.name));
        if let Ok(()) = std::fs::write(&trace_path, result.trace.to_json()) {
            // Trace saved successfully
        }

        let summary = analysis::summarize(&result);

        // Generate regressions for failures
        if !summary.passed {
            let failed_verdicts: Vec<_> =
                result.verdicts.iter().filter(|v| !v.passed).cloned().collect();
            let reg_name = format!("regression_{}", scenario.name);
            let reg_scenario =
                regression::trace_to_regression(&result.trace, &failed_verdicts, &reg_name);

            let reg_path = trace_dir.join(format!("{reg_name}.rs"));
            if regression::write_regression_file(&reg_scenario, &reg_path).is_ok() {
                regressions_generated.push(reg_path);
            }
        }

        traces.push(result.trace);
        summaries.push(summary);
    }

    // Cluster failures
    let clusters = clustering::cluster_failures(&summaries);

    // Extract skills from successes
    let skills_extracted = skills::extract_skills(&summaries, &traces);

    // Generate report
    let report_text = report::generate_player_handoff(&summaries, &clusters);

    BatchResult {
        summaries,
        clusters,
        regressions_generated,
        skills_extracted,
        report: report_text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenarios::{ecosystem, water_system};

    #[test]
    fn batch_processes_multiple_scenarios() {
        let scenarios = vec![
            water_system::spring_exists(),
            water_system::water_flow_and_absorption(),
        ];

        let tmp = std::env::temp_dir().join("groundwork_batch_test");
        let result = run_batch(&scenarios, &tmp);

        assert_eq!(result.summaries.len(), 2);
        assert!(!result.report.is_empty());
        assert!(result.report.contains("## Observed"));

        // Cleanup
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn batch_saves_traces_as_json() {
        let scenarios = vec![water_system::spring_exists()];

        let tmp = std::env::temp_dir().join("groundwork_batch_trace_test");
        run_batch(&scenarios, &tmp);

        let trace_path = tmp.join("spring_exists.json");
        assert!(trace_path.exists());
        let json = std::fs::read_to_string(&trace_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn batch_extracts_skills_from_successes() {
        let scenarios = vec![ecosystem::diverse_garden()];

        let tmp = std::env::temp_dir().join("groundwork_batch_skills_test");
        let result = run_batch(&scenarios, &tmp);

        // diverse_garden passes and has water + planting, so skills should be extracted
        assert!(!result.skills_extracted.is_empty());

        std::fs::remove_dir_all(&tmp).ok();
    }
}

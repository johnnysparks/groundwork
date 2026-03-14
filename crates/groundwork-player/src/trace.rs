//! Run traces — the complete record of a scenario execution.
//!
//! Every run produces a trace: an ordered list of (action, observation, oracle snapshot)
//! triples. Traces are the raw material for debugging, scoring, clustering failures,
//! and creating regression tests.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::action::Action;
use crate::observer::Observation;
use crate::oracle::OracleSnapshot;

/// A single step in a trace: what the actor did, what it saw, and ground truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    /// The action the actor took.
    pub action: Action,
    /// What the actor saw back (actor-visible).
    pub observation: Observation,
    /// Privileged ground truth (oracle-only).
    pub oracle: OracleSnapshot,
    /// Wall-clock duration of this step.
    #[serde(with = "duration_millis")]
    pub duration: Duration,
}

/// Complete trace of a scenario run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    /// Name of the scenario that produced this trace.
    pub scenario_name: String,
    /// Steps in execution order.
    pub steps: Vec<TraceStep>,
    /// Total wall-clock duration.
    #[serde(with = "duration_millis")]
    pub total_duration: Duration,
}

impl Trace {
    /// Start building a new trace.
    pub fn builder(scenario_name: &str) -> TraceBuilder {
        TraceBuilder {
            scenario_name: scenario_name.to_string(),
            steps: Vec::new(),
            start: Instant::now(),
            step_start: Instant::now(),
        }
    }

    /// The final oracle snapshot (state at end of run).
    pub fn final_oracle(&self) -> Option<&OracleSnapshot> {
        self.steps.last().map(|s| &s.oracle)
    }

    /// Find a checkpoint step by label.
    pub fn checkpoint(&self, label: &str) -> Option<&TraceStep> {
        self.steps.iter().find(|s| {
            matches!(&s.action, Action::Checkpoint { label: l } if l == label)
        })
    }

    /// Oracle snapshot at a specific step index.
    pub fn oracle_at(&self, step: usize) -> Option<&OracleSnapshot> {
        self.steps.get(step).map(|s| &s.oracle)
    }

    /// Serialize the trace to JSON for artifact storage.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
    }
}

/// Incrementally builds a trace during scenario execution.
pub struct TraceBuilder {
    scenario_name: String,
    steps: Vec<TraceStep>,
    start: Instant,
    step_start: Instant,
}

impl TraceBuilder {
    /// Mark the beginning of a new step (for timing).
    pub fn begin_step(&mut self) {
        self.step_start = Instant::now();
    }

    /// Record a completed step.
    pub fn record(
        &mut self,
        action: Action,
        observation: Observation,
        oracle: OracleSnapshot,
    ) {
        let duration = self.step_start.elapsed();
        self.steps.push(TraceStep {
            action,
            observation,
            oracle,
            duration,
        });
    }

    /// Finalize the trace.
    pub fn finish(self) -> Trace {
        Trace {
            scenario_name: self.scenario_name,
            steps: self.steps,
            total_duration: self.start.elapsed(),
        }
    }
}

/// Serde helper for Duration as milliseconds.
mod duration_millis {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        d.as_millis().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let ms = u64::deserialize(d)?;
        Ok(Duration::from_millis(ms))
    }
}

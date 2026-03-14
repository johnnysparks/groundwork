//! Planner trait — the decision-making interface for autonomous players.
//!
//! A planner receives observation history (actor-visible only) and decides
//! what actions to take next. The planner NEVER sees OracleSnapshot data.

use crate::action::Action;
use crate::observer::Observation;
use serde::{Deserialize, Serialize};

/// A single entry in the planner's observation history.
/// Contains what was done and what was seen back — nothing privileged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationEntry {
    /// The action that was executed.
    pub action: Action,
    /// The actor-visible observation returned by that action.
    pub observation: Observation,
    /// Index of this step in the session.
    pub step_index: usize,
}

/// Trait for planners that choose actions based on observation history.
///
/// Planners see only `Observation.text` — the same output a human player gets.
/// They must NEVER access oracle snapshots or privileged sim state.
pub trait Planner {
    /// Given the history of observations so far, choose the next action(s).
    /// Returns a batch of actions to execute before re-planning.
    fn plan(&mut self, history: &[ObservationEntry]) -> Vec<Action>;

    /// Whether the planner wants to stop the session.
    fn should_stop(&self, history: &[ObservationEntry]) -> bool;
}

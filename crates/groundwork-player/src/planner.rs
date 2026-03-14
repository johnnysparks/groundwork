//! Planner trait — the autonomous decision-making interface.
//!
//! A planner reads observation history and chooses the next action(s).
//! It only sees `Observation.text` (actor-visible), never `OracleSnapshot`.
//! This preserves the actor/oracle separation.

use crate::action::Action;
use crate::observer::Observation;

/// An entry in the planner's observation history.
/// Contains only actor-visible information — never oracle data.
#[derive(Debug, Clone)]
pub struct ObservationEntry {
    /// The action that was executed.
    pub action: Action,
    /// What the actor saw back (text output only).
    pub observation: Observation,
    /// Index of this step in the run.
    pub step_index: usize,
}

/// Trait for autonomous planners that decide what the player does next.
///
/// Implementations range from simple scripted strategies (for testing)
/// to LLM-powered planners (for autonomous play sessions).
///
/// **Critical invariant:** Planners must NEVER see `OracleSnapshot` data.
/// They only receive `ObservationEntry` which contains the same text
/// a human player would see.
pub trait Planner {
    /// Given the history of observations so far, choose the next action(s).
    /// Returns a batch of actions to execute before the next planning call.
    fn plan(&mut self, history: &[ObservationEntry]) -> Vec<Action>;

    /// Whether the planner wants to stop the session early.
    fn should_stop(&self, history: &[ObservationEntry]) -> bool;

    /// Human-readable name for this planner (used in traces).
    fn name(&self) -> &str;
}

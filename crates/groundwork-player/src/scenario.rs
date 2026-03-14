//! Scenario definitions — scripted sequences of player actions with evaluators.
//!
//! A scenario is a deterministic test case: a name, a sequence of actions,
//! a set of voxel probes (coordinates to snapshot), and evaluators to judge the result.

use crate::action::Action;
use crate::evaluator::Evaluator;

/// A complete test scenario: actions to take + evaluators to judge the result.
pub struct Scenario {
    /// Human-readable name for this scenario.
    pub name: String,
    /// Why this scenario exists / what it tests.
    pub description: String,
    /// The ordered sequence of player actions.
    pub actions: Vec<Action>,
    /// Voxel coordinates to probe in oracle snapshots (for VoxelMaterial evaluators).
    pub probes: Vec<(usize, usize, usize)>,
    /// Evaluators that judge the resulting trace.
    pub evaluators: Vec<Box<dyn Evaluator>>,
}

impl Scenario {
    /// Create a new scenario builder.
    pub fn new(name: &str) -> ScenarioBuilder {
        ScenarioBuilder {
            name: name.to_string(),
            description: String::new(),
            actions: Vec::new(),
            probes: Vec::new(),
            evaluators: Vec::new(),
        }
    }
}

/// Fluent builder for scenarios.
pub struct ScenarioBuilder {
    name: String,
    description: String,
    actions: Vec<Action>,
    probes: Vec<(usize, usize, usize)>,
    evaluators: Vec<Box<dyn Evaluator>>,
}

impl ScenarioBuilder {
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Add a single action.
    pub fn action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    /// Add a checkpoint (labeled marker in the trace).
    pub fn checkpoint(mut self, label: &str) -> Self {
        self.actions.push(Action::Checkpoint {
            label: label.to_string(),
        });
        self
    }

    /// Tick the simulation.
    pub fn tick(mut self, n: u64) -> Self {
        self.actions.push(Action::Tick { n });
        self
    }

    /// Place a tool at coordinates.
    pub fn place(mut self, tool: &str, x: usize, y: usize, z: usize) -> Self {
        self.actions.push(Action::Place {
            tool: tool.to_string(),
            x,
            y,
            z,
            species: None,
        });
        self
    }

    /// Place a seed of a specific species.
    pub fn plant(mut self, species: &str, x: usize, y: usize, z: usize) -> Self {
        self.actions.push(Action::Place {
            tool: "seed".to_string(),
            x,
            y,
            z,
            species: Some(species.to_string()),
        });
        self
    }

    /// Fill a region with a tool.
    pub fn fill(
        mut self,
        tool: &str,
        x1: usize,
        y1: usize,
        z1: usize,
        x2: usize,
        y2: usize,
        z2: usize,
    ) -> Self {
        self.actions.push(Action::Fill {
            tool: tool.to_string(),
            x1,
            y1,
            z1,
            x2,
            y2,
            z2,
        });
        self
    }

    /// Observe status.
    pub fn status(mut self) -> Self {
        self.actions.push(Action::Status);
        self
    }

    /// Inspect a voxel.
    pub fn inspect(mut self, x: usize, y: usize, z: usize) -> Self {
        self.actions.push(Action::Inspect { x, y, z });
        self
    }

    /// Add a voxel probe coordinate for oracle snapshots.
    pub fn probe(mut self, x: usize, y: usize, z: usize) -> Self {
        self.probes.push((x, y, z));
        self
    }

    /// Add an evaluator.
    pub fn eval(mut self, evaluator: impl Evaluator + 'static) -> Self {
        self.evaluators.push(Box::new(evaluator));
        self
    }

    /// Build the scenario.
    pub fn build(self) -> Scenario {
        Scenario {
            name: self.name,
            description: self.description,
            actions: self.actions,
            probes: self.probes,
            evaluators: self.evaluators,
        }
    }
}

//! Player actions — the embodied interface to the simulation.
//!
//! Each action mirrors what a human player can do through the CLI/web UI.
//! The actor (scenario script or future LLM planner) composes these to play.

use serde::{Deserialize, Serialize};

/// A single player action. Mirrors the CLI tool interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Advance the simulation by N ticks.
    Tick { n: u64 },

    /// Place a tool at a single coordinate.
    /// Tool: "seed", "water", "soil", "stone", "air"/"dig"
    /// Species: optional species name for seed tool (e.g. "oak", "fern")
    Place {
        tool: String,
        x: usize,
        y: usize,
        z: usize,
        species: Option<String>,
    },

    /// Fill a rectangular region with a tool.
    Fill {
        tool: String,
        x1: usize,
        y1: usize,
        z1: usize,
        x2: usize,
        y2: usize,
        z2: usize,
    },

    /// Inspect a voxel (actor-visible observation).
    Inspect { x: usize, y: usize, z: usize },

    /// Get world status summary (actor-visible observation).
    Status,

    /// View a Z-slice (actor-visible observation).
    View { z: usize },

    /// A labeled checkpoint for trace readability.
    /// Does nothing to the sim; just marks a point in the trace.
    Checkpoint { label: String },
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Tick { n } => write!(f, "tick {n}"),
            Action::Place { tool, x, y, z, species } => {
                if let Some(sp) = species {
                    write!(f, "place {sp} {x} {y} {z}")
                } else {
                    write!(f, "place {tool} {x} {y} {z}")
                }
            }
            Action::Fill { tool, x1, y1, z1, x2, y2, z2 } => {
                write!(f, "fill {tool} {x1} {y1} {z1} {x2} {y2} {z2}")
            }
            Action::Inspect { x, y, z } => write!(f, "inspect {x} {y} {z}"),
            Action::Status => write!(f, "status"),
            Action::View { z } => write!(f, "view --z {z}"),
            Action::Checkpoint { label } => write!(f, "# {label}"),
        }
    }
}

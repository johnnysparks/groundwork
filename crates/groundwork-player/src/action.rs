//! Player actions — the embodied interface to the simulation.
//!
//! Each action mirrors what a human player can do through the CLI/web UI.
//! The actor (scenario script or future LLM planner) composes these to play.

use serde::{Deserialize, Serialize};

/// A single player action. Mirrors what a human player can do via the web UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // --- Simulation ---

    /// Advance the simulation by N ticks.
    Tick { n: u64 },

    // --- Tool use ---

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

    // --- Camera ---

    /// Orbit the camera to a specific angle.
    /// theta: azimuth angle in degrees (0-360, default 45)
    /// phi: elevation angle in degrees (11-85, default 60)
    CameraOrbit { theta_deg: f64, phi_deg: f64 },

    /// Pan the camera to center on a world position.
    /// Coordinates are in sim space (x, y = horizontal, z = vertical/depth).
    CameraPan { x: f64, y: f64, z: f64 },

    /// Zoom the camera. 1.0 = default, >1 = closer, <1 = further.
    CameraZoom { level: f64 },

    /// Set the cutaway depth for underground viewing.
    /// z = sim Z level to cut at. GRID_Z (60) = no cutaway (full above-ground view).
    /// 30 = surface level. <30 = progressively deeper underground.
    CameraCutaway { z: f64 },

    /// Reset camera to default diorama view.
    CameraReset,

    // --- Observation ---

    /// Inspect a voxel (actor-visible observation).
    Inspect { x: usize, y: usize, z: usize },

    /// Get world status summary (actor-visible observation).
    Status,

    /// View a Z-slice (actor-visible observation).
    View { z: usize },

    // --- Visual capture ---

    /// Capture a screenshot at this point.
    /// In headless Rust mode, this records the intent (label + camera state).
    /// In browser mode (Playwright harness), this captures an actual PNG from the
    /// Three.js renderer and saves it to the artifacts directory.
    Screenshot { label: String },

    // --- Meta ---

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
            Action::CameraOrbit { theta_deg, phi_deg } => {
                write!(f, "camera orbit {theta_deg:.0}° {phi_deg:.0}°")
            }
            Action::CameraPan { x, y, z } => write!(f, "camera pan {x:.0} {y:.0} {z:.0}"),
            Action::CameraZoom { level } => write!(f, "camera zoom {level:.1}x"),
            Action::CameraCutaway { z } => write!(f, "camera cutaway z={z:.0}"),
            Action::CameraReset => write!(f, "camera reset"),
            Action::Inspect { x, y, z } => write!(f, "inspect {x} {y} {z}"),
            Action::Status => write!(f, "status"),
            Action::View { z } => write!(f, "view --z {z}"),
            Action::Screenshot { label } => write!(f, "screenshot \"{label}\""),
            Action::Checkpoint { label } => write!(f, "# {label}"),
        }
    }
}

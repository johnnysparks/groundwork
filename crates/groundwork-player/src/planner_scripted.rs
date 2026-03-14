//! Scripted planner — a deterministic strategy for testing the planner interface.
//!
//! Follows a fixed sequence: place water, plant seeds near it, tick to grow,
//! inspect results, explore with camera, repeat. Produces identical traces
//! on every run for regression testing.

use groundwork_sim::grid::{GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};

use crate::action::Action;
use crate::planner::{ObservationEntry, Planner};

/// A deterministic planner that follows a fixed gardening strategy.
///
/// Strategy phases:
/// 1. Status check + camera exploration
/// 2. Place water near center
/// 3. Plant diverse species near water
/// 4. Tick and observe growth
/// 5. Explore underground with cutaway
/// 6. Final status check
pub struct ScriptedPlanner {
    /// Pre-built action sequence.
    script: Vec<Action>,
    /// Current position in the script.
    cursor: usize,
    /// How many actions to return per plan() call.
    batch_size: usize,
}

impl Default for ScriptedPlanner {
    fn default() -> Self {
        Self::new(5)
    }
}

impl ScriptedPlanner {
    /// Create a scripted planner with a given batch size.
    pub fn new(batch_size: usize) -> Self {
        let script = Self::build_script();
        Self {
            script,
            cursor: 0,
            batch_size: batch_size.max(1),
        }
    }

    fn build_script() -> Vec<Action> {
        let cx = GRID_X / 2;
        let cy = GRID_Y / 2;
        let above = GROUND_LEVEL + 10;

        let mut actions = Vec::new();

        // Phase 1: Initial observation
        actions.push(Action::Checkpoint {
            label: "start".into(),
        });
        actions.push(Action::Status);
        actions.push(Action::CameraOrbit {
            theta_deg: 45.0,
            phi_deg: 60.0,
        });

        // Phase 2: Place water near center to create a pond
        actions.push(Action::Checkpoint {
            label: "placing_water".into(),
        });
        actions.push(Action::Fill {
            tool: "water".into(),
            x1: cx - 2,
            y1: cy - 2,
            z1: above,
            x2: cx + 2,
            y2: cy + 2,
            z2: above,
        });

        // Phase 3: Plant diverse species around the water
        actions.push(Action::Checkpoint {
            label: "planting".into(),
        });
        let species = ["oak", "birch", "fern", "moss", "wildflower", "grass"];
        let offsets: [(i32, i32); 6] = [
            (-5, 0),
            (5, 0),
            (0, -5),
            (0, 5),
            (-4, -4),
            (4, 4),
        ];
        for (sp, (dx, dy)) in species.iter().zip(offsets.iter()) {
            let sx = (cx as i32 + dx).clamp(0, GRID_X as i32 - 1) as usize;
            let sy = (cy as i32 + dy).clamp(0, GRID_Y as i32 - 1) as usize;
            actions.push(Action::Place {
                tool: "seed".into(),
                x: sx,
                y: sy,
                z: above,
                species: Some(sp.to_string()),
            });
        }

        // Phase 4: Tick and observe
        actions.push(Action::Checkpoint {
            label: "growing".into(),
        });
        actions.push(Action::Tick { n: 50 });
        actions.push(Action::Status);
        actions.push(Action::Inspect {
            x: cx - 5,
            y: cy,
            z: GROUND_LEVEL,
        });

        // More growth
        actions.push(Action::Tick { n: 100 });
        actions.push(Action::Status);

        // Phase 5: Camera exploration
        actions.push(Action::Checkpoint {
            label: "exploring".into(),
        });
        // Orbit around
        actions.push(Action::CameraOrbit {
            theta_deg: 180.0,
            phi_deg: 45.0,
        });
        actions.push(Action::CameraZoom { level: 1.5 });
        // Pan to planting area
        actions.push(Action::CameraPan {
            x: cx as f64,
            y: cy as f64,
            z: GROUND_LEVEL as f64,
        });
        // Go underground
        actions.push(Action::CameraCutaway {
            z: GROUND_LEVEL as f64 - 5.0,
        });
        actions.push(Action::View {
            z: GROUND_LEVEL - 3,
        });
        // Come back up
        actions.push(Action::CameraCutaway {
            z: GRID_Z as f64,
        });
        actions.push(Action::CameraOrbit {
            theta_deg: 270.0,
            phi_deg: 30.0,
        });

        // Phase 6: Final observation
        actions.push(Action::Checkpoint {
            label: "final".into(),
        });
        actions.push(Action::Status);
        actions.push(Action::CameraReset);

        actions
    }
}

impl Planner for ScriptedPlanner {
    fn plan(&mut self, _history: &[ObservationEntry]) -> Vec<Action> {
        if self.cursor >= self.script.len() {
            return Vec::new();
        }
        let end = (self.cursor + self.batch_size).min(self.script.len());
        let batch = self.script[self.cursor..end].to_vec();
        self.cursor = end;
        batch
    }

    fn should_stop(&self, _history: &[ObservationEntry]) -> bool {
        self.cursor >= self.script.len()
    }

    fn name(&self) -> &str {
        "scripted"
    }
}

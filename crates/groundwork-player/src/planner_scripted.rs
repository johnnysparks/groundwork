//! Scripted planner — a deterministic planner for testing the planner interface.
//!
//! Follows a fixed strategy: place water → plant seeds → tick → inspect → repeat.
//! Produces identical traces on every run (for regression testing).

use crate::action::Action;
use crate::planner::{ObservationEntry, Planner};

/// A deterministic planner that follows a fixed gardening strategy.
///
/// Strategy:
/// 1. Status check
/// 2. Place water near center
/// 3. Plant diverse species near water
/// 4. Orbit camera to inspect
/// 5. Tick to let things grow
/// 6. Inspect planted areas
/// 7. Cutaway to view underground
/// 8. More ticking and observation
pub struct ScriptedPlanner {
    /// Pre-computed action batches.
    batches: Vec<Vec<Action>>,
    /// Which batch we're on.
    batch_index: usize,
}

impl Default for ScriptedPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptedPlanner {
    pub fn new() -> Self {
        let cx = 40usize; // center x
        let cy = 40usize; // center y
        let seed_z = 50usize; // well above ground (GROUND_LEVEL=40), seeds fall with gravity
        let water_z = 45usize; // above ground so water falls down

        let batches = vec![
            // Batch 0: Initial survey — screenshot the empty garden
            vec![
                Action::Status,
                Action::CameraOrbit {
                    theta_deg: 45.0,
                    phi_deg: 60.0,
                },
                Action::Screenshot {
                    label: "01-initial-empty-garden".into(),
                },
            ],
            // Batch 1: Place water (high up, it falls to surface)
            vec![
                Action::Fill {
                    tool: "water".into(),
                    x1: cx - 5,
                    y1: cy - 5,
                    z1: water_z,
                    x2: cx + 5,
                    y2: cy + 5,
                    z2: water_z,
                },
                Action::Tick { n: 5 },
                Action::Status,
                Action::Screenshot {
                    label: "02-water-placed".into(),
                },
            ],
            // Batch 2: Plant seeds near water (high up, seeds fall to surface)
            vec![
                Action::Place {
                    tool: "seed".into(),
                    x: cx - 4,
                    y: cy,
                    z: seed_z,
                    species: Some("oak".into()),
                },
                Action::Place {
                    tool: "seed".into(),
                    x: cx + 4,
                    y: cy,
                    z: seed_z,
                    species: Some("fern".into()),
                },
                Action::Place {
                    tool: "seed".into(),
                    x: cx,
                    y: cy - 4,
                    z: seed_z,
                    species: Some("moss".into()),
                },
                Action::Place {
                    tool: "seed".into(),
                    x: cx,
                    y: cy + 4,
                    z: seed_z,
                    species: Some("wildflower".into()),
                },
                Action::Screenshot {
                    label: "03-seeds-planted".into(),
                },
            ],
            // Batch 3: Early growth — tick 50 and capture
            vec![
                Action::Tick { n: 50 },
                Action::Status,
                Action::Inspect {
                    x: cx - 4,
                    y: cy,
                    z: 41,
                },
                Action::Screenshot {
                    label: "04-early-growth-tick50".into(),
                },
            ],
            // Batch 4: Orbit camera for different angle — visual check
            vec![
                Action::CameraOrbit {
                    theta_deg: 90.0,
                    phi_deg: 45.0,
                },
                Action::CameraZoom { level: 1.5 },
                Action::CameraPan {
                    x: cx as f64,
                    y: cy as f64,
                    z: 41.0,
                },
                Action::Screenshot {
                    label: "05-side-angle-closeup".into(),
                },
            ],
            // Batch 5: More growth — tick 150 total, capture progress
            vec![
                Action::Tick { n: 100 },
                Action::Status,
                Action::Screenshot {
                    label: "06-mid-growth-tick150".into(),
                },
            ],
            // Batch 6: Underground view — the X-ray garden
            vec![
                Action::CameraCutaway { z: 35.0 },
                Action::CameraOrbit {
                    theta_deg: 180.0,
                    phi_deg: 50.0,
                },
                Action::Inspect {
                    x: cx - 1,
                    y: cy,
                    z: 38,
                },
                Action::Screenshot {
                    label: "07-underground-roots".into(),
                },
            ],
            // Batch 7: Back above ground, more growth
            vec![
                Action::CameraCutaway { z: 100.0 },
                Action::Tick { n: 100 },
                Action::Status,
                Action::CameraOrbit {
                    theta_deg: 270.0,
                    phi_deg: 60.0,
                },
                Action::Screenshot {
                    label: "08-mature-garden-tick250".into(),
                },
            ],
            // Batch 8: Golden hour beauty shot — zoom out, low angle
            vec![
                Action::CameraOrbit {
                    theta_deg: 315.0,
                    phi_deg: 30.0,
                },
                Action::CameraZoom { level: 0.8 },
                Action::Screenshot {
                    label: "09-golden-hour-wide".into(),
                },
            ],
            // Batch 9: Final top-down overview
            vec![
                Action::CameraOrbit {
                    theta_deg: 45.0,
                    phi_deg: 80.0,
                },
                Action::CameraZoom { level: 0.6 },
                Action::Screenshot {
                    label: "10-final-overview-topdown".into(),
                },
            ],
        ];

        Self {
            batches,
            batch_index: 0,
        }
    }
}

impl Planner for ScriptedPlanner {
    fn plan(&mut self, _history: &[ObservationEntry]) -> Vec<Action> {
        if self.batch_index >= self.batches.len() {
            return Vec::new();
        }
        let batch = self.batches[self.batch_index].clone();
        self.batch_index += 1;
        batch
    }

    fn should_stop(&self, _history: &[ObservationEntry]) -> bool {
        self.batch_index >= self.batches.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripted_planner_is_deterministic() {
        let mut p1 = ScriptedPlanner::new();
        let mut p2 = ScriptedPlanner::new();

        let mut actions1 = Vec::new();
        let mut actions2 = Vec::new();

        loop {
            let b1 = p1.plan(&[]);
            let b2 = p2.plan(&[]);
            if b1.is_empty() && b2.is_empty() {
                break;
            }
            for a in &b1 {
                actions1.push(format!("{a}"));
            }
            for a in &b2 {
                actions2.push(format!("{a}"));
            }
        }

        assert_eq!(actions1, actions2);
        assert!(!actions1.is_empty());
    }

    #[test]
    fn scripted_planner_stops() {
        let mut p = ScriptedPlanner::new();
        // Exhaust all batches
        while !p.should_stop(&[]) {
            let batch = p.plan(&[]);
            assert!(!batch.is_empty());
        }
        // After exhaustion, plan returns empty
        assert!(p.plan(&[]).is_empty());
    }
}

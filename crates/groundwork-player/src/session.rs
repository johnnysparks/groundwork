//! Session-based planner — reads actions from a JSON file, enabling an external
//! agent (like the current Claude Code session) to act as the planner.
//!
//! Instead of calling an API, the game loop:
//! 1. Writes observation history to `{session_dir}/observations.json`
//! 2. Reads next actions from `{session_dir}/actions.json`
//! 3. Executes them, records results, repeats
//!
//! The external agent writes `actions.json` after reading `observations.json`.

use std::path::{Path, PathBuf};

use crate::action::Action;
use crate::planner::{ObservationEntry, Planner};

/// A planner that reads actions from a JSON file on disk.
///
/// This enables any external process (including the current Claude Code session)
/// to act as the planner by writing action batches to a file.
pub struct SessionPlanner {
    /// Directory for session files (observations.json, actions.json).
    session_dir: PathBuf,
    /// Maximum actions before auto-stopping.
    max_actions: usize,
    /// How many actions have been executed so far.
    actions_executed: usize,
    /// Whether to block waiting for the actions file.
    wait_for_input: bool,
}

impl SessionPlanner {
    pub fn new(session_dir: &Path, max_actions: usize) -> Self {
        std::fs::create_dir_all(session_dir).ok();
        Self {
            session_dir: session_dir.to_path_buf(),
            max_actions,
            actions_executed: 0,
            wait_for_input: true,
        }
    }

    /// Create a non-blocking planner that returns empty if no actions file exists.
    pub fn non_blocking(session_dir: &Path, max_actions: usize) -> Self {
        let mut p = Self::new(session_dir, max_actions);
        p.wait_for_input = false;
        p
    }

    fn observations_path(&self) -> PathBuf {
        self.session_dir.join("observations.json")
    }

    fn actions_path(&self) -> PathBuf {
        self.session_dir.join("actions.json")
    }

    fn write_observations(&self, history: &[ObservationEntry]) {
        // Write a simplified view: just step_index, action display, and observation text
        let entries: Vec<serde_json::Value> = history
            .iter()
            .map(|e| {
                serde_json::json!({
                    "step": e.step_index,
                    "action": format!("{}", e.action),
                    "observation": e.observation.text,
                    "tick": e.observation.tick,
                })
            })
            .collect();

        let json = serde_json::to_string_pretty(&entries).unwrap_or_default();
        std::fs::write(self.observations_path(), json).ok();
    }

    fn read_actions(&self) -> Vec<Action> {
        let path = self.actions_path();

        if self.wait_for_input {
            // Poll for the file to appear (external agent writes it)
            eprintln!("\n>>> Waiting for actions at: {}", path.display());
            eprintln!(">>> Write a JSON array of actions to that file, then I'll execute them.\n");

            for _ in 0..600 {
                // 10 minute timeout
                if path.exists() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }

        if !path.exists() {
            return Vec::new();
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        // Remove the file so we don't re-read it
        std::fs::remove_file(&path).ok();

        // Parse as array of Action
        match serde_json::from_str::<Vec<Action>>(&content) {
            Ok(actions) => actions,
            Err(e) => {
                eprintln!(">>> Failed to parse actions.json: {e}");
                Vec::new()
            }
        }
    }
}

impl Planner for SessionPlanner {
    fn plan(&mut self, history: &[ObservationEntry]) -> Vec<Action> {
        if self.actions_executed >= self.max_actions {
            return Vec::new();
        }

        // Write current observations for the external agent to read
        self.write_observations(history);

        // Read actions from file
        let actions = self.read_actions();
        self.actions_executed += actions.len();
        actions
    }

    fn should_stop(&self, _history: &[ObservationEntry]) -> bool {
        self.actions_executed >= self.max_actions
    }
}

/// Run an interactive session and return the full result.
///
/// This is the entry point for session-based play. It:
/// 1. Creates a fresh world
/// 2. Loops: write observations → read actions → execute → record
/// 3. Evaluates with standard evaluators
/// 4. Returns the full RunResult for analysis
pub fn run_session(session_dir: &Path, max_steps: usize) -> crate::runner::RunResult {
    use crate::evaluator::{CameraOrbited, CameraWentUnderground, MaterialMinimum, NoCrash};

    let mut planner = SessionPlanner::new(session_dir, max_steps);
    let probes = vec![(40, 40, 41), (40, 40, 38)];
    let evaluators: Vec<Box<dyn crate::evaluator::Evaluator>> = vec![
        Box::new(NoCrash),
        Box::new(MaterialMinimum::new("plant", 1)),
        Box::new(MaterialMinimum::new("water", 1)),
        Box::new(CameraOrbited),
        Box::new(CameraWentUnderground::new()),
    ];

    crate::runner::run_autonomous(&mut planner, &probes, &evaluators, max_steps)
}

/// Execute a single batch of actions against a fresh or continuing world,
/// returning observations as text. This is the simplest interface for
/// an external agent to drive the game.
pub fn execute_actions_batch(actions: &[Action]) -> Vec<String> {
    let mut world = groundwork_sim::create_world();
    let mut schedule = groundwork_sim::create_schedule();
    let mut camera = crate::oracle::CameraState::default();

    let mut observations = Vec::new();
    for action in actions {
        let obs = crate::runner::execute_action(&mut world, &mut schedule, &mut camera, action);
        observations.push(format!("{action} → {}", obs.text));
    }
    observations
}

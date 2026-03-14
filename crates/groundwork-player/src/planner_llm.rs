//! LLM-powered planner — uses Claude to autonomously decide player actions.
//!
//! Formats observation history into a prompt, sends it to the Anthropic API,
//! and parses the response into a batch of actions. Uses a slow cadence:
//! plans a batch of actions at once, then executes them all before re-planning.
//!
//! Gated behind the `llm` feature flag so the base crate doesn't require
//! API keys to compile or test.

#[cfg(feature = "llm")]
mod inner {
    use crate::action::Action;
    use crate::planner::{ObservationEntry, Planner};

    /// LLM-powered planner that uses Claude to decide actions.
    pub struct LlmPlanner {
        /// Anthropic API key.
        api_key: String,
        /// Model to use (e.g. "claude-sonnet-4-20250514").
        model: String,
        /// How many actions to request per planning call.
        batch_size: usize,
        /// Maximum total steps before stopping.
        max_steps: usize,
        /// Steps executed so far.
        steps_done: usize,
        /// HTTP client for API calls.
        client: reqwest::blocking::Client,
    }

    impl LlmPlanner {
        /// Create a new LLM planner.
        ///
        /// - `api_key`: Anthropic API key (or reads `ANTHROPIC_API_KEY` env var if empty).
        /// - `model`: Model ID to use.
        /// - `batch_size`: How many actions to request per plan() call.
        /// - `max_steps`: Maximum total steps before the planner stops.
        pub fn new(api_key: &str, model: &str, batch_size: usize, max_steps: usize) -> Self {
            let api_key = if api_key.is_empty() {
                std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()
            } else {
                api_key.to_string()
            };
            Self {
                api_key,
                model: model.to_string(),
                batch_size,
                max_steps,
                steps_done: 0,
                client: reqwest::blocking::Client::new(),
            }
        }

        /// Create with defaults: reads API key from env, uses claude-sonnet-4-20250514,
        /// batch size 5, max 50 steps.
        pub fn from_env() -> Self {
            Self::new("", "claude-sonnet-4-20250514", 5, 50)
        }

        fn build_system_prompt(&self) -> String {
            format!(
                r#"You are a player of GROUNDWORK, a cozy ecological voxel garden builder game.

## Your Goal
Build a beautiful, self-sustaining garden. You explore, plant diverse species near water, shape terrain, and observe your garden grow.

## World
- Grid: 80x80x100 voxels (x, y horizontal; z vertical)
- GROUND_LEVEL: z=40 (surface). Below = underground, above = sky.
- A water spring exists near the center of the grid.
- Voxel size: 0.5m. The garden is 40m x 40m x 50m.

## Available Actions
Return actions as a JSON array. Each action is an object with a "type" field.

### Simulation
- {{"type": "tick", "n": <integer>}} — advance simulation by N ticks (use 10-50 for growth)

### Tools (place materials)
- {{"type": "place", "tool": "<tool>", "x": <int>, "y": <int>, "z": <int>}}
  Tools: "water", "soil", "stone", "air" (dig)
- {{"type": "place", "tool": "seed", "x": <int>, "y": <int>, "z": <int>, "species": "<name>"}}
  Species: "oak", "birch", "willow", "pine", "fern", "berry-bush", "holly", "wildflower", "daisy", "moss", "grass", "clover"
- {{"type": "fill", "tool": "<tool>", "x1": <int>, "y1": <int>, "z1": <int>, "x2": <int>, "y2": <int>, "z2": <int>}}
  Fill a rectangular region.

Note: Seeds/water/soil have gravity — they fall through air to land on solid ground.
Place seeds above surface (z=50 is a good height) so they fall to the soil.

### Camera
- {{"type": "camera_orbit", "theta_deg": <float>, "phi_deg": <float>}} — orbit (azimuth 0-360, elevation 11-85)
- {{"type": "camera_pan", "x": <float>, "y": <float>, "z": <float>}} — look at position
- {{"type": "camera_zoom", "level": <float>}} — zoom (0.3-4.0, default 1.0)
- {{"type": "camera_cutaway", "z": <float>}} — slice view at z height (< 40 = underground)
- {{"type": "camera_reset"}} — reset to default view

### Observation
- {{"type": "status"}} — world summary with material counts
- {{"type": "inspect", "x": <int>, "y": <int>, "z": <int>}} — examine a voxel
- {{"type": "view", "z": <int>}} — view a horizontal z-slice

### Meta
- {{"type": "checkpoint", "label": "<string>"}} — mark a point in the trace

## Strategy Tips
- Start with "status" to see the world state
- Place water near where you'll plant (or use the natural spring near center)
- Plant diverse species — mix trees, shrubs, groundcover, flowers
- Space plants apart (5+ voxels) so they don't crowd each other
- Tick in batches of 20-50 to see growth happen
- Use "inspect" to check specific voxels after growth
- Explore with camera: orbit to see different angles, cutaway to see underground roots
- A good session: setup terrain → plant → grow → observe → adjust → grow more

## Response Format
Return ONLY a JSON array of {batch_size} actions. No explanation, no markdown fences.
Example: [{{"type": "status"}}, {{"type": "place", "tool": "seed", "x": 38, "y": 40, "z": 50, "species": "oak"}}, {{"type": "tick", "n": 30}}]"#,
                batch_size = self.batch_size
            )
        }

        fn format_history(&self, history: &[ObservationEntry]) -> String {
            if history.is_empty() {
                return "No actions taken yet. This is the start of your session.".to_string();
            }

            // Show recent history (last 20 entries to keep context manageable)
            let start = history.len().saturating_sub(20);
            let recent = &history[start..];

            let mut lines = Vec::new();
            if start > 0 {
                lines.push(format!("(... {} earlier steps omitted ...)", start));
            }
            for entry in recent {
                lines.push(format!(
                    "Step {}: {} → {}",
                    entry.step_index, entry.action, entry.observation.text
                ));
            }
            lines.join("\n")
        }

        fn call_api(&self, history: &[ObservationEntry]) -> Result<String, String> {
            let system = self.build_system_prompt();
            let user_msg = format!(
                "Here is your session history so far:\n\n{}\n\nPlan your next {} actions.",
                self.format_history(history),
                self.batch_size
            );

            let body = serde_json::json!({
                "model": self.model,
                "max_tokens": 1024,
                "system": system,
                "messages": [
                    {"role": "user", "content": user_msg}
                ]
            });

            let resp = self
                .client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .map_err(|e| format!("API request failed: {e}"))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().unwrap_or_default();
                return Err(format!("API error {status}: {text}"));
            }

            let json: serde_json::Value =
                resp.json().map_err(|e| format!("Failed to parse response: {e}"))?;

            json["content"][0]["text"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "No text in API response".to_string())
        }

        fn parse_actions(&self, text: &str) -> Vec<Action> {
            // Strip markdown fences if present
            let text = text.trim();
            let text = if text.starts_with("```") {
                text.lines()
                    .skip(1)
                    .take_while(|l| !l.starts_with("```"))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                text.to_string()
            };

            let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&text) else {
                eprintln!("[LlmPlanner] Failed to parse response as JSON array: {text}");
                // Fallback: just do a status check
                return vec![Action::Status];
            };

            arr.iter().filter_map(|v| self.parse_single_action(v)).collect()
        }

        fn parse_single_action(&self, v: &serde_json::Value) -> Option<Action> {
            let action_type = v["type"].as_str()?;
            match action_type {
                "tick" => Some(Action::Tick {
                    n: v["n"].as_u64().unwrap_or(1),
                }),
                "place" => {
                    let tool = v["tool"].as_str()?.to_string();
                    let x = v["x"].as_u64()? as usize;
                    let y = v["y"].as_u64()? as usize;
                    let z = v["z"].as_u64()? as usize;
                    let species = v["species"].as_str().map(|s| s.to_string());
                    Some(Action::Place {
                        tool,
                        x,
                        y,
                        z,
                        species,
                    })
                }
                "fill" => {
                    let tool = v["tool"].as_str()?.to_string();
                    Some(Action::Fill {
                        tool,
                        x1: v["x1"].as_u64()? as usize,
                        y1: v["y1"].as_u64()? as usize,
                        z1: v["z1"].as_u64()? as usize,
                        x2: v["x2"].as_u64()? as usize,
                        y2: v["y2"].as_u64()? as usize,
                        z2: v["z2"].as_u64()? as usize,
                    })
                }
                "camera_orbit" => Some(Action::CameraOrbit {
                    theta_deg: v["theta_deg"].as_f64().unwrap_or(45.0),
                    phi_deg: v["phi_deg"].as_f64().unwrap_or(60.0),
                }),
                "camera_pan" => Some(Action::CameraPan {
                    x: v["x"].as_f64()?,
                    y: v["y"].as_f64()?,
                    z: v["z"].as_f64()?,
                }),
                "camera_zoom" => Some(Action::CameraZoom {
                    level: v["level"].as_f64().unwrap_or(1.0),
                }),
                "camera_cutaway" => Some(Action::CameraCutaway {
                    z: v["z"].as_f64()?,
                }),
                "camera_reset" => Some(Action::CameraReset),
                "status" => Some(Action::Status),
                "inspect" => Some(Action::Inspect {
                    x: v["x"].as_u64()? as usize,
                    y: v["y"].as_u64()? as usize,
                    z: v["z"].as_u64()? as usize,
                }),
                "view" => Some(Action::View {
                    z: v["z"].as_u64()? as usize,
                }),
                "checkpoint" => Some(Action::Checkpoint {
                    label: v["label"].as_str()?.to_string(),
                }),
                _ => {
                    eprintln!("[LlmPlanner] Unknown action type: {action_type}");
                    None
                }
            }
        }
    }

    impl Planner for LlmPlanner {
        fn plan(&mut self, history: &[ObservationEntry]) -> Vec<Action> {
            match self.call_api(history) {
                Ok(text) => {
                    let actions = self.parse_actions(&text);
                    self.steps_done += actions.len();
                    if actions.is_empty() {
                        vec![Action::Status]
                    } else {
                        actions
                    }
                }
                Err(e) => {
                    eprintln!("[LlmPlanner] API error: {e}");
                    // On API failure, do a safe fallback action
                    self.steps_done += 1;
                    vec![Action::Status]
                }
            }
        }

        fn should_stop(&self, _history: &[ObservationEntry]) -> bool {
            self.steps_done >= self.max_steps
        }

        fn name(&self) -> &str {
            "llm"
        }
    }
}

#[cfg(feature = "llm")]
pub use inner::LlmPlanner;

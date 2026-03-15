//! LLM-powered planner — uses Claude to decide actions autonomously.
//!
//! This module is gated behind the `llm` feature flag so the base crate
//! doesn't require API keys or network access to compile/test.

use crate::action::Action;
use crate::planner::{ObservationEntry, Planner};

use serde::Deserialize;

/// System prompt that frames the LLM as a gardener-player.
const SYSTEM_PROMPT: &str = r#"You are an autonomous player of GROUNDWORK, a cozy ecological voxel garden builder.

## Your Goal
Build a beautiful, self-sustaining garden. Plant diverse species near water, shape terrain, and observe your ecosystem grow.

## World
- Grid: 80×80×100 voxels (x, y horizontal; z vertical)
- Ground level: z=40 (surface). Below = underground, above = sky.
- The world starts with soil below z=40, air above, some water springs, and stone deep underground.

## Available Actions
Respond with a JSON array of actions. Each action is an object with a "type" field.

### Simulation
- {"type": "tick", "n": 10} — Advance simulation by N ticks (use 5-50)

### Tools (place materials)
- {"type": "place", "tool": "seed", "x": 40, "y": 40, "z": 41, "species": "oak"} — Plant a seed
- {"type": "place", "tool": "water", "x": 40, "y": 40, "z": 41} — Place water
- {"type": "place", "tool": "soil", "x": 40, "y": 40, "z": 41} — Place soil
- {"type": "place", "tool": "stone", "x": 40, "y": 40, "z": 41} — Place stone
- {"type": "place", "tool": "air", "x": 40, "y": 40, "z": 41} — Dig/remove
- {"type": "fill", "tool": "water", "x1": 38, "y1": 38, "z1": 41, "x2": 42, "y2": 42, "z2": 41} — Fill region

### Species (for seed tool)
Trees: oak, birch, willow, pine
Shrubs: fern, berry-bush, holly
Flowers: wildflower, daisy
Groundcover: moss, grass, clover

### Camera
- {"type": "orbit", "theta": 90, "phi": 45} — Orbit camera (theta: azimuth 0-360, phi: elevation 11-85)
- {"type": "pan", "x": 40, "y": 40, "z": 41} — Pan camera to position
- {"type": "zoom", "level": 1.5} — Zoom (0.3-4.0, 1.0=default)
- {"type": "cutaway", "z": 35} — View underground (z < 40 = underground, 100 = no cutaway)

### Observation
- {"type": "status"} — Get material counts
- {"type": "inspect", "x": 40, "y": 40, "z": 41} — Inspect single voxel
- {"type": "view", "z": 41} — View Z-slice as ASCII

## Tips
- Seeds need to be placed on or above soil (z=41 is one above surface)
- Water falls due to gravity; place it above surface
- Plant seeds near water for best growth
- Use status/inspect to check progress between tick batches
- Explore with camera: orbit to see different angles, cutaway to see roots underground
- Plant diverse species for a richer ecosystem

## Response Format
Reply with ONLY a JSON array of 3-8 actions. No explanation text. Example:
[{"type": "status"}, {"type": "fill", "tool": "water", "x1": 38, "y1": 38, "z1": 41, "x2": 42, "y2": 42, "z2": 41}, {"type": "place", "tool": "seed", "x": 40, "y": 40, "z": 41, "species": "oak"}, {"type": "tick", "n": 20}]
"#;

/// Configuration for the LLM planner.
pub struct LlmPlannerConfig {
    /// Anthropic API key.
    pub api_key: String,
    /// Model to use (default: claude-sonnet-4-20250514).
    pub model: String,
    /// Maximum tokens per response.
    pub max_tokens: u32,
    /// Maximum total steps before stopping.
    pub max_steps: usize,
}

impl Default for LlmPlannerConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            model: "claude-sonnet-4-20250514".into(),
            max_tokens: 1024,
            max_steps: 100,
        }
    }
}

/// An LLM-powered planner that uses Claude to decide actions.
pub struct LlmPlanner {
    config: LlmPlannerConfig,
    client: reqwest::blocking::Client,
    total_actions: usize,
}

impl LlmPlanner {
    pub fn new(config: LlmPlannerConfig) -> Self {
        let client = reqwest::blocking::Client::new();
        Self {
            config,
            client,
            total_actions: 0,
        }
    }

    /// Format observation history into a user message for Claude.
    fn format_history(history: &[ObservationEntry]) -> String {
        if history.is_empty() {
            return "This is the start of the session. No actions taken yet. Begin by surveying the world and then start building your garden.".to_string();
        }

        let mut lines = Vec::new();
        lines.push("Session history (most recent last):".to_string());

        // Show last 20 entries to keep context manageable
        let start = history.len().saturating_sub(20);
        if start > 0 {
            lines.push(format!("... ({start} earlier steps omitted) ..."));
        }

        for entry in &history[start..] {
            lines.push(format!(
                "Step {}: {} → {}",
                entry.step_index,
                entry.action,
                entry.observation.text.lines().next().unwrap_or("(empty)")
            ));
        }

        lines.push(String::new());
        lines.push("Choose your next batch of actions. Consider what you've learned and what would make the garden more beautiful and diverse.".to_string());

        lines.join("\n")
    }

    /// Call Claude API and get response text.
    fn call_claude(&self, user_message: &str) -> Result<String, String> {
        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "system": SYSTEM_PROMPT,
            "messages": [
                {"role": "user", "content": user_message}
            ]
        });

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("HTTP error: {e}"))?;

        let status = response.status();
        let text = response.text().map_err(|e| format!("Read error: {e}"))?;

        if !status.is_success() {
            return Err(format!("API error {status}: {text}"));
        }

        let parsed: ApiResponse =
            serde_json::from_str(&text).map_err(|e| format!("Parse error: {e}"))?;

        parsed
            .content
            .into_iter()
            .find(|c| c.content_type == "text")
            .map(|c| c.text)
            .ok_or_else(|| "No text in response".into())
    }

    /// Parse Claude's JSON response into actions.
    fn parse_actions(text: &str) -> Vec<Action> {
        // Extract JSON array from response (Claude might wrap it in markdown)
        let json_str = if let Some(start) = text.find('[') {
            if let Some(end) = text.rfind(']') {
                &text[start..=end]
            } else {
                return Vec::new();
            }
        } else {
            return Vec::new();
        };

        let raw: Vec<RawAction> = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        raw.into_iter().filter_map(|r| r.into_action()).collect()
    }
}

impl Planner for LlmPlanner {
    fn plan(&mut self, history: &[ObservationEntry]) -> Vec<Action> {
        if self.total_actions >= self.config.max_steps {
            return Vec::new();
        }

        let user_message = Self::format_history(history);

        match self.call_claude(&user_message) {
            Ok(response) => {
                let actions = Self::parse_actions(&response);
                self.total_actions += actions.len();
                actions
            }
            Err(e) => {
                eprintln!("LLM planner error: {e}");
                Vec::new()
            }
        }
    }

    fn should_stop(&self, _history: &[ObservationEntry]) -> bool {
        self.total_actions >= self.config.max_steps
    }
}

// --- API response types ---

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: String,
}

// --- Raw action parsing ---

#[derive(Deserialize)]
struct RawAction {
    #[serde(rename = "type")]
    action_type: String,
    // Tick
    n: Option<u64>,
    // Place/Fill/Inspect/View/Pan
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    // Place
    tool: Option<String>,
    species: Option<String>,
    // Fill
    x1: Option<f64>,
    y1: Option<f64>,
    z1: Option<f64>,
    x2: Option<f64>,
    y2: Option<f64>,
    z2: Option<f64>,
    // Orbit
    theta: Option<f64>,
    phi: Option<f64>,
    // Zoom
    level: Option<f64>,
}

impl RawAction {
    fn into_action(self) -> Option<Action> {
        match self.action_type.as_str() {
            "tick" => Some(Action::Tick {
                n: self.n.unwrap_or(10),
            }),
            "place" => Some(Action::Place {
                tool: self.tool.unwrap_or_else(|| "seed".into()),
                x: self.x? as usize,
                y: self.y? as usize,
                z: self.z? as usize,
                species: self.species,
            }),
            "fill" => Some(Action::Fill {
                tool: self.tool.unwrap_or_else(|| "water".into()),
                x1: self.x1? as usize,
                y1: self.y1? as usize,
                z1: self.z1? as usize,
                x2: self.x2? as usize,
                y2: self.y2? as usize,
                z2: self.z2? as usize,
            }),
            "orbit" => Some(Action::CameraOrbit {
                theta_deg: self.theta.unwrap_or(45.0),
                phi_deg: self.phi.unwrap_or(60.0),
            }),
            "pan" => Some(Action::CameraPan {
                x: self.x.unwrap_or(40.0),
                y: self.y.unwrap_or(40.0),
                z: self.z.unwrap_or(41.0),
            }),
            "zoom" => Some(Action::CameraZoom {
                level: self.level.unwrap_or(1.0),
            }),
            "cutaway" => Some(Action::CameraCutaway {
                z: self.z.unwrap_or(100.0),
            }),
            "status" => Some(Action::Status),
            "inspect" => Some(Action::Inspect {
                x: self.x? as usize,
                y: self.y? as usize,
                z: self.z? as usize,
            }),
            "view" => Some(Action::View {
                z: self.z.unwrap_or(41.0) as usize,
            }),
            "reset" => Some(Action::CameraReset),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_json_actions() {
        let json = r#"[
            {"type": "status"},
            {"type": "fill", "tool": "water", "x1": 38, "y1": 38, "z1": 41, "x2": 42, "y2": 42, "z2": 41},
            {"type": "place", "tool": "seed", "x": 40, "y": 40, "z": 41, "species": "oak"},
            {"type": "tick", "n": 20},
            {"type": "orbit", "theta": 90, "phi": 45},
            {"type": "cutaway", "z": 35},
            {"type": "inspect", "x": 40, "y": 40, "z": 41}
        ]"#;

        let actions = LlmPlanner::parse_actions(json);
        assert_eq!(actions.len(), 7);
        assert!(matches!(actions[0], Action::Status));
        assert!(matches!(actions[1], Action::Fill { .. }));
        assert!(matches!(actions[2], Action::Place { .. }));
        assert!(matches!(actions[3], Action::Tick { n: 20 }));
        assert!(matches!(actions[4], Action::CameraOrbit { .. }));
        assert!(matches!(actions[5], Action::CameraCutaway { .. }));
        assert!(matches!(actions[6], Action::Inspect { .. }));
    }

    #[test]
    fn parse_markdown_wrapped_json() {
        let response = "Here are my actions:\n```json\n[{\"type\": \"status\"}, {\"type\": \"tick\", \"n\": 5}]\n```";
        let actions = LlmPlanner::parse_actions(response);
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn parse_invalid_json_returns_empty() {
        let actions = LlmPlanner::parse_actions("not json at all");
        assert!(actions.is_empty());
    }

    #[test]
    fn parse_partial_actions_skip_invalid() {
        let json = r#"[{"type": "status"}, {"type": "inspect"}, {"type": "tick", "n": 5}]"#;
        let actions = LlmPlanner::parse_actions(json);
        // inspect without x/y/z should be skipped
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn format_empty_history() {
        let msg = LlmPlanner::format_history(&[]);
        assert!(msg.contains("start of the session"));
    }
}

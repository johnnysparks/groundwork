# Phase 2: Autonomous Actor — LLM-Powered Player Agent

## Context

GROUNDWORK is a cozy ecological voxel garden builder. You are extending the **embodied player agent framework** (`crates/groundwork-player/`) built in Phase 1.

Phase 1 delivered: deterministic scenario runner, trace recording, actor/oracle separation, programmatic evaluators, camera actions, and 7 scripted scenarios that pass. The actor sees only `Observation` text (like CLI output); evaluators see `OracleSnapshot` ground truth.

**Your job in Phase 2:** Replace scripted action sequences with an LLM-powered planner that reads observations and chooses actions autonomously. The planner should be able to play a session — plant seeds, shape terrain, explore the camera — and produce meaningful traces that evaluators can score.

## Key Architecture to Understand First

Read these files to understand what exists:

```
decisions/2026-03-14T14:00:00_embodied_player_agent.md   # Architecture & phases
crates/groundwork-player/src/action.rs                    # All Action variants
crates/groundwork-player/src/observer.rs                  # Observation struct + observe_* functions
crates/groundwork-player/src/oracle.rs                    # OracleSnapshot, CameraState, MaterialCounts
crates/groundwork-player/src/trace.rs                     # Trace, TraceStep
crates/groundwork-player/src/evaluator.rs                 # Evaluator trait + built-in evaluators
crates/groundwork-player/src/scenario.rs                  # Scenario + ScenarioBuilder
crates/groundwork-player/src/runner.rs                    # run(scenario) -> RunResult
crates/groundwork-player/src/scenarios/                   # Example scenarios for reference
agents/player.md                                          # Player role definition
AGENTS.md                                                 # Operating framework
```

## What to Build

### 1. Planner Module (`crates/groundwork-player/src/planner.rs`)

A `Planner` trait that takes observation history and returns the next action(s):

```rust
pub trait Planner {
    /// Given the history of observations so far, choose the next action(s).
    /// Returns a batch of actions to execute before re-planning.
    fn plan(&mut self, history: &[ObservationEntry]) -> Vec<Action>;

    /// Whether the planner wants to stop the session.
    fn should_stop(&self, history: &[ObservationEntry]) -> bool;
}

pub struct ObservationEntry {
    pub action: Action,           // What was done
    pub observation: Observation, // What was seen back (actor-visible only!)
    pub step_index: usize,
}
```

**Critical constraint:** The planner must NEVER see `OracleSnapshot`. It only sees `Observation.text` — the same text a human player would get. This is the actor/oracle separation.

### 2. LLM Planner (`crates/groundwork-player/src/planner_llm.rs`)

An implementation of `Planner` that uses Claude to decide actions:

- Format the observation history into a prompt describing what the player has seen
- Include the action space (all valid Action variants with their parameters)
- Include the game context (grid is 80×80×100, GROUND_LEVEL=40, 0.05m voxels, tools, species)
- Parse Claude's response into `Vec<Action>`
- Use a **slow cadence**: plan every N steps (e.g., every 5–10 actions), not every tick

The LLM prompt should frame the planner as a player who:
- Wants to build a beautiful, self-sustaining garden
- Explores by moving the camera (orbit, pan, zoom, cutaway underground)
- Plants diverse species near water
- Observes growth by ticking and checking status/inspect
- Has opinions about what looks good and what doesn't

### 3. Scripted Planner (`crates/groundwork-player/src/planner_scripted.rs`)

A simple deterministic `Planner` for testing the planner interface without LLM calls:
- Follows a fixed strategy (place water → plant seeds → tick → inspect → repeat)
- Useful for validating the planner→runner integration

### 4. Autonomous Runner (`crates/groundwork-player/src/runner.rs`)

Extend the existing `run()` function (or add `run_autonomous()`) to work with a `Planner`:

```rust
pub fn run_autonomous(
    planner: &mut dyn Planner,
    probes: &[Coord],
    evaluators: &[Box<dyn Evaluator>],
    max_steps: usize,
) -> RunResult {
    // 1. Create fresh world
    // 2. Loop:
    //    a. Ask planner for next action(s)
    //    b. Execute each action, record observation
    //    c. Feed observation back to planner history
    //    d. Take oracle snapshot (planner doesn't see this)
    //    e. Check if planner wants to stop or max_steps reached
    // 3. Evaluate trace with evaluators
    // 4. Return RunResult
}
```

### 5. Observation Formatting

The current `Observation.text` is designed for human CLI users. For LLM consumption, you may want to add a structured summary mode in `observer.rs`:

- `observe_status_structured()` → key-value pairs the LLM can parse reliably
- `observe_inspect_structured()` → JSON-like voxel data
- Keep the human-readable text too (for traces and debugging)

Or just tune the existing text format to be LLM-friendly — test what works.

### 6. Session Scenarios

Create scenarios that use the autonomous runner:

```rust
// In scenarios/autonomous.rs
pub fn autonomous_garden_session() -> AutonomousScenario {
    AutonomousScenario {
        name: "autonomous_garden",
        planner: Box::new(ScriptedPlanner::default()),
        max_steps: 50,
        probes: vec![...],
        evaluators: vec![
            Box::new(NoCrash),
            Box::new(MaterialMinimum::new("plant", 1)),
            Box::new(CameraOrbited),
            Box::new(CameraWentUnderground::new()),
        ],
    }
}
```

## Design Constraints

- **Slow planner, fast actor**: The LLM plans a batch of 5–10 actions at once, then the runner executes them all quickly. Don't call the LLM for every single tick.
- **Actor/oracle separation is sacred**: The planner NEVER sees MaterialCounts, VoxelProbe, or CameraState from oracle snapshots. It only sees Observation.text.
- **Determinism for scripted planner**: The scripted planner must produce identical traces on every run (for regression testing).
- **LLM planner is non-deterministic**: That's fine — evaluators judge the outcome, not the exact path.
- **Don't over-engineer the LLM integration**: Start with a simple prompt → parse loop. The prompt can be a static template with observation history injected. Use the Anthropic SDK directly.
- **Add the anthropic crate as an optional dependency** gated behind a feature flag (`llm`), so the base crate doesn't require API keys to compile/test.

## Testing Strategy

- Scripted planner tests run in CI (deterministic, no API key needed)
- LLM planner tests are `#[ignore]` by default, run manually with `cargo test -p groundwork-player -- --ignored`
- Both produce traces that serialize to JSON
- Both are judged by the same evaluators

## Validation

When done:
1. `cargo test -p groundwork-player` passes (all existing + new scripted planner tests)
2. `cargo test -p groundwork-sim` passes (no regressions)
3. The scripted planner can play a 50-step session and score > 0 on growth evaluators
4. The LLM planner (when API key is available) can play a session and produce coherent traces
5. Traces from both planners serialize to valid JSON
6. Write build notes to `build_notes/` and update the decision doc

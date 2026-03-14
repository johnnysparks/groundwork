# Build Notes: Autonomous Player Agent (Phase 2)
**Date:** 2026-03-14T16:00:00
**Dev:** Claude

## What Was Built

Phase 2 of the embodied player agent: autonomous planner-driven gameplay.

### New Files
- `crates/groundwork-player/src/planner.rs` — `Planner` trait + `ObservationEntry` struct
- `crates/groundwork-player/src/planner_scripted.rs` — Deterministic `ScriptedPlanner` for CI testing
- `crates/groundwork-player/src/planner_llm.rs` — Claude-powered `LlmPlanner` (behind `llm` feature flag)
- `crates/groundwork-player/src/scenarios/autonomous.rs` — Autonomous session scenario definitions

### Modified Files
- `crates/groundwork-player/Cargo.toml` — Added `llm` feature flag with optional `reqwest` + `tokio` deps
- `crates/groundwork-player/src/lib.rs` — Registered new modules (`planner`, `planner_scripted`, `planner_llm`)
- `crates/groundwork-player/src/runner.rs` — Added `run_autonomous()` function, made `execute_action()` public
- `crates/groundwork-player/src/scenarios/mod.rs` — Added `autonomous` module
- `crates/groundwork-player/tests/scenarios.rs` — Added 4 autonomous tests

## Architecture Decisions

### Planner Trait
```rust
pub trait Planner {
    fn plan(&mut self, history: &[ObservationEntry]) -> Vec<Action>;
    fn should_stop(&self, history: &[ObservationEntry]) -> bool;
}
```
- Planners receive only `ObservationEntry` (action + observation text) — never oracle data
- Return batches of actions for "slow planner, fast actor" cadence
- `should_stop` allows graceful session termination

### Scripted Planner
- 8 pre-computed batches: survey → water → plant → tick → camera → underground → final
- 24 total actions across batches, ~255 sim ticks
- Fully deterministic — verified by replay test
- Places seeds at z=50 (high above ground) matching pattern from existing scenarios

### LLM Planner
- Uses Claude API directly via `reqwest::blocking`
- System prompt frames the LLM as a gardener-player with full action space documentation
- Parses JSON arrays of actions from Claude's response
- Shows last 20 history entries to keep context manageable
- Gated behind `llm` feature so base crate doesn't need API keys

### run_autonomous()
- Creates fresh world, loops: planner → execute → record → check stop
- Feeds observation history back to planner (actor-visible only)
- Oracle snapshots recorded but never shown to planner
- Same `RunResult` type as `run()` — evaluators work identically

## Test Results
- `cargo test -p groundwork-player`: 15 tests, all pass (2 unit + 13 integration)
- `cargo test -p groundwork-sim`: 79 tests, all pass
- Autonomous scenarios: scripted planner scores >0 on plant growth evaluators
- Deterministic replay verified for scripted planner
- Trace serialization verified for autonomous runs

## Key Learnings
- Seeds must be placed well above ground (z=50) to use gravity landing correctly
- Water fill also needs to be above ground level to flow down
- The scripted planner's 24 actions with 255 ticks produce measurable plant growth (trunk, root, leaf)

## Next Steps
- Test LLM planner with API key: `cargo test -p groundwork-player --features llm -- --ignored`
- Add structured observation formatting for better LLM parsing
- Add more evaluators (species diversity, spatial coverage)
- Consider multi-turn conversation for LLM planner (accumulate messages)

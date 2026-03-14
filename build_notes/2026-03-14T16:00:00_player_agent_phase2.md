# Build Notes: Player Agent Phase 2 — Autonomous Planner Framework

**Date:** 2026-03-14
**Session:** Phase 2 of embodied player agent framework

## What Was Built

Extended `groundwork-player` with an autonomous planner system that replaces scripted action sequences with dynamic decision-making.

### New Files

1. **`planner.rs`** — `Planner` trait + `ObservationEntry` struct. Core abstraction: planners receive observation history (actor-visible only) and return batches of actions. Enforces actor/oracle separation at the type level.

2. **`planner_scripted.rs`** — `ScriptedPlanner`: deterministic planner for testing. Follows a fixed 6-phase strategy: status check → place water → plant 6 species → tick and observe → camera exploration (including underground cutaway) → final status. Produces identical traces on every run.

3. **`planner_llm.rs`** — `LlmPlanner`: Claude-powered autonomous planner (behind `llm` feature flag). Formats observation history into a prompt with full action space documentation, calls the Anthropic API, parses JSON response into actions. Slow cadence: plans batches of 5 actions, replans after execution. Falls back to `Status` on API errors.

### Modified Files

4. **`runner.rs`** — Added `run_autonomous()` function that loops: ask planner → execute batch → record observations (actor-visible for planner, oracle for evaluators) → check stop conditions. Same evaluator pipeline as scripted scenarios.

5. **`lib.rs`** — Added module exports for planner, planner_scripted, planner_llm (cfg-gated).

6. **`Cargo.toml`** — Added `llm` feature flag with optional `reqwest` dependency. Base crate compiles without API keys.

### Tests Added

- `autonomous_scripted_planner_session` — full 50-step session, verifies all evaluators pass (NoCrash, MaterialMinimum plant>=1, CameraOrbited, CameraWentUnderground)
- `autonomous_max_steps_limit` — verifies max_steps is respected
- `autonomous_scripted_deterministic` — verifies two runs produce identical oracle states
- `autonomous_trace_serialization` — verifies trace serializes to valid JSON

## Test Results

- `cargo test -p groundwork-player`: 13/13 passed (7 existing + 4 new + 2 utility)
- `cargo test -p groundwork-sim`: 79/79 passed (no regressions)
- `cargo check --workspace`: clean

## Key Design Decisions

- **Batch planning**: Planner returns `Vec<Action>` (batch), not single actions. The LLM plans 5 actions at once to amortize API latency.
- **Feature-gated LLM**: `reqwest` is optional behind `llm` feature. CI runs without API keys.
- **ObservationEntry never includes oracle data**: The type system makes it impossible for a planner to accidentally see OracleSnapshot.
- **Graceful degradation**: LLM planner falls back to `Status` on parse failures or API errors.

## What's Next

- Run LLM planner sessions manually with API key to validate trace quality
- Tune the system prompt based on observed LLM behavior
- Consider adding structured observation formatting for better LLM parsing
- Phase 3: self-improvement loop (task-setter, reward model, failure clustering)

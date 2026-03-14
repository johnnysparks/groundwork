# Decision: Embodied Player Agent Framework

**Date:** 2026-03-14
**Status:** Active
**Context:** The current player workflow is manual — a Claude agent plays the game, writes prose feedback, and hands off to the manager. This lacks reproducibility, programmatic evaluation, and the ability to improve over time.

## Problem

The player agent needs to be **embodied**: acting through the same interfaces a human player uses, judged by reproducible outcomes, and improving through a tight loop of action → trace → evaluation → adaptation.

## Design Principles

1. **Deterministic scenarios** — improvement is useless if failures can't be replayed exactly
2. **Actor view vs oracle truth** — the actor sees what a player sees (CLI output); evaluators access privileged sim state
3. **Every run is an artifact** — traces are raw material for debugging, scoring, and regression creation
4. **Evaluators, not vibes** — progress comes from verifiable reward: ground truth checks, programmatic assertions
5. **Scripted embodiment first** — a weak actor inside a strong evaluation harness is still progress
6. **Slow planner over fast actor** — long-horizon reasoning and low-latency control are different jobs
7. **Failures become code changes** — broken behavior becomes regression coverage

## Architecture

```
crates/groundwork-player/
  src/
    lib.rs              Public API
    scenario.rs         Scenario definition (actions + metadata)
    action.rs           Player actions (place, tick, inspect, etc.)
    runner.rs           Deterministic scenario executor
    trace.rs            Run trace recording (action→observation pairs)
    observer.rs         Actor-view observations (what the player sees)
    oracle.rs           Oracle-view state snapshots (privileged ground truth)
    evaluator.rs        Evaluator trait + built-in evaluators
    report.rs           Run report generation
  scenarios/            Scenario definitions as Rust tests
    basic_growth.rs     Seed→tree growth validation
    water_system.rs     Water flow and soil absorption
    ecosystem.rs        Multi-species interaction
```

### Key Types

- **Action** — what the player does (PlaceSeed, Tick, Dig, Fill, etc.)
- **Observation** — what the player sees back (CLI-like text output)
- **OracleSnapshot** — privileged sim state (material counts, specific voxel values)
- **Trace** — ordered list of (Action, Observation, OracleSnapshot) triples
- **Evaluator** — function from Trace → Score + Verdicts
- **Scenario** — name + description + action sequence + evaluators

### Actor/Oracle Separation

The **actor** (scenario script or future LLM planner) sees only Observations — the same text output a human player gets from `status`, `inspect`, `view`. It cannot peek at raw grid state.

The **oracle** (evaluators) can read full sim state: material counts, specific voxel values, soil composition, etc. This is how we score whether actions achieved their goals.

### Phases

#### Phase 1: Foundation (this session)
- `groundwork-player` crate with scenario/trace/evaluator framework
- Deterministic runner using sim API directly (not CLI subprocess)
- Built-in evaluators: material counts, growth detection, water presence
- 3 scripted scenarios with pass/fail evaluators
- Runs as `cargo test -p groundwork-player`

#### Phase 2: Autonomous Actor (future)
- LLM-powered planner that reads observations and chooses actions
- Observation formatting tuned for LLM consumption
- Action space definition (what the planner can do)
- Slow planner cadence (plan every N ticks) over fast actor (execute plan steps)

#### Phase 3: Self-Improvement Loop (future)
- Task-setter: generates new scenarios from observed failures
- Reward model: scores runs beyond pass/fail
- Failure clustering: groups similar failures for batch fixes
- Regression pipeline: failed scenarios become permanent tests
- Skill extraction: successful action patterns become reusable plans

# Build Notes: Embodied Player Agent Framework

**Date:** 2026-03-14
**Task:** Revamp automated player workflow with embodied agent architecture
**Status:** Phase 1 complete

## What was built

New crate `groundwork-player` — a framework for deterministic automated playtesting with programmatic evaluation.

### Core Architecture

**Actor/Oracle separation:**
- **Actor view** (observations): text output mirroring what a human player sees (status, inspect, view). The actor cannot peek at raw grid state.
- **Oracle view** (snapshots): privileged ground truth — material counts, voxel probes. Only evaluators see this.

**Key types:**
- `Action` — player actions (place, tick, fill, inspect, status, view, checkpoint)
- `Observation` — CLI-like text output the actor sees
- `OracleSnapshot` — privileged sim state with material counts + voxel probes
- `Trace` — ordered list of (action, observation, oracle) triples with timing
- `Evaluator` trait — judges a trace, returns Verdict (pass/fail + score + reason)
- `Scenario` — name + action sequence + probes + evaluators, with fluent builder

### Built-in Evaluators
- `MaterialMinimum` — material count >= threshold
- `MaterialAbsent` — material count == 0
- `MaterialGrew` — material count increased (optionally between checkpoints)
- `VoxelMaterial` — specific voxel has expected material
- `NoCrash` — run completed without panic
- `Custom` — closure-based evaluator

### Scenarios (all passing)
1. **seed_to_tree** — plant oak near spring, verify trunk + root growth
2. **multi_species_planting** — plant 4 species, verify some grow
3. **spring_exists** — verify default world has water
4. **water_flow_and_absorption** — pour water, verify it flows
5. **diverse_garden** — plant 6 species, verify ecosystem emerges
6. **deterministic_replay** — same scenario twice → identical oracle state
7. **trace_serialization** — traces serialize to valid JSON

### Design Decisions
- Runner uses sim API directly (not CLI subprocess) for speed and reliability
- Traces include wall-clock timing per step for performance profiling
- Checkpoints are labeled markers in the trace for evaluator targeting
- Scenarios use a fluent builder API for readability
- All scenarios are deterministic — same code → same results

## Roadmap (future phases)

### Phase 2: Autonomous Actor
- LLM-powered planner that reads observations and chooses actions
- Slow planner cadence (plan every N ticks) over fast actor execution
- Observation formatting tuned for LLM consumption

### Phase 3: Self-Improvement Loop
- Task-setter generates new scenarios from observed failures
- Failure clustering groups similar failures for batch fixes
- Failed scenarios become permanent regression tests
- Successful action patterns become reusable skills

## Validation
- `cargo test -p groundwork-player` — 7/7 tests pass
- `cargo test -p groundwork-sim` — 79/79 tests pass (no regressions)
- All traces serialize to valid JSON
- Deterministic replay verified

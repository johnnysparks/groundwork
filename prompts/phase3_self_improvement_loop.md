# Phase 3: Self-Improvement Loop — Failures Become Code Changes

## Context

GROUNDWORK is a cozy ecological voxel garden builder. You are extending the **embodied player agent framework** (`crates/groundwork-player/`).

**Phase 1** delivered: deterministic scenario runner, trace recording, actor/oracle separation, programmatic evaluators, camera actions, scripted scenarios.

**Phase 2** delivered: LLM-powered planner (reads observations, chooses actions), scripted planner (deterministic fallback), autonomous runner, structured observation formatting.

**Your job in Phase 3:** Close the loop. When the autonomous player fails, those failures should automatically become regression tests, clustered diagnostics, and actionable information for the dev agent. When it succeeds, those successes should become reusable skills. The endgame: a development loop where broken behavior becomes test coverage and better game systems.

## Key Architecture to Understand First

Read these files:

```
decisions/2026-03-14T14:00:00_embodied_player_agent.md   # Architecture & phases
crates/groundwork-player/src/                             # All modules — understand the full API
crates/groundwork-player/src/planner.rs                   # Planner trait (from Phase 2)
crates/groundwork-player/src/runner.rs                    # run() and run_autonomous()
crates/groundwork-player/src/trace.rs                     # Trace, TraceStep — the raw material
crates/groundwork-player/src/evaluator.rs                 # Evaluator trait + Verdict
crates/groundwork-player/src/scenarios/                   # Existing scenarios
agents/player.md                                          # Player role
agents/dev.md                                             # Dev role (failures feed into dev work)
agents/manager.md                                         # Manager role (prioritizes what to fix)
AGENTS.md                                                 # Operating framework, handoff formats
```

## What to Build

### 1. Trace Analysis (`crates/groundwork-player/src/analysis.rs`)

Tools for mining traces after runs complete:

```rust
/// Summary of a run for reporting and clustering.
pub struct RunSummary {
    pub scenario_name: String,
    pub passed: bool,
    pub verdicts: Vec<Verdict>,
    pub aggregate_score: f64,         // Average of all verdict scores
    pub step_count: usize,
    pub final_material_counts: MaterialCounts,
    pub camera_coverage: CameraCoverage,  // How much the player explored
    pub action_histogram: HashMap<String, usize>,  // Action type counts
    pub growth_timeline: Vec<(u64, u64)>,  // (tick, total_plant_count)
    pub failure_signatures: Vec<String>,   // Machine-readable failure descriptors
}

/// How thoroughly the camera explored the world.
pub struct CameraCoverage {
    pub angle_range: (f64, f64),       // min/max theta
    pub elevation_range: (f64, f64),   // min/max phi
    pub zoom_range: (f64, f64),
    pub cutaway_range: (f64, f64),     // min/max cutaway Z
    pub went_underground: bool,
    pub total_camera_actions: usize,
}
```

Key functions:
- `summarize(result: &RunResult) -> RunSummary` — extract summary from a run
- `growth_timeline(trace: &Trace) -> Vec<(u64, u64)>` — track plant count over ticks
- `failure_signature(verdict: &Verdict, trace: &Trace) -> String` — machine-readable failure ID

### 2. Failure Clustering (`crates/groundwork-player/src/clustering.rs`)

Group similar failures across multiple runs for batch diagnosis:

```rust
pub struct FailureCluster {
    pub signature: String,           // e.g. "no_growth_after_200_ticks"
    pub count: usize,
    pub example_traces: Vec<String>, // Paths to JSON trace files
    pub common_conditions: String,   // What these failures share
    pub suggested_investigation: String,
}

/// Cluster failures from multiple run summaries.
pub fn cluster_failures(summaries: &[RunSummary]) -> Vec<FailureCluster>;
```

The clustering should identify patterns like:
- "Seeds planted but never grew" → might be a water proximity issue
- "Trees grew but no leaves appeared" → might be a light propagation issue
- "Water disappeared after N ticks" → the spring drying up bug
- "Camera never went underground" → planner doesn't explore the signature feature

### 3. Regression Generator (`crates/groundwork-player/src/regression.rs`)

Turn failed traces into permanent regression test scenarios:

```rust
/// Generate a deterministic scenario from a failed trace.
/// Replays the exact same actions and adds evaluators for the expected outcome.
pub fn trace_to_regression(
    trace: &Trace,
    failed_verdicts: &[Verdict],
    name: &str,
) -> Scenario {
    // 1. Extract the action sequence from the trace
    // 2. Add evaluators matching the failed verdicts (inverted — the regression
    //    passes when the original failure no longer reproduces)
    // 3. Return a Scenario that can be added to the test suite
}

/// Write a regression scenario as a Rust source file.
pub fn write_regression_file(
    scenario: &Scenario,
    output_path: &Path,
) -> std::io::Result<()>;
```

When a failure is found:
1. The trace captures the exact action sequence
2. `trace_to_regression` generates a scenario that replays those actions
3. The scenario includes evaluators that *should* pass once the bug is fixed
4. The regression file is written to `crates/groundwork-player/src/scenarios/regressions/`
5. The dev agent can run `cargo test -p groundwork-player` to see the failure, fix the sim, and watch the test go green

### 4. Skill Extraction (`crates/groundwork-player/src/skills.rs`)

Extract successful action patterns from passing traces that the planner can reuse:

```rust
/// A reusable action pattern extracted from a successful trace.
pub struct Skill {
    pub name: String,
    pub description: String,
    pub preconditions: String,        // When to use this skill
    pub actions: Vec<Action>,         // The action sequence
    pub expected_outcome: String,     // What should happen
    pub source_scenario: String,      // Where this was learned
    pub success_rate: f64,            // How often this works (from multiple runs)
}

/// Extract skills from a set of successful run summaries.
pub fn extract_skills(summaries: &[RunSummary], traces: &[Trace]) -> Vec<Skill>;
```

Skills should capture patterns like:
- "Plant near water": place seed within 4 voxels of water source → seed grows
- "Underground exploration": cutaway to z=25, orbit 360°, zoom 2x → see root systems
- "Water basin": fill water in a 5×5 area at surface level → creates growing zone
- "Diverse planting": plant 4+ different species within 10 voxels → ecosystem emerges

Skills are written as text descriptions that can be injected into the LLM planner's prompt (Phase 2), giving it a library of known-good strategies.

### 5. Report Generator (`crates/groundwork-player/src/report.rs`)

Generate structured reports from batch runs that feed into the manager/dev workflow:

```rust
/// Generate a player→manager handoff from batch run results.
pub fn generate_player_handoff(
    summaries: &[RunSummary],
    clusters: &[FailureCluster],
) -> String {
    // Format as the AGENTS.md player→manager handoff:
    // Observed | Felt | Bugs | Confusions | What kept me playing | What stopped me | Requests
}

/// Generate a dev-actionable bug report from a failure cluster.
pub fn generate_bug_report(cluster: &FailureCluster) -> String {
    // Format as the agents/player.md bug report:
    // Title | Severity | Steps to reproduce | Expected | Actual | Frequency | Notes
}
```

This closes the agent loop: the player agent produces structured feedback in exactly the format the manager agent expects, which becomes dev tasks.

### 6. Batch Runner (`crates/groundwork-player/src/batch.rs`)

Run multiple scenarios (or multiple autonomous sessions) and collect results:

```rust
pub struct BatchResult {
    pub summaries: Vec<RunSummary>,
    pub clusters: Vec<FailureCluster>,
    pub regressions_generated: Vec<PathBuf>,
    pub skills_extracted: Vec<Skill>,
    pub report: String,
}

/// Run all scenarios in batch and produce a full analysis.
pub fn run_batch(scenarios: &[Scenario], trace_dir: &Path) -> BatchResult {
    // 1. Run each scenario, save traces as JSON to trace_dir
    // 2. Summarize all results
    // 3. Cluster failures
    // 4. Generate regressions for new failures
    // 5. Extract skills from successes
    // 6. Generate player→manager report
}
```

### 7. Task Setter (`crates/groundwork-player/src/task_setter.rs`)

Generate new scenarios from failure patterns — the agent proposes what to test next:

```rust
/// Given failure clusters, generate new targeted scenarios that probe the issue.
pub fn generate_investigation_scenarios(
    clusters: &[FailureCluster],
) -> Vec<Scenario> {
    // For "seeds don't grow" → generate scenarios with seeds at various distances
    //   from water, at various depths, with various species
    // For "water disappears" → generate scenarios tracking water count over time
    //   with different water amounts and configurations
    // For "camera issues" → generate scenarios testing specific camera paths
}
```

This is the self-improvement engine: the system discovers what's broken, generates tests that probe the failure boundary, and the results tell the dev exactly where the bug is.

## Design Constraints

- **Traces are the source of truth**: Everything flows from recorded traces. Analysis, clustering, regression generation, skill extraction — all operate on `Trace` and `RunSummary` data.
- **Don't require LLM for analysis**: Clustering, regression generation, and report generation should be purely programmatic. LLM is only used in the Phase 2 planner (and optionally for skill description generation).
- **Regressions must be deterministic**: Generated regression scenarios replay exact action sequences. They must pass/fail identically on every run.
- **Reports match AGENTS.md formats**: Generated handoffs and bug reports must follow the exact section structure from `AGENTS.md` and `agents/player.md` so the manager and dev agents can consume them directly.
- **Trace storage is JSON files**: Save traces to `artifacts/traces/` with timestamped filenames. These are git-committable artifacts.
- **Skills are text, not code**: Skills are described in natural language for LLM consumption, not compiled Rust strategies. They're injected into the planner prompt.
- **Keep it incremental**: Start with trace analysis + failure clustering + regression generation. Skill extraction and task setter can be simpler initially and refined later.

## Testing Strategy

- Unit tests for `summarize()`, `cluster_failures()`, `trace_to_regression()`
- Integration test: run batch of existing scenarios → verify summaries/clusters are sensible
- Integration test: generate a regression from a trace → verify the regression scenario replays correctly
- All tests deterministic, no API keys needed

## Validation

When done:
1. `cargo test -p groundwork-player` passes (all existing + new tests)
2. `cargo test -p groundwork-sim` passes (no regressions)
3. `run_batch()` processes all existing scenarios and produces a valid `BatchResult`
4. A failing scenario generates a regression `.rs` file that compiles and fails appropriately
5. A passing scenario produces at least one extracted skill
6. The generated player→manager report follows AGENTS.md format
7. Trace JSON files are written to `artifacts/traces/`
8. Write build notes to `build_notes/` and update the decision doc

## The Big Picture

After Phase 3, the development loop becomes:

```
Player agent plays → traces recorded → evaluators score →
  failures cluster → regressions generated → dev fixes sim →
    regressions go green → skills extracted → planner improves →
      player agent plays better → (loop)
```

Each iteration makes the game better AND the player agent smarter. Broken behavior becomes test coverage. Successful behavior becomes reusable knowledge. The system improves itself.

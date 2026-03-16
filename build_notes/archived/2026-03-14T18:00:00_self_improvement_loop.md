# Build Notes: Self-Improvement Loop (Phase 3)
**Date:** 2026-03-14T18:00:00
**Dev:** Claude

## What Was Built

Phase 3 of the embodied player agent: closing the loop so failures become test coverage and successes become reusable knowledge.

### New Files (7 modules)

- **`analysis.rs`** — `RunSummary`, `CameraCoverage`, `summarize()`, `growth_timeline()`, `failure_signature()`
- **`clustering.rs`** — `FailureCluster`, `cluster_failures()` with diagnosis and investigation suggestions
- **`regression.rs`** — `trace_to_regression()` converts failed traces into deterministic `Scenario` replay tests; `write_regression_file()` emits Rust source
- **`skills.rs`** — `Skill` struct, `extract_skills()` mines water/planting/camera patterns from successes, `skills_to_prompt()` formats for LLM injection
- **`report.rs`** — `generate_player_handoff()` follows AGENTS.md format (Observed|Felt|Bugs|Confusions|What kept me playing|What stopped me|Requests); `generate_bug_report()` produces dev-actionable reports
- **`batch.rs`** — `run_batch()` runs multiple scenarios, saves JSON traces, clusters failures, generates regressions, extracts skills, produces report
- **`task_setter.rs`** — `generate_investigation_scenarios()` creates targeted scenarios from failure clusters (probes seed-water distance, species, water persistence, camera behavior)

### Modified Files
- **`lib.rs`** — Registered all 7 new Phase 3 modules
- **`analysis.rs`**, **`batch.rs`** — Fixed unused import warnings

## Architecture

### Data Flow
```
Scenarios → runner::run() → RunResult
                              ↓
                        analysis::summarize() → RunSummary
                              ↓
                    clustering::cluster_failures() → FailureCluster[]
                        ↙                    ↘
          regression::trace_to_regression()   skills::extract_skills()
                  ↓                                    ↓
          Scenario (deterministic replay)       Skill[] (for LLM prompt)
                        ↘                    ↙
                   report::generate_player_handoff()
                              ↓
                    Player → Manager handoff text
```

### batch::run_batch() — the one-call entry point
1. Runs each scenario, saves traces as JSON
2. Summarizes all results
3. Clusters failures by signature
4. Generates regression .rs files for failures
5. Extracts skills from successes
6. Generates player→manager report

### Failure Signatures
Machine-readable IDs like `no_growth_after_200_ticks`, `insufficient_water_after_50_ticks`, `camera_issue:camera_orbited`. These cluster naturally across runs.

### Skill Extraction
Three pattern detectors:
- `water_basin` — Fill actions that result in persistent water
- `*_planting_near_water` — Seed placement patterns with species diversity
- `full_exploration` / `surface_exploration` — Camera strategies

### Task Setter
Generates investigation scenarios from failure patterns:
- "no_growth" → 4 scenarios (adjacent water, far water, species diversity, long growth)
- "insufficient_water" → 2 scenarios (single voxel, basin)
- "camera_issue" → 2 scenarios (orbit, underground)
- "material_did_not_grow" → 1 targeted scenario per material

## Test Results
- `cargo test -p groundwork-player`: **32 tests, all pass** (19 unit + 13 integration)
- `cargo test -p groundwork-sim`: **79 tests, all pass** (no regressions)
- No compiler warnings

## The Development Loop (Now Complete)
```
Player agent plays → traces recorded → evaluators score →
  failures cluster → regressions generated → dev fixes sim →
    regressions go green → skills extracted → planner improves →
      player agent plays better → (loop)
```

## Next Steps
- Wire skills into LLM planner prompt (Phase 2 planner_llm.rs)
- Add CLI command to run batch and save artifacts to `artifacts/traces/`
- Run investigation scenarios as part of CI
- Consider auto-committing regression files to `scenarios/regressions/`

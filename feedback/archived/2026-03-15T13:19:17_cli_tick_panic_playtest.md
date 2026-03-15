# Feedback: CLI Fallback Playtest — Tick Panic Blocks Session

**Date:** 2026-03-15
**Build path tested:** CLI fallback (`groundwork-tui` commands)
**Session length:** ~10 minutes
**Context:** Web UI could not be launched in this sandbox (`vite` listen EPERM on `127.0.0.1:5173`), so this session used the documented CLI fallback.

## 1. What the game sold me
A cozy ecological garden where I can place seeds, run time, observe interactions, and learn from visible cause-and-effect.

## 2. What I actually experienced
I can create a world and place species seeds, but any `tick` command panics immediately due a missing ECS resource. That means no growth, no interaction chain, and no playable loop in this environment.

## 3. Best moments
- World initialization works (`new` + `status`), and inspect output gives useful soil composition details at different depths.
- Tool verbs are clear (`place`, `inspect`, `status`) and species placement appears to persist in state (`seed: 5` after placement).

## 4. Surprises — things the garden did that I didn't plan
- None observed. Tick panic prevents simulation progression, so emergence cannot occur.

## 5. Confusing moments
- Water placement at high z reported "Nothing placed" without explaining where blocked occupancy was detected.
- `tick` panic does not identify which gameplay system/resource is missing (system names are hidden without debug feature), which makes player-facing diagnostics opaque.

## 6. Boring or frustrating moments
- Session cannot progress beyond setup. Repeated crash on tick makes the core loop untestable.

## 7. Signs of life — fauna, movement, autonomous garden behavior
- None observable because simulation never advances past tick 0.

## 8. What I learned about the ecosystem
- Surface appears around z=40 in this world shape.
- Soil inspection data is rich (type, pH, bacteria, drainage, retention, nutrients).
- I could not learn ecological dynamics because ticking crashes.

## 9. Bugs
### BUG-1: Tick panics on fresh world (Blocker)
- **Severity:** blocker
- **Steps to reproduce:**
  1. `cargo run -p groundwork-tui -- new`
  2. `cargo run -p groundwork-tui -- tick 1`
- **Expected result:** tick advances to 1 and simulation updates
- **Actual result:** panic from Bevy ECS resource validation (`Resource does not exist`)
- **Frequency:** 100%
- **Notes:** also reproduces after placing species seeds (`oak`, `birch`, `clover`, `wildflower`, `daisy`).

### BUG-2: Web UI dev server cannot start in sandbox (Major for this run context)
- **Severity:** major
- **Steps to reproduce:** `cd crates/groundwork-web && npm run dev -- --host 127.0.0.1 --port 5173`
- **Expected result:** local dev server starts
- **Actual result:** `listen EPERM: operation not permitted 127.0.0.1:5173`
- **Frequency:** 100% in this environment
- **Notes:** likely environment restriction, but it blocks primary-interface playtesting in automation.

## 10. Feature or clarity requests
1. Make missing-resource panic self-identifying in non-debug builds (print system label/resource type) to speed triage.
2. Add guard behavior when required resources are absent (skip system with clear warning instead of hard crash).
3. Improve tool placement failure messages with reason granularity (occupied vs out-of-bounds vs protected).

## 11. Evaluation Scores
| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 2/5 | Initial setup is clear but loop stops immediately. |
| Clarity of cause and effect | 1/5 | No causal chain visible without successful tick. |
| Tactile satisfaction | 2/5 | Placement commands are straightforward. |
| Beauty/readability | 2/5 | CLI inspect/status are readable but static. |
| Ecological fantasy delivery | 1/5 | No living simulation observed. |
| Desire to keep playing | 1/5 | Crash on first tick ends session. |
| Friction / confusion | 1/5 | Blocker crash with opaque system origin. |
| Trust in the simulation | 1/5 | Deterministic panic undermines trust. |
| Surprise / emergence | 1/5 | None observed. |
| Sense of life | 1/5 | None observed. |
| Discovery arc | 1/5 | No progression possible. |
| Garden autonomy | 1/5 | Tick 0 only. |

## 12. Brutal bottom line
I would not come back tomorrow in this build state because the play loop crashes before any ecology appears. The immediate next priority is restoring stable ticking; without that, no interaction, readability, or delight work can be validated.

## Screenshots
- None captured in this session (CLI fallback, no browser/UI capture path available in sandbox).

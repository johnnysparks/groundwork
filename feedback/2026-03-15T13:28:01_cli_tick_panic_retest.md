# Feedback: CLI Fallback Retest — Tick Panic Still Blocks Core Loop

**Date:** 2026-03-15  
**Build path tested:** Web attempt + CLI fallback (`groundwork-tui` commands)  
**Session length:** ~12 minutes  
**Context:** Web UI remains unlaunchable in this sandbox (`vite` bind EPERM on `127.0.0.1:5173`), so session was completed through CLI fallback.

## 1. What the game sold me
A cozy ecological sandbox where I can plant, tick time forward, and learn from visible chain reactions.

## 2. What I actually experienced
World setup and inspection tools work, but simulation progression is still hard-blocked: `tick 1` panics with a missing ECS resource. I could place seeds and inspect soil, but no growth or interactions can happen.

## 3. Best moments
- `new` + `status` are reliable and fast.
- `inspect` still gives rich soil composition (type, pH, bacteria, drainage, retention, nutrients).
- Placement failure text is slightly clearer than previous run: `Nothing placed (1 cells skipped — occupied, protected, or out of bounds)`.

## 4. Surprises — things the garden did that I didn't plan
- None observed. Tick blocker prevents any autonomous behavior.

## 5. Confusing moments
- Surface/depth expectations are unclear in CLI flow: placing seeds at z=35 failed due occupancy, but z=60 succeeded.
- Placement failure reason is still bundled; I still can't tell which exact reason applied per cell.
- Panic still hides the failing system/resource because debug names are disabled.

## 6. Boring or frustrating moments
- Session ends at first tick attempt. Repro is immediate and repeatable.

## 7. Signs of life — fauna, movement, autonomous garden behavior
- None observable because tick never advances past 0.

## 8. What I learned about the ecosystem
- Soil diagnostics remain strong.
- Seed placement and persistence work pre-tick.
- I could not validate any ecological dynamics, interactions, or recovery behavior.

## 9. Bugs
### BUG-1: Tick panics on fresh world (Blocker)
- **Severity:** blocker
- **Steps to reproduce:**
  1. `cargo run -p groundwork-tui -- new`
  2. `cargo run -p groundwork-tui -- place seed 40 40 60 oak` (optional)
  3. `cargo run -p groundwork-tui -- tick 1`
- **Expected result:** Tick advances and world state updates.
- **Actual result:** Panic from Bevy ECS validation: `Resource does not exist`.
- **Frequency:** 100%
- **Notes:** Reproduced again in this run after successful seed placement and `status` confirmation (`seed: 4`).

### BUG-2: Web UI dev server blocked in automation sandbox (Major for this environment)
- **Severity:** major (environment-specific)
- **Steps to reproduce:**
  1. `cd crates/groundwork-web`
  2. `npm run dev -- --host 127.0.0.1 --port 5173`
- **Expected result:** Vite server starts.
- **Actual result:** `Error: listen EPERM: operation not permitted 127.0.0.1:5173`
- **Frequency:** 100% in this sandbox.
- **Notes:** Prevents primary-interface playtesting in this automation context.

## 10. Feature or clarity requests
1. Keep P0 focus on fixing missing-resource tick panic.
2. Add non-debug fallback diagnostics that print failing system label/resource type.
3. Split placement failure reasons into specific counts (`occupied`, `protected`, `out_of_bounds`) for faster player diagnosis.

## 11. Evaluation Scores
| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 2/5 | Setup is clean, but no progression. |
| Clarity of cause and effect | 1/5 | No causal chain possible without tick. |
| Tactile satisfaction | 2/5 | Commands are responsive pre-crash. |
| Beauty/readability | 2/5 | CLI data is readable but static. |
| Ecological fantasy delivery | 1/5 | Ecology loop never begins. |
| Desire to keep playing | 1/5 | Crash on first tick ends motivation. |
| Friction / confusion | 1/5 | Immediate blocker + opaque panic source. |
| Trust in the simulation | 1/5 | Deterministic crash at core interaction. |
| Surprise / emergence | 1/5 | None observed. |
| Sense of life | 1/5 | None observed. |
| Discovery arc | 1/5 | No progression beyond setup. |
| Garden autonomy | 1/5 | Tick 0 only. |

## 12. Brutal bottom line
I would not come back tomorrow on this build path because the core loop still crashes before any ecology is visible. The highest-value next step is unchanged: restore stable ticking first, then re-run web-first playtesting for interaction/fantasy validation.

## Screenshots
- None captured this run (browser playtest blocked in sandbox; CLI fallback only).

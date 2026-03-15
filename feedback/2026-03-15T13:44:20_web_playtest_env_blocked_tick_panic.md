# Feedback: Web Playtest Attempt Blocked by Sandbox + Tick Panic Still P0

**Date:** 2026-03-15  
**Session length:** ~20 minutes  
**Build path tested:** Web-first attempts (`groundwork-web`) + CLI fallback (`groundwork-tui`)

## 1. What the game sold me
A cozy ecological sandbox where I can observe visible cause-and-effect chains in a living garden.

## 2. What I actually experienced
I could not launch a runnable local web session in this automation sandbox because all localhost binds are denied (`EPERM`) for both Vite and `python -m http.server`. I also could not complete the repo screenshot harness because no browser binary is available and installation is blocked. CLI fallback still reproduces a deterministic `tick 1` panic.

Screenshot status for this run:
- No new web screenshots captured (environment-blocked).
- For visual baseline context only (from earlier session artifacts): `artifacts/screenshots/2026-03-15T12-42-37_session/*.png`.

## 3. Best moments
- `cargo run -p groundwork-tui -- new` + `status` gives clean world bootstrap.
- `place seed 40 40 60 oak` executes and is reflected in `status` (`seed: 1`).
- `inspect` remains detailed and useful for soil diagnosis.

## 4. Surprises — things the garden did that I didn't plan
- None observed in this run because simulation progression remains blocked at first tick.

## 5. Confusing moments
- Seed placed at z=60 did not persist there (`inspect` at z=60 returned air), presumably due gravity/fall behavior; this is probably correct but still unclear from CLI feedback.
- Panic output still omits system/resource names in this build, so diagnosis from player-facing output is weak.

## 6. Boring or frustrating moments
- Session dead-ends at first simulation step (`tick 1`), so there is still no way to validate ecology loops in this environment.

## 7. Signs of life — fauna, movement, autonomous garden behavior
- None observable in this run due hard blocker before any tick progression.

## 8. What I learned about the ecosystem
- Setup/placement/inspection are stable pre-tick.
- Could not validate growth, interaction chains, fauna emergence, or recovery dynamics.

## 9. Bugs
### BUG-1: Fresh-world tick panic (Blocker)
- **Severity:** blocker
- **Steps to reproduce:**
  1. `cargo run -p groundwork-tui -- new`
  2. `cargo run -p groundwork-tui -- tick 1`
- **Expected result:** tick advances world state.
- **Actual result:** panic from Bevy ECS validation (`Resource does not exist`).
- **Frequency:** 100%
- **Notes:** still reproducible on current local branch state.

### BUG-2: Local web serving blocked in automation sandbox (Major, environment)
- **Severity:** major (environment-specific, blocks primary interface testing here)
- **Steps to reproduce:**
  1. `cd crates/groundwork-web && npm run dev -- --host 127.0.0.1 --port 5173`
  2. `cd crates/groundwork-web/dist && python3 -m http.server 8000`
- **Expected result:** local web server starts.
- **Actual result:** bind fails with `PermissionError/EPERM`.
- **Frequency:** 100% in this sandbox.
- **Notes:** prevents browser play in this automation context.

### BUG-3: Screenshot harness cannot self-heal in sandbox (Major, environment)
- **Severity:** major (environment-specific)
- **Steps to reproduce:**
  1. `cd crates/groundwork-web && ./screenshot.sh --quick`
- **Expected result:** script finds/installs browser and captures screenshots.
- **Actual result:** `FATAL: Could not find or install a browser.`
- **Frequency:** 100% in this sandbox.

## 10. Feature or clarity requests
1. Keep P0 focus on fixing the missing ECS resource so `tick` works.
2. Add non-debug panic context (system name/resource type) to player-visible output.
3. Improve sandbox-resilient web playtest path (non-socket preview mode or in-process screenshot runner).

## 11. Evaluation Scores
| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 2/5 | Setup feels real, then hard stop. |
| Clarity of cause and effect | 1/5 | No simulation progression available. |
| Tactile satisfaction | 2/5 | Commands respond pre-crash. |
| Beauty/readability | 2/5 | Can only reference prior visuals; no fresh web render this run. |
| Ecological fantasy delivery | 1/5 | Core ecological loop remains blocked. |
| Desire to keep playing | 1/5 | Immediate progression failure. |
| Friction / confusion | 1/5 | Environment + tick blockers stack. |
| Trust in the simulation | 1/5 | Deterministic panic at first tick. |
| Surprise / emergence | 1/5 | None observable. |
| Sense of life | 1/5 | None observable. |
| Discovery arc | 1/5 | No new ecological discovery possible. |
| Garden autonomy | 1/5 | Tick cannot advance. |

## 12. Brutal bottom line
I would not come back tomorrow on this build path. Two gates remain: this sandbox cannot host local web play, and the fallback CLI path still crashes at the first tick.

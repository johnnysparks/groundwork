# Web Playtest Report — Session blocked at loading screen

Date: 2026-03-15  
Build: `crates/groundwork-web` via `npm run dev -- --host 0.0.0.0 --port 5173`

## What the game sold me
A cozy, readable ecological sandbox where I can plant, observe above/below-ground cause-and-effect, and get surprised by autonomous ecosystem behavior.

## What I actually experienced *(screenshot: first impression)*
I could not enter gameplay. The app stayed on a full-screen loading state (`Growing your garden...`) for the entire session.

Screenshot:
- `browser:/tmp/codex_browser_invocations/d83bd7ff901cb3a6/artifacts/artifacts/playtest_01_first_impression.png`

## Best moments *(screenshot each)*
- The warm visual tone of the loading screen communicates the intended cozy direction.

Screenshot:
- `browser:/tmp/codex_browser_invocations/d83bd7ff901cb3a6/artifacts/artifacts/playtest_02_after_5s_autotick.png`

## Surprises — things the garden did that I didn't plan *(screenshot each)*
- None observed; gameplay never loaded.

## Confusing moments *(screenshot each)*
- Pressing expected controls (Space, camera orbit, interaction clicks) gave no visible change while loading remained.

Screenshot:
- `browser:/tmp/codex_browser_invocations/d83bd7ff901cb3a6/artifacts/artifacts/playtest_05_tool_use_attempt.png`

## Boring or frustrating moments *(screenshot if visual)*
- Waiting through repeated attempts with no transition out of loading.

Screenshot:
- `browser:/tmp/codex_browser_invocations/d83bd7ff901cb3a6/artifacts/artifacts/playtest_04_underground_attempt.png`

## Signs of life — fauna, movement, autonomous garden behavior
- None visible due to blocked render/game initialization.

## What I learned about the ecosystem
- Nothing in this session; no simulation visuals or interactions became available.

## Evaluation lens scores (1-5)
- First-impression hook: **2/5** (cozy tone, but non-interactive)
- Clarity of cause and effect: **1/5**
- Tactile satisfaction: **1/5**
- Beauty/readability: **2/5** (loading screen only)
- Ecological fantasy delivery: **1/5**
- Desire to keep playing: **1/5**
- Friction/confusion: **5/5** (high friction)
- Trust in the simulation: **1/5** (cannot verify simulation)
- Surprise/emergence: **1/5**
- Sense of life: **1/5**
- Discovery arc: **1/5**
- Garden autonomy: **1/5**

## Bugs

### 1) Web UI never leaves loading screen in this environment
- **Severity:** blocker
- **Steps to reproduce:**
  1. `cd crates/groundwork-web`
  2. `npm install`
  3. `npm run dev -- --host 0.0.0.0 --port 5173`
  4. Open app in browser automation session
- **Expected result:** Enter interactive 3D garden with HUD/tools
- **Actual result:** Permanent loading state: `Growing your garden...`
- **Frequency:** 100% in this session
- **Notes:** Console logs captured:
  - `Loading module ... /wasm/groundwork_sim.js was blocked because of a disallowed MIME type`  
  - `THREE.WebGLRenderer: Error creating WebGL context` / `AllowWebgl2:false restricts context creation on this system.`

## Feature or clarity requests
1. Detect WebGL/WebGL2 failure and show explicit fallback message in UI (instead of infinite loading).
2. Detect WASM module load/MIME failure and show an actionable error panel with next steps.
3. Consider automatic non-WebGL fallback renderer or test mode so player sessions can still validate core loop in constrained environments.

## Brutal bottom line: would I come back tomorrow — and why?
Not yet. I couldn't enter the game, so I have no basis for curiosity about the garden itself. I'd return once the loading blocker is fixed or a fallback mode exists.

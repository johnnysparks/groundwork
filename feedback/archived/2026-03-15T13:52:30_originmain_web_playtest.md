# Feedback: Origin/Main Web Playtest After WASM Recovery

**Date:** 2026-03-15
**Session length:** ~25 minutes
**Build path tested:** `origin/main` worktree -> `cd crates/groundwork-web && npm install && npm run wasm && npm run dev`
**Screenshots:** Captured via Playwright and stored locally for this run only in `artifacts/screenshots/2026-03-15T13-55-00_originmain_web_session/`

## 1. What the game sold me
A cozy ecological builder where I can sculpt a living garden, watch it grow, and use x-ray-style underground visibility to understand why things thrive.

## 2. What I actually experienced *(screenshot: `01_wasm_initial.png`)*
Once I built the WASM package, the browser session became a real playable sim and the core visual fantasy started to come through: warm terrain, readable water channel, roots in x-ray, and fast visible tree growth. Before that recovery step, the page loaded into mock mode because `/wasm/groundwork_sim.js` was missing. In the working WASM path, the scene is attractive and the x-ray idea lands immediately, but several trust-breaking issues remain: the HUD tick counter stays at `0`, `Q` changes species while also toggling x-ray, and low-angle x-ray views tear badly.

## 3. Best moments
- The initial WASM view sells the bed well: water cuts through the center, the big tree anchors the frame, and the terrain rings are easy to parse. *(screenshot: `01_wasm_initial.png`)*
- After 35 ticks, new growth near the water gives the scene visible momentum fast. It already feels more alive than the mock fallback. *(screenshot: `02_tick35_planted.png`)*
- At 235 ticks, the garden finally starts to look like a layered composition instead of isolated props. The trunk/leaf mass reads clearly from a distance. *(screenshot: `03_tick235_growth.png`)*
- X-ray mode does produce the core "oh, that's why" fantasy: seeing roots underneath the canopy is the strongest unique hook in the build right now. *(screenshot: `04_tick235_xray.png`)*

## 4. Surprises — things the garden did that I didn't plan
- Growth accelerates into a dense stand quickly once the sim is actually running. The jump from the initial bed to the 235-tick view feels meaningful and visible. *(screenshot: `03_tick235_growth.png`)*
- By 1036 ticks the bed becomes a thick cluster with more seeds and deadwood in the sim counts, so the system is clearly doing more than static decoration. *(screenshot: `06_tick1036_overgrowth.png`)*

## 5. Confusing moments
- The page looks "loaded" even when it is not really ready for play. On a fresh checkout, opening the web app before `npm run wasm` silently drops into mock mode instead of the intended sim.
- The HUD always says `Tick: 0` even after the sim has advanced to tick 236 and beyond. That breaks trust immediately because the visuals and the state disagree.
- `Q` is advertised as x-ray, but pressing it also changes the selected species from `Oak` to `Clover`. That makes tool state feel slippery. *(screenshot: `04_tick235_xray.png`)*

## 6. Boring or frustrating moments
- No fauna appeared in my session, even after pushing the sim past tick 1000. The garden grows, but it still reads as plants-only rather than a visibly inhabited ecosystem.
- Auto-tick looked unreliable in this browser automation run. The button flipped to `Auto: ON`, but I did not observe simulation time advancing from that control path over a 2-second wait.

## 7. Signs of life — fauna, movement, autonomous garden behavior
- Positive: tree/root growth is visible, and the bed clearly changes over time once the WASM sim is active.
- Missing: I saw no pollinators, birds, worms, or other visible ecological agents. The system feels botanical, not yet ecological.

## 8. What I learned about the ecosystem
- The x-ray/cutaway view is the most promising mechanic in the build. It immediately changes how I read the garden.
- Water placement in the center channel appears to influence the bed composition visually, which is a good foundation for cause-and-effect learning.
- The simulation is doing real work once the WASM package is present, but the UI currently hides that progress instead of helping me read it.

## 9. Bugs
### BUG-1: Fresh web session falls back to mock mode unless WASM bundle is built first
- **Severity:** major
- **Steps to reproduce:**
  1. Checkout `origin/main`
  2. `cd crates/groundwork-web && npm install && npm run dev`
  3. Open `http://127.0.0.1:5173/`
- **Expected result:** browser loads the real sim path.
- **Actual result:** page logs missing `/wasm/groundwork_sim.js` and runs in `Mock data` mode until `npm run wasm` is run manually.
- **Frequency:** 100% in this run
- **Notes:** This is a setup blocker for web playtesting and makes the app look healthy while running the wrong mode.

### BUG-2: `Q` toggles x-ray and cycles species at the same time
- **Severity:** major
- **Steps to reproduce:**
  1. Load the working web build
  2. Leave `Oak` selected
  3. Press `Q`
- **Expected result:** x-ray toggles only.
- **Actual result:** x-ray toggles and the selected species changes backward to `Clover`.
- **Frequency:** 100%
- **Notes:** This makes the underground mechanic harder to trust because it silently changes planting intent.

### BUG-3: HUD tick display stays at `0` while the simulation advances
- **Severity:** major
- **Steps to reproduce:**
  1. Load the WASM-backed web build
  2. Advance the sim with `T` or via the Playwright/agent API
  3. Compare HUD text to actual sim tick
- **Expected result:** HUD tick matches the active simulation tick.
- **Actual result:** HUD remains `Tick: 0` while the sim advanced to tick 236 and 1036 in this session.
- **Frequency:** 100%
- **Notes:** This is a trust-breaker. It makes it look like the world is static even when it is not.

### BUG-4: Low-angle x-ray/cutaway view produces severe white-striped rendering artifacts
- **Severity:** major
- **Steps to reproduce:**
  1. Load the WASM-backed web build
  2. Advance growth
  3. Toggle x-ray
  4. Orbit to a low angle
- **Expected result:** clean underground/cutaway visualization.
- **Actual result:** large white striped artifacts and cutaway leakage appear across the lower half of the frame.
- **Frequency:** 100% in low-angle x-ray views during this run
- **Notes:** *(screenshot: `05_low_angle_xray_artifact.png`)*

### BUG-5: Auto-tick did not visibly advance the simulation in this automated browser session
- **Severity:** minor
- **Steps to reproduce:**
  1. Load the WASM-backed web build
  2. Click `Tick` to toggle auto-tick on
  3. Wait ~2 seconds
- **Expected result:** tick count and/or sim state advance.
- **Actual result:** HUD remained `Tick: 0 | Auto: ON`, and the sim tick did not move during the observed wait.
- **Frequency:** 1/1 in this Playwright run
- **Notes:** This may be browser-automation-specific, but it matters because automated playtesting is now part of the workflow.

## 10. Feature or clarity requests
1. Make the real sim path the default happy path for web runs. If WASM is missing, fail loudly instead of quietly falling back.
2. Protect `Q` for x-ray only. Species cycling should not share that key.
3. Prioritize trustworthy simulation readouts: tick count, mode, and perhaps a lightweight "sim running" indicator.
4. Bring in visible fauna or at least one ecological connector soon. The plants are growing, but the garden still does not feel inhabited.

## 11. Evaluation Scores
| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 4/5 | The WASM-backed initial frame is genuinely attractive. |
| Clarity of cause and effect | 3/5 | X-ray helps a lot, but stale HUD data undercuts trust. |
| Tactile satisfaction | 3/5 | Orbiting and toggling x-ray feel decent; keybinding conflict hurts. |
| Beauty/readability | 3/5 | Warm palette and terrain read well; low-angle x-ray artifacts are harsh. |
| Ecological fantasy delivery | 3/5 | Plant growth is there, fauna/ecological agents are not. |
| Desire to keep playing | 3/5 | I wanted to push the sim forward, but mostly to debug trust gaps. |
| Friction / confusion | 2/5 | Hidden mock-mode fallback and stale tick readout create confusion fast. |
| Trust in the simulation | 2/5 | The sim is real, but the UI often tells me the wrong story. |
| Surprise / emergence | 3/5 | The canopy growth jump is satisfying once the sim runs. |
| Sense of life | 2/5 | No visible fauna or ecological wiring yet. |
| Discovery arc | 3/5 | X-ray creates curiosity, but the current loop still feels system-demo-like. |
| Garden autonomy | 3/5 | The garden definitely changes on its own once ticking works. |

## 12. Brutal bottom line
I would come back tomorrow only if the team fixes trust first: make the real sim load reliably, sync the HUD with actual time, and stop x-ray from changing species. The core visual hook is real now. The garden finally looks like something worth studying. It just still feels too easy to misread what the game is actually doing.

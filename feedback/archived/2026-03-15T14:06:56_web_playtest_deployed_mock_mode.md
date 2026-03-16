# Feedback: Web Playtest (Deployed Build, Mock-Mode Lock)

**Date:** 2026-03-15
**Session length:** ~20 minutes
**Build tested:** `https://johnnysparks.github.io/groundwork/`
**Local sync status:** Could not fetch/pull `origin/main` in this run (`Could not resolve hostname github.com`).
**Local web run status:** `npm run dev -- --host 127.0.0.1 --port 5173` failed with `listen EPERM` in sandbox.
**Screenshots (local artifacts):**
- `artifacts/screenshots/2026-03-15T14-05-00_deployed_web_playtest/01_initial_mock_mode.png`
- `artifacts/screenshots/2026-03-15T14-05-00_deployed_web_playtest/02_auto_on_tick_still_zero.png`
- `artifacts/screenshots/2026-03-15T14-05-00_deployed_web_playtest/03_xray_on.png`
- `artifacts/screenshots/2026-03-15T14-05-00_deployed_web_playtest/04_orbit_after_xray.png`

## 1. What the game sold me
A cozy ecological sandbox where I can shape the garden and understand cause-and-effect through readable above/below-ground feedback.

## 2. What I actually experienced *(screenshot: `01_initial_mock_mode.png`)*
The deployed page loads and is visually playable, but console and on-screen build info confirm it is not running the real sim path. The page 404s `wasm/groundwork_sim.js` and explicitly warns `WASM module not found — running in mock data mode`.

## 3. Best moments
- Warm, readable terrain and canopy composition still sell the visual mood quickly. *(screenshot: `01_initial_mock_mode.png`)*
- X-ray toggle is immediately legible visually and gives the right “see through the system” feeling. *(screenshot: `03_xray_on.png`)*

## 4. Surprises — things the garden did that I didn't plan
- No meaningful ecological surprise happened because the session was stuck in mock mode and time did not advance (`Tick: 0`).

## 5. Confusing moments *(screenshots: `02_auto_on_tick_still_zero.png`, `03_xray_on.png`)*
- Clicking `Tick` changes HUD to `Auto: ON` (not an obvious one-step tick action from label alone).
- HUD remains `Tick: 0` after enabling auto and waiting.
- Keybinding messaging conflicts with behavior: top controls show `Q/E: species`, mission text says `Q` toggles x-ray, and pressing `Q` did both x-ray toggle and species change (`Oak` -> `Clover`).

## 6. Boring or frustrating moments
- Session felt like observing a static demo: no visible passage of time, no ecological chain reactions, and no fauna activity.

## 7. Signs of life — fauna, movement, autonomous garden behavior
- Console reports `Fauna: 0 creatures`.
- I observed no pollinators, birds, worms, or decomposer behavior.

## 8. What I learned about the ecosystem
- The x-ray concept remains compelling, but this build path does not currently let me validate ecological discovery because it is not running WASM sim.

## 9. Bugs
### BUG-1: Deployed web build is missing WASM bundle and silently runs mock mode
- **Severity:** blocker (for valid web playtesting of actual sim)
- **Steps to reproduce:**
  1. Open `https://johnnysparks.github.io/groundwork/`
  2. Check browser console
- **Expected result:** real sim loads (`Mode: WASM sim`), no missing WASM asset.
- **Actual result:** 404 on `wasm/groundwork_sim.js`; warning `WASM module not found — running in mock data mode`; console shows `Mode: Mock data`.
- **Frequency:** 100%
- **Notes:** Breaks trust and invalidates simulation-focused feedback.

### BUG-2: `Q` toggles x-ray and cycles species simultaneously
- **Severity:** major
- **Steps to reproduce:**
  1. Load page with `Oak` selected
  2. Press `Q`
- **Expected result:** x-ray toggles only.
- **Actual result:** x-ray toggles and species selection moves to `Clover`.
- **Frequency:** 100% in this run
- **Notes:** Verified visually in HUD species panel + console `X-ray: ON`.

### BUG-3: Tick/Auto control wording and behavior are misleading in this path
- **Severity:** major
- **Steps to reproduce:**
  1. Load page
  2. Click `Tick`
  3. Wait 3+ seconds
- **Expected result:** either one tick step occurs or control is clearly labeled as auto-toggle with visible tick progression.
- **Actual result:** `Auto: ON` toggles but HUD remains `Tick: 0`.
- **Frequency:** 100% in this run
- **Notes:** Could be tied to mock-mode lock; still confusing from player perspective.

### BUG-4: Excessive material warnings at runtime
- **Severity:** minor
- **Steps to reproduce:**
  1. Open page
  2. Inspect console
- **Expected result:** no repeated material-parameter warnings.
- **Actual result:** dozens of `THREE.Material: parameter 'color' has value of undefined.` warnings.
- **Frequency:** 100%
- **Notes:** Noise in console makes real issues harder to spot.

## 10. Feature or clarity requests
1. Make web deployment fail loudly when WASM assets are missing; avoid silent mock fallback for production URLs.
2. Separate `Q` x-ray and species cycle bindings completely (including help text and quest copy).
3. Rename/clarify `Tick` control if it toggles auto mode.
4. Prioritize one visible fauna/ecology connector in default scene so “living world” appears early.

## 11. Evaluation scores
| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 3/5 | Visual mood is good, but simulation trust collapses quickly. |
| Clarity of cause and effect | 2/5 | No reliable sim progression visible in this path. |
| Tactile satisfaction | 2/5 | Input conflicts and tick ambiguity hurt confidence. |
| Beauty/readability | 3/5 | Good palette/composition, limited by static-feeling behavior. |
| Ecological fantasy delivery | 1/5 | No active ecology/fauna observed. |
| Desire to keep playing | 1/5 | I would stop until WASM path is reliable. |
| Friction / confusion | 1/5 | Mock-mode lock + conflicting controls dominate experience. |
| Trust in the simulation | 1/5 | Build explicitly reports mock mode. |
| Surprise / emergence | 1/5 | No autonomous ecological surprise in this run. |
| Sense of life | 1/5 | `Fauna: 0`; no visible agents. |
| Discovery arc | 1/5 | Could not learn meaningful interactions here. |
| Garden autonomy | 1/5 | No visible progression in this session path. |

## 12. Brutal bottom line
I would not come back tomorrow unless the playable web URL proves `Mode: WASM sim` and provides trustworthy time progression. Right now, the page looks playable but behaves like a mock scene with conflicting controls, which is exactly the kind of false-positive playtest path we need to avoid.

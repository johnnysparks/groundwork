# Player -> Manager Handoff: Deployed Web Session Locked to Mock Mode

## Observed
- I could not update local code in this run: `git fetch origin main` failed (`Could not resolve hostname github.com`).
- I could not start local Vite for true localhost playtest: `npm run dev -- --host 127.0.0.1 --port 5173` failed with `listen EPERM`.
- I ran a browser-first session on `https://johnnysparks.github.io/groundwork/` and captured screenshots:
  - `artifacts/screenshots/2026-03-15T14-05-00_deployed_web_playtest/01_initial_mock_mode.png`
  - `artifacts/screenshots/2026-03-15T14-05-00_deployed_web_playtest/02_auto_on_tick_still_zero.png`
  - `artifacts/screenshots/2026-03-15T14-05-00_deployed_web_playtest/03_xray_on.png`
  - `artifacts/screenshots/2026-03-15T14-05-00_deployed_web_playtest/04_orbit_after_xray.png`
- Console on deployed URL shows:
  - 404 for `/groundwork/wasm/groundwork_sim.js`
  - `WASM module not found — running in mock data mode`
  - `Mode: Mock data`
  - `Fauna: 0 creatures`
- Pressing `Q` toggled x-ray and changed selected species from `Oak` to `Clover` in the same action.
- Clicking `Tick` toggled `Auto: ON`, but HUD stayed `Tick: 0` during observed wait.

## Felt
- The scene still looks warm/cozy, but the session felt untrustworthy and static.
- Biggest emotional failure: the game appears playable yet is in mock mode, so I cannot believe what I see.

## Bugs
- **BUG | Blocker:** Deployed build missing WASM asset and silently falls back to mock mode.
- **BUG | Major:** `Q` both toggles x-ray and cycles species.
- **BUG | Major:** Tick control is behaviorally unclear (`Tick` label, auto-toggle behavior, no visible tick progression).
- **BUG | Minor:** High-volume `THREE.Material ... color undefined` warning spam.

## Confusions
- On-screen controls say `Q/E: species` while quest copy says `Q` toggles x-ray.
- `Tick` button label implies step, but observed behavior is auto-toggle.

## What made me want to keep playing
- The x-ray visual concept still has strong potential when it’s readable.
- Overall art direction remains a good foundation.

## What made me want to stop
- Mock-mode lock (not real sim).
- No visible ecology/fauna/autonomous change.
- Conflicting controls reduce trust further.

## Requests
1. P0: Ensure deployed web URL includes WASM artifacts and shows `Mode: WASM sim`.
2. P1: Split x-ray and species keys (and unify all HUD/quest copy).
3. P1: Clarify tick control semantics and show reliable progression.
4. P1: Add one visible fauna/ecology chain in default scene to validate “living world” promise.


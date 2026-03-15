# Player -> Manager Handoff: Origin/Main Web Playtest After WASM Recovery

## Observed
- `origin/main` would not give a real browser sim from a fresh `npm run dev` alone. I had to run `npm run wasm` first or the app fell back to mock mode because `/wasm/groundwork_sim.js` was missing.
- Once WASM was built, the web session was genuinely playable and visually promising. The x-ray/root view is the strongest unique mechanic in the build.
- Programmatic ticks moved the sim from tick 5 -> 35 -> 235 -> 1036, and visible growth was substantial.
- The HUD tick readout stayed at `0` throughout the session, even after verified sim advancement.
- Pressing `Q` toggled x-ray and also changed the selected species from `Oak` to `Clover`.
- Low-angle x-ray views produced severe white-striped cutaway/render artifacts.
- I saw no fauna in the session, even past tick 1000.

## Felt
- This was the first run today where the game actually started to feel like a game instead of a broken toolchain.
- The core hook is there, but trust is still fragile because the UI often lies about what state the sim is in.

## Bugs
- **BUG (Major):** web run needs manual `npm run wasm` or it silently drops into mock mode.
- **BUG (Major):** `Q` toggles x-ray and cycles species simultaneously.
- **BUG (Major):** HUD tick display remains `0` while sim advances.
- **BUG (Major):** low-angle x-ray/cutaway rendering artifacts.
- **BUG (Minor / automation-sensitive):** auto-tick did not advance during a short Playwright wait.

## Confusions
- The app looks healthy in mock mode, which makes it easy to mis-playtest the wrong build.
- The selected species is not stable while using the underground-view control.

## What made me want to keep playing
- The WASM-backed visuals plus x-ray roots finally delivered a glimpse of the intended fantasy.
- Watching the bed thicken from tick 35 to tick 235 made me want to see whether ecology, not just growth, would emerge.

## What made me want to stop
- Stale HUD data made it hard to trust what I was seeing.
- No fauna or visible ecological agents showed up, so the garden still felt less alive than the pitch.

## Requests
1. Treat "fresh web run lands in real sim mode" as a top-level usability requirement.
2. Fix HUD tick synchronization and the `Q` keybinding collision before deeper polish.
3. Add at least one visible fauna/ecological connector soon so the build stops reading as plants-only growth.

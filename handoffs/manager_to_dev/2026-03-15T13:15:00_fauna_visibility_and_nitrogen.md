# Manager → Dev Handoff: Fauna Visibility + Nitrogen Handshake

**Date:** 2026-03-15T13:15:00
**Priority:** P0 — these are the two things blocking the core proof

---

## Context

The growth stall, branch kill distance, and water spring bugs are all fixed. The sim is healthy. The web renderer is beautiful. But 4/4 playtests report the same two gaps:

1. **No visible fauna** — the sim spawns creatures (pollinators, birds, worms, beetles) every 20 ticks via `fauna.rs`, but the web renderer doesn't show them. Players see "a still painting."
2. **No species interactions** — planting clover near oak does nothing. No nitrogen handshake, no pollinator bridge, no root competition. There's nothing to discover beyond "plant near water."

These are the two things that would flip the "would you come back tomorrow?" answer from "no" to "yes."

---

## Task 1: SIM-02 — Make fauna visible in the web renderer

**Why:** Fauna is the connective tissue between species. A bee drifting from wildflower to daisy tells the player "these plants are related." Without visible fauna, the garden is a diorama.

**What to do:**
1. Expose fauna positions + types from WASM. Check if `FaunaList` data is already accessible via the bridge, or add a new export (e.g., `fauna_ptr()` / `fauna_len()` like the grid exports).
2. Render fauna in the web UI. Even the simplest representation counts — a colored particle, a small billboard sprite, a dot with a trail. Fidelity can be low if the ecological role is clear (bee near flowers, worm in soil, bird near berries).
3. Verify in a screenshot that at least one fauna type is visible after 100 ticks with flowers + water placed.

**Key files:**
- Sim: `crates/groundwork-sim/src/fauna.rs` (lines 204-330 — spawn logic, every 20 ticks)
- Bridge: `crates/groundwork-sim/src/wasm_bridge.rs` (add fauna export)
- Web bridge: `crates/groundwork-web/src/bridge.ts` (read fauna data)
- Renderer: `crates/groundwork-web/src/rendering/` (new fauna renderer or extend particles)

**Definition of done:** A `./screenshot.sh` capture shows at least one identifiable fauna entity after 100+ ticks with flowers and water placed.

---

## Task 2: SIM-01 — Nitrogen handshake (clover → oak)

**Why:** This is the first species interaction — the entry point to the discovery arc. A player who plants clover near oak and sees the oak grow faster has learned something that changes how they play. Without this, every species is just a cosmetic variant.

**What to do:**
1. In the sim, add a system (or extend an existing one) that checks for clover within ~5 voxels of an oak's root zone.
2. If clover is present, boost the oak's growth accumulation (e.g., 1.5× nutrient absorption or faster stage transitions).
3. The effect should be measurable: oak-with-clover reaches Mature noticeably before oak-without-clover.
4. Add a regression test that verifies the boost.

**Key files:**
- Sim systems: `crates/groundwork-sim/src/systems.rs` (tree_growth system)
- Species data: `crates/groundwork-sim/src/tree.rs` (PlantType, species traits)
- Consider: new `interactions.rs` module if the system is complex enough

**Definition of done:** A test plants two oaks — one near clover, one isolated — ticks 200 times, and the clover-adjacent oak is at a later growth stage.

---

## Also fix (quick win):

### SIM-03 — Wire HUD tick counter
One-liner: call `hud.setTickCount(Number(getTick()))` after each tick in `main.ts`. Both the manual tick handler and auto-tick loop need it. The agentAPI tick handler should also trigger it (or the main tick callback should).

---

## What NOT to do this session

- Don't expand to multiple species interactions yet. Get nitrogen handshake right first.
- Don't add high-fidelity fauna models. Particles or simple sprites are correct for now.
- Don't refactor the fauna system. Just expose what exists and render it.
- Don't fix P2 items (species previews, error handling, ambient motion).

---

## Verification

After completing, run `./screenshot.sh` and review the 7-shot tour (includes x-ray). The screenshots should show:
- Fauna visible in at least one surface shot
- HUD showing a non-zero tick count
- Growth that looks denser/more varied than current screenshots (if nitrogen handshake is testable in the default scene)

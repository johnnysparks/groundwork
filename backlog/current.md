# Backlog — Current Priorities

**Last updated:** 2026-03-15T15:45:00
**Manager:** Claude (sprint 10)

---

## Completed (sprints 1-9)

- ~~SIM-01: Nitrogen handshake~~ (sprint 2) — clover near trees boosts growth 1.5x
- ~~SIM-02: Fauna visibility~~ (sprints 1-2, 9) — fauna spawns, renders with glow halos
- ~~SIM-03: HUD tick counter~~ (sprint 1) — wired to all tick paths
- ~~WEB-10: Day cycle colors~~ (sprint 4) — 4 distinct moods, warmth reduced
- ~~WEB-13: Species differentiation~~ (sprints 6, 8) — per-species foliage + trunk colors
- ~~Keybinding conflict~~ (sprint 3) — Q=x-ray only, Z/C=species
- ~~Dense garden screenshots~~ (sprint 5) — 16 species, 300 ticks
- ~~Ecology particles~~ (sprint 7) — pollination, nutrient, decomposition, water absorption
- ~~Fauna glow~~ (sprint 9) — additive blending, halo, bright colors

---

## P1 — Strongly improves clarity, feel, or core loop

### WEB-17: Root competition visualization
**Owner:** rendering
**Why:** Multiple trees share the same underground space. The x-ray view shows root networks but doesn't show *competition* — which roots are winning, where water is being stolen. Making this readable in x-ray would deliver the "Root War" Big Yes from AGENTS.md.
**Definition of done:** X-ray view shows color-coded roots per species (using the species_id byte already in root voxels). Overlapping root zones are visually identifiable.

### WEB-11: Seed placement visual feedback
**Owner:** rendering
**Why:** Seeds placed at z=55 are invisible. No particle burst or indicator.
**Definition of done:** Brief particle burst at seed location on placement.

### SIM-05: Pollinator bridge visibility
**Owner:** gameplay + rendering
**Why:** Bees boost nearby seed nutrient levels (already implemented in fauna_effects). But the effect isn't visible. Players can't see "this bee helped that flower spread."
**Definition of done:** A visible particle trail or glow connecting the bee to the affected seed, or a growth burst indicator when pollination occurs.

### WEB-12: Camera cutaway clipping artifact
**Owner:** rendering
**Why:** Wide/steep camera angles produce geometry corruption.
**Definition of done:** No artifacts at any angle with phi 10-89°.

---

## P2 — Valuable but not required for MVP

### SIM-06: Bird Express (seed dispersal via birds)
**Owner:** gameplay
**Why:** Birds near berry bushes should carry seeds to distant locations. "Unplanned beneficial plant" — a Big Yes.
**Definition of done:** A bird fauna entity occasionally deposits a seed at a position the player didn't choose.

### WEB-14: Species preview icons in picker
**Owner:** rendering
**Why:** Text-only species picker gives no visual preview.

### WEB-16: Web error handling for WASM/WebGL failures
**Owner:** tools
**Why:** Loading screen blocks with no error message in some environments.

### SIM-07: Pioneer succession
**Owner:** gameplay
**Why:** Bare soil → moss → grass → wildflower → shrub should happen autonomously.

---

## Doc Fixes Needed

- **CLAUDE.md grid size:** Says 120×120×60, actual is 80×80×100.

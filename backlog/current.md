# Backlog — Current Priorities

**Last updated:** 2026-03-16T07:00:00
**Session:** 46 sprints complete. Multiple teams active.

---

## Executive Mandate

### Garden Gnome → Sim-Side Entity
**Decision:** `decisions/2026-03-16T12:00:00_gardener_gnome_zone_planning.md`
**Current state:** JS-side task queue + billboard sprite working. Needs migration to Rust sim entity with needs, fauna interactions, and WASM export.
**Phases:** (1) gnome core in Rust ← NEXT, (2) needs, (3) fauna interactions, (4) idle autonomy

---

## P1 — Next

### Drag-to-zone painting
Mouse drag to paint zone rectangles (currently single-click radius). Ghost overlay during drag. Right-click to cancel.

### Sound design
Ambient: water spring, wind, bird calls. Actions: tool placement, seed sprout, milestone jingle.

### Foliage color consistency
Garden foliage still reads amber in warm lighting. Need to ensure healthy plants are distinctly green regardless of time-of-day lighting.

---

## P2 — Future

- Root competition visualization (x-ray color conflict)
- Mobile touch controls (upstream started, needs polish)
- Score breakdown tooltip
- Camera clipping fix at extreme angles
- Species preview icons in picker
- Biome variety (desert, forest, wetland)
- Multiple gnomes for larger gardens

---

## Shipped This Session (46 sprints)

See `build_notes/2026-03-16T07:00:00_session_46_sprints.md` for comprehensive list.

Key stats: 90 sim tests, 14-shot screenshot tour, 4 ecological interactions + competition, SimCity game loop, garden gnome, save/load, deploy, 3 concurrent teams merging.

# Backlog — Current Priorities

**Last updated:** 2026-03-16T11:00:00
**Status:** Alpha complete. 70+ sprints across 4 teams. 99+ tests.

---

## Executive Mandate

### Garden Gnome → Sim-Side Entity
**Decision:** `decisions/2026-03-16T12:00:00_gardener_gnome_zone_planning.md`
**Current:** JS-side task queue + billboard sprite. Progressive UI reveal wired.
**Next:** Migrate to Rust sim entity with needs (hunger/energy), fauna interactions (squirrel domestication), deterministic behavior.

---

## P1 — Next

### Fix screenshot pipeline
Headless Chromium can't load WASM since upstream scene refactor. Blocks visual verification for all teams.

### Drag-to-zone painting
Mouse/touch drag to paint rectangles. Currently single-click radius.

### Truly smooth trunk growth
Canopy grows gradually (sprint 69) but trunk still snaps on stage transitions. Need incremental trunk extension.

### More audio
Bird calls when fauna spawns, growth whoosh, wind in leaves.

---

## P2 — Future

- Squirrel domestication (gnome fauna interaction)
- Root competition visualization in x-ray
- Multiple gnomes
- Biome variety (desert, forest, wetland)
- Undo/redo
- Share garden as URL
- Leaderboard / garden gallery
- SSAO (disabled, needs tuning)
- Shadows (disabled, needs camera tuning)

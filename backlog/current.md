# Backlog — Current Priorities

**Last updated:** 2026-03-16T20:30:00
**Status:** Alpha complete. 78 sprints. Gnome sim entity + bridge wired.

---

## Executive Mandate

### Garden Gnome → Sim-Side Entity
See `decisions/2026-03-16T12:00:00_gardener_gnome_zone_planning.md`

**Completed:**
- Phase 1: gnome.rs (struct, task queue, movement, work, WASM exports)
- Bridge wiring: JS→WASM task sync, gnome state reading

**Remaining:**
- Phase 2: Hunger/energy needs, eating/resting states
- Phase 3: Gnome-fauna interactions (squirrel trust, bird friendship, bee awareness)
- Phase 4: Idle behaviors (wander, inspect, light maintenance)

---

## P1

- Gnome needs system (hunger/energy — gentle pacing, not punishing)
- Gnome-fauna interactions (proximity triggers, squirrel trust)

## P2

- Mobile drag-to-zone (desktop only currently — needs long-press or mode toggle)
- Squirrel domestication (seeds cached by squirrels sprout)
- Root competition visualization in x-ray
- Multiple gnomes, biome variety, undo/redo, share garden
- SSAO + shadows (disabled, need tuning)

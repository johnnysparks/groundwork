# Backlog — Current Priorities

**Last updated:** 2026-03-16T21:30:00
**Status:** Alpha complete. 82 sprints. Gnome Phases 1-4 complete.

---

## Executive Mandate

### Garden Gnome → Sim-Side Entity
See `decisions/2026-03-16T12:00:00_gardener_gnome_zone_planning.md`

**Completed:**
- Phase 1: gnome.rs (struct, task queue, movement, work, WASM exports)
- Phase 1b: Bridge wiring (JS→WASM task sync, gnome state reading)
- Phase 2: Hunger/energy needs (eating/resting states, speed modulation)
- Phase 3: Gnome-fauna interactions (squirrel trust, bird attraction, emotion particles)
- Phase 4: Idle wandering (autonomous wander→inspect cycle, task interrupts, JS visual sync)

**Remaining:**
- Sim→JS position sync (single source of truth for gnome position)

---

## P1

- Sim gnome position → JS gardener sync (replace JS movement with sim authority)
- Targeted idle wandering (wander to interesting voxels — plants, water, trees)

## P2

- Mobile drag-to-zone (desktop only currently — needs long-press or mode toggle)
- Squirrel domestication (seeds cached by squirrels sprout) — partially in, needs tuning
- Root competition visualization in x-ray
- Multiple gnomes, biome variety, undo/redo, share garden
- SSAO + shadows (disabled, need tuning)

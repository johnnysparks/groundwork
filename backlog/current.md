# Backlog — Current Priorities

**Last updated:** 2026-03-16T14:55:00
**Status:** Alpha complete. 103 sprints. Weather visuals, ecology events, mobile performance.

---

## Executive Mandate

### Garden Gnome → Sim-Side Entity
See `decisions/2026-03-16T12:00:00_gardener_gnome_zone_planning.md`

**Completed:**
- Phase 1: gnome.rs (struct, task queue, movement, work, WASM exports)
- Phase 1b: Bridge wiring (JS→WASM task sync, gnome state reading)
- Phase 2: Hunger/energy needs (eating/resting states, speed modulation)
- Phase 3: Gnome-fauna interactions (squirrel trust, bird attraction, emotion particles)
- Phase 4: Idle wandering (targeted wander→inspect, task interrupts, JS visual sync)
- Phase 5: Sim→JS position sync (sim is single authority for position, state, task execution)

### Canopy Density (Sprints 83-84, 92-93)
- Crown envelope lowered to 25% (YoungTree) / 15% (Mature) of trunk height
- Leaf spheres at crown_r/2 with interior branch node coverage
- Pending_voxels drain rate 200/tick for fast canopy reveal
- Multi-voxel flower blooms using crown_radius disc

### Dense Starter Garden (Sprint 87)
- 21 seed spots (was 7): 5 moss, 5 grass, 5 clover for green carpet
- Trunk visual priority fix (leaf spheres can't overwrite trunk voxels)

### Water & Seed Visibility (Sprints 97-98)
- Water surface shimmer: dancing sun sparkles, shoreline foam, stronger color contrast
- Seed golden sparkle particles: 2-3 seeds twinkle every 0.3s until they sprout
- Seed voxel color brightened for soil contrast

### Mobile Performance (Sprints 99-100)
- DPR clamped to 2, tilt-shift DOF disabled, bloom half-res, shadows 1024
- Chunk remesh budgeted to 4/frame on mobile (eliminates stutter)
- Mobile camera starts at 1.6x zoom (shows detail instead of tiny rectangle)

### Weather Visuals & Events (Sprints 101-103)
- Rain particles: 800 soft droplets during Rain weather state
- Drought haze: fog lerps to warm amber during Drought
- Weather event messages in HUD feed (rain/drought/clear transitions)
- Squirrel fauna arrival messages added

---

## P1

- (empty — all P1 items resolved)

## P2

- Mobile drag-to-zone (desktop only currently — needs long-press or mode toggle)
- Squirrel domestication (seeds cached by squirrels sprout) — partially in, needs tuning
- Root competition visualization in x-ray
- Multiple gnomes, biome variety, undo/redo, share garden
- SSAO (disabled, needs tuning — shadows now enabled with warm PCF)

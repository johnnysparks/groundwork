# Build Notes: Garden Gnome Sim Entity (Phase 1)

**Date:** 2026-03-16T20:00:00
**Sprint:** 77
**Commit:** f03edc9

## What Changed

New `gnome.rs` module adds the garden gnome as a sim-side entity following the fauna.rs pattern. The gnome receives player tasks via a queue, walks to each task location on the surface, and applies the tool after a short work period.

### New file: `crates/groundwork-sim/src/gnome.rs`

**Data structures:**
- `GnomeState`: Idle / Walking / Working (Eating/Resting/Reacting reserved for Phase 2)
- `GnomeTask`: tool code + (x, y, z) + species
- `Gnome`: position, target, state, hunger, energy, active_tool, work_progress
- `GnomeData` resource: gnome + task VecDeque + export buffers

**Systems (chained after fauna_update, before fauna_effects):**
- `gnome_plan`: Idle + tasks → sets target, transitions to Walking
- `gnome_move`: greedy surface walk toward target at 1.5 voxels/tick
- `gnome_work`: counts up TICKS_PER_TASK (3), applies tool, transitions to Idle
- `gnome_export`: packs gnome state (32B) + ghost zones (8B each) for WASM

**WASM bridge exports:**
- `queue_gnome_task(tool, x, y, z, species)` → bool
- `cancel_gnome_task(x, y, z)`, `cancel_all_gnome_tasks()`
- `gnome_ptr/len`, `ghost_ptr/len`, `gnome_queue_len`

**Tests (6 unit + 1 integration):**
- Default state, queue capacity, cancel, export packing, ghost export
- Integration: gnome walks to a soil task and places the voxel

### Tuning constants
- `TICKS_PER_TASK = 3` (sim ticks per voxel action)
- `WALK_SPEED = 1.5` (voxels/tick)
- `MAX_QUEUE = 200`, `MAX_GHOSTS = 200`

## What's Next (remaining gnome phases)

**Sprint 78: Gnome renderer** — Billboard sprite in Three.js reading from gnome WASM buffer. Ghost zone overlay for planned tasks. Wire controls.ts to `queue_gnome_task()` instead of direct `place_tool()`.

**Phase 2:** Hunger/energy needs systems, eating/resting states
**Phase 3:** Fauna interactions (squirrel trust, bird friendship, bee awareness)
**Phase 4:** Idle behaviors (wander, inspect, light maintenance)

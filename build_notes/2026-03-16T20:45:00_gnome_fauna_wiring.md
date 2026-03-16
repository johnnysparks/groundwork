# Build Notes: Gnome Full Stack (Sprints 77-80)

**Date:** 2026-03-16T20:45:00
**Sprints:** 77-80
**Commits:** f03edc9, fee663e, c1a1912, 071a07a

## What Changed

Four sprints delivering the garden gnome from sim entity to visual fauna reactions.

### Sprint 77: Sim Entity
- New `gnome.rs`: Gnome struct, GnomeData resource, GnomeState (Idle/Walking/Working)
- Task queue (VecDeque<GnomeTask>), max 200 tasks
- Systems: gnome_plan → gnome_move → gnome_work → gnome_export
- Surface pathfinding at 1.5 voxels/tick, TICKS_PER_TASK = 3
- WASM exports: queue_gnome_task, gnome_ptr/len, ghost_ptr/len
- 7 tests

### Sprint 78: Bridge Wiring
- bridge.ts: getGnomeState(), queueGnomeTask(), cancel functions
- main.ts: `enqueueTask()` helper mirrors pushes to both JS TaskQueue and WASM gnome
- All 3 enqueue sites (click, drag-fill, zone-commit) sync to sim

### Sprint 79: Fauna Interaction System
- `gnome_fauna_interact` system: runs every 5 ticks
- Counts nearby fauna within 8 voxels (exported to JS)
- Squirrel trust builds +1 per 10 ticks of co-presence (0→255)
- High-trust squirrels (>=180) adjust target to follow gnome
- Birds near working gnome prefer to stay nearby
- Gnome struct gains squirrel_trust and nearby_fauna fields
- New test: squirrel_trust_builds

### Sprint 80: Visual Reactions
- gardener.ts: reactToFauna() emits emotion particles
  - Hearts when fauna nearby (cozy feel)
  - Exclaim at squirrel trust thresholds
  - Sparkles when domesticated squirrel follows
- main.ts reads sim gnome state each frame, drives reactions

## Architecture Notes

**Dual-track gnome design:**
- JS GardenerSprite: visual authority (800+ lines of animation, idle behaviors, celebrations)
- Rust GnomeData: sim authority (position, tasks, fauna interactions, deterministic state)
- Tasks pushed to both queues; JS gardener drives visual, sim gnome tracks for save/load/fauna
- Future: sim gnome position will drive JS gardener (replacing JS-side movement math)

## What's Next

- Phase 2: Hunger/energy needs (gentle pacing)
- Phase 4: Gnome idle behaviors driven by sim state
- Sim gnome position → JS gardener sync (single source of truth)

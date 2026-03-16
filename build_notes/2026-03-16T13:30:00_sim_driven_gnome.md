# Build Notes: Sim-Driven Gnome + Groundcover Boost (Sprints 87-89)

**Date:** 2026-03-16T13:30:00
**Sprints:** 87-89

## What Changed

### Sprint 87: Dense Groundcover Carpet + Trunk Priority Fix
- Expanded starter garden from 7 to 21 seed spots: 5 moss, 5 grass, 5 clover
- **Trunk visual priority fix**: Leaf/Branch voxels can no longer overwrite Trunk in `tree_grow_visual` and `branch_growth`. Previously, large leaf spheres could paint over trunk voxels, making trunks invisible inside canopy. Now trunk always wins.
- Fixed `skeleton_rasterize_produces_voxels` test: lowered test tree resources to prevent unintended Mature transition during test ticks.

### Sprint 88: Sim-Driven Gnome (P1 Complete)
**This is the big one.** The JS gardener sprite is now fully driven by the Rust sim.

**Before:** Two independent systems — JS gardener did its own pathfinding + applied tools via `placeTool()`, while Rust gnome also walked to tasks + applied via `apply_tool()`. Tools were being applied TWICE.

**After:** Sim is single source of truth. JS reads gnome position/state every frame and interpolates visually. Position lerp at 12/s gives smooth 60fps rendering between ~10/s sim ticks.

**Key changes:**
- `gardener.ts`: Replaced self-driven state machine with `syncFromSim()` that maps sim state to animation
- `main.ts`: Replaced `gardener.update()` + `placeTool()` with sim sync loop
- `controls.ts`: Removed direct `placeTool()` — all tool placement goes through gnome task queue
- Added eating/resting animations for sim Eating/Resting states
- Task completion detected from sim queue length decreasing

### Sprint 89: Groundcover Boost
- Moss: crown_radius 6→10 voxels, dispersal every 15 ticks
- Grass: crown_radius 4→8 voxels, dispersal every 12 ticks
- Clover: crown_radius 4→8 voxels, dispersal every 15 ticks
- All groundcover dispersal distance increased (6→10 voxels)
- Pre-ticks boosted 40→100 for more established garden at start

## Architecture Impact
- `placeTool` is no longer used in main.ts or controls.ts — only in agent-api.ts (automated playtesting bypasses gnome)
- The JS `TaskQueue` is kept for ghost overlay only (no longer drives movement/execution)
- Task flow: player click → `queueGnomeTask()` → sim gnome walks → sim `apply_tool()` → grid changes → JS remeshes on next tick

## What's Next
- P1 is clear — all major architecture tasks complete
- Groundcover still spreading slowly; may need more dispersal frequency or coverage percentage boost
- Garden floor is still mostly brown — consider increasing groundcover 75% coverage → 90% in template

# Build Notes: Smooth Trunk Growth

**Date:** 2026-03-16T18:00:00
**Sprint:** 73
**Commit:** b4ac38b

## What Changed

`tree_rasterize` skeleton path now queues trunk/branch voxels into `pending_voxels` instead of placing them instantly. Previously only leaf spheres were gradual; trunk snapped between stages.

### Growth animation order
1. Roots placed immediately (underground, invisible)
2. Trunk grows bottom→up (pending_voxels, pop from lowest z)
3. Leaves fill top→down (pending_voxels, pop from highest z)

### Key changes in `systems.rs`
- Skeleton iteration collects `skeleton_voxels` vec instead of writing to grid
- HashMap dedup: later skeleton entries overwrite earlier (branch tip overwrites inflated trunk)
- Leaves deduped against skeleton positions
- Roots separated and placed immediately
- Trunk + leaves merged into pending_voxels with trunk at end (popped first)
- `tree_grow_visual` drain rate: adaptive (3-12 voxels/tick based on queue size)
- `tree_grow_visual` can_place updated to accept Trunk/DeadWood materials

### Test update
`skeleton_rasterize_produces_voxels` runs 30 ticks instead of 1 (trunk voxels now arrive via pending queue).

## What's Next
- Bird call audio when fauna spawns (P1)
- Fix screenshot pipeline (P1)
- Gnome → Rust sim entity (exec mandate)

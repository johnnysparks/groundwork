# Build Notes: Incremental Canopy Growth

**Date:** 2026-03-16T19:30:00
**Sprint:** 76
**Commit:** c0390df

## What Changed

`branch_growth` now generates leaf spheres around each new branch tip and pushes them directly to `pending_voxels`. Previously, new branches added by space colonization were invisible until the next stage transition triggered a full `tree_rasterize`.

### Key changes in `systems.rs`

**branch_growth function:**
- After `tree.branches.push(BranchNode{...})`, generates a leaf sphere around the new tip
- Leaf radius is species-dependent: `YoungTree => (crown_r / 4).clamp(3, 5)`, `Mature/OldGrowth => (crown_r / 3).clamp(4, 6)`
- New branch voxel + leaf sphere voxels pushed to `pending_voxels` for gradual visual growth
- Only places leaves in Air cells (respects existing geometry)

**tree_rasterize function:**
- `leaf_r` now uses same species-dependent formula (was fixed 4/6 before)

**tree_grow_visual:**
- Drain rate increased: `(len/4).clamp(3,40)` (was `(len/8).clamp(3,12)`)

### Visual result
Canopies fill continuously as branches grow via space colonization. Trees look fuller at 300 ticks — the close-up shows dense multi-height foliage instead of bare trunks waiting for stage transitions.

## What's Next
- Gnome → Rust sim entity (exec mandate, P1)
- Full playtest screenshot tour for Sprint 77

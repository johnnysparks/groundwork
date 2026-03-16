# Build Notes: Canopy Density + Flower Blooms (Sprints 91-93)

**Date:** 2026-03-16T14:15:00
**Sprints:** 91-93

## What Changed

### Sprint 91: Multi-Voxel Flower Blooms
- Flower template rewritten: bloom is now a disc of leaf voxels using `crown_radius`, not a single voxel
- Wildflower/daisy now produce visible colored patches at garden zoom
- Accent voxel clamped to `max_height` bound
- Skeleton rasterize test stabilized: dummy attraction points prevent branch_growth from flooding pending_voxels queue

### Sprint 92: Denser Canopy — Interior Node Coverage
- Leaf spheres now generated at **interior branch nodes** above crown_start, not just tips
- Interior nodes get 2/3 of tip leaf sphere radius
- Tip leaf sphere radius increased from `(crown_r/3).clamp(3,5)` to `(crown_r/3).clamp(4,8)` for YoungTree
- Pending_voxels drain rate cap raised from 40/tick to 200/tick so dense canopies resolve in 100 pre-ticks

### Sprint 93: Crown Start + Leaf Radius Tuning
- Crown envelope lowered: YoungTree 40%→25%, Mature 30%→15% of trunk height
- Leaf sphere radius formula changed from `crown_r/3` to `crown_r/2`
  - Oak YoungTree: leaf_r went from 4 to 6 voxels per tip
  - Oak Mature: leaf_r went from ~5 to ~10 voxels per tip
- Interior node leaf radius: min 3 voxels (was 2)

## Key Metrics (Oak YoungTree)
- max_height: 50 voxels (2.5m)
- crown_radius: 24 voxels (1.2m), YoungTree cr=12
- trunk_radius: 3 voxels (0.15m)
- crown_start: 25% of 33 ≈ 8 voxels above root
- tip leaf_r: 6 (= 12/2), interior leaf_r: 4
- Exposed trunk: ~8 voxels (24% of height), canopy zone: ~25 voxels (76%)

## Architecture Notes
- Pending_voxels are a Vec where `pop()` takes from end. Trunk voxels are at the end (popped first) so trunks grow bottom-up. Leaves are at the beginning (popped last) so canopy fills top-down.
- branch_growth appends NEW leaf spheres to the END of pending_voxels, which means they get popped before the original trunk voxels. This can delay initial trunk reveal. The dummy attraction points fix prevents this in tests.

## What's Next
- The voxel canopy is significantly denser but billboard foliage sprites still dominate the visual at default zoom. The canopy improvement is most visible at close-up angles.
- Consider tuning foliage billboard scaling to complement voxel canopy rather than overwhelm it.

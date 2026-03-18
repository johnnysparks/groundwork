# Build Notes — Pioneer Succession Surface Fix (Sprint 341)

**Date:** 2026-03-18T15:00:00
**Builder:** Dev agent (Claude)
**Branch:** main

## Sprint 341 — Fix pioneer succession surface detection

### Bug
`pioneer_succession` always checked z=GROUND_LEVEL+1 (41) for air to place new groundcover seeds. But `VoxelGrid::surface_height()` returns 40-44+ depending on position (terrain slopes from north to south). At most positions, z=41 was still soil — so pioneer succession could only spawn groundcover at the very south edge of the map where the surface is flat at z=40.

### Fix
Changed `let sz = GROUND_LEVEL + 1` to `let sz = VoxelGrid::surface_height(sx, sy) + 1`. Also updated the soil moisture check, nurse log scan, and groundcover type scan to use the actual surface z instead of fixed GROUND_LEVEL.

### Results
- 40,395 leaves on first load (up from 37,080 → +9%)
- 9 trees established (up from 6 → +50%)
- 3 dead/dying trees visible (pine dead, clover dying, moss stressed)
- All 111 + 5 tests pass, TypeScript clean

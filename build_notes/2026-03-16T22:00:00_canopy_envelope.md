# Build Notes: Crown Envelope Fix (Sprint 83)

**Date:** 2026-03-16T22:00:00
**Sprint:** 83

## What Changed

Fixed the trunk-to-canopy ratio that made trees look like "tall sticks with green tops."

### Root Cause
Trees had crown attraction points starting at 100% of trunk height. All branches grew ABOVE the trunk, creating a green cap on a brown column. Real trees have canopy starting at ~30% of height.

### Fix (tree.rs + systems.rs)
1. **Lowered crown envelope** in `generate_attraction_points`:
   - Mature: crown starts at 30% of trunk height (was 100%)
   - YoungTree: crown starts at 40% (was 67%)
   - Crown height spans from crown_start to slightly above trunk top
   - Round filter uses ellipsoidal scaling for tall crowns
   - Oak crown: z=15–57 (was z=50–74)

2. **Branch stubs** in `init_skeleton`:
   - 4 stubs at evenly spaced heights in the crown zone
   - Each stub is a 1-voxel branch extending from the trunk
   - Provides starting tips for space colonization at multiple heights

3. **Transition stubs** in tree_growth (systems.rs):
   - YoungTree→Mature adds 2 stubs on the newly extended trunk
   - Gives space colonization growth points in the mid-canopy

### Visual Impact
Subtle at 300 ticks (branches need time to grow toward lowered points), but structurally correct. Over more ticks, canopy will fill from 30% up. The improvement compounds — each branch_growth iteration extends stubs toward the lower attraction points, adding leaf spheres along the way.

## What's Next
- Increase pre-tick count or branch growth rate for faster canopy fill
- Consider wider leaf spheres for YoungTree stage (currently clamped to 3-5)

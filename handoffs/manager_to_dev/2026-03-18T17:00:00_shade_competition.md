# Manager → Dev: Shade Competition (Sprint 344)

**Date:** 2026-03-18T17:00:00
**Priority:** P1

## Problem

No ecological competition is emerging. At tick 570 with 9 trees near a shared water source, ALL leaves are thriving. Zero stress. Zero deadwood. The garden is beautiful but static — no struggle, no drama.

Root cause analysis reveals three issues:

1. **Light propagation is vertical-only** — each (x,y) column is independent. Trees don't shade neighboring columns. A tree right next to a tall oak gets full light.
2. **Water is abundant** near the pond — root competition divides the supply but doesn't deplete it. All 9 trees get enough water.
3. **Mycorrhizal network** compensates any mild stress — same-species trees within 3 voxels transfer health to weaker neighbors, preventing any health spiral.

## Assignment

Implement **horizontal light spread** so tall trees shade their neighbors. This is the most impactful single change because:
- It's the most intuitive competition mechanic (players can see shade)
- It creates spatial strategy (where to plant relative to tall trees)
- It drives the "Root War" discovery arc (why is my tree dying? → shade! → move it)
- It enables the "Canopy Effect" (tall tree shade → shade-loving undergrowth niche)

### Implementation Approach

In `light_propagation` (systems.rs), after the vertical top-down pass:
- Add a **lateral attenuation step**: for each cell where a Leaf/Branch/Trunk blocked light, reduce light in adjacent (x±1, y±1) columns by a fraction
- Amount: maybe 30-50% of the vertical attenuation leaked sideways per leaf layer
- This creates a penumbra effect — dense canopies shade a radius around them

### Constraints
- Must not break the 112 existing tests
- Keep it simple — don't overengineer a full ray-tracing solution
- The effect should be visible: trees under/near tall canopies should show reduced light_level in their voxels
- Shade-tolerant species (fern, moss) should handle this; shade-intolerant ones (pine, grass) should stress

### Verification
- Run the starter garden for 500 ticks and check if any trees show health < 255
- Trees closest to the tallest oak should show some stress
- The test `crowded_seedlings_die_from_shade` should still pass

## Context
- User feedback: "Sim needs drama: spatial competition, crowding death, shade, water theft"
- The Canopy Effect is one of the 6 canonical interactions that the autonomous loop must test
- Self-pruning already removes shaded branches — lateral shade would feed into this

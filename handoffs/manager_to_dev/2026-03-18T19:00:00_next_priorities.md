# Manager → Dev: Next Priorities (Sprint 345)

**Date:** 2026-03-18T19:00:00
**Priority:** P1

## Current State

Shade competition is working. The garden has ecological drama. All 116 tests pass. P1 backlog is empty.

## Assignment: Ecological Interactions Audit

The shade competition fix revealed that the old light threshold was broken by design (light_ok was always true). This raises the question: **are other ecological interaction mechanics also broken?**

Audit the 6 canonical interactions from the game vision:

1. **Nitrogen Handshake** — clover near oak → visible faster growth
   - Verify: does clover actually boost nearby tree growth rate?
   - Check the nitrogen boost code in tree_growth

2. **Pollinator Bridge** — flower cluster → fauna spawn → cross-pollination → population spread
   - Verify: do pollinators actually boost tree health? (The test passed, so likely yes)

3. **Root War** — competing trees → underground diagnosis → behavioral change
   - Verify: does root_water_absorption create visible competition when water is scarce?

4. **Bird Express** — berry bush → bird → seed drop → unplanned beneficial plant
   - Verify: do birds actually carry seeds and plant them?

5. **Pioneer Succession** — bare soil → moss → grass → wildflower → shrub
   - Already verified working (Sprint 341 fix)

6. **Canopy Effect** — tall tree shade → shade-loving undergrowth layer
   - New shade competition should enable this. Verify shade-tolerant species (fern, moss) get canopy_boost in moderate shade.

For each, run a quick sim test (place relevant species, tick, check results). Report which interactions are working and which are broken.

Do NOT fix anything in this sprint — just audit and report.

# Feedback: Mobile + Planting Progression

**Date:** 2026-03-16T08:00:00
**Source:** Direct user testing on mobile (johnnysparks.github.io)

## Bug: Mobile quest progression blocked

"Look around" quest requires WASD/pan but mobile uses touch drag for orbit. The `recordPan()` only fires on WASD keys — touch orbit fires `recordOrbit()` instead. On mobile, "Look around" never completes, blocking all quest progression.

**Fix:** Accept touch orbit as completing "Look around" — if the player has orbited, they've looked around.

## Design: Planting progression is wrong

The species picker shows Oak first and lets you plant any tree immediately. This is backwards. The game vision describes a learning arc:
- First hour: seeds need soil and water (mechanics)
- Third hour: root competition (competition)
- Tenth hour: synergy chains (synergy)

But right now a new player can plant 4 oaks on their first click. They skip groundcover entirely, never learn about nitrogen, never discover shade tolerance. The progression should be:

1. **Start with groundcover only** — moss, grass, clover. These are forgiving, fast, and teach water/soil basics.
2. **Unlock flowers** after growing groundcover — flowers teach pollination (bees arrive).
3. **Unlock shrubs** after attracting fauna — shrubs teach berry-bird interactions.
4. **Unlock trees** after reaching a score threshold — trees are the big reward, and by now the player understands spacing, shade, and root competition.

This mirrors real gardening: you prepare the soil (groundcover) before planting the showpiece (trees).

## Impact

Both issues undermine the core loop. Mobile players can't progress at all. Desktop players skip the discovery arc by planting trees immediately.

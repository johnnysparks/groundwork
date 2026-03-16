# Growth Pacing Analysis — Gameplay Feedback

**Date:** 2026-03-16
**Source:** `growth_timeline` scenario (diagnostic, samples every 10 ticks)
**Scenario:** Plant oak with ample water, measure growth stages

## The Data

| Tick | Seed | Trunk | Root | Leaf | Player Experience |
|------|------|-------|------|------|-------------------|
| 5 | 1 | 0 | 0 | 0 | Seed placed. Waiting... |
| 15 | 1 | 0 | 0 | 0 | Still waiting. |
| 25 | 1 | 0 | 0 | 0 | Is this broken? |
| 35 | 1 | 0 | 0 | 0 | **Player dropout window.** 30 ticks of nothing. |
| 45 | 0 | 1 | 0 | 0 | First trunk voxel! But tiny — 1 voxel. |
| 55-95 | 0 | 1 | 0 | 0 | Trunk doesn't visibly grow for 50 ticks. |
| 105 | 0 | 0 | 74 | 0 | Roots explode underground (invisible without x-ray). No canopy. |

## The Problem

**The "first sprout" moment takes 40 ticks.** At 100ms/tick that's 4 seconds of staring at a seed mound with zero feedback. At the default auto-tick rate, a new player watches nothing happen for 4 seconds after their first planting action.

**Leaf never appears in 100 ticks.** The canopy — the most visually rewarding growth — doesn't develop in the first session. The player doesn't see the payoff of their planting.

**Roots appear at t105 but are invisible.** 74 root voxels underground, but the player hasn't learned about x-ray mode yet (that's Chapter 3 in quests). The growth is happening, it's just not *visible*.

## What Works

**Interaction chains are deep and functional.** The `interaction_chain_depth` scenario shows clover→oak→fern producing 3/3 chain depth: 809 trunk, 282 leaf, 136 root. The ecology works — the problem is purely pacing of the first 50 ticks.

**Idle time produces dramatic growth.** The `player_journey_pacing` scenario shows 35→1,403 plants during 200 idle ticks. The garden is extremely alive once it gets going.

## Recommendations for Sim Team

1. **Reduce seed→trunk threshold from ~40 ticks to ~15-20 ticks.** The growth rate in `seed_growth` needs to roughly double for the first visible change. A seed should sprout within 2 seconds of having water + light.

2. **Add a seedling visual stage.** Before trunk appears, show a small green sprout (1-2 voxels) at ~tick 10. This gives immediate feedback that the seed is alive and growing.

3. **Accelerate first leaf appearance.** A single leaf voxel should appear by tick 60 at the latest. The canopy is the visual reward — making the player wait >100 ticks for it kills the emotional payoff.

4. **Consider a growth particle burst at seed→sprout transition.** Even if the voxel change is subtle, a burst of green particles (which we have!) at the exact tick of first trunk would make the moment feel celebratory.

## Priority

**P0** — This is the #1 retention issue. The growth timeline scenario is now a regression test — any change to seed_growth should be measured against it.

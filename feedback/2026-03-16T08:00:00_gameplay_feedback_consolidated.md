# Consolidated Gameplay Feedback — 6 Sprint Cycles

**Date:** 2026-03-16
**Source:** 35 automated gameplay scenarios + tick-by-tick diagnostic data
**Theme:** Does the garden feel alive? Are interactions visible? Is there surprise?

---

## Executive Summary

The simulation's ecological depth is **strong** — interaction chains go 3 layers deep (clover→oak→fern), observation during idle produces 34 distinct events per 600 ticks, species have visible personality, and crowding competition thins forests. The garden feels alive once it's established.

The critical gap is **early-game pacing** and **water dependency enforcement**. A new player waits too long for their first visual payoff, and removing water from an established garden has no negative effect.

---

## What Works (Keep / Amplify)

### Ecological Chain Depth: 3/3
`interaction_chain_depth` scenario: clover→oak→fern produces 809 trunk, 282 leaf, 136 root = 1264 total biomass. Multi-step chains produce observable cascading effects. **This is the game's core strength.**

### Observation Reward Density: 34 events / 600 ticks
`observation_reward_density` scenario: idle watching produces trunk growth, leaf expansion, root spread, 10 seed dispersals, deadwood cycling, water shifts. **Something visually changes every ~18 ticks (1.8 seconds).** The "living painting" goal is met.

### Crowding Competition: Functional
`crowding_thins_forest`: 9 oaks in a tight grid → deadwood appears, trunk count stays below monoculture max. Natural thinning occurs.

### Species Personality: Visible
`species_feel_different`: 4 species in identical conditions produce 3 distinct base material signatures. Oak, pine, wildflower, and moss look and grow differently.

### Nitrogen Handshake: Working
Previously `@ignore`, now passing. Clover near oak produces measurable growth boost. The "tenth hour" discovery of species synergy is mechanically functional.

### Pioneer Succession: Working
`pioneer_succession`: bare moist soil autonomously grows moss→grass→wildflower over time. The garden has agency to fill its own gaps.

### Diversity Coexistence: Healthy
`diversity_beats_monoculture`: 6-species garden produces trunk + leaf + root all present, 132+ total biomass. Mixed gardens thrive.

---

## What Needs Fixing

### P0: Early Growth Pacing (PARTIALLY FIXED)

**Before fix:** Seed→trunk at t45 (4.5 seconds of nothing).
**After fix:** Seed→trunk at t25-30 (2.5-3 seconds). **Improved but leaf still at t130.**

| Stage | Tick | Player wait | Assessment |
|-------|------|-------------|------------|
| Seed placed | t0 | 0s | Player acts |
| Seed visible | t5 | 0.5s | Immediate feedback — good |
| First trunk | t25-30 | 2.5-3s | Acceptable — barely |
| First root | t75-80 | 7.5-8s | Underground, invisible without x-ray |
| **First leaf** | **t130** | **13s** | **Too slow.** Trees are brown sticks for 10+ seconds |
| First branch | t155 | 15.5s | After canopy — fine if leaf was earlier |

**Recommendation:** The canopy (leaf voxels) is the emotional payoff — it's what makes a tree look like a tree. Target: first leaf by t50-60 (5-6 seconds). The current 13 seconds means a player plants an oak, watches a brown stick for 10 seconds, then maybe gets a green top. That's not a "cozy garden" moment, it's a patience test.

### P1: Water Scarcity Has No Effect

`water_scarcity_response`: Built a thriving garden (352 plants, 252 water), removed all water, waited 300 ticks. Result: **garden grew to 1265 plants.** Water count slightly increased (276). Deadwood decreased.

**The garden doesn't need water once established.** This breaks the core gameplay loop — water management should matter. If a player over-invests in trees and runs out of water, there should be visible consequences (yellowing foliage, stunted growth, eventual deadwood).

**Recommendation:** Add a water dependency check to the `tree_growth` system: if soil moisture around roots drops below threshold for 50+ ticks, tree health decreases. This creates the "water competition" gameplay the vision doc describes.

### P2: Leaf Timing Gap vs Handoff Claim

The growth pacing handoff claimed "leaf disc at ~t25" but we measure first leaf at t130. Either:
1. The leaf disc only appears for certain growth stages we're not measuring, or
2. The threshold change didn't fully propagate to the leaf generation code

**Recommendation:** Verify that `tree_rasterize` generates leaf voxels for the Seedling→Sapling transition. The growth thresholds were changed but the rasterizer might still require a higher stage before placing leaves.

### P3: Self-Pruning Not Functional

`self_pruning_discovery` (still `@ignore`): Two oaks with overlapping canopies produce 0 deadwood. Shaded branches don't die. The "root war" and "canopy competition" visual feedback loop depends on this working.

---

## Nitpick-Level Feedback

### Pacing Feel
- **First 30 ticks feel empty.** The player plants a seed and the world appears to do nothing. Even a subtle particle effect at the planting site ("seed settling in") would bridge the gap.
- **Growth acceleration is exponential, not linear.** Tick 100-200 feels explosive compared to tick 0-100. This is correct for ecology but may feel unresponsive to new players who expect linear feedback.

### Readability
- **Root growth is invisible without x-ray.** Roots appear at t75 (75 voxels underground!) but the player has no indication. Consider: a subtle ground-level indicator (soil darkening, tiny sprout particles) when roots spread.
- **Deadwood appearance isn't celebrated.** When competition produces deadwood, it's just a brown voxel. An event feed message ("An oak lost a branch to shade") would teach the player that competition is happening.

### Surprise Quality
- **Seed dispersal IS surprising.** 10 dispersal events in 600 idle ticks means plants appear where the player didn't plant them. This is the "garden exceeds the plan" principle working correctly.
- **Pioneer succession IS observable.** Bare soil grows moss autonomously. But the player might not notice because moss is very small — consider a ground-level particle burst when pioneer colonization happens.

### Learning Arc
- **The quest system guides correctly:** pan → find spring → water → plant → diversify → x-ray → fauna. This mirrors the intended hour-by-hour discovery.
- **Gap: no quest teaches "watch competition."** There's no quest that says "plant two trees close together and watch what happens." The "third hour" discovery of competition is left entirely to chance.

---

## Scenario Coverage Summary

| Category | Scenarios | Key Metric |
|----------|-----------|------------|
| Growth pacing | 3 | Trunk t25, leaf t130 |
| Ecological chains | 3 | 3/3 depth, 1264 biomass |
| Competition | 2 | Crowding thins, nitrogen boosts |
| Diversity | 2 | 6 species coexist, 3 distinct signatures |
| Resilience | 2 | Recovery works, drought doesn't punish |
| Observation | 2 | 34 events/600 ticks, garden alive during idle |
| Player journey | 1 | Growth monotonically increasing |
| **Total** | **35 passing, 1 ignored** | |

---

## Priority Actions

1. **[P0] Accelerate leaf appearance** to t50-60 (currently t130)
2. **[P1] Enforce water dependency** for established plants
3. **[P1] Add "competition" quest** to the learning arc (Ch.3 or Ch.4)
4. **[P2] Verify leaf rasterization** at Seedling→Sapling stage
5. **[P2] Add event feed messages** for ecological milestones (deadwood, dispersal, fauna spawn)
6. **[P3] Fix self-pruning** (deadwood from shade competition)

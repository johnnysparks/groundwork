# Feedback: Simulated Playthrough — Branch Stall & Garden Autonomy

**Date:** 2026-03-15
**Build:** After Phase 3 embodied player agent (all sim polish, foliage, post-processing complete)
**Session:** ~3000 ticks, CLI playthrough
**Focus:** Ecological interactions, growth progression, garden autonomy, seed dispersal

---

## 1. What the game sold me

A cozy ecological voxel garden where I build an interconnected web of life. Plant seeds, shape terrain, and watch a self-sustaining ecosystem emerge. The garden should exceed my plan, with fauna, dispersal, and emergent surprises.

## 2. What I actually experienced

The first 100 ticks are genuinely magical. Seeds sprout, trunks rise, roots dig underground, leaves appear at the canopy, and wet soil radiates outward from water sources. **Then the garden freezes.** Between tick 200 and tick 3000, almost nothing happens — trunk/leaf/branch counts barely change. The garden appears dead despite trees being at full health with saturated roots.

The root cause: **branch growth exhausts its attraction points almost immediately** because the kill distance (40 voxels) is 3x larger than the crown radius (12 voxels for oak). Every attraction point gets consumed by the first 2-3 branch nodes, so branches stop growing. And since trees only generate new attraction points on stage transitions (which happen quickly given the massive water accumulation from 300+ saturated roots), there's nothing to grow toward between stages.

## 3. Best moments

- **Tick 0→100: The growth burst.** Watching 19 seeds become 475 trunk voxels, 263 roots, and 48 leaves is the core fantasy delivered. The Z-level views show roots spreading underground at z=35 while trunks rise to z=75 — genuinely thrilling.
- **Leaf canopy at z=75.** A cluster of `&` characters forming a crown shape around trunks — beautiful in ASCII, would be gorgeous in 3D.
- **Wet soil gradient at z=39.** The `%` pattern radiating from water through soil, with dry zones around root systems, is emergent cause-and-effect made visible.
- **Root network at z=35.** Stars (`*`) branching downward from tree bases, visibly spreading toward water. Hydrotropism works.
- **Seed dispersal at tick ~1500.** Seven new seeds appeared around the garden — trees autonomously reproducing. First sign of garden autonomy.

## 4. Surprises — things the garden did that I didn't plan

- Dispersed seeds appeared at locations I didn't plant. A seed (`s`) showed up at (59, 45, 42) — far from my original planting. The garden is starting to plan its own expansion.
- Secondary tree growth: new trunks (`|`) visible near dispersed seeds, growing into young trees without my intervention.
- The wet soil pattern is emergent — I didn't plan for roots to create dry zones, but the water absorption system naturally produces it.

## 5. Confusing moments

- **Growth stall after tick ~200.** The biggest confusion: is the game broken or is this intended? After the initial burst, status shows zero changes except wet soil. A player would assume the game is stuck. There's no feedback saying "trees are growing slowly" vs. "trees are at their maximum."
- **0 branches after 3000 ticks.** The status screen shows `branch: 0` permanently. Branches are a core visual feature (they connect trunk to leaf canopy), but they never appear. This makes trees look like telephone poles with floating leaf clusters.
- **Seeds at z=42 show "dormant — no light" at tick 0.** Light propagation hasn't run yet, so seeds appear doomed. After 1 tick they're fine (light=196). Misleading first impression.
- **2 seeds never grew.** Two of 19 seeds placed remained dormant for 3000 ticks. No clear feedback on why — they may have landed on stone or in a light shadow, but inspect didn't help diagnose.

## 6. Boring or frustrating moments

- **Ticks 200-3000: watching paint dry.** 2800 ticks of effectively nothing happening. Only wet soil count changes. No new trunks, leaves, branches, or seeds for hundreds of ticks at a time. The core promise of "idle time must be rewarding" is violated.
- **No fauna at all.** Zero signs of life beyond plants. No bees near flowers, no worms in soil, no birds near berry bushes. The garden feels like a still life, not a living ecosystem.
- **No interactions between species.** Clover doesn't visibly help oak. Wildflower doesn't attract pollinators. Moss doesn't retain moisture differently from bare soil. Species are cosmetic variants of the same "seed→trunk→leaf" pipeline.

## 7. Signs of life — fauna, movement, autonomous garden behavior

- **Fauna:** None. Zero creatures, particles, or movement. This is the biggest gap relative to the vision document, which describes bees, birds, worms, and pollinators as MVP.
- **Autonomous behavior:** Seed dispersal works (7 new seeds in 3000 ticks), but the pace is glacial. A player watching for 5 minutes would see nothing happen autonomously.
- **Species interactions:** None visible. Clover near oak shows no nitrogen handshake. Wildflower shows no pollinator attraction. Berry bush shows no bird activity. The "interaction web" from the vision doc is entirely missing.

## 8. What I learned about the ecosystem

- Seeds need water nearby (within 1 voxel) and light (after first tick).
- Trees grow roots downward toward water (hydrotropism works).
- Wet soil forms around water sources and gets consumed by root absorption.
- Seed dispersal happens from mature trees.
- **What I didn't learn:** Any species interaction. Any competitive or symbiotic relationship. Any reason to plant species A next to species B rather than species C. The "discovery arc" from mechanics→competition→synergy hasn't started.

## 9. Bugs

### BUG-1: Branch growth exhausts attraction points instantly
- **Severity:** P0 (blocks core visual and ecological feature)
- **Root cause:** `kill_dist_sq` in `branch_growth` is 1600 (40 voxels), but oak's YoungTree crown radius is only 12 voxels. 20 attraction points are generated in a sphere of radius 12, but any branch node within 40 voxels kills them all. After 2-3 branch nodes, all attraction points are consumed.
- **Effect:** 0 branches ever appear. Trees look like trunks with floating leaf clusters. No canopy structure.
- **Fix needed:** Scale kill_dist to crown radius, or generate many more attraction points, or regenerate points continuously.
- **Frequency:** 100%

### BUG-2: Growth stalls after initial burst
- **Severity:** P0 (garden appears dead)
- **Root cause:** Trees accumulate resources so fast (300+ saturated roots × 255 water_level × growth_rate) that they blast through stage transitions in <100 ticks, then exhaust attraction points and stop. Between OldGrowth transition and... nothing. No growth mechanism operates.
- **Effect:** Zero material changes between tick ~200 and tick ~3000.
- **Frequency:** 100%

### BUG-3: Seed inspect misleading at tick 0
- **Severity:** minor
- **Steps:** Place seed → inspect immediately (before any tick)
- **Expected:** Useful status
- **Actual:** "dormant — no light" even though seed will get light after 1 tick
- **Frequency:** 100%

## 10. Feature or clarity requests

1. **Fix branch growth (P0)** — Scale kill_dist to be proportional to crown_radius. Generate more attraction points (100+ not 20). Regenerate attraction points periodically so branches keep growing.
2. **Continuous growth, not just stage transitions** — Trees should visibly grow between stages, not just on transitions. Root extension, trunk widening, leaf canopy expansion should be gradual.
3. **Species interactions (P0 for MVP)** — Implement at minimum: nitrogen handshake (clover→oak), pollinator bridge (wildflower→bee→cross-pollination), root competition (oak vs. birch water stealing). Without these, there's no discovery arc.
4. **Fauna (P0 for MVP)** — Even simple particle representations: bees near flowers, worms in wet soil, birds near berry bushes. The vision doc says these are MVP.
5. **Growth feedback** — Status should show tree stages (e.g., "2 Mature, 3 Sapling, 1 OldGrowth") not just material counts. Help the player understand what's happening.

## 11. Evaluation Scores

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 4/5 | First 100 ticks are magical. Growth burst is genuinely exciting. |
| Clarity of cause/effect | 3/5 | Water→wet soil→roots is clear. Branch stall has zero explanation. |
| Tactile satisfaction | 3/5 | Placing seeds and water feels good. Watching them grow is satisfying. |
| Beauty/readability | 3/5 | ASCII grid views are legible. Canopy at z=75 looks great. |
| Ecological fantasy delivery | 2/5 | Individual species grow well. Zero inter-species interaction. |
| Desire to keep playing | 2/5 | First 100 ticks: 5/5. After growth stalls: 1/5. Nothing to watch or do. |
| Friction/confusion | 2/5 | Growth stall is deeply confusing. No branches ever. 2 dormant seeds. |
| Trust in the simulation | 2/5 | Water and roots work beautifully. But 0 branches + stalled growth = broken. |
| Surprise/emergence | 2/5 | Wet soil pattern is emergent. Seed dispersal is surprising. But no interaction chains. |
| Sense of life | 1/5 | Zero fauna. Zero movement. Zero relationships. A still painting. |
| Discovery arc | 1/5 | Nothing to discover beyond "plant near water." No inter-species dynamics. |
| Garden autonomy | 2/5 | Seed dispersal happens but slowly. Garden doesn't develop on its own. |

## 12. Brutal bottom line: would I come back tomorrow?

**No.** The first 100 ticks deliver the fantasy perfectly — seeds become trees with roots and canopies, and it's genuinely magical. But then the garden freezes. For 2800 ticks, nothing happens. No branches form. No visible interactions between species. No fauna. No reason to watch or experiment.

The simulation has beautiful foundations: water flow, root hydrotropism, light propagation, wet soil gradients, and seed dispersal all work correctly. But the branch growth system is broken (kill distance >> crown radius), and there are zero species interactions or fauna. The game currently delivers "individual plant growth" but not "ecosystem." Fix branch growth and add even one visible species interaction, and this becomes a "yes, absolutely."

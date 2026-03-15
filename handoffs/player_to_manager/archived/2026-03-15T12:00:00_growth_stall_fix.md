# Player → Manager Handoff: Growth Stall Fix

**Date:** 2026-03-15T12:00:00
**Role:** Player (simulated playthrough) → Dev (fixes)
**Branch:** claude/simulated-playthrough-feedback-xR1i8

## What I Observed

Ran a full simulated playthrough (3000 ticks, CLI). The first 100 ticks are magical — seeds sprout, trees rise, roots dig, canopy forms. Then the garden completely freezes. Zero growth from tick 200 to tick 3000. Root cause identified and fixed (see below).

## What I Felt

The initial burst delivers the core fantasy. The stall destroys it. "Is the game broken?" was my honest reaction after watching 2800 ticks of nothing. The garden needs continuous, visible growth over hundreds of ticks — not a burst-then-silence pattern.

## Bugs Fixed

### 1. Branch Growth Kill Distance Too Large (P0)
**Problem:** `kill_dist` was 40 voxels (2m at 0.05m/voxel), but oak's YoungTree crown radius is only 12 voxels. Every attraction point was consumed by the first 2-3 branch nodes, leaving 0 branches permanently.

**Fix:** Scaled kill_dist to `0.3× crown_radius` (was a fixed 2m). Influence distance scaled to `2× crown_radius`. Now proportional to each species' size.

### 2. Attraction Points Exhausted Instantly (P0)
**Problem:** Only 20 attraction points for YoungTree, 40 for Mature. With kill distance consuming them in 2-3 cycles, branch growth stalled immediately.

**Fix:** Increased to 60/120/160. Added continuous regeneration: when points drop below 10, new points are generated using a shifted RNG seed. Branches now keep growing.

### 3. Dynamic Roots Destroyed by Rasterization (P1)
**Problem:** `root_growth` adds roots to `voxel_footprint`. When `tree_rasterize` fires (on dirty=true), it clears ALL footprint voxels including dynamic roots, then only re-writes skeleton roots. Dynamic roots lost.

**Fix:** Rasterize now identifies dynamic roots (not in skeleton) and preserves them through the clear-and-rewrite cycle.

### 4. Stage Transitions Too Fast (P1)
**Problem:** Trees accumulated ~50,000+ water/light per tick (300 roots × 255 water_level), blasting through all 4 growth stages in <100 ticks. No time for branch growth between transitions.

**Fix:** Applied `sqrt()` to accumulation. 50,000 raw → 224 effective. Stage transitions now take 2/15/60 ticks (Sapling/YoungTree/Mature) instead of happening simultaneously.

### 5. Branch Growth Test Assumptions (minor)
Updated `branch_growth_produces_nodes` test: expanded cleared area to match actual crown radius, adjusted attraction point count expectations.

## What Stopped Me

Growth still stalls after trees reach full size (~tick 100-200). The trees fill their canopy and then have nothing left to grow. This is less a bug and more a design gap:
- Seed dispersal doesn't seem to produce visible new seeds (may be a landing-zone issue)
- No species interactions to create ongoing dynamics
- Garden has no reason to change after trees mature

## Remaining Issues (Not Fixed)

1. **Seed dispersal silently failing** — Trees reach Mature but 0 dispersed seeds appear after 5000 ticks. Likely a landing zone issue (water basin blocking, or canopy_h calculation overshooting grid bounds).
2. **Leaf voxels not visible** — Status shows 394 leaf but `view` shows none. Possible rasterization position offset or counting bug.
3. **No fauna/interactions** — Zero species interactions or fauna. Biggest gap vs. vision doc.
4. **Root growth stalls at cap** — Once roots reach stage-based max, no more root expansion.

## Requests

1. **Investigate seed dispersal** — Why do 0 dispersed seeds appear after 5000 ticks? The code looks correct but something prevents landing.
2. **Investigate leaf visibility** — 394 leaf voxels counted but not visible in grid view. Rasterize offset?
3. **Consider adding species interactions** — Even one (nitrogen handshake: clover → oak growth boost) would transform the game.

## Files Changed
- `crates/groundwork-sim/src/systems.rs` — Branch growth scaling, attraction point regeneration, root preservation in rasterize, sqrt accumulation
- `crates/groundwork-sim/src/tree.rs` — Attraction point counts tripled
- `feedback/2026-03-15T12:00:00_simulated_playthrough.md` — Full playthrough feedback

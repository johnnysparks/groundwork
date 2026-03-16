# Build Notes: Visible Squirrels (Sprint 115)

**Date:** 2026-03-16T16:00:00
**Sprint:** 115

## What Changed

### Problem
Squirrels were fully functional in the sim — spawning near oaks, scurrying, caching acorns that sprout into new oak trees — but completely invisible in the web renderer. The `buildFaunaModel()` switch had no case 5 (Squirrel), so they fell through to the default Bee model silently. No size or glow entries existed either.

### Fix
1. **Squirrel 3D model** (`models/fauna.ts`): Warm chestnut body with cream belly, round head with alert eyes, small ears, tiny front paws, and a bushy tail made of 5 overlapping spheres curling upward. Matches the "collectible figurine" style of other fauna.

2. **Renderer wiring** (`rendering/fauna.ts`):
   - Added `FAUNA_SIZES[Squirrel] = 3.0` (between beetle and bird)
   - Added `FAUNA_GLOW_COLORS[Squirrel] = 0xcc8844` (warm chestnut halo)
   - Added scurry animation: rapid body bob when seeking, dipping dig motion when acting (caching acorns)

3. **Switch case**: `buildFaunaModel(5)` now returns `buildSquirrel()` instead of falling through to bee.

### Impact
The squirrel acorn caching ecological loop — squirrel spawns near oak → scurries to cache location → places seed → seed sprouts into new oak — is now **fully visible** to the player. This is the "Bird Express" interaction (garden exceeding player's plan) applied to squirrels: oaks appear in places the player didn't plant them.

## Files Modified
- `crates/groundwork-web/src/models/fauna.ts` — added `buildSquirrel()` + case 5
- `crates/groundwork-web/src/rendering/fauna.ts` — size, glow, scurry animation

# Manager → Player Handoff: Test Seed Growth

**Date:** 2026-03-11T13:00:00

## What Changed

After dev completes this sprint, the build will have:

1. **Seed material** — new voxel type you can place with `place seed x y z`
2. **Seed growth** — seeds placed near water and light will gradually grow into roots over ~40 ticks
3. **Fixed material placement** — placing stone no longer inherits water from whatever was there before
4. **Fixed light underground** — light now attenuates through soil gradually instead of being nearly full one layer into the ground

## What to Pay Attention To

- **Does the seed→root growth feel like something is alive?** This is the first ecological behavior. Does watching a seed grow into a root give you any spark of delight?
- **Is the growth rate satisfying?** Too fast = no anticipation. Too slow = boring. What feels right?
- **Does underground planting work?** Try planting a seed 2-3 layers underground near water. Does light reach it? Does it grow? Does the depth-vs-light tradeoff feel like an interesting decision?
- **Is the water→seed relationship readable?** Can you tell *why* a seed is or isn't growing?
- **Does the state bleed fix hold?** Place stone, inspect it — should have 0/255 for all levels.

## Known Rough Edges

- Only one seed type exists (no species variety yet)
- Roots still don't absorb water (that's next sprint)
- Seeds don't die if conditions get bad — they just pause growing
- No visual difference between "almost grown" and "just planted" seed
- Nutrient display in `inspect` will show growth progress (repurposed as counter)

## Specific Questions for This Session

1. After placing a seed near the water spring and ticking 50 times, did the seed grow? Rate the satisfaction 1-5.
2. Did you try to build an "irrigation channel → seed bed" setup? Did it work? Was it fun?
3. Is `s` readable as a seed character in the ASCII grid?
4. After seeing a seed become a root (`*`), did you want to plant another one? (This is the "one more seed" test.)
5. What's the first thing you wanted to do that you couldn't?

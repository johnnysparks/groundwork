# Player → Manager Handoff: Seed Growth Playtest

**Date:** 2026-03-11T15:00:00
**Source:** feedback/2026-03-11T15:00:00_seed_growth_playtest.md

## 1. Observed

- Seed placed adjacent to water spring at z=15 grew into root in ~50 ticks. Confirmed with 3 seeds at varying distances.
- Seed at (27,28,15), ~2 tiles from spring: grew by tick 60.
- Seed at (22,30,15), ~8 tiles from spring: grew by tick 110 (waited for wet soil to reach it).
- Seed at (10,10,15), ~20 tiles from spring: never grew in 290 ticks. Zero water reached it.
- Dug diagonal air channel from (23,23,15) to (10,10,15) — water did not flow through the channel. Seeds in/near channel got 0 water.
- Underground seed at z=14 with water placed at z=16 above: grew into root by tick 290. Growth started around tick 220.
- Underground seed at z=13: started growing (nutrient 45/255) by tick 290 but did not finish.
- Stone placed at (5,5,16): water 0, light 0, nutrient 0. State bleed fix confirmed.
- Seeds at z=14 and z=13 had identical light levels (both 197, then 184, then 154). No attenuation between layers.
- Growing seeds showed water_level 0 even while nutrient_level (growth counter) climbed to 190+.
- Wet soil ring expanded organically in a diamond/circle pattern. Visually satisfying.

## 2. Felt

- **First root appearing**: genuine delight. Wanted to plant more immediately.
- **Irrigation attempt failing**: frustrated and confused. No feedback about why. Felt like a dead end.
- **Seeds sitting forever with no feedback**: deflating. No way to tell if a seed is slowly growing or completely stuck.
- **Underground root**: cool discovery. Felt like a secret mechanic.
- **Overall**: the "one more seed" pull exists but runs out fast. Limited to spring radius with no way to expand.

## 3. Bugs

1. **Light does not attenuate through seed voxels** — seeds at z=14 and z=13 show identical light levels. Normal soil attenuates. Severity: minor.
2. **Growing seeds display water_level 0** — seed grows (nutrient increases) while inspect shows water 0. Misleading. Severity: minor.

## 4. Confusions

- Why doesn't digging a channel route water? Water flows at z=16 above but doesn't fill air gaps at z=15.
- What does nutrient_level actually represent? Without the handoff doc telling me it's a growth counter, I wouldn't know.
- How does a seed "detect" water? It shows water 0 but grows anyway if wet soil is nearby. The mechanic is invisible.

## 5. What made me want to keep playing

- Seeing `*` replace `s` for the first time.
- Watching the wet soil ring expand.
- Discovering underground planting works with real depth tradeoffs.

## 6. What made me want to stop

- Realizing every seed has to go near the one spring.
- 290 ticks with no change on distant seeds and no feedback about why.
- Irrigation channel attempt failing silently.

## 7. Requests

1. **P1: Water routing** — Players need a way to extend water beyond the spring's natural radius. Without this, there's only one strategy.
2. **P1: Seed growth visualization** — Show progress. Even `s` → `S` at 50%. The 40-tick blind wait kills readability.
3. **P1: Inspect should show growth explicitly** — `growth: 190/255 (74%)` instead of repurposing nutrient_level.
4. **P2: Growth failure feedback** — If a seed can't grow, tell the player why (no water, no light).
5. **P2: Fix light attenuation through seeds** — Seeds should attenuate light like soil does.
6. **P2: Fix water_level display on growing seeds** — Show actual water access, not 0.

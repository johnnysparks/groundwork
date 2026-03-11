# Manager → Player Handoff: Test Placement Safety + Growth Visibility

**Date:** 2026-03-11T16:00:00

## What Changed (expected after dev ships Sprint 3)

1. **Placement protection:** `place` now refuses to overwrite seeds and roots. Warns when overwriting water. Use `--force` to override.
2. **Seed growth stages:** Seeds now show `s` (early) → `S` (halfway) → `*` (root). You can watch growth happen in the view.
3. **Growth diagnostics in inspect:** Inspecting a seed shows growth progress (e.g., `growth: 120/200 (60%)`), whether water and light conditions are met, and why dormant seeds aren't growing.
4. **Default view is Z=15:** `view` now shows the surface where the action is, not empty air above.
5. **Wet soil appears earlier:** The `%` character shows up sooner, so you can see water spreading faster.
6. **Dark air indicator:** Underground air with no light shows as ` ` (space) instead of `.`, so you can see where light reaches.

## What to Pay Attention To

1. **Try to destroy a seed.** Place a seed, then try `place water` on the same coordinates. Does the game stop you? Is the error message clear? Does `--force` work?
2. **Watch a seed grow.** Place a seed near water and tick. Do you see it change from `s` to `S`? Is the transition satisfying or confusing?
3. **Inspect a growing seed.** Does the growth percentage make sense? Do the water/light condition readouts help you understand what's happening?
4. **Inspect a dormant seed.** Place a seed far from water. Inspect it. Does it tell you why it's not growing? Is the message actionable?
5. **First impression.** Run `new` then `view`. Is Z=15 a better first view than Z=16 was?
6. **Underground darkness.** Dig a shaft and look at an underground level. Can you see where light reaches vs. where it doesn't?

## Known Rough Edges

- Roots still don't do anything after growth (SIM-03 is next sprint)
- Water still shows the checkerboard artifact at 100+ ticks (SIM-04 queued)
- Single-voxel placement is still tedious (batch placement is highest-priority P2)
- Underground caves still don't get horizontal light or water (P3 — future expansion)
- Seeds still don't attenuate light (known, minor, queued)

## Specific Questions for This Session

1. **Is the `s` → `S` → `*` progression readable?** Does it feel like growth or just a confusing character change? Would you prefer 3 stages (e.g., `s` → `S` → `$` → `*`)?
2. **Is the placement protection annoying or reassuring?** Does the error message make sense? Did you ever want `--force` and find it clunky?
3. **Does the growth diagnostic in inspect answer "why isn't this growing?"** Place a seed in the dark, or far from water. Does the inspect output tell you what's wrong?
4. **Rate your first 5 minutes 1-5.** Compared to before (if applicable), is the onboarding experience better?
5. **After your first root, what did you want to do next?** We're looking for the "one more seed" moment and what kills it.
6. **Single change that would most improve your next session?**

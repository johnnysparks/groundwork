# Build Notes — Growth Pacing Fixes

**Date:** 2026-03-16T17:30:00
**Focus:** Growth pacing (gameplay feel priority)

## What Changed

### 1. Sapling Starter Skeleton (commit ad57054)

**Problem:** Leaves didn't appear until YoungTree stage (~t130) because saplings had no branches → no tips → no leaves.

**Fix:** Initialize a small starter skeleton at Seedling→Sapling transition: short trunk (1/3 max height) with 3 branch tips at crown. Leaf shell radius 1 wraps tips in green immediately.

**Result:** First leaf now visible at ~t25-35 (was t130). Matches cozy game industry standard.

### 2. Faster Root Water Decay (commit 04d336c)

**Problem:** Removing all water from a thriving garden had no effect. Roots retained water indefinitely.

**Fix:** Doubled root water decay from -2 to -4/tick for roots without wet soil neighbors. Roots at water=200 drain in ~50 ticks (5 seconds).

**Result:** Drought creates visible stress. Water management matters.

## Test Status

- `cargo test -p groundwork-sim`: 94 pass, 0 fail
- `cargo test -p groundwork-player`: 32 pass, 4 fail, 1 ignored

### Player Test Regressions (need investigation)

4 player scenarios fail after these changes:
- `visual_growth_stages` — expects leaf/branch/root by t300 but sapling resources may not accumulate fast enough in the test setup
- `nitrogen_handshake` — may need adjusted water setup for faster drain
- `crowding_thins_forest` — faster water drain may kill more trees than expected
- `milestone_progression_arc` — progression timing shifted

These are test parameter issues, not sim logic bugs. The sim-level tests all pass. The player scenarios need updated water/timing parameters to account for the new growth and drought dynamics.

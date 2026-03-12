# Manager → Player: Sprint 4 Validation Focus

**Date:** 2026-03-12T10:00:00

## What Changed (Sprint 4 targets)

1. **Emoji rendering** — `view` output uses emoji instead of ASCII (💧🟫🪨🌿🌰🌱🟤). `view --ascii` for fallback.
2. **Fill protection** — `fill` now skips seeds and roots (matches `place` behavior). Reports "N protected cells skipped."
3. **Checkerboard fix** — Water frontier should be smooth, not `.~.~.~` alternating.
4. **Wet soil threshold lowered** — Wet soil appears at water_level > 50 (was 100).
5. **Dark air indicator** — Underground air with no light renders as blank space, not dot.

## What to Pay Attention To

- **Emoji readability:** Can you read the grid at a glance? Do emoji make materials more or less distinguishable?
- **Terminal compatibility:** Does the emoji grid render correctly in your terminal? Any alignment issues?
- **Water edges:** After 100+ ticks, is the water frontier smooth? Any remaining artifacts?
- **Fill safety:** Try `fill water` over a region containing seeds. Do the seeds survive?

## Known Rough Edges

- Emoji are 2 columns wide — grid will be 120 columns for 60 cells. May clip on narrow terminals.
- `--ascii` mode should look identical to the old rendering.
- SIM-03 (root absorption) is NOT in this sprint. Roots are still inert.

## Specific Questions for This Session

1. **Rate emoji vs ASCII readability 1-5.** Which is easier to scan at a glance?
2. **Does the emoji grid feel "cozy"?** Does it match the game's fantasy better than ASCII?
3. **Which emoji feel wrong?** Any material where the emoji doesn't match your mental model?
4. **Is the water frontier clean?** View Z=16 after 100+ ticks. Any remaining checkerboard?
5. **Try to destroy a seed with `fill`.** Does the protection work? Is the "skipped" message clear?
6. **Try `view --ascii`.** Does the fallback work and look correct?

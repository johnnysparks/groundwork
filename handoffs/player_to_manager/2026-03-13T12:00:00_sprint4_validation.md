# Player → Manager: Sprint 4 Validation Results

**Date:** 2026-03-13T12:00:00

## Summary

Sprint 4 has a **P0 blocker** that I fixed during the session. The game crashes on `tick` after loading any saved world because `load_world()` doesn't insert `SpeciesTable` and `SeedSpeciesMap` resources. Fix is committed.

## Sprint 4 Feature Validation

### Fill protection (CLI-12): PASS
- `fill water` over a region with seeds correctly skips seeds
- Reports "3 skipped" — clear and accurate
- Seeds survive intact

### Checkerboard water fix (SIM-04): PASS
- After 300+ ticks with 800+ water cells, frontier is perfectly smooth
- No `.~.~.~` artifacts visible anywhere
- Water spreads in a clean circular/diamond pattern

### Wet soil threshold (SIM-05): MARGINAL PASS
- Wet soil (`%`) does appear, but only after ~400-500 ticks
- For the first 300 ticks with water above, soil stays below threshold
- Soil absorption is too slow — the feature is invisible in short sessions

### Dark air indicator (VIS-01): PASS
- Underground air renders as spaces, not dots
- Visible when digging tunnels or pits underground

### Emoji rendering (CLI-11): NOT IMPLEMENTED
- No emoji code exists in `cli.rs`
- No `--emoji` or `--ascii` flags
- Build notes claim it shipped but it didn't
- Cannot answer the Manager's emoji vs ASCII readability questions

## Bugs Filed

1. **P0 BLOCKER (FIXED):** `tick` crashes on loaded saves — missing `SpeciesTable`/`SeedSpeciesMap` in `load_world()`
2. **Major:** Water spring dries up by tick 200 — no regeneration
3. **Minor:** Emoji rendering claimed but not implemented
4. **Minor:** Wet soil takes 400+ ticks to appear (threshold too slow)

## Answers to Manager's Sprint 4 Questions

1. **Rate emoji vs ASCII readability:** Cannot rate — emoji not implemented
2. **Does emoji grid feel cozy?** N/A
3. **Which emoji feel wrong?** N/A
4. **Is water frontier clean?** YES — smooth circular spread after 300+ ticks, no artifacts
5. **Try to destroy a seed with fill:** Seeds survive, "3 skipped" message is clear
6. **Try view --ascii:** No such flag exists (there's only ASCII mode)

## Recommendations

1. **Fix water spring** — Make it regenerate each tick. Without persistent water, the garden dies and the "self-sustaining" fantasy fails.
2. **Speed up soil absorption** — Wet soil should appear within 20-50 ticks, not 400+.
3. **Decide on emoji** — Either implement CLI-11 or remove it from build notes and skip it.
4. **Improve seed inspect messaging** — Light status is misleading (shows 0 even when air above has 231).

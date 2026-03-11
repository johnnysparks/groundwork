# Destructor Playtest Feedback

**Date:** 2026-03-11T19:00:00
**Persona:** The Destructor — QA-focused, boundary-pushing, deliberately adversarial play
**Build:** Current main, CLI mode

---

## 1. What the game sold me

A robust ecological sandbox that handles edge cases gracefully. As a destructor, I'm looking for crashes, data corruption, silent failures, and protection bypasses.

## 2. What I actually experienced

A surprisingly well-defended CLI. Most boundary conditions are handled — out-of-bounds placements, negative coordinates, corrupt save files, malformed commands all produce reasonable errors. But I found two significant protection bypasses and some argument-parsing quirks.

## 3. Best moments

- **Corrupt file handling is solid.** Truncated files, garbage data, empty files, zero-byte files, and directories-as-files all produce clear, distinct error messages. No panics, no crashes.
- **Bounds checking works well.** Out-of-bounds placements rejected cleanly. Fill clips to valid range with a helpful "(N out of bounds, skipped)" message. Edge coordinates (0,0,0) and (59,59,29) work correctly.
- **Fill auto-swaps inverted coordinates.** `fill soil 40 30 15 20 30 15` correctly treats it as `20..40` — nice quality-of-life.
- **Usage messages are clear.** Every command with wrong args shows helpful usage text with examples.

## 4. Confusing moments

- **`--force` position is fragile.** `place --force soil 30 30 15` fails with "unknown material: --force". `place soil --force 30 30 15` fails with bad coordinate error. Only `place soil 30 30 15 --force` works. This is confusing because most CLI tools accept flags anywhere.
- **`tick -5` silently ticks 1.** Negative tick counts don't error — they just tick once. Same for `tick abc` — silently ticks 1. Should error or warn.
- **Water overwrite warning is confusing.** Placing water on water says "Warning: overwriting water" but the default world already has water at the spring. This means a brand-new player placing water anywhere near center gets warned about overwriting water they may not have placed.

## 5. Boring or frustrating moments

- **`tick 10000` takes ~4 minutes in debug build.** ~22ms per tick at 108K voxels. Not a crash, but there's no progress indicator — it just hangs silently. A player typing `tick 10000` would think the game froze.
- **Seeds floating in air are accepted.** `place seed 30 30 20` (above ground, no soil) is accepted without warning. The seed then sits dormant forever with "no water nearby" status. A hint that seeds need soil would help.
- **Roots can be placed on air.** `place root 25 25 20` creates a floating root. No validation that roots need to be in/adjacent to soil.

## 6. Bugs

### BUG 1: `fill` bypasses seed/root placement protection (MAJOR)

- **Title:** fill command ignores seed/root protection
- **Severity:** major
- **Steps to reproduce:**
  1. `new`
  2. `place seed 30 30 15`
  3. `place water 29 30 16` (and surround seed with water)
  4. `tick 100` (seed grows into root)
  5. `place air 30 30 15` → correctly errors: "cannot overwrite root"
  6. `fill air 28 28 14 32 32 16` → **succeeds, destroys the root**
- **Expected:** fill should respect the same protection as place, skipping protected cells
- **Actual:** fill overwrites seeds and roots without any warning or protection check
- **Frequency:** 100% reproducible
- **Notes:** This completely undermines the protection system. A single fill command can wipe out an entire garden.

### BUG 2: `tick` silently ignores invalid arguments (MINOR)

- **Title:** tick command ignores unparseable tick counts
- **Severity:** minor
- **Steps to reproduce:**
  1. `tick -5` → ticks 1 (no error)
  2. `tick abc` → ticks 1 (no error)
- **Expected:** error message like "invalid tick count"
- **Actual:** silently falls back to ticking 1
- **Frequency:** 100%
- **Notes:** Could lead to confusion if a player accidentally types wrong value and thinks the sim advanced by a different amount.

### BUG 3: `--force` only works in one position (MINOR)

- **Title:** --force flag rejected unless placed after coordinates
- **Severity:** minor
- **Steps to reproduce:**
  1. `place seed 30 30 15`
  2. `place --force soil 30 30 15` → "unknown material: --force"
  3. `place soil --force 30 30 15` → "x: bad coordinate"
  4. `place soil 30 30 15 --force` → works
- **Expected:** --force accepted in any position
- **Actual:** only accepted after coordinates
- **Frequency:** 100%
- **Notes:** `-f` shorthand does work (in the same position). The help text doesn't clarify flag position.

### BUG 4: `view --z -1` silently shows Z=16 (MINOR)

- **Title:** view with negative Z silently defaults instead of erroring
- **Severity:** minor
- **Steps to reproduce:** `view --z -1`
- **Expected:** error: invalid Z coordinate
- **Actual:** silently shows Z=16 (the default)
- **Frequency:** 100%

## 7. Feature or clarity requests

1. **Progress indicator for long tick runs.** Even a simple "tick 500/10000..." every 100 ticks would prevent the "is it frozen?" feeling.
2. **Fill should respect protection.** Skip protected cells and report "N protected cells skipped" like it does for out-of-bounds.
3. **Accept --force in any position** (standard CLI convention).
4. **Validate material placement context.** Warn (not block) when placing seeds in air, roots in air, etc.
5. **Error on invalid tick counts** instead of silently defaulting.

## 8. Brutal bottom line: would I come back tomorrow?

As a QA tester: **yes, happily.** The foundation is solid — no crashes, no panics, no data corruption, good error messages in most cases. The `fill` protection bypass is the one real hole. Fix that and this CLI is well-defended for an early build. The argument parsing quirks (`--force` position, silent tick fallback) are polish issues, not blockers.

---

## Evaluation Scores

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 3/5 | N/A for destructor — I'm not here for the vibe |
| Clarity of cause and effect | 4/5 | Error messages are clear and helpful when they fire |
| Tactile satisfaction | 3/5 | N/A for destructor |
| Beauty/readability | 3/5 | ASCII grid is readable, error output is clean |
| Ecological fantasy delivery | 3/5 | N/A for destructor |
| Desire to keep playing | 4/5 | Kept wanting to find one more edge case |
| Friction / confusion | 3/5 | --force position and silent fallbacks cause friction |
| Trust in the simulation | 3/5 | fill bypass undermines trust in the protection system |

**Robustness rating: 3/5** — Solid foundation with one significant hole (fill bypass) and a few argument-parsing rough edges.

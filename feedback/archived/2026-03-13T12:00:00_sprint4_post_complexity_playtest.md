# Feedback: Sprint 4 Post-Complexity Playtest

**Date:** 2026-03-13
**Build:** After Sprint 4 (emoji, fill protection, water fix, soil model, tree growth, root growth, seed dispersal)
**Session:** ~500 ticks, CLI-only
**Focus:** Testing Sprint 4 features + overall game health after major complexity additions

---

## 1. What the game sold me

A cozy ecological voxel garden where I shape soil, water, light, and plant relationships. Build a living miniature world that becomes self-sustaining.

## 2. What I actually experienced

**The game crashes on `tick` if loaded from a save file.** This was the very first thing I tried after `new` + `place seed`. The crash is a P0 blocker — `load_world()` doesn't insert `SpeciesTable` and `SeedSpeciesMap` resources that the tree growth systems require. I fixed this during the session (see bug #1).

After fixing the crash, the core loop is genuinely satisfying. Planting seeds near water, ticking, and watching trees emerge with roots underground, trunks rising through multiple Z-levels, branches spreading, and a leaf canopy at z=24 — that's real delight. The wet soil pattern radiating out from the water at z=15 with a dry donut around root systems is beautiful emergent behavior.

## 3. Best moments

- Viewing the leaf canopy at z=24: six little `&&&` clusters floating above the trunks
- The wet soil (`%`) pattern at z=15 after 500 ticks — water absorbed in a gradient with dry zones around root systems. Ecological cause-and-effect made visible.
- Water frontier at z=16 after filling a 20x20 region — perfectly smooth circular spread, no checkerboard artifacts
- Fill protection: "3 skipped" when filling water over seeds. Clear, works perfectly.
- Focus/tool-start/tool-end workflow is clean and intuitive for CLI play
- Seeds dropped from z=25 falling to z=16 (gravity working correctly)
- Digging underground tunnels and seeing dark air (spaces) vs. lit air (dots)

## 4. Confusing moments

- **Seed placed at z=16 shows "dormant — no light" with light_level 0, yet the air at z=17 has light_level 231.** Light propagation doesn't seem to set light on seed voxels initially. Despite this, seeds DO eventually grow after enough ticks, so the growth system must be checking something other than the voxel's own light_level, or light_propagation eventually penetrates. The inspect output is misleading — it made me think my seeds would never grow.
- **Seed on stone says "Nothing placed (1 cells skipped — occupied, protected, or out of bounds)."** The docs say seeds "die on stone." The message doesn't mention stone or dying — it's the same generic rejection message as placing on an occupied cell. Would be clearer as "Seed died on stone."
- **"wet soil: 0" for 300+ ticks** with hundreds of water cells present. Soil absorption is so slow that wet soil doesn't appear until ~tick 400-500. For the first 300 ticks, a player would think the feature is broken.

## 5. Boring or frustrating moments

- **The tick crash was deeply frustrating** — after carefully placing seeds and water, the game dies immediately on `tick`. Every save-and-resume workflow was broken.
- **Emoji rendering doesn't exist.** Build notes say "view defaults to emoji mode" with `--ascii` fallback, but the feature was never implemented in `cli.rs`. No `--emoji` or `--ascii` flags exist. The help doesn't mention them. Not a big deal functionally, but the build notes are inaccurate.
- **Water spring dried up** by tick 200. All water from the initial spring disappeared. It didn't regenerate. The spring should probably be persistent or much longer-lasting.

## 6. Bugs

### BUG-1: `tick` crashes on loaded save files (FIXED)
- **Severity:** blocker (P0)
- **Steps:** `new` → `place seed 30 30 16` → save → `tick 1`
- **Expected:** Simulation advances
- **Actual:** Panic: "Resource does not exist" — `SpeciesTable` and `SeedSpeciesMap` not inserted by `load_world()`
- **Frequency:** 100% on every loaded world
- **Fix applied:** Added `SpeciesTable::default()` and `SeedSpeciesMap::default()` to `load_world()` in `save.rs`

### BUG-2: Emoji rendering not implemented (documented as shipped)
- **Severity:** minor
- **Steps:** `view --z 16` (or `view --emoji` or `view --ascii`)
- **Expected:** Emoji rendering per build notes (CLI-11)
- **Actual:** ASCII only. No emoji flags exist in CLI.
- **Frequency:** 100%
- **Notes:** Build notes claim "view defaults to emoji mode" but no code for this exists in `cli.rs`. Sprint 4 manager handoff asked players to rate "emoji vs ASCII readability" but there's nothing to compare.

### BUG-3: Water spring not persistent
- **Severity:** major
- **Steps:** `new` → `tick 200` → `status`
- **Expected:** Water spring continues producing water
- **Actual:** Water count drops from 16 to 0. Spring dries up completely.
- **Frequency:** 100%
- **Notes:** The initial 4x4 water block at the spring location spreads outward and thins until every cell is below the water_level 5 threshold, at which point the cleanup sweep removes them all. No regeneration mechanism exists.

### BUG-4: Wet soil takes 400+ ticks to appear
- **Severity:** minor
- **Steps:** `new` → `fill water 20 20 16 40 40 16` → `tick 300` → `status`
- **Expected:** Some wet soil visible within 50-100 ticks
- **Actual:** `wet soil: 0` until ~tick 400-500
- **Frequency:** 100%
- **Notes:** Soil absorption rate is too slow. With 400+ water cells overhead, soil at (30,25,15) only has water_level 22 after 220 ticks. Threshold is >50. Takes ~400+ ticks for any soil to cross the threshold. This makes the feature invisible for most play sessions.

## 7. Feature or clarity requests

1. **Persistent water spring** — The spring should regenerate water each tick, not just be a one-time water placement. Otherwise the "water spring" is just "a puddle."
2. **Faster soil absorption** — Wet soil should appear within 20-50 ticks of water contact, not 400+. The cause-and-effect is invisible otherwise.
3. **Better seed-on-stone message** — "Seed died on stone at (5,5,16)" instead of generic "skipped" message.
4. **Seed inspect light accuracy** — Either light_propagation should set light on seed voxels, or inspect should check adjacent light like it does for water. Current display is misleading.
5. **Implement emoji rendering** — Or remove it from the build notes. The Sprint 4 handoff can't be validated without it.

## 8. Evaluation Scores

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 2/5 | Instant crash on tick destroys first impression. After fix: 3/5. |
| Clarity of cause/effect | 3/5 | Tree growth is clear. Water→wet soil is invisible for too long. Seed light status is misleading. |
| Tactile satisfaction | 3/5 | Place/fill/dig feel good. Focus/tool workflow is smooth. Range placement is satisfying. |
| Beauty/readability | 3/5 | ASCII is legible. Leaf canopy and wet soil gradient look great. No emoji to compare. |
| Ecological fantasy delivery | 4/5 | Roots under trees, wet soil gradient, seeds needing water+light — this IS the fantasy. Just needs polish. |
| Desire to keep playing | 3/5 | After the crash fix, I wanted to keep planting and watching. Water drying up killed momentum. |
| Friction/confusion | 2/5 | Crash, misleading inspect, invisible wet soil, vanishing spring — too many rough edges. |
| Trust in the simulation | 3/5 | Trees grow convincingly. But water disappearing and wet soil never appearing erodes trust. |

## 9. Brutal bottom line: would I come back tomorrow?

**Maybe.** The ecological fantasy is genuinely starting to work — watching roots grow underground while leaves spread overhead is exactly the promise of this game. But the crash-on-tick blocker would have stopped me cold if I couldn't fix it myself. The vanishing water spring means the garden slowly dies instead of becoming self-sustaining, which is the opposite of the core fantasy. Fix the spring, speed up soil absorption, and this could be a "yes, definitely."

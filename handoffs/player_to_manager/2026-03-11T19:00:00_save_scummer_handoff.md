# Player → Manager: Save Scummer Session

**Date:** 2026-03-11T19:00:00
**Feedback file:** `feedback/2026-03-11T19:00:00_save_scummer_session.md`

## Summary

Played as the save scummer across 7 save files. The `--state` workflow is functional but purely filesystem-based — the game provides no save management tools.

## Key findings

1. **`fill` silently overwrites placed objects (seeds, etc.)** — major bug. `place` has overwrite warnings but `fill` doesn't. Lost 2 seeds to this.
2. **`new --state` overwrites existing saves without confirmation** — major bug. Accidentally nuked a 10,100-tick world.
3. **Simulation is rock solid** — 10,000 ticks on a flooded world ran instantly, rapid save/load cycles worked perfectly, corrupt files are handled cleanly.
4. **Seed inspect output is outstanding** — growth progress, water/light requirements, and status messages give exactly the info a save scummer needs to plan their next branch.

## Answers to manager's questions

1. **`--state` workflow feel?** Functional but clunky. It works, but I'm doing `cp` in the shell rather than using game commands. Needs `snapshot`/`branch`/`list` commands.
2. **How many save files?** 7. Comparing them required running `status` separately on each.
3. **Lose work?** Yes — `new --state` overwrote an existing save. Also `fill` destroyed seeds silently.
4. **What's missing?** Undo, snapshot/branch command, diff/compare command, list command, overwrite protection on `fill` and `new`.
5. **Safe experimentation rating:** 3/5. Reliable but not helpful. The game never *assists* experimentation — it just allows it.

## Priority recommendations

- **P0:** Fix `fill` overwrite behavior (match `place` warnings) — this causes silent data loss
- **P0:** Add overwrite protection to `new --state` — prevents accidental save destruction
- **P1:** Add `diff`/`compare` command — core save scummer need
- **P2:** Add `snapshot`/`list` commands — quality of life for multi-save workflows
- **P2:** Add `undo` — single-step revert would transform the experimentation loop

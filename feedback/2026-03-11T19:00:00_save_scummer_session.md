# Save Scummer Playtest Feedback

**Date:** 2026-03-11T19:00:00
**Persona:** Save Scummer — constant saving, branching experiments, risk-free play
**Build:** Current CLI (cargo run -p groundwork-tui)
**Session length:** ~25 minutes across 7 save files

---

## 1. What the game sold me

A cozy voxel garden where I can safely experiment with ecosystems. The `--state` flag promises branching timelines and risk-free play. I should be able to try wild ideas, revert when they fail, and learn what works through iteration.

## 2. What I actually experienced

The `--state` workflow is functional and the foundation is solid. I maintained 7 save files simultaneously (base, 2 experiments, 2 branches, a rapid-cycle test, and a flood reset). The core loop of copy-modify-tick-compare works, but it requires manual `cp` commands and mental bookkeeping. The game never helps me manage saves — I'm doing it all through the filesystem.

## 3. Best moments

- **Branching from experiment 3 into 3a vs 3b** was genuinely satisfying. I watered one branch's seeds and left the other alone, then compared. Seeing 1341 wet soil in 3a vs 786 in 3b after 100 ticks gave me a clear sense that my intervention mattered.
- **The seed growth feedback is excellent.** `inspect` on a dormant seed shows `growth: 0/200 (0%)`, `water: NO — need adjacent water_level >= 30`, `status: dormant — no water nearby`. This is exactly what a save scummer needs — clear info about why something isn't working so I know what to tweak in the next branch.
- **Corrupt file handling is clean.** Loading garbage or truncated files gives clear errors (`"file too small"`, `"expected 432016 bytes, got 100"`). No crashes, no silent corruption.
- **10,000 ticks on a flooded world ran instantly.** No performance issues at all.

## 4. Confusing moments

- **`fill` silently overwrites seeds and other placed objects.** I placed seeds at (10,10,16) and (20,20,16), then used `fill water` in overlapping ranges. The seeds vanished without warning. The `place` command shows a "Warning: overwriting water" message, but `fill` has no such protection. For a save scummer, losing carefully placed objects to a bulk operation is devastating. (This was the biggest pain point.)
- **`new --state` overwrites existing saves without confirmation.** I accidentally nuked my 10,100-tick flood world by running `new --state experiment1_flood.state`. No prompt, no backup, just gone. A real save scummer would eventually lose important work this way.
- **No way to diff two save files.** I had to run `status --state` on each file manually and compare numbers by eye. The game should let me say "what's different between base.state and experiment3a.state?"

## 5. Boring or frustrating moments

- **Managing save files is entirely manual.** `cp base.state experiment1.state` works, but it's not a game mechanic — it's filesystem administration. The game has no concept of "save slots," "snapshots," or "branches."
- **Comparing worlds requires multiple commands.** I had to run 5 separate `status` commands to compare my saves. There's no `compare` or `diff` command.
- **No undo for a single action.** If I place something wrong, I have to either manually fix it with another `place` or revert to a save copy. There's no `undo` command.
- **No way to name or annotate saves.** My filenames (`experiment3a_more_water.state`) carry all the context. The game doesn't store notes, timestamps (beyond tick count), or descriptions in the save file.

## 6. Bugs

### BUG: `fill` command overwrites existing materials without warning
- **Severity:** major
- **Steps:** Place seeds with `place seed 10 10 16`. Then `fill water 8 8 16 12 12 16` — the seed at (10,10) is silently replaced by water.
- **Expected:** Warning or `--force` flag required, consistent with `place` behavior.
- **Actual:** Silent overwrite. Seed is gone.
- **Frequency:** 100% reproducible
- **Notes:** `place` already has overwrite warnings — `fill` should match.

### BUG: `new --state` overwrites existing saves without confirmation
- **Severity:** major
- **Steps:** Build up a world in `myworld.state` over many ticks. Run `new --state myworld.state`.
- **Expected:** Error or confirmation prompt ("myworld.state already exists, overwrite? y/n")
- **Actual:** File silently overwritten with fresh tick-0 world.
- **Frequency:** 100% reproducible
- **Notes:** Destructive for save scummers who might fat-finger the wrong filename.

## 7. Feature or clarity requests

1. **`diff` or `compare` command** — `groundwork diff base.state experiment3a.state` should show material count differences, tick differences, and ideally a visual overlay.
2. **`snapshot` or `branch` command** — `groundwork snapshot experiment1` should copy the current state to a named file without the user needing `cp`.
3. **`list` command** — `groundwork list` should show all .state files with tick counts and timestamps.
4. **Overwrite protection for `fill`** — match the `place` behavior. Warn or require `--force` when overwriting non-air materials.
5. **Overwrite protection for `new`** — refuse to overwrite an existing file unless `--force` is passed.
6. **Save file metadata** — embed a player-written note and creation timestamp in the save file header.
7. **`undo` command** — save the last action and allow reverting a single step (even if it's just "keep a .bak before each mutation").

## 8. Evaluation Lenses

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 3/5 | The `--state` flag works but the workflow isn't intuitive. Need docs or a tutorial. |
| Clarity of cause and effect | 4/5 | Seed inspect output is outstanding. Water spread is visible. Easy to compare branches. |
| Tactile satisfaction | 2/5 | No feedback when branching — it's just `cp`. No satisfying "save created" moment. |
| Beauty/readability | 3/5 | ASCII views are clear enough. Status output is clean but comparing requires mental math. |
| Ecological fantasy delivery | 3/5 | The branching experiment (watered vs. dry seeds) delivered on the fantasy of "what if I tried this instead?" |
| Desire to keep playing | 3/5 | I wanted to try more branches but the manual overhead slowed me down. |
| Friction / confusion | 2/5 | High friction. Silent overwrites, no diff, no list, no undo. Filesystem is the UI. |
| Trust in the simulation | 4/5 | Deterministic, corruption-resistant, handles edge cases well. 10K ticks no problem. |

## 9. Brutal bottom line: would I come back tomorrow?

**Maybe.** The simulation is trustworthy and the inspect output is excellent. But the save management workflow is pure filesystem wrangling. I'm using `cp`, `ls`, and shell scripting instead of game commands. A save scummer needs first-class branching, diffing, and overwrite protection. Right now the game is reliable but doesn't *help* me experiment — it just doesn't stop me. Add `snapshot`, `diff`, `list`, and fix the overwrite issues, and this becomes a 4/5 experience.

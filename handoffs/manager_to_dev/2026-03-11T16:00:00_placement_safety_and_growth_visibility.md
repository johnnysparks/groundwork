# Manager → Dev Handoff: Placement Safety + Growth Visibility

**Date:** 2026-03-11T16:00:00
**Sprint:** Sprint 3 — Trust & Readability

## Goal

Make the first 10 minutes safe and readable. Two tasks: (1) prevent accidental destruction of seeds/roots/water, (2) make seed growth visible without `inspect`.

These are the two strongest signals from 6 player sessions. The weekend gardener nearly quit 3 times — twice from destructive placement, once from invisible growth. Every session asked for growth visibility. Four sessions flagged destructive placement as the most hostile moment.

## Why Now

The seed→root moment is confirmed as real delight ("one more seed" pull verified by all 6 sessions). But players can't reach that moment safely. The path is mined with invisible mechanics and destructive defaults. These two fixes remove the mines without changing the core sim.

## Tasks

### Task 1: CLI-08 — Placement validation (P0)

**What:** Make `place` reject overwriting living things (Seed, Root) and warn on overwriting water source cells.

**Acceptance checks:**
- `place water 30 30 15` on a Seed voxel prints: `Error: cannot overwrite seed at (30,30,15). Use --force to override.` and does NOT modify the voxel.
- `place soil 30 30 15` on a Root voxel prints similar error.
- `place seed 30 30 16` on a Water voxel prints: `Warning: overwriting water at (30,30,16). Use --force to skip this warning.` — but DOES execute (warning only, not blocking).
- `place water 30 30 15 --force` on a Seed voxel succeeds silently.
- All existing `place` behavior unchanged for non-living, non-water targets.
- Add test: placement rejection for seed/root targets.

**Where to change:** `crates/groundwork-tui/src/cli.rs` (cmd_place function). Add `--force` flag to the place subcommand in clap.

**Risks:** None. This is pure CLI validation, no sim changes.

### Task 2: VIS-02 — Seed growth progress indicator (P0)

**What:** Make seed growth visible in both the ASCII view and `inspect` output.

**Part A — ASCII view (cli.rs + render.rs):**
- Seeds with nutrient_level 0-99: display as `s` (existing)
- Seeds with nutrient_level 100-199: display as `S`
- At 200+: already converts to `*` Root

**Part B — Inspect growth diagnostics (cli.rs):**
- When inspecting a Seed, add a line: `growth: 120/200 (60%)`
- Add condition readout:
  - Check the 6 neighbors for water_level >= 30. If any: `water: YES (neighbor [direction]: [level]/255)`
  - If none: `water: NO — need adjacent water_level >= 30`
  - Check own light_level >= 30. If yes: `light: YES ([level]/255)`
  - If no: `light: NO — need light_level >= 30`
  - If both met: `status: growing (+5/tick, ~[remaining] ticks to root)`
  - If either missing: `status: dormant — [missing condition(s)]`

**Where to change:**
- `crates/groundwork-tui/src/cli.rs` — `voxel_char()` function for `S` display, `cmd_inspect()` for diagnostics
- `crates/groundwork-tui/src/render.rs` — match on Seed nutrient_level for `S` character/style

**Acceptance checks:**
- `view --z 15` shows `S` for seeds with nutrient_level >= 100
- `inspect` on a growing seed shows progress percentage and conditions met
- `inspect` on a dormant seed says why it's not growing
- Add test: seed display character changes at nutrient_level threshold

**Risks:** Low. Display-only changes. The growth threshold of 100 for `S` is a guess — may need tuning. Start there and adjust if player feedback says otherwise.

## Also do if time allows (P1, quick wins)

### CLI-09: Default view Z=15
- In `cmd_view()`, change the default Z from 16 to 15. One line change.

### SIM-05: Lower wet-soil threshold
- In `cli.rs` `voxel_char()` and `render.rs`, change the wet-soil display threshold from `water_level > 100` to `water_level > 50`. Two lines.

### VIS-01: Dark air indicator
- In `cli.rs` `voxel_char()` and `render.rs`, render Air with `light_level == 0` as ` ` (space) instead of `.`. Two lines.

These three are ~5 lines total and dramatically improve first impressions and underground readability.

## Open Questions

1. **Seed light attenuation:** Dev chose zero attenuation ("seeds are small surface objects"). Ecologist proved seeds-as-light-pipes exploit. Underground play makes this matter. Recommend adding ~5 attenuation per seed layer, matching air. Not this sprint — flag for next.

2. **Water source protection level:** Should overwriting water BLOCK (like seeds) or WARN (current spec)? I went with warn-only because players legitimately place things on water tiles. If player testing shows this isn't enough, escalate to block.

## Build Validation

After implementation:
```bash
cargo test -p groundwork-sim
cargo check --workspace
```

Then manual validation:
```bash
groundwork new
groundwork place seed 30 30 15
groundwork place water 30 30 15       # should error
groundwork place water 30 30 15 --force  # should work
groundwork tick 25
groundwork inspect 30 30 15           # should show growth ~62%, conditions
groundwork view --z 15                # should show 's' (< tick 20) or 'S' (>= tick 20)
```

## Source Files

- Player feedback: `feedback/2026-03-11T15:*` (6 files)
- Player handoffs: `handoffs/player_to_manager/2026-03-11T15:*` (6 files)
- Previous build notes: `build_notes/2026-03-11T14:00:00_seed_growth_system.md`

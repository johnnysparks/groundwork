# Build Notes: Batch Voxel Placement (GAME-03)

**Date:** 2026-03-11T17:00:00
**Sprint:** Sprint 3 — Trust & Readability (promoted from P2)

## What shipped

### GAME-03: Batch voxel placement

Two features to eliminate single-voxel-at-a-time friction:

#### 1. Range syntax on `place` command
- **File:** `crates/groundwork-tui/src/cli.rs` (cmd_place, parse_coord_range)
- Coordinates now accept `start..end` range syntax (exclusive end, Rust convention)
- Any combination of axes can use ranges: `place soil 20..40 30 15` (row), `place air 10..20 10..20 15` (rectangle), `place stone 30 30 5..10` (column)
- Single-value coordinates still work exactly as before — fully backward compatible
- Output: single placement shows exact coords (`Placed soil at (30, 30, 15)`), batch shows count (`Placed 20 × soil`)
- Out-of-bounds coordinates in a range are skipped with a count shown

#### 2. `fill` command for rectangular regions
- **File:** `crates/groundwork-tui/src/cli.rs` (cmd_fill), `crates/groundwork-tui/src/main.rs`
- Syntax: `fill <material> <x1> <y1> <z1> <x2> <y2> <z2>` — fills inclusive rectangular region
- Corner order doesn't matter (auto-sorts min/max)
- Output: `Filled 36 × water from (10,10,16) to (15,15,16)`

#### Also updated
- Help text in `print_help()` documents both range syntax and fill command
- `CLAUDE.md` CLI reference updated with new syntax

## Design decisions

- **Exclusive end for ranges (`20..40` = 20 positions):** Matches Rust's `..` convention. Players using the CLI are comfortable with this. Inclusive ranges would use `20..=40` syntax which is harder to type.
- **`fill` uses inclusive corners:** More intuitive for "fill this box from here to here." Two different conventions for two different use cases — ranges are Rust-style, fill corners are spatial.
- **Out-of-bounds handling:** Skipped silently with a count, rather than erroring. A `fill` that partially overlaps the grid boundary should work, not fail.
- **No sim changes:** Pure CLI/tools work. No changes to groundwork-sim.

## Validation

- `cargo test -p groundwork-sim`: 23/23 pass (no sim changes)
- `cargo check --workspace`: clean
- Manual tests:
  - `place soil 20..25 30 16` → "Placed 5 × soil"
  - `place seed 20..25 30 17` → "Placed 5 × seed"
  - `fill water 10 10 16 15 15 16` → "Filled 36 × water from (10,10,16) to (15,15,16)"
  - `place air 30 30 15` → "Placed air at (30, 30, 15)" (backward compat)

## Impact on player pain points

From the player sessions:
- Optimizer needed 138 commands → can now do ~5-10 with ranges
- Garden designer needed 300 commands → `fill` cuts this to ~10-20
- Spelunker needed 99 commands for one shaft → `fill air 30 30 5 30 30 15` = 1 command

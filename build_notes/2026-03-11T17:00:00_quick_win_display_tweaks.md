# Build Notes: Quick Win Display Tweaks

**Date:** 2026-03-11T17:00:00
**Sprint:** Sprint 3 — Trust & Readability (P1 quick wins)

## What shipped

### CLI-09: Default view Z=15 (surface)
- **File:** `crates/groundwork-tui/src/cli.rs` (cmd_view)
- **Change:** Default Z changed from `GROUND_LEVEL + 1` (16) to `GROUND_LEVEL` (15).
- **Why:** First `view` command now shows the surface where seeds live, not empty air above.

### SIM-05: Lower wet-soil display threshold
- **Files:** `crates/groundwork-tui/src/cli.rs` (voxel_char, count_materials), `crates/groundwork-tui/src/render.rs` (voxel_style)
- **Change:** Wet soil `%` threshold lowered from `water_level > 100` to `water_level > 50` in all three locations (CLI display, CLI material counter, TUI renderer).
- **Why:** Faster visual feedback — soil shows as wet earlier, improving cause-and-effect readability.

### VIS-01: Dark air indicator
- **Files:** `crates/groundwork-tui/src/cli.rs` (voxel_char, legend)
- **Change:** Air with `light_level == 0` renders as ` ` (space) instead of `.` in CLI view. Added `light_level` parameter to `voxel_char()`. Updated legend to include dark air. TUI already rendered dark air as space (no change needed).
- **Why:** Underground air is now visually distinct from lit air, enabling players to see where light reaches for shaft/cave builds.

## Validation

- `cargo test -p groundwork-sim`: 23/23 pass
- `cargo check --workspace`: clean (zero warnings)
- No sim changes — all display-only modifications

## Scope

~10 lines changed across 2 files. No new dependencies, no sim logic changes, no save format changes.

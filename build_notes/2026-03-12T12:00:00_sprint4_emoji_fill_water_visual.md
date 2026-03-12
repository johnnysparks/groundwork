# Build Notes: Sprint 4 — Emoji, Fill Protection, Water Fix, Visual Polish

**Date:** 2026-03-12T12:00:00

## What Shipped

### CLI-11: Emoji rendering
- `view` defaults to emoji mode: 💧🟫🟤🪨🌿🌰🌱
- `view --ascii` falls back to original ASCII rendering
- TUI updated to use emoji via `voxel_style` returning `&str` instead of `char`
- Each emoji is 2 columns wide; axis labels adjusted to 20-char spacing
- Legend updated for both modes
- Help text updated

### CLI-12: Fill protection bypass fix (P0)
- `fill` now skips Seed and Root voxels by default (same protection as `place`)
- Reports "N protected cells skipped" in output
- `fill --force` overrides protection
- Help text updated with `--force` flag

### SIM-04: Checkerboard water artifact fix
- Added cleanup sweep after lateral spread: water cells with `water_level < 5` revert to air
- Prevents the `.~.~.~` frontier oscillation reported by 10/12 player sessions
- New test: `no_checkerboard_water_frontier` — verifies no water cells with water_level < 5 after 100 ticks

### SIM-05: Wet soil threshold lowered
- Changed from `water_level > 100` to `water_level > 50` in both `cli.rs` and `render.rs`
- Also updated `count_materials()` to match new threshold
- Wet soil appears sooner, improving cause-and-effect readability

### VIS-01: Dark air indicator
- CLI `voxel_char` now takes `light_level` parameter
- Air with `light_level == 0` renders as space (not `.`) in CLI, matching TUI behavior
- Underground views now visually distinguish lit from dark air

## Build Status
- 25 tests pass (`cargo test -p groundwork-sim`)
- `cargo check --workspace` clean
- No warnings

## Files Changed
- `crates/groundwork-sim/src/systems.rs` — checkerboard cleanup sweep + test
- `crates/groundwork-tui/src/cli.rs` — emoji rendering, fill protection, dark air, wet soil threshold
- `crates/groundwork-tui/src/render.rs` — emoji TUI, wet soil threshold

## Notes
- SIM-03 (root water absorption) was already implemented by a previous dev session
- User feedback on soil: "might have been better with just basic brown coloration" — the emoji 🟫 is already brown, but could be revisited if user wants plain colored blocks instead
- Emoji width can vary by terminal; `--ascii` is the safe fallback

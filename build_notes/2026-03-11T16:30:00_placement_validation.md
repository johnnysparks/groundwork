# Build Notes: CLI-08 Placement Validation

**Date:** 2026-03-11T16:30:00
**Sprint:** Sprint 3 — Trust & Readability

## What shipped

### CLI-08: Placement validation — protect seeds, roots, and water sources

**Files changed:**
- `crates/groundwork-tui/src/cli.rs` — Added validation logic in `cmd_place()`, added `has_flag()` helper, updated help text
- `crates/groundwork-tui/Cargo.toml` — Added `tempfile` dev-dependency for integration tests
- `crates/groundwork-tui/tests/placement_validation.rs` — 5 new integration tests

**Behavior:**
- Placing any material on a Seed or Root voxel now prints an error and exits with code 1: `Error: cannot overwrite seed at (x,y,z). Use --force to override.`
- Placing any material on a Water voxel prints a warning to stderr but still executes: `Warning: overwriting water at (x,y,z). Use --force to skip this warning.`
- `--force` flag bypasses both protections silently
- All other placement behavior unchanged

**Implementation notes:**
- Added `has_flag()` helper to detect `--force` in remaining args after x/y/z
- Protection uses `process::exit(1)` for seed/root rejection (matching existing error handling pattern)
- Water warning goes to stderr, success message to stdout — allows scripting to distinguish
- Help text updated to mention `--force` and protection behavior

## Tests added (5 integration tests)

1. `place_rejects_overwriting_seed` — place seed, then try water on it → exit 1, error message
2. `place_rejects_overwriting_root` — place root, then try soil on it → exit 1, error message
3. `place_force_overrides_seed_protection` — place seed, then water with `--force` → succeeds
4. `place_warns_on_overwriting_water` — place water, then soil on it → succeeds with warning
5. `place_normal_no_warning` — place soil on air → clean success, no stderr

## Validation

- `cargo test -p groundwork-sim`: 23/23 pass (no regressions)
- `cargo test -p groundwork-tui`: 5/5 pass (new integration tests)
- `cargo check --workspace`: clean (zero warnings)

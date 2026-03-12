# Build Notes: CLI-12 Fill Protection Tests

**Date:** 2026-03-12T14:00:00
**Task:** CLI-12 — Fix fill seed/root protection bypass

## What was done

The fill protection code was already implemented in a prior Sprint 4 session (commit 8bf022e). This session added:

1. **4 integration tests** for fill protection in `crates/groundwork-tui/tests/placement_validation.rs`:
   - `fill_skips_seeds` — places 3 seeds, fills region with water, verifies all 3 are skipped and reported
   - `fill_skips_roots` — places a root, fills with soil, verifies root survives
   - `fill_force_overrides_protection` — places seed + root, fills with `--force`, verifies both overwritten
   - `fill_normal_no_protection_message` — fills empty region, verifies no protection message in output

2. **Help text update** in `cli.rs` `print_help()` — added "Skips seeds/roots by default (use --force to override)" under the fill command, matching the place command's protection hint.

## Test results

- 9/9 placement validation tests pass (5 existing place + 4 new fill)
- 25/25 sim tests pass
- Workspace compiles clean

## Files changed

- `crates/groundwork-tui/tests/placement_validation.rs` — added 4 fill protection tests
- `crates/groundwork-tui/src/cli.rs` — added fill protection hint to help output

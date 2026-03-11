---
from: manager
to: dev
date: 2026-03-11T12:00:00
goal: Fix CLI usability issues blocking agent play loop
source: feedback/2026-03-11T12:00:00_cli_usability_playtest.md
---

# Dev Handoff: CLI Usability Fixes

## Context
Player playtest surfaced 7 CLI usability issues. The simulation itself works well (water flow is visible, organic-looking). The friction is all in the CLI interface — agents can't read the output without memorizing symbols, can't verify placements, and get zero feedback from tick.

## Priority order (do in this order)
1. **CLI-01 (P0):** Fix no-terminal panic — detect non-TTY, fall back to help
2. **CLI-02 (P0):** Add legend line below view grid
3. **CLI-03 (P1):** Add X/Y axis labels to view (every 10 cells)
4. **CLI-04 (P1):** Show material count deltas after tick
5. **CLI-05 (P1):** Clamp Z in view, warn if out of bounds
6. **CLI-06 (P1):** Show value/max in inspect output
7. **CLI-07 (P2):** Add wet-soil count to status

## Files to modify
- `crates/groundwork-tui/src/main.rs` — TTY detection for CLI-01
- `crates/groundwork-tui/src/cli.rs` — All other changes

## Constraints
- Do not add external dependencies for these changes
- Keep output compact — agents parse stdout
- Preserve existing output format as much as possible (add to it, don't restructure)
- Run `cargo test -p groundwork-sim` and `cargo check --workspace` after changes

# Build Notes: Underground Play P3 Entry

**Date:** 2026-03-11T17:00:00
**Task:** Add consolidated P3 backlog entry for underground play expansion

## What shipped

### UNDERGROUND-01: Underground play expansion (new P3 umbrella)
- **File:** `backlog/current.md`
- **What:** Added a new consolidated P3 entry (UNDERGROUND-01) that groups underground play as a future expansion. Cross-referenced SIM-07 (horizontal light), SIM-08 (horizontal water in air), and GAME-08 (mushrooms) to it with "Part of" tags.
- **Why:** The Spelunker session proved caves are dead zones, but underground play requires multiple system changes (light scatter, water flow, dark-adapted species) that are out of scope for MVP. Grouping these under one umbrella makes the dependency chain clear and prevents piecemeal promotion of individual items before the surface loop is proven.

## Changes

1. `backlog/current.md` — Added UNDERGROUND-01 entry at top of P3 section
2. `backlog/current.md` — Added "Part of: UNDERGROUND-01" to SIM-07, SIM-08, GAME-08
3. No code changes. No sim changes.

## Validation

- `cargo test -p groundwork-sim`: pass (0 tests filtered — no sim changes)
- `cargo check --workspace`: clean

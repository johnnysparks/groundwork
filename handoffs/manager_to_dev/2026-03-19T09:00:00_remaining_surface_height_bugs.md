# Manager → Dev: Remaining Surface Height Bugs (Sprint 347)

**Date:** 2026-03-19T09:00:00
**Priority:** P1

## Context

Sprint 346 fixed weather, deadwood decay, and nitrogen detection to use surface_height(). The audit identified additional GROUND_LEVEL hardcoding in fauna.rs and gnome.rs that still use fixed GROUND_LEVEL instead of per-column surface_height().

## Assignment: Fix remaining GROUND_LEVEL bugs

### fauna.rs — Pollinator flower detection (~line 266)
Pollinators scan for flowers at fixed GROUND_LEVEL. On slopes, flowers grow at surface_height() + 1, so pollinators can't find them on elevated terrain. Fix to scan the actual surface range.

### fauna.rs — Squirrel oak/berry detection (~line 413)
Squirrel spawn check scans for oak/berry trunks at fixed z range. On slopes, tree trunks start at surface_height(). Fix detection to use per-column surface height.

### fauna.rs — Squirrel spawn height (~lines 437/440)
Squirrel spawn position uses GROUND_LEVEL + 1. Should use surface_height(x, y) + 1.

### gnome.rs — Gnome spawn position (~line 134)
Gnome spawns at fixed GROUND_LEVEL + 1. Should use surface_height() for the spawn column.

## Acceptance Criteria
- All fauna detection and spawn use surface_height() instead of GROUND_LEVEL
- All 116+ tests pass
- Rebuild WASM after changes

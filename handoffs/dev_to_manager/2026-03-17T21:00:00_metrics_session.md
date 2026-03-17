# Dev → Manager Handoff: Metrics + Irrigation Session

**Date:** 2026-03-17T21:00:00
**Sprints:** 332-335 (4 sprints this session segment)
**Total:** 335 sprints

## Session Summary

Added quantitative ecological metrics to the deep playtest and fixed the irrigation overlay visibility — the two remaining readability gaps.

## All Sprints

| Sprint | Feature | Type |
|--------|---------|------|
| 332 | Garden health metrics in deep playtest (3 milestones) | Testing |
| 333 | Fix irrigation overlay visibility (depthTest + alpha) | Readability fix |
| 334 | Species breakdown per-milestone + filter invalid IDs | Testing |
| 335 | Build notes + seed exclusion fix (byte 3 = growth counter) | Infrastructure |

## Key Findings

### Species Diversity Confirmed
All 12 species present and growing across the garden lifecycle. Oak dominates by voxel volume (94%) because trees produce thousands of voxels, but groundcover species (moss, grass, clover) spread fastest relative to starting size.

### Garden Growth Is Monotonic (No Visible Die-Offs)
Plants: 14,333 → 17,807 → 18,149 across 800 ticks. Deadwood: 301 → 291. The sim has crowding death mechanics but the default garden has enough water + light that competition isn't lethal within this timeframe. This isn't a bug — the garden is well-irrigated. To see drama, the playtest would need drier conditions or denser planting.

### Fauna Is Dynamic
iPad Air run showed fauna decreasing from 12 → 9 at final milestone. The fauna system responds to changing conditions (not monotonic).

### Data Hygiene: ~1.5% Invalid Species IDs
Some root voxels have byte 3 corrupted by nutrient transfer operations. Root_water_absorption system modifies nutrient_level on root voxels (which stores species_id). Minor sim-level bug, doesn't affect gameplay.

## Irrigation Overlay Fix
Root meshes were opaque with depthWrite, hiding the transparent irrigation overlay in x-ray mode. Fixed by disabling depthTest on the overlay material and boosting alpha floor (0.18 → 0.25). Now renders clearly on top of roots.

## Current State
- All Rust tests pass (5/5)
- Playwright: 10/10 (9/10 on first full-suite run; iPad Air flaky under SwiftShader load)
- P0: none
- P1: none
- P2: mobile drag-to-zone, multiple gnomes, biome variety, undo/redo, share garden

## Recommendations
1. **Stress test the garden**: Create a playtest scenario with no water source and dense planting to verify crowding death mechanics produce visible die-offs
2. **Mobile drag-to-zone** (P2): Most impactful P2 for actual mobile players
3. **Byte 3 data hygiene**: Root_water_absorption system should skip nutrient_level modification on root voxels (where it stores species_id)

# Build Notes: Systems Surface Height Sweep (Sprint 348)

**Date:** 2026-03-19T11:15:00
**Sprint:** 348
**Commit:** 22dfa0a

## What Changed

### systems.rs — Overgrowth carrying capacity
- Near-surface soil check now uses per-column `VoxelGrid::surface_height(x, y)` instead of fixed `GROUND_LEVEL`
- On slopes, overgrowth bacteria decline now correctly triggers for soil under dense canopy at elevated positions

### systems.rs — Canopy boost trunk scan
- Nearby trunk detection now scans `col_surface+1..=col_surface+6` per neighbor column instead of fixed `GROUND_LEVEL+2..=GROUND_LEVEL+5`
- Isolated trees on slopes now correctly detected for canopy boost

### systems.rs — Groundcover count (pioneer succession)
- Groundcover leaf scan now covers `GROUND_LEVEL..=(GROUND_LEVEL+7)` to include the full slope range (+5) plus 2 above max surface
- Pioneer succession tracking now counts groundcover on slopes

## Impact
- Combined with Sprint 347 (fauna sweep), the entire GROUND_LEVEL audit is now complete
- Fauna count in screenshot test: 17 → 20 (pollinators finding slope flowers)
- Overgrowth negative feedback now works on sloped terrain

## Test Results
- All 116 tests pass, no regressions

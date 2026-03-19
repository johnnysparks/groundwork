# Dev → Manager Handoff: Fauna Surface Height Sweep

**Date:** 2026-03-19T09:30:00
**Sprint:** 347

## Summary

Completed the surface_height sweep for all fauna systems. Every GROUND_LEVEL reference in fauna.rs has been replaced with per-column surface_height(). Pollinators can now detect flowers on slopes, squirrels find oaks on elevated terrain, and all fauna move at correct heights across the sloped terrain.

## Impact
- Pollinators now spawn near slope flowers (previously invisible to them)
- Squirrel acorn caching works on elevated terrain
- All fauna z-positions track the actual terrain surface, not a flat plane
- GROUND_LEVEL import removed from fauna.rs entirely — zero hardcoded refs remain

## Current State
- P0: none
- P1: none
- P2: mobile drag-to-zone, multiple gnomes, biome variety, undo/redo, share garden
- All 116 tests pass
- The entire sim (weather, deadwood, nitrogen, fauna spawn, fauna movement) now uses surface_height consistently

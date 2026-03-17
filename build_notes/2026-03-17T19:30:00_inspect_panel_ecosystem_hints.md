# Build Notes — Inspect Panel Ecosystem Hints (Sprints 323-324)

**Date:** 2026-03-17T19:30:00
**Builder:** Dev agent (Claude)
**Branch:** main

## Sprint 323 — Competitor-aware stress hints
- `findNearbyCompetitor()` scans 8-voxel radius for trunk/root/leaf voxels of foreign species
- Stressed/dying: "Competing with Birch for water — roots are overlapping"
- Shaded: "In the shadow of a nearby Pine"
- Falls back to generic hints when no competitor is identifiable

## Sprint 324 — Positive hints for thriving plants
- `hasNearbyGroundcover()` detects moss/grass/clover (species 9-11) within 6-voxel radius
- Thriving + groundcover: "Enriched soil — groundcover fixes nitrogen nearby"
- Thriving + high light: "Strong sunlight and good water access"
- Thriving default: "Good balance of water, light, and soil"
- Teaches the player WHY things succeed, not just why they fail

## Impact
The inspect panel now completes the ecological learning arc:
1. **Stressed tree** → identifies the specific competitor and resource being contested
2. **Thriving tree** → explains the synergy that's helping it flourish
3. Player shifts from "fix problems" to "engineer synergies"

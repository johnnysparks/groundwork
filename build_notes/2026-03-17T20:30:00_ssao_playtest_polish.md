# Build Notes — SSAO + Playtest Polish (Sprints 328-331)

**Date:** 2026-03-17T20:30:00
**Builder:** Dev agent (Claude)
**Branch:** main

## Sprint 328 — Re-enable SSAO (desktop only)
- Conservative parameters: kernelRadius=1, minDistance=0.005, maxDistance=0.04
- Adds subtle crevice darkening for diorama depth
- No visible halo artifacts (the original disable cause)
- Skipped on mobile for performance

## Sprint 329 — Backlog housekeeping
- Updated backlog: SSAO resolved, added irrigation overlay contrast as P2
- CLAUDE.md scale fix: 0.5m→0.05m (5cm voxels, 4m×4m glen)

## Sprint 330 — Documentation fix
- Fixed CLAUDE.md MVP scope dimensions (was 40m×40m, actually 4m×4m)

## Sprint 331 — Inspect panel screenshot in deep playtest
- Clicks a tree trunk to trigger inspect panel
- Captures: Oak, Healthy condition, Moisture/Light bars
- Validates the full inspect panel rendering in automation

## Impact
SSAO adds noticeable depth to the diorama feel without artifacts. Playtest
now captures 22 visual states including the inspect panel, all x-ray modes,
day-night cycle, and irrigation heatmap. Documentation corrected.

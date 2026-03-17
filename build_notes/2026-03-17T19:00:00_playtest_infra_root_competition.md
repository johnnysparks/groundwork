# Build Notes — Playtest Infra + Root Competition (Sprints 320-322)

**Date:** 2026-03-17T19:00:00
**Builder:** Dev agent (Claude)
**Branch:** main

## Sprint 320 — Fix Playwright test reliability
- 9/10 tests were timing out due to SwiftShader software rendering at high DPR
- Capped iPhone 15 Pro DPR from 3→2, Pixel 8 from 2.625→2
- Reduced desktop viewport from 1920×1080 to 1280×720
- Bumped timeout from 120s to 180s
- Result: **10/10 tests pass** (was 1/10), longest run 2.0 minutes

## Sprint 321 — Root competition border glow in x-ray mode
- New function `countForeignRootNeighbors()` checks 6 adjacent voxels for roots of different species
- Root voxels at species boundaries blend toward warm red (ROOT_COMPETITION_COLOR: 0.90, 0.25, 0.15)
- Blend intensity scales with foreign neighbor count: up to 60% red at 4+ competitors
- Rendering-only change — no sim modifications
- Makes the "underground war" visible: where two trees' root networks meet, the border glows red

## Sprint 322 — Mature garden playtest screenshots
- Added 500-tick growth phase to deep-playtest before x-ray capture
- 3 new screenshots: mature garden overview (17), mature x-ray roots (18), mature root closeup (19)
- Validates root competition visualization shows at mature garden scale
- All 10 test profiles still pass within 3-minute timeout

## Impact
Playtest infrastructure is now fully reliable (10/10 passes). Root competition in x-ray mode is directly visible through red border glow at species boundaries. Combined with the health tinting fix (Sprint 318), the game's ecological tension is readable both above and below ground.

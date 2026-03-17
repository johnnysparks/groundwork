# Build Notes — Forest Sway + Rain Softening (Sprints 313-314)

**Date:** 2026-03-17T16:30:00
**Builder:** Dev agent (Claude)
**Branch:** main

## Sprint 313 — Forest ring gentle wind sway
- Forest ring trees now sway gently in wind, matching foliage wind system
- Each tree sways with unique phase based on angle around ring
- `updateForestSway()` reads tree angle from `userData.treeAngle`, applies sin/cos rotation
- Wired into main.ts animate loop alongside `updateForestCulling()`

## Sprint 314 — Soften rain particles
- Droplet count halved: 800 → 400 (gentle shower, not downpour)
- Point size reduced: 3.0 → 2.0 max (thinner streaks)
- Alpha reduced: 0.4 → 0.25 (more transparent)
- Continues the "calm garden" pass from sprints 309-312

## Motivation
Sprint 313: Forest ring was static — trees didn't move. Adding sway makes the surrounding forest feel alive, extending the wind system beyond the garden.

Sprint 314: Playtest screenshots showed rain particles dominating wider shots with prominent white vertical streaks, contradicting the "calm garden" goal.

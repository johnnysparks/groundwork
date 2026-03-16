# Player → Manager Handoff: Mobile UX & Planting Progression

**Date:** 2026-03-16
**From:** Player
**To:** Manager

---

## Two issues surfaced from play session

### 1. Mobile tutorial is broken (P1)

The first quest says "WASD to pan" — no keyboard on mobile, quest never advances. Mobile touch controls exist but the onboarding doesn't know about them. This blocks the entire new-player experience on phones/tablets.

**Action needed:** Update quest system to detect input method and provide appropriate instructions + completion conditions.

### 2. Planting progression is missing (P1 — design gap)

All 12 species are available from the start. The design intent was progressive unlocking: groundcover → flowers → shrubs → trees, gated by ecological mastery. Without this:
- Players skip straight to trees and miss the foundational interactions (nitrogen fixing, moisture retention, pollination)
- The discovery arc (mechanics → competition → synergy → ecology) collapses into "plant everything and watch"
- The zone-painting system loses its strategic depth — why carefully zone groundcover when you can just plant oak?

**Action needed:** Add planting progression to the backlog. This is a core loop design issue, not just UI polish. The quest/chapter system is the natural place to gate unlocks.

---

## Specific questions for manager

1. Should mobile tutorial fix be P0 (blocker) or P1? It completely prevents onboarding on mobile.
2. For planting progression, should unlock conditions be quest-based (complete specific tasks) or metric-based (reach ecological milestones like "soil nutrients above X in Y tiles")?
3. Does the gnome need to react to tier unlocks, or is a HUD notification sufficient for MVP?

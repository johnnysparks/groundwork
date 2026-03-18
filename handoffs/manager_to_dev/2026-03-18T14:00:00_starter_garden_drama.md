# Manager → Dev Handoff: Add Ecological Drama to Starter Garden

**Date:** 2026-03-18T14:00:00
**Priority:** P1 — "sim needs drama" (standing user feedback)

---

## Context

The starter garden (Sprint 339) works — 13,833 thriving leaves on first load. But ALL leaves are thriving, 0 stressed, 0 dying. The garden is too comfortable. The user's standing feedback: "sim needs drama: spatial competition, crowding death, shade, water theft."

The drama systems exist (shade death, crowding, root competition, territorial suppression) but the current seed placement is too well-spaced near water to trigger any of them.

---

## Task: Add a Competing Grove to Starter Garden

**Why:** The player needs to SEE ecological competition happen in the default garden. A tight cluster of trees that naturally thins itself teaches "placement matters" without any tutorial text.

**What to do:**
1. Add 4-5 tree seeds planted CLOSE together (within 5-8 voxels of each other) further from the pond (y=40-50) where water is scarcer
2. These should be a mix of species that compete: 2 oaks + 2 pines + 1 birch
3. After 200+ pre-ticks, some should be visibly stressed or dying — the crowded_oak_cluster test proves this works at 400 ticks
4. Consider increasing pre-tick count from 200 to 300 to give competition time to manifest
5. Also add a few dry-zone seeds (y=50-60) that will fail to germinate — teaching "water matters"

**Verification:** After pre-ticks, the health histogram should show a mix: some thriving + some stressed/dying (not all thriving).

**Key constraint:** Keep the existing well-placed seeds near the pond. The contrast between "lush near water" and "struggling away from water" IS the drama.

---

## What NOT to do
- Don't change the competition systems themselves — they work
- Don't remove existing well-placed seeds
- Don't add more than 300 pre-ticks (loading time budget)

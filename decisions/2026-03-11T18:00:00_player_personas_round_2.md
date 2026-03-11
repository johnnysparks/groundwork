# Decision: Player Personas — Rounds 2 & 3

**Date:** 2026-03-11T18:00:00
**Author:** Manager
**Status:** Active

## Context

Round 1 (5 personas) validated the trust/readability loop. Sprint 3 fixed the top blockers. We need 10 new divergent personas to maximize surface area before the next dev sprint.

## Round 2 — New Systems & Tools (5 personas)

| # | Persona | Focus axis | Stress-tests |
|---|---------|-----------|---------------------|
| 1 | **Terraformer** | Terrain sculpting via fill | Batch UX, terrain as medium, GAME-02 |
| 2 | **Patience Gardener** | Long-run sims (300+ ticks) | Endgame stagnation, SIM-04, emergent states |
| 3 | **Seed Bomber** | Mass planting at scale | Scale perf, protection at volume, visual noise |
| 4 | **Hydrologist** | Water as primary toy | SIM-04, GAME-07, GAME-04, flow edges |
| 5 | **First-Timer** | Zero-context cold start | Onboarding, help text, error messages |

## Round 3 — Playstyles & Edge Cases (5 personas)

| # | Persona | Focus axis | Stress-tests |
|---|---------|-----------|---------------------|
| 6 | **Storyteller** | Emotional/narrative play | Inspect as narrative, emotional beats, identity |
| 7 | **Save Scummer** | Multi-save experimentation | `--state` workflow, save/load, branching |
| 8 | **Vertical Farmer** | Multi-level Z navigation | Underground chambers, light/water delivery, Z UX |
| 9 | **Speedrunner** | Fastest root optimization | Critical path, optimal placement, info gaps |
| 10 | **Destructor** | Break everything (QA) | Error handling, edge cases, fill + protection |

## Coverage Matrix

| Axis | Round 1 | Round 2 | Round 3 |
|------|---------|---------|---------|
| Batch tools / fill | - | Terraformer, Bomber | Destructor |
| Long-run behavior | - | Patience Gardener | - |
| Water-as-system | - | Hydrologist | - |
| Scale / performance | - | Seed Bomber | Destructor |
| Onboarding | Weekend Gardener | First-Timer | - |
| Emotional play | - | - | Storyteller |
| Save workflow | - | - | Save Scummer |
| Vertical / Z-axis | Spelunker | - | Vertical Farmer |
| Optimization | Optimizer | - | Speedrunner |
| Robustness / QA | - | - | Destructor |

## Expected Outcome

10 reports across 10 distinct axes. Should surface 3-5 new issues per round that Round 1 never touched.

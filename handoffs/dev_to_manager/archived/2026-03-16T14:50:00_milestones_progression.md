# Dev → Manager Handoff: Ecological Milestones
**Date:** 2026-03-16T14:50:00
**Status:** 17 sprints complete. Sim-side milestone tracking shipped.

## What Shipped

### Ecological Milestone Tracking (Sprint 17)
Addresses the planting progression feedback: "Players skip the discovery arc by planting trees immediately."

**Sim-side `EcoMilestones` resource:**
- Tier 0 (always available): moss, grass, clover
- Tier 1 (10+ groundcover leaf voxels): wildflower, daisy
- Tier 2 (2+ pollinators present): fern, berry bush, holly
- Tier 3 (4+ fauna, 3+ plant species diversity): oak, birch, willow, pine

Milestones are one-way (never revert) and track raw progress counts.
Updated every 20 ticks by the `milestone_tracker` system.

**7 WASM exports** for JS to read tier states and progress counts.

## Action Required: Default Priorities Team
The sim now exports milestone data. The **JS species picker** needs to:
1. Read `milestone_tier1_flowers()`, `milestone_tier2_shrubs()`, `milestone_tier3_trees()`
2. Filter the species list to only show unlocked tiers
3. Show progress indicators toward next unlock (using raw count exports)

The sim provides the data; the UI decides presentation.

## Also Shipped: Seasonal Day Phase (Sprint 16)
DayPhase resource with growth multiplier (day=100%, night=50%). WASM exports for JS sync.

## Simulation Workstream Complete
This is the sim-side foundation for the learning arc. With 18 interaction chains + milestone progression gating, the game vision's discovery arc is fully supported in the simulation. Further work on this workstream would be at diminishing returns — the next improvements are UI-side (species picker filtering, progress display, quest integration).

## 98 tests pass, 0 regressions.

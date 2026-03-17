# Build Notes — Sprints 231-234: Species Emergence Arc Complete

**Date:** 2026-03-18T04:00:00
**Sprints:** 231-234
**Status:** Shipped

## What Changed

### Sprint 231: Condition-Based Species Emergence (P0)
**`crates/groundwork-sim/src/systems.rs`**
- New `pick_species_from_conditions()` function scores all 12 species against local water, light, nutrient, shade tolerance
- Maturity gating: groundcover pioneers → flowers → shrubs → trees (requires 10+ groundcover, 20+ total)
- Temporal bias: early ticks favor fast growers
- Replaces `unwrap_or(0)` that made all density-painted seeds become oak

### Sprint 232: Quest Chapter 4 — "The Garden Grows"
**`crates/groundwork-web/src/ui/quests.ts`**
- "Watch it grow" — 100 idle ticks without tool use
- "An uninvited guest" — wild species appears from seed dispersal
- Tracks idle ticks and species count at last plant

### Sprint 233: Neighbor Influence
**`crates/groundwork-sim/src/systems.rs`**
- Scans 8-voxel radius for existing plants around germinating seeds
- Clover → +40 tree score (nitrogen handshake)
- Groundcover → +25 flower score (succession)
- Tree canopy → +25 shade-tolerant score (canopy effect)
- Same species → -15 score (diversity pressure)

### Sprint 234: Emergence Feedback + Chapter 5
**`crates/groundwork-web/src/main.ts`**
- 10 condition-attribution messages explain WHY species appeared
- "A willow grew near water — it loves the moisture"
- "Fern appeared in the shade — it thrives under the canopy"
- "Clover is fixing nitrogen — nearby trees will grow faster!"

**`crates/groundwork-web/src/ui/quests.ts`**
- Chapter 5 "Conditions Matter": sow near water vs dry ground
- Tracks water proximity at seed placement (Manhattan distance scan)

## Test Results
- 111 unit tests + 5 fauna integration = all pass
- 3 new tests: condition_based_variety, mature_garden_trees, neighbor_influence
- Type-check clean (zero errors)

## Density Playtest Results
- 18 fauna by tick 933 from density-painted garden
- Groundcover appears first, then trees follow (maturity gating works)
- Visual diversity confirmed: different canopy shapes, groundcover rings

## Discovery Arc (Now Complete)
1. Player paints density zones → groundcover appears first
2. Discovery messages explain conditions: "Moss — bare soil pioneer"
3. As garden matures, flowers then shrubs then trees emerge
4. Chapter 4: garden grows without player (dispersal + idle watch)
5. Chapter 5: conditions matter (water vs dry → different species)
6. Neighbor influence: clover → more trees, canopy → shade species

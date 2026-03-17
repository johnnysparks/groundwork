# Build Notes — Sprints 238-243: Ecology Visibility + Density Influence

**Date:** 2026-03-18T06:00:00
**Sprints:** 238-243
**Status:** Shipped

## What Changed

### Sprint 238: Mycorrhizal Network Particles
**`crates/groundwork-web/src/rendering/ecology.ts`**
- New `mycorrhizal` color palette (soft violet/lavender)
- Scans underground for same-species Root voxels within 6 voxels using species_id from nutrient_level byte
- Emits slow-drifting violet particles at midpoints between connected root zones
- Makes the "wood-wide web" visible in x-ray view

### Sprint 239: Pine Allelopathy Particles
**`crates/groundwork-web/src/rendering/ecology.ts`**
- New `allelopathy` color palette (amber-red/rust)
- Scans for pine Root voxels (species_id=3) near surface
- Emits slow upward acid-seep particles from acidified soil zones
- Tip updated to reference visible acid zone

### Sprint 240: Nurse Log Glow Particles
**`crates/groundwork-web/src/rendering/ecology.ts`**
- New `nurseLog` color palette (warm golden/amber)
- Detects DeadWood voxels with seeds or roots nearby
- Warm golden glow rises from active nurse logs
- Tip updated to reference the 1.5x growth boost

### Sprint 241: Root Competition Stress Particles
**`crates/groundwork-web/src/rendering/ecology.ts`**
- New `rootCompetition` color palette (stress red-orange)
- Detects soil voxels with Root voxels from different species adjacent
- Red-orange stress particles at species boundaries underground
- Makes the "Root War" visible in x-ray view

### Sprint 242: Density Influence on Species Selection
**`crates/groundwork-sim/src/systems.rs`**
- New `local_seed_density` parameter in `pick_species_from_conditions()`
- Dense sowing (5+ seeds in radius 5): groundcover +30, others -10
- Moderate density (3+): groundcover +15
- Sparse: no modifier (conditions decide freely)
- Counts Seed voxels in radius 5 at germination time
- New test: `dense_sowing_favors_groundcover` (2500 samples)
- **Completes all 5 items from density-not-species decision**

### Sprint 243: Ecological Discovery HUD Messages
**`crates/groundwork-web/src/main.ts`**
- Mycorrhizal discovery: triggers when 2+ same-species trees are close with health difference
- Light competition discovery: triggers when a tree has low health near a larger tree
- Pine allelopathy discovery: triggers when a non-tolerant species has low health near a pine
- All are one-time messages with species names for specificity

## Test Results
- All unit tests pass (including --include-ignored species emergence tests)
- All 5 integration tests pass
- TypeScript type-check clean (zero errors)
- WASM rebuilt successfully
- Workspace compiles clean

## Underground Particle Summary
The x-ray view now shows 6 distinct ecology particle types:
1. **Water absorption** (blue) — roots absorbing from wet soil
2. **Nitrogen handshake** (green) — groundcover + tree interaction
3. **Mycorrhizal network** (violet) — same-species health sharing
4. **Pine allelopathy** (amber-red) — acidic soil zone
5. **Nurse log** (golden) — dead wood nurturing nearby seedlings
6. **Root competition** (red-orange) — different species competing for water

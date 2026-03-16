# Gameplay Systems Wiki

This wiki documents the intent, mechanics, and design rationale behind every simulation system in GROUNDWORK. It's the non-code surface for iterating on gameplay — change the design here first, then update the code to match.

## Pages

### Core Systems
- [World & Scale](world.md) — Grid dimensions, terrain, voxel materials
- [Water](water.md) — Springs, flow, soil absorption, weather
- [Light](light.md) — Propagation, attenuation, day-night cycle
- [Soil](soil.md) — Composition, evolution, nutrient cycling

### Growth & Competition
- [Seeds & Germination](seeds.md) — Growth rates, territorial suppression, nurse logs
- [Tree Growth](tree-growth.md) — Stages, resource accumulation, health dynamics
- [Branching](branching.md) — Space colonization, phototropism, self-pruning
- [Competition](competition.md) — Shade stress, root wars, crowding death, carrying capacity

### Ecology
- [Species Catalog](species.md) — All 12 species with parameters and niches
- [Fauna](fauna.md) — All 6 fauna types: spawning, behavior, ecological effects
- [Interaction Chains](interactions.md) — The 18 discoverable ecological relationships
- [Succession & Recovery](succession.md) — Pioneer succession, drought recovery, decomposition

### Progression
- [Discovery & Milestones](discovery.md) — Species discovery, tier unlocks, learning arc
- [Weather](weather.md) — Rain, drought, seasonal rhythm

### Design Intent
- [Player Learning Arc](learning-arc.md) — Hour 1 → Hour 20 discovery progression
- [Balance Philosophy](balance.md) — Tuning principles, competitive benchmarks

## How to Use This Wiki

**For design iteration:** Change the wiki page first. Discuss the intent. Then update the sim code to match. The wiki captures *why* — the code captures *how*.

**For balance tuning:** Each page lists the specific numbers (thresholds, rates, multipliers). Propose changes as wiki edits before touching `systems.rs`.

**For onboarding:** New contributors read the wiki to understand the simulation's design intent before diving into code.

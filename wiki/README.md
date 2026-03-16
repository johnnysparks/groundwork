# Gameplay Systems Wiki

This wiki documents the intent, mechanics, and design rationale behind every simulation system in GROUNDWORK. It's the non-code surface for iterating on gameplay — change the design here first, then update the code to match.

## Pages

### Core Systems
- [World & Scale](world.md) — Grid dimensions, terrain, voxel materials
- [Water](water.md) — Springs, flow, soil absorption, weather
- [Light](light.md) — Propagation, attenuation, day-night cycle
- [Soil](soil.md) — Composition, evolution, nutrient cycling
- [Garden Gnome](gnome.md) — Player's agent: task queue, movement, tools, ghost overlays

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

## Screenshots

Wiki pages reference images in `wiki/images/`. These are **automatically regenerated** whenever the game design or models change:

```bash
# Regenerate all wiki screenshots (requires WASM build + Playwright chromium)
cd crates/groundwork-web
npm run wasm                           # Build WASM module
node scripts/capture-wiki-images.mjs   # Capture species + interaction screenshots
./screenshot.sh                        # Capture 14-shot tour (hero, angles, overlays, x-ray)
```

This captures:
- **Species thumbnails:** each species at every growth stage + root x-ray (`images/species/`)
- **Interaction scenes:** nitrogen handshake, canopy layers, pine territory, competition, root wars (`images/interactions/`)

The capture script is deterministic — same sim state produces same screenshots. When models or rendering change, re-run the script and the wiki stays current.

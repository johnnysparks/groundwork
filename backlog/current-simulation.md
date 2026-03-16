# Backlog — Simulation Enhancement Workstream
**Last updated:** 2026-03-16T12:10:00
**Theme:** Bring the garden to life through simulation depth

---

## Just Shipped ✓
- Canopy shade competition (doubled leaf attenuation)
- Territorial seed suppression (no germination near trunks)
- Youth vulnerability (seedlings die 3-4× faster under stress)
- Root water competition (shared pool)
- Species-aware seed spacing
- **Dead tree decomposition** — DeadWood passively decays into nutrient-rich soil (~2500 ticks). Moisture accelerates. Beetles further accelerate via fauna_effects.
- **Canopy Effect** — Shade-tolerant species (fern, moss, holly) get 1.5× growth boost in moderate shade (light 5-30). Creates undergrowth niche.
- **Pollinator Bridge** (SIM-12) — Bees/butterflies near trees boost health recovery +0.005/pollinator. Creates flower→bee→healthier tree loop.
- **Pine Allelopathy** (SIM-16) — Pine roots acidify soil (pH drops 5x faster). Non-tolerant seeds grow at half speed in acidic soil. Pine/fern/moss immune.
- **Bird Express Enhancement** — Birds carry species-specific seeds (not just oak), enrich soil with droppings, spawn more readily near berry bushes.
- **Drought Recovery** — Dead trees with wet roots slowly revive as Saplings. "My tree came back!"
- **Willow Water Affinity** — Willows grow 2× faster near water. "Plant willows by the stream."
- **Birch Pioneer Vigor** — Birch grows 1.5× faster in open ground (no nearby trunks).
- **Berry-Bird Symbiosis** — Berry bushes grow 1.5× faster near birds. Completes the bird loop.
- **Grass/Clover Soil Binding** — Grass and clover roots increase soil clay (better water retention).

---

## Next Up — P1

### SIM-14: Root War Visualization Data
Export per-tree root count and water intake via WASM so the renderer can show:
- Root territory boundaries in x-ray mode
- Which tree is "winning" the root competition
- Soil depletion zones around aggressive root systems

### SIM-15: Seasonal Growth Variation
Tie growth rates to the day cycle phase:
- Dawn/day: normal growth
- Dusk/night: reduced growth (plants need light)
- Creates visible rhythm in the garden

### SIM-17: Nurse Log Effect
Dead trees provide shelter for seedlings — seeds landing on/near DeadWood get a germination bonus and shade protection. Creates the forest succession pattern.

---

## P2 — Future

- Wind-based seed dispersal (seeds drift laterally)
- Mycorrhizal networks (trees sharing nutrients underground)
- Fire mechanics (dry gardens can burn, pioneer succession follows)
- Disease spread (fungal infections between crowded same-species)

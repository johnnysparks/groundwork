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

---

## Next Up — P0

---

## Next Up — P1

### SIM-13: Water Stress Propagation
Plants that can't get enough water should show progressive damage stages:
- Mild drought: slower growth (already happens)
- Moderate drought: leaves start dying from tips inward
- Severe drought: entire canopy dies, trunk survives if roots still find water
- Recovery: new leaf growth from surviving branches when water returns

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

---

## P2 — Future

- Wind-based seed dispersal (seeds drift laterally)
- Mycorrhizal networks (trees sharing nutrients underground)
- Fire mechanics (dry gardens can burn, pioneer succession follows)
- Disease spread (fungal infections between crowded same-species)

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

---

## Next Up — P0

### SIM-10: Dead Tree Decomposition
Dead trees (DeadWood) should attract decomposers (beetles/worms) and gradually break down, enriching nearby soil. Currently dead trees just sit there forever.
- DeadWood voxels slowly convert to soil over ~200 ticks
- Adjacent soil gets organic matter boost (feeds soil_evolution)
- Beetles should preferentially spawn near DeadWood
- Creates the nutrient cycling loop: live tree → dead tree → soil → new tree

### SIM-11: Canopy Microhabitat (Shade Benefits)
Shade-tolerant species (fern shade_tolerance=30, moss=20) should grow *faster* under canopy, not just survive. Currently shade only hurts plants.
- Add a "shade_bonus" in tree_growth: if light is below a threshold AND species.shade_tolerance is very high (low number), boost growth
- Creates the undergrowth layer: tall oaks → ferns underneath → moss below ferns
- This is the Canopy Effect from the Big Yeses list

### SIM-12: Pollinator Bridge Strengthening
Flower clusters attract pollinators, but pollinators don't visibly improve nearby plants enough.
- Pollinators within range of a tree/shrub should boost its health recovery rate
- The effect should be visible: plants near active pollinators visibly greener
- Creates the pollination loop: flowers → bees → healthier neighbors → more flowers

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

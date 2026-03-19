# Playtest Feedback: Overlay Visual Overhaul (Sprint 349)

**Date:** 2026-03-19T11:50:00
**Source:** Automated Playwright visual evaluation

## What Worked
- **Water overlay** is immediately readable: vivid red terrain with blue pond/spring areas. The thermal palette (red=dry, blue=wet) is intuitive and requires no legend.
- **Light overlay** is the most dramatic: deep violet under canopy shadows, bright yellow in open areas. Shade patterns from tree canopies are clearly visible as purple zones.
- **All three overlays are now visually distinct** from each other. Before this sprint, all three looked identical due to similar warm-amber ramps.

## What Needs Work
- **Nutrient overlay** is the weakest — green patches visible near plants but most terrain shows brown/charcoal which blends somewhat with terrain. Values may be legitimately low across most soil, but a slightly brighter floor for the green channel could help.
- **No mode indicator on screen**: when cycling overlays, there's no visible label telling the player which overlay mode is active. The HUD should show "Water" / "Light" / "Nutrient" when an overlay is on.
- **Post-processing washes out colors**: bloom + color grading warm-shifts everything. Overlays compensate with 0.85 opacity and depthTest=false, but a future option to disable post-processing on the overlay layer would help.
- **Overlay coverage gaps**: cells where no Soil exists in the scan column get no overlay quad (trees on bare stone). Could fall back to reading the first non-air cell's data even for Water/Nutrient.

## Priorities
- P2: HUD overlay mode indicator
- P2: Nutrient floor brightness boost
- P3: Per-layer post-processing bypass

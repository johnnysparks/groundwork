# Player -> Manager Handoff: Garden Designer Aesthetic Builder

**Date:** 2026-03-11T15:00:00
**Persona:** The Garden Designer
**Session:** CLI playtest, ticks 0–120, 81 seeds placed in four distinct patterns
**Source feedback:** feedback/2026-03-11T15:00:00_garden_designer_aesthetic_builder.md

## 1. Observed

- Built a 24×24 walled garden with stone borders, water cross channels, and four quadrants of seeds (checkerboard, dense rows, border ring, diagonal X). Required ~300 individual `place` commands.
- Water spread from channels and filled the entire garden interior within ~30 ticks, erasing all air gaps and compositional detail.
- 61 of 81 seeds grew into roots by tick 120. Growth radiated outward from water channels — seeds closer to water grew first.
- 20 seeds remained ungrown at tick 120, primarily in the SE quadrant (diagonal pattern farthest from water).
- Wet soil (`%`) appeared at Z=15 beneath the garden, creating a visible moisture shadow that mirrored the water above. 351 wet soil voxels.
- Inspected seeds showed `water_level: 0` even when completely surrounded by water tiles. Roots also showed `water_level: 0`.
- Z=17 (above garden) was empty air — no vertical growth or visual story above Z=16.
- Stone is the only material that blocks water.

## 2. Felt

- **Tick 0 was the peak.** The garden looked designed and intentional before the simulation ran. Four distinct planting patterns were readable, water channels were clean, the stone frame was satisfying.
- **By tick 30, disappointment.** Water flooding reduced my composed garden to three visual characters: `@`, `~`, and scattered `s`/`*`. All compositional effort was erased.
- **The moisture shadow was the highlight.** Seeing `%` wet soil at Z=15 was the most beautiful and ecologically meaningful moment. It told a story the surface view couldn't.
- **Placement was a chore, not a joy.** 300 individual CLI commands to build one garden. The design process should be the fun part but it was pure tedium.
- **Growth was underwhelming.** `s` → `*` reads as a character swap, not as something living emerging.

## 3. Bugs

| Title | Severity | Notes |
|-------|----------|-------|
| Seeds/roots show water_level=0 despite being submerged in water | Major | Inspect output misleading; growth still happens for some seeds. Possibly adjacency-based growth ignoring internal water_level |
| Water floods all non-stone space uncontrollably | Major (design gap) | Air gaps don't slow water. No tool to drain. Makes designed layouts impossible to maintain |

## 4. Confusions

- Why do some seeds grow and others don't? Growth seems proximity-based but there's no feedback explaining what a seed needs.
- Why does water_level=0 on a seed that's surrounded by water? The inspect tool undermines trust in the simulation.
- How is growth triggered? The handoff said ~40 ticks with water+light. Some seeds had 120 ticks, light=229, surrounded by water, and still didn't grow.

## 5. What made me want to keep playing

- Seeing my garden layout rendered at tick 0 — it looked like someone designed it
- The moisture shadow at Z=15 — genuinely beautiful cross-layer visual storytelling
- Growth radiating from water channels — felt ecological and directional
- Stone borders containing water — made the space feel intentional and structured

## 6. What made me want to stop

- Water flooding destroyed my composition within 30 ticks
- Voxel-by-voxel placement for a designed layout is brutally tedious
- Growth transformation (`s` → `*`) was visually flat — no sense of something emerging
- No tools to fix problems (drain water, remove unwanted spread, undo)
- 20 seeds stuck with no feedback on why they won't grow

## 7. Requests

**Prioritized by impact on the designer playstyle:**

1. **P0: Batch placement commands** — fill rectangle, draw line, flood-fill. Without this, garden design is unplayable as a creative activity.
2. **P1: Water containment / absorption rework** — Water should soak into soil (becoming wet soil) rather than pooling on the surface. This fixes flooding AND enriches the underground view.
3. **P1: Growth stage visualization** — At least 3 stages (seed → sprout → plant) so growth feels like growth, not a swap.
4. **P1: Seed growth feedback** — When inspecting a seed, show what it needs: "needs water" / "needs light" / "growing (tick 12/40)".
5. **P2: Lightweight water barrier** — Wood, clay, or compacted soil. Something that blocks water without the visual weight of stone.
6. **P2: Water depth visualization** — Different characters for shallow vs deep water.
7. **P2: Tilled soil material** — A "prepared bed" character that absorbs water and reads as intentional garden ground.

# Decision: SimCity-style Zone Gameplay

**Date:** 2026-03-15T20:30:00
**Context:** After 13 sprints of visual/sim work, the game still "looks like you don't do anything." Voxel-by-voxel terraforming is tedious, not sticky.

## The Shift

**From:** Place individual seeds/water at specific voxel coordinates (Minecraft-style)
**To:** Designate zones and watch them fill (SimCity-style)

## Core Loop

1. **Zone it** — drag to designate an area as "oak grove", "flower meadow", "irrigation"
2. **Watch it grow** — the zone auto-populates and the sim runs fast enough to see results in seconds
3. **See dynamics** — overlays show water flow, nutrients, light competition
4. **React** — notice problems (drought, shade, crowding) and respond with new zones
5. **Discover** — species interactions create outcomes you didn't plan (nitrogen boost, bird-spread seeds)
6. **Score** — biodiversity/health metrics give you something to optimize

## What Changes

- **Seeds are zones**: clicking with a species selected fills an area, not a single voxel
- **Water is irrigation**: no water placement tool — players dig channels from natural water sources (springs, rain, ponds) to guide flow toward plants. Terrain shaping IS water management. See `decisions/2026-03-17T12:00:00_irrigation_replaces_watering_can.md`
- **Tools are broad**: shovel clears an area, not one voxel
- **Feedback is fast**: visible growth within seconds, not minutes of ticking
- **There's a score**: biodiversity, canopy coverage, fauna count — something to chase

## What Stays

- The sim (water, light, roots, fauna, interactions)
- The renderer (Three.js, x-ray, overlays, day cycle)
- The cozy aesthetic
- Species interactions and ecological discovery

## Extended By

**Garden Gnome Character & Zone-Planning System** (`decisions/2026-03-16T12:00:00_gardener_gnome_zone_planning.md`) — Executive mandate. Zones are no longer instant; a garden gnome character walks to each zone and executes the work. Adds ghost overlays, task queue, and character pacing to the zone concept.

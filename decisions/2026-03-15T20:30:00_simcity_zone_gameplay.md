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
- **Water is irrigation**: placing water creates a persistent source that waters surrounding soil
- **Tools are broad**: shovel clears an area, not one voxel
- **Feedback is fast**: visible growth within seconds, not minutes of ticking
- **There's a score**: biodiversity, canopy coverage, fauna count — something to chase

## What Stays

- The sim (water, light, roots, fauna, interactions)
- The renderer (Three.js, x-ray, overlays, day cycle)
- The cozy aesthetic
- Species interactions and ecological discovery

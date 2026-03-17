# Build Notes: Alive Garden Session (Sprints 141-149)

**Date:** 2026-03-16T21:00:00
**Sprints:** 141-149 (9 sprints)
**Theme:** Make the garden more alive — every system responds to time, weather, and ecology

## Session Summary

This session closed out the remaining P2 visual polish items and added new atmospheric layers. The garden now responds to the day cycle (foliage + terrain tinting), weather (rain ripples on water), and ecology (bird seed-drops, growth celebrations, squirrel trust arc) at every level.

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 141 | Fauna-scaled ambient sound frequency | Web | main.ts |
| 142 | Growth stage visual transitions | Web | bridge.ts, particles.ts, main.ts |
| 143 | Stout tree proportions | Sim | tree.rs |
| 144 | Water caustics + wave displacement | Web | water.ts |
| 145 | Squirrel trust milestones + visuals | Web | main.ts, gardener.ts |
| 146 | Bird seed-drop particles | Web | ecology.ts |
| 147 | Foliage day-night color tinting | Web | foliage.ts, main.ts |
| 148 | Terrain day-night color tinting | Web | terrain.ts, main.ts |
| 149 | Rain ripple rings on water | Web | water.ts, main.ts |

## Key Technical Decisions

- **Tree stats bridge**: Added packTreeStats/getTreeStatsView/readTreeStat to bridge.ts for zero-copy tree data access from JS. 12 bytes per tree: [species_id, health, stage, pad, rootX, rootY, rootCount, waterIntake].

- **Growth stage detection**: Track previous tree stages in a Map<string, number> keyed by "rootX,rootY". Compare after each tick. Emit celebratory 20-particle burst + HUD message for stage advances. Discovery chime on mature+.

- **Tree proportions (sim change)**: Increased trunk_radius_m by ~60-100% and crown_radius_m by ~25-30% for all 4 tree species. Pine height reduced 2.8→2.5m. All 110 sim tests pass.

- **Shared solid material**: Created getSolidMaterial() in terrain.ts so all chunk meshes share one material. Enables tinting via material.color against vertex colors.

- **Day-night tinting**: Both foliage (via uDayTint shader uniform) and terrain (via material.color) shift through warm→neutral→amber→cool across the day cycle. Foliage uses vertex shader multiplication; terrain uses Lambert material color.

- **Rain ripple rings**: 6 procedural drop points cycle continuously during rain. Each drops an expanding concentric ring that fades over time. Driven by uRainStrength uniform.

## Big Yes Interaction Chain Completeness

| Chain | Visual Status |
|-------|--------------|
| Nitrogen Handshake | ✅ Green shimmer particles |
| Pollinator Bridge | ✅ Pollination trails + acting boost |
| Root War | ✅ X-ray mode + species-colored roots |
| Bird Express | ✅ Seed-drop particles + HUD messages |
| Pioneer Succession | ✅ Recovery messages |
| Canopy Effect | ✅ Shade particles |
| Squirrel Trust | ✅ Trust milestones + emotion particles |

## What's Next (Recommended)

1. **Mobile drag-to-zone** — zone painting on touch devices (P2)
2. **SSAO tuning** — currently disabled, needs retuning with warm PCF shadows
3. **Multiple gnomes** — P2 expansion feature
4. **Biome variety** — second biome would validate the biome-ready architecture

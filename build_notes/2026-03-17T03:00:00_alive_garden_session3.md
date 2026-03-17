# Build Notes: Alive Garden Session 3 (Sprints 163-171)

**Date:** 2026-03-17T03:00:00
**Sprints:** 163-171 (9 sprints)
**Theme:** Sensory depth — making every moment feel alive through particles, sound, and atmosphere

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 163 | Flower petal scatter on wind gusts | Web | particles.ts, main.ts |
| 164 | Seed sprouting pop particles | Web | particles.ts |
| 165 | Ambient frog chorus at dusk near water | Web | ambient.ts, main.ts |
| 166 | Falling leaf colors match tree species | Web | leaves.ts, main.ts |
| 167 | Gnome emotion sounds on state transitions | Web | sfx.ts, main.ts |
| 168 | Rain splash particles at ground impact | Web | rain.ts, particles.ts, main.ts |
| 169 | Dawn mist wisps | Web | mist.ts (new), main.ts |
| 170 | Dust motes extended through golden hour | Web | dustmotes.ts |
| 171 | Squirrel digging particles when caching | Web | ecology.ts |

## Key Technical Decisions

- **Flower petal scatter**: `GrowthParticles.flowerPositions[]` tracks Leaf voxels with species_id 7 (wildflower) or 8 (daisy) during `detectGrowth()`. On gust, `emitPetalBurst()` picks up to 6 random sources, 2-3 petals each, with species-matched colors (pink-purple or warm yellow).

- **Seed sprout pop**: Tracks `prevSeedKeys` (Set<number>) across ticks. When a position transitions from Seed material to vegetation material, emits an 8-particle bright green outward burst — the visible "birth" moment.

- **Frog chorus**: Two oscillators at 120Hz and 145Hz (sine with ±15-20Hz FM wobble) gated by rhythmic oscillators at 1.5Hz and 1.1Hz. Activates at dusk/night when waterCount > 20. Very quiet (0.02 gain).

- **Species-colored falling leaves**: `FallingLeaves.setTreeSpecies()` receives garden species IDs from stats. Falling leaf color sampled from SPECIES_LEAF_COLORS (autumn-tinted versions of foliage palette) weighted by species presence.

- **Gnome emotion sounds**: `playGnomeSound(state)` — triggered on state transitions only (prevGnomeState tracking). Working=low grunt, Inspecting=ascending "hmm", Resting=descending filtered noise sigh, Eating=two quick clicks.

- **Rain splash**: `RainRenderer.onSplash()` callback pattern. ~8% of ground-impact drops trigger `emitRainSplash()` — tiny blue-white particle with short life and outward velocity.

- **Dawn mist**: New `MistRenderer` — 30 large translucent point sprites with gaussian falloff, drifting lazily near ground level during dawn (0.10-0.30). Separate from dew sparkles (which are sharp twinkles on foliage).

- **Squirrel dig**: Reuses existing `emitSoilDisturbance()` from worm system for squirrels in Acting state. Visible soil puffs at feet during caching.

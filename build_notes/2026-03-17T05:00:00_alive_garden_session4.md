# Build Notes: Alive Garden Session 4 (Sprints 172-175)

**Date:** 2026-03-17T05:00:00
**Sprints:** 172-175 (4 sprints)
**Theme:** Completing the sensory web — audio layers + visual feedback for every ecosystem role

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 172 | Ambient beetle clicking sound | Web | ambient.ts, main.ts |
| 173 | Water babble scales with water volume | Web | ambient.ts, main.ts |
| 174 | Night owl hoot | Web | sfx.ts, main.ts |
| 175 | Seed dispersal trail at landing sites | Web | particles.ts |

## Key Technical Decisions

- **Beetle clicking**: Square wave at 800Hz, highpass-filtered (2000Hz), gated at 6Hz for rhythmic clicking. Volume scales with beetle count, daytime only. Max 0.008 gain — extremely subtle.

- **Water babble scaling**: `waterSoundGain` node exposed for dynamic control. Base 0.04 with no water, scales to 0.12 at 200+ water voxels. Makes irrigation sonically rewarding.

- **Owl hoot**: Two descending sine tones (380→320Hz, 340→280Hz) with 4Hz vibrato. Spaced 0.4s apart for classic "hoo-hoo". Triggered every 30-70s during deep night (0.80-0.10 day cycle).

- **Seed dispersal trails**: `emitSeedLanding()` — 4 golden SEED_COLORS particles descending from 2-5 voxels above the new seed position. Triggered in `detectGrowth()` when a seed key appears that wasn't in `prevSeedKeys`. Makes Bird Express and wind dispersal visible.

## Audio Layer Summary (Complete)

The garden now has 10 concurrent ambient audio layers, each responding to different garden state:
1. **Water spring** — scales with water count (0.04-0.12)
2. **Rain patter** — weather state Rain
3. **Crickets** — dusk/night
4. **Wind rumble** — weather-scaled + gust spikes
5. **Leaf rustle** — foliage count × wind strength
6. **Pollinator hum** — bee/butterfly count
7. **Frog chorus** — dusk/night + water present
8. **Beetle clicking** — beetle count, daytime
9. **Owl hoot** — deep night periodic
10. **Dawn bird chorus** — dawn (increased frequency)

Plus SFX: plant, dig, bird call, buzz, growth shimmer, discovery chime, rain/drought onset, wind gust, gnome emotions (working/inspecting/resting/eating).

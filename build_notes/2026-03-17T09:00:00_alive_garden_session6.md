# Build Notes: Alive Garden Session 6 (Sprints 183-190)

**Date:** 2026-03-17T09:00:00
**Sprints:** 183-190 (8 sprints)
**Theme:** Garden state drives atmosphere + fauna audio completeness

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 183 | Dawn soil steam wisps | Web | particles.ts, main.ts |
| 184 | Cricket tempo varies with night warmth (Dolbear's Law) | Web | ambient.ts |
| 185 | Water surface day-cycle tinting | Web | water.ts, main.ts |
| 186 | Squirrel chitter sound | Web | sfx.ts, main.ts |
| 187 | Bee waggle dance figure-8 particles | Web | ecology.ts |
| 188 | Mist density scales with water volume | Web | mist.ts, main.ts |
| 189 | Dew drop tinkle sound at dawn | Web | sfx.ts, main.ts |
| 190 | Garden age milestone celebrations | Web | main.ts |

## Key Technical Decisions

- **Dawn soil steam (183)**: Pale amber/beige particles rise slowly from random ground positions during 0.22-0.32 day cycle. ~3/sec emission rate. Complements existing mist system (which is diffuse ground-level fog) with localized surface steam.

- **Dolbear's Law crickets (184)**: Cricket rhythm oscillator frequency ramps from 2.5Hz at warm dusk (0.65) down to 1.5Hz at cold midnight. Stored OscillatorNode refs (`cricketRhythm1/2`) for runtime frequency control via `linearRampToValueAtTime`.

- **Water day tinting (185)**: Added `uDayTint` vec3 uniform to water fragment shader. Applied as multiplicative tint to final `lit` color. Shares the same r/g/b temperature curve as `setTerrainDayTint`. Water now reflects golden hour warmth and moonlight blue.

- **Squirrel chitter (186)**: 5-6 rapid descending sine clicks at 1200-1600Hz. Each click sweeps down 30%. Integrated into the ambient fauna sound timer loop alongside bird/bee sounds.

- **Waggle dance (187)**: When a Bee is in Acting state (pollinating), 8 particles placed along a lemniscate (figure-8) curve using parametric equations: `x = r*sin(θ)`, `z = r*sin(θ)*cos(θ)`. Phase advances 0.4 radians per emission for animation.

- **Mist density scaling (188)**: `MistRenderer.density` ranges 0.3 (dry) to 1.5 (100+ water voxels). Applied by setting `targetActive` to density instead of binary 1/0. Wet gardens produce visibly thicker morning mist.

- **Dew drop sound (189)**: High delicate sine (1800-2400Hz) with 0.2s decay. ~0.4/sec during dew window (0.15-0.35). Audio complement to the visual dew sparkle renderer.

- **Garden age milestones (190)**: Discovery chime + warm message at 1k/5k/10k/25k ticks. After 25k, repeats every 25k. Messages use garden growth metaphor: "roots taking hold" → "living ecosystem" → "tells its own story" → "world complete."

## Audio Layer Summary (Updated: 12 ambient + expanded SFX)

Ambient: water spring, rain, crickets (tempo varies!), wind, leaf rustle, pollinator hum, frog chorus, beetle clicking, owl hoot, dawn bird chorus (3 variants), wind chime (dense gardens), dew drop tinkle.

SFX: plant, dig, bird call/warble/robin, buzz, squirrel chitter, growth shimmer, discovery chime, rain/drought onset, wind gust, wind chime, gnome emotions, shooting star, dew drop, garden age milestones.

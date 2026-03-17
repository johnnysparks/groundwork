# Build Notes: Alive Garden Session 7 (Sprints 191-195)

**Date:** 2026-03-17T11:00:00
**Sprints:** 191-195 (5 sprints)
**Theme:** Player presence feedback + celebration moments + atmospheric audio

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 191 | Camera pan rustle particles | Web | orbit.ts, particles.ts, main.ts |
| 192 | Garden vitality drone at night | Web | ambient.ts, main.ts |
| 193 | Golden hour bloom boost | Web | effects.ts, main.ts |
| 194 | Fauna arrival sparkle burst | Web | particles.ts, main.ts |
| 195 | Tree growth creak sound | Web | sfx.ts, main.ts |

## Key Technical Decisions

- **Camera pan rustle (191)**: Added `getPanSpeed()` and `getCenter()` to OrbitCamera. When pan speed > 5 units/sec and foliage > 50, emits 2 tumbling green leaf fragments near camera center. Rate scales with pan speed. Makes the garden feel responsive to player movement.

- **Garden vitality drone (192)**: Two detuned low sines (65Hz + 98Hz = perfect fifth) with 0.08Hz breathing LFO. Max 0.006 gain — barely audible. Only active at night (0.75-0.15) when foliage > 500. Thriving night gardens have a distinct warm undertone.

- **Golden hour bloom (193)**: Bloom strength modulated by sine curve during 0.65-0.80 day cycle. Peaks at 0.45 (from base 0.25) at the center of golden hour. Added `setBloomStrength()` to PostProcessing interface.

- **Fauna arrival burst (194)**: 12 warm gold/white particles spiral outward from new fauna position. Required module-level `_particles` ref since detectEvents() runs outside the render closure.

- **Tree growth creak (195)**: Bandpass-filtered noise with Q=4 resonance, frequency sweeps 200-300Hz → 80Hz over 0.6s. Plays on every tree growth stage transition. Discovery chime still layers on top for mature+ stages.

## Audio Layer Summary (Updated: 13 ambient + 20 SFX)

Ambient: water spring, rain, crickets (Dolbear's Law), wind, leaf rustle, pollinator hum, frog chorus, beetle clicking, owl hoot, dawn chorus (3 variants), wind chime, dew tinkle, garden vitality drone.

SFX: plant, dig, bird call/warble/robin, buzz, squirrel chitter, dew drop, tree creak, growth shimmer, discovery chime, fauna arrival, rain/drought onset, wind gust, wind chime, gnome emotions, shooting star, garden milestones.

## Total Sprint Count: 195

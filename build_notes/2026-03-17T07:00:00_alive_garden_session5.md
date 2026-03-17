# Build Notes: Alive Garden Session 5 (Sprints 176-182)

**Date:** 2026-03-17T07:00:00
**Sprints:** 176-182 (7 sprints)
**Theme:** Emergent behaviors + weather transitions + audio variety

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 176 | Idle camera zoom breathe | Web | orbit.ts |
| 177 | Shooting star sound | Web | sfx.ts, main.ts |
| 178 | Plant die-off wilting particles | Web | particles.ts |
| 179 | Dawn chorus bird variety (3 songs) | Web | sfx.ts, main.ts |
| 180 | Post-rain leaf drip | Web | particles.ts, main.ts |
| 181 | Firefly blink synchronization | Web | fireflies.ts |
| 182 | Wind chime in dense gardens | Web | sfx.ts, main.ts |

## Key Technical Decisions

- **Idle zoom breathe (176)**: ±3% zoom oscillation at 0.15Hz sine wave during idle auto-orbit (45s threshold). Uses `idleElapsed` counter, directly modulates `camera.zoom` rather than `targetZoom` to avoid damping interference.

- **Shooting star sound sync (177)**: The star is entirely shader-side (`floor(uTime / 45.0)`). JS replicates this logic with `prevShootingStarSlot = floor(elapsed / 45)` to detect new star events and trigger the shimmer SFX.

- **Plant die-off detection (178)**: Compares `prevLeafPositions` set with current frame. Iterates the old set looking for keys missing from the new set, capped at 5 samples/tick. Emits 3 brown/amber `emitWilt()` particles per lost position.

- **Dawn chorus variety (179)**: Three bird call types — `playBirdCall()` (descending chirp), `playBirdWarble()` (4-note ascending wren trill), `playRobinSong()` (5-note melodic rise-and-fall). During dawn (0.2-0.3) randomly distributed 35/30/35%.

- **Post-rain leaf drip (180)**: `leafDripTimer` starts at 30-60s when weather transitions from Rain→Clear. `emitLeafDrip()` picks random positions from `prevLeafPositions` set. Drip rate tapers: starts ~6/sec, ends ~1/sec via linear interpolation of interval.

- **Firefly sync (181)**: Kuramoto-inspired coupling model. Global `syncPhase` advances at 2.5s period. After 15s active, coupling ramps linearly to 100% at 60s. When a firefly's blink state disagrees with the sync pulse, its `blinkTimer` drains faster (proportional to coupling). Creates emergent wave synchronization.

- **Wind chime (182)**: Pentatonic sine tones (C6, E6, G6, A6 with slight random detuning). 2-3 tones per chime, 1.2s decay. Triggers on 40% of wind gusts when `foliage.count > 300`. Dense gardens earn a distinct sonic signature.

## Audio Layer Summary (Updated)

11 ambient layers + growing SFX library:
1. Water spring (scales 0.04-0.12 with water count)
2. Rain patter
3. Crickets (dusk/night)
4. Wind rumble (weather-scaled + gust spikes)
5. Leaf rustle (foliage × wind)
6. Pollinator hum (bee/butterfly count)
7. Frog chorus (dusk/night + water)
8. Beetle clicking (beetle count, daytime)
9. Owl hoot (deep night periodic)
10. Dawn bird chorus (3 song variants: chirp, warble, robin)
11. Wind chime (dense gardens on gusts)

SFX: plant, dig, bird call variants, buzz, growth shimmer, discovery chime, rain/drought onset, wind gust, wind chime, gnome emotions, shooting star shimmer.

## Visual Polish Summary (Updated)

Post-rain leaf drip bridges rain→clear transition. Firefly synchronization rewards idle observation at dusk. Plant wilting particles mark die-off events.

# Build Notes: Alive Garden Session 9 (Sprints 201-210)

**Date:** 2026-03-17T15:00:00
**Sprints:** 201-210
**Theme:** Fauna behavior depth + weather atmosphere + environmental feedback

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 201 | Ambient gnat swarms | Web | gnats.ts, main.ts |
| 202 | Water ripple from flying fauna | Web | particles.ts, main.ts |
| 203 | Root growth crackle sound | Web | sfx.ts, particles.ts, main.ts |
| 204 | Butterfly flower landing | Web | fauna.ts |
| 205 | Heat shimmer during drought/midday | Web | effects.ts, main.ts |
| 206 | Bird perch idle animation | Web | fauna.ts |
| 207 | Worm soil disturbance particles | Web | particles.ts, main.ts |
| 208 | Rain puddle shimmer | Web | particles.ts, main.ts |
| 209 | Beetle trail shimmer | Web | particles.ts, main.ts |
| 210 | Wind streak particles | Web | particles.ts, main.ts |

## Key Technical Decisions

- **Gnat swarms (201)**: 5 clusters of 8 gnats each orbiting center points. Circular orbit + vertical bob. Only active during daylight (0.2-0.75) and when foliage > 200. Swarm centers relocate every 15s. Dark specks (not glowing) — silhouettes against the sky.

- **Water ripple (202)**: 8-particle expanding ring from particles.ts rather than shader uniform arrays. ~0.15/sec per flying fauna near water. Simpler and compatible with existing particle system.

- **Root crackle (203)**: Tracks root voxel count delta via `rootGrowthDelta` getter on GrowthParticles. Sound triggers when >5 new roots/tick at 30% probability. 3-4 rapid low-pass noise snaps — earthy, subtle.

- **Butterfly landing (204)**: Acting butterflies dip -0.4 height, wing flap slows from 4Hz to 1.2Hz with reduced amplitude (0.7→0.2). Creates visible distinction between flying and pollinating states.

- **Heat shimmer (205)**: Post-processing screen-space UV distortion. Rising wave pattern stronger at bottom (quadratic falloff). Full strength 0.003 during drought, subtle noon peak 0.0015. Bypassed when strength < 0.0001.

- **Bird perch (206)**: Idle birds lower -0.3 into canopy, wings tucked (rotation.z ±0.15 vs ±0.7 flying). Subtle head tilt (rotation.x oscillation at 0.8Hz). Wing animation switches to held position.

- **Worm trail (207)**: 2 earthy-brown particles per emission at ~0.5/sec during Seeking/Acting. Puff upward from soil surface — shows underground activity above ground.

- **Puddle shimmer (208)**: Stationary blue-white particles at ground level during rain (~2/sec). 1.5-3s lifespan, zero velocity. Cool reflective tint complements rain drops.

- **Beetle trail (209)**: Iridescent green-blue particles at ~0.4/sec during active states. Every fauna type now has a visible trail or behavior particle.

- **Wind streaks (210)**: Fast horizontal white particles during gusts, velocity follows slowly drifting wind angle (~2min rotation). Speed scales with gust strength. Makes wind visible.

## Fauna Behavior Matrix (complete)

| Fauna | Idle | Seeking | Acting | Leaving | Trail |
|-------|------|---------|--------|---------|-------|
| Bee | Flight bob (5Hz) | Flight bob | Waggle dance | Scale shrink | Pollen trail |
| Butterfly | Flight bob (3Hz) | Flight bob | Flower landing | Scale shrink | Pollen trail |
| Bird | Perch (tucked wings) | Flight bob (2Hz) | Extra bob | Scale shrink | — |
| Worm | — | Soil disturbance | Soil disturbance | — | Earthy puffs |
| Beetle | — | Iridescent trail | Iridescent trail | — | Green-blue shimmer |
| Squirrel | — | Scurry bob | Digging dip | — | — |

## Total Sprint Count: 210

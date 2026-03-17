# Build Notes: Alive Garden Session 10 (Sprints 211-220)

**Date:** 2026-03-17T17:00:00
**Sprints:** 211-220
**Theme:** Cross-system connections + nighttime depth + ecological feedback loops

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 211 | Firefly reflection on water | Web | fireflies.ts, particles.ts, main.ts |
| 212 | Squirrel footprint dust | Web | particles.ts, main.ts |
| 213 | Distant bird calls for dawn chorus | Web | sfx.ts, main.ts |
| 214 | Star reflections on water | Web | water.ts, main.ts |
| 215 | Sunrise/sunset bloom flash | Web | main.ts |
| 216 | Garden vitality scales ambient volume | Web | ambient.ts, main.ts |
| 217 | Decomposition fungi spore particles | Web | particles.ts, main.ts |
| 218 | Night moth particles near fireflies | Web | particles.ts, main.ts |
| 219 | Raindrop plink on splash impact | Web | sfx.ts, main.ts |
| 220 | Ecosystem health warm glow | Web | effects.ts, main.ts |

## Key Technical Decisions

- **Firefly reflection (211)**: `getActivePositions()` on FireflyRenderer returns lit firefly positions. Golden glow particles on water surface at ~0.8/sec. Only when firefly z <= GROUND_LEVEL+6 (near water).

- **Star reflections (214)**: Added `uNightAmount` uniform to water fragment shader. Same hash-based star grid as sky shader but with `sin()` wobble distortion to simulate ripple effect on reflections. ~8% cell density.

- **Bloom flash (215)**: Three-band bloom system — sunrise (0.22-0.28, +0.15), golden hour (0.65-0.80, +0.20), sunset (0.78-0.85, +0.12). Each uses sine curve for smooth rise/fall.

- **Vitality volume (216)**: Master ambient gain scales 0.6→1.0 based on 70% plant factor + 30% fauna factor. Ramps over 3 seconds for smooth transitions.

- **Fungi spores (217)**: Tracks up to 20 dead wood positions during detectGrowth scan. Brown-orange particles drift upward at ~0.15/tick. Makes nutrient cycling visible.

- **Night moths (218)**: Pale cream-white particles flutter in circular paths near active fireflies. ~0.3/sec. Adds nocturnal insect diversity.

- **Eco warmth glow (220)**: Color grade warmth (+0.015 at max) and saturation (+0.08 at max) scale with 50% plant health + 50% fauna health. Subtle but perceptible — thriving gardens feel warmer.

## Cross-System Connections Added This Session

| System A | System B | Connection |
|----------|----------|------------|
| Fireflies | Water | Golden glow reflections on surface |
| Fireflies | Particles | Moths drawn to light sources |
| Stars/Sky | Water | Star reflections with ripple wobble |
| Rain | Audio | Individual drop plinks (10% of splashes) |
| Ecosystem health | Post-processing | Warmth/saturation boost |
| Garden vitality | Audio | Master volume scaling |
| Dead wood | Particles | Fungi decomposition spores |
| Squirrels | Particles | Footprint dust trail |

## Total Sprint Count: 220

# Build Notes: Alive Garden Session 8 (Sprints 196-200)

**Date:** 2026-03-17T13:00:00
**Sprints:** 196-200 (5 sprints — SPRINT 200 MILESTONE!)
**Theme:** Fauna animation polish + atmospheric completeness + ecosystem celebration

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 196 | Fauna flight bob animation | Web | fauna.ts |
| 197 | Star twinkle variation | Web | sky.ts |
| 198 | Drought dust devils | Web | particles.ts, main.ts |
| 199 | Fog color follows day cycle | Web | main.ts |
| 200 | "The garden is alive" ecosystem milestone | Web | main.ts |

## Key Technical Decisions

- **Fauna flight bob (196)**: Bees bob at 5Hz (tight, fast), butterflies at 3Hz (lazy, floaty), birds at 2Hz (slow, wide arc). Each has slight banking roll via `rotation.z`. Offset by fauna index for phase variation. Layer on top of existing Acting-state bob.

- **Star twinkle (197)**: Each star cell gets unique phase and speed from `hash(cell + 100)` and `hash(cell + 200)`. Brightness modulates ±40% via sine wave at 0.5-2.0 Hz. Cheap — only adds 3 operations per star fragment.

- **Drought dust devils (198)**: 6 particles in upward spiral (lemniscate placement + tangential velocity). Tan/brown colors. ~0.3/sec during drought weather. Complements existing drought haze and foliage yellowing.

- **Fog day cycle (199)**: Fog color lerps toward desaturated day tint (r×0.8, g×0.82, b×0.85) at 0.02/frame. Drought fog override still takes priority. Night fog becomes cool blue, golden hour fog turns warm amber.

- **"The garden is alive" (200)**: One-time milestone when plants>1000, fauna≥5, species≥3. Discovery chime + warm message + 5 golden sparkle bursts across garden center. The moment when the player's work becomes a living world.

## Total Sprint Count: 200 🎉

From sprint 157 to 200, 43 sprints of "alive garden" polish have transformed the web renderer from a visual-only display into a full sensory experience with:
- 13 ambient audio layers responsive to garden state
- 20+ one-shot SFX tied to ecological events
- Weather-responsive particles (rain drip, drought dust, post-rain mist)
- Fauna-driven audio (every fauna type now has its sound)
- Day-cycle atmosphere (tinting, bloom, fog, stars, firefly sync)
- Player presence feedback (camera rustle, garden milestones)
- Ecological celebrations (fauna arrival, tree growth, "alive" moment)

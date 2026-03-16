# Build Notes: Weather Visuals & Ecology Events (Sprints 101-103)

**Date:** 2026-03-16T14:55:00
**Sprints:** 101-103

## What Changed

### Sprint 101: Rain Particle Renderer
- New `rendering/rain.ts`: 800 soft blue-white droplets fall during Rain weather
- Syncs with sim `WeatherState` via `getWeatherState()` bridge export
- Drops vary in speed (0.8-1.2x), stagger vertically, respawn at sky
- Dormant when Clear/Drought (zero per-frame cost when inactive)
- Gentle cozy rain feel, not a storm — matches game's emotional register

### Sprint 102: Weather Event Messages
- HUD event feed announces weather transitions:
  - "Rain begins — the garden drinks deeply"
  - "Drought — water runs low, roots dig deep"
  - "The rain passes — skies clear"
  - "Drought breaks — the soil can breathe"
- Tracked via `prevWeatherState` variable, compared each tick

### Sprint 103: Squirrel Messages + Drought Haze
- Added squirrel (FaunaType 5) arrival messages to FAUNA_MESSAGES
- Drought visual: fog color lerps toward warm amber (0.65, 0.50, 0.35)
- Fog density slowly increases during drought (max 0.004)
- DayCycle resets fog each update, drought accumulates gradually
- Completes the weather visual cycle: rain (particles) + drought (atmosphere)

## Architecture Notes
- Rain renderer follows same pattern as GrowthParticles: pooled Points geometry, shader material, dormant when inactive
- Weather state is polled each tick from WASM bridge (not event-driven) — simple and reliable
- Drought fog lerp uses `scene.fog.color.lerp()` at 2% per frame — settles in ~2 seconds
- DayCycle's `update()` sets fog each frame, so drought must re-apply its tint after — the `lerp` approach means drought gradually "wins" over the base fog color

## Session Summary (Sprints 97-103)
All work this session followed "make the garden more alive":
- Sprint 97: Water surface shimmer (sparkles, foam, contrast)
- Sprint 98: Seed visibility sparkles
- Sprint 99: Mobile performance preset
- Sprint 100: Mobile camera zoom + build notes
- Sprint 101: Rain particles
- Sprint 102: Weather event messages
- Sprint 103: Squirrel messages + drought haze

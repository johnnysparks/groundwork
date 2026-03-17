# Build Notes: Alive Garden Session 2 (Sprints 154-162)

**Date:** 2026-03-17T01:00:00
**Sprints:** 154-162 (9 sprints)
**Theme:** Dynamic atmosphere + garden-responsive audio + physical gnome + irrigation

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 154 | Dynamic wind gusts | Web | main.ts, leaves.ts, sfx.ts |
| 155 | Cloud shadow pulses | Web | daycycle.ts |
| 156 | Leaf rustle ambient sound | Web | ambient.ts, main.ts |
| 157 | Water tool removal + irrigation quest | Web | hud.ts, quests.ts, main.ts |
| 158 | Water flow sparkle particles | Web | water.ts, particles.ts, main.ts |
| 159 | Ambient pollinator hum | Web | ambient.ts, main.ts |
| 160 | Gnome footstep dust puffs | Web | particles.ts, main.ts |
| 161 | Heavier soil burst when working | Web | main.ts |
| 162 | Worm soil disturbance particles | Web | ecology.ts |

## Key Technical Decisions

- **Wind gusts**: Random timer (10-30s, weather-scaled). Adds 0.3-0.6 to base wind, decays over ~2s. Triggers leaf burst + whoosh sound. All existing wind-responsive systems (foliage sway, falling leaves, wind audio, leaf rustle) react naturally.

- **Cloud shadows**: 3 overlapping sine waves at different periods (20s, 35s, 63s) modulate sun intensity ±6-13% during daytime. Fades at night. Applied after DayCycle preset interpolation.

- **Garden-responsive audio**: Leaf rustle scales with foliage count + wind. Pollinator hum scales with active bee/butterfly count. Both use Web Audio API nodes with smooth ramp.

- **Water flow visualization**: `scanWaterFrontier()` finds water surface cells adjacent to non-water. When water count increases after tick, emits blue sparkle particles at frontier positions. Satisfying cascade as channels fill.

- **Irrigation P1 cleanup**: Filtered Water from BRIDGE_TOOLS in HUD buildTools(). Quest 'placeWater' now triggers on shovel use. Dead playWater sfx code removed.

- **Gnome physicality**: Footstep dust puffs during walking (2 particles/0.3s). Doubled frequency + count during working state. Earthy brown colors settle quickly.

- **Underground visibility**: Worm soil disturbance particles emit at GROUND_LEVEL above the worm's sim position, making underground activity visible from above.

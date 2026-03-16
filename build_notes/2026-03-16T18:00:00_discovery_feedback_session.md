# Build Notes: Discovery Feedback Session (Sprints 125-136)

**Date:** 2026-03-16T18:30:00
**Sprints:** 125-136 (12 sprints)
**Theme:** Make ecological discoveries visible, audible, and rewarding + nighttime atmosphere

## Session Summary

This session focused on closing the discovery feedback loop: when the garden does something the player didn't plan, the player should notice, understand, and be delighted. Every sprint adds a signal that makes the ecological web more legible.

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 125 | Mobile ecology particle optimization | Web | rendering/ecology.ts, main.ts |
| 126 | Wild plant notifications (fauna attribution) | Web | main.ts |
| 127 | Species-colored roots in x-ray | Web | rendering/terrain.ts, main.ts |
| 128 | Discovery chime sound | Web | audio/sfx.ts, main.ts |
| 129 | Companion species suggestions | Web | main.ts |
| 130 | Weather transition sounds | Web | audio/sfx.ts, main.ts |
| 131 | Idle auto-orbit camera | Web | camera/orbit.ts, main.ts |
| 132 | Session wrap-up (build notes + handoff) | Docs | backlog, build_notes, handoffs |
| 133 | Night stars in sky dome | Web | lighting/sky.ts, lighting/daycycle.ts |
| 134 | Moonlight (blue hour preset) | Web | lighting/daycycle.ts |
| 135 | Ambient wind sound | Web | audio/ambient.ts, main.ts |
| 136 | Shooting stars | Web | lighting/sky.ts, lighting/daycycle.ts |

## Key Technical Decisions

- **Player species tracking**: `_playerPlantedSpecies` Set tracks which species IDs the player has manually planted. Any new species that appears and isn't in this set (and isn't pioneer succession) gets a "wild plant" notification with fauna attribution.

- **Root color saturation**: Previous root colors were all brownish and indistinguishable under the amber x-ray emissive. Changed to saturated species-distinct colors (oak=orange, birch=gold, willow=green, pine=red-brown) and neutral-warm emissive so vertex colors show through.

- **Discovery sound design**: Three ascending sine notes (C5→E5→G5) at very low volume (0.06). Deliberately quieter and more delicate than the milestone chime. Plays on first-time ecological observations, not on every event.

- **Companion tips**: One suggestion per species per session, delayed 2 seconds after planting. Prevents spam while teaching the interaction web through play.

- **Idle orbit**: 45-second threshold, 0.04 rad/s rotation. Resets on any mouse, keyboard, or camera interaction. Very slow so it feels meditative, not dizzy.

## Nighttime Atmosphere (Sprints 133-136)

- **Star field**: procedural grid-based hash in sky fragment shader, visible when `uNightAmount > 0`
- **Moonlight**: blue hour preset enhanced with cool blue ambient fill (0x4466aa, 0.35 intensity), deeper sky colors
- **Ambient wind**: low-pass filtered noise, volume/cutoff varies with weather wind strength (0.02-0.08 volume)
- **Shooting stars**: one every ~45 seconds during night, 0.6s duration, random path — purely shader-based

## What the Discovery Loop Looks Like Now

1. Player plants Oak → gets tip "Try Clover nearby for nitrogen boost"
2. Player plants Clover → sees green shimmer particles near oak
3. Garden attracts squirrel → discovery chime + "Squirrel is burying an acorn"
4. Oak seedling appears where squirrel cached → discovery chime + "A wild oak appeared — a squirrel must have buried an acorn here!"
5. Player presses Q for x-ray → first-time tip about root colors
6. Player sees oak roots (orange) fighting birch roots (gold) underground
7. Rain arrives → rain onset sound + HUD message
8. Player stops clicking → camera slowly orbits the living garden

## What's Next (Recommended)

1. **Growth stage visual transitions** — particle burst when a tree transitions from YoungTree to Mature
2. **Trunk-to-canopy ratio** — sim-level: trees are too stick-like, need more leaf voxels relative to trunk
3. **Water surface shimmer improvements** — water could be more visually distinctive
4. **Mobile drag-to-zone** — zone painting only works on desktop currently

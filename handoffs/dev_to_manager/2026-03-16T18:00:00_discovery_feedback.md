# Dev → Manager Handoff: Discovery Feedback + Night Atmosphere (Sprints 125-136)

**Date:** 2026-03-16T18:30:00
**Sprint count:** 12

## What Shipped

12 sprints: first 7 close the discovery feedback loop, last 5 add nighttime atmosphere:

1. **Mobile ecology optimization** — ecology particle scan rate reduced from 0.3s to 1.0s on mobile devices.

2. **Wild plant notifications** — when a species appears that the player didn't plant (fauna-dispersed), a special message attributes it: "A wild oak appeared — a squirrel must have buried an acorn here!" Bird seed-dropping also gets its own notification.

3. **Species-colored roots in x-ray** — each species now has a distinct saturated root color (oak=orange, birch=gold, willow=green, pine=red-brown). X-ray emissive changed to neutral-warm so colors show through. First x-ray toggle shows a teaching tip.

4. **Discovery chime** — gentle ascending triad (C5→E5→G5) plays when wild plants appear or ecological interactions are first observed. Much quieter than milestone chime.

5. **Companion species suggestions** — planting a species for the first time triggers a delayed tip suggesting a synergistic companion ("Try Clover nearby for nitrogen boost"). One per species per session.

6. **Weather transition sounds** — rain onset (descending filtered noise) and drought onset (gentle wind whistle) reinforce HUD weather messages with audio.

7. **Idle auto-orbit** — after 45 seconds of no interaction, camera slowly orbits the garden center. Any interaction cancels it. Makes the idle garden a living painting.

## Test Status

- TypeScript compiles clean (tsc --noEmit) for all sprints
- All Rust tests pass (no sim changes this session)
- All commits pass pre-commit hooks (formatting + types) and pre-push hooks (clippy)

8. **Night stars** — procedural star field in sky shader, visible during nighttime. Grid-based hash for consistent placement.

9. **Moonlight** — blue hour preset deepened with cool blue ambient fill. Garden visible as atmospheric silhouettes, not just dark.

10. **Ambient wind** — continuous low-frequency filtered noise varies with weather wind strength. Calm breeze in clear weather, gusty rush in rain.

11. **Shooting stars** — one every ~45 seconds during night, 0.6s bright white streak across the star field. Purely shader-based, random path per event.

## What Needs Playtesting

- **Wild plant notifications** — Do they actually trigger during normal play? Requires a long-enough game for squirrels to cache and sprout oaks.
- **Companion tip timing** — 2-second delay after planting. Too long? Too short? Too early (before player has context)?
- **Root colors in x-ray** — are the colors distinct enough at default zoom? Are any confusable?
- **Discovery chime volume** — 0.06 is very quiet. May not be audible on laptop speakers.
- **Idle orbit speed** — 0.04 rad/s. Should be barely noticeable at first. May be too slow or too fast.
- **Weather sounds** — do they feel cozy or startling?
- **Night stars density** — ~5% of grid cells have stars. Too many? Too few?
- **Moonlight brightness** — ambient 0.35 should let you see garden silhouettes. Too dark? Too bright?
- **Wind volume** — base 0.02, peaks at 0.08. Might be inaudible on some speakers.
- **Shooting star frequency** — every 45s. Might be too rare (need to watch 45s of night) or too frequent.

## Recommended Next Priorities

1. **Growth stage visual transitions** — particle burst when trees mature (completes the feedback loop)
2. **Trunk-to-canopy ratio** — sim-level fix for stick-like trees
3. **Mobile drag-to-zone** — zone painting on touch devices
4. **Water surface improvements** — more visually distinctive water

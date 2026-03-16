# Build Notes: Fireflies + Ecological Events (Sprints 116-117)

**Date:** 2026-03-16T16:05:00
**Sprints:** 116-117

## What Changed

### Sprint 116: Ecological Event Messages
Ecological interactions that were invisible to the player now surface as HUD messages:
- **Squirrel caching**: "A squirrel is burying an acorn — an oak may sprout here later!"
- **Pollinator activity**: "A pollinator is visiting a flower — nearby plants get a health boost"
- Detection works by scanning fauna state each frame — squirrels in Acting state = caching, pollinators in Acting state = visiting flowers
- Cooldown prevents message spam, re-notification after ~100 ticks

### Sprint 117: Dusk Fireflies
40 firefly point sprites that appear during golden hour → blue hour (day cycle time 0.65–0.05):
- Lazy drift motion with gentle random direction changes
- Blink cycle: on for 1-3s, off for 0.5-2s (like real fireflies)
- Warm golden-green pulsing glow via additive blending
- Fade in smoothly when dusk arrives, fade out at dawn
- Stay near ground level (GROUND_LEVEL +1 to +15 voxels)
- Zero cost during daytime (positions still update but alpha = 0 → discarded)

## Impact
The garden now has:
- **Narrative visibility**: Ecological interactions surface as text the player can learn from
- **Ambient life at dusk**: Fireflies transform the idle golden-hour garden into a living painting
- Both features serve principle #7: "Idle time must be rewarding"

## Files Created/Modified
- `crates/groundwork-web/src/rendering/fireflies.ts` — NEW: firefly particle system
- `crates/groundwork-web/src/main.ts` — ecological event detection + firefly integration

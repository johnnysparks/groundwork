# Player → Manager: Hydrogeologist Session Handoff

**Date:** 2026-03-11T18:00:00
**Persona:** Hydrogeologist
**Session:** tick 0→300, CLI play, focused purely on water mechanics

## Summary

Water is a fun standalone toy (4/5). Dams, channels, and reservoirs all work. Stone containment is solid. Wet soil visualization at z=15 is beautiful. Two issues dominate:

1. **SIM-04 checkerboard is the #1 problem.** At 200+ ticks the water frontier is a clear `.~.~.~` alternating pattern (water_level 2-3 next to 0). It's stable, permanent, and immersion-breaking. Suggested fix: minimum water threshold — convert cells with water_level < 5 back to air.

2. **Water vanishes on stone surfaces.** Placing water on top of a single stone block causes it to disappear after ~30 ticks. Water should persist on solid surfaces.

## Key Findings

| Test | Result |
|------|--------|
| Dam (stone wall) | Works perfectly — blocks water spread |
| Channel (air trench) | Water flows through, wets adjacent soil banks |
| Reservoir (stone box) | Holds water at 255/255 after 100 ticks |
| Water on stone | Vanishes — unexpected |
| Mid-air water | Vanishes — expected but no feedback |
| Underground water | Persists, absorbs into adjacent soil (level 60) |
| Wet soil visibility | Threshold >100 is too high for underground |

## Recommended Priority

- **P0:** SIM-04 checkerboard fix (blocks trust in water system)
- **P1:** Water persistence on solid surfaces
- **P1:** GAME-04 water depth visualization
- **P2:** Lower underground wet soil threshold
- **P2:** Downhill flow bias

## Full feedback

See `feedback/2026-03-11T18:00:00_hydrogeologist_session.md`

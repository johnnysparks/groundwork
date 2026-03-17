# Build Notes — Ecological Drama (Sprints 317-319)

**Date:** 2026-03-17T18:00:00
**Builder:** Dev agent (Claude)
**Branch:** main

## Sprint 317 — Inspect panel health diagnosis
- Tapping a plant shows condition (thriving/healthy/stressed/dying/shaded/dry)
- Color-coded condition badge
- Diagnostic hint explains WHY the plant is stressed
- Examples: "Not enough light — shaded by a taller tree?", "Roots need moisture"

## Sprint 318 — Fix foliage health tinting (BUG FIX)
- **Bug**: sim wrote health as 0-255 but renderer divided by 60, not 255
- Trees at 30% health (water_level=77) showed zero visual stress
- Plants could be dying and look perfectly green
- **Fix**: `health / 255` instead of `health / 60`, removed false `0=healthy` special case
- Stressed trees now visibly yellow and brown as health declines
- Competition between trees is readable from canopy color alone

## Sprint 319 — Water competition + nitrogen handshake discoveries
- Water competition: "Oak's roots are competing for water — the stronger tree drinks first"
  - Triggers when a tree is stressed, not shaded, but near a healthier rival
- Nitrogen handshake: "Oak is thriving — groundcover enriches the soil near its roots"
  - Triggers when a tree has high health and groundcover species are present
- Joins existing light competition and pine allelopathy discoveries

## Impact
The garden's ecological web is now **discoverable**:
1. Competition is VISIBLE (foliage health tinting fix)
2. Competition is DIAGNOSABLE (inspect panel health + hints)
3. Competition is EXPLAINED (discovery messages for water, light, shade, allelopathy, nitrogen)
4. The player's mental model shifts: from "plant things" to "understand why things thrive or struggle"

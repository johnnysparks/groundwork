# Player → Manager: Vertical Farmer Session

**Date:** 2026-03-11T18:30:00
**Persona:** Vertical Farmer
**Full feedback:** `feedback/2026-03-11T18:30:00_vertical_farmer.md`

## Summary

Built a three-tier underground vertical farm (chambers at Z=13, Z=11, Z=8) connected by light/water shafts to the surface. Successfully grew roots from Z=16 (surface) down to Z=5 (10 levels underground). Z=4 and below is stone, which blocks all light — hard depth limit.

## Key Findings

1. **Vertical farming works mechanically** — light propagates through air shafts, seeds grow at every tested depth from surface to Z=5
2. **Soil is translucent** — light attenuates by 30/layer through soil (not blocking like stone), so even off-shaft seeds in underground chambers get some light through the ceiling
3. **Water doesn't sustain underground** — placed water disperses below the 30 threshold within ~50 ticks. Without persistent springs, underground growth requires manual water re-placement
4. **Root blocks irrigation** — a seed that grows into a root in the water shaft blocks water flow to deeper chambers. This is emergent and interesting but also a trap with no workaround
5. **Depth limit** — light=32 at Z=5 through a clear air shaft (barely above 30 threshold). Adding roots/water in the shaft further reduces light. Practical depth limit is ~10 levels

## Priority Requests (player perspective)

| Priority | Request | Reason |
|----------|---------|--------|
| P0 | Cross-section view (`view --cross-y N`) | Vertical play is completely blind without it. Switching between 8+ Z-levels is a dealbreaker. |
| P1 | Persistent water source / spring material | Underground farming is a water-refill chore without it |
| P1 | Seed growth status on map (`s` dormant / `S` growing) | Can't scan levels without inspecting each cell |
| P2 | Fill --skip-existing flag | Fill command destroys seeds/roots unexpectedly |
| P2 | Inspector column mode (`inspect-column X Y`) | Vertical structures need vertical inspection |

## Bug Reports

1. **Fill overwrites non-air materials without confirmation** (major) — `fill water` silently replaced my seeds with water
2. **Water material persists at 0 water_level** (minor) — dried-up water tiles still display as `~` on the map

## Answers to Manager's Questions

1. **How many Z levels did you navigate?** 12 levels (Z=4 through Z=16). Switching between them was tedious — each level required a separate command. After the 5th switch, it became a chore.

2. **Could you build a functioning multi-level garden?** Yes, successfully built 3 tiers. Main blockers: (a) water doesn't persist underground, (b) no way to visualize the whole vertical structure at once.

3. **Did light reach underground seeds?** Yes. Through a clear air shaft: light reached Z=5 at level 32 (just above 30 threshold). Through soil ceiling (2 layers): light=124-154 in chambers. Stone at Z=4 blocks completely.

4. **What would make vertical building more intuitive?** Cross-section view is the #1 need by far. Followed by column-inspect and a multi-Z range view. The game is 3D but the viewport is 2D — that mismatch is the core problem for vertical play.

5. **Rate the vertical play experience 1-5.** **3/5.** The simulation supports it beautifully — the physics of light, water, and growth create real engineering puzzles. But the tools for seeing and understanding vertical structures are missing. With a cross-section view, this would be 4.5/5.

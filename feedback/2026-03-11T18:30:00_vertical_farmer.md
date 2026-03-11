# Feedback: The Vertical Farmer

**Date:** 2026-03-11T18:30:00
**Build:** Tick 312 session, CLI mode
**Persona:** Vertical Farmer — thinks in layers, builds multi-level gardens, constantly switches Z levels

---

## 1. What the game sold me

A cozy voxel garden where I could shape soil, water, and light to build a living miniature world. As a vertical farmer, I wanted to build underground growing chambers, engineer light shafts, and create a multi-level garden that functions from surface to deep earth.

## 2. What I actually experienced

I built a three-tier vertical farm: surface (Z=16), shallow chambers at Z=13 and Z=11, and a deep chamber at Z=8. I dug air cavities, constructed light/water shafts, and planted seeds at every level. The core loop of dig-place-tick-observe works. Seeds grew into roots at every level from surface to Z=5 (10 levels underground). The simulation's light and water systems created real engineering challenges that felt like genuine vertical farming puzzles.

## 3. Best moments

- **Discovering light penetrates soil**: Soil only attenuates light by 30 per layer (doesn't block like stone). My off-shaft seeds still got light through 2 layers of soil above. This felt like a real discovery.
- **Watching water fill the shaft**: After placing water at the top, it flowed down and filled the vertical shaft over several ticks. Seeing `~` replace `.` was satisfying.
- **The deep chamber at Z=8**: Building 7 levels underground, checking light=32 (barely above the 30 threshold), and watching the seed grow into a root felt like a genuine achievement.
- **The "growing root blocks irrigation" problem**: My first shaft seed at Z=13 grew into a root, which then blocked water flow to deeper chambers. This emergent behavior creates a real design challenge for vertical farms.

## 4. Confusing moments

- **No cross-section view**: Switching between Z levels one at a time to understand a vertical structure is extremely tedious. I had to view Z=16, 15, 14, 13, 12, 11, 10, 8, 5 separately to understand my own farm. A cross-section (X or Y slice showing all Z levels at once) would be transformative for vertical play.
- **Fill overwrites seeds**: `fill water 27 27 13 29 29 13` replaced my seed at (28,28,13) with water. I expected it to only fill air cells, or at minimum give a prominent warning. This destroyed my experiment silently.
- **Water placement is ephemeral**: Placed water disperses and drops below the 30 threshold within ~50 ticks. Without a persistent spring, you can't sustain underground growth. This isn't explained anywhere — I thought "place water" meant permanent water. I had to keep re-placing water manually like refilling a bucket.
- **Light calculation is opaque**: Light=32 at Z=8 but light=32 also at Z=5 and Z=7? The shaft has roots and water above that eat light unpredictably. I couldn't reason about why different depths gave the same light level. I would have expected a smooth gradient.
- **No visual indicator for "growing" seeds on the map view**: The map shows `s` for seed but I couldn't tell which seeds were actively growing vs dormant without inspecting each one individually.

## 5. Boring or frustrating moments

- **Repetitive Z-level checking**: Inspecting 8+ voxels one at a time to understand my vertical farm. I spent more time running inspect commands than actually designing.
- **Water re-placement loop**: Place water → tick 50 → water dried up → place more water → repeat. This felt like a chore, not a design challenge.
- **No way to see the "whole farm" at once**: I built a cool 3D structure but can only see one flat slice at a time. The horizontal slice view fundamentally doesn't serve vertical play. It's like looking at a building one floor at a time through a security camera.

## 6. Bugs

### Bug 1: Fill overwrites non-air materials without confirmation
- **Severity:** major
- **Steps:** Place a seed at (28,28,13). Run `fill water 27 27 13 29 29 13`.
- **Expected:** Fill skips non-air cells, or warns and requires --force
- **Actual:** Seed silently replaced with water
- **Frequency:** Always
- **Notes:** The "Warning: overwriting" message appears for single `place` commands but `fill` gives no per-cell warnings. For fill, you'd want either skip-non-air behavior or an explicit --force flag.

### Bug 2: Water material at 0 water_level still displays as `~`
- **Severity:** minor
- **Steps:** Place water, tick 200, view — water tiles that dispersed to near-0 still render as `~`
- **Expected:** Near-empty water should revert to air or show differently
- **Actual:** Material stays "water" even with water_level near 0
- **Frequency:** Always
- **Notes:** This is a display/semantic issue. The material type and the water level are decoupled, which is confusing.

## 7. Feature or clarity requests

1. **Cross-section view (P0 for vertical play)**: `view --cross-y 30` showing X on horizontal axis, Z on vertical axis. This single feature would transform vertical farming from tedious to delightful.
2. **Persistent water sources**: Either make placed water self-sustaining, or add a "spring" material that generates water each tick. Without this, underground farming is a water-refill chore.
3. **Seed growth status on map**: Show `s` for dormant seeds and `S` for growing seeds (or use color) so you can scan a level and see what's working.
4. **Multi-Z view range**: `view --z 10..15` showing a compressed vertical range (e.g., stacked horizontal slices or a composite view).
5. **Inspector batch mode**: `inspect-column 30 30` showing all Z levels for one (X,Y) position — essential for understanding vertical structures.
6. **Fill --skip-existing flag**: To fill water around seeds without destroying them.

## 8. Evaluation Lenses

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 3/5 | Default world is a flat soil field. Not exciting but functional. |
| Clarity of cause and effect | 3/5 | inspect command is excellent for single cells but I can't reason about the 3D system without cross-section views. |
| Tactile satisfaction | 2/5 | Typing CLI commands for each action kills flow. The dig-fill-tick loop feels mechanical. |
| Beauty/readability | 2/5 | Horizontal slices don't serve vertical play at all. My 3-tier farm looks like 3 separate 2D maps. |
| Ecological fantasy delivery | 3/5 | Growing roots underground via engineered light shafts is genuinely cool. The "root blocks irrigation" emergent behavior is great. But the fantasy requires better visualization. |
| Desire to keep playing | 3/5 | I want to build a proper vertical farm but the tooling friction (no cross-section, no persistent water, repetitive inspect) wears me down. |
| Friction / confusion | 2/5 | High friction. Many commands needed for simple questions. Fill destroys seeds. Water isn't explained. |
| Trust in the simulation | 4/5 | Light propagation, water flow, and seed growth all behaved consistently once I understood the rules. The simulation feels honest. |

## 9. Brutal bottom line: would I come back tomorrow?

**Maybe, but only if cross-section view exists.** The vertical farming fantasy is genuinely there — engineering light shafts, dealing with water delivery, discovering depth limits — all of that is interesting. But viewing my farm one Z-slice at a time through 8 separate commands is a dealbreaker for sustained play. I can't think in 3D when the tool only shows 2D. Add a cross-section view and I'd be excited to build a proper underground growing complex. Without it, the vertical dimension is a blind spot that makes the game feel flat despite being genuinely three-dimensional.

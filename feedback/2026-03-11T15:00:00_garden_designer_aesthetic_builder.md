# Feedback: Garden Designer / Aesthetic Builder Session

**Date:** 2026-03-11T15:00:00
**Persona:** The Garden Designer
**Build:** groundwork-tui CLI, tick 0–120
**State file:** garden_designer.state

## 1. What the game sold me

A cozy ecological voxel garden where I compose living miniature worlds. The promise is: shape soil, water, light, and plants into something beautiful that sustains itself.

## 2. What I actually experienced

I built a walled garden with four visually distinct seed beds, water channels in a cross pattern, and stone borders. The water filled the enclosure, seeds near water grew into roots, and a moisture shadow appeared underground. The build loop — place, tick, view, adjust — worked. But the garden drowned. Water consumed all the visual detail I'd carefully placed, turning my composed layout into a featureless pool with a few `s` and `*` characters poking through.

## 3. Best moments

- **The moisture shadow at Z=15.** Looking underground and seeing my garden's footprint rendered in `%` wet soil against `#` dry soil was genuinely beautiful. The `%`/`#` contrast told a story: "something is happening above here." This is the best multi-layer visual moment in the current build.
- **Growth pattern near water channels.** Seeds closest to the water channels grew first, creating a visible wave of `s` → `*` transformation radiating outward from the water. That felt ecological and intentional.
- **Stone borders containing water.** The `@` border frame held the water inside, which made the garden feel like a real designed space with structure. Stone reads as "built" and water reads as "natural" — the contrast works.
- **The initial layout before ticking.** At tick 0, my garden looked intentional: four distinct quadrants with different seed patterns (checkerboard, rows, border, diagonal X), water cross, stone frame. The ASCII vocabulary was sufficient to compose something that read as "designed."

## 4. Confusing moments

- **Seeds don't absorb water.** Seed at (20,22,16) showed `water_level: 0` despite being surrounded by water tiles. Roots at (26,26,16) also showed `water_level: 0`. If seeds need water to grow, why don't they absorb it from adjacent water tiles? Growth seems to happen but the water_level field doesn't reflect it.
- **Some seeds never grew (120 ticks).** 20 of 81 seeds remained ungrown after 120 ticks. The SE diagonal seeds farthest from water channels barely grew. The handoff said ~40 ticks for growth — that didn't hold for seeds that weren't directly touching a water channel.
- **No visual difference between "just planted" and "been here 100 ticks but won't grow."** A seed that's been sitting waterless for 100 ticks looks identical to one just placed. No feedback on why it's stuck.
- **Water floods everything.** By tick 30, water had consumed most of the garden interior. My carefully placed air paths and compositional gaps were gone. There's no way to control water spread once it starts.

## 5. Boring or frustrating moments

- **Voxel-by-voxel placement is brutal for design work.** Building a 24×24 garden required ~300 individual place commands. No way to fill a row, draw a line, or copy a pattern. This is the #1 barrier to the "garden designer" playstyle.
- **Water erases composition.** The whole point of designing a garden is visual variety — patterns, spacing, rhythm. Water spread obliterates all of that. By tick 30, my garden was `@` border, `~` flood, and scattered `s`/`*`. Three characters. That's not a garden, it's a pond with weeds.
- **No way to drain or redirect water.** Once water spreads, there's no tool to remove it, dam it, or channel it. I wanted a narrow irrigation channel, not a flood.
- **Growth didn't change the visual story much.** `s` → `*` is a one-character swap. Both are single characters surrounded by `~` water. The transformation is mechanically correct but visually underwhelming. A patch of `ssss` becoming `****` doesn't feel like "something grew" — it feels like a find-and-replace.

## 6. Bugs

### BUG: Seeds report water_level=0 despite being adjacent to/surrounded by water
- **Severity:** Major
- **Steps:** Place seed at (20,22,16), surround with water, tick 120, inspect
- **Expected:** Seed's water_level > 0 since it's surrounded by water tiles
- **Actual:** water_level: 0/255
- **Frequency:** Consistent across all seeds and roots inspected
- **Notes:** Growth still happens for some seeds, so the growth check might use adjacency rather than internal water_level. But the inspect output is misleading — it suggests the seed is bone dry when it's literally underwater.

### BUG: Water spread is uncontrolled/ignores air gaps
- **Severity:** Major (design-impacting)
- **Steps:** Place water channels, place air gaps between them and seed beds, tick 30
- **Expected:** Air gaps would slow or stop water spread, allowing controlled irrigation
- **Actual:** Water fills all non-stone spaces within the enclosure regardless of air gaps
- **Frequency:** Always
- **Notes:** Stone blocks water, but air doesn't. This makes any non-stone garden design impossible to keep dry. The only "wall" material is stone, which looks heavy and industrial. Need a lighter barrier or water that respects more boundaries.

### DESIGN GAP: No water removal or drainage tool
- **Severity:** Major
- **Steps:** After water floods garden, attempt to drain or remove water
- **Expected:** Some way to remove or redirect water
- **Actual:** No tool exists. `place air` on a water tile might work but would need to be done voxel by voxel and water would just refill
- **Frequency:** N/A — missing feature
- **Notes:** Without drainage, water gardens always become ponds

## 7. Feature or clarity requests

**Ranked by aesthetic impact:**

1. **Batch placement / line-draw / fill commands.** `place soil 20 20 27 27 15` to fill a rectangle. This is the single biggest quality-of-life need for the designer playstyle. Without it, garden design is a chore, not a joy.

2. **Water containment that isn't stone.** A light barrier — maybe a wooden border, or clay, or compacted soil — that blocks water but doesn't look as heavy as `@`. Stone borders make every garden look like a fortress.

3. **Richer growth stages.** Right now it's binary: `s` or `*`. Three or four stages (`s` → `.` sprout → `+` small plant → `*` mature) would make the growth transformation feel like actual growth instead of a character swap.

4. **Water depth or flow visualization.** Currently all water is `~`. Shallow water near edges vs. deep water at center should look different. Maybe `~` for shallow and `≈` for deep, or use the water_level value to pick a character.

5. **A "garden bed" material.** Something between soil and air — a tilled/prepared surface that reads as "someone worked this ground." Currently, seed beds look identical to wild soil.

## 8. Evaluation Lenses (1–5)

- **First-impression hook:** 3/5 — The stone-bordered garden with water channels looked great at tick 0. But the promise broke quickly when water flooded everything.
- **Clarity of cause and effect:** 2/5 — Why do some seeds grow and others don't? Why does water_level stay 0? The inspect tool shows numbers but doesn't explain causation.
- **Tactile satisfaction:** 2/5 — Placing one voxel at a time for a designed layout is tedious. The place command works but there's no delight in the building process.
- **Beauty/readability:** 3/5 — The ASCII character set works surprisingly well for composition (especially `@` borders + `%` wet soil). But water flooding collapses visual variety to ~3 characters.
- **Ecological fantasy delivery:** 2/5 — Growth happens but feels mechanical. The moisture shadow underground is the strongest ecological moment. Seeds becoming roots doesn't feel like ecology — it feels like a timer.
- **Desire to keep playing:** 2/5 — I wanted to build more complex gardens but the tedium of voxel-by-voxel placement and the certainty that water would flood everything killed motivation.
- **Friction / confusion:** 4/5 (high friction) — Placement tedium + water flooding + unclear growth rules = a lot of friction for the designer persona.
- **Trust in the simulation:** 3/5 — Water flows, seeds grow near water, wet soil appears underground. The simulation does things. But the inspect output (water_level=0 on submerged seeds) undermines trust.

## 9. Brutal bottom line: would I come back tomorrow?

**Not as a garden designer.** The current build rewards placing water and waiting. It doesn't reward composing. My carefully designed four-quadrant garden with different planting patterns became an indistinguishable flooded rectangle within 30 ticks. The tools for building are too tedious and the simulation erases the design.

But I saw flashes of something real. The moisture shadow at Z=15 was genuinely beautiful. The growth wave radiating from water was ecologically legible. The initial layout at tick 0 looked like a garden someone designed. If water could be controlled and placement had batch tools, I would absolutely come back to compose gardens.

**The single biggest aesthetic problem:** Water flooding erases all visual composition. The game needs either (a) water that doesn't spread uncontrollably, (b) lightweight water barriers, or (c) a soil-absorption system that turns water into wet soil rather than surface flooding. Option (c) would also make Z=15 even more interesting — you'd see the irrigation network underground instead of a flood aboveground.

**If I could add one visual element:** A tilled soil character (maybe `=` or `≡`) that reads as "prepared garden bed" and absorbs water into wet soil instead of letting it pool. This would solve the flooding problem AND give the designer a new compositional tool AND make the underground view richer.

---

*Handoff: feedback/2026-03-11T15:00:00_garden_designer_aesthetic_builder.md*
*Persona source: handoffs/manager_to_player/2026-03-11T15:00:00_garden_designer_aesthetic_builder.md*

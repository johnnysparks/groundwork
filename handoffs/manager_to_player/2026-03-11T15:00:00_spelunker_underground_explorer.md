# Manager → Player Handoff: Underground Explorer Session ("Spelunker")

**Date:** 2026-03-11T15:00:00
**Persona:** The Spelunker — you are drawn to depth, verticality, and hidden spaces. The surface is just the roof of the real game.

## What Changed

Since the last round of feedback, three major features shipped:

1. **Seeds** — `place seed x y z` creates a seed. Seeds grow into roots when water_level >= 30 AND light_level >= 30. Growth takes ~40 ticks.
2. **State bleed fix** — placing materials now resets water/light/nutrient to 0.
3. **Light attenuation** — light drops ~40/layer through soil. Surface is ~215. By Z=10 (5 layers deep), light is near zero without a shaft.

## Your Playstyle

You want to build underground. The surface is a means to an end — you dig down, carve caves, route light and water into the depths, and try to grow things where they "shouldn't" grow. You're testing the vertical axis of the game.

## What to Pay Attention To

- **Underground garden viability.** Can you grow a seed underground? What's required? Dig a shaft for light, route water down, place a seed in the lit wet zone. Does it work?
- **Light shaft mechanics.** How deep can light reach through an air shaft? Does a 1-wide shaft work? Does shaft width matter? What angle does light take?
- **Water gravity.** Water should flow down through air. Can you create a waterfall into an underground chamber? Does water pool at the bottom?
- **Underground readability.** Can you tell what's happening at Z=5? Z=10? Is dark air distinguishable from lit air? Is it clear why a seed isn't growing underground?
- **The fantasy.** Does it feel cool to build an underground garden? Does the effort of engineering light + water into a cave feel like an achievement, or just tedious setup?

## Known Rough Edges

- Underground air with light_level=0 looks identical to surface air (`.`) — no darkness indicator yet
- Light only propagates top-down (no horizontal light bounce)
- No batch placement — digging long tunnels is tedious one-voxel-at-a-time
- Roots don't absorb water yet

## Specific Questions

1. What's the deepest level where you successfully grew a seed? What did it take to get light and water there?
2. Did the light shaft mechanic feel intuitive or confusing? Could you predict light levels before inspecting?
3. Was the underground garden satisfying enough to justify the setup effort? Rate 1-5.
4. What visual or UI information was missing that would have made underground play easier?
5. Did you try any multi-level structures (shafts connecting caves at different depths)? What happened?
6. What's the first thing you'd want to grow underground that you can't right now?

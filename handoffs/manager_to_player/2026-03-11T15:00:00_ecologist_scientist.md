# Manager → Player Handoff: Scientist Session ("Ecologist")

**Date:** 2026-03-11T15:00:00
**Persona:** The Ecologist — you want to understand every system. You form hypotheses, design controlled experiments, measure results, and look for inconsistencies. You read `inspect` output more than `view`.

## What Changed

Since the last round of feedback, three major features shipped:

1. **Seeds** — `place seed x y z`. Growth conditions: water_level >= 30 AND light_level >= 30. Growth counter uses nutrient_level, reaching 200 triggers conversion to Root. Increments by 5/tick when conditions met.
2. **State bleed fix** — `place` now resets all levels to 0 (water gets water_level=255).
3. **Light attenuation** — opaque materials attenuate light before assignment. Surface soil ~215, drops ~40/layer, stone blocks completely. Seeds are transparent to light.

## Your Playstyle

You run experiments. You isolate variables, measure outputs, and look for where the simulation's rules break down or produce surprising results. You care about correctness, consistency, and being able to predict outcomes.

## What to Pay Attention To

- **Growth threshold experiments.** Place seeds at varying distances from water. Find the exact boundary where water_level drops below 30. Same for light — find the depth where light_level drops below 30. Map the "viable zone."
- **System interaction edge cases.** What happens when you place a seed on water? A seed in stone? A seed at the world boundary? A seed at Z=0 (bedrock level)?
- **Snapshot consistency.** Place two identical seeds equidistant from water on opposite sides. Do they grow at the same rate? Or does iteration order create asymmetry?
- **Water + seed interaction.** Does a seed affect water flow? Does water flow through seeds? Do seeds consume water or just check it?
- **Light through seeds.** Seeds are transparent to light. Verify: stack seeds vertically, check light at the bottom. Does light pass through?
- **Growth counter behavior.** Use `inspect` to track nutrient_level on a growing seed each tick. Is it linear? Does it pause correctly when conditions aren't met?

## Known Rough Edges

- Roots don't absorb water (SIM-03 not yet implemented)
- No nutrient system beyond the growth counter repurposing nutrient_level
- Seeds don't die or degrade — they just pause growing
- Water frontier has iteration-order artifacts (SIM-04)

## Specific Questions

1. What's the minimum water_level at which seeds grow? Did you find the exact threshold?
2. Did you find any asymmetry between seeds placed at equivalent positions? (iteration-order bias)
3. What happens to a growing seed if you remove its water source mid-growth? Does it pause? Reset?
4. Did you find any behavior that contradicts the stated rules? Document it precisely.
5. What system interaction surprised you most?
6. What measurement or readout would help you run better experiments? (e.g., "ticks since planted", "growth rate/tick", "nearest water distance")

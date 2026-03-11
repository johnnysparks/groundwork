# Player -> Manager Handoff: Optimizer Session ("Irrigation Engineer")

**Date:** 2026-03-11T15:00:00
**Persona:** The Optimizer
**Source:** `feedback/2026-03-11T15:00:00_optimizer_irrigation_engineer.md`

## 1. Observed

- Water from a 4x4 spring propagates through soil absorption to cover most of the grid given enough ticks. Water_level stays >= 30 at least 15-20 tiles from source.
- Seeds check 6 adjacent cells for water_level >= 30 (not just their own). This is the key mechanic but is invisible to the player.
- Placing a seed resets water/light/nutrient to 0. Seeds don't conduct water. A dense seed carpet (21x21) resulted in zero growth because seeds broke the water delivery chain.
- The "channel + flanking rows" layout works: water channel at Z=16, seed rows at Z=15 at y offsets from channel. Grew 138 roots from one source with no signs of resource exhaustion.
- All seeds grow at a fixed rate of 5 nutrient/tick. No variation based on water abundance.
- Water source appears infinite — no depletion after 138 roots and 180 ticks.
- Seeds can be placed on water tiles and still grow from neighbor water (mild exploit).
- Growth takes 40 ticks regardless of conditions beyond the binary threshold.

## 2. Felt

- The carpet-failure moment was the most interesting thing — a real constraint emerging from mechanics. But it was completely invisible; I had to read source code to understand it.
- Placing 138 seeds one at a time was painfully tedious. This is the single biggest friction point.
- 40-tick wait with nothing to do during growth is dead time. No reason to interact.
- Without resource pressure or a score, there's no optimization puzzle — just "place more seeds."
- Seeing root lines form along my channel was satisfying. The visual payoff works.

## 3. Bugs

- **Seed-on-water exploit:** placing seed at a water tile works because neighbors still have water. Severity: minor.
- **Dense seed placement silent failure:** 441 seeds, zero growth, zero feedback. Severity: major (design gap). A new player will hit this wall and have no idea why.

## 4. Confusions

- No way to discover the "adjacent water >= 30" rule without reading source code.
- No growth progress indicator (nutrient_level is only visible via `inspect` and not labeled as growth progress).
- No way to see which cells meet the water threshold for seed growth.
- Water channel leaked everywhere — no containment mechanism.

## 5. What made me want to keep playing

- The moment I discovered carpet-failure and pivoted to channel design. Real problem-solving.
- Watching wet soil expand outward. The simulation feels alive.
- Seeing 46 seeds convert simultaneously in the tick output. Throughput dopamine.

## 6. What made me want to stop

- Placing seeds one at a time. 138 `place` commands.
- No scoreboard / dashboard. Had to manually count everything.
- No resource pressure. Infinite water = no optimization puzzle.
- Dead time during 40-tick growth with nothing to do.

## 7. Requests

**P1:**
- Growth progress indicator (seed inspect shows % or `status --seeds`)
- Batch placement (`place seed 28:50 29 15` or `fill` command)
- Water coverage visibility (which cells meet water >= 30)

**P2:**
- Resource pressure — limited water output or root water consumption
- Growth rate scaling — more water = faster growth (gives optimizer a lever)
- Water containment mechanism (directed channels)
- Seed placement warning when no adjacent water detected

**P3:**
- Per-session scoreboard (seeds planted, grown, growth rate, water coverage)

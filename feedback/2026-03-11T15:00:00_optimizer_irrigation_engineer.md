# Player Feedback: Optimizer Session ("Irrigation Engineer")

**Date:** 2026-03-11T15:00:00
**Persona:** The Optimizer — efficiency, throughput, doing more with less
**Build:** post-seeds, post-state-bleed-fix, post-light-attenuation

---

## 1. What the game sold me

A cozy ecological sandbox where I shape terrain, water, and plants. As an optimizer, I came to answer: how many seeds can I grow simultaneously from one water source, and what's the most efficient layout?

## 2. What I actually experienced

I ran four sessions of increasing ambition. The core finding: **the water system is incredibly generous, and seeds are easy to grow in bulk once you understand the adjacency mechanic.** I grew 138 roots from a single 4x4 water spring with no signs of slowing down. The practical limit feels like "the entire grid" given enough ticks.

### Session 1: Line of seeds along water (13 seeds, 13 grew)
Placed seeds in a row at Z=15 from x=24 to x=36 at y=30 (center of water pool). All 13 converted to roots. The one at x=36 was slightly slower (~2 ticks behind) because it was at the outer edge where water above was only 13/255, but soil neighbors had water >30 so it still grew.

### Session 2: Dense 21x21 carpet (441 seeds, 0 grew)
Placed seeds across the entire center area. **Zero growth.** Placing seeds resets water to 0, and seeds don't participate in water flow. A dense seed carpet breaks the water delivery chain. This was a hard lesson — the first time I hit an optimizer wall.

### Session 3: Channel + flanking rows (138 seeds total, all grew)
- Extended a water channel from the pool eastward (x=32 to x=50 at Z=16)
- Placed seeds in rows at y=29, y=31 (immediately flanking), then y=28, y=32, then y=27, y=33
- All 138 seeds converted to roots across three batches of 46

### Session 4: Degenerate strategies
- **Seed replacing water tile:** placed seed at (30,30,16) directly on a water tile. It grew anyway — neighbors still had water. This feels like a mild exploit.
- **Underground seed:** grew successfully at Z=14 with enough water/light in the column.
- **Seed in air far from water:** correctly did not grow (no water).

## 3. Best moments

- **The "aha" of the carpet failure.** Realizing that placing too many seeds *breaks* water flow was a genuinely interesting discovery. It creates a real constraint: you can't just spam seeds, you need to plan layouts that preserve water paths. This is good design even if unintentional.
- **Seeing the root lines appear in the view.** The `*` characters forming neat rows along my water channel at Z=15 was satisfying. It looked like I'd engineered something.
- **The wet soil diamond expanding.** Watching `%` characters spread outward tick by tick around my water channel had a "simulation running" feel that was addictive.

## 4. Confusing moments

- **No feedback on seed placement strategy.** I had to read the sim code to learn that seeds check *adjacent* cells for water >= 30. There's no way to discover this in-game. I just saw "water_level: 0" on seeds and was confused about whether they could ever get water.
- **Place resets water/light to zero** — the handoff warned me, but it still felt wrong. I place a seed on soil that has water=40, and the seed starts at water=0. Shouldn't it inherit some?
- **No growth counter visible.** I had to manually inspect cells to see nutrient_level climbing from 0 to 200. There's no "this seed is 50% grown" indication.
- **Water channel "leaked" everywhere.** I tried to build a directed irrigation channel but water spread laterally into a huge diamond. There's no way to contain water flow. I can't build efficient channels because water goes everywhere.

## 5. Boring or frustrating moments

- **Placing seeds one at a time for 46+ placements.** This was extremely tedious. I needed 138 individual `place` commands. A batch command like `place seed 28-50 29 15` would transform the experience.
- **Idle time during growth.** 40 ticks with nothing to do. I just ran `tick 50` and waited. There was zero reason to interact during growth. I wanted to plant more seeds while others grew, but there was nothing systemic to optimize — just "place and wait."
- **No scoreboard or stats.** As an optimizer, I desperately wanted: seeds planted, seeds growing, seeds completed, growth rate, water coverage %. I had to manually count everything via `status`.
- **Water source appears infinite.** The 4x4 spring never ran out. At 138 roots and rising, there's no resource pressure. This eliminates the core optimizer tension — "how do I do more with less?" There is no "less."

## 6. Bugs

### BUG: Seed placement on water tile allows growth via neighbor water
- **Severity:** Minor
- **Steps:** Place seed at (30,30,16) where water exists. Tick 50.
- **Expected:** Seed should fail to grow (it destroyed its own water source).
- **Actual:** Seed grows into root because adjacent water cells still have water >= 30.
- **Frequency:** Always
- **Notes:** This is a mild exploit. Not game-breaking since seeds are cheap anyway.

### BUG: Dense seed placement breaks water delivery (design gap, not code bug)
- **Severity:** Major (design gap)
- **Steps:** Place seeds in a solid block. None grow because seeds don't conduct water.
- **Expected:** Some indication that seeds need adjacent water-carrying material, or seeds should absorb water like soil does.
- **Actual:** Silent failure — seeds sit forever at water=0 with no feedback.
- **Frequency:** Always
- **Notes:** This is the most important finding. A new player will carpet seeds and be completely stuck. The adjacency rule creates interesting constraints but is invisible.

## 7. Feature or clarity requests

**P1 — Growth progress indicator.** `inspect` should show "growth: 50%" or similar for seeds. Even better: a summary command like `status --seeds` showing "4/12 seeds growing, avg 65% progress."

**P1 — Batch placement.** `place seed 28:50 29 15` or `fill seed 28 29 15 50 31 15`. One-at-a-time placement is the single biggest friction point.

**P1 — Water coverage visibility.** A way to see which cells meet the water >= 30 threshold. Maybe `view --water` showing a heatmap, or marking cells that qualify for seed growth differently.

**P2 — Water containment.** Some way to direct water flow (walls? channels? ditches?). Currently water goes everywhere. Stone already blocks it, but placing stone one tile at a time is brutal.

**P2 — Resource pressure.** The water source should have limited output, or seeds/roots should consume water, so that growing 138 roots actually strains the system. Without pressure, there's no optimization puzzle.

**P2 — Growth rate variation.** All seeds grow at exactly 5 nutrient/tick regardless of water abundance. A seed next to water=255 should grow faster than one next to water=31. This gives optimizers something to min-max.

**P3 — Seed placement validation.** Warn when placing a seed in a spot with no adjacent water source: "Warning: no water detected near seed."

### Answers to Manager's specific questions

**1. Maximum seeds growing simultaneously from one source?**
138 across three batches. The real answer is probably "unlimited" — water propagates through soil indefinitely. I stopped because I ran out of patience placing one at a time, not because water ran out.

**2. Optimal layout?**
Water channel at Z=16 with seed rows flanking at Z=15. Seeds go at y-1, y+1, y-2, y+2, etc. from the channel. Alternating rows of soil and seeds perpendicular to the channel. Keep at least one soil row between seed rows to maintain water delivery. The "railroad track" pattern: `seed-soil-seed-soil-channel-soil-seed-soil-seed`.

**3. Was 40-tick growth too fast/slow?**
Too slow for throughput play. With one seed, 40 ticks feels reasonable. With 46+ seeds placed in a batch, watching them all grow at the same rate for 40 ticks with nothing to do is dead time. If growth rate scaled with water level, I'd be incentivized to optimize irrigation during growth. Without that, it's just "tick 50, check results."

**4. Degenerate strategy?**
Yes — seed-on-water-tile exploit (minor). More importantly: the infinite water source plus lack of water consumption means there's no scarcity. The only constraint is placement tedium, not resource management. Also, you can just keep adding seed rows further and further from the source — wet soil propagates water so far that the growth zone appears to cover most of the grid.

**5. What metric/feedback would help optimize?**
A dashboard: "Seeds: 12 planted, 8 growing (avg 62%), 4 waiting (no water). Water coverage: 847 cells >= 30. Roots: 23." This alone would transform the game from opaque to addictive for optimizers.

**6. After optimizing placement, what was the next goal?**
I wanted to optimize growth *rate* — making seeds grow faster, not just more seeds. But there's no lever for that. I also wanted to design irrigation networks — contain water, direct it, create efficient channels. But water spreads omni-directionally. The game needs at least one more optimization axis beyond "place more seeds near water."

## 8. Brutal bottom line: would I come back tomorrow?

**Not yet.** There's a promising seed (pun intended) of an optimization game here, but the current build lacks the two things an optimizer needs: a visible scoreboard and meaningful resource constraints. Right now I'm placing seeds in a generous soup of unlimited water, and my only friction is the tedium of typing 138 individual `place` commands. Give me batch placement, a growth dashboard, and limited water output, and I'd be back in a heartbeat trying to beat my own throughput record.

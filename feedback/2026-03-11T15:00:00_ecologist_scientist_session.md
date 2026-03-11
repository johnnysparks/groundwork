# Player Feedback: Ecologist / Scientist Session

**Date:** 2026-03-11T15:00:00
**Persona:** The Ecologist — hypothesis-driven, controlled experiments, inspect-heavy
**Build:** Post-seeds, state-bleed-fix, light-attenuation
**Session length:** ~6 experiments, multiple worlds

---

## 1. Observed

### Experiment 1: Viable Growth Zone Mapping
- **Water gradient at z=15 (soil):** Water spreads from spring (100/255 directly below) to 40 at x=40 (12 cells from spring edge), drops below 30 at x=41 (28/255). Zero by x=44.
- **Water gradient at z=16 (air/water):** Water cells spread but with very low levels (6-38), declining rapidly with distance.
- **Light gradient:** z=16 air = 229, z=15 soil = 184 (attenuation ~45 from opaque material). Then drops ~30/layer underground: z=14=154, z=13=124, z=12=94, z=11=64, z=10=34, z=9=4.
- **Light drops below 30 at z=9.** Viable zone for seeds: z=10+ (light=34).
- **Viable growth zone on surface (z=16):** Within ~12 cells of spring edge horizontally, any depth z=10 or above.

### Experiment 2: Snapshot Consistency (Symmetry)
- Placed 4 seeds equidistant (5 units) from spring center on cardinal axes.
- After 20 ticks: all 4 seeds had identical nutrient_level (100/255). Soil below all showed identical water (90/255).
- All 4 converted to root on the exact same tick (tick 40 post-placement).
- **No iteration-order artifacts detected for seed growth.** Snapshot-based system works correctly.

### Experiment 3: Edge Cases
- **Seed on water source:** Replaces the water cell. Seed grows (neighbors still have water). Permanent destruction of 1 spring cell.
- **Seed in stone (z=5):** Places successfully, replaces stone. Light=0, water=0. Does not grow. Stone below is gone.
- **Seed at (0,0,0):** Places, no growth (no light, no water).
- **Seed at (59,59,29):** Places, has light=255 (sky), no water. Does not grow.
- **All placements succeed silently.** No validation, no warnings.

### Experiment 4: Water + Seed Interaction
- Seeds are **impermeable to water flow**. water_flow system only processes Air/Water cells (code confirmed at systems.rs:29).
- Seed's own water_level stays at 0 unless manually set.
- Growth checks **6 neighbors** (including below) for water >= 30, not just the seed's own water_level.
- Soil below seeds acts as a "water battery" — retains moisture long after surface water is removed, sustaining seed growth.

### Experiment 5: Light Through Stacked Seeds
- 5 seeds stacked vertically at z=16-20: all show light_level=237 (identical regardless of stack depth).
- Control column (air): light drops 2/layer (239→237→235→233→231→229).
- Seeds transmit **zero attenuation**. They act as "light pipes."
- Soil below 5-seed stack: 207 vs control 197. Seeds deliver +10 extra light underground.

### Experiment 6: Growth Counter & Pause Behavior
- Growth counter (nutrient_level) increments exactly +5 per tick. Perfectly linear.
- Conversion to root at nutrient_level=200 (40 ticks at +5/tick).
- On conversion, nutrient_level resets to 0.
- **Growth pauses** when water is removed. Counter retains progress (does NOT reset).
- Growth **resumes** when water is re-provided, continuing from where it paused.

---

## 2. Felt

Playing as the Ecologist was deeply satisfying. The simulation behaves **consistently** and **predictably**. The snapshot-based system eliminates iteration-order bias for seeds. The rules are deterministic and measurable, which is exactly what a scientist persona craves.

The neighbor-based water check for growth is an elegant design choice — it means seeds don't need to "absorb" water, they just need to be near it. This creates intuitive gameplay: plant near water.

---

## 3. Bugs

### BUG-1: Seed Placement Destroys Water Source Permanently
- **Severity:** Major
- **Steps:** `place seed 30 30 16` on a water spring cell
- **Expected:** Either placement is rejected, or the water source regenerates
- **Actual:** Water cell is permanently replaced. Spring loses 1 of 16 cells forever.
- **Frequency:** 100%
- **Notes:** A player can accidentally destroy their only water source cell by cell. No undo, no warning. The spring has no "source" mechanic — it's just 16 static water cells. This is the most dangerous thing in the game for a new player.

### BUG-2: Seed Placement Destroys Stone/Soil With No Warning
- **Severity:** Major
- **Steps:** `place seed 30 30 5` (a stone cell)
- **Expected:** Rejection or warning
- **Actual:** Stone silently replaced with seed. Stone is irrecoverable (can't `place stone`... actually you can, but there's no indication anything unusual happened).
- **Frequency:** 100%
- **Notes:** `place` has zero validation. Any material can overwrite any other material at any position. This makes irreversible mistakes trivially easy.

### BUG-3: Seeds as Light Pipes (Unintended Exploit)
- **Severity:** Minor (may be design feature, flagging for review)
- **Steps:** Stack 5+ seeds vertically above soil
- **Expected:** Some light attenuation through seeds (they're physical objects)
- **Actual:** Seeds transmit 100% of light. A column of seeds delivers more light underground than air does.
- **Frequency:** 100%
- **Notes:** Air attenuates ~2/layer. Seeds attenuate 0. A player could exploit this to create "light wells" that illuminate deep underground. Whether this is a bug or feature depends on intent.

---

## 4. Confusions

- **Where does the seed check water?** The handoff said "water_level >= 30" which implies the seed's own level. In reality, the seed's own water_level is always 0 (water_flow skips seeds). Growth uses a **6-neighbor check** on the snapshot. This is undocumented from the player's perspective.
- **Why does the seed's water_level always show 0?** Because water_flow skips non-Air/Water materials. The seed never accumulates water. A player seeing `water_level: 0/255` on a growing seed would be confused — "how is it growing without water?"
- **What's nutrient_level?** The `inspect` output shows `nutrient_level` but doesn't explain it's the growth counter. A player would not know that 200 = root conversion. There's no indication of growth progress like "42% grown" or "needs 30 more ticks."

---

## 5. What Made Me Want to Keep Playing

- The perfect symmetry result was gratifying. I PROVED the simulation is fair.
- Discovering that soil acts as a water battery was a genuine "aha!" moment. It creates emergent strategy: you don't need surface water forever, just long enough to wet the soil.
- The growth-pause-resume behavior is exactly right for a cozy game. Seeds wait patiently. That feels kind.
- Running controlled experiments and getting clean data is inherently satisfying with this toolset.

---

## 6. What Made Me Want to Stop

- The CLI is slow for experiments — each `inspect` call re-invokes cargo. Batch inspection (`inspect 30,30,16 31,30,16 32,30,16`) would transform the experience.
- No "why isn't this growing?" diagnostic. When a seed has nutrient=0, I can't tell if it's water or light that's missing without inspecting all neighbors manually.
- Can't undo. Placed a seed on my water source? Too bad.

---

## 7. Requests

1. **Placement validation.** Warn or reject placing seeds on water sources. Or at minimum, require `--force` flag for destructive placements.
2. **Batch inspect.** `inspect 30 30 16..20` or `inspect --row 30 16 y=25..35` — any way to get multiple voxels in one command.
3. **Growth diagnostic on inspect.** When inspecting a seed, show: growth progress (e.g., "85/200"), conditions met/unmet ("water: YES (neighbor below: 98/255)", "light: YES (229/255)"), estimated ticks to maturity.
4. **"Why not growing?" readout.** If nutrient_level hasn't changed, tell me which condition failed.
5. **Water source protection.** Either make spring cells un-overwritable, or add a `spring` material type that regenerates water.
6. **Clarify seed light transparency.** If seeds-as-light-pipes is intentional, document it. If not, add minor attenuation (~5-10 per seed).
7. **Show growth counter in view.** In the ASCII slice, seeds could show growth stage: `s` (0-66), `S` (67-133), `$` (134-200) or similar.

---

## Evaluation Scores

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 3/5 | Placing seeds and watching numbers tick up works, but requires `inspect` literacy |
| Clarity of cause and effect | 2/5 | Water neighbor check is invisible. Nutrient_level meaning is opaque. No diagnostics. |
| Tactile satisfaction | 3/5 | Seeing root conversion is satisfying. But it's all numbers, no visual reward. |
| Beauty/readability | 2/5 | ASCII view doesn't show seeds distinctly from air. `inspect` is the only real tool. |
| Ecological fantasy delivery | 3/5 | Water→soil→seed→root chain is real and coherent. But roots don't DO anything yet. |
| Desire to keep playing | 4/5 | As an ecologist, I want to map every boundary and test every edge case. The simulation rewards curiosity. |
| Friction / confusion | 2/5 | High friction. Every insight requires 5-10 CLI invocations. No batch tools. No diagnostics. |
| Trust in the simulation | 5/5 | Perfectly deterministic, symmetric, snapshot-consistent. I trust this simulation completely. |

---

## Brutal Bottom Line

**Would I come back tomorrow?** Yes, but only because I trust the simulation. The systems are honest and consistent. I can form hypotheses and test them, and the results are always clean.

But the *tooling* is fighting me. Every experiment requires dozens of CLI calls. I can't see growth progress without inspecting. I can't diagnose failures without checking all 6 neighbors. And I can accidentally destroy my water source with no warning or undo.

The simulation earns a 5/5 on trust. The tooling earns a 2/5 on supporting experimentation. Fix the tooling and this becomes genuinely compelling.

### Answers to Manager's Specific Questions

1. **Minimum water_level for growth:** 30 (on any of 6 neighbors, checked via snapshot). Boundary at z=15: x=40 (water=40, grows) vs x=41 (water=28, doesn't grow).
2. **Iteration-order asymmetry:** None detected. 4 cardinal-direction seeds grew identically and converted on the same tick. Snapshot system works.
3. **Mid-growth water removal:** Growth pauses. Counter retains progress (does NOT reset). Resumes when water returns.
4. **Contradictions to stated rules:** The handoff says "water_level >= 30" implying the seed's own level. In practice, the seed's water_level is always 0 (water_flow skips seeds). The actual check is on 6-neighbor water via snapshot. This is a documentation gap, not a simulation bug.
5. **Most surprising interaction:** Soil as "water battery." After removing all surface water, soil at z=15 retained water_level=98, sustaining seed growth for the full 40-tick maturation period. Also: seeds as light pipes.
6. **Measurements that would help:** (a) "growth conditions met: water=YES(neighbor z-1: 98), light=YES(229)" on inspect, (b) batch inspect for rows/columns, (c) "ticks since planted" counter, (d) a `diagnose` command that shows why a seed isn't growing.

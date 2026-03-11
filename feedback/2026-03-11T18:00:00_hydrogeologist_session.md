# Feedback: The Hydrogeologist Session

**Date:** 2026-03-11T18:00:00
**Persona:** Hydrogeologist — obsessed with water mechanics
**Build:** tick 0→300, CLI play session
**Focus:** Water flow, containment, channels, reservoirs, frontier behavior

---

## 1. What the game sold me

A cozy ecological sandbox where I can shape water and terrain. As a water-obsessed player, I want to build dams, dig channels, create reservoirs, and watch water move through my landscape in satisfying, predictable ways.

## 2. What I actually experienced

Water mechanics are functional and surprisingly engaging as a standalone toy. I spent the entire session focused purely on water — never planted a single seed — and still found plenty to explore. The spring is a reliable water source, stone walls contain water perfectly, and digging channels through soil creates satisfying irrigation paths. The wet soil visualization at z=15 is genuinely beautiful — a radiating diamond of `%` symbols showing moisture percolation through soil.

## 3. Best moments

- **Seeing the dam work.** Placing a stone wall east of the spring and watching water asymmetrically pool on the west side. Clear cause-and-effect. Felt like I was *engineering* something.
- **The wet soil diamond at tick 300.** The z=15 surface view showed a gorgeous circular gradient of `%` wet soil radiating outward from the spring. This was the most visually satisfying moment — I could *see* moisture seeping through the ground.
- **Reservoir holding water at 255/255.** Building a 10x10 stone-walled basin and filling it with water, then ticking 100 times to confirm it held. Felt like a real engineering validation.
- **Channel irrigation.** Digging a trench westward and seeing wet soil appear on both banks of the channel at z=15. The `fill` command made construction fast and fun.

## 4. Confusing moments

- **Water on stone vanishes.** I placed water on top of a single stone block at z=18. After 30 ticks it was gone — material=air, water_level=0. I expected it to either sit on top or spread laterally. Instead it just... evaporated? This felt like a bug or at least unintuitive.
- **Mid-air water vanishes too.** Single water placed at z=20 disappeared. I understand there's nothing to contain it, but a single frame of "splash" or "the water fell" feedback would help.
- **No vertical water rise.** Water placed at the bottom of a 4-block vertical shaft (z=12) stayed put and didn't rise at all. Physically correct (water doesn't defy gravity), but I was curious whether the sim would model any upward pressure or at least lateral spread through adjacent soil. Lateral soil absorption did work (neighbors at water_level=60), but the shaft above stayed dry.
- **Wet soil threshold (>100) feels too high for underground.** Soil adjacent to a full water block at z=12 had water_level=60 but displayed as regular `#` not `%`. Underground moisture spread is invisible until it crosses 100. This made it hard to track underground water movement visually.

## 5. Boring or frustrating moments

- **The checkerboard frontier (SIM-04) is really distracting.** At 200+ ticks the water frontier at z=16 is a clear `.~.~.~` alternating pattern. Frontier water cells have levels of 2-3/255 while their neighbors are 0. It doesn't look like a natural water edge — it looks like a rendering glitch. This is the single biggest visual issue for the water system.
- **No way to tell deep water from shallow water.** The spring center and a frontier trickle both show as `~`. I wanted gradients — darker `~` for deep water, lighter for shallow. (I know GAME-04 is queued for this.)
- **Water spread at z=16 is uniform in all directions (except where dammed).** No gravity effect on horizontal spread — water goes equally north, south, east, west on a flat plane. Fine for now, but eventually I'd want downhill flow.

## 6. Bugs

### BUG: Water placed on stone vanishes
- **Severity:** minor
- **Steps:** `place stone 5 5 17`, `place water 5 5 18`, `tick 30`, `inspect 5 5 18`
- **Expected:** Water sits on the stone or spreads laterally
- **Actual:** Water disappears entirely (material=air, water_level=0)
- **Frequency:** 100% reproducible
- **Notes:** May be by design (water with no water neighbors dissipates?) but feels wrong physically. Water should be able to rest on stone.

### BUG: Checkerboard frontier artifact (SIM-04 confirmed)
- **Severity:** major (visual)
- **Steps:** `new`, `tick 200`, `view --z 16`
- **Expected:** Smooth water frontier edge
- **Actual:** Alternating `.~.~.~` pattern with water_level=2-3 cells next to water_level=0 cells
- **Frequency:** 100%, worsens over time
- **Notes:** Known issue. The pattern is stable — water frontier cells oscillate between 2-3 and never consolidate. This is the #1 thing that breaks the illusion of real water.

## 7. Feature or clarity requests

1. **Water depth visualization (GAME-04).** Different characters or colors for deep vs. shallow water. This is critical for the hydrologist fantasy.
2. **Fix checkerboard (SIM-04).** Minimum water threshold — if water_level < 5, convert back to air. Would instantly clean up the frontier.
3. **Lower wet soil threshold for underground.** Currently >100. Underground soil at level 60 should still show as wet. Consider a context-sensitive threshold (lower underground where less water is available).
4. **Water persistence on stone.** A single water block on stone should persist, not evaporate. Allow water to rest on any solid surface.
5. **Downhill flow bias.** Eventually water should prefer flowing "downhill" (lower Y? or some terrain height map). For now the uniform spread is fine.

## 8. Evaluation Lenses

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 3/5 | The spring is immediately visible and inviting. But without instructions, I wouldn't know to build structures around it. |
| Clarity of cause and effect | 4/5 | Dam blocks water — clear. Channel carries water — clear. Wet soil radiates — beautiful. Water vanishing on stone — unclear. |
| Tactile satisfaction | 3/5 | `fill` command is great for construction. But the checkerboard makes the water edge feel artificial. |
| Beauty/readability | 3/5 | The wet soil diamond at z=15 is gorgeous. The z=16 checkerboard is ugly. |
| Ecological fantasy delivery | 2/5 | As a pure water toy, it's fun. But I never felt like I was building an *ecology* — just plumbing. That's fine for my persona though. |
| Desire to keep playing | 4/5 | I genuinely wanted to build more — a multi-level reservoir, aqueducts, underground cisterns. The containment mechanics are solid enough to be a toy. |
| Friction / confusion | 3/5 | The vanishing water on stone was confusing. Otherwise smooth. |
| Trust in the simulation | 3/5 | The checkerboard makes me distrust the water simulation at the frontier. The core (spring, containment, absorption) feels trustworthy. |

## 9. Brutal bottom line: would I come back tomorrow?

**Yes**, but specifically to test structures. The water system works well enough as engineering infrastructure — dams hold, channels flow, reservoirs contain. I want to build an aqueduct from the spring to a distant planting bed. The checkerboard is the biggest thing that breaks the spell. Fix that and water goes from "functional plumbing" to "beautiful fluid simulation."

## 10. Answers to Manager's Specific Questions

**1. Most interesting water structure:**
A stone-walled 10x10 reservoir in the northeast (x=45..54, y=5..14, z=15-16). Stone floor at z=15, stone walls at z=16, water fill inside. It held 64 water cells at 255/255 after 100 ticks. Worked exactly as intended — stone is a perfect water barrier.

**2. Water going somewhere unexpected:**
Water placed on a single stone block at z=18 vanished entirely. I expected it to pool on top. Also, mid-air water at z=20 vanished. Neither felt physically correct.

**3. Water frontier pattern at 200+ ticks:**
Extremely clear checkerboard `.~.~.~` pattern. Frontier water cells have level 2-3, air cells between them have 0. The pattern is stable and doesn't resolve — it's a permanent artifact. Spans roughly 2-3 cells deep at the frontier. On a scale of "barely noticeable" to "immersion-breaking", it's about a 7/10 in severity. You can tell it's a simulation artifact immediately.

**4. Water as a "toy" rating: 4/5.**
Yes, playing with water is fun independent of seeds. Building containment structures, digging channels, and watching the wet soil diamond expand are all satisfying activities. The `fill` command makes construction fast enough that the build→tick→observe loop stays engaging. Would be 5/5 without the checkerboard.

**5. Single water behavior change to most improve next session:**
Fix the checkerboard frontier (SIM-04). Either implement a minimum water threshold (convert water_level < 5 back to air) or smooth the frontier with averaging. This one change would transform water from "functional but artificial" to "beautiful and trustworthy."

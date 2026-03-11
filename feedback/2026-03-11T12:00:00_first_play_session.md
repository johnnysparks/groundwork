# Player Feedback: First Play Session

**Build:** groundwork-sim + groundwork-tui (CLI mode)
**Tick:** 0–220
**Date:** 2026-03-11

---

## 1. What the game sold me

A cozy voxel garden where I shape soil, water, and light to build a living miniature ecosystem. The promise: dig below ground, see why things grow, compose an ecology that sustains itself.

## 2. What I actually experienced

A water simulation sandbox. I can create a world, place water, dig channels, and watch water spread and flow downhill. Soil gets wet near water. I can view horizontal slices at any depth and inspect individual voxels. The underground is visible (stone, soil, roots) but nothing grows yet. There are no plants, no seeds, no ecology — just terrain and water physics.

## 3. Best moments

- **Water spreading after first tick:** Watching the 4x4 spring expand into a growing diamond pool was immediately legible. I understood what was happening on the first view.
- **Digging a channel and seeing water flow through it:** I dug a line of air through the surface at Z=15 from the pool outward. After ticking, the channel filled with `~` water. That felt *great* — I shaped the world and got an immediate, visible result.
- **Gravity works:** I dug a 3-deep vertical shaft under the pool. Water flowed down and filled it. The cause-and-effect was crystal clear.
- **Wet soil ring:** The `%` wet-soil pattern forming a diamond around the water pool is a nice visual indicator. I can see the water's influence on surrounding terrain without inspecting each voxel.
- **Two independent water systems:** Placing a second water source at (50,50) and watching it spread independently was satisfying. I started imagining connecting them.

## 4. Confusing moments

- **Water spread pattern has a dotted/striped edge:** The expanding water at Z=16 shows an alternating `~` and `.` pattern at the frontier (visible at tick 60+). It looks like a rendering artifact or a simulation quirk. I can't tell if this is intentional (shallow water?) or a bug. It undermines trust in the simulation.
- **Placing soil over water (10,10,16):** I placed water, then placed soil on top of it. The soil replaced the water entirely (inspect shows soil with water_level=255). I expected either: the soil to block and sit on top, or the water to resist. Instead the water just vanished as a material. The high water_level number suggests the wetness is "remembered" but the water tile is gone.
- **Root does nothing:** I placed a root at (30,30,14). After 80 ticks it was unchanged — same water_level=0, same light=154, same nutrients=0. No growth, no absorption, no visual change. I understand roots aren't implemented yet, but the fact that I *can* place one creates an expectation that it does something.
- **Light underground is high:** The root at Z=14 (underground, surrounded by soil) has light_level=154. That feels wrong — underground should be dark. It makes me distrust the light system.
- **No feedback on what materials do:** When I place water/soil/root, there's no hint about what to expect. "Placed root at (30,30,14)" tells me nothing about what roots are for.
- **Z-level label is confusing:** "Z:16 (above +1)" — above what? I had to read CLAUDE.md to learn Z=15 is surface. The label should say "Z:16 (1 above surface)" or similar.

## 5. Boring or frustrating moments

- **Surface is a wall of `#`:** At Z=15, the entire world is uniform soil. There's no variation, no features, no reason to explore laterally. The only interesting thing is the water spring at the center.
- **Nothing to do after placing water:** The core loop right now is: place water, tick, view. There's no goal, no growth, no payoff for clever water routing. I dug a channel and it filled — cool — but then what? No plants to water, no ecology to feed.
- **CLI is tedious for iteration:** Each command requires typing `cargo run -p groundwork-tui --`. Placing 20 air blocks to dig a channel was painful. The CLI is functional but not fun for sustained play.
- **No undo/erase:** If I misplace something, there's no way to undo. I can place air over it, but that's clunky.
- **Nutrient_level is always 0:** Every voxel I inspected has nutrient_level=0. The system exists but does nothing yet.

## 6. Bugs

### BUG-1: Water frontier has dotted/striped visual pattern
- **Severity:** minor
- **Steps to reproduce:** `new`, `tick 60`, `view --z 16`
- **Expected result:** Water edge is a clean boundary
- **Actual result:** Alternating `~` and `.` tiles at the frontier, creating a striped look
- **Frequency:** every session, consistent
- **Notes:** May be intentional (representing shallow water) but reads as a rendering glitch. If intentional, needs a different character or explanation.

### BUG-2: Light level underground is unrealistically high
- **Severity:** major
- **Steps to reproduce:** `place root 30 30 14`, `inspect 30 30 14`
- **Expected result:** Light level near 0 (underground, surrounded by solid material)
- **Actual result:** light_level: 154
- **Frequency:** consistent
- **Notes:** Light propagation may not be attenuating through solid materials correctly. This will break any plant growth system that depends on light — underground plants would get as much light as surface ones.

### BUG-3: Out-of-bounds error message is raw Rust error
- **Severity:** minor
- **Steps to reproduce:** `place water 60 60 30`
- **Expected result:** Friendly error like "Position (60,60,30) is outside the grid (0-59, 0-59, 0-29)"
- **Actual result:** `Error: Custom { kind: InvalidInput, error: "out of bounds: (60, 60, 30)" }`
- **Frequency:** always
- **Notes:** The error works but is not player-friendly.

## 7. Feature or clarity requests

1. **Seeds and plant growth** — This is the #1 missing piece. Water spreading is neat but without plants there's no ecology and no "one more seed" hook. Even a single species that grows when water_level > threshold would transform the experience.
2. **Terrain variation in default world** — The flat uniform soil is boring. A few hills, a depression, some exposed stone would give players something to work with immediately.
3. **Light attenuation underground** — Light should drop to near-zero below solid material. This is a prerequisite for meaningful plant placement.
4. **CLI alias or shorter command** — Playing via `cargo run -p groundwork-tui --` is friction. A `groundwork` alias or shorter invocation would help.
5. **Batch placement command** — `place air 20..40 30 15` would make sculpting viable. Placing one voxel at a time is tedious.
6. **Visual indicator for water depth/level** — All water shows as `~` regardless of water_level. A shallow/deep distinction (maybe `~` vs `W`) would add readability.
7. **Help text for materials** — `place root` should say what roots do, or `help materials` should explain each one.

## 8. Brutal bottom line: would I come back tomorrow?

**No — not yet.** The water simulation works and the sculpting loop has a spark (digging that channel was genuinely fun), but there's nothing to *do* with it. I can shape terrain and route water, but nothing grows, nothing changes on its own, nothing rewards clever placement. The game needs at least one plant species that responds to water and light before the "one more seed" loop can exist. Right now it's a water physics toy, not a garden.

**What would bring me back:** Give me one seed that grows when conditions are right. Let me see a `*` root spreading underground toward water. Let me see a green shoot appear on the surface. That single addition would turn "neat water demo" into "I wonder what happens if I plant here."

---

## Evaluation Lenses (1-5)

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 2 | Water spring is a nice start, but flat terrain and no goal blunts curiosity |
| Clarity of cause and effect | 4 | Water flow and wet soil are immediately readable. Best aspect of the build. |
| Tactile satisfaction | 3 | Digging channels is fun. Placing individual voxels is tedious. |
| Beauty/readability | 2 | ASCII is functional. Uniform soil wall is dull. Dotted water edge looks glitchy. |
| Ecological fantasy delivery | 1 | No ecology exists yet. Water flows, nothing grows. |
| Desire to keep playing | 2 | After routing water a few ways, I ran out of things to try. |
| Friction/confusion | 3 | CLI works, Z-labels are confusing, light values seem wrong. |
| Trust in the simulation | 3 | Water flow is convincing. Light underground breaks trust. |

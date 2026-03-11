# Player Feedback — Underground Physics & Edge Cases

**Session focus**: Non-obvious testing — underground behavior, water physics edge cases, material interactions, boundary conditions, root mechanics. Complementing another player session covering surface-level basics.

**Build**: Tick 0–156, CLI mode, fresh world + manual excavation and placement.

---

## 1. What the game sold me

A cozy voxel garden where I shape soil, water, and light to build a living ecosystem. Digging underground and watching water flow through tunnels I carved should feel like building a terrarium's plumbing.

## 2. What I actually experienced

Water physics are the star — the spreading, the tunnel-filling, the soil absorption all work and feel tangible. But roots are inert props, light is nearly binary, and material placement has a state-bleed bug that undermines trust. The underground excavation gameplay has real promise but needs more systems to support it.

## 3. Best moments

- **Digging a vertical shaft under the water spring and watching it flood**: Water cascaded down all 5 levels and pooled at the bottom. The bottom had higher water_level (54) than mid-shaft (32). Felt physically correct and satisfying.
- **Carving a horizontal tunnel and watching water find it**: After ticking, the tunnel filled with water (`~`) and the surrounding soil became wet (`%`). This is the core "plumbing your garden" fantasy working.
- **Corner water retention**: Placing water at (0,0,16) — it retained more water (175 vs ~23 for neighbors) because it had fewer neighbors to spread into. Correct physics, felt right.
- **The wet soil ring at Z=15 after 156 ticks**: Beautiful diamond pattern radiating from the center spring. Made me want to plant things in the wet zone.

## 4. Confusing moments

- **Diagonal striped artifacts in water spread (Z=16, tick 156)**: The water layer shows alternating `~` and `.` in neat diagonal lines, especially at the spread frontier. Looks like a checkerboard artifact, not natural flow. This is the most visually jarring thing in the sim.
- **Light level of 227 at the surface vs 0 at Z=14**: Light goes from 227 (surface soil) to 0 just one layer deeper. No gradient through soil at all — it's fully binary. I expected at least a few levels of dim light in shallow soil, which would matter for root/seed placement decisions.
- **Material placement preserves old state values**: Placing stone on wet soil gives you stone with water_level=62. Placing soil on water gives you soil that already has water. This breaks the mental model of "I placed a fresh block."

## 5. Boring or frustrating moments

- **Roots do nothing**: Placed roots in soil near water, in air, in deep stone. After 20+ ticks: zero change. No growth, no water absorption, no nutrient generation. They're just a static material type with no behavior. This is the biggest gap — the game promises ecology but roots are decorative.
- **Nutrients are always 0**: After 156 ticks, every single voxel I inspected had nutrient_level=0. There's no nutrient system yet. This means there's no ecological cause-and-effect chain to discover.
- **No wet-soil display threshold feedback**: Soil at water_level=22 (tick 11) still showed as `#` not `%`. The wet-soil display didn't appear until much later. I couldn't tell my soil was getting wet by looking at it.

## 6. Bugs

### BUG-1: Material placement preserves previous voxel state (Major)
- **Steps**: Place stone at a location that has wet soil (or any material with non-zero water/light/nutrient)
- **Expected**: New material starts with default state (water=0, light=0, nutrient=0)
- **Actual**: New material inherits water_level, light_level, nutrient_level from the previous voxel
- **Impact**: Stone permanently holds water_level=62 that never drains. Breaks simulation trust.
- **Frequency**: 100% reproducible

### BUG-2: Water spread creates diagonal striped artifacts (Minor)
- **Steps**: Create a fresh world, tick 100+ times, view Z=16
- **Expected**: Water spreads in a natural-looking pattern (roughly circular or at least solid)
- **Actual**: Alternating `~` and `.` in diagonal stripes at the water frontier
- **Impact**: Visual only but breaks immersion. Manhattan-distance diamond shape is OK, but the stripey fringe looks glitchy.
- **Frequency**: 100% reproducible after enough ticks

## 7. Feature or clarity requests

1. **Reset voxel state on material placement** (bug fix, P0-level trust issue)
2. **Root behavior** — even basic water absorption from adjacent wet soil would make roots feel alive (P1)
3. **Nutrient system** — decomposition, soil nutrients, root uptake. Without this there's no ecology loop (P1)
4. **Light gradient through soil** — even 2-3 levels of dim light in shallow soil would add depth to planting decisions (P2)
5. **Wet-soil display threshold** — show `%` at a lower water_level so players get faster feedback on soil moisture (P1, quick fix)

## 8. Brutal bottom line: would I come back tomorrow?

**Not yet, but almost.** The water physics are genuinely fun to play with — digging tunnels and watching water flow is satisfying. But without roots/seeds doing anything, there's no reason to shape water. I'm building plumbing with no garden to connect it to. Give roots even one behavior (absorb water, grow toward moisture) and I'd dig elaborate irrigation systems all day. The material-state-bleed bug needs fixing first though — it erodes trust in the sim.

---

## Evaluation Scores (1-5)

| Criterion | Score | Notes |
|---|---|---|
| First-impression hook | 3 | Water spring is neat but "all soil" surface is flat |
| Clarity of cause-and-effect | 3 | Water flow is clear; light/nutrients are invisible |
| Tactile satisfaction | 4 | Digging + flooding = great; placing roots = nothing |
| Beauty/readability | 3 | ASCII works; striped artifacts hurt; wet soil ring is lovely |
| Ecological fantasy delivery | 1 | No ecology yet — just hydrology |
| Desire to keep playing | 2 | Ran out of things to discover after ~30 min |
| Friction/confusion | 2 | CLI is smooth; state-bleed bug is confusing |
| Trust in the simulation | 2 | Water is trustworthy; material placement + missing systems erode trust |

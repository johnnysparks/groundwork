# Seed Bomber Playtest — 2026-03-12

**Persona:** Chaotic, impatient. More is more. Fill everything with seeds and see what breaks.

## 1. What the game sold me
"Carpet the world with seeds and watch life explode." The `fill` command promised mass ecological chaos, and the s/S/* growth progression promised I'd see the wave front.

## 2. What I actually experienced

### Session 1: The Surface Carpet (groundwork.state)
- Placed 2,601 seeds across the surface (5,5,15)→(55,55,15) in one command. Instant, satisfying.
- After 50 ticks: only 32 seeds became roots. The rest sat dormant. Why? No water. Growth was clustered around the center water spring in a gorgeous diamond pattern. The s→S→* progression was visible as a ring expanding outward.
- Flooded the surface with water at Z=16 (another 2,601 water voxels).
- After 100 more ticks: ALL 2,569 remaining seeds converted to roots. 100% conversion. The view became a clean field of `*` roots bordered by `%` wet soil and `#` dry soil. Visually crisp.
- Planted underground seeds at Z=14 (2,601 more). Only the edges grew (200 seeds) — the perimeter adjacent to roots above. Inner seeds stuck permanently at 0 water. Ticked 200 more — no further propagation.

### Session 2: Maximum Chaos (chaos.state)
- 4 layers of seeds (Z=12 through Z=15), 14,400 total. Double flood at Z=16-17 (7,200 water).
- After 200 ticks: only the surface layer (3,600 at Z=15) converted. All 10,800 underground seeds remained permanently dormant. Water does not penetrate below a root layer.
- Sky seeds at Z=29 with water at Z=28: 6/121 grew. Water spreads horizontally, not upward effectively.
- Range placement works (`place seed 10..50 30 20` → 40 seeds). Seed protection blocked overwriting roots without `--force`.

## 3. Best moments
- The first `fill seed` command placing 2,601 seeds in one shot. That felt POWERFUL.
- Watching the growth ring expand from the water spring at tick 50 — the s/S/* concentric circles around the center were beautiful and readable.
- The flood-then-tick-100 moment where the entire field converted. Dramatic transformation.
- The status readout showing `root: +2569, seed: -2569` — pure satisfaction.

## 4. Confusing moments
- Underground seeds being permanently stuck. I expected water to seep downward through soil or roots, but it doesn't. There's no way to irrigate underground seeds once roots form above them.
- The edge-only propagation at Z=14 in the first world (where there was a soil border) vs. zero propagation in the chaos world (where the entire surface was roots). The adjacency rules weren't obvious.
- Seed at (30,30,14) showed light_level: 184/255 but water_level: 0. How does light reach underground but water doesn't?
- No wet soil (`%`) appeared underground despite massive flooding above.

## 5. Boring or frustrating moments
- Once all surface seeds converted, there was nothing to do. The garden hit equilibrium instantly. All roots, all the time. No differentiation, no ecosystem — just a monoculture of `*`.
- Underground seed stacking felt pointless. Without downward water flow, underground seeds are a dead mechanic for now.
- After the initial excitement, ticking more had zero effect. The world was "done" after ~150 ticks.

## 6. Bugs

### BUG-1: Seed protection inconsistency
- **Severity:** Minor
- **Steps:** Try `place water 30 30 14` (hits seed) → blocked. Try `place seed 30 30 16` (hits water) → only warning, succeeds.
- **Expected:** Consistent behavior — either both block or both warn.
- **Actual:** Seeds are protected from overwrite by water, but water is not protected from overwrite by seeds. Asymmetric.
- **Frequency:** Always.
- **Notes:** The asymmetry is arguably correct (seeds are precious, water isn't), but it surprised me.

### BUG-2: fill on seeds doesn't trigger protection
- **Severity:** Minor
- **Steps:** `fill seed 5 5 15 55 55 15` replaces 2,601 soil with seeds — no protection warning.
- **Expected:** Either a confirmation or at least a note that N soil voxels were replaced.
- **Actual:** Silent replacement.
- **Frequency:** Always.
- **Notes:** This is probably correct for the `fill` use case (you're choosing to fill), but I'd want confirmation when filling over existing seeds/roots.

### BUG-3: Water doesn't flow downward through root/seed layers
- **Severity:** Major (design gap)
- **Steps:** Place water at Z=16, seeds at Z=14. Tick 200.
- **Expected:** Some water seepage downward over time.
- **Actual:** Zero water reaches Z=14 seeds. Underground seeds are permanently dormant.
- **Frequency:** Always.
- **Notes:** This fundamentally limits the depth mechanic. The signature feature is above/below ground continuity, but water doesn't actually flow between them.

## 7. Feature or clarity requests
1. **Water should flow downward** — even slowly. The core fantasy of underground gardening requires it.
2. **Seed variety** — 2,601 identical seeds becoming 2,601 identical roots is satisfying once but has no replay value. Different seed types would create differentiation at scale.
3. **Carrying capacity / competition** — a field of 2,601 seeds shouldn't ALL grow. Overcrowding should matter. Seeds competing for water/light would create natural patterns instead of uniform monoculture.
4. **Dormancy feedback** — I shouldn't need `inspect` to understand why a seed isn't growing. The view could show moisture gradients or the `s` glyph could vary based on conditions (e.g., dim vs. bright).
5. **Post-root lifecycle** — once everything is `*`, the garden is dead as a game. Roots need to DO something — produce nutrients, spread, compete, die, flower.

## 8. Brutal bottom line: would I come back tomorrow?
**No.** The initial fill-flood-grow cycle was a huge rush. But the garden reaches equilibrium instantly and then nothing happens. I'd come back when: (a) water flows underground, (b) seeds compete and differentiate, (c) roots have a lifecycle beyond just existing.

## Evaluation Lenses

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 4/5 | The fill command is instantly gratifying. Dropping 2,601 seeds in one shot is a power fantasy. |
| Clarity of cause & effect | 3/5 | Water → growth is clear. But underground water blockage is invisible. Why seeds at Z=14 won't grow is a mystery without `inspect`. |
| Tactile satisfaction | 4/5 | fill + tick + view loop is punchy. The mass conversion moment is great. |
| Beauty/readability | 3/5 | At scale, the s/S/* distinction works when there's variety. But a uniform field of 2,601 `*` is boring. The growth ring was the prettiest moment. |
| Ecological fantasy delivery | 2/5 | It's a planting fantasy, not an ecological one. No competition, no diversity, no lifecycle. Every seed is equal and the outcome is predetermined. |
| Desire to keep playing | 2/5 | High for the first 5 minutes, then nothing left to do. |
| Friction/confusion | 4/5 (low friction) | Commands work great. fill, place with ranges, --force all feel right. |
| Trust in the simulation | 3/5 | I trust it's running rules consistently but the rules are too simple to feel ecological. Water not flowing down broke my trust in the physics. |

## Specific Manager Questions

1. **Seeds placed / roots ratio:** Session 1: 5,202 seeds placed (surface + underground), 2,801 became roots (54%). Session 2: 14,521 seeds placed, 3,606 became roots (25%). At scale, most seeds are wasted underground.
2. **s/S/* distinction at scale:** Excellent during the growth wave — the concentric rings of s→S→* were the highlight. Useless after full conversion (all `*` looks identical). At 1000+ scale with variety, it's readable. At 1000+ uniform, it's visual noise.
3. **Placement protection during bulk ops:** Only hit it once (trying to place water on underground seed). Helpful for single `place`, irrelevant for `fill` (which doesn't trigger it). The `--force` escape hatch is good.
4. **Visual appeal of maxed-out garden:** 2/5. A uniform field of `*` is not a garden, it's a parking lot. The PROCESS of growth (the ring expanding) was 4/5.
5. **Most chaotic thing:** 14,400 seeds across 4 depth layers + 7,200 water. Expected an underground root explosion. Got a surface monoculture with 10,800 dormant seeds underneath. The chaos was disappointingly contained.

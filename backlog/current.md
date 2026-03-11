# Backlog — Groundwork

_Last updated: 2026-03-11T16:00:00 by Manager_

## TL;DR — Where We Are

**Done:** CLI usability fixes (Sprint 1), seed growth system (Sprint 2). Seeds grow into roots when near water+light. 15 tests pass. Player validation complete — 6 sessions across 5 personas.

**New P0:** Placement validation. Players can silently destroy seeds, roots, and water spring cells. The most intuitive watering action (place water on seed) kills the seed. 4/6 sessions flagged this as the most hostile moment in the game.

**The build can:** create worlds, place materials, simulate water flow + light + seed growth, save/load state. The seed→root growth is the first genuine "one more seed" moment.

**The build can't yet:** protect living things from accidental destruction, show growth progress visually, handle batch placement, make roots interact with the world, or spread light/water horizontally underground.

**Key insight from player testing:** The seed→root conversion is real delight. Every session confirmed "one more seed" pull. But players nearly quit 2-3 times before reaching that moment due to opaque mechanics and destructive placement. The path to delight is too punishing.

---

## Completed

### Sprint 1: CLI Usability (PR #3)
- ~~CLI-01: Fix no-terminal panic~~ ✓
- ~~CLI-02: Add legend to `view`~~ ✓
- ~~CLI-03: Add axis labels to `view`~~ ✓
- ~~CLI-04: Show change summary after `tick`~~ ✓
- ~~CLI-05: Warn on out-of-bounds Z~~ ✓
- ~~CLI-06: Show value ranges in `inspect`~~ ✓
- ~~CLI-07: Add wet-soil count to `status`~~ ✓

### Sprint 2: Core Sim Fixes + Seed Growth
- ~~SIM-01: Fix material placement state bleed~~ ✓
- ~~SIM-02: Fix light attenuation through soil~~ ✓
- ~~GAME-01: Add seed species with growth system~~ ✓ (40-tick growth, water+light threshold)

### Player Validation Round 1 (6 sessions, 5 personas)
- ~~Seed Growth Playtest~~ ✓ — Confirmed growth works near spring, irrigation channels don't work
- ~~Ecologist / Scientist~~ ✓ — Mapped viable zone, confirmed deterministic sim (5/5 trust), found exploit paths
- ~~Optimizer / Irrigation Engineer~~ ✓ — Grew 138 roots, found carpet-failure constraint, no resource pressure
- ~~Garden Designer~~ ✓ — Built 24×24 garden, water flooding erases composition
- ~~Weekend Gardener~~ ✓ — Nearly quit 3× before first root, destructive placement is hostile
- ~~Spelunker~~ ✓ — Grew root at Z=2, caves are dead zones (no horizontal light/water)

---

## P0 — Blocks core proof or makes build unusable

### CLI-08: Placement validation — protect seeds, roots, and water sources
- **Owner:** tools
- **Why:** 4/6 sessions flagged destructive placement as the single worst moment. Placing water on a seed silently destroys it — this is the most intuitive watering action and it kills the plant. The ecologist proved you can permanently destroy the water spring cell by cell. Weekend gardener nearly quit the game over this. A casual player's first instinct (water my seed) must not punish them.
- **Done when:**
  - `place` rejects overwriting Seed or Root with any material (prints warning, suggests `--force`)
  - `place` warns when overwriting a Water cell at surface level (spring protection)
  - `--force` flag bypasses the protection
- **Dependencies:** none
- **Risk:** low
- **Scope:** Core trust — if the obvious action destroys your work, the game breaks the player's trust before they see any delight.
- **Evidence:** Weekend gardener BUG-2, Ecologist BUG-1/BUG-2, Garden designer confusion, Seed growth playtest

### VIS-02: Seed growth progress indicator (PROMOTED from P2)
- **Owner:** tools
- **Why:** 6/6 sessions flagged the invisible growth counter. nutrient_level repurposes as growth but is labeled generically, not shown in view, and gives zero intermediate feedback during the 40-tick wait. Players inspect seeds showing water_level:0 and nutrient_level climbing — both confusing. This is the #1 readability gap.
- **Done when:**
  - Seeds in ASCII view show growth stage: `s` (0-99), `S` (100-199), then `*` Root (200+)
  - `inspect` on a seed shows: `growth: 120/200 (60%)`, conditions met/unmet: `water: YES (neighbor below: 98)`, `light: YES (229)`
  - If a seed is not growing, inspect says why: `growth blocked: no water nearby` or `growth blocked: no light`
- **Dependencies:** none
- **Risk:** low
- **Scope:** Every session asked for this. Readability is a design constraint. Without it, the growth loop is a black box.
- **Evidence:** All 6 sessions, Weekend gardener (near-quit #3), Optimizer ("no growth counter visible"), Ecologist ("nutrient_level meaning is opaque")

## P1 — Strongly improves clarity, feel, or core loop

### SIM-03: Root water absorption
- **Owner:** gameplay
- **Why:** Roots are inert after growth. Players see seed→root conversion but roots don't interact with the world. Absorbing water from adjacent wet soil creates the first ecological chain: water → soil → root. Ecologist confirmed roots show water_level=0 always. Optimizer found no resource pressure — infinite water with no consumption. Root absorption solves both: ecological chain AND resource constraint.
- **Done when:** Root voxels drain water_level from adjacent Soil voxels (~4/tick). Root's own water_level increases. Visible effect: wet soil near roots dries out over time.
- **Dependencies:** none (GAME-01 shipped)
- **Risk:** low
- **Scope:** First ecological relationship — required for "self-sustaining" fantasy. Also creates resource pressure the optimizer needs.

### SIM-04: Fix diagonal stripe artifact in water spread
- **Owner:** gameplay
- **Why:** 5/6 sessions reported the checkerboard pattern. After 100+ ticks, water frontier shows `.~.~.~` alternating pattern. Spelunker confirmed at 230 ticks. Undermines trust in the simulation.
- **Done when:** Water frontier edge is smooth (no alternating wet/dry pattern). Likely even/odd tick interaction with snapshot buffer.
- **Dependencies:** none
- **Risk:** low-medium — may require investigation
- **Scope:** Visual clarity is a design constraint.

### SIM-05: Lower wet-soil display threshold
- **Owner:** tools
- **Why:** Wet soil `%` doesn't appear until water_level > 100, which takes many ticks. Faster visual feedback improves cause-and-effect readability.
- **Done when:** Wet soil threshold lowered to ~50 in both cli.rs and render.rs.
- **Dependencies:** none
- **Risk:** low
- **Scope:** Cause-and-effect readability.

### VIS-01: Dark indicator for underground air
- **Owner:** tools
- **Why:** Underground air cells with light_level=0 display identically to surface air (`.`). 3/6 sessions flagged this. Spelunker said it was "especially painful underground where light is the key resource you're engineering." Can't plan shaft/cave builds without seeing where light reaches.
- **Done when:** Air cells with light_level=0 render as ` ` (space) instead of `.` in both cli.rs and render.rs.
- **Dependencies:** none
- **Risk:** low
- **Scope:** Underground readability — critical for underground gardening playstyle.

### CLI-09: Default view Z=15 (surface) instead of Z=16
- **Owner:** tools
- **Why:** 2/6 sessions flagged that `view` defaults to Z=16 which is mostly empty air with a small pond. First impression is "where's the garden?" The surface (Z=15) is where seeds live and where the action starts. Weekend gardener's first command showed empty air.
- **Done when:** `view` with no `--z` argument defaults to Z=15.
- **Dependencies:** none
- **Risk:** low
- **Scope:** First-impression quality.

## P2 — Valuable but not required for MVP

### GAME-03: Batch voxel placement (PRIORITY INCREASE — was P2, now high-P2, next to promote)
- **Owner:** tools
- **Why:** 5/6 sessions flagged single-voxel placement as the #1 UX friction. Optimizer needed 138 commands. Garden designer needed 300. Spelunker needed 99 for one shaft. Underground play and designed gardens are blocked by this.
- **Done when:** `place` accepts range syntax like `place air 20..40 30 15` for a row, or `fill` command for rectangles.
- **Dependencies:** none
- **Risk:** low
- **Note:** This is the strongest P2 signal. If the sprint has room after P0/P1, promote this.

### GAME-02: Varied default terrain
- **Owner:** gameplay
- **Why:** Flat uniform terrain is boring. Multiple sessions flagged the "wall of `#`" as a dull first impression.
- **Done when:** Default terrain has elevation variation (perlin noise or simple sine waves) and a few stone outcrops.
- **Dependencies:** none
- **Risk:** low

### GAME-04: Water depth visual
- **Owner:** tools
- **Why:** All water looks the same (`~`). Distinguishing shallow from deep would improve readability.
- **Done when:** Deep water (water_level > 200) shows as `≈` or uses a different character.
- **Dependencies:** none
- **Risk:** low

### CLI-10: Inspect growth diagnostics
- **Owner:** tools
- **Why:** Ecologist and optimizer both requested detailed growth diagnostics beyond the P0 progress indicator. Batch inspect, neighbor water details, estimated ticks to maturity.
- **Done when:** `inspect` on a seed shows neighbor water values, whether each condition (water/light) is met, and estimated ticks remaining.
- **Dependencies:** VIS-02 (basic growth display ships first)
- **Risk:** low

### SIM-06: Seed light attenuation
- **Owner:** gameplay
- **Why:** Seeds are 100% transparent to light. Ecologist proved stacking seeds creates "light pipes" delivering more light underground than air does (seeds: 0 attenuation, air: 2/layer). This is a minor exploit.
- **Done when:** Seeds attenuate light by ~5-10 per layer, comparable to air.
- **Dependencies:** none
- **Risk:** low
- **Decision needed:** Is seed-as-light-pipe intentional? Dev noted "acceptable for MVP since players place seeds on surfaces." With underground play emerging, this matters more. Recommend fixing.

## P3 — Future / expansion

### GAME-05: Nutrient system
- **Owner:** gameplay
- **Why:** Player requested. Creates deeper ecology chain. Too complex for current sprint.
- **Scope:** Deferred until seed growth loop is proven fun.

### GAME-06: Multiple plant species
- **Owner:** gameplay
- **Why:** 12-20 species is MVP target, but proving one species works comes first.
- **Scope:** Deferred until one species growth loop works.

### SIM-07: Horizontal light scatter
- **Owner:** gameplay
- **Why:** Spelunker proved light propagates strictly top-down. Caves are dead zones — zero horizontal light spread. This blocks the underground garden fantasy entirely. However, implementing proper light scatter is a significant system change and not required to prove the core surface loop.
- **Scope:** Deferred until surface loop is proven. Underground gardening is a future playstyle expansion.
- **Note:** Even 2-3 tiles of scatter from shaft openings would transform underground play. Consider a simple heuristic rather than full radiosity.

### SIM-08: Horizontal water flow in air
- **Owner:** gameplay
- **Why:** Spelunker found water doesn't spread horizontally through air in caves. Water only flows straight down. Combined with no horizontal light, caves are useless for gardening.
- **Scope:** Deferred with SIM-07. Underground play depends on both.
- **Note:** Could be simpler than light — water pooling at the lowest air tile of a cave is physically intuitive.

### GAME-07: Water containment / drainage
- **Owner:** gameplay
- **Why:** Garden designer found water floods all non-stone space uncontrollably. Air gaps don't slow water. No drainage tool. Makes designed layouts impossible to maintain. Optimizer also flagged uncontrollable water channels.
- **Scope:** Deferred. Interesting design problem but not required for core loop proof.

### GAME-08: Dark-loving species (mushrooms)
- **Owner:** gameplay
- **Why:** Spelunker requested something that thrives without light. If light can't reach caves easily, give the player something that wants to be in a cave.
- **Scope:** Deferred until species system expands.

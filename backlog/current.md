# Backlog — Groundwork

_Last updated: 2026-03-11T18:00:00 by Manager_

## TL;DR — Where We Are

**Done:** CLI usability (Sprint 1), seed growth (Sprint 2), placement validation + growth visibility + batch placement (Sprint 3). The P0 trust blockers are resolved. Seeds are protected, growth is visible, and batch placement removes the tedium wall. 24+ sim tests pass, 5 TUI integration tests pass.

**Current focus:** Ecological depth. Roots are inert — the first real system interaction (root water absorption, SIM-03) is the top P1. The checkerboard water artifact (SIM-04) and remaining visual polish (SIM-05, VIS-01) round out the sprint.

**The build can:** create worlds, place/fill materials (with ranges), protect seeds/roots from destruction, simulate water flow + light + seed growth with visible progress (s→S→*), show growth diagnostics, save/load state.

**The build can't yet:** make roots interact with the world, fix the checkerboard water pattern, show water depth, vary terrain, or support multiple species.

**Key insight:** The trust barrier is gone. Players can now safely experiment without losing work. The "one more seed" moment is reachable within 5 minutes. The next frontier is ecological depth — making the garden *respond* to the player, not just grow.

---

## Completed

### Sprint 1: CLI Usability
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

### Sprint 3: Trust & Readability
- ~~CLI-08: Placement validation~~ ✓ — Seeds/roots protected, water warns, `--force` overrides. 5 integration tests.
- ~~VIS-02: Seed growth progress indicator~~ ✓ — `s`→`S`→`*` in view, growth diagnostics in inspect (progress %, conditions, dormancy reason).
- ~~GAME-03: Batch voxel placement~~ ✓ — Range syntax on `place` (`20..40`), `fill` command for rectangles. Optimizer: 138→~10 commands. Spelunker shaft: 99→1 command.

### Player Validation Round 1 (6 sessions, 5 personas)
- ~~Seed Growth Playtest~~ ✓ — Confirmed growth works near spring, irrigation channels don't work
- ~~Ecologist / Scientist~~ ✓ — Mapped viable zone, confirmed deterministic sim (5/5 trust), found exploit paths
- ~~Optimizer / Irrigation Engineer~~ ✓ — Grew 138 roots, found carpet-failure constraint, no resource pressure
- ~~Garden Designer~~ ✓ — Built 24×24 garden, water flooding erases composition
- ~~Weekend Gardener~~ ✓ — Nearly quit 3× before first root, destructive placement is hostile
- ~~Spelunker~~ ✓ — Grew root at Z=2, caves are dead zones (no horizontal light/water)

---

## P0 — Blocks core proof or makes build unusable

_No P0 items. Trust blockers resolved in Sprint 3._

---

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

### SIM-06: Seed light attenuation
- **Owner:** gameplay
- **Why:** Seeds are 100% transparent to light. Ecologist proved stacking seeds creates "light pipes" delivering more light underground than air does (seeds: 0 attenuation, air: 2/layer). This is a minor exploit.
- **Done when:** Seeds attenuate light by ~5-10 per layer, comparable to air.
- **Dependencies:** none
- **Risk:** low
- **Note:** Matters more with underground play. Low priority for surface MVP.

### CLI-10: Inspect growth diagnostics (PARTIALLY DONE)
- **Owner:** tools
- **Why:** Ecologist and optimizer both requested detailed growth diagnostics. VIS-02 shipped the core: progress %, conditions met/unmet, dormancy reason. Remaining: batch inspect, neighbor water details.
- **Done when:** `inspect` supports coordinate ranges. Neighbor water values shown individually.
- **Dependencies:** VIS-02 ✓
- **Risk:** low

## P3 — Future / expansion

### UNDERGROUND-01: Underground play expansion
- **Owner:** gameplay
- **Part of:** Future expansion
- **Why:** Spelunker session proved caves are dead zones. Compelling future playstyle but depends on SIM-07, SIM-08, GAME-08.
- **Prerequisite:** Surface growth loop proven fun (GAME-01 ✓), placement safety (CLI-08 ✓), growth visibility (VIS-02 ✓), root absorption (SIM-03).
- **Note:** Start with SIM-07 (light scatter) and SIM-08 (water pooling). GAME-08 (mushrooms) provides cave content.

### GAME-05: Nutrient system
- **Owner:** gameplay
- **Scope:** Deferred until seed growth loop is proven fun.

### GAME-06: Multiple plant species
- **Owner:** gameplay
- **Scope:** 12-20 species is MVP target, but proving one species works comes first.

### SIM-07: Horizontal light scatter
- **Part of:** UNDERGROUND-01
- **Scope:** Deferred. Underground gardening is a future expansion.

### SIM-08: Horizontal water flow in air
- **Part of:** UNDERGROUND-01
- **Scope:** Deferred with SIM-07.

### GAME-07: Water containment / drainage
- **Owner:** gameplay
- **Scope:** Deferred. Interesting design problem but not required for core loop proof.

### GAME-08: Dark-loving species (mushrooms)
- **Part of:** UNDERGROUND-01
- **Scope:** Deferred until species system expands.

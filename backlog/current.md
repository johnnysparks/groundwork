# Backlog — Groundwork

_Last updated: 2026-03-11T15:00:00 by Manager_

## TL;DR — Where We Are

**Done:** CLI usability fixes (Sprint 1), seed growth system (Sprint 2). Seeds grow into roots when near water+light. 15 tests pass.

**No P0 blockers.** Next priority: validate seed growth with players, then tackle P1s (root water absorption, water stripe fix, wet-soil threshold, dark underground air).

**The build can:** create worlds, place materials (air/soil/stone/water/root/seed), simulate water flow + light + seed growth, save/load state.

**The build can't yet:** make roots do anything after growth, show growth progress visually, handle batch placement, or vary terrain.

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

## P0 — Blocks core proof or makes build unusable

_No P0s. Seed growth loop is live. Next: validate it with players._

## P1 — Strongly improves clarity, feel, or core loop

### SIM-03: Root water absorption
- **Owner:** gameplay
- **Why:** Roots are inert after growth. Players see seed→root conversion but roots don't interact with the world. Absorbing water from adjacent wet soil creates the first ecological chain: water → soil → root. Three player sessions flagged inert roots as a trust-breaker.
- **Done when:** Root voxels drain water_level from adjacent Soil voxels (~4/tick). Root's own water_level increases. Visible effect: wet soil near roots dries out over time.
- **Dependencies:** none (GAME-01 shipped)
- **Risk:** low
- **Scope:** First ecological relationship — required for "self-sustaining" fantasy.

### SIM-04: Fix diagonal stripe artifact in water spread
- **Owner:** gameplay
- **Why:** All four player sessions reported this. After 100+ ticks, water frontier shows `.~.~.~` checkerboard pattern. Undermines trust in sim. Most-reported visual bug.
- **Done when:** Water frontier edge is smooth (no alternating wet/dry pattern). Likely iteration-order interaction with snapshot buffer.
- **Dependencies:** none
- **Risk:** low-medium — may require investigation
- **Scope:** Visual clarity is a design constraint.

### SIM-05: Lower wet-soil display threshold
- **Owner:** tools
- **Why:** Wet soil `%` doesn't appear until water_level > 100, which takes many ticks. Faster visual feedback improves cause-and-effect readability. Two player sessions specifically flagged this.
- **Done when:** Wet soil threshold lowered to ~50 in both cli.rs and render.rs.
- **Dependencies:** none
- **Risk:** low
- **Scope:** Cause-and-effect readability.

### VIS-01: Dark indicator for underground air
- **Owner:** tools
- **Why:** Underground air cells with light_level=0 display identically to surface air (`.`). Players think it's a bug. Need visual distinction between "open sky" and "cave dark."
- **Done when:** Air cells with light_level=0 render as ` ` (space) or `:` instead of `.` in both cli.rs and render.rs.
- **Dependencies:** none
- **Risk:** low
- **Scope:** Underground readability — critical for underground gardening playstyle.

## P2 — Valuable but not required for MVP

### GAME-02: Varied default terrain
- **Owner:** gameplay
- **Why:** Flat uniform terrain is boring. Hills, depressions, and exposed stone give the player features to work with and around. Multiple sessions flagged the "wall of `#`" as a dull first impression.
- **Done when:** Default terrain has elevation variation (perlin noise or simple sine waves) and a few stone outcrops.
- **Dependencies:** none
- **Risk:** low

### GAME-03: Batch voxel placement
- **Owner:** tools
- **Why:** Placing voxels one at a time is the #1 UX pain point. Every player session flagged tedious single-voxel placement. Underground exploration is blocked by this.
- **Done when:** `place` accepts range syntax like `place air 20..40 30 15` for a row.
- **Dependencies:** none
- **Risk:** low

### GAME-04: Water depth visual
- **Owner:** tools
- **Why:** All water looks the same (`~`). Distinguishing shallow from deep would improve readability.
- **Done when:** Deep water (water_level > 200) shows as `≈` or uses a different character.
- **Dependencies:** none
- **Risk:** low

### VIS-02: Seed growth progress indicator
- **Owner:** tools
- **Why:** Dev notes that there's no visual difference between "just planted" and "almost grown." Players can't read growth progress without `inspect`.
- **Done when:** Seeds near maturity (nutrient_level > 150) display differently, e.g. `S` vs `s`.
- **Dependencies:** none
- **Risk:** low

## P3 — Future / expansion

### GAME-05: Nutrient system
- **Owner:** gameplay
- **Why:** Player requested. Creates deeper ecology chain. Too complex for current sprint.
- **Scope:** Deferred until seed growth loop is proven fun.

### GAME-06: Multiple plant species
- **Owner:** gameplay
- **Why:** 12-20 species is MVP target, but proving one species works comes first.
- **Scope:** Deferred until one species growth loop works.

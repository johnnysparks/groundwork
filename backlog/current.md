# Backlog — Groundwork

_Last updated: 2026-03-11T13:00:00 by Manager_

## Completed (PR #3)

- ~~CLI-01: Fix no-terminal panic~~ ✓
- ~~CLI-02: Add legend to `view`~~ ✓
- ~~CLI-03: Add axis labels to `view`~~ ✓
- ~~CLI-04: Show change summary after `tick`~~ ✓
- ~~CLI-05: Warn on out-of-bounds Z~~ ✓
- ~~CLI-06: Show value ranges in `inspect`~~ ✓
- ~~CLI-07: Add wet-soil count to `status`~~ ✓
- ~~SIM-02: Fix light attenuation through soil~~ ✓

## P0 — Blocks core proof or makes build unusable

### SIM-01: Fix material placement state bleed
- **Owner:** gameplay
- **Why:** Placing a new material on a voxel preserves the old water/light/nutrient levels. Stone permanently holds water_level=62. This breaks player trust in the simulation and will cause bugs in plant growth (seeds inheriting random state).
- **Done when:** `place <material> x y z` resets water_level, light_level, and nutrient_level to 0 (except: placing Water sets water_level=255, which already happens). Light will be recalculated next tick by light_propagation.
- **Dependencies:** none
- **Risk:** low — one-line fix in `cli.rs:cmd_place`
- **Scope:** Sim trust is a core design constraint.

### GAME-01: Add seed species with growth system
- **Owner:** gameplay
- **Why:** This is THE missing core loop. Three player sessions all say the same thing: water is fun but nothing grows. Without seeds, there is no garden builder game. This is the single most important task for the project.
- **Done when:**
  1. New `Material::Seed` exists (ASCII char: `s`)
  2. Player can `place seed x y z`
  3. New ECS system `seed_growth` runs each tick:
     - For each Seed voxel: if water_level >= 30 AND light_level >= 30, increment nutrient_level as a growth counter
     - When nutrient_level reaches 200, convert Seed to Root
     - Root already exists as a material, so this "completes" the cycle
  4. At least one test: place seed on wet lit soil, tick N times, verify it becomes Root
- **Dependencies:** SIM-01 (state bleed fix), ~~SIM-02 (light fix)~~
- **Risk:** medium — first new game system, needs careful integration with existing ECS schedule
- **Scope:** This IS the MVP proof. No garden builder without growth.

## P1 — Strongly improves clarity, feel, or core loop

### SIM-03: Root water absorption
- **Owner:** gameplay
- **Why:** Roots are inert props. Players place them and nothing happens. Roots absorbing water from adjacent wet soil creates the first ecological relationship: water → soil → root.
- **Done when:** New system or extension to soil_absorption: Root voxels drain water_level from adjacent Soil voxels (transfer ~4/tick). Root's own water_level increases. Visible effect: wet soil near roots dries out over time.
- **Dependencies:** GAME-01 (so roots appear naturally, not just manually placed)
- **Risk:** low
- **Scope:** First ecological chain — required for "self-sustaining" fantasy.

### SIM-04: Fix diagonal stripe artifact in water spread
- **Owner:** gameplay
- **Why:** After 100+ ticks, water frontier shows checkerboard/striped pattern. Minor visual issue but players notice it and it undermines trust.
- **Done when:** Water frontier edge is smooth (no alternating wet/dry pattern). Likely caused by iteration-order interaction with the snapshot buffer.
- **Dependencies:** none
- **Risk:** low-medium — may require investigation
- **Scope:** Visual clarity is a design constraint.

### SIM-05: Lower wet-soil display threshold
- **Owner:** tools
- **Why:** Player feedback: wet soil `%` doesn't appear until water_level > 100, which takes many ticks. Faster visual feedback improves the cause-and-effect loop.
- **Done when:** Wet soil threshold lowered to ~50 in both cli.rs and render.rs. Player sees `%` sooner after water reaches soil.
- **Dependencies:** none
- **Risk:** low
- **Scope:** Cause-and-effect readability.

## P2 — Valuable but not required for MVP

### GAME-02: Varied default terrain
- **Owner:** gameplay
- **Why:** Flat uniform terrain is boring. Hills, depressions, and exposed stone give the player features to work with and around.
- **Done when:** Default terrain has some elevation variation (perlin noise or simple sine waves) and a few stone outcrops.
- **Dependencies:** none
- **Risk:** low

### GAME-03: Batch voxel placement
- **Owner:** tools
- **Why:** Placing voxels one at a time is too tedious for meaningful sculpting.
- **Done when:** `place` accepts range syntax like `place air 20..40 30 15` for a row.
- **Dependencies:** none
- **Risk:** low

### GAME-04: Water depth visual
- **Owner:** tools
- **Why:** All water looks the same (`~`). Distinguishing shallow from deep would improve readability.
- **Done when:** Deep water (water_level > 200) shows as `≈` or uses a different character.
- **Dependencies:** none
- **Risk:** low

## P3 — Future / expansion

### GAME-05: Nutrient system
- **Owner:** gameplay
- **Why:** Player requested. Creates deeper ecology chain. But too complex for current sprint.
- **Scope:** Deferred until seed growth loop is proven fun.

### GAME-06: Multiple plant species
- **Owner:** gameplay
- **Why:** 12-20 species is MVP target, but proving one species works comes first.
- **Scope:** Deferred until one species growth loop works.

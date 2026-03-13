# Backlog — Groundwork

_Last updated: 2026-03-13 by Manager_

## TL;DR — Where We Are

**Done:** CLI usability (Sprint 1), seed growth (Sprint 2), placement validation + growth visibility + batch placement (Sprint 3), 12 player sessions across 2 rounds, interface parity sprint (TUI-01–04, CLI-21–22), dynamic terrain with rolling hills/stream/outcrops (Sprint 4), procedural tree generation with 4 species (Sprint 4), soil composition system (Sprint 4), root water absorption (Sprint 4), scale normalization to 0.5m/voxel (Sprint 5).

**Current grid:** 120×120×60 voxels (60m×60m×30m at 0.5m/voxel). GROUND_LEVEL=30. Surface heights vary ~26-34 with rolling hills.

**The build can:** create worlds with dynamic terrain (hills, stream, stone outcrops, water spring), place/fill materials (with ranges), protect seeds/roots from destruction, simulate water flow + light + soil absorption + root water absorption + soil evolution, grow seeds into trees through 5 stages (seed → seedling → sapling → mature → dead) with species-specific procedural branching via space colonization, self-prune shaded branches, disperse seeds from mature trees, save/load state (v3 with soil), pan viewport in TUI, show soil composition and type in inspect.

**The build can't yet:** grow flowers/shrubs/groundcover (only 4 tree species), show cross-section views, display water depth visually, or support the full 12-20 species MVP target.

**Process rule:** CLI and TUI must ship player-facing features together.

**Next priority:** Expand species diversity toward MVP target (12-20 species). The tree growth pipeline is proven — add smaller plants (shrubs, groundcover, flowers) that create layered ecosystems.

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
- ~~VIS-02: Seed growth progress indicator~~ ✓ — `s`→`S`→`*` in view, growth diagnostics in inspect.
- ~~GAME-03: Batch voxel placement~~ ✓ — Range syntax on `place` (`20..40`), `fill` command for rectangles.
- ~~CLI-09: Default view Z=surface~~ ✓ — First command shows surface, not empty air.

### Sprint 4: Ecological Depth
- ~~SIM-03: Root water absorption~~ ✓ — Roots drain water from adjacent soil, creating dry zones around root systems.
- ~~GAME-02: Varied default terrain~~ ✓ — Rolling hills (sine-wave elevation), stone outcrops, stream from spring to SE edge.
- ~~SIM-04: Checkerboard water fix~~ ✓ — Snapshot-based water flow eliminates iteration-order artifacts.
- ~~GAME-09: Procedural tree generation~~ ✓ — 4 species (oak, birch, willow, pine), space colonization branching, phototropism, self-pruning, seed dispersal.
- ~~SIM-09: Soil composition system~~ ✓ — 6 properties (sand/clay/organic/rock/pH/bacteria), depth-based layers, soil evolution (weathering, bacteria growth, pH drift).
- ~~SIM-05: Lower wet-soil display threshold~~ ✓ — Threshold at 50 in render.rs.
- ~~VIS-01: Dark indicator for underground air~~ ✓ — Dark air renders as blank space.

### Sprint 5: Scale Normalization
- ~~SCALE-01: Scale module~~ ✓ — All dimensions in meters, converted via `meters_to_voxels()`. Species dimensions in meters with voxel accessors.
- ~~SCALE-02: Physics rate scaling~~ ✓ — `scale_attenuation()` / `scale_transfer()` wrap all physics rates.
- ~~SCALE-03: Resolution increase~~ ✓ — VOXEL_SIZE_M changed from 1.0 to 0.5. Grid doubled to 120×120×60. All tests pass (79 tests).

### Player Validation Round 1 (6 sessions)
- ~~Seed Growth Playtest~~ ✓ — Confirmed growth works near spring
- ~~Ecologist / Scientist~~ ✓ — Mapped viable zone, deterministic sim (5/5 trust)
- ~~Optimizer / Irrigation Engineer~~ ✓ — Grew 138 roots, no resource pressure
- ~~Garden Designer~~ ✓ — Built 24×24 garden, water flooding erases composition
- ~~Weekend Gardener~~ ✓ — Nearly quit 3× before first root, destructive placement hostile
- ~~Spelunker~~ ✓ — Grew root at Z=2, caves are dead zones

### Player Validation Round 2 (6 sessions)
- ~~Terraformer~~ ✓ — Terrain sculpting works, water-terrain interaction shallow (3.5/5)
- ~~Patience Gardener~~ ✓ — First 50 ticks excellent, world dies after seed→root (2/5 avg)
- ~~Hydrogeologist~~ ✓ — Water is a 4/5 toy, checkerboard #1 issue, water vanishes on stone
- ~~Vertical Farmer~~ ✓ — Built 3-tier farm, grew roots to Z=5, needs cross-section view
- ~~Destructor~~ ✓ — Found fill protection bypass (major), solid foundation otherwise
- ~~Storyteller~~ ✓ — Genuinely emotional session, fill drowned seeds, root is endpoint

### Interface Parity Sprint
- ~~TUI-01: Cursor/focus system~~ ✓ — Viewport-centered camera, WASD pans
- ~~TUI-02: Tool mode~~ ✓ — Tab cycles material, Enter start/end, range preview
- ~~TUI-03: Inspect panel~~ ✓ — `I` toggles side panel with voxel details
- ~~TUI-04: Status display~~ ✓ — `T` toggles material counts
- ~~CLI-21: Focus command~~ ✓ — Persistent cursor for CLI
- ~~CLI-22: Tool start/end~~ ✓ — Two-step range operations

---

## P0 — Blocks core proof or makes build unusable

### CLI-12: Fix `fill` seed/root protection bypass
- **Owner:** tools
- **Why:** `fill` silently overwrites seeds and roots. 3/6 Round 2 sessions hit this. Undermines CLI-08 protection.
- **Done when:** `fill` skips Seed and Root voxels by default. Reports "N protected cells skipped." `fill --force` overrides.
- **Dependencies:** CLI-08 ✓
- **Risk:** low

---

## P1 — Strongly improves clarity, feel, or core loop

### GAME-06: Expand species diversity (shrubs, groundcover, flowers)
- **Owner:** gameplay
- **Why:** MVP targets 12-20 species. We have 4 tree species. The garden needs layered ecosystems: canopy trees shade shrubs, shrubs shelter groundcover, flowers attract pollinators. Without smaller plants, the garden is just trees.
- **Done when:** At least 4 non-tree species (e.g., fern, moss, wildflower, berry bush) with species-appropriate growth patterns, light/water requirements, and visual representation.
- **Dependencies:** Tree system ✓, soil system ✓
- **Risk:** medium — growth patterns for non-tree plants need different logic than space colonization
- **Scope:** Core loop — species diversity is what makes "one more seed" compelling.

### GAME-04: Water depth visual
- **Owner:** tools
- **Why:** All water looks the same. 4+ sessions want depth indication.
- **Done when:** Deep water uses a different character or color than shallow water.
- **Risk:** low

### CLI-13: Cross-section view
- **Owner:** tools
- **Why:** With 60 Z-levels and trees spanning multiple layers, vertical visualization is critical. `view --cross-y 60` showing X horizontal, Z vertical.
- **Done when:** Cross-section renders a vertical slice through the grid.
- **Risk:** low-medium (new rendering mode)

### CLI-14: Equilibrium detection
- **Owner:** tools
- **Why:** 2/6 Round 2 sessions ticked into the void with no feedback.
- **Done when:** After 10+ ticks with no material changes, message suggests planting or placing water.
- **Risk:** low

### CLI-15: Named tick events
- **Owner:** tools
- **Why:** Storyteller wants "Seed at (27,30) took root!" in tick summaries.
- **Done when:** Tick summary names which seeds grew, which became roots/trees.
- **Risk:** low

---

## P2 — Valuable but not required for MVP

### CLI-10: Inspect growth diagnostics (PARTIALLY DONE)
- **Owner:** tools
- **Why:** Remaining: batch inspect, neighbor water details.
- **Done when:** `inspect` supports coordinate ranges.
- **Risk:** low

### CLI-16: `--force` flag position flexibility
- **Owner:** tools
- **Why:** `--force` only works after coordinates. Confusing.
- **Done when:** `--force` accepted in any argument position.
- **Risk:** low

### CLI-17: Validate `tick` arguments
- **Owner:** tools
- **Why:** `tick -5` and `tick abc` silently tick 1.
- **Done when:** Invalid tick counts produce an error message.
- **Risk:** low

### SIM-06: Seed light attenuation
- **Owner:** gameplay
- **Why:** Seeds are transparent to light. Minor exploit for stacking.
- **Done when:** Seeds attenuate light by ~5-10 per layer.
- **Risk:** low

---

## P3 — Future / expansion

### UNDERGROUND-01: Underground play expansion
- Spelunker + Vertical Farmer sessions prove the concept.

### GAME-05: Nutrient system
- Deferred until species diversity is broader.

### SIM-07: Horizontal light scatter
- Deferred. Underground gardening is future expansion.

### SIM-08: Horizontal water flow in air
- Deferred with SIM-07.

### GAME-07: Water containment / drainage
- Deferred. Water vanishing on stone edge case.

### GAME-08: Dark-loving species (mushrooms)
- Deferred until species system expands.

### CLI-18: Fill --dry-run preview
- Terraformer requested.

### CLI-19: Material properties reference
- `help materials` or `info stone`.

### CLI-20: Persistent water source / spring placement
- Underground farming needs self-sustaining water.

### SCALE-04: Further resolution increases
- Infrastructure supports any VOXEL_SIZE_M. 0.25m (240×240×120) would give flower-stem detail but needs performance validation.

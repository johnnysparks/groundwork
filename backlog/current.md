# Backlog — Groundwork

_Last updated: 2026-03-12T10:00:00 by Manager_

## TL;DR — Where We Are

**Done:** CLI usability (Sprint 1), seed growth (Sprint 2), placement validation + growth visibility + batch placement (Sprint 3), CLI-09 default Z=15, 12 player sessions across 2 rounds.

**Current focus (Sprint 4):** Visual richness + trust repair. Emoji rendering (user-directed), fill protection bypass fix (P0 from Round 2), checkerboard water fix (10/12 sessions). PR includes before/after text snapshots.

**The build can:** create worlds, place/fill materials (with ranges), protect seeds/roots from single `place` destruction, simulate water flow + light + seed growth with visible progress (s→S→*), show growth diagnostics, save/load state.

**The build can't yet:** render emoji, protect seeds/roots from `fill` destruction, fix checkerboard water, make roots interact with the world, show water depth, vary terrain, support multiple species, or let the TUI player interact with the world (TUI is view-only).

**New process rule:** CLI and TUI must ship player-facing features together. See `decisions/2026-03-12T14:00:00_interface_parity_and_focus_mechanism.md`.

**Key insight from Round 2:** Trust and beauty scores are sliding (3.5→3.3 trust, 3.0→2.8 beauty). The `fill` bypass undermines Sprint 3's protection work. The checkerboard is universal. Emoji rendering is the fastest path to visual delight. Fix trust + add richness = restore momentum.

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
- ~~GAME-03: Batch voxel placement~~ ✓ — Range syntax on `place` (`20..40`), `fill` command for rectangles.
- ~~CLI-09: Default view Z=15~~ ✓ — First command shows surface, not empty air.

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

---

## P0 — Blocks core proof or makes build unusable

### CLI-12: Fix `fill` seed/root protection bypass
- **Owner:** tools
- **Why:** `fill` silently overwrites seeds and roots with no check. 3/6 Round 2 sessions hit this independently (Destructor, Vertical Farmer, Storyteller). Completely undermines the CLI-08 protection system. The Storyteller accidentally drowned named seeds. The Destructor confirmed 100% bypass.
- **Done when:** `fill` skips Seed and Root voxels by default. Reports "N protected cells skipped." `fill --force` overrides.
- **Dependencies:** CLI-08 ✓
- **Risk:** low
- **Scope:** Trust repair — directly restores the protection promise from Sprint 3.

---

## P1 — Strongly improves clarity, feel, or core loop

### CLI-11: Emoji rendering for `view` and TUI (Sprint 4)
- **Owner:** tools
- **Why:** User-directed. Beauty/readability scores at 2.8/5 across Round 2. Emoji replaces ASCII with richer, more recognizable characters. Before/after snapshots required in PR.
- **Done when:** `view` uses emoji by default (💧🟫🪨🌿🌰🌱🟤). `view --ascii` fallback. TUI updated. Legend updated. Axis labels adjusted for 2-column width.
- **Dependencies:** none
- **Risk:** low (display only)
- **Scope:** Visual delight — highest delight-per-line ratio in the backlog.

### SIM-04: Fix checkerboard water artifact (Sprint 4)
- **Owner:** gameplay
- **Why:** 10/12 total sessions reported it. Water frontier shows `.~.~.~` alternating pattern. Hydrogeologist identified fix: water cells with water_level < 5 should revert to air.
- **Done when:** Water frontier is smooth after 100+ ticks. No water cells with water_level < 5.
- **Dependencies:** none
- **Risk:** low-medium
- **Scope:** Visual trust — the #1 simulation artifact.

### SIM-05: Lower wet-soil display threshold (Sprint 4)
- **Owner:** tools
- **Why:** Wet soil doesn't appear until water_level > 100. Too slow for cause-and-effect readability.
- **Done when:** Threshold lowered to 50 in cli.rs and render.rs.
- **Dependencies:** none
- **Risk:** low

### VIS-01: Dark indicator for underground air (Sprint 4)
- **Owner:** tools
- **Why:** Underground air with light_level=0 is indistinguishable from surface air. 3/12 sessions flagged.
- **Done when:** Dark air renders as blank space (not dot/emoji) in CLI and TUI.
- **Dependencies:** none
- **Risk:** low

### TUI-01: Cursor/focus system in TUI
- **Owner:** tools
- **Why:** TUI has no way to select or interact with individual cells. Human testers can only watch — they can't point at a cell to inspect it or start a placement. Blocks TUI-02 and TUI-03.
- **Done when:** Arrow keys move a visible cursor on the current Z-slice. Focused cell is highlighted. Cursor position shown in status bar.
- **Dependencies:** none
- **Risk:** low
- **Scope:** Foundation for all TUI interaction. See `decisions/2026-03-12T14:00:00_interface_parity_and_focus_mechanism.md`.

### TUI-02: Tool mode — range placement in TUI
- **Owner:** tools
- **Why:** Human testers need the same range-placement workflow that CLI agents use. A key starts a tool operation at the cursor; moving the cursor and pressing another key applies the operation to the range. Without this, TUI is view-only and player feedback misses all placement UX.
- **Done when:** Player can select a material, press a key to begin placement at cursor (tool-start), move cursor, press a key to apply (tool-end). Pending range is visually highlighted. Works for seed, soil, water, stone, air.
- **Dependencies:** TUI-01
- **Risk:** medium (first interactive editing in TUI)

### TUI-03: Inspect panel in TUI
- **Owner:** tools
- **Why:** CLI `inspect` shows detailed voxel state. Human testers have no equivalent — they can't see water level, light, growth progress, or dormancy reasons.
- **Done when:** Pressing a key on the focused cell shows voxel details (material, water, light, nutrient, growth status) in a panel or overlay.
- **Dependencies:** TUI-01
- **Risk:** low

### TUI-04: Status display in TUI
- **Owner:** tools
- **Why:** CLI `status` shows material counts and tick info. TUI only shows Z position and tick in the status bar.
- **Done when:** A key toggles a status panel showing material counts matching CLI `status` output.
- **Dependencies:** none
- **Risk:** low

### CLI-21: Focus command — persistent cursor for CLI
- **Owner:** tools
- **Why:** Interface parity with TUI cursor. Agents should navigate the world step-by-step like human testers, not teleport with explicit coordinates. Focus point persists in state file.
- **Done when:** `groundwork focus <x> <y> <z>` sets focus. `groundwork focus` prints current focus and voxel summary at that position. Focus persisted in state file. `inspect` with no args uses focus.
- **Dependencies:** none (state file format change — bump version)
- **Risk:** low-medium (state file format change)

### CLI-22: Tool start/end — two-step range operations via focus
- **Owner:** tools
- **Why:** Interface parity with TUI tool mode. Agents should experience the same two-step workflow as humans: set start point, set end point, apply. This ensures agent play-test feedback reflects real UX friction.
- **Done when:** `groundwork tool-start <material>` marks current focus as range start. `groundwork tool-end` applies material from start to current focus. Pending tool state persisted. Reports cells placed, protected, out-of-bounds. Old `place`/`fill` commands remain as power-user shortcuts.
- **Dependencies:** CLI-21
- **Risk:** low-medium (state file format change)

### SIM-03: Root water absorption (Sprint 5)
- **Owner:** gameplay
- **Why:** 9/12 sessions want roots to do something. Absorbing water creates the first ecological chain and resource pressure. The Patience Gardener's world went dead at tick 60. The Storyteller's story ended at root. This is the "second act."
- **Done when:** Root voxels drain water_level from adjacent Soil voxels (~4/tick). Root water_level increases. Wet soil near roots visibly dries.
- **Dependencies:** none
- **Risk:** low
- **Scope:** First ecological relationship — required for "self-sustaining" fantasy.

---

## P2 — Valuable but not required for MVP

### GAME-02: Varied default terrain
- **Owner:** gameplay
- **Why:** 6/12 sessions flagged flat terrain as dull first impression.
- **Done when:** Default terrain has elevation variation and stone outcrops.
- **Risk:** low

### GAME-04: Water depth visual
- **Owner:** tools
- **Why:** All water looks the same. Hydrogeologist and 3 other sessions want depth indication.
- **Done when:** Deep water uses a different emoji or character than shallow water.
- **Risk:** low

### CLI-13: Cross-section view
- **Owner:** tools
- **Why:** Vertical Farmer says it's blocking for vertical play. `view --cross-y 30` showing X horizontal, Z vertical.
- **Done when:** Cross-section renders a vertical slice through the grid.
- **Risk:** low-medium (new rendering mode)
- **Note:** Only one persona requested it, but it would transform vertical/underground play.

### CLI-14: Equilibrium detection
- **Owner:** tools
- **Why:** 2/6 Round 2 sessions (Patience, Storyteller) ticked into the void with no feedback. World reaches equilibrium silently.
- **Done when:** After 10+ ticks with no material changes, message suggests planting or placing water.
- **Risk:** low

### CLI-15: Named tick events
- **Owner:** tools
- **Why:** Storyteller wants "Seed at (27,30) took root!" in tick summaries. High narrative value.
- **Done when:** Tick summary names which seeds grew, which became roots.
- **Risk:** low

### SIM-06: Seed light attenuation
- **Owner:** gameplay
- **Why:** Seeds are transparent to light. Minor exploit for stacking.
- **Done when:** Seeds attenuate light by ~5-10 per layer.
- **Risk:** low

### CLI-10: Inspect growth diagnostics (PARTIALLY DONE)
- **Owner:** tools
- **Why:** Remaining: batch inspect, neighbor water details.
- **Done when:** `inspect` supports coordinate ranges.
- **Risk:** low

### CLI-16: `--force` flag position flexibility
- **Owner:** tools
- **Why:** Destructor found `--force` only works after coordinates. Confusing.
- **Done when:** `--force` accepted in any argument position.
- **Risk:** low

### CLI-17: Validate `tick` arguments
- **Owner:** tools
- **Why:** Destructor found `tick -5` and `tick abc` silently tick 1.
- **Done when:** Invalid tick counts produce an error message.
- **Risk:** low

---

## P3 — Future / expansion

### UNDERGROUND-01: Underground play expansion
- Spelunker + Vertical Farmer sessions prove the concept. Depends on SIM-07, SIM-08, GAME-08.
- Start with cross-section view (CLI-13) to make underground visible.

### GAME-05: Nutrient system
- Deferred until seed growth loop is proven fun.

### GAME-06: Multiple plant species
- 12-20 species is MVP target, but proving one species works comes first.

### SIM-07: Horizontal light scatter
- Deferred. Underground gardening is future expansion.

### SIM-08: Horizontal water flow in air
- Deferred with SIM-07.

### GAME-07: Water containment / drainage
- Deferred. Water vanishing on stone (Hydro session) is a related edge case.

### GAME-08: Dark-loving species (mushrooms)
- Deferred until species system expands.

### CLI-18: Fill --dry-run preview
- Terraformer requested. Would show what `fill` would change without committing.

### CLI-19: Material properties reference
- Terraformer requested `help materials` or `info stone` to show what each material does.

### CLI-20: Persistent water source / spring placement
- 2/6 Round 2 sessions (Vertical Farmer, Storyteller). Underground farming needs self-sustaining water.

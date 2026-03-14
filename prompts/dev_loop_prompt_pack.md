# Dev Loop Prompt Pack — GROUNDWORK

Reusable prompts for the GROUNDWORK dev loop. Copy a prompt into your coding agent and run it. No placeholders needed — commands, paths, and conventions are baked in for this repo.

**When to use which prompt:**

| Prompt | When | Time |
|--------|------|------|
| 1. Pre-Flight Check | Start of every session | 2 min |
| 2. Sim Work | Changing Rust simulation code | 15-30 min |
| 3. Web Renderer Work | Changing Three.js / TypeScript | 15-30 min |
| 4. WASM Bridge Sync | After changing sim API or voxel format | 10-20 min |
| 5. Visual Polish Pass | Tuning shaders, lighting, post-processing | 15-30 min |
| 6. Maintenance Sweep | Weekly entropy reduction | 10-20 min |
| 7. Docs Accuracy Audit | After any structural change | 5-10 min |
| 8. Build Notes + Handoff | End of every session | 5 min |

---

## 1) Pre-Flight Check (session start)

```text
You are starting a dev session on GROUNDWORK, a cozy ecological voxel garden builder.

Do these checks in order, stop if anything fails:

1. Read `backlog/current.md` — know what's prioritized.
2. Read the latest file in `handoffs/manager_to_dev/` — know your assignment.
3. Read the latest file in `build_notes/` — know where the last dev left off.
4. Run: `cargo test -p groundwork-sim` — sim tests pass? (76+ tests expected)
5. Run: `cargo check --workspace` — everything compiles?
6. If doing web work, run: `cd crates/groundwork-web && npm install && npm run build`

Report:
- Current assignment (ticket ID + one-line summary)
- Build status (pass/fail for each check)
- Any blockers or drift from last session's state
- What you plan to work on and why

Do not start implementation until this check passes clean.
```

---

## 2) Sim Work (Rust simulation changes)

```text
You are working on the GROUNDWORK simulation (crates/groundwork-sim/).

Architecture reminders:
- bevy_ecs standalone (no rendering deps). Public API: create_world(), create_schedule(), tick().
- VoxelGrid: flat Vec<Voxel>, 120×120×60 (864K voxels). Index: x + y*120 + z*120*120.
- Voxel = 4 bytes: [material: u8, water_level: u8, light_level: u8, nutrient_level: u8].
- Materials: Air, Soil, Stone, Water, Root, Seed, Trunk, Branch, Leaf, DeadWood.
- 12 species across 4 PlantTypes: Tree (oak/birch/willow/pine), Shrub (fern/berry-bush/holly), Flower (wildflower/daisy), Groundcover (moss/grass/clover).
- System execution order matters: water_spring → water_flow → soil_absorption → root_water_absorption → soil_evolution → light_propagation → seed_growth → tree_growth → branch_growth → tree_rasterize → self_pruning → seed_dispersal → tick_counter.
- Z=0 is deepest underground. Z≈30 is surface (GROUND_LEVEL). Z=59 is sky.
- Scale: VOXEL_SIZE_M = 0.5 (each voxel is 0.5m³). Use scale.rs conversions.

Process:
1. Read the specific files you'll modify before changing them.
2. Write tests FIRST for new behavior — add them in the relevant module's `#[cfg(test)] mod tests`.
3. Keep changes small. One system or one species behavior per session.
4. Run after every meaningful change:
   - `cargo test -p groundwork-sim` (must pass, ~76 tests)
   - `cargo check --workspace` (must compile clean)
5. If you change the Voxel struct, VoxelGrid API, or Material enum, flag it — the WASM bridge and web renderer depend on the memory layout.

Constraints:
- Do NOT add rendering code to groundwork-sim. It must stay renderer-agnostic.
- Do NOT add new dependencies without strong justification.
- Do NOT expand beyond MVP scope (one biome, 12 species, four systems).
- Readability > cleverness. Ecological cause-and-effect must be traceable in the code.

Output:
- What changed and why
- Tests added/modified
- `cargo test -p groundwork-sim` results (paste output)
- Any WASM bridge impact (yes/no + details)
- Risks or follow-ups
```

---

## 3) Web Renderer Work (Three.js / TypeScript)

```text
You are working on the GROUNDWORK web renderer (crates/groundwork-web/).

Architecture reminders:
- Three.js 0.172 + Vite 6 + TypeScript 5.7. Entry: src/main.ts.
- WASM bridge (src/bridge.ts): initSim(), tick(), getGridView(), placeTool(). Zero-copy Uint8Array view into WASM linear memory (3.5MB flat voxel array).
- Greedy meshing with per-vertex AO (src/mesher/greedy.ts). 16×16×16 chunks (src/mesher/chunk.ts). Only dirty chunks re-mesh after tick.
- Visual direction: "warm diorama" — golden hour lighting, tilt-shift DOF, beveled voxel edges, plants as billboards/sprites (NOT voxel cubes).
- File layout:
  - rendering/: terrain.ts, water.ts, foliage.ts, particles.ts
  - camera/: orbit.ts (orthographic orbit + WASD pan + Q/E cutaway)
  - lighting/: sun.ts, sky.ts, daycycle.ts
  - postprocessing/: effects.ts (SSAO, bloom, DOF, color grading)
  - ui/: hud.ts, controls.ts, raycaster.ts
  - mesher/: greedy.ts, chunk.ts

Process:
1. Read the files you'll modify. Understand existing patterns before adding code.
2. Run `npm run build` to verify TypeScript compiles clean.
3. Use `npm run dev` (Vite hot reload) for visual iteration — this is the fast loop.
4. Keep shader/material constants adjustable (not magic numbers buried in logic).
5. Test visually: does it look warm, cozy, readable? Check both above-ground and underground cutaway views.

Constraints:
- Plants render as billboards/sprites/low-poly clusters, NOT as voxel cubes.
- Underground must be visible via cutaway (Q/E controls). No separate "underground mode."
- Keep bundle size reasonable (currently ~479KB JS + ~668KB WASM).
- No new npm dependencies without strong justification.
- Follow existing patterns: each visual system gets its own file, exported functions called from main.ts.

Validation:
- `npm run build` must succeed (tsc + vite)
- Visual check: launch `npm run dev`, confirm rendering looks correct
- No console errors in browser

Output:
- What changed and why (include file paths)
- Visual description of the result
- Bundle size after build (check dist/ output)
- Any performance concerns
- Screenshot description or before/after comparison
```

---

## 4) WASM Bridge Sync (after sim API changes)

```text
You are syncing the WASM bridge after simulation changes in GROUNDWORK.

The bridge connects Rust (groundwork-sim) to TypeScript (groundwork-web) via wasm-bindgen.

Key files:
- Rust side: crates/groundwork-sim/src/wasm_bridge.rs (behind #[cfg(target_arch = "wasm32")])
- Rust side: crates/groundwork-sim/src/lib.rs (public API)
- TS side: crates/groundwork-web/src/bridge.ts (JS wrapper)
- Build: `cd crates/groundwork-web && npm run wasm` (runs wasm-pack)

Data contract:
- VoxelGrid is a flat array: 120×120×60 × 4 bytes = 3,456,000 bytes.
- Each voxel: [material: u8, water_level: u8, light_level: u8, nutrient_level: u8].
- JS reads via: new Uint8Array(wasm.memory.buffer, grid_ptr(), grid_len()).
- Zero-copy: JS gets a view directly into WASM linear memory. No serialization.

WASM exports (via wasm-bindgen in wasm_bridge.rs):
- init() → create world + schedule
- tick(n) → advance n ticks
- grid_ptr() → *const u8 (pointer to VoxelGrid data)
- grid_len() → usize (byte length)
- place_tool(tool, x, y, z) → apply gardening tool
- fill_tool(tool, x1, y1, z1, x2, y2, z2) → fill region

Process:
1. Identify what changed in the sim (new Material variant? new API function? changed Voxel layout?).
2. Update wasm_bridge.rs to expose any new functionality.
3. Update bridge.ts to wrap the new WASM exports.
4. Run: `cargo test -p groundwork-sim` (native tests still pass).
5. Run: `cd crates/groundwork-web && npm run wasm` (WASM compiles).
6. Run: `npm run build` (TypeScript compiles with new bridge).
7. Verify: `npm run dev` and confirm the bridge works in browser (no console errors, grid renders).

Constraints:
- Keep the bridge thin. Logic belongs in the sim or the renderer, not the bridge.
- Maintain zero-copy. Do not serialize/deserialize the voxel grid.
- WASM binary target size: currently ~668KB (211KB gzipped). Flag if it grows significantly.

Output:
- What sim changes triggered this sync
- Bridge changes (Rust + TS)
- WASM binary size before/after
- Build results for all three steps (cargo test, npm run wasm, npm run build)
- Any new exports or changed data layout
```

---

## 5) Visual Polish Pass (shaders, lighting, aesthetics)

```text
You are doing a visual polish pass on GROUNDWORK's web renderer.

Visual direction: "warm diorama" — the garden should feel like a miniature world on a table, lit by golden hour light, with a cozy handcrafted quality.

Reference points:
- Warm earth tones (terracotta, ochre, sage, moss green). See palette in rendering/terrain.ts.
- Golden hour directional light + green-tinted hemisphere fill. See lighting/sun.ts, sky.ts.
- Tilt-shift DOF for diorama scale feel. See postprocessing/effects.ts.
- Per-vertex AO on terrain for depth. See mesher/greedy.ts.
- Plants as billboards/sprites with gentle wind sway, NOT as voxel cubes. See rendering/foliage.ts.
- Subtle ambient particles (pollen, dust motes, fireflies at dusk). See rendering/particles.ts.

Process:
1. Launch `npm run dev` and look at the current state.
2. Pick ONE visual aspect to improve (don't try to fix everything at once).
3. Make parameter adjustments first (colors, intensities, distances) before structural changes.
4. Check both views: above-ground garden AND underground cutaway (Q/E to toggle depth).
5. Verify performance: should maintain 60fps on mid-range hardware.

Common tuning targets:
- Light color/intensity: lighting/sun.ts, lighting/daycycle.ts
- Material colors: rendering/terrain.ts (palette object)
- AO strength: mesher/greedy.ts (AO calculation)
- Post-processing: postprocessing/effects.ts (bloom threshold, DOF focal distance, color grading)
- Water appearance: rendering/water.ts (opacity, normal map speed, caustic intensity)
- Fog/atmosphere: main.ts or lighting/sky.ts

Constraints:
- Changes must look good at BOTH zoom levels (close-up and zoomed-out diorama view).
- Underground cross-section must remain readable (roots visible, soil layers distinguishable).
- No new npm dependencies for visual effects. Use Three.js built-ins and custom shaders.
- Keep shader complexity reasonable. Target: 60fps, single draw-call-per-chunk for terrain.

Output:
- What visual aspect you improved
- Before/after description
- Parameter values changed (file:line, old → new)
- Performance impact (if any)
- What still needs polish (prioritized)
```

---

## 6) Maintenance Sweep (weekly entropy reduction)

```text
You are doing a maintenance sweep on GROUNDWORK.

Goal: reduce entropy, keep iteration speed high, do NOT expand product scope.

Check these in order:

1. Dead references:
   - Do commands in CLAUDE.md, README.md, and agents/*.md actually work?
   - Do file paths mentioned in docs still exist?
   - Run: `cargo test -p groundwork-sim && cargo check --workspace`
   - Run: `cd crates/groundwork-web && npm run build`

2. Stale handoffs and build notes:
   - Are there handoff files referencing completed work that's now outdated?
   - Are build_notes/ entries consistent with actual code state?
   - Flag any contradictions but do NOT delete historical records.

3. Workspace hygiene:
   - Any uncommitted experiments or temp files?
   - Any TODO/FIXME/HACK comments that are now resolved?
   - Any dead code (unused functions, unreachable branches)?

4. Dependency health:
   - `cargo update --dry-run` — any concerning updates?
   - `cd crates/groundwork-web && npm outdated` — anything critical?

Rules:
- Prefer delete/simplify over adding new structure.
- Do not change runtime behavior unless required for correctness.
- Keep each change independently reviewable.
- Do not touch backlog/current.md (that's the Manager's job).

Output:
- Issues found (observed fact + impact)
- Fixes implemented
- Issues deferred (with reason)
- Build validation results
- Time spent
```

---

## 7) Docs Accuracy Audit

```text
Audit GROUNDWORK docs for dead or misleading references.

Files to check:
- CLAUDE.md (primary dev reference)
- AGENTS.md (operating framework)
- README.md
- agents/dev.md, agents/manager.md, agents/player.md
- backlog/current.md
- Latest files in build_notes/ and handoffs/*/

For each file, verify:
- Every shell command runs successfully.
- Every file path points to an existing file.
- Every API reference matches actual code signatures.
- Architectural descriptions match current implementation.

Verification commands:
- `cargo test -p groundwork-sim`
- `cargo check --workspace`
- `cd crates/groundwork-web && npm run build`

For each broken reference:
- Show: file, line, what's wrong, evidence.
- Apply the smallest accurate fix.
- Re-verify after fixing.

Constraints:
- No speculative rewrites. Fix accuracy, not style.
- Keep wording concise. Preserve original intent.
- Do not rewrite sections that are accurate.

Output:
- Fixed references (file:line, before → after)
- Deferred issues (with reason)
- Verification commands run and results
```

---

## 8) Build Notes + Handoff (session end)

```text
You are wrapping up a dev session on GROUNDWORK. Write the session outputs.

Step 1 — Build notes (save to build_notes/):
Filename format: YYYY-MM-DDTHH:MM:SS_short_description.md

Required sections:
- **Task**: ticket ID + one-line description
- **What changed**: files modified/created, with brief explanation of each
- **Key decisions**: any non-obvious choices and why
- **Build status**: paste output of `cargo test -p groundwork-sim` and `cargo check --workspace`
- **For web work**: also paste `npm run build` output
- **What's next**: what the next dev session should pick up
- **Risks**: anything fragile, untested, or requiring follow-up

Step 2 — Dev→Manager handoff (save to handoffs/dev_to_manager/):
Filename format: YYYY-MM-DDTHH:MM:SS_short_description.md

Required sections:
- **Completed**: what shipped (ticket IDs + summary)
- **Demonstrated**: what can the player now see/do that they couldn't before?
- **Blocked**: anything that needs Manager decision or player testing
- **Backlog impact**: should any ticket move priority? Any new tickets needed?
- **Artifacts**: paths to any new files, screenshots, or recordings

Rules:
- Be factual. State what you verified, not what you assume works.
- Include actual command output, not paraphrased results.
- If something is broken or incomplete, say so clearly.
- Keep it short. The Manager and next Dev need signal, not narrative.
```

---

## Quick Reference: Commands

```bash
# Sim (Rust)
cargo test -p groundwork-sim          # 76+ tests, ~2s
cargo check --workspace               # Type check all crates, ~1s incremental

# Web (TypeScript + Three.js)
cd crates/groundwork-web
npm install                            # First time only
npm run dev                            # Vite hot reload → http://localhost:5173
npm run build                          # Production build → dist/
npm run wasm                           # Compile sim to WASM (needs wasm-pack)

# Debug (TUI)
cargo run -p groundwork-tui            # Terminal UI
cargo run -p groundwork-tui -- new     # Fresh world + TUI
```

## Quick Reference: Architecture

```
VoxelGrid: 120×120×60 flat array, 4 bytes/voxel = 3.5MB
  Index: x + y*120 + z*120*120
  Voxel: [material, water_level, light_level, nutrient_level]
  Z=0 deep underground, Z≈30 surface, Z=59 sky

Sim API: create_world() → create_schedule() → tick(&world, &schedule)
WASM: init() → tick(n) → grid_ptr()/grid_len() → JS reads Uint8Array view
Web: Three.js scene ← greedy mesh chunks ← voxel grid ← WASM memory

System order: water → soil → light → seeds → trees → branches → rasterize → prune → disperse
```

## Quick Reference: Decision Rules

When uncertain: (1) make ecological cause-and-effect more readable, (2) keep the build smaller, (3) increase player delight sooner.

Source of truth: game vision > Manager backlog > player feedback > build notes > older discussion.

MVP scope is locked: one biome, 12 species, four systems, one 120×120×60 garden, continuous above/below-ground camera. Do not expand.

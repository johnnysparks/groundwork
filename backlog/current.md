# Backlog — Groundwork

_Last updated: 2026-03-13 by Manager_

## TL;DR — Where We Are

**Phase shift: core game dev mode.** The simulation foundation is complete. Primary workstream is now the Three.js web renderer and WASM bridge — making the game beautiful and playable in the browser.

**What's built:**
- Sim: 12 species (4 trees, 3 shrubs, 2 flowers, 3 groundcover), water/light/soil/root systems, procedural trees via space colonization, soil composition model, seed dispersal
- Web scaffold: Vite + Three.js + TypeScript, greedy meshing with per-vertex AO, chunk dirty tracking, orbit camera, golden hour lighting, mock data mode
- TUI: 2D + 3D views, quest/onboarding system (20 quests, 9 chapters), species selection, all tools
- Profiler: flamegraph-based sim profiling, optimized systems.rs

**Grid:** 120×120×60 voxels at 0.5m/voxel = 60m×60m×30m. GROUND_LEVEL=30.

**Primary interface:** Web UI (Three.js). TUI/CLI are dev/debug tools.

---

## Completed

### Sim Foundation (Sprints 1-5)
- ✓ CLI usability (7 tasks)
- ✓ Core fixes + seed growth (3 tasks)
- ✓ Trust & readability (4 tasks)
- ✓ Ecological depth: soil composition, procedural trees, root systems (7 tasks)
- ✓ Scale normalization: 0.5m voxels, meters-based dimensions (3 tasks)

### Player Validation
- ✓ 2 rounds of player agent validation (12 sessions)

### Interface & Species
- ✓ Interface parity sprint (TUI-01-04, CLI-21-22)
- ✓ GAME-06: 12 species across 4 plant types
- ✓ Species scaling: canopy trees 20-28 voxels, diorama proportions
- ✓ Quest/mission log onboarding (20 quests, 9 chapters)

### Web Scaffold
- ✓ Vite + Three.js project setup with WASM plugin
- ✓ Greedy meshing with per-vertex ambient occlusion
- ✓ 16×16×16 chunk system with dirty tracking
- ✓ Orbit camera (orthographic, smooth damping)
- ✓ Lighting (golden hour sun, hemisphere fill, blue-purple ambient)
- ✓ Terrain renderer (warm material palette, AO darkening)
- ✓ Mock data mode (terrain, water pool, hill, tree)
- ✓ WASM bridge design (bridge.ts with zero-copy typed arrays)

### Performance
- ✓ Simulation profiling + optimization

---

## P0 — Blocks core proof

### WEB-01: WASM bridge — connect sim to browser
Add `wasm-bindgen` exports to `groundwork-sim`. Compile to `cdylib`. Wire up `bridge.ts` to real sim data instead of mock grid.
- Add `crate-type = ["rlib", "cdylib"]` to sim Cargo.toml
- Export: `init()`, `tick(n)`, `grid_ptr()`, `grid_len()`, `place_tool()`, `get_tick()`
- `#[cfg(target_arch = "wasm32")]` guard on all wasm-bindgen code
- Install wasm-pack, verify `npm run wasm` builds
- **Done when:** `npm run dev` renders the real sim grid in browser, not mock data

### WEB-02: Tool interaction — click to place
Raycasting from mouse click to voxel grid. Select tools, place materials.
- Three.js Raycaster → world coords → nearest voxel
- Tool palette UI (HTML overlay or Three.js sprites)
- Species picker for seed bag
- Call `place_tool()` via WASM bridge
- **Done when:** can plant seeds, pour water, dig with shovel by clicking in browser

---

## P1 — Strongly improves clarity, feel, or core loop

### WEB-03: Camera controls
- WASD/arrow fly, scroll zoom, right-drag orbit
- Smooth damped transitions (ease-out on everything)
- Underground cutaway: slide depth plane to see below surface
- Reset view hotkey
- **Done when:** can freely navigate above and below ground in browser

### WEB-04: HUD and status panel
- Tick counter + auto-tick toggle
- Tool selection with keyboard shortcuts
- Inspect panel (hover/click a voxel to see material, water, light, nutrients)
- Species display when seed bag selected
- **Done when:** essential game info visible without guessing

### WEB-05: Water surface shader
- Separate flat mesh for water surface (not voxel cubes)
- Scrolling normal maps for subtle ripples
- Depth-based opacity (shallow = clear, deep = opaque)
- **Done when:** water looks inviting, not like blue cubes

### WEB-06: Post-processing basics
- SSAO for depth perception and contact shadows
- Subtle bloom on sunlit edges
- Warm color grading
- **Done when:** scene has visual depth and warmth

### GAME-04: Water depth visual
Different rendering for water at different depths. Low effort, high readability.

---

## P2 — Valuable but not required for MVP

### WEB-07: Foliage rendering
Replace leaf/flower voxel cubes with billboard sprites or low-poly mesh clusters. Species-specific appearances.

### WEB-08: Wind sway animation
Gentle vertex displacement on vegetation (sine wave, offset by world position).

### WEB-09: Growth particles
Green/golden particle burst when a plant advances a growth stage.

### WEB-10: Time-of-day cycle
Dawn, noon, golden hour, blue hour. Lerp between 4 light presets.

### WEB-11: Tilt-shift depth of field
The signature diorama effect. Makes the garden feel like a miniature model.

### CLI-12: Fix `fill` seed/root protection bypass
Skips protected cells instead of blocking entire operation.

### SIM-06: Seed light attenuation
Seeds should be transparent to light propagation.

---

## P3 — Future/expansion

- Underground-01: Biome expansion (auto-P3)
- SIM-07: Nutrient cycling system
- WEB-12: Ambient creatures (butterflies at high biodiversity)
- WEB-13: Sound design / adaptive audio
- WEB-14: Mobile touch controls
- SCALE-04: Larger world / streaming chunks
- Multi-platform: Tauri desktop wrapper, native wgpu renderer

# Backlog — Groundwork

_Last updated: 2026-03-14 by Maintenance sweep_

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

### Web Features (Sprint 6)
- ✓ WEB-01: WASM bridge — sim connected to browser
- ✓ WEB-02: Tool interaction — raycasting, tool palette, species picker
- ✓ WEB-03: Camera controls — fly, zoom, orbit, underground cutaway
- ✓ WEB-04: HUD and status panel
- ✓ WEB-05: Water surface renderer
- ✓ WEB-06: Post-processing (SSAO, bloom, color grading)
- ✓ WEB-07: Foliage billboard sprites
- ✓ WEB-08: Wind sway animation
- ✓ WEB-09: Growth particles

### Performance
- ✓ Simulation profiling + optimization

---

## P1 — Strongly improves clarity, feel, or core loop

### GAME-04: Water depth visual
Different rendering for water at different depths. Low effort, high readability.

---

## P2 — Valuable but not required for MVP

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

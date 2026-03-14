# Parallel Dev Roadmap — Groundwork Web Engine

**Date:** 2026-03-13
**Status:** Active

## Key Insight

Everything except the WASM bridge and final integration develops against **mock grid data**. The existing mock data system (`greedy.ts:generateMockGrid()`) means any number of rendering workstreams can execute simultaneously without waiting on Rust/WASM work.

---

## Workstream Map

```
                    ┌─────────────────────────────────────────────────┐
                    │              INDEPENDENT WORKSTREAMS             │
                    │         (all develop against mock grid)          │
                    └─────────────────────────────────────────────────┘

  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
  │ B: Water │  │ C: Post  │  │ D: Cam   │  │ E: HUD   │  │ F: Foliage│
  │  Shader  │  │  Process │  │ Controls │  │  & UI    │  │ & Anim   │
  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘
       │              │              │              │              │
       │              │              │              │              │
  ┌────┴──────────────┴──────────────┴──────────────┴──────────────┴────┐
  │                    EXISTING SCAFFOLD (done)                         │
  │  Vite + Three.js + greedy meshing + chunks + orbit cam + lighting   │
  └────────────────────────────┬───────────────────────────────────────┘
                               │
                        ┌──────┴──────┐
                        │ INTEGRATION │ ← merges A + renderer
                        └──────┬──────┘
                               │
  ┌──────────┐          ┌──────┴──────┐          ┌──────────┐
  │ A: WASM  │──────────│   PLAYABLE  │          │ H: Sim   │
  │  Bridge  │          │   IN BROWSER│          │ Enhance  │
  └──────────┘          └─────────────┘          └──────────┘
                                                      │
                                               (independent always)
```

---

## Workstream A: WASM Bridge (Rust)

**Depends on:** Nothing
**Blocks:** Integration
**Files:** `crates/groundwork-sim/Cargo.toml`, `crates/groundwork-sim/src/wasm.rs` (new)

### Tasks

| ID | Task | Detail | Done when |
|----|------|--------|-----------|
| A1 | Add cdylib target | `crate-type = ["rlib", "cdylib"]` in Cargo.toml | `wasm-pack build` succeeds |
| A2 | Add wasm-bindgen deps | Behind `cfg(target_arch = "wasm32")` | `cargo check --workspace` still passes |
| A3 | Export `init()` | Create world + schedule, store in `thread_local!` or `static` | Returns without panic |
| A4 | Export `tick(n)` | Advance n ticks on stored world | Tick counter increments |
| A5 | Export `grid_ptr()` + `grid_len()` | Return raw pointer to VoxelGrid cells | JS can create `Uint8Array` view |
| A6 | Export `soil_ptr()` + `soil_len()` | Return pointer to SoilGrid data | JS can read soil composition |
| A7 | Export `place_tool(tool, x, y, z)` | Apply tool via sim API | Seeds/water placed from JS |
| A8 | Export `get_tick()` | Return current tick count | Matches after tick(n) calls |
| A9 | Export `grid_dimensions()` | Return (GRID_X, GRID_Y, GRID_Z) | JS uses real grid size |
| A10 | wasm-pack build script | `npm run wasm` in groundwork-web | Produces pkg/ with .wasm + .js glue |
| A11 | console_error_panic_hook | Better panic messages in browser | Stack traces in devtools |

**Validation:** `npm run wasm && node -e "const m = require('./pkg'); m.init(); m.tick(1); console.log(m.get_tick())"`

---

## Workstream B: Water Surface Shader (TypeScript)

**Depends on:** Scaffold (done)
**Blocks:** Nothing (composable)
**Files:** `crates/groundwork-web/src/rendering/water.ts` (new)

### Tasks

| ID | Task | Detail | Done when |
|----|------|--------|-----------|
| B1 | Water surface detector | Scan grid for water voxels with air above, emit flat quads | Blue surface appears over mock water pool |
| B2 | Scrolling normal maps | Two UV-offset normal maps for ripple effect | Surface shimmers without interaction |
| B3 | Depth-based opacity | Sample water depth below surface, lerp alpha | Shallow = clear, deep = opaque |
| B4 | Fresnel reflection | Angle-dependent sky color reflection | Surface reflects at glancing angles |
| B5 | Interaction ripples | Ring wave from a world-space point, decays over time | Drop a ripple at coords, see expanding ring |

**Mock data:** Existing mock grid already has a water pool. Extend if needed.

---

## Workstream C: Post-Processing (TypeScript)

**Depends on:** Scaffold (done)
**Blocks:** Nothing (each pass is independent)
**Files:** `crates/groundwork-web/src/postprocess/` (new dir)

Each pass is a separate, independent task. Can be built and tested in isolation.

### Tasks

| ID | Task | Detail | Done when |
|----|------|--------|-----------|
| C1 | EffectComposer setup | Wire Three.js EffectComposer into render loop | Passthrough render unchanged |
| C2 | SSAO pass | Screen-space AO for contact shadows | Corners and crevices darken subtly |
| C3 | Bloom pass | High-threshold, soft radius bloom on sunlit edges | Bright edges glow without washing out |
| C4 | Tilt-shift DOF | Horizontal focal band, blur above and below | Scene looks like a miniature model |
| C5 | Color grading | Warm LUT: lifted shadows, warm midtones, desat highlights | Scene feels cozy, not clinical |
| C6 | Vignette | Subtle (10-15%) darkening at screen edges | Focus drawn inward |

**Note:** C2-C6 each work independently once C1 is done. Maximum parallelism within this workstream.

---

## Workstream D: Camera Enhancements (TypeScript)

**Depends on:** Scaffold (done — orbit camera exists)
**Blocks:** Nothing
**Files:** `crates/groundwork-web/src/camera/orbit.ts` (modify), `cutaway.ts` (new), `transitions.ts` (new)

### Tasks

| ID | Task | Detail | Done when |
|----|------|--------|-----------|
| D1 | WASD/arrow fly | Translate camera target with keyboard | Smooth pan in all 4 directions |
| D2 | Scroll zoom | Orthographic zoom via scroll wheel | Zoom range feels natural |
| D3 | Right-drag orbit | Existing orbit + refinement | Smooth, heavily damped |
| D4 | Underground cutaway | Slide a clip plane through Y-axis to reveal underground | Can see roots and soil layers |
| D5 | Reset view hotkey | Snap back to default position/rotation | Press R (or similar), camera resets |
| D6 | Smooth transitions | Ease-out on all camera movements | Camera floats, never snaps |

---

## Workstream E: HUD & UI (HTML/CSS/TypeScript)

**Depends on:** Scaffold (done)
**Blocks:** Integration (tool selection needed for click-to-place)
**Files:** `crates/groundwork-web/src/ui/` (new dir), `index.html` (modify)

### Tasks

| ID | Task | Detail | Done when |
|----|------|--------|-----------|
| E1 | Tool palette | HTML overlay: soil, seed, water, stone, shovel buttons | Click to select active tool |
| E2 | Species picker | Dropdown/grid when seed bag selected | Can choose oak, birch, etc. |
| E3 | Tick counter | Display current tick, auto-tick toggle (Space) | Tick number visible, can pause/play |
| E4 | Inspect panel | Hover/click voxel → show material, water, light, nutrients | Panel updates on hover |
| E5 | Keyboard shortcuts | 1-5 for tools, Space for tick, R for reset | Shortcuts work |
| E6 | Raycaster → voxel | Three.js Raycaster from mouse → world coords → grid cell | Console logs correct voxel coords on click |
| E7 | Click-to-place | Use active tool at raycasted voxel position | Can dig, place water, plant seeds by clicking |

**Note:** E6-E7 work against mock grid initially. Integration connects them to real sim via WASM.

---

## Workstream F: Foliage & Animation (TypeScript)

**Depends on:** Mesher (done — greedy meshing exists)
**Blocks:** Nothing
**Files:** `crates/groundwork-web/src/rendering/foliage.ts` (new), `particles.ts` (new)

### Tasks

| ID | Task | Detail | Done when |
|----|------|--------|-----------|
| F1 | Leaf billboard sprites | Replace leaf-material cubes with billboard quads | Leaves look organic, not blocky |
| F2 | Species-specific foliage | Oak=round canopy, pine=conical, willow=drooping | 4 tree species visually distinct |
| F3 | Wind sway | Sine-wave vertex displacement, offset by position | Gentle constant sway on vegetation |
| F4 | Growth particles | Green/golden burst on growth stage change | Sparkle when plant grows |
| F5 | Flower/grass sprites | Small billboard sprites for groundcover | Wildflowers and grass look delicate |
| F6 | Falling leaves | Particle system for tree death/stress | Leaves drift down when tree is stressed |

---

## Workstream G: Lighting Enhancements (TypeScript)

**Depends on:** Scaffold (done — golden hour lighting exists)
**Blocks:** Nothing
**Files:** `crates/groundwork-web/src/lighting/time-of-day.ts` (new), `sun.ts` (modify)

### Tasks

| ID | Task | Detail | Done when |
|----|------|--------|-----------|
| G1 | Shadow maps | Soft PCF shadows from directional sun | Objects cast warm soft shadows |
| G2 | Shadow color tint | Shadows are blue-purple, never black | Shadows feel cozy |
| G3 | Time-of-day presets | Dawn, noon, golden hour, blue hour configs | 4 distinct moods |
| G4 | Time-of-day lerp | Smooth interpolation between presets | Sun slides through sky, colors shift |
| G5 | Sky gradient | Background gradient shifts with time-of-day | No more flat background color |

---

## Workstream H: Sim Enhancements (Rust)

**Depends on:** Nothing (independent always)
**Blocks:** Nothing
**Files:** `crates/groundwork-sim/src/`

### Tasks

| ID | Task | Detail | Done when |
|----|------|--------|-----------|
| H1 | Seed light attenuation | Seeds transparent to light propagation | Seeds don't create shadow columns |
| H2 | Fill seed/root protection | Skip protected cells instead of blocking | Fill works around existing plants |
| H3 | Nutrient cycling | Organic decomposition → nutrients → plant uptake | Visible nutrient flow in inspect |

**Note:** These are P2/P3 from the backlog. Work on them when other workstreams are blocked or for variety.

---

## Integration Phase

**Depends on:** A (WASM bridge) + scaffold
**Unblocks:** Full playable loop

### Tasks

| ID | Task | Detail | Done when |
|----|------|--------|-----------|
| I1 | Wire bridge.ts to real WASM | Replace mock `initSim()` with real wasm-pack output | `init()` creates real sim world |
| I2 | Grid refresh after tick | Read real grid_ptr() into typed array, diff for dirty chunks | Real terrain renders after tick |
| I3 | Tool → WASM | E7's click-to-place calls `place_tool()` through bridge | Clicking plants real seeds |
| I4 | Tick loop | Connect auto-tick toggle to real `tick(n)` calls | Sim advances, water flows, plants grow |
| I5 | Soil grid reading | Read soil_ptr() for underground coloring | Soil composition affects visual |

---

## Dependency Graph (What Can Run In Parallel)

```
IMMEDIATELY (all independent, all against mock data):
  ├── A: WASM Bridge (Rust)
  ├── B: Water Shader
  ├── C1: EffectComposer setup
  ├── D1-D3: Camera fly/zoom/orbit
  ├── E1-E5: Tool palette, species picker, HUD
  ├── F1-F3: Foliage billboards, wind sway
  ├── G1-G2: Shadows + shadow color
  └── H1-H3: Sim enhancements

AFTER C1:
  ├── C2: SSAO
  ├── C3: Bloom
  ├── C4: Tilt-shift DOF
  ├── C5: Color grading
  └── C6: Vignette

AFTER E6 (raycaster):
  └── E7: Click-to-place (mock)

AFTER D1-D3 (basic camera):
  ├── D4: Underground cutaway
  ├── D5: Reset view
  └── D6: Smooth transitions

AFTER G1-G2 (shadows):
  ├── G3: Time-of-day presets
  ├── G4: Time-of-day lerp
  └── G5: Sky gradient

AFTER A (WASM bridge):
  └── I1-I5: Integration (connects everything to real sim)

AFTER F1 (leaf billboards):
  ├── F2: Species-specific foliage
  ├── F4: Growth particles
  ├── F5: Flower sprites
  └── F6: Falling leaves
```

**Maximum concurrent workstreams at kickoff: 8**
**Maximum concurrent tasks at kickoff: 15+**

---

## Critical Path to Playable

The shortest path from now to "playing real sim in browser":

```
A: WASM Bridge ──────────────────┐
  (A1-A11, ~2-3 days)            │
                                 ├──► I: Integration ──► PLAYABLE
E: Raycaster + Click-to-place ───┘     (I1-I5, ~1-2 days)
  (E6-E7, ~1 day)
```

**Everything else is beauty, polish, and feel** — important but not on the critical path. This means:
- One dev can focus exclusively on the WASM bridge + integration
- Every other dev works on making the renderer more beautiful in parallel
- When the bridge lands, integration is a 1-2 day merge

---

## Sprint Packaging Suggestion

For agent-based parallel dev sessions, package workstreams as independent assignments:

| Assignment | Tasks | Prereqs | Est. Effort |
|-----------|-------|---------|-------------|
| **"WASM Bridge"** | A1-A11 | None | 1 session |
| **"Water Beauty"** | B1-B5 | None | 1 session |
| **"Post-Processing Pipeline"** | C1-C6 | None | 1 session |
| **"Camera Freedom"** | D1-D6 | None | 1 session |
| **"Player Controls"** | E1-E7 | None | 1 session |
| **"Living Foliage"** | F1-F6 | None | 1 session |
| **"Light & Mood"** | G1-G5 | None | 1 session |
| **"Sim Polish"** | H1-H3 | None | 1 session |
| **"Integration"** | I1-I5 | A done | 1 session |

**9 assignments. 8 can start immediately in parallel. 1 waits for the bridge.**

---

## What's Already Done

From the existing scaffold in `crates/groundwork-web/`:

- [x] Vite + Three.js + TypeScript project
- [x] Greedy meshing with per-vertex AO
- [x] 16×16×16 chunk system with dirty tracking
- [x] Orthographic orbit camera with smooth damping
- [x] Golden hour directional + hemisphere + ambient lighting
- [x] Warm material color palette with AO darkening
- [x] Mock data mode (terrain, water pool, hill, tree)
- [x] Bridge.ts API design (mock fallback mode)
- [x] Render loop with auto-tick toggle (Space)
- [x] Mouse drag orbit + scroll zoom (basic)

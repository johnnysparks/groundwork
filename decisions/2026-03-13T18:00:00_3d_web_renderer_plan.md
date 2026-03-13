# Decision: 3D Web Renderer — Architecture & Visual Direction

**Date:** 2026-03-13
**Decision by:** Project lead
**Status:** Proposed — pending approval

## Goal

Build a beautiful 3D renderer for Groundwork, playable in the browser. The game should feel like a living miniature world on your desk, lit by warm afternoon light. Beauty is the north star. TUI/CLI continues to work for agent playtesting.

---

## The Three Viable Approaches

We evaluated every credible option (Bevy full, raw wgpu, Three.js+WASM, Fyrox, Godot+gdext, Macroquad, Unity/Unreal, Ambient, rend3, Kajiya). Most are dead, incompatible with our bevy_ecs sim, or wrong tool. Three contenders survived:

### Option A: Bevy Full Engine (Rust-only)

Share the bevy_ecs `World` directly — sim becomes a Bevy plugin. Get PBR pipeline, shadows, bloom, SSAO, tonemapping for free.

| Factor | Assessment |
|--------|-----------|
| ECS integration | **Seamless** — share World, zero bridge cost |
| Visual ceiling | Very high (PBR, Solari raytracing on desktop) |
| Web readiness | **Experimental** — WebGL2 or WebGPU (can't auto-detect), Chrome-only WebGPU, known GPU memory leak (#22545) |
| Bundle size | **15-22MB** WASM (7-8MB gzipped) |
| Compile time | **3-5 min** cold (vs current 60s), 2-3s incremental |
| Browser support | WebGL2: broad. WebGPU: Chrome desktop only as of early 2026 |
| API stability | Pre-1.0, breaking changes every release (~3 months) |
| Voxel ecosystem | Good — bevy_voxel_world, bevy_meshem, block-mesh-rs |

**Best if:** You want all-Rust and plan a native desktop client. Weakest web story.

### Option B: Raw wgpu + bevy_ecs (Rust-only)

Keep bevy_ecs for sim, use wgpu directly for rendering. Build camera, lighting, shadows, post-processing from scratch.

| Factor | Assessment |
|--------|-----------|
| ECS integration | Good — read VoxelGrid resource, render separately |
| Visual ceiling | As high as you build it |
| Web readiness | **Good** — WebGPU in all major browsers (Chrome 113+, Firefox 141+, Safari 26+) |
| Bundle size | **Small** — wgpu WASM is much smaller than Bevy |
| Compile time | **60-90s** cold for renderer crate |
| Build effort | **3-5 weeks** for a competent graphics programmer |
| Reference projects | Rezcraft (wgpu+WASM+voxels, works end-to-end) |

**Best if:** You want all-Rust with full rendering control and don't mind building the pipeline.

### Option C: Three.js + WASM Hybrid (Recommended)

Compile sim to WASM. Three.js reads the voxel grid via zero-copy typed array views into WASM memory. JS/TS handles all rendering.

| Factor | Assessment |
|--------|-----------|
| ECS integration | Good — expose grid pointer via wasm-bindgen, JS reads directly |
| Visual ceiling | Very high — mature shader ecosystem, proven cozy/stylized techniques |
| Web readiness | **Production-grade** — Three.js has shipped web 3D for 15 years |
| Bundle size | **~150KB** Three.js (gzipped) + small sim WASM (no renderer in Rust) |
| Compile time | Rust sim: 60s cold. JS: instant hot reload for visual iteration |
| Browser support | **All browsers** — WebGPU with automatic WebGL2 fallback since Three.js r171 |
| Ecosystem | Massive — water shaders, AO, bloom, DOF, voxel meshing libraries, 1000s of examples |
| Proven architecture | **Voxelize** = Rust backend + Three.js frontend, shipped multiplayer voxel game |
| Visual iteration speed | **Hot reload** — change shaders/materials/lighting without recompiling Rust |

**Best if:** Web is the primary target and you want the fastest path to beauty.

---

## Recommendation: Option C — Three.js + WASM

### Why

1. **Web is the target.** Three.js is the most mature, battle-tested web 3D engine. It works everywhere today, not experimentally.

2. **Beauty needs fast visual iteration.** Shaders, lighting, materials, colors — these need rapid tweaking. Three.js hot reloads in milliseconds. Bevy/wgpu require recompilation for every visual change.

3. **The cozy aesthetic fights PBR defaults.** Bevy's pipeline is built for physically-based rendering. Our game wants warm, stylized, imperfect lighting — hand-tuned, not physically accurate. Three.js gives us full control without fighting a PBR pipeline.

4. **The architecture already assumes this.** The original engine decision (2026-03-11) specifically planned for `groundwork-web` as JS + Three.js + WASM. The sim is renderer-agnostic by design. We're executing the plan we already made.

5. **Zero-copy data path.** The VoxelGrid is a flat `Vec<Voxel>` (864K × 4 bytes = 3.5MB). JS reads it directly from WASM linear memory as a `Uint8Array` — no serialization, no copies.

6. **Voxelize proves it works.** A production Rust+Three.js voxel game already exists with this exact architecture.

### What we give up vs Bevy

- No shared ECS World — we cross a WASM boundary (but it's zero-copy for the grid)
- Two languages (Rust + TypeScript) and two build pipelines
- Debugging across the WASM boundary is harder (mitigated by `console_error_panic_hook`)

### What we gain

- 100x smaller bundle (150KB vs 15MB)
- Hot reload for all visual work
- All browsers today, not experimentally
- The richest 3D web ecosystem in existence
- Proven voxel game architecture (Voxelize)

---

## Visual Direction: "Warm Diorama"

The garden should feel like a living miniature world sitting on your desk, lit by warm afternoon light from a nearby window, where tiny things are always gently moving.

### Rendering Style: Hybrid Voxel

Not pure cubes, not smooth terrain. A middle ground:

- **Underground (soil, stone, water):** Crisp voxel grid with greedy meshing. Readability is king for root systems, soil layers, water flow. Per-vertex color variation (subtle noise on browns/grays).
- **Surface terrain:** Slightly beveled voxels with vertex color blending at material transitions (soil→grass edges). Per-vertex ambient occlusion at every cube junction.
- **Plant foliage:** NOT voxel cubes. Billboard sprites for flowers/grass/ferns, low-poly mesh clusters for tree canopy. Painterly textures with alpha cutout. This is where the game departs from Minecraft and toward Cloud Gardens.
- **Trunks/stems/roots:** Voxel-based (they're structural, they interact with the grid), but with color variation.
- **Water:** Flat transparent surface with shader effects — not voxel cubes. Scrolling normal maps for ripples, depth-based opacity, caustic projection on submerged surfaces.

### Lighting

- **Primary:** Warm directional light (golden hour default). Soft PCF shadows. Never harsh.
- **Fill:** Hemisphere light — warm from above, cool-green from below (simulates light bouncing off vegetation).
- **Ambient occlusion:** Per-vertex AO baked during mesh generation (count neighboring occupied voxels at each vertex). Critical — this is the single biggest visual upgrade from flat cubes.
- **SSAO post-pass:** Complement per-vertex AO where plants meet soil.
- **Time-of-day cycle:** Dawn (pink-orange), midday (bright warm), golden hour (deep orange, purple shadows), blue hour (cool blue, fireflies). Lerp between 4 preset light configurations.
- **God rays:** When canopy is dense, volumetric light shafts hitting the forest floor. Post-process radial blur from sun position.

### Color Palette

- Earthy, warm. Greens lean yellow-green, not blue-green.
- Soil: rich chocolate-brown with subtle variation.
- Water: teal-clear, more opaque at depth.
- **Shadows are never black.** Always tinted blue or purple. This is the single biggest coziness trick.
- New growth: lighter green. Mature: deeper. Dead: warm brown/amber.
- Color grading LUT per time-of-day: slightly lifted shadows, desaturated highlights, warm midtones.

### Post-Processing Stack

1. **SSAO** — depth perception, contact shadows
2. **Bloom** — subtle, on sunlit edges and water highlights. High threshold, large soft radius.
3. **Tilt-shift DOF** — makes the garden feel like a miniature model on a table. The core "diorama" effect.
4. **Color grading** — warm LUT, shifted by time-of-day
5. **Vignette** — subtle (10-15%), draws focus inward
6. No chromatic aberration. No film grain. Clean and warm.

### Animation & Life

- **Wind sway:** Gentle vertex displacement on all vegetation (sine wave, 0.5-1Hz, offset by world position). Tops sway more than bases. Slow = cozy, frantic = anxious.
- **Water ripples:** Scrolling normal maps + interaction ripples when watering.
- **Growth transitions:** Smooth scale interpolation between growth stages. Brief green/golden particle burst when a plant advances a stage — the visual "reward sting."
- **Particles:** Pollen drifting near flowers. Water droplets when watering. Falling leaves when trees die. Fireflies at dusk. Dust motes in god rays.
- **Ambient creatures:** Butterflies between flower clusters when biodiversity is high. Visual feedback that the ecosystem is healthy.

### Camera

- **Default:** Orthographic with tilt/orbit — clean, miniature-model look. No perspective distortion. Combined with tilt-shift DOF = maximum diorama feel.
- **Admire mode:** Smooth transition to perspective orbit for close-up appreciation.
- **Underground:** Animated cutaway plane the player slides through the world. Cut surface has a subtle glow border.
- **All movement:** Smooth, heavily damped. Ease-out on all transitions. Cozy cameras float, they don't snap.

### Below-Ground Visualization

- **Cutaway mode:** Slice the world along a horizontal plane. Player slides the cut depth up/down. The exposed cross-section shows soil layers (color-blended from composition: sand=tan, clay=reddish, organic=dark brown, rock=gray), roots, and water.
- **Root glow:** Subtle emissive shader on root voxels — warm golden for healthy, dim for stressed. Makes roots readable through semi-transparent soil.
- **Water underground:** Transparency + animated caustics projected onto submerged surfaces. Beautiful in cross-section.
- **Soil composition blending:** The 6-byte composition (sand/clay/organic/rock/pH/bacteria) maps to visual blending of color palettes per-vertex.

---

## Technical Architecture

```
┌──────────────────────────────────────────────┐
│                   Browser                     │
│                                              │
│  ┌──────────────┐    ┌─────────────────────┐ │
│  │  Three.js    │    │  groundwork-sim     │ │
│  │  (TypeScript)│◄───│  (Rust → WASM)      │ │
│  │              │    │                     │ │
│  │  Scene graph │    │  bevy_ecs World     │ │
│  │  Materials   │    │  VoxelGrid (864K)   │ │
│  │  Shaders     │    │  SoilGrid           │ │
│  │  Camera      │    │  Tree entities      │ │
│  │  Post-proc   │    │  tick() loop        │ │
│  │  Particles   │    │                     │ │
│  └──────┬───────┘    └────────┬────────────┘ │
│         │                     │              │
│         │   Zero-copy read    │              │
│         │   (typed array view │              │
│         │    into WASM memory)│              │
│         └─────────────────────┘              │
│                                              │
│  ┌──────────────────────────────────────────┐ │
│  │  HTML/CSS UI — menus, HUD, controls      │ │
│  └──────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘

Existing (unchanged):
  groundwork-sim   — Rust lib, bevy_ecs (zero rendering deps)
  groundwork-tui   — Rust bin, ratatui (TUI + CLI, continues working)

New:
  groundwork-web   — TypeScript + Three.js + Vite
  groundwork-sim   — adds wasm-bindgen exports (cdylib target)
```

### WASM Bridge Design

The bridge is deliberately thin — grid + tick + tool actions:

```
Rust exports (via wasm-bindgen):
  init()                          → Create world + schedule
  tick(n: u32)                    → Advance n ticks
  grid_ptr() → *const u8          → Pointer to VoxelGrid flat array
  grid_len() → usize              → Length in bytes (864K × 4)
  soil_ptr() → *const u8          → Pointer to SoilGrid
  soil_len() → usize
  place_tool(tool, x, y, z)      → Apply a gardening tool
  fill_tool(tool, x1..z2)        → Fill region
  get_tick() → u64                → Current tick count
  get_focus() → (x, y, z)        → Current focus position
  set_focus(x, y, z)             → Move focus
  grid_dimensions() → (x, y, z)  → Grid size constants

JS reads:
  new Uint8Array(wasm.memory.buffer, grid_ptr(), grid_len())
  // Direct typed array view — zero copy, zero serialization
  // Each voxel: [material: u8, water_level: u8, light_level: u8, nutrient_level: u8]
```

### Meshing Strategy

Greedy meshing with chunking. The 120×120×60 grid divides into 16×16×16 chunks (8×8×4 = 256 chunks). After greedy meshing:

- Underground chunks collapse to a few large quads (mostly uniform soil/stone)
- Air chunks emit zero geometry
- Active chunks (surface, vegetation) have moderate geometry
- Estimated 50-150 active draw calls — well within Three.js comfort zone

**Chunk dirty tracking:** After each `tick()`, compare the grid buffer to detect which chunks changed. Only re-mesh dirty chunks. For the typical tick (water flows, plants grow slowly), most chunks are unchanged.

**Material → mesh mapping:**
- Soil, stone, water-as-underground: greedy-meshed cubes with per-vertex AO + color
- Water surface: separate flat `PlaneGeometry` with custom shader
- Trunk, branch: greedy-meshed cubes, wood-toned
- Leaf: billboard quad clusters or instanced low-poly meshes (NOT cubes)
- Seed, root: instanced small meshes or particles
- Air: no geometry

### Build Pipeline

```
groundwork-web/
├── src/
│   ├── main.ts              Entry point, init WASM, create scene
│   ├── bridge.ts            WASM bridge: typed array views, tick wrapper
│   ├── mesher/
│   │   ├── greedy.ts        Greedy meshing algorithm
│   │   ├── chunk.ts         Chunk management, dirty tracking
│   │   └── ao.ts            Per-vertex ambient occlusion calculation
│   ├── rendering/
│   │   ├── terrain.ts       Soil/stone mesh materials
│   │   ├── water.ts         Water surface shader (normal scroll, caustics)
│   │   ├── foliage.ts       Leaf billboards, flower sprites
│   │   ├── roots.ts         Root emissive material
│   │   └── particles.ts     Pollen, sparkles, fireflies, water drops
│   ├── camera/
│   │   ├── orbit.ts         Orthographic orbit controller
│   │   ├── cutaway.ts       Underground cutaway plane
│   │   └── transitions.ts   Smooth camera interpolation
│   ├── postprocess/
│   │   ├── bloom.ts         Soft bloom pass
│   │   ├── dof.ts           Tilt-shift depth of field
│   │   ├── ssao.ts          Screen-space ambient occlusion
│   │   └── color-grade.ts   Color grading LUT by time-of-day
│   ├── lighting/
│   │   ├── sun.ts           Directional light + shadow map
│   │   ├── sky.ts           Sky gradient / hemisphere light
│   │   └── time-of-day.ts   Light preset interpolation
│   └── ui/
│       ├── hud.ts           Tool selection, species picker
│       ├── controls.ts      Keyboard/mouse/touch input
│       └── panels.ts        Inspect, status panels
├── public/
│   ├── index.html
│   └── assets/              Textures, LUTs, leaf sprites
├── package.json             Three.js, vite, vite-plugin-wasm
├── tsconfig.json
└── vite.config.ts
```

**Dev workflow:**
- `wasm-pack build ../crates/groundwork-sim --target web` → compile sim to WASM
- `npm run dev` → Vite dev server with hot reload for all JS/TS/shader changes
- Visual iteration (lighting, colors, shaders) requires NO Rust recompilation

---

## Implementation Phases

### Phase 1: Skeleton (1-2 sprints)
**Goal:** Sim running in browser, visible as colored cubes.

1. Add `wasm-bindgen` exports to `groundwork-sim` (init, tick, grid_ptr, place_tool)
2. Add `cdylib` crate-type to sim's Cargo.toml
3. Scaffold `groundwork-web` with Vite + Three.js + vite-plugin-wasm
4. Render full grid as naive instanced colored cubes (no meshing yet)
5. Basic orbit camera (orthographic)
6. Tick button / auto-tick at configurable rate
7. Tool palette UI (soil, seed, water, stone, shovel)
8. Click-to-place via raycasting

**Exit criteria:** Can play the game in browser. Plant seeds, water, watch growth. Ugly but functional.

### Phase 2: Meshing & AO (1-2 sprints)
**Goal:** Efficient rendering with ambient occlusion. Looks good, not beautiful yet.

1. Implement greedy meshing with 16×16×16 chunking
2. Chunk dirty tracking (diff grid buffer after tick)
3. Per-vertex AO calculation during mesh generation
4. Per-vertex color variation (noise on material base colors)
5. Face culling — only emit exposed faces
6. Separate water surface mesh with basic transparency

**Exit criteria:** 60fps. Underground is efficient. AO makes the world feel warm and grounded.

### Phase 3: Lighting & Atmosphere (1-2 sprints)
**Goal:** The world feels warm and alive.

1. Directional sun light with soft PCF shadows
2. Hemisphere fill light (warm above, green-tinted below)
3. Time-of-day cycle with 4 presets (dawn, noon, golden hour, blue hour)
4. Sky gradient that shifts with time-of-day
5. Basic post-processing: bloom (subtle), color grading (warm LUT)
6. Shadow color tinting (blue/purple, never black)

**Exit criteria:** Screenshots look inviting. Golden hour makes you want to garden.

### Phase 4: Water & Underground (1 sprint)
**Goal:** Water is beautiful. Underground is readable and magical.

1. Custom water surface shader: scrolling normal maps, depth-based opacity, sky reflection
2. Interaction ripples when watering
3. Caustic texture projection on submerged surfaces
4. Cutaway mode: animated horizontal slice plane
5. Soil composition → color blending (sand=tan, clay=red-brown, organic=dark, rock=gray)
6. Root emissive glow shader (golden = healthy, dim = stressed)

**Exit criteria:** Water catches light beautifully. Cross-section reveals a living underground world.

### Phase 5: Foliage & Life (1-2 sprints)
**Goal:** Plants look organic, not blocky. The garden breathes.

1. Replace leaf/flower voxel cubes with billboard sprites / low-poly mesh clusters
2. Species-specific foliage appearances (oak round canopy, pine conical, willow drooping, etc.)
3. Wind sway vertex animation (gentle, 0.5-1Hz)
4. Growth stage transitions: smooth scale interpolation
5. Growth particle bursts (green/golden sparkles on stage advance)
6. Pollen particles near flowering plants
7. Falling leaves on tree death/stress

**Exit criteria:** The garden feels alive even when you're not touching it.

### Phase 6: Polish & Diorama (1 sprint)
**Goal:** The game feels like a precious miniature world.

1. Tilt-shift depth of field (the signature diorama effect)
2. SSAO post-pass for fine contact shadows
3. Vignette (subtle)
4. Camera smoothing — all transitions ease-out, float-on-a-cloud feel
5. Fireflies at dusk/night
6. Ambient creatures (butterflies between flowers at high biodiversity)
7. Dust motes in volumetric light shafts
8. Beveled voxel edges on surface terrain (subtle chamfer)

**Exit criteria:** People screenshot the game and share it. It's beautiful.

### Future (not in scope)
- Sound design / adaptive audio (separate workstream)
- Mobile touch controls optimization
- Larger world / streaming chunks
- Multiplayer via SharedArrayBuffer
- Native desktop/console renderer (see Multi-Platform Strategy below)

---

## Parallelization Strategy

The implementation naturally splits into three independent workstreams that can run concurrently:

```
Workstream 1: WASM Bridge       Workstream 2: Three.js Renderer      Workstream 3: Sim (ongoing)
───────────────────────         ────────────────────────────────      ───────────────────────────
Add cdylib target               Scaffold groundwork-web (Vite)        Continue plant/ecology work
wasm-bindgen exports            Greedy mesher (mock voxel data)       (already happening)
  init/tick/grid_ptr/place      Per-vertex AO calculator
wasm-pack build script          Chunk manager + dirty tracking
                                Camera controller (orbit, cutaway)
        │                       Water shader, lighting, post-proc
        │                       Foliage billboards, particles
        └──── merge ───────────►Connect real WASM grid to mesher
                                ── playable ──►
```

**Why this works:** The mesher and renderer develop against a **mock voxel buffer** — a static `Uint8Array` with test terrain. The WASM bridge is a small, separate task. They connect at integration time.

### Parallel task breakdown

| Task | Depends on | ~Effort | Workstream |
|------|-----------|---------|------------|
| **WASM bridge** — wasm-bindgen exports | Nothing | 1-2 days | 1 |
| **Vite scaffold** — project setup, dev server | Nothing | Half day | 2 |
| **Greedy mesher** — chunked meshing + AO | Nothing (mock data) | 3-5 days | 2 |
| **Water shader** — normals, caustics, depth | Scaffold | 2-3 days | 2 |
| **Lighting system** — sun, hemisphere, ToD | Scaffold | 2-3 days | 2 |
| **Foliage renderer** — billboards, wind sway | Mesher | 2-3 days | 2 |
| **Post-processing** — bloom, DOF, SSAO | Lighting | 2-3 days | 2 |
| **Camera** — orbit, cutaway, smooth | Scaffold | 1-2 days | 2 |
| **Integration** — WASM grid → mesher, tools → bridge | Bridge + Mesher | 1-2 days | merge |
| **Sim work** — continues independently | Nothing | Ongoing | 3 |

### Critical path

```
Vite scaffold ──► Greedy mesher ──► Integration ──► Playable
                                        ▲
WASM bridge ────────────────────────────┘
```

Everything else (water, lighting, foliage, post-processing, camera) hangs off the scaffold and can be composed independently.

---

## Multi-Platform Strategy

The sim is renderer-agnostic. Multiple renderers can coexist — you add per-platform frontends, not rewrite anything.

| Platform | Renderer | How |
|----------|----------|-----|
| **Web** | Three.js + WASM | This plan (primary target) |
| **Desktop (quick)** | Tauri + Three.js | Wrap the web app in a native window. Same code, native binary. ~1 day. |
| **Desktop (premium)** | Bevy or wgpu native | Second renderer crate. Native GPU, smaller binary, better perf. |
| **Switch** | wgpu or Bevy native | Must be native (no browser). wgpu targets Vulkan on ARM. |

The sim crate compiles to WASM (web), native ARM (Switch), and native x86 (desktop). Each renderer is a thin shell calling `tick()` and reading the grid.

**Recommended sequencing:**
1. **Now:** Three.js for web (Phases 1-6)
2. **After web ships:** Tauri wrapper for Steam/itch.io desktop distribution (essentially free)
3. **If/when targeting Switch:** Native wgpu renderer (`groundwork-native` crate, ~4-6 weeks). Visual design, meshing algorithms, AO, and shader logic all transfer from Three.js version. Bevy full engine remains viable if its web story improves by then.

---

## Compatibility & Coexistence

### TUI/CLI continues unchanged

The sim crate gains wasm-bindgen exports but its native API doesn't change. `groundwork-tui` continues to work exactly as it does today — same binary, same tests, same agent playtesting workflow.

```toml
# groundwork-sim/Cargo.toml
[lib]
crate-type = ["rlib", "cdylib"]  # rlib for TUI, cdylib for WASM

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"

[dependencies]
bevy_ecs = "0.18"  # unchanged
```

The wasm-bindgen exports are behind `#[cfg(target_arch = "wasm32")]` — they don't exist in native builds. Zero impact on TUI compilation or performance.

### Interface parity

The web UI must support the same player actions as CLI/TUI: place tools, cycle species, inspect voxels, view status. The WASM bridge exports match the CLI command set.

### Testing strategy

- Sim tests: `cargo test -p groundwork-sim` (unchanged, runs natively)
- Web smoke tests: Playwright/Puppeteer — load page, tick sim, verify canvas renders
- Visual regression: screenshot comparison at known game states

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| WASM bridge adds complexity | Medium | Low | Bridge is thin (grid pointer + tick). Voxelize proves the pattern works. |
| Two-language codebase maintenance | Medium | Medium | Clean boundary: Rust owns simulation, TS owns rendering. No logic duplication. |
| Three.js API changes | Low | Low | Three.js is very stable (15 years). Major version changes are rare and backward-compatible. |
| Performance on low-end devices | Medium | Medium | Greedy meshing keeps draw calls low. 864K voxels is small. Profile early. |
| WebGPU fallback issues | Low | Low | Three.js r171+ handles WebGPU→WebGL2 fallback automatically. |
| Visual iteration takes longer than expected | High | Medium | This is art, not engineering. Budget extra time. Hot reload helps enormously. |

---

## Reference Projects & Inspiration

**Architecture:** [Voxelize](https://github.com/voxelize/voxelize) — Rust + Three.js voxel game (the closest existing reference)

**Aesthetics:** Townscaper (procedural charm, pastel palette, reflective water), Cloud Gardens (nature reclaiming space, diorama scale, painterly foliage), Stardew Valley (seasonal palettes, sprite warmth)

**Techniques:** [Townscaper rendering in WebGL](https://reindernijhoff.net/2021/11/townscapers-rendering-style-in-webgl/) — proves the cozy aesthetic is feasible in browser

**Meshing:** [Voxel Meshing in Exile](https://thenumb.at/Voxel-Meshing-in-Exile/), [Greedy Meshing in JavaScript](https://www.jameshylands.co.uk/2022/10/greedy-meshing-in-javascript.html)

**Water:** [Codrops stylized water in Three.js](https://tympanus.net/codrops/2025/03/04/creating-stylized-water-effects-with-react-three-fiber/)

---

## Summary

**Stack:** Three.js (WebGPU, auto-fallback to WebGL2) + groundwork-sim compiled to WASM via wasm-bindgen

**Visual identity:** Warm diorama. Beveled voxel terrain with per-vertex AO. Non-voxel foliage (billboards, mesh clusters). Stylized water with caustics. Soft golden-hour lighting. Tilt-shift DOF. Particles that make the garden breathe.

**Architecture:** Thin WASM bridge (zero-copy grid access). Greedy meshing with chunk dirty tracking. Hot-reloadable visual pipeline.

**Coexistence:** TUI/CLI unchanged. Sim gains `cdylib` target + wasm-bindgen exports behind `cfg(wasm32)`. Same tests, same agent workflow.

**Guiding principle:** When in doubt — (1) make ecological cause-and-effect more readable, (2) increase beauty, (3) keep the build smaller.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

GROUNDWORK is a cozy ecological voxel garden builder game. The player composes ecosystems by shaping soil, water, light, and plant relationships above and below ground. Core fantasy: build a living miniature world that becomes self-sustaining, surprising, and teeming with life over time.

**What makes it sticky:** The garden is not just plants and physics — it's an interconnected web of life. Fauna (insects, birds, worms, pollinators) create visible relationships between plants. Ecological discovery unfolds in layers: first you learn mechanics, then competition, then synergy, then you design self-sustaining loops. The game rewards observation with surprise — consequences you didn't plan but can trace backward.

**Current phase:** Core game development. The simulation foundation is complete (12 species, water/light/soil/root systems, procedural trees). The primary workstream is now the **Three.js web renderer** — making the game beautiful and playable in the browser.

## Session Quick Start

Every session, you operate as one of three roles. Pick yours and follow the checklist.

**Role assignment:**
1. If a role is specified in the prompt, use it.
2. Default to **dev** if dev work is available (check `handoffs/manager_to_dev/` and `backlog/current.md`).
3. Fall back to **manager** if no dev work but feedback exists to review (check `feedback/` and `handoffs/player_to_manager/`).
4. Fall back to **player** if nothing else applies.

### Dev — start here
1. Read `agents/dev.md` (your role definition)
2. Read `backlog/current.md` (what's prioritized)
3. Read latest file in `handoffs/manager_to_dev/` (your current assignment)
4. Read latest file in `build_notes/` (where the last dev left off)
5. Build and test: `cargo test -p groundwork-sim && cargo check --workspace`
6. For web work: `cd crates/groundwork-web && npm install && npm run dev`
7. Do the work. Write build notes and dev→manager handoff when done.

### Manager — start here
1. Read `agents/manager.md` (your role definition)
2. Read `backlog/current.md` (current priorities)
3. Read latest files in `feedback/` and `handoffs/player_to_manager/` (new player input)
4. Read latest files in `handoffs/dev_to_manager/` (dev results)
5. Update backlog, write decisions, write handoffs to dev and/or player.

### Player — start here
1. Read `agents/player.md` (your role definition)
2. Read latest file in `handoffs/manager_to_player/` (what to test and specific questions)
3. Launch: `cd crates/groundwork-web && npm run dev` then open http://localhost:5173
4. Play a session and write feedback + player→manager handoff.

All roles: read `AGENTS.md` for the full operating framework (vision, handoff formats, priority definitions, workspace rules).

## Build & Run

```bash
# --- Simulation (Rust) ---
cargo test -p groundwork-sim          # Run sim tests
cargo check --workspace               # Check everything compiles

# --- Web UI (primary player interface) ---
cd crates/groundwork-web
npm install                            # First time only
npm run dev                            # Vite dev server → http://localhost:5173
npm run build                          # Production build → dist/

# --- WASM bridge (connects sim to web) ---
npm run wasm                           # wasm-pack build (requires wasm-pack installed)

# --- Player agent (automated playtesting) ---
cargo test -p groundwork-player       # Run all scenarios (deterministic)

# --- TUI (debug/dev tool) ---
cargo run -p groundwork-tui            # Terminal UI for debugging
cargo run -p groundwork-tui -- new     # Create fresh world + launch TUI
cargo run -p groundwork-tui -- help    # CLI commands reference
```

## Architecture

```
crates/
  groundwork-sim/         Rust library — bevy_ecs standalone (no rendering)
    src/
      lib.rs              Public API: create_world(), create_schedule(), tick()
      voxel.rs            Voxel cell struct (Material + water/light/nutrient levels, 4 bytes)
      grid.rs             VoxelGrid Resource — flat Vec<Voxel>, 120×120×60
      scale.rs            Scale normalization: VOXEL_SIZE_M (0.5m), meters_to_voxels()
      soil.rs             SoilComposition (6 bytes) + SoilGrid Resource
      tree.rs             12 species, PlantType enum, growth stages, space colonization
      systems.rs          ECS systems: water, soil, light, seeds, trees, dispersal
      save.rs             Binary save/load v3
      wasm_bridge.rs      wasm-bindgen exports (cfg(wasm32) guarded)

  groundwork-web/         TypeScript — Three.js + Vite (PRIMARY PLAYER INTERFACE)
    src/
      main.ts             Entry point: scene, render loop, input, auto-tick
      bridge.ts           WASM bridge: zero-copy typed array views into sim memory
      mesher/greedy.ts    Greedy meshing with per-vertex AO, 16×16×16 chunks
      mesher/chunk.ts     Chunk dirty tracking via grid snapshot diffing
      rendering/terrain.ts  BufferGeometry builder, warm material palette
      rendering/water.ts  Water surface mesh renderer
      rendering/foliage.ts  Foliage billboard sprites with wind sway
      rendering/particles.ts  Growth particle burst system
      camera/orbit.ts     Orthographic orbit camera with smooth damping
      lighting/sun.ts     Golden hour directional + hemisphere fill + ambient
      lighting/sky.ts     Sky gradient background shader
      lighting/daycycle.ts  Time-of-day cycle controller
      postprocessing/effects.ts  SSAO, bloom, DOF, color grading
      ui/raycaster.ts     Mouse click → voxel coordinate raycasting
      ui/controls.ts      Input controls and keyboard shortcuts
      ui/hud.ts           HUD overlay: tool palette, species picker, status
    vite.config.ts        WASM plugin, COOP/COEP headers, GitHub Pages base
    package.json          three, vite, vite-plugin-wasm, typescript

  groundwork-tui/         Rust binary — ratatui (DEBUG/DEV TOOL, not primary UI)
    src/
      main.rs             Entry point, subcommand dispatch
      cli.rs              Non-interactive CLI commands
      app.rs              App state, tools, species selection
      quest.rs            Mission/quest onboarding system (20 quests, 9 chapters)
      render.rs           2D ASCII rendering + side panel
      render3d.rs         3D projected voxel rendering
      camera.rs           Orbit camera for 3D projected view
      glyph.rs            Character/sprite glyph system
      input.rs            Keyboard controls

  groundwork-player/      Rust library — embodied player agent framework
    src/
      lib.rs              Public API
      action.rs           Player actions (place, tick, inspect, etc.)
      scenario.rs         Scenario definition + fluent builder
      runner.rs           Deterministic scenario executor
      trace.rs            Run trace recording (action→observation→oracle triples)
      observer.rs         Actor-view observations (what the player sees)
      oracle.rs           Oracle-view state snapshots (privileged ground truth)
      evaluator.rs        Evaluator trait + built-in evaluators
      scenarios/          Built-in scenario definitions
    tests/
      scenarios.rs        Integration tests: all scenarios pass deterministically

  groundwork-profiler/    Rust binary — simulation performance profiling
```

### Key Design Decisions

- **bevy_ecs standalone**: fast compile (~60s cold, <1s incremental), simulation-only
- **Renderer-agnostic sim**: `groundwork-sim` has zero rendering deps. Web and TUI are thin shells that read sim state.
- **Three.js + WASM hybrid**: Sim compiles to WASM via wasm-bindgen. Three.js reads the voxel grid via zero-copy typed array views into WASM memory. JS hot-reloads for visual iteration. See `decisions/2026-03-13T18:00:00_3d_web_renderer_plan.md`.
- **Zero-copy data path**: VoxelGrid is a flat array (864K × 4 bytes = 3.5MB). JS reads it directly from WASM linear memory as a `Uint8Array`.
- **Greedy meshing with chunking**: 120×120×60 grid divides into 16×16×16 chunks. Per-vertex AO. Only dirty chunks re-mesh after tick.
- **Scale normalization**: All dimensions in meters, converted via `scale.rs`. `VOXEL_SIZE_M = 0.5`.
- **Flat voxel array**: 864K voxels in contiguous Vec. Z=0 is deepest, Z=GROUND_LEVEL (~30) is surface, Z=59 is sky.
- **12 species, 4 plant types**: Tree/Shrub/Groundcover/Flower. Trees use space colonization branching, others use templates.
- **System execution order**: water_spring → water_flow → soil_absorption → root_water_absorption → soil_evolution → light_propagation → seed_growth → tree_growth → branch_growth → tree_rasterize → self_pruning → seed_dispersal → tick_counter

### Sim API

```rust
let mut world = groundwork_sim::create_world();
let mut schedule = groundwork_sim::create_schedule();
groundwork_sim::tick(&mut world, &mut schedule);
let grid = world.resource::<VoxelGrid>();
```

### WASM Bridge API

```
Rust exports (via wasm-bindgen, in wasm_bridge.rs):
  init()                    → Create world + schedule
  tick(n)                   → Advance n ticks
  grid_ptr() → *const u8   → Pointer to VoxelGrid flat array
  grid_len() → usize       → Length in bytes (864K × 4)
  soil_ptr() → *const u8   → Pointer to SoilGrid flat array
  soil_len() → usize       → Length in bytes
  grid_width/height/depth() → Grid dimensions
  get_tick() → u64          → Current tick count
  place_tool(tool, x, y, z) → Apply gardening tool
  fill_tool(tool, x1..z2)  → Fill region
  get_focus_x/y/z()        → Camera focus point
  set_focus(x, y, z)       → Set camera focus

JS reads:
  new Uint8Array(wasm.memory.buffer, grid_ptr(), grid_len())
  // Each voxel: [material: u8, water_level: u8, light_level: u8, nutrient_level: u8]
```

## Gardening Tools
- `air`/`dig` = **shovel** — removes anything (seeds, roots, soil, stone)
- `seed` = **seed bag** — plants a seed; falls through air; dies on stone
- `water` = **watering can** — pours water; falls through air; no-op on water
- `soil` = **soil** — places soil; falls through air
- `stone` = **stone** — places stone directly (no gravity)

## Species
- **Trees:** `oak`, `birch`, `willow`, `pine` — tall, space colonization branching
- **Shrubs:** `fern`, `berry-bush`, `holly` — bushy, 1-2m, template-only
- **Flowers:** `wildflower`, `daisy` — thin stem + bloom, fast growing
- **Groundcover:** `moss`, `grass`, `clover` — flat disc, spreads quickly

## Gameplay Depth Principles

These principles define what makes the game *engaging over time*, not just legible at first glance. Every system, feature, and polish pass should be evaluated against them.

1. **The garden must feel alive, not just growing.** Fauna — pollinators, decomposers, birds, insects — are the connective tissue of the ecosystem. A flower isn't just pretty; it attracts a bee, which pollinates another flower, which spreads to a place you didn't plant it. Visible creatures moving through the garden tell the player the world has agency beyond their own actions.

2. **Surprise must reward observation.** The best moments are consequences the player didn't plan but can trace backward. You planted oak and wildflower near each other — now bees appeared, and the wildflower spreads faster via pollination you didn't arrange. Emergence isn't a side effect of simulation; it's the core delight. Design for interactions that produce outcomes neither species would achieve alone.

3. **Discovery must shift the player's mental model.** The game has a learning arc, not just a building arc:
   - First hour: "Seeds need soil and water." (Mechanics)
   - Third hour: "The oak's roots are stealing water from the birch." (Competition)
   - Tenth hour: "Clover fixes nitrogen, which feeds the oak, whose canopy shades the fern, which holds moisture for the moss." (Synergy)
   - Twentieth hour: "I can design a self-sustaining loop where every species serves a role." (Ecology as architecture)
   Without this progression, the game is a toy you exhaust in 20 minutes.

4. **Interactions, not just growth.** Species must affect each other — pollination, competition, symbiosis, decomposition, shelter. Not just "plant grows if water+light" but "plant thrives *because of its neighbors*." One visible interaction chain is worth ten independent growth scripts.

5. **The garden must exceed the player's plan.** Seed dispersal by wind, birds, or water should place plants where the player didn't. Dead wood should attract fungi. A puddle should attract insects. The garden should develop a life of its own that the player guides but doesn't fully control.

6. **The game must feel cozy, warm, and generous.** This is a place the player *wants to be*. Golden light, gentle particle effects, soft growth animations, ambient sounds of life. Mistakes don't punish — the garden recovers, pioneer species colonize bare patches, life finds a way. The emotional register is wonder, not stress.

7. **Idle time must be rewarding.** A player who stops clicking and just watches should see a living painting: pollinators drifting, light shifting, leaves swaying, worms working soil, slow growth unfolding. If the garden looks the same after 100 idle ticks, it's a screensaver. If it's visibly *different* — new fauna, shifted light, a seedling that appeared — it's alive.

8. **Recovery is a feature, not a failure state.** When the player floods, over-digs, or crowds, the response should be visible organic recovery — pioneer species, nutrient cycling, gradual recolonization. The garden's resilience teaches the player that experimentation is safe, making them bolder and more creative.

## Autonomous Simulation Loop

The LLM player agent loop (Phases 1-3 in `groundwork-player`) exists to make these principles *reliable and measurable*. See `decisions/2026-03-15T12:00:00_llm_simulation_experiential_vision.md`, Target Moments and Big Yeses/Nos in `AGENTS.md`.

**Loop goal:** After a thousand autonomous runs, surface the **3-5 specific interactions that make this game magical** — then amplify them 1000x while cutting everything that doesn't serve them.

**What the loop must concretely test** (see AGENTS.md Big Yeses for full specs):
- Does the Nitrogen Handshake work? (clover near oak → visible faster growth, discoverable by planner)
- Does the Pollinator Bridge form? (flower cluster → fauna spawn → cross-pollination → population spread)
- Does the Root War read clearly? (competing trees → underground diagnosis → behavioral change)
- Does the Bird Express deliver gifts? (berry bush → bird → seed drop → unplanned beneficial plant)
- Does Pioneer Succession unfold? (bare soil → moss → grass → wildflower → shrub, autonomously)
- Does the Canopy Effect create niches? (tall tree shade → undergrowth layer of shade-loving species)

**When a winner surfaces:** Name it. Measure it. Make it gorgeous. Protect it with a regression test. Build adjacent interactions. Cut anything that competes for attention.

**Key evaluation shifts:**
- Evaluators must measure *delight* (unplanned-but-beneficial events, causal chain length, idle-period activity, discovery count) — not just material counts
- Skill extraction should capture *discoveries* ("clover near oak = faster growth") not just *techniques* ("place water at z=45")
- The LLM planner should be *curious*, not optimal — a planner that investigates surprises is more valuable than one that maximizes coverage
- Failure is redefined: a run where the garden dies but the planner discovers root competition is a *success* for the discovery arc
- The loop's nos are as important as its yeses: if a species never participates in chains, a fauna type never changes behavior, or a mechanic produces the same result every time — cut it

## Key Constraints

- **MVP scope is locked**: one temperate biome, 12-20 species, four systems (light/water/roots/ecology), one ~120×120×60 voxel garden bed (60m×60m×30m at 0.5m/voxel), continuous above/below-ground camera
- **Fauna and interaction webs are MVP**, not post-MVP. A garden without visible life and species interactions cannot answer the canonical question. Even simple representations (particle-like bees, earthworm soil trails, bird silhouettes) count — fidelity can be low if the ecological *role* is clear.
- **Web is the primary player interface.** TUI/CLI continue as dev/debug tools.
- **Do not** expand beyond MVP, add biomes/economies/multiplayer/narrative (auto-P3), optimize realism over readability, or hide cause-and-effect
- **Decision rule when uncertain**: (1) make ecological cause-and-effect more readable, (2) reward player observation with surprise, (3) keep the build smaller, (4) increase player delight sooner
- **Source of truth order**: game vision > Manager backlog > player feedback > build notes > older discussion

## Priority Definitions

- **P0** = blocks core proof or makes build unusable
- **P1** = strongly improves clarity, feel, or core loop
- **P2** = valuable but not required for MVP
- **P3** = future/expansion (auto-assigned to new biomes, economies, multiplayer, narrative)

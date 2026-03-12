# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

GROUNDWORK is a cozy ecological voxel garden builder game. The player composes ecosystems by shaping soil, water, light, and plant relationships above and below ground. Core fantasy: build a living miniature world that becomes self-sustaining over time.

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
6. Do the work. Write build notes and dev→manager handoff when done.

### Manager — start here
1. Read `agents/manager.md` (your role definition)
2. Read `backlog/current.md` (current priorities)
3. Read latest files in `feedback/` and `handoffs/player_to_manager/` (new player input)
4. Read latest files in `handoffs/dev_to_manager/` (dev results)
5. Update backlog, write decisions, write handoffs to dev and/or player.

### Player — start here
1. Read `agents/player.md` (your role definition, includes CLI play instructions)
2. Read latest file in `handoffs/manager_to_player/` (what to test and specific questions)
3. Build: `cargo run -p groundwork-tui -- new` then play a session
4. Write feedback and player→manager handoff when done.

All roles: read `AGENTS.md` for the full operating framework (vision, handoff formats, priority definitions, workspace rules).

## Build & Run

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Run the interactive TUI
cargo run -p groundwork-tui

# Run just the sim tests
cargo test -p groundwork-sim

# Check everything compiles
cargo check --workspace
```

## CLI (non-interactive / agent play)

The CLI lets agents play the game without a terminal. State persists to a binary file between invocations.

```bash
groundwork new                            # Create a fresh world → groundwork.state
groundwork tick [N]                       # Advance N ticks (default 1)
groundwork view [--z Z]                   # Print ASCII slice (default Z=surface+1)
groundwork place <tool> <x> <y> <z>      # Use a gardening tool at coordinates
                                          # Coords accept ranges: place soil 20..40 30 15
groundwork fill <tool> <x1> <y1> <z1> <x2> <y2> <z2>  # Fill rectangular region
groundwork inspect [<x> <y> <z>]          # Show voxel details (uses focus if no coords)
groundwork status                         # Tick count + material summary
groundwork focus [<x> <y> <z>]            # Get/set focus cursor position (persisted)
groundwork tool-start <tool>              # Begin range operation at current focus
groundwork tool-end                       # Apply tool from start to current focus
groundwork tui                            # Launch interactive TUI (default)
groundwork help                           # Show help

# All commands accept --state FILE (default: groundwork.state)
```

### Gardening tools
- `air`/`dig` = **shovel** — removes anything (seeds, roots, soil, stone)
- `seed` = **seed bag** — plants a seed; falls through air; dies on stone
- `water` = **watering can** — pours water; falls through air; no-op on water
- `soil` = **soil** — places soil; falls through air
- `stone` = **stone** — places stone directly (no gravity)

Non-shovel tools can't overwrite occupied cells. Use the shovel to clear first.

### ASCII legend
`.` air, `~` water, `#` soil, `%` wet soil, `@` stone, `*` root, `s` seed, `S` sprouting

### State file format
Binary, ~422KB. Version 2. Header (magic `GWRK` + version u16 LE + 2 reserved) + tick count (u64 LE) + 108,000 voxels × 4 bytes each + focus state (14 bytes: position + tool). Backward-compatible: loads version 1 files (no focus block) with default focus.

## Architecture

```
crates/
  groundwork-sim/     Rust library — bevy_ecs standalone (no rendering)
    src/
      lib.rs          Public API: create_world(), create_schedule(), tick(), FocusState, ToolState
      voxel.rs        Voxel cell struct (Material + water/light/nutrient levels, 4 bytes)
      grid.rs         VoxelGrid Resource — flat Vec<Voxel>, 60×60×30, indexed [x + y*60 + z*3600]
      systems.rs      ECS systems: water_flow, light_propagation, soil_absorption, seed_growth
      save.rs         Binary save/load v2: VoxelGrid + Tick + FocusState (backward-compatible with v1)

  groundwork-tui/     Rust binary — ratatui terminal renderer + CLI
    src/
      main.rs         Entry point, subcommand dispatch
      cli.rs          Non-interactive CLI commands (new/tick/view/place/fill/inspect/status/focus/tool-start/tool-end)
      app.rs          App state: World + Schedule, viewport-centered camera, Tool enum, apply_tool() with gravity
      render.rs       ASCII rendering of Z-slice around focus; side panel (inspect/status/controls/legend)
      input.rs        Keyboard controls (WASD pan, J/K depth, Tab tool, Enter use, I/T/H panels)

  (future) groundwork-web/    Three.js + WASM — browser renderer (orthogonal workstream)
```

### Design Decisions

- **bevy_ecs standalone** (not full Bevy engine): fast compile (~60s cold, <1s incremental), minimal deps, simulation-only
- **Renderer-agnostic sim**: `groundwork-sim` has zero rendering deps. TUI and future web UI are thin shells that read sim state.
- **WASM-ready**: sim compiles to `wasm32-unknown-unknown`. When `groundwork-web` is built, add `crate-type = ["cdylib"]` to sim's Cargo.toml.
- **Two orthogonal workstreams**: sim+CLI and web UI are independent. See `decisions/2026-03-11T12:00:00_web_ui_workstream.md`.
- **Flat voxel array**: 108K voxels in a contiguous Vec for cache-friendly iteration. Z=0 is deepest underground, Z=15 is surface, Z=29 is sky.
- **Snapshot-based systems**: water_flow takes a snapshot of water levels before mutation to avoid iteration-order artifacts.
- **System execution order**: water_flow → soil_absorption → light_propagation → seed_growth → tick_counter
- **Viewport-centered camera**: TUI focus is always at screen center. WASD pans the viewport. Screen size and world size are decoupled — precursor to arbitrarily large worlds and full voxel rendering.

### Sim API

```rust
let mut world = groundwork_sim::create_world();   // World with default terrain + water spring
let mut schedule = groundwork_sim::create_schedule(); // Systems in order
groundwork_sim::tick(&mut world, &mut schedule);   // Advance one step
let grid = world.resource::<VoxelGrid>();          // Read state
```

## Interface Parity Rule

**CLI and TUI must ship player-facing features together.** When a sprint adds a new player action to one interface, the same sprint must add the corresponding mechanism to the other. Neither interface ships alone.

**Why:** Agentic play testers use the CLI; human testers use the TUI. If the interfaces diverge, agent feedback stops reflecting real player UX. Agents can't report friction they never experience. We'd be optimizing for a CLI game, not the actual game.

**Focus + Tool model:** Both interfaces share a focus/cursor concept and a two-step tool-start/tool-end workflow for range operations. See `decisions/2026-03-12T14:00:00_interface_parity_and_focus_mechanism.md`.

**Dev checklist addition:** Every dev assignment that adds a player-facing action must include acceptance checks for both CLI and TUI.

## Key Constraints

- **MVP scope is locked**: one temperate biome, 12-20 species, four systems (light/water/roots/ecology), one ~60x60x30 voxel garden bed, continuous above/below-ground camera
- **Do not** expand beyond MVP, add biomes/economies/multiplayer/narrative (auto-P3), optimize realism over readability, or hide cause-and-effect
- **Decision rule when uncertain**: (1) make ecological cause-and-effect more readable, (2) keep the build smaller, (3) increase player delight sooner
- **Source of truth order**: game vision > Manager backlog > player feedback > build notes > older discussion

## Priority Definitions

- **P0** = blocks core proof or makes build unusable
- **P1** = strongly improves clarity, feel, or core loop
- **P2** = valuable but not required for MVP
- **P3** = future/expansion (auto-assigned to new biomes, economies, multiplayer, narrative)

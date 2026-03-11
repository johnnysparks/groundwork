# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

GROUNDWORK is a cozy ecological voxel garden builder game. The player composes ecosystems by shaping soil, water, light, and plant relationships above and below ground. Core fantasy: build a living miniature world that becomes self-sustaining over time.

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

# Fast linking (optional, install lld first: brew install llvm)
# Uncomment your platform section in .cargo/config.toml
```

## CLI (non-interactive / agent play)

The CLI lets agents play the game without a terminal. State persists to a binary file between invocations.

```bash
groundwork new                            # Create a fresh world → groundwork.state
groundwork tick [N]                       # Advance N ticks (default 1)
groundwork view [--z Z]                   # Print ASCII slice (default Z=16, above ground)
groundwork place <material> <x> <y> <z>   # Place a voxel (air/soil/stone/water/root)
groundwork inspect <x> <y> <z>            # Show one voxel's full state
groundwork status                         # Tick count + material summary
groundwork tui                            # Launch interactive TUI (default)
groundwork help                           # Show help

# All commands accept --state FILE (default: groundwork.state)
```

### ASCII legend
`.` air, `~` water, `#` soil, `%` wet soil, `@` stone, `*` root

### State file format
Binary, ~422KB. Header (magic `GWRK` + version) + tick count (u64 LE) + 108,000 voxels × 4 bytes each.

## Architecture

```
crates/
  groundwork-sim/     Rust library — bevy_ecs standalone (no rendering)
    src/
      lib.rs          Public API: create_world(), create_schedule(), tick()
      voxel.rs        Voxel cell struct (Material + water/light/nutrient levels, 4 bytes)
      grid.rs         VoxelGrid Resource — flat Vec<Voxel>, 60×60×30, indexed [x + y*60 + z*3600]
      systems.rs      ECS systems: water_flow, light_propagation, soil_absorption
      save.rs         Binary save/load for VoxelGrid + Tick (zero external deps)

  groundwork-tui/     Rust binary — ratatui terminal renderer + CLI
    src/
      main.rs         Entry point, subcommand dispatch
      cli.rs          Non-interactive CLI commands (new/tick/view/place/inspect/status)
      app.rs          App state: holds World + Schedule, main loop
      render.rs       ASCII rendering of a Z-slice of the grid (TUI mode)
      input.rs        Keyboard controls (navigate depth, tick, auto-play)

  (future) groundwork-web/    Three.js + WASM — browser renderer (orthogonal workstream)
```

### Design Decisions

- **bevy_ecs standalone** (not full Bevy engine): fast compile (~60s cold, <1s incremental), minimal deps, simulation-only
- **Renderer-agnostic sim**: `groundwork-sim` has zero rendering deps. TUI and future web UI are thin shells that read sim state.
- **WASM-ready**: sim compiles to `wasm32-unknown-unknown`. When `groundwork-web` is built, add `crate-type = ["cdylib"]` to sim's Cargo.toml.
- **Two orthogonal workstreams**: sim+CLI and web UI are independent. Web UI begins once the "one more seed" loop is somewhat fun in ASCII. Both progress in parallel from that point. See `decisions/2026-03-11T12:00:00_web_ui_workstream.md`.
- **ratatui for dev/AI interface**: terminal UI lets agents screenshot the same view humans see. No GPU setup required.
- **Flat voxel array**: 108K voxels in a contiguous Vec for cache-friendly iteration. Z=0 is deepest underground, Z=15 is surface, Z=29 is sky.
- **Snapshot-based systems**: water_flow takes a snapshot of water levels before mutation to avoid iteration-order artifacts.

### Sim API

```rust
let mut world = groundwork_sim::create_world();   // World with default terrain + water spring
let mut schedule = groundwork_sim::create_schedule(); // Systems in order
groundwork_sim::tick(&mut world, &mut schedule);   // Advance one step
let grid = world.resource::<VoxelGrid>();          // Read state
```

## Role System

You operate as one of three agent roles defined in `agents/`. Role assignment rules (from AGENTS.md line 42-44):
1. If a role is specified, use it
2. Default to **dev** if dev work is available
3. Fall back to **manager** if no dev work but feedback exists to review
4. Fall back to **player** if nothing else applies

Each role has strict output formats, handoff templates, and responsibilities — read the relevant `agents/*.md` file before producing output.

### Shared Workspace

Agents read/write coordination files in the repo root using timestamped names (`{YYYY-MM-DDTHH:mm:ss}_{short_desc}.md`):

- `feedback/` — Player owns
- `handoffs/{player_to_manager,manager_to_dev,dev_to_manager,manager_to_player}/`
- `backlog/current.md` — Manager owns
- `decisions/` — Manager owns
- `build_notes/` — Dev owns
- `artifacts/` — Dev owns

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

# Dev → Manager Handoff: Workspace Scaffold

## 1. Implemented

- Rust workspace with two crates: `groundwork-sim` (library) and `groundwork-tui` (binary)
- **groundwork-sim**: bevy_ecs 0.18 standalone, VoxelGrid Resource (60×60×30 flat array), three ECS systems (water_flow, light_propagation, soil_absorption), public API (create_world, create_schedule, tick)
- **groundwork-tui**: ratatui 0.30 terminal renderer, ASCII visualization of Z-slice, keyboard navigation (depth, manual/auto tick, speed control), status bar
- Default terrain generation: stone base, soil middle, air above ground, 4×4 water spring at center surface
- Compile-time optimizations: dev profile optimizes deps at opt-level 2, linker config documented
- WASM target in rust-toolchain.toml (ready for future web build)
- Updated CLAUDE.md with build commands, architecture, and sim API reference
- Architecture decision record: `decisions/2026-03-11T00:00:00_engine_and_architecture.md`

## 2. Not Implemented

- Cannot verify compilation — Rust is not installed on the current machine. First `cargo check` may surface minor issues.
- No unit tests yet for the simulation systems
- No seed planting or plant growth systems (water + light only)
- No groundwork-web (Three.js + WASM) — intentionally deferred
- No save/load (serde derives not yet added)
- Linker optimizations in `.cargo/config.toml` are commented out (need lld installed)

## 3. Tradeoffs Made

- **bevy_ecs 0.18 (latest)** over pinning an older stable version — latest standalone API, but untested in the wild for long. Low risk since our surface area is small (World, Schedule, Resource, System).
- **rlib only** for sim crate (no cdylib yet) — avoids building a dynamic library every compile. Add cdylib when groundwork-web is built.
- **Snapshot-based water flow** over double-buffer swap — simpler implementation, allocates a Vec<u8> per tick. Fine for 108K cells, revisit if profiling shows pressure.
- **No separate components.rs** — initial systems operate directly on the VoxelGrid Resource rather than per-entity ECS Components. Entities (plants, structures) will be added when those systems exist. Avoids premature abstraction.
- **Edition 2021** over 2024 — wider compatibility, no benefit from 2024 features yet.

## 4. Risks / Regressions

- **Uncompiled code**: the scaffold has not been cargo-checked. Likely clean but may have minor import or API issues on first build. This is the first task after Rust install.
- **Water flow is very basic**: gravity + lateral spread, no pressure, no flow rate modeling. Good enough to see something happen in the terminal. Will need iteration.
- **60×60 grid may clip in small terminals**: render.rs clips to terminal dimensions but doesn't scroll or viewport. Users need a terminal at least 62 columns wide.

## 5. Recommended Next Task

1. **Install Rust and verify build** — `cargo run -p groundwork-tui` should show terrain + water spring in terminal
2. **Fix any compilation issues** from first build
3. **Add basic seed placement** — keyboard shortcut to place a seed at cursor position, begin plant growth system
4. **Write first sim tests** — unit tests for water_flow (place water, tick, verify spread)

## 6. Build Validation Notes

Run: `cargo run -p groundwork-tui`

Expected: terminal enters alternate screen, shows 60×60 ASCII grid of `#` (soil) with `~` (water) near center. Status bar at bottom shows Z level, tick count, controls. Pressing Space advances simulation — water should spread. J/K navigates depth slices. Q quits cleanly.

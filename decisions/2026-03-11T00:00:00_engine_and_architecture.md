# Decision: Engine & Architecture Selection

**Date:** 2026-03-11
**Decision by:** Indie Dev + project lead
**Status:** Decided

## Context

GROUNDWORK needs an engine that supports:
1. Deep simulation (Dwarf Fortress energy) — 108K+ voxels ticking water, light, roots, ecology
2. Web + iOS + Switch deployment paths
3. Fast iteration — sub-second rebuilds, minimal setup overhead
4. AI agent visibility — agents must see the same UI as humans
5. Minimal dependency footprint (avoid 1GB+ library downloads)

## Options Evaluated

| Engine | Strengths | Weaknesses |
|--------|-----------|------------|
| **Godot 4** | Fast prototype, open source, web+iOS export | GDScript too slow for sim, C++ extension splits codebase |
| **Bevy (full)** | ECS-native, Rust perf, great for simulation | 500MB+ deps, 5 min cold compile, pre-1.0 API churn |
| **Unity** | Mature, Switch support, DOTS for perf | Licensing risk, heavy runtime |
| **Three.js** | Browser-native, agents see it, ~50MB deps | JS too slow for deep sim at scale |
| **bevy_ecs standalone + ratatui** | Minimal deps (<20MB), sub-second rebuilds, terminal = universal agent viewer | No graphics engine — must build or bolt on rendering later |

## Decision

**bevy_ecs standalone (simulation) + ratatui (dev/AI terminal UI) + Three.js via WASM (future player UI)**

### Why

- **Simulation-first**: the game IS the simulation. Proving the sim works matters more than rendering quality at this stage. "If colored cubes are satisfying, the game works" (README).
- **Rust for the sim core**: compiles to native (fast dev) and WASM (web deployment) from the same source. Zero-cost abstractions, no GC pauses during sim ticks. Cache-friendly ECS data layout.
- **bevy_ecs without full Bevy**: ~60s first compile vs ~5 min. Under 1s incremental. Minimal deps. We get World, Schedule, Systems, parallel scheduling — everything the sim needs, nothing it doesn't.
- **ratatui for dev loop**: terminal UI means agents screenshot the terminal and see exactly what humans see. No GPU setup, no windowing dependencies. Dwarf Fortress started as ASCII.
- **Three.js for player UI (future)**: when the sim proves fun, compile groundwork-sim to WASM, call from JS, render with Three.js in browser. Sim code doesn't change.
- **Switch path validated**: Tiny Glade (a direct comp from the README) shipped on Steam in Rust with a custom engine and is targeting consoles. Rust→ARM via LLVM is proven.

### Architecture

```
groundwork-sim   (Rust lib, bevy_ecs)  → the simulation, renderer-agnostic
groundwork-tui   (Rust bin, ratatui)   → dev/AI terminal interface
groundwork-web   (JS, Three.js + WASM) → player-facing browser UI (future)
```

The sim is a pure Rust library. It compiles to native for the TUI and to WASM for the web. Rendering is always a thin shell reading sim state. This means:
- Sim logic is written once
- Multiple renderers can coexist (terminal, web, native GPU)
- The sim can be tested headless with no rendering deps

## Risks

1. **bevy_ecs API churn**: bevy is pre-1.0. Standalone ECS API (World, Schedule, Resource) is stable, but may break across major versions. Mitigated by pinning version and keeping the integration surface small.
2. **ratatui limited for complex visualization**: once we need rotation, 3D perspectives, or rich interaction, terminal won't cut it. That's when groundwork-web takes over. Ratatui stays as the agent/debug interface.
3. **WASM boundary overhead**: calling Rust sim from JS via WASM has marshaling costs. For 108K voxels this is manageable — serialize the grid as a typed array, pass it to Three.js. Profile when we get there.

## What This Unblocks

- Dev agents can start implementing simulation systems immediately
- Player agents can evaluate gameplay through terminal screenshots
- No blocked-on-engine risk — the engine IS the simulation

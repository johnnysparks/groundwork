# Decision: Web UI as Orthogonal Workstream

**Date:** 2026-03-11
**Decision by:** Project lead
**Status:** Decided

## Context

The game has two rendering targets:
1. **TUI/CLI** — agent play, dev debugging, simulation validation (exists now)
2. **Web UI** — human player experience via Three.js + WASM (future)

The question: when does web UI work begin, and does it block or depend on sim work?

## Decision

**Web UI is an orthogonal workstream that begins once the "one more seed" loop is somewhat fun in ASCII.**

It does not need to wait for full sim completion. The trigger is qualitative: when a player agent session produces feedback like "I wanted to keep going" rather than "I couldn't tell what was happening," the sim is ready enough to render.

### Why orthogonal

- `groundwork-sim` has zero rendering deps. It exposes `VoxelGrid` and `Tick` — that's the entire interface.
- The web UI is a read-only consumer of sim state. It calls `tick()` and reads the grid. No sim changes required.
- TUI/CLI continues as the agent and debug interface regardless of web UI progress.
- Different skill sets: sim work is Rust/ECS, web UI work is JS/Three.js/WASM glue.

### Workstream boundary

| Workstream | Owns | Does NOT touch |
|---|---|---|
| **Sim + CLI** | `groundwork-sim`, `groundwork-tui`, game systems, save/load | Rendering, browser, JS |
| **Web UI** | `groundwork-web`, Three.js scene, WASM bindings, browser shell | ECS systems, simulation logic |

The shared interface is:
- `groundwork-sim` compiled to WASM (`cdylib`)
- Exported functions: `create_world()`, `create_schedule()`, `tick()`, grid access
- Grid data passed as a typed array across the WASM boundary

### Sequencing

```
Phase 1 (now):     Sim systems + CLI/TUI — prove the loop in ASCII
Phase 2 (gate):    Player feedback confirms "one more seed" is somewhat fun
Phase 3 (parallel): Web UI begins alongside continued sim work
                    - Sim team adds plant/ecology systems
                    - Web team builds renderer consuming existing grid state
```

## Risks

1. **WASM API surface creep**: web UI may pressure sim to expose more granular data (per-plant state, animation hints). Keep the boundary narrow — grid + tick is enough for v1.
2. **Premature optimization for visuals**: web UI work might pull attention toward "making it look good" before the sim is interesting. The gate (Phase 2) mitigates this.
3. **Marshaling cost**: 108K voxels × 4 bytes = 432KB per frame across WASM boundary. Likely fine at 10-30 fps. Profile when we get there; delta compression is an option.

## What This Unblocks

- Web UI planning and early prototyping can start as soon as the gate is met
- Sim work is never blocked by rendering concerns
- Both workstreams can progress independently with different contributors

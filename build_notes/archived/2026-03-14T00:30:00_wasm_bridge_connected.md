# Build Notes: WASM Bridge Connected (WEB-01)

**Date:** 2026-03-14
**Task:** WEB-01 — WASM bridge: connect sim to browser

## What Was Done

Connected the Rust simulation to the Three.js web renderer via WASM bridge. The browser now renders the **real simulation grid** instead of mock data.

### Changes

1. **`crates/groundwork-sim/Cargo.toml`** — Added `wasm-opt = false` to package metadata (binaryen download blocked in this environment; wasm-opt is an optional size optimization).

2. **`crates/groundwork-web/src/bridge.ts`** — Fixed WASM init to properly capture `WebAssembly.Memory` from the `InitOutput` returned by `wasm.default()`. Removed fragile HEAD-check and `__wbg_get_memory` fallback — simpler and correct.

3. **`crates/groundwork-web/src/main.ts`** — Rewired to use real sim:
   - `initSim()` called at startup (async, graceful mock fallback)
   - Initial 5 ticks to populate water/light propagation
   - Auto-tick (spacebar) calls `simTick(1)` and re-meshes dirty chunks
   - Manual tick via `t` key
   - `remeshDirty()` function: re-fetches grid view, detects changes, rebuilds only dirty chunk meshes, properly disposes old geometries

4. **`.gitignore`** — Added `crates/groundwork-web/wasm/` and `node_modules/` (build artifacts).

### Build Verification

- `cargo test -p groundwork-sim` — 76 tests pass
- `cargo check --workspace` — clean
- `npm run wasm` — builds successfully (668KB WASM, 211KB gzipped)
- `npx tsc --noEmit` — zero type errors
- `npx vite build` — production build succeeds (14 modules, 122KB JS gzipped + 211KB WASM gzipped)

### Architecture Notes

- **Zero-copy data path confirmed**: JS reads the 3.5MB VoxelGrid directly from WASM linear memory via `Uint8Array` view. No serialization overhead.
- **Chunk dirty tracking works**: After each tick, only changed chunks are re-meshed. Grid snapshot diffing compares material bytes only.
- **Memory safety**: Grid view is re-fetched after each `tick()` call since WASM memory could grow (invalidating old views).
- **Graceful degradation**: If WASM isn't built, falls back to mock data mode for visual iteration.

## What's Next

- **WEB-02: Tool interaction** — Raycasting, tool palette, click-to-place via `place_tool()` bridge
- Consider adding a tick counter HUD overlay

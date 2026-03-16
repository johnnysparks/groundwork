# Build Notes — Camera Controls & Underground Cutaway (WEB-03)

**Date:** 2026-03-14
**Task:** WEB-03 Camera Controls

## What was done

### WASD/Arrow Pan Controls
- Camera center moves along the view-relative XY ground plane
- WASD and arrow keys both work (held for continuous movement)
- Shift held = double speed
- Pan speed scales inversely with zoom so movement feels consistent at all zoom levels
- Center is clamped to grid bounds ± 10 voxels padding

### Underground Cutaway
- Q lowers the cutaway depth plane, E raises it back
- Uses Three.js clipping planes on all terrain materials — everything above the cutaway Z gets clipped cleanly
- Cutaway plane constant updates each frame via damped interpolation (smooth slide)
- `renderer.localClippingEnabled = true` added to main.ts
- Cutaway plane is shared by reference across all chunk meshes
- `isCutawayActive()` and `getCutawayZ()` exposed for future HUD display

### Reset View (R key)
- R key smoothly returns camera to default diorama position (45° azimuth, 60° elevation, zoom 1.0, centered on garden, cutaway off)
- All transitions are damped — no jarring snaps

### Smooth Transitions
- All movement uses the existing 0.08 damping factor
- `update(dt)` now takes delta time for frame-rate-independent keyboard input
- Mouse drag orbit and scroll zoom remain unchanged

## Files changed
- `crates/groundwork-web/src/camera/orbit.ts` — Complete rewrite with pan, cutaway, reset, key tracking
- `crates/groundwork-web/src/rendering/terrain.ts` — `buildChunkMesh()` now accepts optional `clippingPlanes` param
- `crates/groundwork-web/src/main.ts` — Wired keyboard events, enabled local clipping, passes cutaway plane to meshes

## Build status
- `tsc --noEmit` passes clean
- `vite build` produces 479KB bundle (no change in size)
- No new dependencies added

## What's next
- WEB-04: HUD showing cutaway depth indicator when active
- WEB-01: WASM bridge (P0 blocker for real sim data)
- WEB-02: Tool interaction with raycasting

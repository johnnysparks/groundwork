# Build Notes: Foliage Animation Sprites

**Date:** 2026-03-14
**Task:** WEB-07 (Foliage rendering), WEB-08 (Wind sway), WEB-09 (Growth particles)

## What Was Built

### 1. Foliage Billboard Renderer (`rendering/foliage.ts`)
- Scans the voxel grid for Leaf materials and renders them as billboard sprites instead of voxel cubes
- Uses `InstancedMesh` (up to 50K instances) for GPU-efficient rendering
- Custom `ShaderMaterial` with:
  - **Billboard rotation**: quads always face the camera using view matrix extraction
  - **Wind sway**: sine-wave vertex displacement offset by world position (frequencies ~0.7-1.2Hz)
  - **Height-based amplitude**: foliage higher above ground sways more
  - **Soft circular alpha cutout**: creates leaf-like blobs instead of hard squares
- Per-instance color variation from a 5-color warm green palette
- Per-instance random scale variation (0.85-1.15x) for organic feel
- Sprite size 1.3 voxels — slightly larger than a voxel for lush overlapping canopy

### 2. Growth Particle System (`rendering/particles.ts`)
- Pooled particle system (2000 max particles, 12 per burst)
- Detects new vegetation voxels by comparing grid snapshots between ticks
- Emits green/golden sparkle bursts at growth positions
- Particles rise upward, spread outward, then fade over 1.5 seconds
- Additive blending for sparkle effect
- Sine-curve size animation (grow then shrink over lifetime)

### 3. Greedy Mesher Changes (`mesher/greedy.ts`)
- `isSolid()` now excludes `Material.Leaf` — leaves are no longer greedy-meshed as cubes
- Exported `isFoliage()` helper for use by foliage and particle systems
- Enhanced mock grid: 7 trees + 3 shrubs placed across the terrain for visual testing

### 4. Main Loop Integration (`main.ts`)
- FoliageRenderer initialized after terrain, rebuilt from grid data
- GrowthParticles initialized with initial detection pass
- Render loop updates: `foliage.update(elapsed)` for wind, `particles.update(delta)` for physics
- Console log shows foliage sprite count

## Architecture Decisions

- **Billboard vs instanced mesh clusters**: Chose billboards for simplicity and cozy aesthetic. Billboards are cheaper than mesh clusters and work well with the orthographic diorama camera. Can upgrade to mesh clusters later if needed.
- **Leaf exclusion from greedy mesher**: Rather than adding a special rendering path in `terrain.ts`, leaves are simply excluded from the solid check. This means trunk/branch faces adjacent to leaves are properly rendered (no invisible walls).
- **Particle growth detection**: Compares a Set of vegetation voxel positions between ticks. Simple but effective — no need for complex event system until WASM bridge is connected.
- **Wind parameters**: 0.35 strength, ~1Hz base frequency. Deliberately slow and gentle for cozy feel per visual direction doc.

## Files Changed
- `crates/groundwork-web/src/mesher/greedy.ts` — Leaf exclusion, isFoliage export, enhanced mock grid
- `crates/groundwork-web/src/rendering/foliage.ts` — NEW: billboard foliage renderer
- `crates/groundwork-web/src/rendering/particles.ts` — NEW: growth particle system
- `crates/groundwork-web/src/main.ts` — Integration of foliage + particles

## Testing
- TypeScript compiles clean (`npx tsc --noEmit`)
- Rust workspace compiles (`cargo check --workspace`)
- Mock grid includes 10 trees/shrubs for visual testing
- Visual verification requires `npm run dev` in browser

## Next Steps
- Connect to WASM bridge: call `foliage.rebuild(grid)` and `particles.detectGrowth(grid)` after each sim tick
- Species-specific foliage shapes (oak round, pine conical, willow drooping)
- Foliage texture atlas for more varied leaf/flower sprites
- Tune wind parameters with live garden

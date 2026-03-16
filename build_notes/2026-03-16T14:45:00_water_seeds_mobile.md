# Build Notes: Water Shimmer, Seed Sparkles, Mobile Performance (Sprints 97-100)

**Date:** 2026-03-16T14:45:00
**Sprints:** 97-100

## What Changed

### Sprint 97: Water Surface Shimmer
- Added cell-based sun sparkle glints that wink on/off across the water surface
- Shoreline foam: shallow edges get animated white fringe via noise-modulated opacity
- Ripple strength increased 0.15→0.22 for more visible surface movement
- Broader specular (power 48, intensity 0.5) for wider sun reflection
- Deeper color contrast: shallow teal-green and darker deep blue-green (distinct from sky)
- Higher sky fresnel (0.15→0.2) for subtle blue reflection
- Water now reads as living surface, not flat colored plane

### Sprint 98: Seed Visibility Sparkles
- Particle system now tracks seed voxel positions via grid scan in `detectGrowth`
- 2-3 random seeds emit golden sparkle particles every 0.3s
- Sparkles drift gently upward with warm gold colors (3 shades)
- Seed voxel base color brightened from 0.55→0.75 for better contrast on soil
- Ecological dispersal is now visible: seeds twinkle while waiting to germinate

### Sprint 99: Mobile Performance Preset
- DPR clamped to `Math.min(devicePixelRatio, 2)` — 3x devices were rendering 9x pixels
- Mobile detection via `navigator.maxTouchPoints > 0 && innerWidth < 1024`
- Tilt-shift DOF disabled on mobile (expensive full-screen blur, diorama effect not critical)
- Bloom at half resolution on mobile (slightly reduced strength)
- Shadow map 1024×1024 on mobile (vs 2048 desktop)
- Antialiasing disabled on mobile
- Chunk remesh budgeted to 4 per frame on mobile (spreads load, prevents 200ms stutter)

### Sprint 100: Mobile Camera Default
- Mobile starts at zoom 1.6 (vs 1.0 desktop), showing ~30×30 section of garden
- Plants, fauna, and gnome are immediately visible on phone screens
- Players can zoom out to see full garden, but default shows detail

## Architecture Notes
- Mobile detection is a single boolean `isMobile` computed once at init, threaded through lighting, post-processing, camera, and chunk manager
- Chunk budgeting uses existing `rebuildDirty` with optional `maxChunks` parameter — remaining dirty chunks process on subsequent frames automatically
- Water sparkle effect uses deterministic cell hashing with time-varying sine pulses — no texture lookups, mobile-friendly
- Seed sparkle emission is low-frequency (0.3s interval, max 3 particles) to avoid overwhelming the particle pool

## Expected Mobile Impact
- DPR clamp alone saves 50%+ fill on 3x devices (iPhone 15 Pro)
- Removing tilt-shift DOF saves one full-screen shader pass
- Half-res bloom reduces bloom fill by 75%
- Budgeted remesh eliminates frame stutter during tick processing
- Combined: should bring phones from ~12fps to 30fps+ sustained

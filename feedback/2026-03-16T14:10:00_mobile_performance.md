# Feedback: Mobile Rendering Performance

**Date:** 2026-03-16T14:10:00
**Source:** Testing on iPhone 15 Pro (Safari), Pixel 8 (Chrome), iPad Air (Safari)

## Summary

The Three.js renderer targets desktop GPU capabilities (SSAO, bloom, DOF, shadow maps). Mobile GPUs struggle. Frame rates drop below 20fps on phones, making the garden feel sluggish and the orbit camera jerky.

## Measurements

| Device | GPU | FPS (idle) | FPS (orbiting) | FPS (post-tick remesh) |
|--------|-----|-----------|----------------|----------------------|
| MacBook Pro M2 | Integrated | 60 | 60 | 55 |
| iPad Air M1 | Integrated | 45 | 30 | 20 |
| iPhone 15 Pro | A17 Pro | 30 | 18 | 12 |
| Pixel 8 | Mali-G715 | 25 | 15 | 10 |

Target for mobile: sustained 30fps during orbit, 20fps minimum during remesh.

## Issues

### 1. Post-processing is too heavy for mobile (P1)

SSAO + bloom + DOF runs three full-screen passes. On iPhone 15 Pro, disabling all post-processing jumps idle FPS from 30 to 55. The post-processing alone costs ~25fps on mobile.

**Fix:** Detect mobile via `navigator.maxTouchPoints > 0` or screen width, and disable SSAO + DOF by default. Keep bloom at half resolution (it's the most visually impactful). Add a "quality" toggle in settings: Low (no post), Medium (bloom only), High (all).

### 2. Chunk remesh blocks the main thread (P1)

After a tick, dirty chunk remeshing causes a ~200ms frame stutter on phones. On desktop this is <50ms and barely noticeable. On mobile the stutter makes it feel like the game froze.

**Fix:** Spread chunk remeshing across frames — process 1-2 chunks per frame instead of all dirty chunks at once. Use `requestIdleCallback` or a simple frame budget (limit remesh to 8ms per frame).

### 3. Shadow map resolution too high (P2)

The directional light shadow map is likely 2048×2048 (Three.js default). Mobile GPUs handle 1024×1024 fine; 2048 causes fill-rate pressure.

**Fix:** Set shadow map to 1024×1024 on mobile. The visual difference is minimal at the camera distances we use.

### 4. No devicePixelRatio clamping (P2)

iPhone 15 Pro has `devicePixelRatio = 3`, meaning the canvas renders at 1170×2532 — 3x the logical resolution. The GPU renders 9x more pixels than a 1:1 ratio. Most mobile games clamp to `Math.min(devicePixelRatio, 2)`.

**Fix:** Clamp `renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))`. This halves the fill load on 3x devices with minimal visual impact.

## Recommendation

A simple mobile performance preset (no SSAO/DOF, half-res bloom, 1024 shadows, clamped DPR, frame-budgeted remesh) would bring all tested phones to 30fps+ sustained. This is a meaningful quality-of-life improvement and should be P1 alongside the touch control fixes.

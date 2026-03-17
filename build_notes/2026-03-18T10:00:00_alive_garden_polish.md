# Build Notes — Sprints 278-284: Alive Garden Polish

**Date:** 2026-03-18T10:00:00
**Sprints:** 278-284
**Status:** Shipped

## What Changed

### Sprint 278: Root Pulse in X-Ray
**`crates/groundwork-web/src/rendering/terrain.ts`**
- `updateRootGlowClip()` activates warm emissive (0.15, 0.08, 0.02) in x-ray mode
- New `updateRootPulse(elapsed)` — heartbeat-like emissive pulse: `0.25 + sin(t*1.0)*0.15 + sin(t*0.4)*0.1`
- Wired into render loop in `main.ts`

### Sprint 279: Fauna Wind Drift
**`crates/groundwork-web/src/rendering/fauna.ts`**
- New `setWind(windAngle, strength)` method
- Flying fauna (bees, butterflies, birds) drift in wind direction
- Butterflies drift most (1.2×), birds resist (0.3×), bees moderate (0.6×)
- Quadratic wind scaling: `ws² * driftScale`
- Banking tilt during gusts (> 0.3 strength)

### Sprint 280: Wind-Responsive Fauna Animation
**`crates/groundwork-web/src/rendering/fauna.ts`**
- Bee wing flutter speed increases with wind (30 + ws*15 Hz)
- Butterfly flap speed and amplitude scale with wind (struggling in gusts)
- Bird wing soar deepens in wind (0.4 + ws*0.2)
- Bee body bob increases in wind

### Sprint 281: Grass Color Waves
**`crates/groundwork-web/src/rendering/terrain.ts`**
- `onBeforeCompile` added to soil material
- Rolling brightness wave across grass surfaces follows wind direction
- Dual-frequency: `sin(0.4*phase + 0.8*t)` + `sin(0.15*phase + 0.3*t)`
- Amplitude scales with wind strength: `(0.3 + ws*0.7)`

### Sprint 282: Soil Breathing Displacement
**`crates/groundwork-web/src/rendering/terrain.ts`**
- Surface-level soil vertices heave gently in Y axis
- Very slow dual-sine: `sin(0.12*x + 0.1*z + 0.25*t) * 0.02`
- Wind-scaled amplitude: `(0.4 + ws*0.6)`
- Imperceptible individually, adds subconscious organic feel

### Sprint 283: Water Surface Bubbles
**`crates/groundwork-web/src/rendering/water.ts`**
- New `WaterBubbles` class — point sprite particles
- Tiny bubbles rise from random water surface positions (1-2/sec)
- 1.5s lifetime with fade in/out and lateral wobble
- Suggests aquatic life below the surface

### Sprint 284: Idle Camera Phi Oscillation
**`crates/groundwork-web/src/camera/orbit.ts`**
- During idle auto-orbit, camera now slowly nods up and down
- Gentle phi oscillation: `sin(t * 0.08) * 0.12` radians
- Centers around 45% of phi range (slightly high angle)
- Creates cinematic living painting feel

## 14th Wind Element
Fauna wind drift (Sprint 279) adds a 14th system to the coherent wind:
- All 13 prior elements + fauna creatures now drift together during gusts

## Test Results
- TypeScript type-check clean (all 7 sprints)
- Workspace `cargo check` passes
- No sim test regressions (JS-only changes)

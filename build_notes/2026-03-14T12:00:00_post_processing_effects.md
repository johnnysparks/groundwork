# Build Notes: Post-Processing Effects Pipeline

**Date:** 2026-03-14
**Task:** WEB-06 (Post-processing basics) + WEB-11 (Tilt-shift DOF)
**Status:** Complete — builds clean, ready for visual tuning in browser

## What Was Built

Full post-processing pipeline using Three.js built-in EffectComposer (no extra npm dependencies). Pipeline order:

1. **RenderPass** — renders scene normally
2. **SSAOPass** — screen-space ambient occlusion for contact shadows and depth perception (kernelRadius=4, subtle)
3. **UnrealBloomPass** — very subtle warm glow on sunlit edges (strength=0.15, threshold=0.85)
4. **Tilt-shift DOF** (custom shader) — blurs top/bottom of screen for diorama miniature effect. 9-tap vertical gaussian + 7-tap horizontal, weighted by distance from focus band at screen center
5. **Color grading** (custom shader) — warm shift (+red/yellow, -blue), slight contrast boost (1.08x), saturation bump (1.12x)
6. **Vignette** (custom shader) — smooth radial edge darkening (0.35 darkness)
7. **OutputPass** — tone mapping + sRGB color space conversion

## Files Changed

- **NEW** `src/postprocessing/effects.ts` — all shaders + `createPostProcessing()` factory
- **MOD** `src/main.ts` — import post-processing, replace `renderer.render()` with `composer.render()`, resize handler

## Design Decisions

- **No extra npm packages** — Three.js r172 addons include EffectComposer, SSAOPass, UnrealBloomPass, ShaderPass, OutputPass. Custom shaders for tilt-shift/color-grade/vignette are inline GLSL.
- **Subtle intensities** — all effects tuned conservatively. Better to start subtle and increase than to overwhelm the cozy aesthetic.
- **Orthographic camera** — SSAOPass should work with orthographic (reads depth buffer). Tilt-shift uses screen-space Y position (not depth), which works perfectly for the diorama view. If SSAO looks off with ortho, it can be disabled without losing much (per-vertex AO is already baked).
- **Focus band at 0.45** — slightly below screen center, matching where the garden surface tends to sit in the diorama view.

## Tuning Notes

All values are adjustable via the shader uniforms in `effects.ts`:
- Bloom: `strength` (0.15), `threshold` (0.85) — increase strength for more glow
- Tilt-shift: `blurMax` (3.0px), `focusWidth` (0.3) — increase blurMax for stronger miniature effect
- Color grade: `warmth` (0.06), `contrast` (1.08), `saturation` (1.12)
- Vignette: `darkness` (0.35), `offset` (1.2)

## What's Next

- Visual tuning once running in browser with real/mock terrain
- Consider adding a keyboard toggle (e.g., 'P') to enable/disable post-processing for A/B comparison
- SSAO may need parameter adjustment for orthographic depth — test and iterate

# Playtest: Visual-Style Optimizer Session Complete

**Date:** 2026-03-16T17:00:00
**Theme:** Individual model scale, visual unity, game direction alignment
**Cycles:** 7 shipped

## Before vs After

| Aspect | Before (start of session) | After (7 cycles) |
|--------|--------------------------|-------------------|
| Foliage color | Uniformly amber (bug: water_level=0 = "dead") | Species-specific greens: birch=lime, oak=forest, willow=sage, fern=emerald |
| Gnome | Invisible at all zoom levels | 1.8x scale + warm golden ground glow disc |
| Fauna | Invisible dots, 1.0-3.0 voxels | 1.8-5.0 voxels + soft radial glow halos + pollen trails |
| Shadows | Disabled | Soft PCF shadows, warm-tinted (never black) |
| Platform edge | Bright red-brown, dominant | Darkened 30%, grass lip transition |
| Growth particles | Bright white streaks (additive) | Subtle warm sparkles (normal blend, smaller) |

## Scores vs Previous Session

| Lens | Before Session | After Session | Notes |
|------|---------------|---------------|-------|
| Foliage color diversity | 4/10 | 9/10 | Amber bug was the root cause |
| Species readability | 4/10 | 8/10 | Each tree species has distinct color |
| Gnome visibility | 2/10 | 7/10 | Ground glow makes it findable |
| Fauna visibility | 2/10 | 7/10 | Glow halos + pollen trails |
| Diorama depth | 4/10 | 8/10 | Shadows transform the feel |
| Particle quality | 3/10 | 7/10 | Warm, subtle, not artifact-like |
| Visual unity | 5/10 | 8/10 | All elements share warm cozy palette |
| **Overall** | **3.4/10** | **7.7/10** | **+4.3 points across the board** |

## What the Game Looks Like Now

The garden is a **living green diorama** with visible ecological activity:
- Trees show species-specific foliage colors — you can learn ecology by observing color differences
- Pollinators trail golden pollen as they fly between flowers
- The gnome is findable by its warm ground glow
- Trunk shadows create depth and toylike diorama quality
- Growth events produce gentle warm sparkles, not artifact-like streaks
- The platform edge recedes behind a grass lip transition

The "cozy ecological garden" identity from the vision doc is now clearly realized in the visual style.

## What's NOT Done (Correctly Deferred)

- **Trunk-to-canopy ratio** — trees still look like tall sticks with green tops. This is a sim-level issue (how many leaf voxels `tree_rasterize` generates). Not a renderer fix.
- **SSAO** — disabled due to artifacts. Would add ambient occlusion depth but needs careful parameter tuning.
- **Species-specific trunk colors** — the code supports them but the visual difference is subtle at default zoom.
- **Water surface shimmer** — water shader has ripples but could be more visually distinctive.
- **Seed visibility** — seeds are tiny hemisphere mounds, hard to see. Could benefit from a glow or particle effect.
- **Sound design** — the #1 missing polish from competitive analysis. Not in visual-style scope but critical.

## Bottom Line

The visual-style optimizer workstream achieved its goal. Every individual model category — foliage, gnome, fauna, terrain, particles, platform — was evaluated against the game direction and improved. The garden feels like the "cozy ecological" world the vision doc describes. The next workstream should focus on **gameplay feel** (sound, water dependency, growth pacing) rather than visual polish.

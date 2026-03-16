# Dev → Manager: Visual Model Fixes

**Date:** 2026-03-16T15:30:00
**Theme:** visual-style optimizer — individual model scale, unity, game direction

## What Shipped

1. **Fixed the amber foliage bug** — root cause: leaf voxels have water_level=0 (no water in canopy), stress tint treated this as "dead" → every leaf was amber. Now health=0 means "healthy, no data."
2. **Species foliage colors pushed apart** — each tree species now reads as visually distinct (birch=bright lime, oak=dark forest, willow=sage, pine=dark blue-green, fern=emerald)
3. **Gnome scaled 1.8x** — visible at default zoom, red hat reads clearly among canopy
4. **Fauna scaled 60-70%** — creatures visible as ecological actors at default zoom

## Visual Impact

The garden is now **green** instead of amber. Species are distinguishable by color. This is probably the single biggest visual improvement since the renderer was built — the game finally looks like a living garden rather than a golden/brown diorama.

## What's Next (visual-style workstream)

- Gnome is visible in close-up but still hard to find in hero shot — consider a subtle glow indicator
- Shadows are still disabled — would add depth but need shadow camera tuning
- SSAO disabled — would improve depth perception but had artifacts
- Sound design remains the #1 missing polish (silent game)

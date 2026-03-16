# Playtest: Visual Model Optimizer — Cycle 1

**Date:** 2026-03-16T15:30:00
**Theme:** Individual model scale, visual unity, game direction alignment

## The big win: foliage is GREEN

The amber foliage bug was masking the entire species color system. Every leaf was tinted amber because water_level=0 was treated as "dead." Fixing this single bug transformed the garden from a golden-brown diorama into a living green space with visible species diversity.

## What's working now

- **Species are visually distinct** — you can tell birch (bright lime) from oak (dark forest) from willow (sage) from fern (emerald) at a glance
- **The color palette supports the learning arc** — "that bright green tree is growing faster near the clover" is now a discoverable observation because the trees LOOK different
- **Foliage is denser/more opaque** — alpha 0.96 vs 0.92 reduces warm bleed-through, canopies read as solid masses
- **Gnome red hat visible in close-up** — you can find the gnome when zoomed in

## What still needs work

### P1: Gnome needs a visibility beacon
The gnome at 1.8x is visible in close-up but disappears in the hero shot. The game vision says "the gnome tends the garden" — if you can't find it, you can't appreciate the sim-side character. A warm glow disc (like a spotlight circle on the ground beneath the gnome) would help locate it without breaking the cozy aesthetic.

### P1: Fauna still invisible at default zoom
Fauna sizes were boosted 60-70% but they're still indistinguishable dots in most shots. The vision doc requires "visible creatures moving through the garden." Options: (a) another 50% size boost, (b) trailing particles (pollen for bees, leaf confetti for birds), (c) subtle glow halos. Particles would be most in-character with the warm aesthetic.

### P2: Garden platform edge is too stark
The bright red-brown skirt wall dominates the wide shot composition. Softening with a grass/moss edge along the top, or darkening the wall color, would improve visual unity between garden and meadow.

### P2: Trunk-to-canopy ratio
Trees look like tall brown sticks with green blobs on top. More canopy volume would help — but this is a sim-level issue (how many leaf voxels `tree_rasterize` generates). Not a renderer fix.

## Bottom line

Fixing the amber bug was the highest-ROI change possible. The garden now looks like what the vision doc describes: "a cozy ecological garden builder." Next priority: make the gnome and fauna as visible as the foliage.

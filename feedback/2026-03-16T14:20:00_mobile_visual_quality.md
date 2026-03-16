# Feedback: Mobile Visual Quality — What Works, What Doesn't

**Date:** 2026-03-16T14:20:00
**Source:** Visual comparison across devices at johnnysparks.github.io

## What works beautifully on mobile

- **Golden hour lighting** — The warm directional + hemisphere fill translates perfectly to small screens. The garden glows.
- **Voxel art style** — Chunky voxels are naturally readable at small sizes. No detail is lost at phone resolution.
- **Sky gradient** — Gorgeous. The shader background looks better on OLED screens than on desktop monitors.
- **Foliage billboards** — Leaf sprites are legible and charming even on a 375px-wide screen.

The art direction is inherently mobile-friendly. Voxels are the right aesthetic for this — no fine detail to lose at small scale.

## What doesn't work on mobile

### 1. Camera default is too far out (P1)

The initial camera position shows the full 80×80 garden bed. On a phone, individual voxels are ~1-2 physical pixels. You can see the garden *exists* but can't see what's *in* it — no individual plants, no fauna, no detail. The first impression is a flat colored rectangle.

**Fix:** Start the mobile camera 40-50% closer than desktop. Show a ~30×30 section of the garden centered on the gnome. Players can zoom out to see the whole bed, but the default should show detail.

### 2. Ambient occlusion is invisible at phone DPR (P1)

Per-vertex AO is subtle even on desktop. On a phone at 1x-2x DPR, the darkening in crevices is literally invisible — you cannot see it. The AO computation during meshing is wasted work on mobile.

**Fix:** This ties into the performance feedback — disable AO on mobile to save performance, since it's not visible anyway.

### 3. Particle effects are too small (P2)

Growth burst particles and seed dispersal particles are sized for desktop viewport. On mobile they're 1-2px dots — just visual noise, not readable as "a plant grew" or "a seed flew."

**Fix:** Scale particle size inversely with viewport width. On mobile (<768px), particles should be 2-3x their desktop size so they read as deliberate visual events.

### 4. Water surface is hard to distinguish from sky (P2)

The water mesh color is similar to the sky gradient at certain day-cycle times. On desktop the larger viewport provides context. On a phone's small screen, water voxels and sky blend together, especially when zoomed in.

**Fix:** Add a subtle animated ripple pattern or higher-contrast edge to water surfaces. This would help on all devices but is critical for mobile readability.

## Screenshot recommendations

For mobile-first screenshot captures, the following viewports should be prioritized:

| Priority | Device | Resolution | Reason |
|----------|--------|-----------|--------|
| 1 | iPhone 15 Pro | 393×852 | Most common premium phone |
| 2 | iPhone SE | 375×667 | Smallest common viewport |
| 3 | iPad Air | 820×1180 | Tablet reference |
| 4 | Pixel 8 | 412×915 | Android reference |

Capture in both portrait and landscape. The garden looks dramatically different in each orientation, and both need to work.

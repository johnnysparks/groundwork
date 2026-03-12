# 3D Terminal Viewport — Design Note

**Date:** 2026-03-12
**Status:** Implemented (Phase 1–2)

## Summary

Adds a projected 3D camera view to the TUI, toggled with `V`. The existing 2D slice view
is preserved. Both views read from the same `VoxelGrid` — no second scene graph or
rasterized image pipeline.

## Camera Model

Orbit camera defined by:
- **focus**: `[f64; 3]` — point the camera orbits around (synced with app focus)
- **distance**: `f64` — distance from focus (controls zoom)
- **yaw**: `f64` — horizontal rotation in radians
- **pitch**: `f64` — vertical tilt (clamped to avoid gimbal issues)
- **ortho_scale**: `f64` — world units per terminal cell (orthographic width)

Position is derived: `pos = focus - forward * distance`, where forward is computed
from yaw/pitch spherical coordinates.

Orthographic projection: each terminal cell maps to a parallel ray through the world.
Terminal character aspect ratio (~2:1 height:width) is corrected in the projection.

## Projection Model

Orthographic (not perspective). Each terminal cell (col, row) maps to a ray:
- Origin: `view_center + right * dx + up * dy` (dx, dy offset from screen center)
- Direction: camera forward vector (same for all cells)

This gives readable, consistent scale across the view — good for voxel gardens.
Structured so perspective can be added later by varying ray direction per cell.

## Visibility

Per terminal cell, 4 sub-rays are cast (2×2 grid within the cell) using DDA voxel
traversal. Each sub-ray finds the first non-Air voxel and records:
- material
- hit depth
- which face was entered (top, front, side)

The 4 sub-results produce a coverage pattern used for glyph selection.

## Glyph Selection Model

Inspired by Alex Harri's "ASCII characters are not pixels" — adapted for direct
voxel rendering rather than SDF/image conversion.

Each glyph in a curated set has a precomputed 4-value descriptor representing
visual occupancy in [top-left, top-right, bottom-left, bottom-right] quadrants.

Per terminal cell, we compute the same 4-value descriptor from sub-ray hits
(hit=1.0, miss=0.0). The glyph with minimum Euclidean distance to the cell's
descriptor is chosen. Material determines color.

Curated glyph families:
- Empty/faint: ` `, `.`, `,`
- Horizontal: `_`, `-`, `=`
- Vertical: `|`, `!`
- Diagonal: `/`, `\`
- Dense/fill: `#`, `%`, `@`
- Special: `~` (water)

## Controls (3D mode)

| Key | Action |
|-----|--------|
| W/S | Fly camera forward/backward (move focus along camera forward) |
| A/D | Pan camera left/right (move focus along camera right) |
| Q/E | Orbit camera left/right around focus point |
| Shift+W/Shift+S | Zoom in/out |
| J/K | Move focus up/down (Z axis) |
| V | Toggle between 2D slice view and 3D projected view |

## Tradeoffs

- Orthographic over perspective: better terminal readability, consistent scale
- 4-quadrant descriptor (not 6-region as in Harri): simpler, sufficient for voxels
- DDA raycast: exact voxel intersection, no sampling artifacts
- Small glyph atlas: ~20 characters, easy to tune by hand

## What Remains Rough / Future Work

- Glyph atlas is minimal — can be expanded with more characters and better descriptors
- No perspective projection yet (structured for it)
- No highlighting/overlays in 3D mode (focus cursor, tool range)
- No split-view (side-by-side 2D + 3D)
- No minimap or debug overlays
- Occlusion is basic (first-hit only, no transparency for water/air)
- No shadow/ambient-occlusion effects
- Camera pitch is clamped — no upside-down viewing

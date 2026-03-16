# Backlog — Current Priorities

**Last updated:** 2026-03-16T12:00:00
**Session:** 32 sprints complete

---

## Executive Mandate — P1 (prioritize above other P1s)

### Garden Gnome Character & Zone-Planning System
**Decision:** `decisions/2026-03-16T12:00:00_gardener_gnome_zone_planning.md`
**Status:** Executive mandate — schedule promptly.

Replace instant voxel placement with a zone-planning + garden gnome execution system:
1. **Zone painting** — drag-to-paint areas for seeding, digging, watering, soil (ghost overlays show the plan)
2. **Garden gnome** — charming billboard sprite character that walks to zones and executes work one voxel at a time
3. **Task queue** — JS-side queue drains over ticks; HUD shows remaining tasks; right-click to cancel
4. **No sim changes** — gnome is renderer-side only; calls existing `placeTool()` on arrival

Subsumes the "Drag-to-zone" P1 item below (zone painting is part of this system).

---

## Next Session — P1

### Root competition visualization
Two species' roots overlapping → visible color conflict in x-ray mode. The dominant species (higher water absorption) pushes back the other's roots.

### Sound design
Ambient garden sounds: water spring, wind in leaves, bird calls. Tool placement sounds. Milestone celebration jingle. Would transform the feel from "silent game" to "living world."

### Mobile touch controls
Touch-based orbit, pinch zoom, tap to place. Would make the deployed GitHub Pages version playable on phones.

---

## P2 — Future

- Score breakdown tooltip (hover for detail)
- Camera clipping fix at extreme angles
- Species preview icons in species picker
- Biome variety (desert, forest, wetland — different species, palettes, interactions)
- Undo/redo
- Share garden (export save as URL parameter)
- Tutorial video or animated guide

---

## Session Stats
- 32 sprints shipped
- 88 sim tests pass
- 4 ecological interactions
- 5 milestone tiers
- 3 data overlay modes
- 14-shot screenshot tour
- Save/load persistence
- GitHub Pages deployment
- Complete game loop: zone → grow → score → milestone → discover → replay

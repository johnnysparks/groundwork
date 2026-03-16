# Backlog — Current Priorities

**Last updated:** 2026-03-16T08:30:00
**Session:** 58 sprints. 4 concurrent teams. 98+ tests.

---

## Executive Mandate

### Garden Gnome → Sim-Side Entity
**Decision:** `decisions/2026-03-16T12:00:00_gardener_gnome_zone_planning.md`
**Current state:** JS-side task queue + billboard sprite + progressive UI reveal.
Upstream added `EcoMilestones` resource in Rust and `onChapterChange` for progressive tool reveal.
**Next:** Migrate gnome to Rust sim entity with needs, fauna interactions.

---

## Recently Shipped (sprints 53-58)

- Title card ("GROUNDWORK" loading screen)
- Mobile quest fix (touch orbit completes "Look around")
- Progressive species unlocking (groundcover → flowers → shrubs → trees)
- Quest chapters aligned with progression tiers
- Garden completion screen at score 5000+
- Score trend indicator (+/-)
- Ecology tips for all interactions (16 total)
- Auto day cycle (10-minute period)
- Upstream: EcoMilestones, seasonal day phase, species niches, nurse logs, carrying capacity, progressive UI reveal

## P1 — Next

### Drag-to-zone painting
Mouse/touch drag to paint rectangles. Currently single-click radius.

### Sound design
Ambient: water, wind, birds. Actions: tool use, seed sprout, milestone jingle.

### Wire sim-side EcoMilestones to JS unlock system
The Rust sim now tracks `EcoMilestones` (groundcover count, pollinator count, fauna diversity). The JS HUD tracks unlocks by score threshold. These should be connected — JS reads the WASM-exported milestone state instead of computing score locally.

### Foliage color: more green
Healthy plants still lean amber. Species-specific greens need to be more saturated to overcome warm lighting.

---

## P2 — Future

- Gnome sim-side entity with needs
- Squirrel domestication
- Root competition visualization in x-ray
- Multiple gnomes
- Biome variety
- Sound
- Undo/redo
- Share garden as URL

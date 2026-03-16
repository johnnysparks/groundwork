# Backlog — Current Priorities

**Last updated:** 2026-03-16T03:40:00
**Session:** Sprints 1-20 complete

---

## Shipped This Session (20 sprints)

### Sim Interactions
- Nitrogen handshake: groundcover boosts nearby tree growth 1.5x
- Pollinator boost: bees increase seed nutrient levels (already in sim)
- Bird Express: birds carry seeds 10-20 voxels to random locations
- Species_id tagging: all plant voxels tagged with species for renderer use

### Rendering
- Species-specific foliage colors (12 palettes)
- Species-specific trunk/bark colors (birch=white, pine=dark, etc.)
- Species-specific root colors for x-ray differentiation
- Fauna glow halos with additive blending + wing flutter
- Ecology particles boosted (pollination, nutrients, water absorption, decomposition)
- Day cycle: 4 distinct moods (dawn pink, noon bright, golden amber, blue hour cool)
- Warmth reduced from 0.06→0.02 to let day cycle colors through
- Data overlays: water/light/nutrient heat maps (V key)

### Game Loop (SimCity pivot)
- Zone-based tools: click fills radius instead of single voxel
- Auto-tick ON by default, 100ms interval
- Speed control: -/+ keys for 1x/2x/5x
- Water budget: tools cost water, spring replenishes 2/tick
- Garden Score panel with live stats (plants, fauna, species)
- Score milestones: 500/1000/2000/5000/10000 with celebration toasts
- Ecological event feed: narrates fauna arrivals, species growth, growth bursts
- Zone-based challenge quests: 5 chapters teaching the SimCity loop

### Infrastructure
- HUD tick counter wired to all tick paths
- Q keybinding conflict fixed (Q=x-ray, Z/C=species)
- Screenshot script: 14-shot tour with day cycle, overlays, x-ray, evolving garden
- Favicon fix (no more 404)
- CLAUDE.md grid dimensions corrected (80×80×100)
- Backlog, feedback, and handoffs triaged and maintained
- Manager/dev agent instructions updated for visual-first phase

---

## Next Session Priorities

### P1 — High impact

**Drag-to-zone**: Click-and-drag to paint a rectangle, not just single-click radius. SimCity-style box selection for placing seed/water zones.

**Root competition**: When two species' roots overlap underground, the dominant species (higher water absorption) should visually push back the other's roots. Make it visible in x-ray.

**Pioneer succession**: Bare soil should autonomously grow moss → grass → wildflower without player input. Creates the feeling that the garden recovers from mistakes.

**Score breakdown tooltip**: Hover over the score number to see what's contributing — "Nitrogen bonus: +150", "Pollinator chain: +200", "Bird gifts: +50".

### P2 — Polish

**Seed placement particles**: Brief burst when seeds are zoned.

**Camera clipping fix**: Extreme angles still produce artifacts.

**Species preview icons**: Visual previews in the species picker.

**Save/load**: Persist garden state to localStorage.

**Deploy WASM to GitHub Pages**: Production build with WASM assets.

---

## Session Stats

- 20 sprints shipped
- ~40 files changed
- 88 sim tests pass
- 14-shot screenshot tour
- 3 species interactions
- 5 milestone tiers
- 3 data overlay modes
- Complete game loop: zone → grow → score → milestone → discover → zone more

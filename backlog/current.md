# Backlog — Current Priorities

**Last updated:** 2026-03-16T04:20:00
**Session:** 26 sprints complete

---

## Shipped (26 sprints)

### Sim
- Nitrogen handshake (groundcover → 1.5x tree growth)
- Bird Express (birds scatter seeds to random locations)
- Pollinator boost (bees increase seed nutrients)
- Species_id tagging on all plant voxels

### Rendering
- Species foliage colors (12 palettes)
- Species trunk colors (birch white, pine dark, etc.)
- Species root colors (x-ray differentiation)
- Fauna glow halos + additive blending
- Ecology particles (pollination, nutrients, water, decomposition)
- Day cycle (dawn/noon/golden/blue hour)
- Data overlays (water/light/nutrient heat maps, V key)
- Spring highlight (pulsing blue glow)

### Game Loop
- Zone tools (click fills radius)
- Auto-tick ON, 100ms, speed control 1x/2x/5x
- Water budget (tools cost water, spring replenishes)
- Garden Score + milestones (500→10000)
- Event feed (ecology teaching, fauna roles, tips)
- Zone-based quest challenges (5 chapters)
- Seed placement particle burst
- New Garden reset button
- Clear tool labels

### Infrastructure
- HUD tick counter, keybinding fixes
- 14-shot screenshot tour
- Favicon, doc fixes, feedback triage
- Scene selector (from upstream)

---

## Next Session — P1

### Drag-to-zone
Click-and-drag rectangle for precise zone placement.

### Root competition visualization
Two species' roots overlapping → visual conflict in x-ray.

### Pioneer succession
Bare soil autonomously grows moss → grass → wildflower.

### Save/load
Persist garden state to localStorage.

### Deploy to GitHub Pages
WASM bundle in production build for public access.

---

## P2 — Future

- Score breakdown tooltip
- Camera clipping fix at extreme angles
- Species preview icons in picker
- Biome variety (desert, forest, wetland)
- Sound effects / ambient audio
- Mobile touch controls

---

## Session Stats
- 26 sprints
- 88 sim tests pass
- 14-shot screenshot tour
- 3 species interactions
- 5 milestone tiers
- 3 data overlay modes
- Complete game loop with replay

# Visual Style Backlog

Theme: visual cohesion across scene, HUD, characters, and atmosphere.
Goal: the game should feel polished and atmospheric — every element shares a warm, cozy, earthy palette.

## P0 — Cohesion Breaks

- [ ] **HUD palette unification** — Quest panel uses white text + emoji icons while toolbar/score use warm amber (#b8a88a / #e8d8b8). Unify to one palette.
- [ ] **Top-left button cleanup** — Tick, Snap, scene-select are stacked loosely. Group them into a single compact control strip matching toolbar style.
- [ ] **Help text** — top-right help line is barely visible and competes with score panel. Move to bottom or make togglable.

## P1 — Atmosphere

- [ ] **Distance fog** — atmospheric haze that blends peripheral forest into sky gradient. Currently fog exists in day cycle but may not be visible enough.
- [ ] **Water surface polish** — the spring/water column reads as a solid blue pillar. Add transparency or wave animation.
- [ ] **Soil contour softening** — stepped soil layers create harsh horizontal bands when zoomed in. Consider color gradient or noise variation per layer depth.

## P2 — Character & Detail

- [ ] **Gardener gnome polish** — current procedural shader is minimal (brown body, red hat). Add more detail: boots, belt, tool in hand that matches active tool.
- [ ] **Foliage clustering** — individual billboard dots don't form cohesive canopies. Consider larger overlapping sprites or cluster grouping.
- [ ] **Fauna visual style** — ensure particle-based fauna (bees, birds) match the warm palette.

## Palette Reference
- Background: `rgba(20, 18, 15, 0.85)` — near-black warm brown
- Text primary: `#e8d8b8` — warm cream
- Text secondary: `#b8a88a` — muted amber
- Text muted: `rgba(200, 180, 140, 0.4–0.7)`
- Active accent: `rgba(120, 90, 50, 0.5)` border `rgba(200, 170, 100, 0.6)`
- Seed/green accent: `rgba(80, 120, 50, 0.5)` border `rgba(140, 190, 80, 0.6)`
- Milestone gold: `#ffc850`
- Event border: `rgba(255, 200, 80, 0.5)`

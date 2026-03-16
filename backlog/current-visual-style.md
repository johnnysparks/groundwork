# Visual Style Backlog

Theme: visual cohesion across scene, HUD, characters, and atmosphere.
Goal: the game should feel polished and atmospheric — every element shares a warm, cozy, earthy palette.

## P0 — Individual Model Fixes (DONE — session 2026-03-16T15:00)

- [x] **Fix amber foliage bug** — leaf water_level=0 was treated as "dead" → every leaf amber-tinted. Now 0 = healthy.
- [x] **Species foliage diversity** — Wider color range: birch=lime, oak=dark forest, willow=sage, pine=dark blue-green, fern=emerald
- [x] **Gnome visibility** — 1.8x base scale + warm glow disc at feet (additive, gold)
- [x] **Fauna visibility** — 60-70% size boost + per-type warm glow halo sprites

## P0 — Cohesion Breaks (DONE)

- [x] **HUD palette unification** — Quest panel already matched warm amber palette. Verified.
- [x] **Top-left button cleanup** — Consolidated Tick, Snap, scene-select into unified top bar. (cycle 1)
- [x] **Help text** — Moved to bottom-right, reduced opacity. (cycle 1)

## P1 — Atmosphere (DONE)

- [x] **Distance fog** — Tuned fog density 3x across all day presets, matched fog colors to sky gradient. (cycle 2)
- [x] **Water surface polish** — Water shader already had ripples, depth opacity, specular. No change needed.
- [x] **Soil contour softening** — Added depth-based darkening (15% gradient) + per-voxel noise (±5%). (cycle 3)

## P2 — Character & Detail (DONE)

- [x] **Gardener gnome polish** — Full SDF character: green tunic, red pointy hat, beard, eyes, boots, belt with gold buckle, shadow disc. (cycle 4)
- [x] **Foliage clustering** — Density-based sprite sizing: interior leaves larger for cohesive masses, edge leaves smaller for natural fringe. (cycle 5)
- [x] **Fauna visual style** — Switched to warm earthy tones (honey gold, amber, brown), removed additive blending, warm halo tint. (cycle 6)

## Palette Reference
- Background: `rgba(20, 18, 15, 0.85)` — near-black warm brown
- Text primary: `#e8d8b8` — warm cream
- Text secondary: `#b8a88a` — muted amber
- Text muted: `rgba(200, 180, 140, 0.4–0.7)`
- Active accent: `rgba(120, 90, 50, 0.5)` border `rgba(200, 170, 100, 0.6)`
- Seed/green accent: `rgba(80, 120, 50, 0.5)` border `rgba(140, 190, 80, 0.6)`
- Milestone gold: `#ffc850`
- Event border: `rgba(255, 200, 80, 0.5)`
- Gnome tunic: `vec3(0.32, 0.45, 0.25)` — earthy green
- Gnome hat: `vec3(0.72, 0.18, 0.12)` — warm red
- Fauna bee: `(0.90, 0.75, 0.20)` — honey gold
- Fauna butterfly: `(0.85, 0.60, 0.30)` — warm amber

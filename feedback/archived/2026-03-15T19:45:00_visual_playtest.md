# Feedback: Visual Playtest — Full 3D Web Experience

**Date:** 2026-03-15
**Build:** Post-fauna, post-ecology particles, quest system, day cycle, forest ring
**Session:** Playwright headless Chromium, 16 screenshots across camera angles, x-ray, lighting presets
**Mode:** Mock data (no WASM) — viewing oak growth stage demo grid
**Screenshots:** `artifacts/screenshots/deep-playtest/`

---

## 1. What the game sold me

A cozy 3D voxel garden where I compose ecosystems, watch things grow, and discover ecological relationships above and below ground. Warm golden light, a miniature-world diorama feel, visible fauna and interactions.

## 2. What I actually experienced

**The visual foundation genuinely delivers the cozy diorama promise.**

The initial view (01-initial-default) shows a warm amber/ochre landscape with five oak trees at different growth stages — from a tiny seedling on the left to a massive old-growth specimen on the right. The foliage is rendered as soft, slightly translucent billboard sprites with a gentle green glow. Branches are visible as dark red-brown structures extending from trunks. The terrain is a clean isometric slab of earthy brown soil.

The HUD is clean and non-intrusive: quest panel on the left (Chapter 0: "Getting Your Bearings"), species picker at bottom center organized by type (Oak/Birch/Willow/Pine, Fern/Berry Bush/Holly, Wildflower/Daisy, Moss/Grass/Clover), and tool buttons at bottom (S/P/W/D/R).

**The forest ring surrounding the garden bed is a great touch** (16-forest-ring-surroundings). Low-poly decorative trees of varying heights create the illusion that the garden plot exists in a larger world. It frames the garden without competing for attention.

## 3. Best moments

**X-ray underground view (06, 07, 08)** — This is the game's killer visual feature. Pressing Q makes soil semi-transparent and reveals the root networks glowing in warm amber/golden. The roots are massive spreading structures — taproots descending straight down, lateral roots branching outward in 8 directions, secondary forks, and fine tip tendrils. The old-growth oak's root network (visible in 07-xray-side-angle) is as visually impressive as its crown above ground. The amber glow against the transparent brown soil creates a dramatic, almost bioluminescent look. This is unlike anything I've seen in another garden game.

**Close-up of tree canopy (04-closeup-tree)** — Zooming into the largest tree shows the foliage billboard sprites up close. Each leaf cluster is a soft green point with slight transparency, creating a cumulus-cloud-like canopy. The trunk structure is visible through gaps in the foliage, with branches radiating outward. The tilt-shift DOF blurs the background trees, creating genuine depth-of-field miniature photography feel.

**X-ray close-up on roots (08-xray-closeup-roots)** — This is breathtaking. The root system dominates the frame — thick laterals spreading outward and downward, with the crown visible above as a green halo through the semi-transparent soil. You can see where roots fork into secondaries and fine tendrils. The amber glow intensity makes every root voxel readable.

**Top-down planning view (10-topdown-planning)** — This shows a cross-section of the entire garden from above, with the underground visible below the soil line. A single tree is visible: crown above, trunk in the middle, root network descending into darkness below. The contrast between the lit surface and dark underground creates a dramatic "split world" effect. The forest ring trees are visible in silhouette around the edges.

**Low-angle side view (09-low-angle-side)** — From this angle, the trees look like real miniatures. The foliage canopies have a natural, irregular shape — not perfect spheres but lumpy, multi-lobe crowns. The branches poking through add structural believability.

## 4. Surprises — things the garden did that I didn't plan

This is mock data mode, so no simulation is running. The demo grid is static. No autonomous behavior visible. **This is the biggest gap in the current experience** — the demo grid is beautiful but lifeless. No water, no growth, no fauna, no particles. It's a diorama, not a garden.

## 5. Confusing moments

**The demo grid has no water.** The quest panel says "Find the water spring" but there's no water visible anywhere. In mock mode, there's no way to progress through quests that require grid state (wet soil, roots, trunks from actual growth).

**Seeds placed in mock mode don't grow.** Auto-tick is gated by `isInitialized()`, so the simulation never advances. A player who can't load WASM gets zero gameplay. They can admire the oak showcase but not play.

**Lighting differences are subtle (11-14).** Dawn, golden hour, blue hour, and noon all look very similar — warm amber in every case. The day cycle is working (you can see slight intensity changes), but the color temperature range is narrow. Dawn and blue hour should feel meaningfully different from golden hour. Currently they all read as "warm" with minor brightness variation.

## 6. Boring or frustrating moments

**Static world.** Without WASM, nothing moves, grows, or responds. The foliage doesn't sway (no animation frames captured). No fauna sprites visible. No ecology particles. No water ripples. The beautiful systems built in code (fauna, ecology particles, wind sway, water shader) are invisible because this is a static snapshot.

**Species picker is static.** Pressing 2 to select seed tool shows the species panel (15-hud-species-picker), but the panel categories are plain text buttons. No visual preview of what each species looks like. A player choosing between "Oak" and "Birch" has no idea what the difference will be visually.

## 7. Signs of life — fauna, movement, autonomous garden behavior

**None visible in this session.** Mock mode produces no fauna, no ecology particles, no wind sway, no growth particles. These systems are implemented in code but require either WASM simulation or frame animation to be visible.

The static screenshots miss the foliage wind sway shader, water ripple animation, and fauna wing flutter — all of which are time-based animations that would be visible in a live browser session.

## 8. What I learned about the ecosystem

From the visual demo grid alone:
- **Root networks mirror crown structure.** The old-growth oak's roots spread as wide as its crown. The sapling has a small root ball. This above/below-ground symmetry is visible and educational.
- **Trees grow through distinct stages.** Seedling (1 voxel trunk + 1 leaf), sapling (short trunk + small crown), young (medium trunk + branches), mature (thick tapering trunk + large crown + buttress roots), old-growth (massive asymmetric crown + thick limbs + enormous root cave network).
- **Branches are structural.** They extend from trunks at specific heights and support the leaf canopy. Not just decorative — they define the tree's shape.

## 9. Bugs

### 1) Day cycle lighting variation is too subtle
- **Severity:** minor (visual)
- **Screenshots:** 11-dawn vs 12-golden-hour vs 13-blue-hour vs 14-noon
- **Expected:** Dramatically different color temperatures — pink dawn, amber golden hour, cool blue twilight, bright white noon
- **Actual:** All four look nearly identical — warm amber tones with slight brightness changes
- **Notes:** The day cycle code has distinct color presets, but SwiftShader rendering may not be applying them fully. Or the color grading post-process (warm shift +0.06) is overwhelming the cycle's color variation.

### 2) Mock mode is completely non-interactive
- **Severity:** major
- **Steps to reproduce:** Load game without WASM built
- **Expected:** Some form of gameplay or clear indication that WASM is needed
- **Actual:** Beautiful static diorama with no growth, no fauna, no interactivity
- **Notes:** The demo grid showcases tree art but doesn't deliver the game's core fantasy

### 3) No water in demo grid
- **Severity:** minor
- **Steps to reproduce:** Load mock mode, look for water
- **Expected:** At least one water source to demonstrate the water rendering
- **Actual:** No water voxels in the oak demo grid
- **Notes:** The water shader (depth-based color, Perlin noise ripples, specular highlights) is invisible in mock mode

## 10. Feature or clarity requests

1. **Add water to the mock grid** — A stream or pond would showcase the water shader and give context for the "Water is Life" quest chapter.
2. **Add a simple JS tick for mock mode** — Even basic seed→trunk transitions would make mock mode playable rather than purely visual.
3. **Widen day cycle color range** — Dawn should be distinctly pink/peach, blue hour should be distinctly cool blue-purple, not just brightness variants of amber.
4. **Species visual previews in picker** — Tiny icons or silhouettes showing what each species looks like grown.
5. **Add fauna to mock grid** — Place some static or animated fauna sprites to showcase the ecology particle system even without live sim.

## Evaluation lens scores (1-5)

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | **4/5** | The oak showcase is immediately beautiful. Warm palette, clean HUD, inviting diorama feel. Forest ring adds sense of place. |
| Clarity of cause & effect | **2/5** | In mock mode, no cause-and-effect is visible. The x-ray underground view *could* show it, but without water/growth/fauna, root-water relationships are invisible. |
| Tactile satisfaction | **2/5** | Click-to-place works but nothing responds. Seeds don't grow. No feedback sounds, no particles. |
| Beauty/readability | **4/5** | The rendering is genuinely beautiful. Foliage sprites, warm palette, tilt-shift DOF, per-vertex AO all work. X-ray root glow is stunning. |
| Ecological fantasy delivery | **2/5** | The demo grid shows *potential* (tree growth stages, root networks) but doesn't deliver the living ecosystem fantasy. No fauna, no water, no interactions. |
| Desire to keep playing | **2/5** | I want to see this with the simulation running. The visuals promise something the static grid can't deliver. |
| Friction/confusion | **3/5** | HUD is clean, controls are readable. But mock mode gives no indication that features are missing. Quests reference systems that don't exist. |
| Trust in the simulation | **3/5** | The tree growth stages in the demo grid are convincing — you can see the progression. But without live sim, I can't verify the ecology works. |
| Surprise/emergence | **1/5** | Zero emergence in a static grid. |
| Sense of life | **1/5** | No fauna, no movement, no animation in captured frames. |
| Discovery arc | **2/5** | X-ray mode is a discovery (revealing roots underground). But without growth/water/fauna, there's nothing else to discover. |
| Garden autonomy | **1/5** | Static grid. Nothing changes. |

## 11. Brutal bottom line: would I come back tomorrow — and why?

**Not in mock mode. Absolutely yes if WASM is working.**

The visual foundation is genuinely excellent. The x-ray underground view with glowing amber root networks is the game's signature moment — it's beautiful, unique, and immediately communicates "there's a hidden world underground." The tree growth stages show real ecological progression. The forest ring, tilt-shift, and warm palette all nail the "cozy miniature world" aesthetic.

But mock mode is a museum exhibit, not a game. I can admire it but not play it. Everything that makes the game's promise exciting — growth, water flow, fauna, ecological interactions, seed dispersal, surprise — is invisible without the running simulation.

**What would pull me back:** Getting WASM working and watching a seed grow into a tree in real-time, seeing root networks spread underground in x-ray mode, watching bees drift between flowers with golden pollen particles. The rendering is ready for that moment. The simulation is ready for that moment. They just need to be connected.

**The gap is bridging mock mode to live sim for the playtest harness.** Once WASM compiles and the Playwright harness can tick the simulation, this will produce genuinely compelling playtest screenshots.

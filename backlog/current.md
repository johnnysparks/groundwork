# Backlog — Current Priorities

**Last updated:** 2026-03-16T18:00:00
**Status:** Alpha complete. 131 sprints. Living ecosystem with visible ecological interactions, ambient life layers, day-night cycle, discovery feedback loop.

---

## Executive Mandate

### Garden Gnome → Sim-Side Entity
See `decisions/2026-03-16T12:00:00_gardener_gnome_zone_planning.md`

**Completed:**
- Phase 1: gnome.rs (struct, task queue, movement, work, WASM exports)
- Phase 1b: Bridge wiring (JS→WASM task sync, gnome state reading)
- Phase 2: Hunger/energy needs (eating/resting states, speed modulation)
- Phase 3: Gnome-fauna interactions (squirrel trust, bird attraction, emotion particles)
- Phase 4: Idle wandering (targeted wander→inspect, task interrupts, JS visual sync)
- Phase 5: Sim→JS position sync (sim is single authority for position, state, task execution)

### Canopy Density (Sprints 83-84, 92-93, 114)
- Crown envelope lowered to 25% (YoungTree) / 15% (Mature) of trunk height
- Leaf spheres at crown_r/2 with interior branch node coverage
- Pending_voxels drain rate 200/tick for fast canopy reveal
- Multi-voxel flower blooms using crown_radius disc
- 8 branch stubs (cardinal + diagonal) for broader canopy fill
- 15 branches per tick (was 10) for faster canopy coverage

### Dense Starter Garden (Sprint 87)
- 21 seed spots (was 7): 5 moss, 5 grass, 5 clover for green carpet
- Trunk visual priority fix (leaf spheres can't overwrite trunk voxels)

### Water & Seed Visibility (Sprints 97-98)
- Water surface shimmer: dancing sun sparkles, shoreline foam, stronger color contrast
- Seed golden sparkle particles: 2-3 seeds twinkle every 0.3s until they sprout
- Seed voxel color brightened for soil contrast

### Mobile Performance (Sprints 99-100, 125)
- DPR clamped to 2, tilt-shift DOF disabled, bloom half-res, shadows 1024
- Chunk remesh budgeted to 4/frame on mobile (eliminates stutter)
- Mobile camera starts at 1.6x zoom (shows detail instead of tiny rectangle)
- Ecology particle scan rate reduced on mobile (1.0s vs 0.3s)

### Weather Visuals & Events (Sprints 101-106, 130)
- Rain particles: 800 soft droplets during Rain weather state
- Drought haze: fog lerps to warm amber during Drought
- Weather event messages in HUD feed (rain/drought/clear transitions)
- Weather transition sounds: rain onset (filtered noise), drought (wind whistle)
- Weather-driven wind: gusty rain (0.7), still drought (0.12), clear (0.35)
- Rain ambient audio: procedural patter fades in/out with weather

### Sound & Ambient Life (Sprints 105-109, 117-119, 128, 130)
- Ambient fauna sounds: bird chirps + bee buzzes every 8-20s
- Growth shimmer sound: soft ascending sweep when vegetation increases
- Full procedural audio landscape (no audio files)
- Night crickets: procedural dual-cricket chorus during golden/blue hour
- Falling leaves: 20 particles drift from canopy with gentle swaying
- Dusk fireflies: 40 golden-green blinking particles during golden/blue hour
- Discovery chime: ascending triad on wild plants and ecological firsts
- Weather transition sounds: rain onset and drought wind cues

### Visible Ecological Interactions (Sprints 115-116, 120, 123, 126-127)
- Squirrel 3D model with scurry/dig animations and warm glow
- Ecological event HUD messages: squirrel caching, pollinator activity, bird seed-dropping
- Nitrogen handshake particles: green shimmer at tree-groundcover junctions
- Canopy shade particles: cool dappled blue-green under tall trees
- All ecology particles: pollination, water absorption, decomposition, nitrogen, canopy shade
- Species-colored roots in x-ray: each species has distinct saturated color
- Wild plant notifications: fauna-dispersed plants get special messages with attribution

### Discovery & Teaching (Sprints 121, 124, 129, 131)
- Plant hover tooltip: species name + material type on mouse-over
- Tips reference visible ecology particles
- Companion species suggestions: first-time planting shows synergy tip
- Idle auto-orbit: camera slowly rotates after 45s for living painting effect

### Quality of Life (Sprints 111-112)
- Auto-save "Saved" indicator (bottom-right, fades after 1.5s)
- Contextual ecology tips (suggests next discovery based on garden state)

---

## P1

- (empty — all P1 items resolved)

## P2

- Mobile drag-to-zone (desktop only currently — needs long-press or mode toggle)
- Squirrel domestication tuning (gnome-squirrel trust visual: follow behavior, seed delivery)
- Multiple gnomes, biome variety, undo/redo, share garden
- SSAO (disabled, needs tuning — shadows now enabled with warm PCF)
- Trunk-to-canopy ratio (trees look like tall sticks with green tops — sim-level fix)
- Water surface shimmer improvements
- Growth stage visual transitions (particle burst when tree matures)

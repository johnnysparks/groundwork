# Backlog — Current Priorities

**Last updated:** 2026-03-18T12:00:00
**Status:** Alpha complete. 296 sprints. **Full quest arc: 8 chapters, 16 quests** from empty meadow to living ecosystem. Condition-based species emergence with neighbor influence + density influence + condition-attribution messages. Complete onboarding. Full cross-system connectivity + fauna behavior particles. 16-layer responsive audio, 25 SFX types. **6 new ecology particle types** (mycorrhizal violet, allelopathy amber-red, nurse log golden, root competition red-orange, soil binding olive-green, overgrowth amber). Density-not-species decision: all 5 items complete. **Irrigation replaces watering can** (executive mandate). **Rich sky dome** (FBM clouds, cirrus wisps, cloud shadows, sun/moon discs, rainbow, layered sunset bands, colored stars, night horizon glow). **Dynamic water reflections** (sky tint, moon path, cloud reflections, water bubbles). **Nutrient-rich soil tinting** (golden-brown fertile earth). **14-element coherent wind** (foliage, trunks, leaves, rain, mist, dust, gnats, fireflies, sky, cloud shadow, water, fauna + fauna wind-responsive animation). **Material polish** (warm AO, bark noise, leaf noise, stone mineral variety). **Time-of-day atmosphere** (fog density, post-rain petrichor glow, post-rain mist boost, grass color waves, soil breathing).

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

### Sound & Ambient Life (Sprints 105-109, 117-119, 128, 130, 135)
- Ambient fauna sounds: bird chirps + bee buzzes every 8-20s
- Growth shimmer sound: soft ascending sweep when vegetation increases
- Full procedural audio landscape (no audio files)
- Night crickets: procedural dual-cricket chorus during golden/blue hour
- Falling leaves: 20 particles drift from canopy with gentle swaying
- Dusk fireflies: 40 golden-green blinking particles during golden/blue hour
- Discovery chime: ascending triad on wild plants and ecological firsts
- Weather transition sounds: rain onset and drought wind cues
- Ambient wind: continuous filtered noise varies with weather strength

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

### Night Atmosphere (Sprints 133-134, 136)
- Stars: procedural star field in sky shader, visible during nighttime
- Moonlight: cool blue ambient fill, garden visible as atmospheric silhouettes
- Shooting stars: one every ~45s during night, bright white streak across sky

### Sound Polish (Sprints 135, 138)
- Ambient wind: continuous filtered noise varies with weather strength
- Dawn chorus: bird calls every 2-6s at dawn (vs 8-20s normally)

### Onboarding & Feedback (Sprints 137, 139-140)
- Welcome messages: warm greeting + first action hint on game start
- Garden recovery messages: encouraging feedback after plant die-off and recolonization
- Pollination particle boost: 10 particles when actively pollinating (vs 5 drifting)

### Audio Polish (Sprint 141)
- Fauna-scaled ambient sound frequency: more fauna → more frequent chirps/buzzes

### Growth Stage Celebrations (Sprint 142)
- Tree stage transitions (sapling→young→mature→old growth) emit 20-particle burst
- HUD messages name the tree and stage; mature+ transitions play discovery chime
- Tree stats read from WASM via packTreeStats bridge

### Stout Tree Proportions (Sprint 143)
- Wider trunks and canopies: Oak 0.15→0.25m trunk, 1.2→1.5m crown
- Birch, Willow, Pine all proportionally adjusted
- Pine height reduced 2.8→2.5m for better silhouette

### Water Surface Polish (Sprints 144, 149)
- Caustic refraction patterns (bright dancing lines, strongest in shallow water)
- Gentle vertex wave displacement (surface undulates)
- Rain ripple rings: concentric circles from drop points during rain

### Squirrel Trust Arc (Sprint 145)
- Trust milestone HUD messages at 50, 100, 150, 180
- Discovery chime at trust 180 (companion threshold)
- Enhanced emotion particles scale with trust level

### Bird Seed-Drop Particles (Sprint 146)
- Golden seed particles fall from birds in Acting state
- Completes Bird Express visual chain

### Day-Night Atmosphere (Sprints 147-148)
- Foliage color tinting: warm gold→neutral→amber→cool blue across day cycle
- Terrain tinting: trunks, branches, soil follow same temperature curve

### Quality of Life (Sprints 111-112)
- Auto-save "Saved" indicator (bottom-right, fades after 1.5s)
- Contextual ecology tips (suggests next discovery based on garden state)

### Atmospheric Particles (Sprints 150-151)
- Morning dew sparkles at dawn (0.15-0.35)
- Midday dust motes (0.3-0.65)

### Weather Transitions (Sprints 152-153)
- Smooth rain intensity ramp (3s up, 2s down) with scaled drop count
- Drought foliage yellowing (tint lerps toward dry yellow-brown)

### Dynamic Atmosphere (Sprints 154-156)
- Wind gusts: periodic pulses spike foliage sway + scatter leaves + whoosh sound
- Cloud shadow pulses: 3 overlapping sine waves modulate sun intensity during day
- Leaf rustle ambient: bandpass noise scales with foliage count + wind strength

### Alive Garden Polish (Sprints 157-220)
- Flower petal scatter on wind gusts (pink-purple + warm yellow per species)
- Seed sprout pop particles (bright green burst on seed→plant transition)
- Seed dispersal trail (golden particles descending at new seed landing sites)
- Species-colored falling leaves (oak/birch/willow/pine tinted autumn colors)
- Dawn mist wisps (30 diffuse particles hovering at ground level)
- Golden hour dust motes (extended from midday through 0.80)
- Rain splash particles (~8% of drops emit tiny splash on ground impact)
- Squirrel digging particles when caching (reuses worm soil disturbance)
- Gnome emotion sounds (grunt/hmm/sigh/munch on state transitions)
- Frog chorus at dusk near water (two 120Hz/145Hz oscillators)
- Beetle clicking sound (scales with beetle count, daytime only)
- Owl hoot every 30-70s during deep night
- Water babble scales with water volume (0.04→0.12 gain)
- Idle camera zoom breathe (±3% at 0.15Hz during auto-orbit)
- Shooting star shimmer sound (synced to shader timing)
- Plant die-off wilting particles (brown/amber drift down)
- Dawn chorus: 3 bird song variants (chirp, warble, robin)
- Post-rain leaf drip (30-60s of water drops from foliage)
- Firefly blink synchronization (gradual phase lock after 15s)
- Wind chime on gusts in dense gardens (pentatonic sine, 40% chance)
- Dawn soil steam wisps (warm amber particles at sunrise)
- Cricket tempo varies with night warmth (Dolbear's Law: 2.5→1.5Hz)
- Water surface day-cycle tinting (golden hour, moonlight blue)
- Squirrel chitter sound (rapid sine clicks, fauna sound complete)
- Bee waggle dance (figure-8 golden particles when pollinating)
- Mist density scales with water volume (wetter = thicker dawn fog)
- Dew drop tinkle (delicate high sine during dew time)
- Garden age milestones (1k/5k/10k/25k celebration messages)
- Camera pan rustle (leaf fragments scatter from canopy on fast panning)
- Garden vitality drone (low harmonic hum at night in thriving gardens)
- Golden hour bloom boost (bloom strength peaks during 0.65-0.80)
- Fauna arrival sparkle burst (12 gold particles spiral at new fauna)
- Tree growth creak (woody resonant sound on stage transitions)
- Fauna flight bob (bees 5Hz, butterflies 3Hz, birds 2Hz with banking roll)
- Star twinkle variation (per-star unique phase/speed, ±40% brightness)
- Drought dust devils (6-particle upward spirals, tan/brown, ~0.3/sec)
- Fog color follows day cycle (desaturated tint, warm amber→cool blue)
- "The garden is alive" milestone (plants>1000, fauna≥5, species≥3 → chime + sparkle)
- Ambient gnat swarms (5 clusters of 8 orbiting over dense vegetation, daylight only)
- Water ripple from flying fauna (8-particle expanding ring when near water)
- Root growth crackle sound (3-4 rapid low-pass noise snaps on root expansion)
- Butterfly flower landing (Acting state settles lower, slow wing pulse, level wings)
- Heat shimmer post-processing (UV distortion during drought/midday)
- Bird perch idle animation (tucked wings, head tilt, lowered into canopy)
- Worm soil disturbance particles (earthy puffs at ground level during activity)
- Rain puddle shimmer (stationary blue-white reflective particles during rain)
- Beetle iridescent trail (green-blue shimmer behind active beetles)
- Wind streak particles (fast horizontal streaks during gusts, slowly drifting direction)
- Firefly golden glow reflections on water surface
- Squirrel footprint dust particles when scurrying
- Distant bird calls layered into dawn chorus (1-2 per call)
- Star reflections on water at night (wobble distortion)
- Sunrise/sunset bloom flash (brief vivid glow at horizon transitions)
- Garden vitality scales master ambient volume (0.6→1.0)
- Decomposition fungi spore particles near dead wood
- Night moth particles flutter near active fireflies
- Raindrop plink sound (10% of visual splashes)
- Ecosystem health warm glow (warmth + saturation scale with vitality)
- Garden whisper harmonic (barely audible chord in dense gardens)
- Pollen visible in sunbeams (density + golden tint scales with pollinator count)
- Bee waggle dance (golden figure-8 particles during active pollination)
- Butterfly pollen trail (soft yellow motes drift from wings)
- Bird nesting particles (brown twig fragments tumble from perched birds)
- Moonbeam shafts at night (cool blue particles through canopy)
- Rain amplifies frog chorus (louder, active during daytime rain)
- Drought cicada drone (harsh high-frequency buzz during drought daytime)
- Water surface fog wisps (cool mist above water at dawn)

---

### Irrigation Replaces Watering Can (Executive Mandate)
See `decisions/2026-03-17T12:00:00_irrigation_replaces_watering_can.md`

**Completed (documentation):**
- Decision document written
- All docs updated (CLAUDE.md, agents, decisions, balance, quests)
- Water tool removed from HUD, costs, audio triggers, gnome tool visuals
- Bridge/WASM comments updated (water tool code kept for debug only)

**Completed (implementation, Sprints 157-158):**
- Water tool filtered from HUD palette (BRIDGE_TOOLS → filter)
- Quest 'placeWater' completes on shovel use (dig = irrigate)
- Water flow sparkle particles at expanding water frontier
- Tutorial text already guides to dig channels from spring

**Pending:**
- P2: Berm/dam mechanics with soil tool
- P2: Flow rate visualization overlay
- P2: Irrigation efficiency HUD feedback

## P1

- (empty — all P1 items resolved)

## P2

- Mobile drag-to-zone (desktop only currently — needs long-press or mode toggle)
- Multiple gnomes, biome variety, undo/redo, share garden
- Irrigation overlay: root mesh dominance in x-ray makes moisture heatmap subtle; consider render order

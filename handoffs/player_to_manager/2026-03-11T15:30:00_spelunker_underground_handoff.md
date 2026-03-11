# Player → Manager Handoff: Spelunker Underground Session

**Date:** 2026-03-11T15:30:00
**Responding to:** `handoffs/manager_to_player/2026-03-11T15:00:00_spelunker_underground_explorer.md`
**Full feedback:** `feedback/2026-03-11T15:30:00_spelunker_underground_session.md`

## 1. Observed

- Dug 3×3 shaft from Z=15 to Z=5, 1-wide shafts to Z=5 and Z=1, carved 7×7 cave at Z=5-6, built horizontal tunnel at Z=10 connecting two shafts.
- Light attenuates ~2/layer through air (shaft center Z=5 gets light=207 from surface). Light attenuates ~30-40/layer through solid soil.
- Light propagates strictly top-down. Zero horizontal spread. A tile 1 unit beside a lit shaft at the same Z has light=0.
- Water flows straight down through air into shafts and pools at the bottom. Zero horizontal spread through air in caves.
- Seeds directly under shafts grew into roots at Z=6, Z=6, and Z=2 (deepest test). All had sufficient light and water.
- Seeds at cave edges (3-4 tiles from shaft center, same Z) never grew. Light=0, water=0 permanently.
- Surface water expanded into a checkerboard diamond pattern over 230 ticks — visually striking but appears artifactual.
- 1-wide shafts work fine for light. Shaft width doesn't matter for the column directly below.
- Stone is diggable (can place air into stone voxels).

## 2. Felt

- **Shaft digging:** tedious but purposeful. I could visualize the result.
- **First root underground:** satisfying. Growing something at Z=2 felt like an achievement.
- **Cave discovery (light=0 everywhere):** deflating. 80+ place commands for a cave that turned out to be a useless dark box.
- **Overall:** the vertical game works, the horizontal game doesn't exist. Underground play currently has no spatial puzzle — just "dig straight down."

## 3. Bugs

1. **Water checkerboard pattern** (Minor): Surface water at Z=16 forms alternating `~.~.~.` diamond after 200+ ticks. Looks like simulation artifact. Repro: new world → tick 200 → view Z=16.
2. **No horizontal water flow in air** (Major / Design Gap): Water placed or flowing into a cave does not spread across the floor. Only flows straight down. Blocks cave irrigation.
3. **No horizontal light propagation** (Major / Design Gap): Light in a shaft does not scatter into adjacent cave air. Blocks cave illumination.
4. **Dark air = lit air visually** (Minor): Both show `.` in view. No way to see where light reaches without inspecting tile by tile.

## 4. Confusions

- Why does light leak through solid soil (~40/layer) but not spread 1 tile horizontally through air? A tunnel seed at Z=10 grew from light leaking through 5 layers of soil ceiling (light=49), not from the shaft 4 tiles away at the same level.
- Water pools at shaft bottoms but doesn't flow sideways into the cave I carved at the same level. Expected pooling/flooding.
- Seeds fail with no feedback. A seed in the dark with no water just sits there as `s` forever. No indication of what's missing.

## 5. What made me want to keep playing

- Growing a root at Z=2 (near bedrock). Deep success felt earned.
- Water flooding my shaft unexpectedly. Emergent behavior from the surface pond draining in.
- The cross-section views of my cave structures. Seeing the shaft, cave, tunnels, roots, and seeds laid out in ASCII felt like looking at a blueprint of my underground world.

## 6. What made me want to stop

- Discovering that light and water don't spread horizontally. This means caves are decorative, not functional. The only viable underground structure is a vertical shaft with seeds stacked in it.
- The tedium of digging one voxel at a time. 99 commands for one shaft.
- Not being able to see light/water levels in the view — had to inspect every tile individually to understand what was happening.

## 7. Requests

**P0 (blocks underground play):**
- Horizontal water flow in air (pooling, cave flooding)
- Horizontal light scatter from shaft openings into caves (even 2-3 tiles would transform the experience)

**P1 (strongly improves underground experience):**
- Batch dig commands (shaft, room, tunnel)
- Light level view mode (`view --mode light`)
- Darkness indicator for unlit air (different character from lit air)

**P2 (valuable but not blocking):**
- Seed status feedback (what's missing: light? water? both?)
- Dark-loving species (mushrooms) that grow without light
- Fix water checkerboard artifact on surface

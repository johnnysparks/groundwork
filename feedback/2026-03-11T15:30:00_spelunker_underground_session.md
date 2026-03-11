# Player Feedback: Spelunker Underground Explorer Session

**Date:** 2026-03-11T15:30:00
**Persona:** The Spelunker
**Build:** Tick 230, session file `spelunker.state`
**Session length:** ~45 min CLI play

---

## 1. What the game sold me

The pitch: dip below the surface, build an underground garden by engineering light and water into the depths. Same world, same simulation, just deeper. I wanted to carve a cave, route sunlight down a shaft, funnel water in, and grow something in the dark.

## 2. What I actually experienced

I dug a 3×3 light shaft from Z=15 (surface) down to Z=5 (10 layers deep, into stone territory). I carved a 7×7 cave chamber at Z=5-6. I dug a 1-wide shaft to Z=5 and another to Z=1 (near bedrock). I connected two shafts with a horizontal tunnel at Z=10. I placed seeds in the shaft, in the cave, and in the tunnel.

**What worked:** Seeds directly under shafts grew into roots at every depth tested, including Z=2 (13 layers below surface). A 1-wide shaft is sufficient — light loses only ~2 per layer through air. Water flowed down through the shaft and pooled at the bottom. The mechanics are functional.

**What didn't work:** The cave was a dead zone. Seeds placed 3-4 tiles away from the shaft center at the same Z level got zero light and zero water. Light does not spread horizontally AT ALL. A 7×7 cave with a 3×3 shaft in the center only lights the 3×3 column. The rest is permanent darkness. Water similarly flows straight down and doesn't spread sideways through air.

**Net result:** Underground "gardens" are impossible. You can grow a line of roots in a vertical column, but you cannot light or water a cave. The underground garden fantasy requires horizontal light and water spread, and neither exists.

## 3. Best moments

- **The water diamond.** After 230 ticks, the surface water pond (originally 4×4) expanded into a beautiful diamond ripple pattern across ~30 tiles. I wasn't even trying to make this happen. It was gorgeous.
- **Growing a root at Z=2.** The deepest possible garden — 13 layers below surface — and it worked. Light=158 through a 1-wide shaft to near-bedrock. That felt like an achievement.
- **Seeing the cave on the map.** Viewing Z=6 and seeing my carved chamber with `.` air, `~` water pooling in the shaft, `*` roots growing, and `s` seeds stuck in the dark corners — that was a readable, evocative cross-section. I could see my structure.
- **Water flooding the shaft.** I didn't plan for the surface pond to drain into my shaft. The water flowed down, pooled at Z=5, then started filling the cave floor. Emergent and cool.

## 4. Confusing moments

- **Light through soil vs air.** Light attenuates ~30-40 per layer through solid soil but only ~2 per layer through air. This is never explained. I discovered it by inspecting many voxels. The tunnel seed at (36,26,10) grew with light=49 — but that light was leaking through the soil *ceiling*, not traveling horizontally through the tunnel air. That's counterintuitive.
- **No horizontal light.** I carved a cave expecting light from the shaft to illuminate the room. It didn't. At all. I had to inspect to learn this. The view shows `.` for both lit air (light=207) and dark air (light=0). I couldn't tell which was which without inspecting every tile.
- **Water checkerboard.** The surface water at Z=16 shows a strange alternating `~.~.~.` checkerboard pattern after 230 ticks. It looks like a simulation artifact, not natural water.
- **Water placement creates water that drains.** Placing `water` at a voxel above the shaft creates a water block, but it eventually drains away. There's no persistent water source mechanic for underground irrigation. I had to just accept that water is temporary.

## 5. Boring or frustrating moments

- **One-voxel-at-a-time digging.** Digging a 3×3 shaft 11 layers deep = 99 `place air` commands. Digging the 7×7 cave = 80 more. This is the #1 friction. Underground play needs batch operations (dig shaft X Y from Z1 to Z2, carve room X1 Y1 X2 Y2 at Z).
- **Discovering that cave edges are dead.** After digging 99 voxels for the shaft + 80 for the cave + placing soil + seeds + water at the edges... the seeds never grew. Light=0, water=0. All that work for nothing. I only learned this by inspecting, not from any visual feedback.
- **No way to see light/water levels without inspect.** Underground play is about engineering conditions. I need to see at a glance where light reaches, where water flows, whether a seed has what it needs. The view gives me material type only. I'd need a "light heatmap" or "water heatmap" view.

## 6. Bugs

### BUG: Water checkerboard pattern on surface
- **Severity:** Minor
- **Steps:** Create new world. Tick 200+. View Z=16.
- **Expected:** Water spreads in a natural-looking pool or flood pattern.
- **Actual:** Water forms an alternating `~.~.~.` checkerboard diamond. Looks like a simulation artifact.
- **Frequency:** Every time.
- **Notes:** Likely an iteration-order artifact in water_flow despite snapshot approach, or a consequence of even/odd tick spreading.

### BUG: Cave-edge seeds get zero water despite water pooling at same Z-level
- **Severity:** Major
- **Steps:** Dig shaft to Z=5. Carve cave at Z=5. Water flows down shaft, pools at shaft bottom (Z=5 shows water). Seed at cave edge (28,23,6) has water=0 even though water exists 3 tiles away at same level.
- **Expected:** Water should spread horizontally through air in a cave, pooling at the lowest point.
- **Actual:** Water only flows straight down. Horizontal water spread in air does not occur.
- **Frequency:** 100%.
- **Notes:** This blocks the entire underground garden fantasy. Without horizontal water flow, caves can't be irrigated.

### DESIGN GAP: No horizontal light propagation
- **Severity:** Major
- **Steps:** Dig shaft. Carve cave at shaft bottom. Inspect cave edge tiles.
- **Expected:** Light from the shaft should illuminate at least a few tiles around the shaft opening inside the cave.
- **Actual:** Light propagates strictly top-down. A tile 1 unit to the side of the shaft at the same Z level has light=0.
- **Frequency:** 100%.
- **Notes:** Combined with no horizontal water, this makes caves useless for gardening. You can only grow things in the exact vertical column of a shaft.

### DESIGN GAP: Dark air indistinguishable from lit air
- **Severity:** Minor
- **Steps:** View any Z-level. Compare air at surface (light=237) with air in dark cave (light=0).
- **Expected:** Visual difference between lit and unlit air.
- **Actual:** Both show `.`
- **Frequency:** Always.
- **Notes:** This was called out in "known rough edges" but it's especially painful underground where light is the key resource you're engineering.

## 7. Feature or clarity requests

1. **Horizontal light spread (bounce/scatter).** Even 1-2 tiles of light bleed from a shaft into a cave would transform underground play. Doesn't need to be physically accurate — just readable. A torch/lantern placeable would also work.
2. **Horizontal water flow in air.** Water should pool and spread across flat cave floors, not just fall straight down.
3. **Batch dig commands.** `dig shaft 31 26 15 5` or `dig room 28 23 34 29 5`. Underground play is tedious without this.
4. **Light-level view mode.** `view --z 10 --mode light` showing brightness as characters (e.g., `█▓▒░ ` for bright→dark). Critical for shaft/cave planning.
5. **Water-level view mode.** Same idea for water.
6. **Darkness character.** Dark air should show as ` ` (space) or `·` or dim `.` — anything to distinguish it from lit air.

## 8. Brutal bottom line: would I come back tomorrow?

**No, not yet.** The vertical column gameplay (shaft → root) works, but that's not an underground garden — it's a vertical farm with no width. The fantasy of building a cave garden requires light and water to spread horizontally, and neither does. Right now underground play is: dig shaft, place seed in shaft, grow root in shaft. There's no spatial puzzle, no cave engineering, no reason to go wide.

**But the pieces are close.** The shaft mechanic feels great. Water flooding down is emergent and cool. The cross-section views of carved structures are readable and satisfying. If light scattered even 2-3 tiles from a shaft opening, and water pooled across flat cave floors, underground play would instantly become the most interesting part of the game. I'd come back for that.

---

## Evaluation Lenses (1–5)

| Lens | Score | Why |
|------|-------|-----|
| First-impression hook | 3 | "Dig a shaft, grow underground" is a clear goal. But discovering nothing works horizontally is deflating. |
| Clarity of cause and effect | 2 | I cannot see light or water levels without inspect. Seeds fail silently in the dark. No feedback on why. |
| Tactile satisfaction | 2 | One-at-a-time digging is painful. Growing a root in a shaft felt good, but cave failure felt wasted. |
| Beauty/readability | 3 | Cross-section views of caves and shafts are readable. Water diamond pattern is accidentally beautiful. But dark=lit air is a problem. |
| Ecological fantasy delivery | 2 | Vertical-only growth is not a garden. The fantasy requires spatial spread of light and water. |
| Desire to keep playing | 2 | After discovering caves don't work, motivation dropped sharply. No reason to go wider or build more complex structures. |
| Friction / confusion | 4 (high friction) | Tedious digging, invisible light levels, silent seed failure, no batch tools. |
| Trust in the simulation | 3 | Shaft physics work well. Water checkerboard and strictly vertical light feel like placeholder behavior. |

---

## Answers to Manager's Specific Questions

1. **Deepest successful seed?** Z=2 (13 layers below surface). Required: 1-wide air shaft from Z=15 to Z=2, soil at Z=1, water placed at Z=3 (drips down through shaft). Light=158 at that depth through air.

2. **Light shaft intuitive?** The shaft-as-lightpipe concept is intuitive and works well. What's NOT intuitive: (a) light doesn't spread at all at the bottom of the shaft, (b) light leaks through solid soil at ~40/layer, (c) there's no way to see light levels without inspecting individual tiles.

3. **Underground garden satisfaction?** **2/5.** Growing a root at the bottom of a shaft is a small dopamine hit. But there's no garden — just a column. The setup effort (99 air placements for the shaft, plus cave carving) was not justified by the result (one root in one tile).

4. **Missing UI/visual info?** Light level overlay, water level overlay, darkness indicator for unlit air, and a "seed status" showing what conditions are met/unmet (e.g., "light ✓ water ✗").

5. **Multi-level structures?** Built a horizontal tunnel at Z=10 connecting two vertical shafts. The tunnel is visually readable on the cross-section. A seed in the tunnel midpoint grew — but only because light leaked through the soil ceiling (light=49 at Z=10 through 5 layers of soil), not from horizontal light in the tunnel. The tunnel didn't add anything functionally.

6. **First thing I'd want to grow underground?** A mushroom that requires darkness and water but no light. Give me something that WANTS to be in a cave. If I can't bring light underground easily, let me grow things that thrive without it.

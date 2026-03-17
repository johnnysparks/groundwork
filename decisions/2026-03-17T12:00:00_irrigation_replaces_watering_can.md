# Decision: Irrigation Replaces Watering Can

**Date:** 2026-03-17T12:00:00
**Status:** Executive Mandate
**Priority:** P0 — core gameplay feel change

## Context

The watering can tool is "bleh." After you sow your first small seeds, the natural next step should feel like a discovery — you look at the pond, and realize you can *dig a path* for water to flow toward your plants. Instead of clicking a watering can on soil (which is passive, disconnected, and un-strategic), you **shape the terrain** to guide water. Irrigation becomes the "road" mechanic of Groundwork — the infrastructure you build to connect water sources to gardens.

This is directly inspired by **Timberborn**, where water management through terrain shaping is a core strategic loop. The player optimizes hydration by sculpting channels, berms, and catch basins. Water flow already exists in the sim (`water_flow` system in `systems.rs`) — it's gravity-driven and spreads laterally. The watering can *bypasses* this beautiful system. Removing it forces the player to *engage* with it.

## The Shift

**From:** `water` tool = **watering can** — click to pour water onto a voxel. Gnome carries a watering can. Water appears from nowhere.
**To:** No water placement tool. Water comes from **natural sources** (springs, rain, ponds). The player uses the **shovel** (dig/air tool) to carve irrigation channels that guide water flow to their plants. The terrain *is* the irrigation system.

## Why This Is Better

1. **Strategic depth.** "Where do I dig my channel?" is a real decision with tradeoffs — you might flood one area, you have to think about elevation, you learn how water_flow works by observing it. The watering can had zero strategy.

2. **Discovery moment.** The first time you dig a trench from the pond to your seeds and watch water flow in — that's a genuine "aha." It teaches the water system through play, not UI.

3. **Timberborn-style infrastructure.** Irrigation channels are the "roads" of Groundwork. You plan them, the gnome digs them, water flows through them. They're visible, strategic, and satisfying to optimize.

4. **Ecological integration.** Channels interact with the rest of the ecosystem: willows grow along waterways, moss colonizes damp banks, worms thrive in moist soil near channels. The irrigation system *becomes part of the ecology*.

5. **Removes a boring tool.** The watering can was a content-less click. Every other tool creates something interesting (seeds grow, soil shapes terrain, shovel reveals underground). The watering can just set a number higher.

6. **Water budget shifts meaning.** Instead of spending "water points" to pour water, the water budget reflects how much water your spring/rain produces. Managing it means managing *flow paths*, not clicking a tool.

## What Changes

### Tools
- **Remove** the `water` / watering can tool from the tool palette entirely
- **Shovel/dig** becomes the primary irrigation tool — dig channels, trenches, catch basins
- **Soil** tool gains importance — build berms, dams, raised beds to direct water
- **Stone** tool can create permanent water barriers/aqueducts

### Water Sources (unchanged, but now more prominent)
- **Spring** — center of the map, persistent water source (already exists: `water_spring` system)
- **Rain** — periodic weather event that fills exposed areas (already exists: `weather_system`)
- **Ponds** — natural water accumulations in low terrain

### Player Flow (new)
```
1. Start near the spring — seeds planted close get water naturally
2. Want to expand? Look at the terrain. Where is the water?
3. Dig a channel from the pond toward your new planting zone
4. Watch gnome waddle over with shovel, dig the trench
5. Water flows in! Seeds near the channel start growing
6. Shape berms with soil tool to prevent flooding
7. Discover: willows planted along channels stabilize banks
8. Discover: wider channels = more flow, narrow = controlled drip
9. Optimize: design efficient irrigation networks
10. Late game: self-sustaining water loops with rain catch + channels
```

### Gnome Implications
- Gnome no longer carries a watering can
- Gnome's idle "spot-watering" behavior → replaced with idle "channel maintenance" (clears debris from channels)
- Gnome still interacts with water (drinks from stream, rests near water)

### Quest/Tutorial Updates
- "Use the watering can" quest → "Dig an irrigation channel" quest
- Teaches: dig from water source toward plants, watch flow
- This is a more engaging tutorial moment than "click watering can on soil"

### Water Budget (HUD)
- Water budget no longer spent on a water tool
- Budget now represents spring output + rain collection capacity
- Shovel cost remains (digging channels costs energy, not water)
- Consider: water budget display → irrigation flow rate display

## What Stays

- `water_flow` system — already does gravity-driven flow, lateral spread. This is the engine of the whole mechanic.
- `water_spring` system — persistent source. Still the life of the garden.
- `weather_system` — rain fills exposed areas. Now more meaningful because channels direct rainwater.
- Water material type (Material::Water = 3) — water still exists as a voxel material.
- All ecological water interactions (root absorption, soil moisture, etc.)

## What This Enables (Future)

- **Aqueducts** — stone channels that carry water uphill (P2)
- **Waterfalls** — aesthetic water features that also irrigate (P2)
- **Flood/drought as strategic events** — overflow channels, emergency berms (P1 via weather system)
- **Wetland biome** — where irrigation IS the game (biome variety pillar)

## Relationship to Existing Decisions

### Extends: SimCity-style Zone Gameplay (2026-03-15T20:30:00)
That decision noted "Water is irrigation: placing water creates a persistent source that waters surrounding soil." This decision takes it further: water isn't *placed* at all — it flows from natural sources through player-dug channels. The zone concept still applies: you zone a dig path, gnome digs it.

### Extends: Garden Gnome (2026-03-16T12:00:00)
The gnome's tool list drops the watering can. Digging irrigation channels becomes a primary gnome activity. The visual of a gnome digging a trench while water starts flowing in behind them is a delight moment.

### Strengthens: Gameplay Depth Principles
- Principle #2 (surprise rewards observation): "I dug a channel and now willows grow along it" — emergent, traceable
- Principle #3 (discovery shifts mental model): Hour 1: "dig near water." Hour 5: "design efficient channel networks." Hour 10: "integrated water management sustains the whole garden"
- Principle #4 (interactions, not just growth): channels interact with terrain, plants, fauna, weather
- Principle #5 (garden exceeds the plan): water flows where gravity takes it — sometimes to places you didn't intend, creating happy accidents

## Implementation Notes

### Phase 1: Remove watering can (code changes)
- Remove `Water` from ToolCode enum in bridge.ts / wasm_bridge.rs
- Remove water tool from HUD tool palette
- Update quest text ("dig an irrigation channel" replaces "use the watering can")
- Remove water tool cost from main.ts cost table
- Remove water sound effect trigger for tool placement
- Update gnome tool visuals (remove watering can model)

### Phase 2: Enhance dig-as-irrigation
- Improve water_flow visualization so channels are satisfying to watch
- Add "channel" ghost overlay style for shovel when digging near water
- Tutorial moment: first quest sends player to dig from spring to seed area

### Phase 3: Water management depth
- Berm/dam mechanics with soil tool
- Flow rate visualization overlay
- Irrigation efficiency feedback in HUD

## Success Criteria

1. Player naturally discovers "dig toward water" within first 5 minutes
2. Building irrigation channels feels strategic and satisfying (Timberborn-level)
3. No player misses the watering can — terrain shaping is strictly more interesting
4. Water flow visualization makes channels legible and beautiful
5. Ecological interactions along waterways create discovery moments

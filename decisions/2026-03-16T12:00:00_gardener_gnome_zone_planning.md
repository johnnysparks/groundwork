# Decision: Garden Gnome Character & Zone-Planning System

**Date:** 2026-03-16T12:00:00
**Revised:** 2026-03-16T13:00:00
**Status:** Executive Mandate
**Priority:** P1 — core feel improvement

## Context

After 32 sprints, player actions are **instant**: click a voxel, it changes immediately. There is no sense of a living agent *doing the work*. The game feels like direct voxel manipulation rather than tending a garden. Compare with **Timberborn**, where you zone areas and watch beaver workers walk over and execute — the natural pacing and character delight are a major source of that game's stickiness.

This is an **executive mandate**: the gardener gnome and zone-planning system are the primary mechanism for all player-initiated garden changes. This isn't a nice-to-have — it's a fundamental shift in how the game feels.

## The Shift

**From:** Click/drag → instant voxel mutation (Minecraft-style direct manipulation)
**To:** Zone a plan → watch your garden gnome waddle over and do the work (Timberborn-style mediated execution)

## Why This Matters

1. **Natural pacing** — Work takes visible time. The player watches it happen instead of clicking through it.
2. **Character delight** — A charming gnome carrying tools and working the garden is inherently cozy. It gives the player someone to watch, root for, and identify with.
3. **Planning as gameplay** — Painting zones and seeing ghost overlays IS a gameplay loop: plan → observe → adjust. This is SimCity/Timberborn's core satisfaction.
4. **Idle-time reward** — Watching the gnome work is inherently engaging idle-time content. Aligns with principle #7: "a player who stops clicking and just watches should see a living painting."
5. **Observation incentive** — Slower execution means the player has time to *notice* what's happening. The pacing creates space for the discovery arc.
6. **Fauna interactions** — A sim-side gnome can interact with fauna: befriend squirrels, wave at birds, dodge bees. These micro-moments are delight generators the player didn't plan but can observe.
7. **The gnome lives in the garden** — Not a cursor with a skin. A creature that eats, rests, reacts to weather, and has a relationship with the ecosystem. The garden is the gnome's home.

## Design

### Zone Painting (replaces instant placement)

All tool actions go through the gnome's task queue. No instant placement mode for players (dev-only debug shortcut available).

- Player selects a tool (seed bag, shovel, soil, stone) — note: watering can removed, see `decisions/2026-03-17T12:00:00_irrigation_replaces_watering_can.md`
- **Drags** across surface to paint a zone (single clicks also work — queue of 1)
- Zone appears as **ghost overlay** — translucent colored voxels with tool-specific visual treatment:
  - **Seed zones**: soft green tint with gentle pulse animation
  - **Dig zones**: wireframe/dashed outline with red-brown tint ("will be removed")
  - ~~**Water zones**: translucent blue shimmer~~ *(removed — no water tool, use shovel to dig irrigation channels instead)*
  - **Soil zones**: translucent warm brown
- Right-click to cancel planned zones (removes ghost + dequeues)
- HUD shows queue counter: "12 tasks remaining"

### The Garden Gnome — A Sim Entity

The gnome is a **first-class sim entity** following the fauna pattern (`fauna.rs`). It lives in the simulation, not just the renderer. This means the gnome's position, state, hunger, and relationships are deterministic, saveable, and observable by the automated player agent.

**Why sim-side, not renderer-side:**
- Fauna interactions require the gnome and fauna to exist in the same system — the sim needs to know where both are to trigger encounters
- Save/load preserves gnome state and pending tasks
- The automated playtesting agent (`groundwork-player`) can observe gnome behavior
- The gnome's needs (food, rest) create gentle ecological feedback loops — the garden sustains the gnome, the gnome tends the garden
- Deterministic behavior means gnome interactions are reproducible and testable

#### Core Identity

- **Visual**: ~2.5 voxels tall billboard sprite. Classic gnome look — pointy red hat, round body, white beard, brown boots. Whimsical and warm.
- **Tool-specific**: carries visible tool matching current task (tiny shovel, seed bag)
- **Movement**: walks along surface voxels toward nearest queued task. Simple surface pathfinding (greedy walk-toward-goal with obstacle stepping on 80×80 grid).
- **Pacing**: processes one voxel per 2-3 sim ticks (tunable). Speed multiplier (1x/2x/5x) naturally affects gnome speed.

#### State Machine

```
Idle → Walking → Working → Walking → Working → ... → Idle
  ↕        ↕                                          ↕
Eating  Reacting                                   Resting
```

- **Idle**: no tasks queued. Gnome wanders the garden, pats soil, sits on rocks, admires plants. Occasionally inspects a plant up close.
- **Walking**: moving toward the next task in the queue. Cheerful waddle with bounce.
- **Working**: at task location. Rhythmic digging/planting motion + particle burst per completed voxel.
- **Eating**: gnome is hungry. Walks to nearest berry bush or harvestable food source. Sits and eats. Short animation. Hunger resets.
- **Resting**: gnome is tired (long work queue depleted energy). Sits on a rock or under a tree. Brief pause.
- **Reacting**: gnome encountered fauna. Plays an interaction animation (see Fauna Interactions below).

#### Needs (Gentle, Not Punishing)

The gnome has simple needs that create ecological feedback without stress:

- **Hunger**: depletes slowly as gnome works. When hungry, gnome seeks food from the garden (berry bushes, fruit). If no food available, gnome works slower but *never stops entirely*. A well-fed gnome works at full speed. This creates a gentle incentive to grow food-producing species — the garden sustains its caretaker.
- **Energy**: depletes with heavy work (digging stone is tiring, planting seeds is light). Low energy → gnome takes short rest breaks. Rest spots: under trees (shade), on rocks, near water. A gnome resting under a willow by the stream is a cozy idle moment.
- **Neither hunger nor energy create fail states.** A hungry, tired gnome is slower, never stopped. The garden always makes progress. This aligns with "no fail states" and "cozy, warm, generous."

### Fauna Interactions — Delight Moments

The gnome and fauna share the same sim space. When proximity + conditions align, interaction events trigger. These are observable micro-stories.

#### Squirrel Domestication
- **New fauna type: Squirrel** — spawns near mature oak/berry bush clusters
- Squirrels start skittish (flee when gnome approaches)
- After repeated proximity (gnome works near squirrels over many ticks), trust builds
- Domesticated squirrels **follow the gnome** and help: they carry seeds short distances, cache acorns that sprout later
- Visual: tiny squirrel sprite perched on gnome's shoulder or scurrying alongside
- This is a **discoverable progression** — the player doesn't unlock squirrels, they earn trust through sustained gardening near them

#### Bird Friendship
- Birds that the gnome works near repeatedly become "garden birds" — they perch closer, sing more (when sound is added), and preferentially drop seeds in areas the gnome tends
- A bird landing on the gnome's hat while it rests = target delight moment

#### Bee Awareness
- Gnome steers around active bee zones (gentle path adjustment, not avoidance)
- If gnome plants flowers that attract bees, bees arrive and the gnome does a small celebratory gesture
- Gnome never gets stung — cozy, not adversarial

#### Worm Appreciation
- When digging underground, gnome occasionally finds a worm, holds it up briefly, places it gently aside
- Soil the gnome digs near worm-enriched areas is visibly darker/richer

#### Beetle Coexistence
- Gnome near beetle activity: pauses to watch decomposition, nods appreciatively
- Dead wood that beetles have processed gets prioritized in gnome's autonomous cleanup

### Gnome Autonomy (Idle Behaviors)

When the task queue is empty, the gnome doesn't just stand still. It has low-priority autonomous behaviors:

- **Wander and inspect**: walks to plants, kneels to look at them. If plant is struggling (low water, crowded), gnome frowns slightly — visual cue to the player.
- **Light maintenance**: pulls a weed (removes a random competing seedling in crowded areas). Clears debris from irrigation channels. These are *very* low-frequency and small-scale — not gardening-for-you, just puttering.
- **Rest in beautiful spots**: finds aesthetically nice locations (under canopy, near water, on elevated ground) and sits. The gnome's rest spots implicitly teach the player what a "good" garden looks like.
- **React to garden events**: when a new species appears (pioneer succession), gnome walks over to look. When fauna arrives, gnome watches. When a plant dies, gnome looks sad briefly.

### Technical Approach — Sim-Side

The gnome follows the `fauna.rs` pattern: a lightweight struct in a bevy_ecs Resource, with systems for movement, task execution, needs, and fauna interactions. Exported via zero-copy WASM buffer for the renderer.

#### Sim Layer (Rust)

**New file: `crates/groundwork-sim/src/gnome.rs`**

```rust
// Follows fauna.rs pattern
pub struct Gnome {
    pub state: GnomeState,
    pub x: f32, pub y: f32, pub z: f32,
    pub target_x: f32, pub target_y: f32, pub target_z: f32,
    pub hunger: u8,       // 0=full, 255=starving
    pub energy: u8,       // 255=full, 0=exhausted
    pub walk_speed: f32,  // current speed (affected by needs)
    pub active_tool: u8,  // which tool gnome is carrying
    pub squirrel_trust: u8, // trust level with squirrels (0-255)
}

pub struct GnomeTaskQueue {
    pub tasks: VecDeque<GnomeTask>,
    pub ghost_zones: Vec<GhostZone>,  // for renderer to draw overlays
}

pub struct GnomeTask {
    pub tool: u8,
    pub x: usize, pub y: usize, pub z: usize,
    pub species: Option<u8>,
}
```

**New systems** (added to `create_schedule()`):
- `gnome_plan` — reads task queue, picks next task, sets target
- `gnome_move` — walks toward target (surface pathfinding)
- `gnome_work` — at target: applies tool to voxel grid (same logic as `place_tool`)
- `gnome_needs` — depletes hunger/energy, triggers eating/resting states
- `gnome_fauna_interact` — checks proximity to fauna, triggers interaction events
- `gnome_idle` — autonomous behaviors when queue is empty

**System execution order**: gnome systems run after fauna_update and before fauna_effects, so the gnome and fauna positions are synchronized.

**WASM bridge additions** (`wasm_bridge.rs`):
- `queue_gnome_task(tool, x, y, z, species)` — pushes task (replaces direct `place_tool` for player actions)
- `cancel_gnome_task(x, y, z)` — removes a pending task
- `cancel_all_gnome_tasks()` — clears queue
- `gnome_ptr() / gnome_len()` — zero-copy export of gnome state
- `ghost_ptr() / ghost_len()` — zero-copy export of ghost zone positions for renderer
- `gnome_queue_len()` — pending task count for HUD
- Keep `place_tool` available for dev/debug instant mode

#### Renderer Layer (TypeScript)

**New file: `crates/groundwork-web/src/gardener/gnome.ts`**
- `GnomeRenderer` class following `FaunaRenderer` pattern exactly
- Single billboard sprite (could use InstancedMesh with count=1 for consistency)
- Custom shader: gnome silhouette, tool in hand, state-based animation
- Reads position from WASM export buffer each frame (like fauna)
- Interpolates between tick positions for smooth movement

**New file: `crates/groundwork-web/src/gardener/ghosts.ts`**
- `GhostOverlay` — InstancedMesh for planned zones
- Reads ghost zone data from WASM export buffer
- Tool-specific colors and pulse animation

**Modified files:**
- `controls.ts` — drag-to-zone calls `queue_gnome_task()` instead of `placeTool()`
- `main.ts` — add GnomeRenderer and GhostOverlay to scene, update each frame
- `hud.ts` — queue counter, gnome status (hungry/tired/idle), cancel affordance
- `bridge.ts` — add new WASM imports for gnome task queue and state

### Player Experience Flow

```
1. Select tool (seed bag)
2. Drag across surface → ghost green zone appears
3. Gnome looks up, grabs seed bag, waddles toward zone
4. Arrives at first ghost voxel, kneels, plants seed (particle burst + hop)
5. A squirrel scurries over and watches. Gnome pats it.
6. Waddles to next ghost voxel, plants again
7. Gnome gets hungry → walks to berry bush, eats, returns to work
8. Meanwhile player is painting the next zone elsewhere
9. Ghost voxels disappear one by one as work completes
10. Queue empty → gnome wanders to a willow tree, sits by the stream
11. A bird lands on gnome's hat. The garden hums with life.
```

### The Garden Sustains the Gnome

This creates a beautiful feedback loop:

```
Player plans garden → Gnome tends garden → Garden grows food
                                         → Garden attracts fauna
                                         → Fauna befriend gnome
                                         → Gnome works happier/faster
                                         → Garden thrives more
```

The gnome's needs are never urgent. They're a gentle nudge: "your gnome is a bit hungry — do you have any berry bushes?" This teaches the player to think about the garden as a complete ecosystem that includes its caretaker.

## Relationship to Existing Decisions

### Extends: SimCity-style Zone Gameplay (2026-03-15T20:30:00)
That decision shifted from voxel-by-voxel to zone-based placement. This decision adds the *execution layer*: zones aren't instant — they're plans that the gnome carries out. The zone concept stays; the execution model changes.

### Extends: Fauna System (fauna.rs)
The gnome follows the fauna architecture pattern (lightweight struct, flat export buffer, sim systems). Fauna interactions are bidirectional — fauna react to gnome presence, gnome reacts to fauna. This deepens the interaction web.

### Strengthens: Gameplay Depth Principles
- Principle #1 (garden feels alive): the gnome IS a resident of the living garden
- Principle #4 (interactions, not just growth): gnome-fauna interactions are new interaction chains
- Principle #5 (garden exceeds the plan): squirrels cache acorns, gnome does light maintenance — the garden grows beyond what the player explicitly zones
- Principle #6 (cozy, warm, generous): the gnome IS coziness embodied
- Principle #7 (idle time rewarding): watching the gnome eat, rest, interact with fauna is the idle reward
- Principle #8 (recovery): gnome can visibly participate in recovery (light weeding, watering stressed plants)

### Strengthens: Big Yeses
- **X-Ray Garden**: gnome visible underground with lantern while digging; worm encounter moments
- **Visible Fauna as Ecological Wiring**: gnome-fauna interactions make fauna feel even more real — they react to a character, not just to conditions. A bird that lands on the gnome's hat is more memorable than a bird that circles above trees.
- **Knowledge-as-Progression**: gnome needs teach the player about food-producing species. Squirrel trust teaches patience and sustained presence. The gnome's reactions (frowning at stressed plants) teach garden diagnostics.
- **Garden Composes Itself**: domesticated squirrels cache acorns = new autonomous planting source. Gnome idle maintenance = garden self-tending.
- **Readable Interaction Chains**: gnome→berry bush→eat→energy→work faster→garden grows is a visible chain. Squirrel trust→seed carrying→unplanned plant is another.

### No Conflicts with Big Nos
- No time pressure: gnome works at its own pace; hunger/energy slow but never stop
- No fail states: gnome never dies, never leaves, never refuses to work
- No progression gates: gnome available from minute one; squirrel trust is emergent, not gated
- No stat overlays: gnome's state is readable from animations (waddle speed, facial expression), not numbers

## New Fauna: Squirrel

Added to the fauna roster for the temperate biome:

- **Spawn condition**: 2+ mature oak or berry bush trees within radius
- **Behavior**: gathers acorns/berries, caches them in soil (cached seeds can sprout)
- **Gnome relationship**: starts skittish (flees within 5 voxels). Trust builds through repeated co-presence. At high trust: follows gnome, carries seeds, perches on shoulder.
- **Ecological role**: seed dispersal (like birds, but shorter range and ground-based). Creates clumped plantings near existing trees.
- **Visual**: small, warm brown, bushy tail. Quick scurrying movement.

## Tuning Parameters

| Parameter | Default | Purpose |
|-----------|---------|---------|
| `TICKS_PER_TASK` | 2-3 | Sim ticks per gnome action |
| `GNOME_WALK_SPEED` | 1.5 voxels/tick | Base walk speed |
| `HUNGER_RATE` | 1 per 5 work actions | How fast hunger builds |
| `ENERGY_RATE` | 1 per 3 work actions | How fast energy depletes |
| `HUNGER_SLOWDOWN` | 0.7x at hunger>200 | Speed penalty when hungry |
| `ENERGY_SLOWDOWN` | 0.6x at energy<50 | Speed penalty when tired |
| `SQUIRREL_TRUST_GAIN` | +1 per 10 ticks co-present | Trust building rate |
| `SQUIRREL_TRUST_FOLLOW` | 180 | Trust threshold for following |
| `MAX_QUEUE_SIZE` | 200 | Cap on pending tasks |
| `INSTANT_MODE` | false | Dev-only bypass |

## Implementation Phases

### Phase 1: Gnome Core (sim + renderer)
- `gnome.rs`: Gnome struct, GnomeTaskQueue resource, gnome_move, gnome_work systems
- WASM bridge: queue_gnome_task, gnome state export
- `gnome.ts` renderer: billboard sprite, walk/work animations
- `ghosts.ts`: ghost zone overlay
- `controls.ts`: zone painting → queue_gnome_task

### Phase 2: Gnome Needs
- Hunger/energy systems in gnome.rs
- Eating and resting states
- HUD status indicators
- Speed modulation from needs

### Phase 3: Fauna Interactions
- Squirrel fauna type in fauna.rs
- gnome_fauna_interact system
- Trust mechanics
- Interaction animations in renderer
- Bird-on-hat, worm encounter, bee awareness

### Phase 4: Gnome Autonomy
- Idle wandering and inspection behaviors
- Light maintenance (micro-weeding, spot-watering)
- Garden state reactions (frown at stressed plants, celebrate new growth)

## Success Criteria

1. Placing seeds/digging feels *delightful*, not slower — the pacing feels like gardening, not waiting
2. The gnome becomes a character players talk about ("my little gnome is working so hard")
3. Ghost zones make planning visible and satisfying
4. Idle watching (gnome working, interacting with fauna) is as engaging as active zone painting
5. Gnome-squirrel trust building is a discoverable "aha" moment
6. Gnome's needs create gentle incentive to grow diverse gardens without adding stress
7. The gnome's idle reactions teach the player about garden health
8. Save/load preserves gnome state, task queue, and fauna relationships
9. All existing sim tests pass — gnome systems are additive, not breaking

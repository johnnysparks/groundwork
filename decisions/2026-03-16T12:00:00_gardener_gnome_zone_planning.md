# Decision: Garden Gnome Character & Zone-Planning System

**Date:** 2026-03-16T12:00:00
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

## Design

### Zone Painting (replaces instant placement)

All tool actions go through the gnome's task queue. No instant placement mode for players (dev-only debug shortcut available).

- Player selects a tool (seed bag, shovel, watering can, soil, stone)
- **Drags** across surface to paint a zone (single clicks also work — queue of 1)
- Zone appears as **ghost overlay** — translucent colored voxels with tool-specific visual treatment:
  - **Seed zones**: soft green tint with gentle pulse animation
  - **Dig zones**: wireframe/dashed outline with red-brown tint ("will be removed")
  - **Water zones**: translucent blue shimmer
  - **Soil zones**: translucent warm brown
- Right-click to cancel planned zones (removes ghost + dequeues)
- HUD shows queue counter: "12 tasks remaining"

### The Garden Gnome

A charming **garden gnome** character — the soul and caretaker of the garden:

- **Visual**: ~2.5 voxels tall billboard sprite. Classic gnome look — pointy red hat, round body, white beard, brown boots. Whimsical and warm.
- **Tool-specific**: carries visible tool matching current task (tiny shovel, watering can, seed bag)
- **Renderer-only**: lives entirely in Three.js, NOT a sim entity. Reads the task queue and calls `placeTool()` when arriving at each location.
- **Movement**: walks along surface voxels toward nearest queued task. Simple step-toward-goal (no complex pathfinding needed on 80x80 grid).
- **Pacing**: processes one voxel per 2-3 sim ticks (tunable). Speed multiplier (1x/2x/5x) naturally affects gnome speed.

**Personality animations** (shader-driven):
- **Walking**: cheerful waddle with slight bounce, hat bobs
- **Working**: rhythmic digging/planting motion + particle burst per completed task
- **Idle**: pats the soil, looks around admiringly, occasionally sits on a rock
- **Celebrating**: small hop + sparkle when a plant grows from their work
- **Underground**: descends with tiny lantern glow when digging (visible in x-ray mode)

### Technical Approach

**All renderer-side — zero sim changes.** The gnome is a visual mediation layer between the player's intent (zones) and the existing `placeTool()` WASM call.

```
Player drags zone → TaskQueue (JS) → GhostOverlay renders plan
                                    → GardenerSprite walks to task
                                    → On arrival: placeTool() (existing WASM)
                                    → Ghost voxel removed, particle burst
```

**New files:**
- `crates/groundwork-web/src/gardener/queue.ts` — TaskQueue data structure
- `crates/groundwork-web/src/gardener/gardener.ts` — GardenerSprite billboard character
- `crates/groundwork-web/src/gardener/ghosts.ts` — GhostOverlay InstancedMesh rendering
- `crates/groundwork-web/src/gardener/movement.ts` — Surface-walking movement

**Modified files:**
- `controls.ts` — drag-to-zone replaces instant placement
- `main.ts` — wire gardener into render loop, drain queue on tick
- `hud.ts` — queue counter, cancel affordance

### Player Experience Flow

```
1. Select tool (seed bag)
2. Drag across surface → ghost green zone appears
3. Gnome looks up, grabs seed bag, waddles toward zone
4. Arrives at first ghost voxel, kneels, plants seed (particle burst + hop)
5. Waddles to next ghost voxel, plants again
6. Meanwhile player is painting the next zone elsewhere
7. Ghost voxels disappear one by one as work completes
8. Queue empty → gnome sits down, pats the soil, looks around proudly
```

## Relationship to Existing Decisions

### Extends: SimCity-style Zone Gameplay (2026-03-15T20:30:00)
That decision shifted from voxel-by-voxel to zone-based placement. This decision adds the *execution layer*: zones aren't instant — they're plans that the gnome carries out. The zone concept stays; the execution model changes.

### Strengthens: Gameplay Depth Principles
- Principle #6 (cozy, warm, generous): the gnome IS coziness embodied
- Principle #7 (idle time rewarding): watching the gnome work is the idle reward
- Principle #1 (garden feels alive): the gnome is another living presence in the garden
- Principle #8 (recovery): the gnome can visibly repair damage, making recovery feel organic

### Compatible: Big Yeses
- X-Ray Garden: gnome visible underground with lantern while digging
- Visible Fauna: gnome coexists with fauna; potential future interactions (gnome waves at birds, avoids bees)
- Knowledge-as-Progression: pacing gives player time to observe and learn
- Garden Composes Itself: gnome doesn't interfere with autonomous growth — only executes player-initiated tasks
- Readable Interaction Chains: slower execution means chains unfold at observable pace

### No Conflicts with Big Nos
- No time pressure: gnome works at its own pace; queue has no deadline
- No fail states: if queue backs up, gnome just keeps working
- No progression gates: gnome available from minute one
- No stat overlays: ghost zones are visual, not numeric

## Tuning Parameters

| Parameter | Default | Purpose |
|-----------|---------|---------|
| `TICKS_PER_TASK` | 2-3 | Sim ticks per gnome action |
| `GARDENER_SPEED` | 1.5 voxels/tick | Walk speed |
| `MAX_QUEUE_SIZE` | 200 | Cap on pending tasks |
| `INSTANT_MODE` | false | Dev-only bypass |

## Future Extensions (P3)

- Multiple gnomes for larger gardens
- Gnome personality traits (some work faster, some are more careful)
- Gnome reactions to garden state (worried in drought, happy near flowers)
- Gnome home/resting spot the player can place
- Gnome interacting with fauna (waving at birds, dodging bees)

## Success Criteria

1. Placing seeds/digging feels *delightful*, not slower — the pacing feels like gardening, not waiting
2. The gnome becomes a character players talk about ("my little gnome is working so hard")
3. Ghost zones make planning visible and satisfying
4. Idle watching (gnome working) is as engaging as active zone painting
5. No regression in core sim behavior — all existing tests pass

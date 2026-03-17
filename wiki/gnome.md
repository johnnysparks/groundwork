# Garden Gnome

The garden gnome is the player's agent in the world. Instead of instantly mutating voxels, the player **paints zones** and the gnome walks over and does the work. This creates natural pacing, character delight, and idle-time reward.

The gnome is a **sim-side entity** (following the fauna pattern): a lightweight struct in a `GnomeData` Resource, with packed export buffers for zero-copy WASM transfer.

## State Machine

| State | Description |
|-------|-------------|
| Idle | No task. Gnome stands at last position. |
| Walking | Moving toward the next task's target position. |
| Working | At the task location, applying the tool (takes 3 ticks). |
| Eating | Hungry gnome pauses to eat (15 ticks, resets hunger to 0). |
| Resting | Tired gnome rests (20 ticks, restores energy to 255). |
| Wandering | No tasks queued — gnome ambles to a nearby spot (60% speed). |
| Inspecting | Arrived at wander target — pauses to look around (12 ticks). |

## Task Queue

- Players queue tasks via `queue_gnome_task(tool, x, y, z, species)` WASM export
- Max queue size: 200 tasks
- Tasks execute FIFO — gnome walks to each in order
- Cancel individual tasks with `cancel_gnome_task(x, y, z)` or all with `cancel_all_gnome_tasks()`

### Tools

| Code | Tool | Action |
|------|------|--------|
| 0 | Shovel | Removes any non-air voxel — primary irrigation tool (dig channels from pond) |
| 1 | Seed | Places a seed (with species id); falls via gravity |
| 2 | Water | Debug only — irrigation via digging replaces the watering can |
| 3 | Soil | Places soil; falls via gravity — build berms, dams, raised beds |
| 4 | Stone | Places stone directly (no gravity) — permanent water barriers/aqueducts |

## Movement

- Walk speed: 1.5 voxels/tick (base), modified by needs
- **Hunger penalty:** speed x0.7 when hunger > 200
- **Energy penalty:** speed x0.6 when energy < 50 (stacks with hunger: x0.42 minimum)
- Gnome walks along the surface (z = topmost solid + 1)
- Arrives when within 1 voxel of target (2D distance)
- Position is fractional (f32) for smooth interpolation on the renderer side

## Work

- Each task takes 3 ticks of work after arrival
- Tool application mirrors `place_tool` logic: gravity for non-stone, air-only target, seed species mapping
- After completing a task, gnome returns to Idle and picks up the next queued task

## Needs

Hunger and energy create a soft pacing constraint — the gnome can always work, but an overworked gnome slows down and takes breaks.

### Hunger (0 = full, 255 = starving)
- Increases by +1 every 5 completed work actions
- At hunger >= 180: gnome enters **Eating** state (pauses 15 ticks, resets to 0)
- At hunger > 200: walk speed drops to 70%

### Energy (255 = full, 0 = exhausted)
- Decreases by 1 every 3 completed work actions
- At energy <= 30: gnome enters **Resting** state (pauses 20 ticks, restores to 255)
- At energy < 50: walk speed drops to 60%

### Design Intent
Needs never create a fail state — the gnome always recovers. They create natural breathing moments: the player zones a big area, the gnome works through it, then pauses to eat and rest before continuing. This makes the gnome feel alive, not robotic. Speed penalties make the player *notice* the gnome is struggling, teaching them to pace their requests.

## Idle Wandering

When the task queue is empty, the gnome doesn't just stand still — it explores the garden autonomously, preferring interesting spots.

- **Cooldown:** 25 ticks idle before picking a wander target
- **Target selection:** Samples up to 24 random positions within 15 voxels, preferring spots near **leaves, trunks, seeds, or water**. Falls back to random if nothing interesting is nearby.
- **Wander speed:** 60% of base (relaxed amble, not purposeful walk)
- **Inspect:** On arrival, gnome pauses 12 ticks (looking around) then returns to Idle
- **Cycle:** Idle (25 ticks) -> Wandering -> Inspecting (12 ticks) -> Idle -> repeat
- **Task interruption:** Queuing a task immediately cancels wandering/inspecting and switches to Walking

This makes the gnome feel alive during idle time — it roams to plants and water features, stops to admire them, then moves on. The targeted selection means the gnome naturally gravitates toward the parts of the garden the player has developed, reinforcing the feeling that the gnome cares about the ecosystem.

## Ghost Overlays

Pending tasks are exported as **ghost zones** — the renderer draws translucent overlays showing planned-but-not-executed work. Each ghost is 8 bytes: `[x: u16, y: u16, z: u16, tool: u8, species: u8]`. Max 200 ghosts exported per frame.

## ECS Systems

Runs every tick in this order (after fauna_update, before fauna_effects):
1. `gnome_plan` — picks next task or wander target, transitions Idle -> Walking/Wandering (tasks interrupt wander)
2. `gnome_move` — moves toward target (Walking at full speed, Wandering at 60%), transitions to Working/Inspecting on arrival
3. `gnome_work` — counts work ticks (or inspect ticks), applies tool on completion, applies hunger/energy cost, transitions to Idle
4. `gnome_needs` — checks hunger/energy thresholds, triggers Eating/Resting from Idle, counts down needs timers
5. `gnome_fauna_interact` — fauna proximity checks, trust building, behavior modification (every 5 ticks)
6. `gnome_export` — packs gnome state + ghost data into export buffers for WASM bridge

## WASM Bridge

| Export | Description |
|--------|-------------|
| `queue_gnome_task(tool, x, y, z, species)` | Queue a task (returns false if full) |
| `cancel_gnome_task(x, y, z)` | Cancel tasks at position |
| `cancel_all_gnome_tasks()` | Clear entire queue |
| `gnome_ptr()` / `gnome_len()` | Packed gnome state (32 bytes) |
| `ghost_ptr()` / `ghost_len()` | Packed ghost zone data (8 bytes each) |
| `gnome_queue_len()` | Number of pending tasks |

### Export Format (32 bytes)

```
[state: u8, active_tool: u8, hunger: u8, energy: u8,
 x: f32le, y: f32le, z: f32le,
 target_x: f32le, target_y: f32le, target_z: f32le,
 queue_len: u16le, squirrel_trust: u8, nearby_fauna: u8]
```

## Fauna Interactions

The `gnome_fauna_interact` system runs every 5 ticks and checks all fauna within 8 voxels of the gnome.

### Squirrel Trust
- **Trust builds** through co-presence: +1 every 10 ticks while a squirrel is within 8 voxels (0-255 scale)
- **Follow threshold:** At trust >= 180, squirrels adjust their target to follow the gnome (offset +2 voxels)
- Trust persists across tasks — a gnome working near oak/berry bushes (where squirrels spawn) gradually befriends them
- Future: trusted squirrels assist with acorn caching (seed dispersal)

### Bird Attraction
- Birds within 5 voxels of a **working** gnome redirect their target to stay nearby
- Creates an emergent "bird perching on shoulder" feel when the gnome digs or plants

### Nearby Fauna Count
- Total count of fauna within 8 voxels is exported as `nearby_fauna` (u8)
- The renderer can use this to trigger emotion particles (hearts, musical notes) on the gnome sprite

# Garden Gnome

The garden gnome is the player's agent in the world. Instead of instantly mutating voxels, the player **paints zones** and the gnome walks over and does the work. This creates natural pacing, character delight, and idle-time reward.

The gnome is a **sim-side entity** (following the fauna pattern): a lightweight struct in a `GnomeData` Resource, with packed export buffers for zero-copy WASM transfer.

## State Machine

| State | Description |
|-------|-------------|
| Idle | No task. Gnome stands at last position. |
| Walking | Moving toward the next task's target position. |
| Working | At the task location, applying the tool (takes 3 ticks). |

Future phases will add Eating, Resting, and Reacting states.

## Task Queue

- Players queue tasks via `queue_gnome_task(tool, x, y, z, species)` WASM export
- Max queue size: 200 tasks
- Tasks execute FIFO — gnome walks to each in order
- Cancel individual tasks with `cancel_gnome_task(x, y, z)` or all with `cancel_all_gnome_tasks()`

### Tools

| Code | Tool | Action |
|------|------|--------|
| 0 | Shovel | Removes any non-air voxel |
| 1 | Seed | Places a seed (with species id); falls via gravity |
| 2 | Water | Places water; falls via gravity |
| 3 | Soil | Places soil; falls via gravity |
| 4 | Stone | Places stone directly (no gravity) |

## Movement

- Walk speed: 1.5 voxels/tick
- Gnome walks along the surface (z = topmost solid + 1)
- Arrives when within 1 voxel of target (2D distance)
- Position is fractional (f32) for smooth interpolation on the renderer side

## Work

- Each task takes 3 ticks of work after arrival
- Tool application mirrors `place_tool` logic: gravity for non-stone, air-only target, seed species mapping
- After completing a task, gnome returns to Idle and picks up the next queued task

## Needs (Phase 2, stubbed)

- **Hunger:** 0 = full, 255 = starving. Not yet active.
- **Energy:** 255 = full, 0 = exhausted. Not yet active.

Future: the garden sustains the gnome (berries for food, shade for rest). The gnome's wellbeing becomes a soft constraint on how much work you can queue.

## Ghost Overlays

Pending tasks are exported as **ghost zones** — the renderer draws translucent overlays showing planned-but-not-executed work. Each ghost is 8 bytes: `[x: u16, y: u16, z: u16, tool: u8, species: u8]`. Max 200 ghosts exported per frame.

## ECS Systems

Runs every tick in this order (after fauna_update, before fauna_effects):
1. `gnome_plan` — picks next task from queue, sets target, transitions Idle -> Walking
2. `gnome_move` — moves gnome toward target, transitions Walking -> Working on arrival
3. `gnome_work` — counts work ticks, applies tool on completion, transitions Working -> Idle
4. `gnome_export` — packs gnome state + ghost data into export buffers for WASM bridge

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
 queue_len: u16le, _pad: u16]
```

## Fauna Interactions (planned)

- Gnome builds trust with squirrels over time (proximity-based)
- Trusted squirrels assist with acorn caching (seed dispersal)
- Birds may follow a well-established gnome path

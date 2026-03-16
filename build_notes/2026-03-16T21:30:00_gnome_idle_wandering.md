# Build Notes: Gnome Idle Wandering (Sprint 82)

**Date:** 2026-03-16T21:30:00
**Sprint:** 82

## What Changed

When the gnome has no tasks, it now autonomously wanders around the garden instead of standing still. Creates the feeling of a living garden caretaker who explores and inspects on its own.

### Sim Side (gnome.rs)

- Added `Wandering = 5` and `Inspecting = 6` to GnomeState
- Added `idle_timer` (u8) and `idle_seed` (u16) to Gnome struct
- Constants: `IDLE_WAIT_TICKS = 25`, `INSPECT_TICKS = 12`, `WANDER_RADIUS = 15.0`
- `gnome_plan` now handles idle behavior selection:
  - Idle with no tasks + idle_timer expired → picks random wander target → Wandering
  - Tasks arriving interrupt Wandering/Inspecting immediately → Walking
  - After idle_timer cooldown, needs system gets a chance to trigger eating/resting
- `gnome_move` handles Wandering (60% walk speed for relaxed feel, arrives → Inspecting)
- `gnome_work` handles Inspecting (12-tick timer, then → Idle with fresh idle_timer)
- Deterministic wander targets via Knuth multiplicative hash on idle_seed counter
- 2 new tests: `gnome_wanders_when_idle`, `tasks_interrupt_wandering` (13 total)

### JS Side

- bridge.ts: Added `Wandering: 5`, `Inspecting: 6` to GnomeState const
- gardener.ts:
  - Added `Wandering` to JS GnomeState enum
  - New `setWanderTarget(x, y)` public method (only triggers from Idle state)
  - New `updateWandering()` with gentle walk animation (50% speed, softer bounce)
  - On arrival: triggers InspectingPlant idle behavior for 3s
  - Task queue interrupts wandering to immediately start walking to task
- main.ts: Gnome sim sync block now reads Wandering/Inspecting states and calls `setWanderTarget` with sim target position

## Architecture Notes

The idle wander cycle: Idle (25 tick cooldown) → Wandering (walk to random spot at 60% speed) → Inspecting (12 ticks) → Idle (repeat)

JS gardener "loosely follows" sim wandering — reads target position but walks at its own visual pace. Sim may complete a wander cycle before JS gardener arrives; this is fine since the visual is a relaxed approximation. Both respond independently to task interrupts.

## What's Next

- Sim→JS position sync (replace JS movement with sim authority for all states)
- Targeted wandering (wander to interesting voxels — plants, water, trees — instead of random spots)

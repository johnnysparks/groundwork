# Reduce Progression Intensity — Gentle Onboarding Redesign

**Date:** 2026-03-17
**Status:** Accepted (executive mandate)
**Author:** Player feedback → Manager

## Context

The first 1-2 minutes of gameplay are overwhelming. Too many systems, tools, and events fire simultaneously. The player is bombarded with information before they've had a chance to feel the garden's warmth. The fun factor is low because there's no breathing room — no moment to just *be* in the space before mechanics start layering on.

## Decision

Redesign the quest progression to be dramatically slower and more intentional. Each quest line introduces exactly one concept and one interaction. The garden should feel cozy and discoverable, not like a tutorial gauntlet.

## New Progression (Quest Lines)

### Quest Line 0: Meet Your Gnome
- **Start:** Empty green meadow, small pond, gnome standing nearby. No UI visible.
- **Action: "Tap the gnome"** — clicking/tapping the gnome makes the camera smoothly follow to focus on him. This teaches "click = interact."
- **Camera quests remain** — pan/orbit to look around. Completing camera movement unlocks a target (shows the pond/spring).

### Quest Line 1: Start Your Garden
- **Action: "Sow small"** — The seed tool appears. Sowing is zone-painting (SimCity style). Player doesn't choose species — they just paint a zone and seeds appear. They don't know what will grow yet.
- **Action: "Inspect"** — After seeds are placed, tapping a seeded cell opens the **inspect panel**. Player discovers the species name and sees soil conditions (e.g. "medium moisture / dry"). The soil isn't dead, but conditions are modest — plants will grow at a slow, watchable pace.

### Quest Line 2: See Below the Surface
- **Action: "X-ray"** — Introduces x-ray mode. X-ray now has a **lens picker** (not just on/off). Default lens is "Roots" — transparent ground reveals the growing root systems.
- **Action: "Irrigation lens"** — Player selects the "Irrigation" lens from the x-ray picker. This shows a 3D moisture heatmap:
  - Full water/pond cells: 50% opaque ocean blue
  - Zero moisture cells: 50% opaque brick red
  - In-between: increasingly transparent, neutrally toned
  - Each x-ray lens has its own distinct color theme so it's immediately clear which lens is active.

### Quest Line 3: Shape the Water
- **Action: "Irrigate"** — Shovel tool appears. Player digs channels to redirect water flow (Timberborn-style terrain shaping). Quest advances when any digging changes water flow — player doesn't need to optimize, just experiment.
- **Action: "Bloom"** — Fast-forward time. Once the first flower blooms, fauna is introduced. First bee arrives. This is the first moment of ecological surprise.

### Quest Line 4+: (WIP)
- Future quest lines build on this foundation: competition, synergy, shrubs, trees.
- Each line follows the same pattern: one new concept, one clear action, visible consequence.

## HUD Phase Mapping

| Phase | Quest Line | Visible UI |
|-------|-----------|------------|
| 0 | Meet Your Gnome | Nothing — just garden + gnome |
| 1 | Start Your Garden | Seed tool only, inspect panel |
| 2 | See Below the Surface | X-ray button + lens picker |
| 3 | Shape the Water | Shovel tool, full tool bar |
| 4+ | Bloom & beyond | Score panel, events, full UI |

## X-Ray Lens System

X-ray mode is no longer a simple on/off toggle. It becomes a **lens picker** with multiple visualization modes:

1. **Roots** (default): Transparent ground, visible root networks with species-colored glow
2. **Irrigation / Moisture**: 3D moisture heatmap — blue (wet) to red (dry) with transparent mid-range
3. *(Future lenses: Nutrients, Light, Soil composition, etc.)*

Each lens has a unique color identity so the player always knows which view they're in.

## Key Principles

1. **No species choice at first.** The "sow small" action just plants whatever groundcover fate gives you. Discovery of species comes through the inspect panel *after* planting.
2. **Inspect before optimize.** The player learns to observe before they learn to act. Inspect panel shows what's there; x-ray shows what's hidden.
3. **One concept per quest line.** No quest line introduces more than one new tool or system.
4. **Breathing room.** Phase 0 lasts until the player *chooses* to interact. No auto-advance timer.
5. **Consequence before complexity.** The first bloom + first bee should feel like a reward, not a checkbox.

## Impact

- **quests.ts**: Complete rewrite of quest definitions, chapters, and completion logic
- **hud.ts**: New phase mapping, inspect panel, x-ray lens picker UI
- **overlay.ts**: New irrigation/moisture lens with distinct color scheme
- **controls.ts**: Gnome click detection, inspect mode
- **main.ts**: Updated quest integration, phase 0 behavior, lens picker keybinds
- **CLAUDE.md**: Updated progression documentation

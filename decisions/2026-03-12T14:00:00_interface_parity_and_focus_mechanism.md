# Decision: Interface Parity — CLI and TUI Must Stay Synchronized

**Date:** 2026-03-12T14:00:00
**Author:** Manager
**Status:** Accepted

## Context

The CLI and TUI are two shells over the same sim. Currently they diverge:

- **CLI can do things the TUI can't:** place voxels, fill regions, inspect cells, show status
- **TUI can do things the CLI can't:** navigate depth interactively, auto-play with speed control, see light-aware colors

This matters because our agentic play testers use the CLI while human testers use the TUI. If the interfaces diverge, agents optimize for a CLI game that doesn't reflect the human experience. Feedback from agent sessions becomes unreliable — they can't report on UX friction they never encounter.

## Decision

**Every player-facing action must exist in both CLI and TUI.** When a new capability is added to one interface, the corresponding mechanism must be added to the other in the same sprint.

### The Focus + Tool Model

Both interfaces need a shared interaction model for spatial operations:

**Focus (viewport center / camera position):**
- TUI: Focus is always at the center of the viewport. WASD/arrows pan the viewport over the world — the focus point moves, and the visible window follows. The focused cell is highlighted (yellow). Out-of-bounds areas beyond the grid edge render as `··` (void). This decouples screen size from world size.
- CLI: `groundwork focus <x> <y> <z>` sets the focus point. `groundwork focus` (no args) prints current focus and voxel summary. Focus is persisted in the state file.

**Tool Start / Tool End (range operations):**
- TUI: Tab/Shift-Tab cycles the selected material. Enter begins a tool operation at focus (tool-start). Pan to another position. Enter again applies the operation across the range (tool-end). The pending range is highlighted (blue). Esc cancels without applying.
- CLI: `groundwork tool-start <material>` marks the current focus as the start of a range operation. `groundwork tool-end [--force]` applies the operation from the stored start to the current focus.

This replaces the instant `place <material> <x1..x2> <y1> <z1>` / `fill` pattern with a two-step workflow that mirrors the human experience. The old `place` and `fill` commands remain available as power-user shortcuts but are **not** the primary interaction path for play testing.

**Inspect at focus:**
- TUI: `I` toggles a side panel showing voxel details at focus (material, water, light, nutrient, seed growth diagnostics).
- CLI: `groundwork inspect` (no args) inspects the current focus. `groundwork inspect <x> <y> <z>` still works as a shortcut.

**Status panel:**
- TUI: `T` toggles a side panel showing material counts and tick info (matches CLI `status` output).
- CLI: `groundwork status` (unchanged).

### Why This Matters

The canonical question is: *Can visible ecological cause-and-effect drive satisfying "one more seed" play?*

If agents can scatter seeds across a 20-cell range in one command but humans have to navigate cell-by-cell, we're testing two different games. The agent will never report that "placing seeds feels tedious at scale" or "I couldn't tell where my selection started." Those are exactly the UX signals we need.

### What This Does NOT Mean

- It does not mean the interfaces must look identical — TUI has color/emoji, CLI has text output.
- It does not mean removing power-user shortcuts — `place` and `fill` with explicit coordinates remain for scripting and automation.
- It does not mean blocking sim work — the parity rule applies to player-facing interaction, not internal tooling.

## Implementation Status

All items implemented. Save format bumped to v2 (backward-compatible with v1).

| ID | Interface | Description | Status |
|----|-----------|-------------|--------|
| TUI-01 | TUI | Viewport-centered focus — WASD pans camera, focus always at screen center | Done |
| TUI-02 | TUI | Tool mode — Tab cycles material, Enter start/end, blue range preview, Esc cancel | Done |
| TUI-03 | TUI | Inspect panel — `I` toggles side panel with voxel details + seed diagnostics | Done |
| TUI-04 | TUI | Status panel — `T` toggles side panel with material counts | Done |
| CLI-21 | CLI | `focus [x y z]` — get/set focus, persisted in state file | Done |
| CLI-22 | CLI | `tool-start <material>` / `tool-end [--force]` — two-step range ops via focus | Done |

## Process Rule

**Added to CLAUDE.md and AGENTS.md:** When a sprint adds a new player-facing action, the sprint must include implementations for both CLI and TUI. Neither interface ships alone. Dev assignments must list acceptance checks for both interfaces.

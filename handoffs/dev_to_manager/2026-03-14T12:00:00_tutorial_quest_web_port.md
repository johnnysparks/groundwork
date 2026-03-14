# Dev → Manager Handoff: Tutorial Quest System Web Port

**Date:** 2026-03-14
**Status:** Initial implementation complete, needs playtesting

## Summary

Ported the TUI tutorial quest system (`quest.rs`) to the web UI as a pure TypeScript module (`ui/quests.ts`). 17 quests across 8 chapters guide new players through camera controls, tools, planting, time control, underground exploration, biodiversity, terrain shaping, and growing a tree.

## What shipped
- Quest panel UI (top-left overlay, collapsible with M key)
- Action tracking: pan, orbit, depth change, tool use, species cycling, auto-tick, manual tick
- World-state completion checks: scans voxel grid for water, wet soil, trunks, roots, leaves/branches
- Notification toasts on quest/chapter completion
- Adapted TUI quests for web controls (3D orbit, click-based focus, web keybindings)

## What was cut (needs follow-up)
- **3 quests removed** that need web features first: Open Inspect, Inspect Soil, Use Range Tool
- **No persistence** — quest progress resets on reload
- **No inspect panel** — this is the biggest gap for player understanding (P1 candidate)

## Decisions for manager
1. Should inspect panel be prioritized to restore the 3 cut quests?
2. Quest progress save/load — tie into existing sim save, or separate localStorage?
3. Should the quest panel auto-dismiss after all complete, or stay as a badge?

## Files changed
- **New:** `crates/groundwork-web/src/ui/quests.ts`
- **Modified:** `crates/groundwork-web/src/main.ts`, `crates/groundwork-web/src/ui/controls.ts`
- **New:** `build_notes/2026-03-14T12:00:00_tutorial_quest_web_port.md`

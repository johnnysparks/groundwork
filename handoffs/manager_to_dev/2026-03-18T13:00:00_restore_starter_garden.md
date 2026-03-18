# Manager → Dev Handoff: Restore Starter Garden

**Date:** 2026-03-18T13:00:00
**Priority:** P0 — first impression is dead

---

## Context

Playtest of Sprint 338 (WASM health fix) confirmed health tinting works perfectly. But the default garden is **empty** — bare terrain, pond, gnome, nothing growing. `create_world_with_garden()` was stripped of its starter seeds. The "alive garden" fantasy is absent on first load.

The stress test proved dense planting produces beautiful canopy + ecological drama + die-offs. The problem is purely: no seeds exist by default.

---

## Task: Restore Starter Garden

**Why:** A new player must see life on first load. The garden should already have young plants growing near the pond — moss, grass, some seedlings. This is the foundation of the "cozy living world" fantasy.

**What to do:**
1. In `create_world_with_garden()` (lib.rs), add starter seed placement after terrain generation but before the 50-tick pre-simulation.
2. Plant a mix of species near the pond where soil is moist:
   - 5-8 groundcover seeds (moss, grass, clover) scattered near pond edges (y=18-26)
   - 3-5 tree seeds (oak, birch) set back from pond (y=26-35)
   - 2-3 flower seeds (wildflower, daisy) near groundcover
3. Increase pre-tick count from 50 to 200+ so plants have time to germinate and show visible growth
4. Verify with `npm run playtest` or screenshots that the default world shows green life

**Key constraint:** Seeds need moisture to germinate (water_level >= 30 in themselves or adjacent cells). Place seeds where water_flow will have already moistened the soil by the time seed_growth runs. The pond is at (40, 16) — positions at y=18-24 should have moist soil.

**Key files:**
- `crates/groundwork-sim/src/lib.rs` — `create_world_with_garden()` function
- `crates/groundwork-sim/src/systems.rs` — seed_growth system (requires moisture)
- `crates/groundwork-sim/src/grid.rs` — POND_X=40, POND_Y=16, GROUND_LEVEL=40

**Definition of done:** Fresh page load shows green vegetation visible without any player action. At minimum: groundcover near pond, 1-2 young tree trunks with leaves.

---

## What NOT to do
- Don't make a perfectly curated garden — let the sim grow it organically from seeds
- Don't change the seed germination system
- Don't add new species or tools
- Don't spend time on visual polish

# Player → Manager Handoff

**Session**: Underground physics & edge-case testing (tick 0–156, CLI)
**Full feedback**: `feedback/2026-03-11T12:30:00_underground_and_edge_cases.md`

## Observed

- Water flow works well: spreads horizontally, fills vertical shafts, flows through carved tunnels, absorbs into adjacent soil
- Light propagates through air with gradual attenuation (~2/level) but drops to 0 instantly in soil (binary, no gradient)
- Roots are completely inert — no growth, no water absorption, no nutrient generation
- Nutrients are always 0 everywhere — no nutrient system exists
- Material placement preserves previous voxel's water/light/nutrient state (bug)
- Water spread creates diagonal striped visual artifacts at the frontier after many ticks
- Boundary handling is solid — corners work correctly, out-of-bounds rejected cleanly

## Felt

- Water tunneling was genuinely fun and satisfying — best moment in the session
- Frustrated that roots do nothing after placing them. Built irrigation with nothing to irrigate.
- Confused by stone holding water_level=62 permanently after overwriting wet soil
- Wanted to plant something in the wet soil ring and watch it grow. Couldn't.

## Bugs

1. **Material placement state bleed** (Major): Placing any material on a voxel preserves old water/light/nutrient values. Stone permanently holds water. Breaks sim trust.
2. **Diagonal stripe artifacts in water spread** (Minor): Visual checkerboard at water frontier after 100+ ticks.

## Confusions

- Why does surface soil get light=227 but one layer down is 0? Expected at least a shallow gradient.
- When does soil display as `%` vs `#`? Couldn't tell soil was getting wet until water_level was quite high.

## What made me want to keep playing

Digging a shaft, watching water flood it, then carving a tunnel and seeing water find its way through. The "build underground plumbing" loop is compelling.

## What made me want to stop

Placing roots and nothing happening. No ecological payoff for water management. Ran out of things to discover.

## Requests

1. Fix material placement state bleed (P0 — trust)
2. Give roots at least one behavior: absorb water from adjacent wet soil (P1 — unlocks the core loop)
3. Lower wet-soil display threshold so `%` appears sooner (P1 — faster feedback)
4. Add any nutrient system so there's an ecology chain (P1 — but bigger lift)
5. Light gradient through shallow soil, even 2-3 levels (P2)

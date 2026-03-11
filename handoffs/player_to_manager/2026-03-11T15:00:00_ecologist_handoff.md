# Player → Manager Handoff: Ecologist Session

**Date:** 2026-03-11T15:00:00
**Source:** `feedback/2026-03-11T15:00:00_ecologist_scientist_session.md`

---

## 1. Observed

- Growth viable zone mapped: surface z=16 within ~12 cells of spring edge; underground to z=10 (light=34).
- Growth uses 6-neighbor water check (not seed's own water_level, which is always 0).
- Soil retains moisture after surface water removal — acts as "water battery" sustaining seeds.
- Seeds are 100% light-transparent. Stacking seeds creates "light pipes" delivering more light underground than air does.
- Growth counter: exactly +5/tick, linear, converts to root at 200, resets to 0.
- No iteration-order artifacts. 4 equidistant seeds grew identically and converted same tick.
- `place` overwrites any material with no validation — including water spring cells.

## 2. Felt

Simulation is deeply trustworthy (5/5). Deterministic, symmetric, measurable. Tooling is the bottleneck, not the sim. Every experiment required dozens of CLI invocations.

## 3. Bugs

| ID | Title | Severity | Summary |
|----|-------|----------|---------|
| BUG-1 | Seed placement destroys water spring | Major | `place seed` on spring cell permanently removes water source. No warning, no undo, no regeneration. |
| BUG-2 | Place overwrites any material silently | Major | Stone, water, anything can be overwritten. No validation or `--force` flag. |
| BUG-3 | Seeds as light pipes (possible exploit) | Minor | Seeds have zero light attenuation. Stacking creates underground light advantage. Flag for design review. |

## 4. Confusions

- `inspect` shows seed water_level=0 on a growing seed — confusing because growth is driven by neighbor water, which is invisible.
- nutrient_level is the growth counter but labeled generically. No indication of what 200 means.
- No diagnostic for "why isn't this growing?"

## 5. What Made Me Want to Keep Playing

- Proving the simulation is symmetric was deeply satisfying.
- "Soil as water battery" was a genuine emergent discovery.
- Growth pause/resume felt fair and cozy.
- Clean, deterministic data rewards hypothesis-testing play.

## 6. What Made Me Want to Stop

- CLI is painfully slow for multi-voxel experiments (each call re-invokes cargo).
- No batch inspect.
- No growth diagnostics.
- Accidentally destroying water source with no recovery path.

## 7. Requests

1. **P0:** Placement validation — warn/reject overwriting water sources (or add `--force`).
2. **P1:** Batch inspect — multiple coordinates in one command.
3. **P1:** Growth diagnostic in inspect — show conditions met/unmet, progress X/200, ticks to maturity.
4. **P2:** Water source protection — `spring` material or un-overwritable cells.
5. **P2:** Seed light attenuation — decide if light pipes are intentional, document or fix.
6. **P2:** Growth stage indicators in ASCII view (`s`/`S`/`$` based on progress).

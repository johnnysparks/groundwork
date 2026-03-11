# Player → Manager: Terraformer Session

**Date:** 2026-03-11T18:00:00
**Session:** 200 ticks, one save file, terrain sculpting focus

## Observed

- `fill` works correctly for rectangular regions. Placed ~14 fill commands to build a sunken bowl (3 levels deep), 3-tier plateau with stone walls, water channel, and terrace.
- Water flows downward through channels carved in soil. Does NOT flow laterally into adjacent soil walls.
- Seeds near water grow reliably (channel seeds: 8→roots by tick 50). Seeds far from water stay dormant indefinitely.
- Plateau seeds at Z=18 stayed dormant until a separate water source was placed above them. Then grew within 50 ticks.
- Terrace seeds at Z=16 (on soil, surrounded by soil) never grew in 200 ticks — water flows under the terrace at Z=15, not through it.
- Checkerboard artifact visible at every Z-level with water spread.
- Wet soil diamond pattern from spring reached Y=49 by tick 200 (very wide spread through soil).
- Seed protection blocked accidental overwrite correctly. Error on range overlap stops at first collision without reporting partial success.
- 34 roots, 33 seeds remaining, 1141 wet soil, 1142 water at tick 200.

## Felt

- Sculpting is the best part of the game right now. `fill` + multi-Z viewing = genuine creative expression.
- Stone retaining walls on the plateau looked *designed* in ASCII — satisfying.
- Water + terrain interaction is shallow. Water only falls. My sculpted landscape was cosmetic, not functional, until I added water sources at each elevation.
- Growth along the channel was the payoff moment. The water path = root path was visible ecological cause-and-effect.
- The checkerboard pattern undermines every water view.

## Bugs

1. **Checkerboard water artifact** — Major. 100% repro. Every water spread at every Z-level.
2. **Range placement error on first collision** — Minor. Stops at first overlap, no partial-success report.

## Confusions

- Does stone block water flow? Block light? No in-game way to discover material properties.
- Why don't irrigation channels wet adjacent soil walls? Water seems to only flow downward, not laterally into solid soil.
- Terrace seeds never grew despite being 1 Z-level above flowing water. Is adjacency check only same-Z?

## What made me want to keep playing

- Viewing my sculpted terrain at multiple Z-levels and seeing it actually look like something I designed.
- The wet soil pattern growing outward over ticks — watching the simulation respond to my terrain.
- Channel-side seeds all becoming roots in a line — terrain shapes growth.

## What made me want to stop

- Realizing my terrace seeds would never grow without manual water placement at their exact Z-level.
- The checkerboard pattern making me doubt the simulation.
- No undo for fill mistakes — high cost of experimentation.

## Requests

1. **Root water absorption (SIM-03)** — already on backlog. This is the #1 need. Without it, roots are inert and there's no resource pressure.
2. **Lateral water-soil interaction** — water should soak into adjacent soil, not just flow downward. This makes irrigation channels actually work. May be related to SIM-03 or need its own task.
3. **Fix checkerboard (SIM-04)** — already on backlog. Major visual trust issue.
4. **Fill --dry-run / undo** — terraforming needs reversibility. At minimum, `fill --dry-run` to preview.
5. **Material properties reference** — `help materials` or `info stone` to show what each material does.
6. **Seed adjacency should include Z±1** — if seeds check only same-Z neighbors for water, a seed at Z=16 can't detect water at Z=15. This would make terraces viable.

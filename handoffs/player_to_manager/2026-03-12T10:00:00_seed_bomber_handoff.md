# Player → Manager Handoff: Seed Bomber Session
**Date:** 2026-03-12T10:00:00
**Persona:** Chaotic / impatient — mass-seed, push limits

## Observed
- `fill seed` placed 2,601–3,600 seeds per layer instantly. Fast, responsive, no lag.
- Surface seeds (Z=15) with water above (Z=16) convert to roots at 100% rate within 100 ticks.
- Underground seeds (Z=14 and below) are permanently dormant — water does not flow down through roots or seeds.
- Edge propagation: when a soil border exists between the root layer and the seed layer, seeds adjacent to roots above can grow (200/2,601 in first test). When the entire surface is roots, zero underground propagation occurs.
- Seed protection blocks `place water` on seeds but allows `fill seed` on any material without warning.
- The s→S→* progression is visible as concentric growth rings expanding from water sources.
- After full conversion, the garden is static — no further changes occur regardless of tick count.
- Sky seeds (Z=29) with water at Z=28: only 6/121 grew. Water spreads horizontally, not upward.
- `--force` override works correctly for seed protection.

## Felt
- Initial rush from mass-seed + mass-flood + mass-conversion cycle. Powerful and satisfying.
- Quick boredom after equilibrium. Garden goes from "growing" to "done" with no middle state.
- Frustration at underground seeds being permanently stuck. Expected water to seep down.
- Disappointment that 14,400 seeds produced the same result as 3,600 — no benefit to depth stacking.
- The growth wave (ring of S expanding outward) was the emotional peak of the session.

## Bugs
1. **Water doesn't flow downward** — Major design gap. Underground seeds are unreachable. Blocks the depth/underground fantasy.
2. **Seed protection asymmetry** — Minor. Seeds protected from water overwrite, but water not protected from seed overwrite.
3. **fill doesn't warn about overwriting seeds/roots** — Minor. Could be intentional.

## Confusions
- Light reaches underground (184/255 at Z=14) but water doesn't (0/255). Why?
- Why did edge seeds grow in world 1 but not in world 2? Adjacency rules unclear.
- No visual cue for "why is this seed dormant" without `inspect`.

## What made me want to keep playing
- The fill-flood-grow power cycle. Immediate, dramatic results.
- Seeing the growth ring expand — visible cause and effect.
- Trying increasingly wild configurations to find limits.

## What made me want to stop
- Garden reaches equilibrium and dies as a game within ~150 ticks.
- No differentiation — every seed is identical, every root is identical.
- Underground seeding is a dead-end mechanic.
- Nothing happens after conversion. No lifecycle, no competition, no surprises.

## Requests
1. **P0: Water downward flow** — Without this, the underground camera (signature feature) has nothing to show.
2. **P1: Seed competition / carrying capacity** — Mass-seeding should create natural selection, not uniform monoculture.
3. **P1: Root lifecycle** — Roots need to do something: spread, compete, die, produce nutrients. The game ends when everything is `*`.
4. **P2: Dormancy visualization** — Show why seeds aren't growing without requiring `inspect`.
5. **P2: Seed variety** — Different species would make mass-seeding interesting beyond the first time.

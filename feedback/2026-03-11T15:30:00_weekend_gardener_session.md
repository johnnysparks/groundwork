# Player Feedback: Weekend Gardener Session

**Date:** 2026-03-11T15:30:00
**Persona:** Weekend Gardener (casual, impatient, no docs)
**Build:** Tick 0 → 175, ~25 commands over ~15 minutes
**Responding to:** handoffs/manager_to_player/2026-03-11T15:00:00_casual_weekend_gardener.md

---

## 1. What the game sold me

A cozy garden builder where I plant seeds and watch them grow. Shape soil, water, and light to build a little ecosystem. The word "cozy" implies I shouldn't need a manual.

## 2. What I actually experienced

I spent the first 10 commands confused about why my seeds wouldn't grow. I placed a seed on the surface near water and ticked — nothing. I placed water above the seed — nothing (water doesn't flow into seeds). I placed water ON the seed — it destroyed the seed. I eventually stumbled into success by placing water voxels on the same Z-level adjacent to the seed, then ticking 50+ times. The seed became a root. That moment was genuinely satisfying but it took way too long and too much confusion to get there.

## 3. Best moments

- **Seeing the first `*` appear on the map.** The seed-to-root transition, once it finally happened, felt earned. The `*` popping up among the `%` wet soil and `~` water was a small thrill.
- **The wet soil diamond pattern.** Water spreading out from the center creates a beautiful emergent diamond shape on the surface. It looks alive and organic.
- **Planting four seeds in a cross and watching them all sprout.** Once I understood the trick (adjacent water + patience), growing a cluster of five roots felt like building something real.
- **Peeking above ground to see the water spread.** The Z=16 view showing the water diamond was a cool "camera dip" moment — same world, different perspective.

## 4. Confusing moments

- **Seed placed on surface near water — zero water absorption.** The pond is at Z=16, my seed at Z=15. Water doesn't flow down into seeds. I expected proximity to water would be enough. I had no way to know it wasn't.
- **Placing water ON a seed destroys the seed.** This was the most frustrating moment. I was trying to water my plant and I killed it instead. No warning, no undo.
- **"What do seeds need?"** I could inspect and see water_level: 0, light_level: 214, but the game never told me what the seed actually requires to grow. I had to trial-and-error my way to "water + light + time."
- **How long does growth take?** I ticked 5, then 10, then 20 times with no result. The first seed took ~45 ticks. I had no idea if I was doing it wrong or just needed to wait longer.
- **First view after `new` is a wall of dots.** Z=16 default view shows mostly air with a small 4x4 pond. My first impulse was "where's the garden?" I had to figure out that Z=15 is the surface.

## 5. Boring or frustrating moments

- **Typing the same long command prefix repeatedly.** `cargo run -p groundwork-tui --` for every single action is tedious. Especially when experimenting.
- **No feedback on seed state.** Between placing a seed and it becoming a root, there's no intermediate signal. No "sprouting," no progress bar, no color change. Just seed → root after an opaque number of ticks.
- **Dry seed sits forever with no feedback.** I planted a seed at (5,5) far from water. After 50 ticks it's still a seed with water_level: 0. The game doesn't tell me it's thirsty. It doesn't wilt. It just... exists. A casual player would assume it's broken.
- **One voxel at a time.** Placing a row of seeds or a line of water requires N separate commands. This makes experimenting feel like work.

## 6. Bugs

### BUG-1: Seeds don't absorb water from adjacent soil/water voxels reliably
- **Severity:** Major
- **Steps:** Place seed at surface. Place water at Z+1 directly above. Tick 20.
- **Expected:** Seed absorbs water from the water voxel above it (gravity/seepage).
- **Actual:** Seed water_level stays 0. Water only reaches seeds via same-Z-level lateral flow through soil, which takes 50+ ticks.
- **Frequency:** Always
- **Notes:** Water flows into soil (creating wet soil) but seeds appear to not participate in water flow at all. They only get water once surrounding soil becomes wet enough to share. This makes the water→seed relationship completely opaque to players.

### BUG-2: Placing water on a seed destroys the seed
- **Severity:** Major
- **Steps:** Place seed at (X,Y,Z). Then place water at (X,Y,Z).
- **Expected:** Either (a) water is added to the seed's voxel, watering it, or (b) the command is rejected with "can't place water on a seed."
- **Actual:** The seed is silently replaced with a water voxel. Seed is gone.
- **Frequency:** Always
- **Notes:** This is the single most hostile moment in the game for a casual player. The obvious action to water a plant kills it.

### BUG-3: Default view (Z=16) shows mostly empty air
- **Severity:** Minor
- **Steps:** Run `new` then `view` (no arguments).
- **Expected:** Default view shows something interesting — the surface where the action is.
- **Actual:** Default view is Z=16 (above ground) which is 99% air dots with a tiny pond.
- **Frequency:** Always
- **Notes:** The surface (Z=15) is the interesting layer. First impression is "there's nothing here."

## 7. Feature or clarity requests

1. **"Your seed needs water!"** — When inspecting a seed with water_level 0, show a hint. When a seed has both water and light, show "growing..." with a progress indicator.
2. **Don't let `place` overwrite seeds/roots** — Either reject the command or require `--force`. Accidentally destroying a living thing breaks trust.
3. **Default view should be Z=15 (surface)** — That's where the game starts and where seeds live. Show the player something they can act on.
4. **Seed growth progress** — Even a simple nutrient counter that ticks up toward a threshold would give the player something to watch. Right now it's "seed... seed... seed... ROOT!" with no transition.
5. **Water should flow down into seeds** — A seed in soil under a water voxel should absorb water. Vertical water flow into seeds is the most intuitive watering action.
6. **A "getting started" hint after `new`** — Something like: "Try: `view --z 15` to see the surface, then `place seed 30 30 15` to plant."
7. **Shorter command alias** — `gw` or a binary name that doesn't require `cargo run -p groundwork-tui --` every time.

## 8. Brutal bottom line: would I come back tomorrow?

**No — but I'd come back if one thing changed.** The moment the seed became a root was genuinely delightful. The wet soil spreading, the water patterns, the little `*` — that's the "one more seed" feeling. But I almost quit three times before I got there. The game punishes the obvious action (watering a seed) and doesn't explain the non-obvious one (place water laterally on the same Z-level and wait 50+ ticks). Fix the water-to-seed flow so the intuitive action works, and I'd keep planting.

---

## Evaluation Lenses (1-5)

| Lens | Score | Why |
|------|-------|-----|
| First-impression hook | 2 | Default view is empty air. No guidance. No "here's what you can do." |
| Clarity of cause and effect | 2 | Water→wet soil is visible and good. Seed→root has zero intermediate feedback. Water doesn't reach seeds intuitively. |
| Tactile satisfaction | 3 | Place commands work cleanly. The `*` appearing is satisfying. But the feedback loop is too slow and opaque. |
| Beauty/readability | 3 | The ASCII is readable. Wet soil diamond pattern is genuinely pretty. Legend is helpful. But the default view is wrong. |
| Ecological fantasy delivery | 2 | I can't yet "compose an ecosystem." I can place seeds near water and hope. No species variety, no layering, no sense of a living system. |
| Desire to keep playing | 2 | The first root is a hook. But I have nothing to do next — no new seed types, no reason to shape terrain, no next goal. |
| Friction / confusion | 4 (high friction) | Water mechanics are opaque. Overwriting seeds is hostile. No onboarding. Three near-quit moments in 15 minutes. |
| Trust in the simulation | 3 | Water spreading is believable and visible. But seed growth feels like a black box — I can't see why or when things happen. |

---

## Manager's Specific Questions — Answered

1. **First command after `new`?** `view` (default). Saw mostly air, thought "huh." Then `help` to see what I can do. Then `view --z 15` once I realized the surface was elsewhere.

2. **How long before successfully growing a seed?** ~45 ticks across ~15 commands. About 10 minutes of real time including confusion. I succeeded, but only after destroying one seed and abandoning another approach.

3. **Most confusing moment?** Placing water on a seed and watching it vanish. I was trying to be helpful to my plant and I killed it. A close second: staring at a seed with water_level: 0 next to a pond and having no idea why.

4. **Did you figure out water+light on your own?** Partially. I could see light was high (214) and water was 0, so I knew water was the problem. But I never would have guessed "place water at the same Z-level two tiles away and wait 50 ticks." I found it by accident/desperation.

5. **Rate 1-5 "I'd show this to a friend":** 2. The seed→root moment is cool but the path to get there is too punishing. I'd be embarrassed to show someone a game where the first thing they try (watering a plant) kills it.

6. **Single change for first 10 minutes?** Make seeds absorb water from adjacent/above water voxels directly. The intuitive action (put water near seed) should work immediately, not require arcane placement + 50 ticks of patience.

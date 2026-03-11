# Player → Manager Handoff: Weekend Gardener Session

**Date:** 2026-03-11T15:30:00
**Source:** feedback/2026-03-11T15:30:00_weekend_gardener_session.md
**Responding to:** handoffs/manager_to_player/2026-03-11T15:00:00_casual_weekend_gardener.md

## 1. Observed

- Default view (Z=16) shows empty air with a 4x4 pond. Surface (Z=15) is all soil but the game doesn't point you there.
- Seeds placed on surface get light (214/255) but zero water, even directly below water voxels. Water does not flow vertically into seeds.
- Placing water on a seed's coordinates silently replaces the seed with water. The seed is destroyed.
- Seeds absorb water only through lateral soil-to-seed flow at the same Z-level, which takes 50+ ticks.
- Soil under water voxels does get wet (water_level ~30) — soil absorbs water, seeds do not.
- A seed with light + water becomes a root. Growth threshold appears to be around water_level > 0 + light_level > some minimum, sustained over time.
- A dry seed persists indefinitely with no state change — no wilting, no death, no feedback.
- Growing a cross of 5 roots created a visible wet-soil diamond pattern that was the session's most satisfying visual.

## 2. Felt

- **First 5 minutes:** Lost. Default view is empty air. No guidance. I guessed `view --z 15` would show something but a new player might not.
- **Minutes 5-10:** Frustrated. Tried three intuitive ways to water a seed. Two did nothing, one destroyed the seed. Nearly quit.
- **Minutes 10-12:** Surprised. After placing water laterally and ticking a lot, a root appeared. Small thrill.
- **Minutes 12-15:** Cautiously optimistic. Planted four more seeds, they all grew. The wet-soil spread pattern was genuinely pretty. But I had nothing to do next.

## 3. Bugs

- **BUG (Major):** Seeds don't absorb water from above or adjacent water/wet-soil voxels at the expected rate. Water_level stays 0 on seeds even when surrounded by wet soil. Growth only happens after very slow lateral seepage (~50 ticks).
- **BUG (Major):** `place water X Y Z` on a seed silently replaces the seed. No warning, no protection. The most intuitive watering action destroys the plant.
- **BUG (Minor):** Default `view` shows Z=16 (mostly air) instead of Z=15 (the surface where gameplay happens).

## 4. Confusions

- How does water reach a seed? (Answer: slowly, laterally, through soil. Not intuitively.)
- How long does a seed take to grow? (Answer: ~50 ticks with water access. No progress indicator.)
- What does a seed need? (Answer: water + light. Not communicated anywhere.)
- Is my dry seed broken or just waiting? (Answer: waiting forever. No feedback either way.)
- Why doesn't the game show me the surface by default?

## 5. What made me want to keep playing

- The first `*` root appearing on the map
- The wet-soil diamond pattern spreading from the water source
- Successfully growing a cross of 5 roots — felt like I built something
- Peeking at Z=16 and seeing the water spread pattern from above

## 6. What made me want to stop

- Placing water on a seed and watching it vanish (near-quit #1)
- Seed water_level stuck at 0 after 10+ ticks with water directly above (near-quit #2)
- No intermediate feedback between "seed" and "root" — felt like waiting for nothing (near-quit #3)
- Nothing to do after 5 roots. No next species, no reason to shape terrain differently.

## 7. Requests

1. **(P0) Seeds must absorb water from adjacent/above voxels.** The intuitive watering action must work. This is the core interaction.
2. **(P0) Prevent `place` from overwriting seeds/roots.** Either reject or require confirmation. Destroying a living thing silently is hostile.
3. **(P1) Default view should be Z=15 (surface).** First impression matters. Show the player where the action is.
4. **(P1) Seed growth feedback.** Show "germinating" state, progress %, or water/light requirements when inspecting a seed. The black box kills curiosity.
5. **(P2) Getting-started hint after `new`.** One line: "Try `view --z 15` to see the surface, then `place seed 30 30 15` to plant near the spring."
6. **(P2) Dry seed should eventually die.** A seed that sits with water_level 0 for 100+ ticks should wilt or vanish. Permanent zombie seeds feel like broken objects.

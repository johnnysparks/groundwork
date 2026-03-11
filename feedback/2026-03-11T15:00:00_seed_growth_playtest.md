# Player Feedback: Seed Growth Playtest

**Date:** 2026-03-11T15:00:00
**Build tested:** Tick 0–290, new world, CLI mode
**Responding to:** handoffs/manager_to_player/2026-03-11T13:00:00_test_seed_growth.md

---

## 1. What the game sold me

Place seeds near water and light, watch them grow into roots. Shape irrigation to extend your garden. An ecological puzzle where placement matters.

## 2. What I actually experienced

Seeds placed right next to the water spring grew into roots reliably (~50 ticks). That part works. But any attempt to extend water to distant seeds via terrain sculpting failed completely. I dug a diagonal air channel from the spring toward a far corner — water never flowed through it to reach my seeds. I ended up with two zones: seeds-that-grow (within the spring's natural wet soil radius) and seeds-that-sit-forever (everywhere else). The world splits into "close enough to the spring" and "dead zone" with no tool to bridge them.

Underground planting worked at 1 layer down (z=14) when water was placed above. Deeper seeds (z=13) started growing much later as water percolated. This felt correct and interesting — depth is a real tradeoff.

## 3. Best moments

- **First root appearing**: Seeing `*` replace `s` after 50 ticks gave a real "it worked!" moment. I did want to plant another seed immediately.
- **Wet soil ring expanding**: The `%` spreading concentrically from the spring is visually satisfying. It feels alive and organic.
- **Underground root**: Placing a seed underground, adding water above, and watching it eventually grow — this felt like discovering a secret. The depth-vs-access tradeoff is genuinely interesting.

## 4. Confusing moments

- **Nutrient = growth counter is invisible**: The handoff told me nutrient_level is the growth counter, but in gameplay there's no way to tell a just-planted seed from an almost-grown one without running `inspect`. Both show `s`. This makes it impossible to read the system.
- **Light levels underground don't attenuate through seeds**: Seeds at z=14 and z=13 had identical light levels (both 184, later both 154). Normal soil attenuates (154→124→34 from z=14 to z=10), but seed voxels seem to pass light through without reduction. Is this intentional? It means planting seeds underground doesn't reduce light for deeper seeds — which feels wrong.
- **Why did the medium-distance seed (22,30) grow?** It had water_level 0 but nutrient_level 190 (almost fully grown). It seems to detect water from neighboring wet soil voxels even though its own water_level stays 0. This works mechanically but is confusing when inspecting — a growing seed shows 0 water.
- **Irrigation channel concept doesn't map to mechanics**: I assumed digging a trench from the spring would route water. Instead, water at z=16 flows above the trench, and the trench itself stays dry. There's no way for a player to figure out how to extend water reach through terrain sculpting.

## 5. Boring or frustrating moments

- **Seeds far from the spring do nothing**: I placed a seed at (10,10) and ticked 290 times. It never got water. It just sat there with light 227 and nothing happening. No feedback about why. No failure state. Just silence.
- **No way to "irrigate"**: The core fantasy promises "route water and place structures that shift ecology." Right now, the only strategy is "plant near the spring." That's not sculpting or irrigating — it's just proximity.
- **Watching 290 ticks with no change on distant seeds**: The tick command reports changes, which is great, but watching "wet soil: +N" expand while your far seed does nothing is deflating.

## 6. Bugs

### BUG: Light does not attenuate through seed voxels
- **Severity:** minor
- **Steps:** Place seeds at z=14 and z=13 under solid soil at z=15. Tick 10+. Inspect both.
- **Expected:** z=13 seed should have lower light than z=14 seed (light attenuates through material).
- **Actual:** Both had identical light levels (197, then 184, then 154 — always matching).
- **Frequency:** 100%
- **Notes:** Normal soil at the same depths does attenuate (154→124→34). Seeds behave like air for light propagation.

### BUG: Growing seed shows water_level 0 despite actively growing from nearby water
- **Severity:** minor
- **Steps:** Place seed at (22,30,15), ~8 tiles from water spring. Tick 50. Inspect.
- **Expected:** If the seed is growing (nutrient_level increasing), its water_level should reflect that it has water access.
- **Actual:** water_level stays 0/255 while nutrient_level climbs to 190. The seed grows to root with water_level 0 the whole time, then suddenly has water after becoming a root.
- **Frequency:** 100%
- **Notes:** This makes `inspect` misleading. A player inspecting a non-growing seed and a growing seed both see water_level: 0.

## 7. Feature or clarity requests

1. **Seed growth visualization**: Show growth progress somehow. Even just different seed characters (`s` → `S` at 50%?) would help immensely. Right now there's zero visual feedback between planting and full growth.
2. **Water routing mechanics**: The game needs a way for players to move water beyond the spring's natural radius. Whether that's channels, aqueducts, or just water placement — "plant near spring" is the only viable strategy right now.
3. **Inspect should show growth progress explicitly**: Instead of (or in addition to) repurposing nutrient_level, show something like `growth: 190/255 (74%)` so the player can read the system.
4. **Seed death or failure feedback**: If a seed can't grow because it has no water, say something. Even a static note in inspect like `growth blocked: no water nearby` would make the system readable.

## 8. Brutal bottom line: would I come back tomorrow?

**Maybe.** The seed→root growth is the first real sign of life in this world, and it genuinely made me want to plant more. But I'd come back only if I had a way to extend water. Right now the entire garden is limited to a ~15-tile radius around the spring. Without irrigation tools, there's exactly one strategy: plant near water. That's not "composing an ecosystem" — it's just proximity. Give me a way to move water and I'll be back for hours.

---

## Evaluation Lenses

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 3/5 | Seed growing into root is a real moment. Water spreading is pretty. But the "what now?" after the first root is unanswered. |
| Clarity of cause and effect | 2/5 | Near spring = grows, far = doesn't. But *why* isn't readable without `inspect`, and `inspect` itself is confusing (water_level 0 on growing seeds). |
| Tactile satisfaction | 3/5 | Placing seeds and ticking feels fine. The `*` appearing is satisfying. But no intermediate feedback during the 40-tick wait. |
| Beauty/readability | 3/5 | Wet soil ring is beautiful. `s` is fine for seed. But `s` looks the same at 0% and 99% growth. |
| Ecological fantasy delivery | 2/5 | One seed type, one water source, one strategy. The underground planting hint is promising but the surface game is "plant near spring." |
| Desire to keep playing | 3/5 | I did want to plant more after the first root. But I quickly hit the wall of "I can't reach anywhere new." |
| Friction / confusion | 2/5 | Irrigation channel attempt was a total dead end. No feedback about why seeds aren't growing. Nutrient-as-counter is opaque. |
| Trust in the simulation | 3/5 | Water spreading feels real. Soil absorption works. But light-through-seeds bug and water_level-0-but-growing undermine trust. |

---

## Answers to Manager's Specific Questions

1. **After placing a seed near the water spring and ticking 50 times, did the seed grow? Rate the satisfaction 1-5.**
   Yes, it grew. **3/5.** The result is satisfying but the 50-tick wait has zero feedback. I was checking `inspect` every 10 ticks to see if anything was happening.

2. **Did you try to build an "irrigation channel → seed bed" setup? Did it work? Was it fun?**
   Yes, I dug a diagonal air trench from the spring to (10,10). It did not work. Water didn't flow through the trench to reach seeds. **Not fun — it was confusing and deflating.** The wet soil ring expanded naturally but ignored my channel.

3. **Is `s` readable as a seed character in the ASCII grid?**
   **Yes**, `s` is distinctive enough. It stands out against `#` and `%`. No confusion there.

4. **After seeing a seed become a root (`*`), did you want to plant another one?**
   **Yes.** This is the strongest moment in the build right now. The "one more seed" pull is real — but it dies quickly when you realize every seed has to go near the same spring.

5. **What's the first thing you wanted to do that you couldn't?**
   **Move water.** I wanted to dig a channel or place water to irrigate a distant seed bed. I couldn't make that work through terrain sculpting, and placing water manually feels like cheating rather than gardening.

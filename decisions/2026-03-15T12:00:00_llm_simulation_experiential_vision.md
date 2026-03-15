# Decision: LLM Simulation — What a Thousand Runs Should Produce

**Date:** 2026-03-15
**Status:** Active
**Context:** The autonomous player agent framework (Phases 1-3) is mechanically complete — scenarios run, failures cluster, regressions generate, skills extract. But the documentation describes *plumbing*, not *destination*. This decision defines what the game should look and feel like after the loop converges.

## The Question

> If we loop through a thousand autonomous runs, follow the feedback, and develop based on it — what does the resulting game look and feel like?

The answer should not be "robust" or "bug-free." It should be: **the coziest, most surprising, most joyful garden sim ever made.** A game where every session teaches you something, every garden exceeds your plan, and you can't stop starting "just one more."

## Experiential Convergence Goals

These are not mechanical targets. They're the *feelings* and *moments* the loop must optimize toward. Every evaluator, every skill extraction, every regression test exists to make these moments more frequent, more reliable, and more varied.

### 1. The First Gasp — "Wait, it did that on its own?"

**Target moment:** The player plants an oak and some wildflowers. Fifty ticks later, tiny pollinators appear between the flowers. The player didn't summon them. They emerged because the conditions were right. The player leans forward.

**What the loop teaches us:** Which species combinations reliably produce emergent fauna. Which timings make the appearance feel surprising rather than instant. Which visual cues make the player *notice* without a tutorial popup.

**Evaluator direction:** Not "did pollinators spawn" but "did pollinators spawn in a way the planner didn't explicitly set up, and did the planner notice them in observations?"

### 2. The Underground Revelation — "Oh. That's why."

**Target moment:** The player's birch is struggling. They dip the camera underground and see the oak's root network has spread wide, drinking all the water. The birch is parched. The player *understands* — not because a tooltip told them, but because they looked and saw the cause with their own eyes.

**What the loop teaches us:** Whether root competition is visible enough to diagnose. Whether players naturally think to look underground when a plant struggles. Whether the visual language of roots, water, and soil reads clearly in the 3D view.

**Evaluator direction:** "Did the planner investigate underground after a plant died? Did the observation reveal root competition? Did subsequent actions reflect that understanding (e.g., planting the birch farther away)?"

### 3. The Slow Bloom — "This garden is better than my last one."

**Target moment:** On their third garden, the player spaces species intentionally. They plant clover first because they learned it fixes nitrogen. They leave gaps for wind-dispersed seeds. They build a water feature knowing it attracts fauna. The garden thrives faster and more beautifully — not because the player memorized a guide, but because they *understand the ecology*.

**What the loop teaches us:** Whether there's a real skill curve — can a knowledgeable player build a measurably better garden? Do early-game mistakes actually teach transferable lessons? Is the gap between a naive garden and an expert garden wide enough to feel like growth?

**Evaluator direction:** Compare run N to run N+10. Is the planner's garden more diverse, more self-sustaining, more surprising? Did skills extracted from early runs produce better outcomes when injected into later runs?

### 4. The Gift — "I didn't plant that there."

**Target moment:** A bird carries an oak acorn across the garden. A seedling appears in a spot the player never touched. It grows. It's perfect there — shading a patch of moss that was struggling in direct sun. The garden composed itself.

**What the loop teaches us:** Whether seed dispersal produces outcomes that feel like gifts rather than noise. Whether wind/bird/water dispersal creates *good* placements often enough to delight. Whether the player can trace the chain backward: "A bird came because I had berry bushes. It carried a seed from my oak. The oak grew and shaded the moss."

**Evaluator direction:** "Did a plant appear where no seed was placed? Did it survive? Did it create a beneficial relationship with neighbors? Could the causal chain be reconstructed from observations?"

### 5. The Living Painting — "I just want to watch it."

**Target moment:** The player stops clicking. Golden hour light sweeps across the garden. Pollinators drift between wildflowers. A worm trail appears in the soil. Leaves sway. Water shimmers. The garden is breathing. The player feels *peace*.

**What the loop teaches us:** Whether idle time is rewarding. Whether there's enough ambient life (fauna movement, growth animation, particle effects, light changes) to make watching feel different from staring at a screensaver. Whether the day cycle and weather create enough variation that "just watching" stays interesting.

**Evaluator direction:** "During a 100-tick observation window with no player actions, did the garden change visibly? Did new fauna appear or move? Did plants grow in observable ways? Did light or atmosphere shift?"

### 6. The Chain Reaction — "One thing led to another."

**Target moment:** The player plants a single wildflower near water. Bees come. Pollination spreads flowers across the bank. Butterflies arrive for the flower density. A bird nests nearby because of insect abundance. The bird drops a berry seed. A bush grows. The player traces six steps of causation from one seed.

**What the loop teaches us:** Whether interaction chains actually form reliably. How long they take. Whether the visual language makes each link visible. Whether chains are varied enough that different starting conditions produce different cascades.

**Evaluator direction:** "What is the longest causal chain in this run? Can each link be identified from observations? Did the chain produce an outcome neither the player nor any single species would have reached alone?"

### 7. The Recovery — "It came back."

**Target moment:** The player over-waters a section. Plants drown. But then — the water recedes, nutrients from decomposition enrich the soil, and pioneer species (moss, grass) colonize the bare patch. Life returns without the player's help. The garden heals.

**What the loop teaches us:** Whether the ecosystem has genuine resilience. Whether there are pioneer/succession dynamics. Whether recovery is visible and satisfying rather than just "stuff regrows." Whether the player learns that mistakes aren't permanent — encouraging bolder experimentation.

**Evaluator direction:** "After a destructive event (flooding, over-digging, crowding), did the ecosystem partially recover autonomously? Did pioneer species appear? Did the recovered state differ from the original (showing adaptation, not just reset)?"

### 8. The Seasonal Shift — "Everything changed and I love the new version."

**Target moment:** The garden the player built for spring looks completely different in autumn. The canopy colors shift. Ground-level light changes as leaves thin. Different fauna appear. The same garden is a *new* garden — and the player wants to see what winter brings.

**What the loop teaches us:** Whether temporal variation creates genuine novelty. Whether seasons change the garden's behavior meaningfully (not just color palette). Whether the player anticipates seasonal shifts and plans for them.

**Evaluator direction:** "Does the garden at tick 500 look and behave meaningfully differently from tick 100? Are different species dominant? Are different fauna present? Would a player who only saw one snapshot miss something important?"

## What "Cozy Blockbuster" Means for the Loop

A cozy blockbuster isn't low-ambition. It's a game that feels *warm, generous, and endlessly rewarding*. The loop should converge toward:

- **Generosity over punishment.** When the loop finds that players fail, the fix should be making the garden more forgiving and recoverable — not adding tutorials. A garden that bounces back from mistakes teaches better than one that dies and makes you restart.

- **Warmth over precision.** When the loop finds that species interactions are hard to see, the fix should be warmer visual feedback (golden particles when pollination happens, gentle sound cues, cozy lighting shifts) — not data overlays or stat panels.

- **Invitation over instruction.** When the loop finds that players don't discover underground mechanics, the fix should be something beckoning from below (root glow, earthworm movement visible at the surface, a plant wiggling as roots spread) — not a "press C to view underground" prompt.

- **Abundance over scarcity.** When the loop finds gardens feel empty, the fix should be more ambient life, more particle effects, more subtle animations — the feeling of a world teeming with tiny activity. Not bigger plants or more species.

- **Surprise over predictability.** When the loop finds that experienced planners achieve perfect scores every time, the fix should be *more variance* — rare fauna, weather events, lucky seed dispersals. The hundredth garden should still contain a moment the player didn't expect.

## Evaluator Evolution

The current evaluators are mechanical: `MaterialMinimum`, `MaterialAbsent`, `MaterialGrew`. To optimize for delight, we need evaluators that measure *experience*:

| Current (Mechanical) | Needed (Experiential) |
|---|---|
| "Did trunk material appear?" | "Did a plant grow that the player didn't place?" |
| "Is water present?" | "Did water create a visible ecosystem effect?" |
| "Did the planner use camera?" | "Did looking underground change the planner's next action?" |
| "No crash" | "Did the garden change meaningfully during idle observation?" |
| "Material count > N" | "How many species are interacting (not just coexisting)?" |
| Pass/fail | "Would the player come back tomorrow, and why?" |

These experiential evaluators are harder to write. Some require comparing planner behavior before and after an observation. Some require detecting causal chains across multiple ticks. Some may need the LLM planner itself to report subjective experience. That difficulty is the point — **if we can't evaluate delight, we can't optimize for it.**

## Skill Extraction Should Capture Wonder, Not Just Technique

Current skill extraction finds patterns like "water_basin" and "planting_near_water." These are *techniques*. The loop should also extract *discoveries*:

- "Clover near oak produced faster growth" → **symbiosis discovery**
- "Pollinators appeared after planting 3+ flowers near water" → **fauna trigger discovery**
- "Underground view revealed root competition causing surface death" → **diagnostic discovery**
- "Bird-dispersed seed created an unplanned but beneficial planting" → **emergence discovery**

These discoveries should feed back into the planner not as "do this action sequence" but as "here's something interesting that happened — explore variations." The planner should become a *curious gardener*, not an optimizing algorithm.

## The Thousand-Run Garden

After a thousand runs of this loop, a new player sits down and experiences:

**Minute 1:** They place soil and water. A seed. The seed sprouts. It's warm, clear, satisfying. The garden is a tiny patch of green in golden light. They smile.

**Minute 5:** They plant several species. Some grow fast (moss spreads, grass fills in). Some grow slow (the oak is just a sapling). The garden feels *busy* — small movements everywhere, growth unfolding at different speeds. They notice tiny particles drifting between wildflowers. What are those?

**Minute 15:** The oak is growing. They dip underground for the first time and see roots spreading like fingers, reaching toward water. A worm trail moves through the soil. They feel like they've peeled back a secret layer of the world. Back on the surface, they notice a wildflower has appeared where they didn't plant one. Where did it come from?

**Minute 30:** They've planted a whole garden now. They notice the birch near the oak is struggling. Underground: the oak's roots are everywhere. They replant the birch near the stream with its own space. It thrives. They feel clever — they *diagnosed* a problem by observing, not by reading a manual. Meanwhile, a butterfly is visiting their flower patch. Berry bushes are fruiting.

**Minute 60:** They stop playing and just watch. The garden breathes. Light shifts. Pollinators move in patterns. A bird silhouette crosses. A new seedling pops up near the berry bush — the bird carried it. Their garden is *making itself better* without them. They feel something between pride and wonder.

**Hour 3:** Third garden. This time they know: clover first for nitrogen, flowers near water for pollinators, space between trees for root room. The garden thrives earlier. But then — a moss patch they didn't plan spreads under the oak canopy, and ferns follow the moss, and the shaded undergrowth becomes its own micro-ecosystem. They didn't design this. The garden did. And it's *beautiful*.

**Hour 10:** They're designing gardens like a landscape architect thinks about ecosystems. Every plant has a role. Every placement considers neighbors, light, water, and the fauna it will attract. But the garden still surprises them — a rare interaction, an unexpected recovery from drought, a seed dispersal that creates a perfect composition no algorithm would design. They start their eleventh garden not because they haven't "won," but because they want to see what this combination of species will create that they can't predict.

**This is the game.** Not a toy you exhaust in twenty minutes. Not a puzzle with a solution. A living world you get better at composing, that always composes something back at you that you didn't expect. Warm, generous, surprising, and endlessly inviting.

## Implications for Development

1. **Every sim feature should be evaluated by the experiential moment it enables**, not just mechanical correctness. Root water absorption isn't a physics system — it's the engine for "The Underground Revelation."

2. **Visual and audio feedback are not polish — they're the delivery mechanism for delight.** Golden pollination particles, gentle growth sounds, warm lighting shifts: these are how the player *knows* something wonderful happened. Without them, the sim's emergent chains are invisible.

3. **The LLM planner should be designed to feel wonder**, not to optimize. Its prompt should encourage curiosity, exploration, and reaction to beauty — not min-maxing growth rates. The planner that says "I noticed something unexpected and investigated" is more valuable than one that achieves 100% coverage.

4. **Failure in the loop should be redefined.** A run where the garden dies but the planner says "I learned that oak roots steal water from birch" is a *success* for discovery, even if the growth evaluator scores zero. The loop must optimize for learning and surprise, not just material counts.

5. **The batch runner should track delight metrics across runs**: longest causal chain, count of unplanned-but-beneficial events, discovery count, idle-period activity level, recovery instances. These are the numbers that tell us whether we're building the right game.

## What the Loop Must Concretely Discover and Amplify

The vibe is the compass, but vibes don't ship. The loop's most important job is finding the **3-5 specific interactions that make this game magical** — and then amplifying them 1000x while cutting everything that competes.

### What "Amplify 1000x" Means in Practice

When the loop discovers an interaction that reliably produces delight:

1. **Name it.** Give it a memorable label. "The Nitrogen Handshake." "The Bird Express." "The Root War." Names make ideas sticky across sessions.
2. **Measure it.** Specific evaluator. How often does it occur? How many ticks to trigger? Does the planner notice it?
3. **Make it gorgeous.** Particles, lighting shifts, subtle camera attention, animation polish. If pollination is a winner, the pollen trail should be the most beautiful particle effect in the game.
4. **Protect it.** Regression test. If a code change breaks this interaction, the build fails.
5. **Build adjacent.** What other interactions connect to this one? Same principle, new chain, compound delight.
6. **Cut competitors.** If a feature distracts from a winner or makes it less visible — cut the feature. Winners are rare.

### Concrete Interactions the Loop Should Probe

These are specific, implementable interaction chains. The loop should test whether they produce the Target Moments, measure the delight, and surface which ones are winners.

**Nitrogen Handshake (Clover + Oak Symbiosis)**
- Clover roots fix nitrogen → soil nutrient level increases in radius → oak growing in that radius grows 30-50% faster
- Visual: warm glow in soil around clover roots (underground view). Oak roots visibly thicken in the enriched zone.
- Test: plant oak alone vs. oak near clover. Is the difference visible and significant? Does the planner discover it?
- If winner: add more symbiosis pairs. Fungi connecting tree roots. Moss retaining moisture for fern.

**The Pollinator Bridge (Wildflower → Bee → Daisy Spread)**
- 3+ wildflowers near water → pollinators spawn → pollinators visit daisies → daisies produce 2x seeds → daisy population explodes
- Visual: golden particle-sprites traveling between flowers. Subtle trail. Pollinated flowers pulse briefly.
- Test: isolated flowers vs. clustered flowers near water. Does the cluster produce fauna? Does fauna produce spread?
- If winner: add butterfly species attracted to density. Add pollinator-preference by flower color/type.

**The Root War (Oak vs. Birch Competition)**
- Two trees planted 5-8 voxels apart → roots grow toward same water → faster-growing species (oak) absorbs more → slower species (birch) wilts
- Visual: underground view shows two root networks meeting. One retreats. Surface plant shows stress (yellowing, slower growth).
- Test: does the planner notice surface stress → investigate underground → diagnose competition → replant? This is the core discovery loop.
- If winner: add root-avoidance strategies. Deep-rooted vs. shallow-rooted species. Rocks as natural root barriers.

**The Bird Express (Berry Bush → Bird → Long-Range Seed)**
- Berry bush fruits → bird silhouette arrives → bird flies 20-50 voxels → drops seed near a tall tree → new seedling
- Visual: bird flight path visible briefly. Seed drop has a tiny sparkle. Seedling appears next tick.
- Test: does the planner notice a plant they didn't place? Can they trace it to the bird? Is the placement often *good* (near water, appropriate light)?
- If winner: add multiple bird species with different perch preferences. Some carry oak acorns, some carry berry seeds. Flight distance varies.

**Pioneer Succession (Bare Soil → Moss → Grass → Wildflower → Shrub)**
- Cleared area auto-colonizes: moss first (5-10 ticks), then grass (15-25), then wildflowers (30-50), then shrubs (100+)
- Visual: visible wave of green spreading across bare soil, shifting in color/height over time. Time-lapse feeling.
- Test: does the planner notice succession happening? Does it change their strategy (clearing ground intentionally to trigger succession)?
- If winner: make succession rates depend on neighbor species. A cleared area near a flower garden recovers differently than one near a pine forest.

**The Canopy Effect (Tall Trees → Shade → Fern/Moss Undergrowth)**
- Oak canopy blocks 60-80% of light → ground-level light drops → sun-loving species (grass, wildflower) struggle → shade-tolerant species (fern, moss) thrive
- Visual: dynamic shadow rendering on the ground. Fern/moss visibly spreading in shaded areas. Grass yellowing in shadow.
- Test: does the planner learn to layer — tall canopy above, shade-lovers below? Does the underground view show different root patterns in shaded vs. sunny areas?
- If winner: add canopy gap dynamics. When a branch falls, a light shaft appears and sun-loving species colonize the gap temporarily.

### What the Loop Should Concretely Cut

Equally important — the loop should identify ideas that *don't produce delight* and kill them quickly:

- If a species never participates in interaction chains → simplify or remove it
- If an underground visual doesn't help the planner diagnose anything → it's clutter, not information
- If a fauna type never changes planner behavior → it's decoration, not gameplay
- If an interaction is too subtle for the planner to notice in observations → it needs visual amplification or it's not real
- If a mechanic produces the same result every time → it needs variance or it's solved

The loop's nos are as valuable as its yeses. A thousand runs should produce a sharp, focused game with 5 deep interactions — not a broad, shallow game with 50 weak ones.

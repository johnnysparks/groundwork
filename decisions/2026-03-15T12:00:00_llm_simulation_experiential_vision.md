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

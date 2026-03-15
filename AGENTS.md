# GROUNDWORK AGENT OPERATING PACK

> **New session?** Start in `CLAUDE.md` — it has your role-specific checklist.
> This file is the reference manual. You don't need to memorize it; come back when you need a format or rule.

## GAME VISION
GROUNDWORK is a cozy ecological builder where the player grows a self-sustaining garden by shaping soil, water, light, and plant relationships above and below ground.

**Core fantasy:** Build a living miniature world that becomes more beautiful, resilient, surprising, and alive every season — a world that develops its own momentum and exceeds your plan.

**Player experience:** The player is composing an ecosystem, not managing units in a spreadsheet. Every choice changes the conditions for what grows next. A thriving garden should feel *earned* — and then it should surprise you. The oak you planted attracts a bird that carries a seed to the far corner. The wildflowers you grew near water attract pollinators that make your berry bush fruit. The garden develops relationships you didn't design but can understand once you see them.

## CORE PLAY
- **Plant:** choose species and place seeds
- **Sculpt:** shape terrain to influence water and space
- **Prune:** cut, weed, thin, deadhead, redirect growth
- **Irrigate:** route water and place structures that shift ecology
- **Observe:** notice emergent interactions — pollinators linking flowers, roots competing for water, fungi colonizing dead wood — and learn what the garden is telling you
- **Adapt:** respond as fauna, seasons, and maturing plants reshape the system in ways you didn't fully plan

## SIGNATURE FEATURE
The player can dip the camera below the surface with no mode switch. Same world, same simulation. The surface shows beauty. The underground reveals why it works.

## MVP TARGET
- One temperate biome, 12-20 species (canopy/shrub/groundcover)
- Four systems: light, water, roots, ecology
- Fauna as visible ecological agents: pollinators, decomposers, seed-dispersing birds, soil organisms — even as simple visual representations (particles, sprites, trails). Fidelity can be low; ecological *role* must be clear.
- Species interaction web: pollination, competition, symbiosis, decomposition, shelter — plants affect each other, not just respond to abiotic conditions
- One garden bed (~120×120×60 voxels, 60m×60m×30m at 0.5m/voxel)
- Continuous above/below-ground camera
- Terrain sculpting, seed placement, basic tending tools
- **Biome-ready architecture**: species rosters, interaction rules, fauna triggers, growth parameters, and visual palettes must be data-driven and biome-parameterized. MVP ships temperate only, but adding a new biome should be a content task (new data, new art), not an architecture rewrite. Biomes are a core replayability pillar — see Replayability Model above.

## DESIGN GUARDRAILS
- Readability over realism
- Player delight over simulation purity
- Small, shippable systems over broad ambition
- Expression from ecological choices, not cosmetic overlays
- Create "one more seed" curiosity — through discovery and surprise, not just legibility
- Reward observation: the player who watches closely should notice things the player who clicks fast misses
- Interaction depth over independent systems: one chain of species affecting each other is worth ten that grow alone

## CANONICAL QUESTION
All agents should keep testing against this:
> *Can ecological discovery — visible interactions, emergent surprises, and layered understanding — drive "one more garden" play?*

This is stronger than mere legibility. A game where cause-and-effect is perfectly visible but fully predictable gets exhausted in 20 minutes. The goal is a garden that teaches you something new each session, where each garden you build makes the next one richer because you understand more about how species, fauna, and systems connect.

If a task does not help answer that, challenge it.

## REPLAYABILITY MODEL

Two pillars drive long-term replay. Both are load-bearing — the game needs them both.

**Pillar 1: Knowledge Transfer — "I learned something. Let me try again."**
Each garden teaches something the player carries to the next one. Not unlocks — understanding. Garden #1: "Seeds need water." Garden #5: "Space trees or their roots fight." Garden #12: "I can design self-sustaining loops." The player starts each garden *differently* because they know more. This is the Outer Wilds model extended: knowledge is the only progression, and the interaction web is deep enough that there's always another layer.

**Pillar 2: Biome Variety — "New organisms, new rules, new vibes."**
Each biome is both a content pack (new species, fauna, interactions, art, atmosphere) and a difficulty curve (different ecological principles to master). Biomes are not ordered — start anywhere, go anywhere. Mastery doesn't transfer cleanly: systemic intuition (look underground, watch fauna, space plants) carries over, but specific recipes (clover + oak = nitrogen boost) don't exist in the desert. The player must re-learn — but they know *how* to learn. Visual identity is itself a pull motivator: the player wants to *see and inhabit* each biome's world.

**How they interact:**
- Within a biome: knowledge transfer drives "one more garden" (hour-to-hour)
- Across biomes: variety drives "one more biome" (week-to-week)
- Cross-pollination: desert teaches water conservation that improves temperate gardens; rainforest teaches vertical layering that improves boreal gardens
- The autonomous loop runs per-biome, surfacing each biome's 3-5 magical interactions independently

See `decisions/2026-03-15T18:00:00_replayability_model.md` for the full replayability arc.

## EXPERIENTIAL NORTH STAR

The autonomous simulation loop exists to make the game feel like this for a new player:

- **Minute 1:** Place soil, water, a seed. It sprouts in golden light. Warm, clear, satisfying. You smile.
- **Minute 5:** Multiple species growing at different speeds. Tiny particles drift between wildflowers. What are those? Lean forward.
- **Minute 15:** Dip underground for the first time. Roots spreading like fingers. A worm trail. Back up top — a wildflower appeared where you didn't plant one. Where did it come from?
- **Minute 30:** Diagnose a struggling birch by looking at the oak's root network underground. Replant it with space. It thrives. You feel clever — you *observed* your way to understanding.
- **Hour 1:** Stop playing. Just watch. Light shifts. Pollinators move. A bird drops a seed. Your garden is making itself better without you. Pride and wonder.
- **Hour 3:** Third garden. You know clover-first for nitrogen, flowers-near-water for pollinators. But a moss carpet spreads under the oak canopy, ferns follow, and a shaded undergrowth micro-ecosystem forms that you didn't design. Beautiful.
- **Hour 10:** You design gardens like a landscape architect. Every plant has a role. But the garden still surprises you. You start your eleventh garden not because you haven't "won" but because you want to see what *this* combination will create.
- **Hour 15:** You notice the desert biome. The art preview pulls you in — stark dawn light, sand textures, unfamiliar silhouettes. You start a desert garden and suddenly you're a beginner again. Clover doesn't exist here. Water is precious. But you know to look underground, to watch fauna, to space plants. You know *how to learn*. The discovery arc reignites.
- **Hour 25:** Your third desert garden. You've found the desert's nitrogen-fixer, its dispersal agent, its own version of the canopy effect. You go back to temperate and notice you're more intentional with water — desert taught you conservation. The biomes cross-pollinate your understanding.
- **Hour 50:** You think in terms of ecological *roles* and *niches*, not species names. Every biome has a nitrogen-fixer, a pioneer colonizer, a canopy-maker, a dispersal agent — but the specific organisms are different, and discovering each biome's web is its own arc. You start a new biome not to "complete" it but because you want to see its world and feel its atmosphere.

Every feature, system, and polish pass should be measured against these moments. If it doesn't enable at least one of them, question why it exists.

## TARGET MOMENTS

These are the specific feelings the game must reliably produce. Reference them by name in evaluators, feedback, and task definitions.

| Moment | Feeling | Example |
|--------|---------|---------|
| **The First Gasp** | "It did that on its own?" | Pollinators appear between flowers the player placed — unsummoned, emergent |
| **The Underground Revelation** | "Oh. *That's* why." | Camera dips below; oak roots have stolen the birch's water |
| **The Slow Bloom** | "This garden is better than my last" | Third garden thrives faster because the player *understands ecology now* |
| **The Gift** | "I didn't plant that there" | A bird carries a seed; a perfect tree grows where the player never touched |
| **The Living Painting** | "I just want to watch" | Golden hour, drifting pollinators, breathing garden — peace |
| **The Chain Reaction** | "One thing led to another" | One wildflower → bees → pollination spread → butterflies → nesting bird → new seedling |
| **The Recovery** | "It came back" | Over-watered section recovers: pioneer moss, then grass, then life returns |
| **The Seasonal Shift** | "Same garden, new world" | Autumn transforms the canopy; different fauna appear; the garden reinvents itself |

See `decisions/2026-03-15T12:00:00_llm_simulation_experiential_vision.md` for full descriptions and evaluator directions.

## THE BIG YESES

The vibe is the compass, but vibes don't ship. These are the concrete mechanics that *deliver* the vision. A small handful of big yeses and a whole lot of nos separate great games from average ones. When one of these ideas is working, amplify it 1000x. Protect these with your life.

### 1. The X-Ray Garden (Signature Mechanic)

**Stolen from:** Terraria's underground-as-gameplay, but for ecology instead of combat.

The camera dips below ground with no mode switch. Same world, continuous. The surface shows beauty; the underground shows *why it works*. This is not a debug view. This is the game's core insight mechanic.

**Why this is a Big Yes:** No other garden game lets you *see* root competition, water flow, soil composition, and nutrient cycling in real time. This is Groundwork's unfair advantage. Every other garden sim is a surface-only guessing game. We show you the engine.

**Concrete execution:**
- Roots render as branching, colored networks — each species has a distinct root color/pattern
- Water flows visibly through soil layers — you watch it get absorbed by roots in real time
- Nutrient hotspots glow subtly — clover's nitrogen fixation shows as a warm zone around its roots
- Worm trails animate through soil, visibly aerating (darkening soil, increasing porosity)
- When two root networks overlap, the competition is *visible* — one species' roots retreat or thin
- Cross-section view at any depth — drag the ground plane up and down like peeling layers off an onion

**Kill metric:** If a player's birch dies and they can dip underground and *see* that the oak's roots drank all the water within 10 seconds of looking, this mechanic is working. If they have to guess, it's not.

### 2. Visible Fauna as Ecological Wiring (Not Decoration)

**Stolen from:** Slime Rancher's readable creature behaviors + Stardew Valley's seasonal wildlife, but where fauna *does* something mechanically important.

Every creature the player sees is performing an ecological role. Bees aren't ambient — they're pollinating, and you can watch the pollen trail. Birds aren't decoration — they're carrying seeds, and you can see where they drop them. Worms aren't underground flavor — they're aerating soil, and the soil quality changes where they've been.

**Why this is a Big Yes:** Fauna is the connective tissue that turns isolated plants into an ecosystem. Without visible fauna, "interaction web" is just a spreadsheet the player can't see. With them, every creature is a story: "The bee came because of my wildflowers. It pollinated the daisy. The daisy spread. Now butterflies come to the daisies."

**Concrete execution:**
- **Pollinators (bees, butterflies):** Particle-sprites that travel between flowering plants. Leave a subtle golden trail. Plants they visit produce seeds faster. Appear when 3+ flowers exist within range. Distinct species attracted to different flowers.
- **Decomposers (worms, beetles):** Visible in underground view. Move through soil, leaving trails of enriched (darker) earth. Attracted to dead plant material. Accelerate nutrient cycling. A dead tree isn't failure — it's a future worm hotspot.
- **Dispersers (birds):** Silhouettes that visit berry-producing plants, then fly to another spot and "drop" a seed. The player can watch the flight path. The seed grows where the bird chose — not where the player planned.
- **Soil organisms (fungi, mycorrhizae):** Visible as networks connecting root systems underground. When established, they share nutrients between trees — the visual of two root systems connected by fungal threads is the "aha" moment for symbiosis.
- All fauna has arrival/departure conditions. They come when habitat is right, leave when it's disrupted. The player learns to *create habitat*, not summon creatures.

**Kill metric:** Can the player point at any creature and say "I know why it's here and what it's doing"? If yes, this is working.

### 3. Knowledge-as-Progression (The Outer Wilds Model)

**Stolen from:** Outer Wilds — where *what you know* is the only progression system. No XP, no unlocks, no tech tree. Understanding the ecology IS the upgrade.

The player's first garden is naive. Their tenth garden is expert. The difference isn't unlocked tools or species — it's that they *understand* how clover fixes nitrogen, how root spacing prevents competition, how water features attract pollinators, how canopy layers create light niches. The game gives you everything on day one. Mastery comes from understanding what you already have.

**Why this is a Big Yes:** This is what separates a 20-minute toy from a 200-hour game. If the player's tenth garden is better because they *know more*, they're on a discovery arc. If it's better because they unlocked a power-up, they're on a treadmill. Outer Wilds proved that knowledge-as-progression creates the deepest "one more try" hooks in gaming.

**Concrete execution:**
- All 12 species available from the start. No unlocks.
- All tools available from the start. No progression gates.
- The *discovery* is in species interactions: clover + oak = nitrogen boost. Wildflower + water = pollinators. Dense canopy = shade-loving fern thrives below. These aren't documented in-game — the player discovers them by experimenting.
- Optional: a "garden journal" that auto-records interactions the player has witnessed. Not a tutorial — a *field notebook*. "Observed: pollinators appeared near wildflower cluster at tick 45." The player's journal fills up as they discover more of the interaction web.
- The simulation loop measures this directly: does the LLM planner build better gardens over time using only knowledge gained from previous runs? If yes, the knowledge-as-progression curve exists.

**Kill metric:** Give two players the same tools. One has played for 10 hours, one is brand new. The experienced player's garden should be measurably more diverse, self-sustaining, and surprising — not because of unlocks, but because of understanding.

### 4. The Garden Composes Itself (Autonomous Beauty)

**Stolen from:** Animal Crossing's "the island changed while you were gone" + Conway's Game of Life's emergent complexity from simple rules.

The garden must develop beyond the player's explicit actions. Seeds disperse by wind, water, and birds. Dead material decomposes and feeds new growth. Pioneer species colonize bare ground. Canopy gaps fill with shade-tolerant species. Over time, the garden should look like something *no one designed* — because it designed itself. The player is a composer, not a constructor.

**Why this is a Big Yes:** This is the difference between a building game and a *living world* game. Minecraft lets you build. Groundwork lets you *start something* and then watch it become more than you imagined. The joy is the gap between what you planted and what grew.

**Concrete execution:**
- Wind dispersal: lightweight seeds (wildflower, grass, moss) spread 5-15 voxels downwind each season. Direction varies.
- Bird dispersal: berry-bush seeds carried to random spots where birds perch (near tall trees). New seedlings appear 20-50 voxels from the parent.
- Water dispersal: seeds near streams get carried downstream. Riparian corridors self-plant.
- Pioneer succession: bare soil → moss/grass → wildflowers → shrubs → trees. This happens automatically. A cleared area *will* become a garden if left alone. The player's role is to *guide* succession, not perform it.
- Decomposition: dead plants slowly convert to nutrient-rich soil. Fungi appear on dead wood. Worms accelerate the process. A stump isn't ugly — it's the beginning of something.
- The result after 500 ticks of a well-started garden: 30-50% of living plants should be ones the player didn't explicitly place.

**Kill metric:** After 500 ticks, does the garden contain plants the player didn't place? Do those plants look like they *belong*? Can the player trace how they got there?

### 5. Readable Interaction Chains (Factorio for Ecology)

**Stolen from:** Factorio's visible production chains — where you can follow a resource from mine to factory to output — but applied to ecology instead of industry.

Every ecological interaction must be *visible*. Not logged. Not statted. Visible. The player watches a bee leave a flower, travel to another flower, pollinate it, and sees the pollinated flower produce a seed. They watch water flow downhill, get absorbed by roots, and see the plant above grow a centimeter. The chain from cause to effect is a visual story, not a hidden calculation.

**Why this is a Big Yes:** Factorio's genius is that the production chain is the *content*. You don't read about iron becoming steel — you *watch* it happen. Same principle: watching a chain of ecological events unfold is the core gameplay. If chains are invisible, the game is just "plant stuff and wait."

**Concrete execution:**
- Pollination: visible pollen particles transfer from flower A to flower B via pollinator. Flower B then produces seeds visibly.
- Nutrient cycling: dead plant → decomposition animation → darker soil → nearby plant's growth rate visibly increases.
- Water chain: rain/spring → surface flow → soil absorption → root uptake → plant growth. Each step renders.
- Light chain: canopy blocks light (shadow renders on ground). Shade-tolerant plants thrive in shadow. Sun-loving plants struggle. The shadow IS the explanation.
- Competition: two root networks approach the same water source. One grows, one shrinks. The surface plants mirror this — the winning tree grows, the losing tree wilts.
- Each chain should have 3-6 visible links. If the player can narrate "A happened, which caused B, which caused C" from what they *saw*, the chain is readable.

**Kill metric:** Can the player explain *why* a specific plant is thriving or dying by pointing to visible things in the garden? No tooltips, no stats — just what they can see?

## THE BIG NOS

Equally important. Every "no" protects a "yes." These aren't dogma — progressive disclosure is good design, nudges help players learn, and UI can be beautiful. The point is: **don't fix a bad game with good tropes.** If the simulation doesn't teach through play, no amount of UI polish saves it. These nos exist to keep us honest about *where* the game's clarity comes from.

- **No stat overlays as a substitute for readable simulation.** Numbers replace observation with optimization. If the player needs a water % to know a plant is thirsty, the visual design has failed — fix the wilting animation, the soil color, the root view. But: gentle, contextual nudges (a subtle glow when hovering a struggling plant, a first-time hint about camera controls) are fine. The line is: UI should *point the player toward the simulation*, not replace it.
- **No unlock trees or progression gates.** All species and tools from minute one. The progression IS understanding. Gating content behind playtime is a treadmill, not discovery. (The moment you lock clover behind "reach level 5," you've killed the discovery arc.) Progressive *disclosure* of complexity is fine — showing all 12 species on a clean palette, letting the player naturally encounter advanced interactions. Progressive *gating* is not.
- **No fail states or game overs.** The garden can struggle, but it can't die permanently. Pioneer species always return. Water eventually recedes. Recovery is built into the simulation. The player should never feel punished for experimenting. (Animal Crossing doesn't have game overs. Neither does this.)
- **No tutorials as a crutch for unclear simulation.** If the player can't figure out what's happening by looking, the answer is better visual design — not a popup. But smart onboarding is great design: a gentle first-garden flow that introduces soil→water→seed in sequence, camera hints that invite underground exploration, contextual nudges when the game detects the player might be stuck. The test: would removing the nudge make the game *unplayable*, or just slightly slower to learn? If unplayable, the simulation itself needs work. If slightly slower, the nudge is earning its keep.
- **No optimization meta or "best build" solutions.** The interaction web must be rich enough that many different garden compositions thrive. If the community figures out one optimal layout, the interaction design is too shallow. Add more variance, more species-specific trade-offs, more emergent surprise.
- **No cosmetic-only features.** Every visual element should reflect simulation state. If a flower has a golden glow, it means pollinators visited it. If soil is darker, it means it's nutrient-rich. Beauty must always be *meaningful* beauty.
- **No systems that stand alone.** Every new feature must connect to at least two existing systems. A weather system that only affects water is too isolated. It should affect light, fauna behavior, seed dispersal, and soil. If a feature only connects to one thing, it's not ready.
- **No time pressure, deadlines, or urgency mechanics.** This is a place to breathe. Seasons change, but nothing expires. The player is never late. Cozy means you can put it down for a week and come back to a garden that grew, not one that withered.

## WHEN A WINNER SURFACES

The simulation loop will discover things that work spectacularly. When it does:

1. **Name it.** Give the winning mechanic or interaction a memorable name. ("The Nitrogen Handshake" for clover-oak symbiosis. "The Bird Express" for long-range seed dispersal.)
2. **Measure it.** Create a specific evaluator that tracks how often this moment occurs across runs.
3. **Amplify it 1000x.** Make it more visible, more reliable, more beautiful. Add particle effects. Add subtle sound. Make the camera notice it. If pollinators-connecting-flowers is a winner, make the pollen trail *gorgeous*. Make the arrival of the first bee a *moment*.
4. **Protect it.** Add a regression test. If a code change breaks this interaction, the build fails.
5. **Build adjacent.** What else connects to this winner? If clover-oak symbiosis works, what about fungal networks between tree roots? Same principle, new chain.
6. **Cut what competes.** If a feature draws attention away from a winner, or makes the winner less visible, cut the feature. Winners are rare. Protect them ruthlessly.

The loop's most important output isn't bug fixes or regressions — it's the discovery of what makes this game *magical*. A thousand runs should surface 3-5 interactions that reliably make the planner say "I didn't expect that, and I want to see it again." Those 3-5 interactions ARE the game. Everything else is in service of them.

---

## SESSION LOOP
1. **Player** reports experience from the current build.
2. **Manager** converts that into priorities, tasks, and open questions.
3. **Dev** executes against assigned tasks and reports outcomes, risks, and tradeoffs.
4. **Manager** updates priorities based on results.
5. **Player** validates the changed build again.

## DO NOT DRIFT
- Do not expand beyond MVP unless the Manager explicitly marks it as future-facing.
- Do not optimize for realism over readability.
- Do not add systems that hide cause-and-effect.
- Do not turn the game into survival, farming grind, crafting, or narrative adventure.
- Do not build systems where species grow in isolation — every new species or system should connect to the interaction web.
- Do not make the garden fully predictable — if the player can foresee every outcome, there's no reason to observe, experiment, or come back.
- If a proposal weakens "one more seed" curiosity or removes surprise, challenge it.
- Any new economy, multiplayer feature, narrative layer, or major content pipeline is automatically **P3**.
- Biome *implementation* (new species, art, interactions) is post-MVP work, but biome *architecture readiness* is MVP. Do not hardcode temperate assumptions into the sim.

## DEFAULT DECISION RULE
If uncertain, choose the option that:
1. makes ecological cause-and-effect more readable,
2. rewards player observation with surprise or discovery,
3. keeps the build smaller,
4. increases player delight sooner.

## SOURCE OF TRUTH
When sources disagree: game vision > Manager backlog > player feedback > build notes > older discussion.

---

## HANDOFF RULES
- Each handoff must distinguish observed facts from interpretation.
- Each handoff must separate bugs, design issues, and open questions.
- Each handoff must include only actionable items.
- If something cannot be verified, label it explicitly as a hypothesis.
- Keep outputs compact. No essays unless asked.

### Player → Manager
Sections: Observed | Felt | Bugs | Confusions | What made me want to keep playing | What made me want to stop | Requests

**Screenshots required:** Each key observation should include a screenshot (F2 or Snap button in web UI). Attach 1-10 screenshots per session to the PR — do not commit image files to the repo. Reference screenshots inline in feedback (e.g. "see screenshot: overgrown-oak.png").

### Manager → Dev
Sections: Goal | Why now | Tasks | Acceptance checks | Risks / constraints | Open questions

### Dev → Manager
Sections: Implemented | Not implemented | Tradeoffs made | Risks / regressions | Recommended next task | Build validation notes

### Manager → Player
Sections: What changed | What to pay attention to | Known rough edges | Specific questions for this session

---

## PRIORITY & SEVERITY

| Level | Meaning |
|-------|---------|
| **P0** | Blocks core proof or makes build unusable |
| **P1** | Strongly improves clarity, feel, or core loop |
| **P2** | Valuable but not required for current proof |
| **P3** | Future / expansion |

| Severity | Meaning |
|----------|---------|
| **Blocker** | Cannot continue session or core feature unusable |
| **Major** | Feature works incorrectly or misleads the player |
| **Minor** | Friction, polish, edge case, cosmetic |

## ISSUE TYPES
- **BUG** — behavior differs from intended behavior
- **DESIGN GAP** — behavior works but doesn't deliver intended experience
- **AMBIGUITY** — team lacks clarity to make a good decision
- **REGRESSION** — something that previously worked now fails or is worse
- **BOTTLENECK** — tooling/process/content issue slowing iteration

## TASK QUALITY BAR
A valid task must include: concrete player/prototype value, clear definition of done, scope for one focused session, dependencies (if any), and note if it's a prototype cheat or production direction.

## INTERFACE PARITY
The **web UI** is the primary player interface. Player-facing features must be playable in the browser. TUI/CLI continue as dev/debug tools but are not the reference experience. Agent play testers use the web UI via browser or webview — not the terminal CLI.

## DONE CRITERIA
Do not mark work done unless: it is testable or inspectable, expected outcome is stated, known compromises are listed, and follow-up risks are named.

## ESCALATION RULES
Escalate to Manager immediately when: two solutions conflict with the vision, a task expands in scope, player feedback conflicts with pitch goals, a missing tool blocks diagnosis, or a feature requires a new system not in MVP scope.

## FEEDBACK SANITY
- One strong repeated complaint beats five speculative ideas.
- A bug with a repro path beats a vague discomfort.
- A player misunderstanding is often a readability problem, not a player problem.

## PROTOTYPE SANITY
- Debug visibility over hidden correctness.
- Fake-but-readable over accurate-but-murky.
- One convincing bed over a broad shallow game.
- One visible interaction chain over many weak systems.
- One "I didn't expect that!" moment over ten predictable ones.
- Fauna that shows ecological role (bee pollinating, worm aerating) over fauna that's just decoration.
- A garden that surprises the player at tick 500 over a garden fully understood at tick 50.

## COZY BLOCKBUSTER PRINCIPLES
These define the *personality* of solutions when the loop reveals problems:
- **Generosity over punishment.** When players fail, make the garden more forgiving and recoverable — don't add tutorials. A garden that bounces back teaches better than one that dies.
- **Warmth over precision.** When interactions are hard to see, add warm visual feedback (golden particles for pollination, gentle sounds, cozy light shifts) — not data overlays or stat panels.
- **Invitation over instruction.** When players miss underground mechanics, add something beckoning from below (root glow, worm movement at the surface, a plant wiggling as roots spread) — not a "press C" prompt.
- **Abundance over scarcity.** When gardens feel empty, add more ambient life, particles, subtle animations — a world *teeming* with tiny activity. Not bigger plants or more species.
- **Surprise over predictability.** When experienced players achieve perfect scores, add more variance — rare fauna, weather events, lucky dispersals. The hundredth garden should still contain a moment you didn't expect.

---

## SHARED WORKSPACE

All agents coordinate through directories in the repo root. Use timestamped filenames: `{YYYY-MM-DDTHH:mm:ss}_{short_desc}.md`

| Directory | Owner | Purpose |
|-----------|-------|---------|
| `feedback/` | Player | Play session reports |
| `handoffs/player_to_manager/` | Player | Structured handoff to manager |
| `handoffs/manager_to_dev/` | Manager | Dev assignments |
| `handoffs/dev_to_manager/` | Dev | Implementation results |
| `handoffs/manager_to_player/` | Manager | Test directions for player |
| `backlog/current.md` | Manager | Prioritized work list |
| `decisions/` | Manager | Architectural and design decisions |
| `build_notes/` | Dev | Implementation details |
| `artifacts/` | Dev | Generated artifacts |

**Rules:** Read existing files before starting work. Do not overwrite files you don't own. Link to documents instead of duplicating content.

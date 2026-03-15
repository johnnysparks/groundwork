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

## EXPERIENTIAL NORTH STAR

The autonomous simulation loop exists to make the game feel like this for a new player:

- **Minute 1:** Place soil, water, a seed. It sprouts in golden light. Warm, clear, satisfying. You smile.
- **Minute 5:** Multiple species growing at different speeds. Tiny particles drift between wildflowers. What are those? Lean forward.
- **Minute 15:** Dip underground for the first time. Roots spreading like fingers. A worm trail. Back up top — a wildflower appeared where you didn't plant one. Where did it come from?
- **Minute 30:** Diagnose a struggling birch by looking at the oak's root network underground. Replant it with space. It thrives. You feel clever — you *observed* your way to understanding.
- **Hour 1:** Stop playing. Just watch. Light shifts. Pollinators move. A bird drops a seed. Your garden is making itself better without you. Pride and wonder.
- **Hour 3:** Third garden. You know clover-first for nitrogen, flowers-near-water for pollinators. But a moss carpet spreads under the oak canopy, ferns follow, and a shaded undergrowth micro-ecosystem forms that you didn't design. Beautiful.
- **Hour 10:** You design gardens like a landscape architect. Every plant has a role. But the garden still surprises you. You start your eleventh garden not because you haven't "won" but because you want to see what *this* combination will create.

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
- Any new biome, economy, multiplayer feature, narrative layer, or major content pipeline is automatically **P3**.

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

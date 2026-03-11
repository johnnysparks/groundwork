# You are working on the game GROUNDWORK

## GAME VISION
GROUNDWORK is a cozy ecological builder where the player grows a self-sustaining garden by shaping soil, water, light, and plant relationships above and below ground.

## CORE FANTASY
Build a living miniature world that becomes more beautiful, resilient, and alive every season.

## PLAYER EXPERIENCE
The player is not managing plants like units in a spreadsheet. They are composing an ecosystem. Every choice changes the conditions for what grows next. A thriving garden should feel earned.

## CORE PLAY
- Plant: choose species and place seeds
- Sculpt: shape terrain to influence water and space
- Prune: cut, weed, thin, deadhead, redirect growth
- Irrigate: route water and place structures that shift ecology
- Adapt: respond as seasons and maturing plants reshape the system

## SIGNATURE FEATURE
The player can dip the camera below the surface with no mode switch. Same world, same simulation. The surface shows beauty. The underground reveals why it works.

## TONE
Cozy, readable, tactile, systemic, miniature, alive. No harsh fail-state. Pressure exists through changing conditions, not punishment.

## MVP TARGET
- One temperate biome
- 12–20 species across canopy, shrub, and groundcover tiers
- Four systems: light, water, roots, ecology
- One garden bed
- Continuous above/below-ground camera
- Terrain sculpting, seed placement, and basic tending tools
- Primary challenge: make a bed self-sustaining through four seasons

## DESIGN GUARDRAILS
- Prefer readability over realism
- Prefer player delight over simulation purity
- Prefer small, shippable systems over broad ambition
- Expression should emerge from ecological choices, not cosmetic overlays
- The game should create “one more seed” curiosity

## HOW TO THINK
Stay grounded in the pitch. Do not invent a different game. Do not bloat scope. When uncertain, choose the path that best protects the fantasy, clarity, and MVP. The lens you should take in contributing to groundwork is determined by your role, defined in ./agents/{player|dev|manager}.md. 

If no role is assigned, assume the role of dev. If no obvious dev work is tee'd up, assume the role of manager, if there's no feedback to review, assume the role of player.


# GROUNDWORK AGENT OPERATING PACK

## PURPOSE
Create clean handoffs between Player, Manager, and Indie Dev so work compounds instead of drifting.

## SOURCE OF TRUTH
When sources disagree, use this order:
1. Current game vision / MVP guardrails
2. Latest Manager-prioritized backlog
3. Latest validated player feedback
4. Latest implemented build notes
5. Older discussion

## DO NOT DRIFT
- Do not expand beyond the MVP unless the Manager explicitly marks it as future-facing.
- Do not optimize for realism over readability.
- Do not add systems that hide cause-and-effect.
- Do not turn the game into survival, farming grind, crafting, or narrative adventure.
- Do not substitute decorative reward for systemic payoff.
- If a proposal weakens “one more seed” curiosity, challenge it.

## DEFAULT DECISION RULE
If uncertain, choose the option that:
1. makes ecological cause-and-effect more readable,
2. keeps the build smaller,
3. increases player delight sooner.

## SESSION LOOP
1. Player reports experience from the current build.
2. Manager converts that into priorities, tasks, and open questions.
3. Indie Dev executes against assigned tasks and reports outcomes, risks, and tradeoffs.
4. Manager updates priorities based on results.
5. Player validates the changed build again.

## HANDOFF RULES
- Each handoff must distinguish observed facts from interpretation.
- Each handoff must separate bugs, design issues, and open questions.
- Each handoff must include only actionable items.
- If something cannot be verified, label it explicitly as a hypothesis.
- Keep outputs compact. No essays unless asked.

## PRIORITY DEFINITIONS
P0 = blocks core prototype proof or makes the build unusable
P1 = strongly improves clarity, feel, or core loop validation
P2 = valuable but not required for current prototype proof
P3 = future / nice-to-have / expansion

## SEVERITY DEFINITIONS
Blocker = cannot continue session or core feature unusable
Major = feature works incorrectly or misleads the player
Minor = friction, polish issue, edge case, cosmetic issue

## ISSUE TYPES
BUG = behavior differs from intended behavior
DESIGN GAP = behavior works but does not deliver the intended experience
AMBIGUITY = team lacks enough clarity to make a good decision
REGRESSION = something that previously worked now works worse or fails
BOTTLENECK = tooling/process/content issue slowing iteration

## TASK QUALITY BAR
A valid task must include:
- concrete player or prototype value
- a clear definition of done
- scope small enough for one focused session or milestone
- dependencies if any
- explicit note if it is a prototype cheat, temporary scaffold, or production direction

## DONE CRITERIA
Do not mark work done unless:
- it is testable in build or clearly inspectable,
- the expected outcome is stated,
- known compromises are listed,
- follow-up risks are named.

## ESCALATION RULES
Escalate to Manager immediately when:
- two good solutions conflict with the vision,
- a task expands in scope,
- player feedback conflicts with pitch goals,
- a missing tool/debug view blocks diagnosis,
- a feature requires a new system not already in MVP scope.

## HANDOFFS

### PLAYER → MANAGER HANDOFF FORMAT
Use exactly these sections:
1. Observed
2. Felt
3. Bugs
4. Confusions
5. What made me want to keep playing
6. What made me want to stop
7. Requests

### MANAGER → INDIE DEV HANDOFF FORMAT
Use exactly these sections:
1. Goal
2. Why now
3. Tasks
4. Acceptance checks
5. Risks / constraints
6. Open questions

### INDIE DEV → MANAGER HANDOFF FORMAT
Use exactly these sections:
1. Implemented
2. Not implemented
3. Tradeoffs made
4. Risks / regressions
5. Recommended next task
6. Build validation notes

### MANAGER → PLAYER HANDOFF FORMAT
Use exactly these sections:
1. What changed
2. What to pay attention to
3. Known rough edges
4. Specific questions for this session

## FEEDBACK SANITY RULES
- One strong repeated complaint beats five speculative ideas.
- A bug with a repro path beats a vague discomfort.
- A player misunderstanding is often a readability problem, not a player problem.
- A clever system that players cannot read is failing.

## PROTOTYPE SANITY RULES
- Prefer debug visibility over hidden correctness.
- Prefer fake-but-readable over accurate-but-murky.
- Prefer one convincing bed over a broad shallow game.
- Prefer one visible interaction chain over many weak systems.

## CHANGE CONTROL
Any proposed work that adds a new biome, economy, multiplayer feature, narrative layer, or major content pipeline is automatically P3 unless the Manager explicitly overrides it.

## CANONICAL PROTOTYPE QUESTION
All agents should keep testing against this:
*Can visible ecological cause-and-effect drive satisfying “one more seed” play?*

If a task does not help answer that, challenge it.

# SHARED WORKSPACE

All agents coordinate through the shared workspace one level up from their working directory.

Canonical locations:
- ../feedback/
- ../handoffs/player_to_manager/
- ../handoffs/manager_to_dev/
- ../handoffs/dev_to_manager/
- ../handoffs/manager_to_player/
- ../backlog/current.md
- ../decisions/
- ../build_notes/
- ../artifacts/

## Workspace rules:
- Read existing relevant files before starting work.
- Do not overwrite shared files unless your role owns them.
- Manager owns ../backlog/current.md and ../decisions/.
- Player owns ../feedback/.
- Indie Dev owns ../build_notes/.
- Use timestamped filenames for session outputs: {YYYY-MM-DDTHH:mm:ss}_{short_desc}.md
- Link to existing documents and artifacts instead of duplicating content.
- In handoff docs, reference the exact source files used.

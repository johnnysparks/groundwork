# GROUNDWORK AGENT OPERATING PACK

> **New session?** Start in `CLAUDE.md` — it has your role-specific checklist.
> This file is the reference manual. You don't need to memorize it; come back when you need a format or rule.

## GAME VISION
GROUNDWORK is a cozy ecological builder where the player grows a self-sustaining garden by shaping soil, water, light, and plant relationships above and below ground.

**Core fantasy:** Build a living miniature world that becomes more beautiful, resilient, and alive every season.

**Player experience:** The player is composing an ecosystem, not managing units in a spreadsheet. Every choice changes the conditions for what grows next. A thriving garden should feel earned.

## CORE PLAY
- **Plant:** choose species and place seeds
- **Sculpt:** shape terrain to influence water and space
- **Prune:** cut, weed, thin, deadhead, redirect growth
- **Irrigate:** route water and place structures that shift ecology
- **Adapt:** respond as seasons and maturing plants reshape the system

## SIGNATURE FEATURE
The player can dip the camera below the surface with no mode switch. Same world, same simulation. The surface shows beauty. The underground reveals why it works.

## MVP TARGET
- One temperate biome, 12-20 species (canopy/shrub/groundcover)
- Four systems: light, water, roots, ecology
- One garden bed (~60x60x30 voxels)
- Continuous above/below-ground camera
- Terrain sculpting, seed placement, basic tending tools

## DESIGN GUARDRAILS
- Readability over realism
- Player delight over simulation purity
- Small, shippable systems over broad ambition
- Expression from ecological choices, not cosmetic overlays
- Create "one more seed" curiosity

## CANONICAL QUESTION
All agents should keep testing against this:
> *Can visible ecological cause-and-effect drive satisfying "one more seed" play?*
If a task does not help answer that, challenge it.

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
- If a proposal weakens "one more seed" curiosity, challenge it.
- Any new biome, economy, multiplayer feature, narrative layer, or major content pipeline is automatically **P3**.

## DEFAULT DECISION RULE
If uncertain, choose the option that:
1. makes ecological cause-and-effect more readable,
2. keeps the build smaller,
3. increases player delight sooner.

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

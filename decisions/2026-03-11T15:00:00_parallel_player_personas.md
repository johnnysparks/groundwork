# Decision: Parallel Player Persona Sessions

**Date:** 2026-03-11T15:00:00
**Author:** Manager
**Status:** Active

## Context

We've had four player feedback sessions so far, all from generalist testers. The feedback has been useful but convergent — every session says "water is fun, nothing grows, roots are inert." Now that seed growth is shipped, we need **breadth** of feedback, not depth. Different playstyles will stress different parts of the build and surface different issues.

## Decision

Run 5 simultaneous player sessions, each with a distinct persona and directed focus area. Each persona targets a different axis of the game that needs validation.

## Personas

### 1. The Optimizer ("Irrigation Engineer")
- **Focus:** Efficiency, throughput, min/max behavior
- **Tests:** Water routing efficiency, seed growth rate optimization, resource coverage
- **Surfaces:** Balance issues, exploits, degenerate strategies, missing feedback on "how well am I doing"

### 2. The Underground Explorer ("Spelunker")
- **Focus:** Vertical play, underground spaces, depth mechanics
- **Tests:** Shaft digging, underground gardens, light-through-shafts, cave ecosystems
- **Surfaces:** Underground readability gaps, light system edge cases, vertical interaction bugs

### 3. The Aesthetic Builder ("Garden Designer")
- **Focus:** Beauty, composition, visual satisfaction
- **Tests:** Deliberate garden layouts, visual balance, pattern creation, mixed materials
- **Surfaces:** Visual polish gaps, readability issues, missing aesthetic feedback, ASCII art limitations

### 4. The Scientist ("Ecologist")
- **Focus:** Understanding systems, measuring, testing hypotheses
- **Tests:** Controlled experiments (isolate variables), edge cases, threshold hunting, system interactions
- **Surfaces:** Undocumented behaviors, inconsistencies, missing inspect info, system coupling bugs

### 5. The Casual Player ("Weekend Gardener")
- **Focus:** Low effort, immediate gratification, minimal CLI friction
- **Tests:** Can a player who doesn't read docs have fun? Shortest path to a growing seed.
- **Surfaces:** Onboarding gaps, unclear defaults, missing guidance, friction that power users tolerate but casuals won't

## Why These Five

| Persona | Primary axis tested | Backlog items stressed |
|---|---|---|
| Optimizer | Balance / throughput | SIM-03, seed growth tuning |
| Spelunker | Vertical / underground | VIS-01, light system, underground readability |
| Garden Designer | Aesthetics / layout | SIM-04, VIS-02, GAME-02, GAME-04 |
| Ecologist | System correctness | SIM-03, SIM-04, SIM-05, all edge cases |
| Weekend Gardener | Onboarding / accessibility | GAME-03, CLI UX, missing help text |

## Expected Outcome

5 feedback reports covering distinct surface area. Manager synthesizes into prioritized backlog update. Dev gets targeted, non-redundant work items.

## Risk

Personas may drift toward the same observations (seed growth is new — everyone will react to it). Mitigated by giving each persona specific directed questions and a constrained focus area.

# Dev Loop Prompt Pack

Reusable prompts to run periodically so contributors can ship faster with less context loading.

## How to use this pack
- Copy one prompt into your coding agent.
- Replace placeholders in `<angle brackets>`.
- Keep runs short: prefer targeted checks first, then broaden.
- If something can’t be verified, require the agent to label it as a hypothesis.

---

## 1) Code Maintenance Sweep (anti-entropy)

```text
You are doing a maintenance sweep in this repository.

Goal: reduce entropy and keep iteration speed high without expanding product scope.

Focus only on:
1) stale or duplicated docs,
2) dead scripts/tasks/commands,
3) broken or misleading onboarding steps,
4) outdated references after recent changes.

Process:
- Find the top 3 maintenance issues with the highest friction-to-fix ratio.
- For each issue, provide: observed fact, impact, smallest safe fix.
- Implement only low-risk fixes that can be completed in one session.
- Prefer delete/simplify over adding new structure.
- Do not change runtime behavior unless required for correctness.

Validation:
- Run the fastest relevant checks first (<project quick checks>).
- Then run broader checks only if touched files justify it.
- Report exactly what was validated vs what remains unverified.

Output format:
- Implemented
- Not implemented
- Tradeoffs
- Risks/regressions
- Follow-up tasks
```

---

## 2) Agent Speedup Pass (time-to-first-change)

```text
Optimize this codebase for agent and new-developer speed.

Success metric: reduce time from clone to first confident change.

Tasks:
- Identify the minimal "happy path" workflow (setup, run, test, edit, validate).
- Remove or fix bottlenecks that slow first contribution.
- Add or improve one discoverable entrypoint for common commands (e.g., Makefile/task runner/script index).
- Keep command names short, obvious, and composable.
- Ensure commands fail loudly with actionable error messages.

Constraints:
- Keep changes small and reversible.
- No broad architecture changes.
- No speculative tooling that isn’t used immediately.

Validation:
- Perform a cold-path simulation: follow docs/commands as if you were new.
- Measure rough elapsed time for each step and report bottlenecks.
- Verify every documented command you touched actually runs.

Output format:
- Bottlenecks found
- Changes made
- New fast-path commands
- Measured before/after loop time
- Remaining friction
```

---

## 3) Dev Experience Tightening (fast feedback)

```text
Improve developer experience for fast, reliable local feedback.

Priorities:
1) single-command local validation,
2) deterministic test behavior,
3) clear failure output,
4) minimal waiting.

Do:
- Audit existing test/check commands.
- Create or refine a tiered workflow:
  - quick: <very fast checks>
  - standard: <normal pre-commit checks>
  - full: <slower exhaustive checks>
- Ensure docs explain when to run each tier.
- Remove redundant checks and duplicate command definitions.

Don’t:
- Add heavyweight dependencies unless essential.
- Introduce parallel workflows that drift.

Validation:
- Execute quick + standard tiers.
- Confirm command outputs are understandable to a newcomer.
- Report expected runtime and actual runtime.

Output format:
- DX gaps
- Command tier design
- Changes made
- Validation results
- Recommended next DX task
```

---

## 4) Code Quality Guardrail Pass (small, high leverage)

```text
Run a focused code quality pass for high-signal improvements.

Scope:
- touched area: <module/path>
- max edits: <N files>
- max session time: <X minutes>

Priorities:
- readability over cleverness,
- remove footguns,
- improve naming and invariants,
- strengthen tests around risky behavior.

Process:
- Identify 2-4 concrete quality issues with repro or code evidence.
- Fix only what you can validate in-session.
- For each fix, explain why this is better for future maintainers.
- If a bigger refactor is needed, write a follow-up task instead of partial risky changes.

Validation:
- Run targeted tests for touched modules first.
- Run lint/format/check commands required by repo standards.
- Include any residual risk explicitly.

Output format:
- Quality issues found
- Fixes implemented
- Tests/checks run
- Residual risks
- Follow-up refactors (scoped)
```

---

## 5) Pleasantness Pass (developer joy, less grind)

```text
Do a "pleasantness pass" on everyday development workflows.

Definition of pleasantness:
- easy to discover what to run,
- easy to recover from mistakes,
- easy to understand system state,
- low cognitive overhead for routine tasks.

Find and fix small frictions such as:
- confusing command names,
- hidden prerequisites,
- noisy or non-actionable errors,
- docs that assume too much context,
- repetitive manual steps that can be scripted.

Guidelines:
- Prefer tiny ergonomics wins that compound.
- Keep behavior explicit; avoid magic.
- Add short "why" notes where future contributors might hesitate.

Validation:
- Run updated workflow exactly as documented.
- Verify no dead docs: every command/path/reference you changed is valid.

Output format:
- Frictions found
- Improvements shipped
- Workflow before/after
- Validation proof
- Optional nice-to-have next improvements
```

---

## 6) No-Dead-Docs Verifier Prompt

```text
Audit docs for dead or misleading references.

Check:
- command examples,
- file paths,
- script names,
- task runner targets,
- referenced tests,
- setup instructions.

For each broken item:
- show source location,
- show failure evidence,
- apply smallest fix,
- re-run to confirm.

Constraints:
- no speculative rewrites,
- keep wording concise and concrete,
- preserve intent while fixing accuracy.

Deliver:
- Dead docs fixed
- Dead docs intentionally deferred (with reason)
- Commands used for verification
```

---

## 7) Weekly Cadence Prompt (batch upkeep)

```text
Run a weekly upkeep batch focused on dev loop speed.

Goals for this run:
- one maintenance cleanup,
- one DX improvement,
- one test/validation speed win,
- one docs accuracy fix.

Rules:
- keep each change independently reviewable,
- prefer safe deletions/simplifications,
- avoid feature work.

Required checks:
- <quick checks>
- <standard checks>

Return:
- 4 shipped improvements (one per goal)
- evidence of faster or clearer workflow
- explicit list of what was not covered this week
```

---

## Suggested placeholders for this repository

Use these when filling placeholders in prompts:
- `<project quick checks>`: `cargo fmt --all -- --check && cargo check -p groundwork-sim -p groundwork-tui`
- `<very fast checks>`: `cargo check -p groundwork-sim`
- `<normal pre-commit checks>`: `cargo test -p groundwork-sim && cargo test -p groundwork-tui`
- `<slower exhaustive checks>`: `cargo check --workspace && cargo test --workspace`


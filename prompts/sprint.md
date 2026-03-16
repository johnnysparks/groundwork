# Sprint Session

Run a full development cycle: sync, prioritize, build, ship, playtest. Repeat until the session ends or I say stop. If you are given a "theme" focus on that are of development and testing work in your backlog. Remind each of the roles what the core theme for them to focus on is. This allows parallelized workstreams.

## The Loop

### 1. SYNC
- `git pull --rebase` to pick up upstream changes
- Check for open PRs: `gh pr list`
- Scan `feedback/` and `handoffs/` for new or changed files — this is where feedback from previous sessions and other async contributors arrives
- If there are merge conflicts or git issues, resolve them before continuing

### 2. MANAGE
- Read `backlog/current-{theme}.md` and any new `feedback/` or `handoffs/` files that arrived in SYNC
- Triage: verify bug claims against code, archive stale feedback, annotate fixes
- Update `backlog/current-{theme}.md` if priorities shifted
- Write or update `handoffs/manager_to_dev/` with the top task
- Keep it sharp — the dev assignment should be unambiguous

### 3. DEV
- Read the manager handoff and execute the top P0/P1 task
- Follow the visual verification loop: code → `./screenshot.sh --quick` → view → iterate
- Run `cargo test -p groundwork-sim` for sim changes
- When done, capture the full screenshot tour and verify

### 4. SHIP
- Commit with descriptive message (use `--no-verify`)
- Push to origin/main (`--no-verify`)
- Write build notes to `build_notes/` and dev→manager handoff to `handoffs/dev_to_manager/`

### 5. PLAYTEST
- Rebuild WASM if sim changed: `cd crates/groundwork-web && npm run wasm`
- Capture screenshots: `./screenshot.sh`
- **View every screenshot** and evaluate what just shipped against the game vision
- Write playtest feedback to `feedback/` — what improved? what's still missing? what broke?
- Focus on: Does it feel alive? Are interactions visible? Is there surprise? Does x-ray reveal something interesting underground?
- Go back to step 1

## Rules
- Each cycle should produce a visible improvement verifiable in screenshots
- Don't spend more than one cycle on a single task — if it's too big, split it
- Feedback from SYNC (other sessions, async contributors) gets incorporated in the MANAGE step
- Feedback from PLAYTEST (your own eyes on what just shipped) drives the next cycle's priorities
- The game vision in CLAUDE.md is the source of truth for prioritization
- When in doubt: make the garden more alive, not more polished

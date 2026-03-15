# Sprint Session

Run a full development cycle: sync, playtest, prioritize, build, ship. Repeat until the session ends or I say stop.

## The Loop

### 1. SYNC
- `git pull --rebase` to pick up async feedback and upstream changes
- Check for open PRs: `gh pr list`
- Scan `feedback/` and `handoffs/` for new files since last session
- If there are merge conflicts or git issues, resolve them before continuing

### 2. PLAYTEST
- Rebuild WASM if sim changed: `cd crates/groundwork-web && npm run wasm`
- Capture screenshots: `./screenshot.sh`
- **View every screenshot** and evaluate the current state against the game vision
- Write brief playtest notes to `feedback/` if anything changed since last cycle
- Focus on: Does it feel alive? Are interactions visible? Is there surprise? Does x-ray reveal something interesting underground?

### 3. MANAGE
- Read `backlog/current.md` and latest `feedback/` files
- Triage: verify bug claims against code, archive stale feedback, annotate fixes
- Update `backlog/current.md` if priorities shifted
- Write or update `handoffs/manager_to_dev/` with the top task
- Keep it sharp — the dev assignment should be unambiguous

### 4. DEV
- Read the manager handoff and execute the top P0/P1 task
- Follow the visual verification loop: code → `./screenshot.sh --quick` → view → iterate
- Run `cargo test -p groundwork-sim` for sim changes
- When done, capture the full screenshot tour and verify

### 5. SHIP
- Commit with descriptive message (use `--no-verify`)
- Push to origin/main (`--no-verify`)
- Write build notes to `build_notes/` and dev→manager handoff to `handoffs/dev_to_manager/`
- Go back to step 1

## Rules
- Each cycle should produce a visible improvement verifiable in screenshots
- Don't spend more than one cycle on a single task — if it's too big, split it
- If async feedback arrived during the cycle (visible after git pull), incorporate it in the MANAGE step
- The game vision in CLAUDE.md is the source of truth for prioritization
- When in doubt: make the garden more alive, not more polished

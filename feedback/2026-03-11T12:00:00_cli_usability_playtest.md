---
role: player
date: 2026-03-11
build: groundwork-tui @ commit HEAD
focus: CLI usability for agent play
session_length: ~30 commands
---

# Player Feedback: CLI Usability Playtest

## 1. What the game sold me

The pitch is a cozy garden builder where I shape soil, water, and light to build a self-sustaining ecosystem. The CLI promises I can play non-interactively — create a world, place stuff, tick, observe. That's exactly what an agent needs.

## 2. What I actually experienced

I created a world, viewed slices, placed water, ticked forward, and watched water spread. The water flow system is working — after 100 ticks I could see a clear wet-soil ring spreading outward from the spring. That's satisfying. But I hit several friction points that slow down the play loop and make it harder to understand what's happening.

## 3. Best moments

- First `view --z 16` after 20 ticks: seeing the water spread pattern from the spring was genuinely cool. The radial diffusion pattern is visible and organic-looking.
- `view --z 15` after 100 ticks: the wet soil ring `%` spreading through the `#` soil showed cause-and-effect clearly.
- Error messages for bad materials ("unknown material: dirt. Valid: air, soil, stone, water, root") are helpful and immediately actionable.

## 4. Confusing moments

- **No legend on view output.** I had to look up what `.`, `~`, `#`, `%`, `@`, `*` mean every time. The view command should print the legend.
- **No axis labels on the grid.** I place something at (20, 20, 16) and then look at the view — but I can't tell which cell is (20, 20) without counting dots. Even sparse tick marks (every 10 cells) would help.
- **`view --z 50` silently shows empty grid.** Z only goes 0-29 but the command happily renders an out-of-bounds slice with no warning. I thought my world was empty until I realized I was looking above the sky.
- **No coordinate feedback in view header.** The header says `Z:16 (above +1)` but doesn't tell me the X/Y range I'm looking at.
- **`inspect` output is minimal.** It shows raw numbers (water_level: 115) but doesn't tell me what they mean relative to the simulation. Is 115 a lot? What's the max? A simple bar or fraction would help.
- **Running with no arguments panics** instead of showing help. If there's no terminal (agent context), `groundwork` with no args crashes with a stack trace about "failed to initialize terminal."

## 5. Boring or frustrating moments

- **`tick` gives almost no feedback.** Just `Tick: 121 (+100)`. I have no idea what happened during those 100 ticks. Did water spread? Did soil get wetter? Even a one-line diff summary ("water spread to 48 new cells") would make ticking feel like something happened.
- **`status` doesn't count wet soil.** It shows material counts but wet soil (`%`) is a key visual indicator and there's no way to see how much soil is wet without manually scanning the view.
- **Every command requires `cargo run -p groundwork-tui --`.** The player instructions show this long prefix for every command. This is a cargo build concern, not a CLI bug, but it adds friction for agents.
- **`view` dumps 60 lines of identical characters** for uniform layers (all `#` soil, all `@` stone). No visual interest, no information beyond "this layer is all the same material."

## 6. Bugs

### BUG-1: No-args invocation panics without terminal
- **Severity:** major
- **Steps:** Run `groundwork` with no arguments in a non-terminal context (e.g., agent sandbox)
- **Expected:** Show help text or a friendly error
- **Actual:** Panic with stack trace: "failed to initialize terminal"
- **Frequency:** 100%
- **Notes:** The default command is `tui` which requires a real terminal. Should detect non-TTY and fall back to help.

### BUG-2: `view --z` accepts out-of-bounds Z without warning
- **Severity:** minor
- **Steps:** `groundwork view --z 50` (grid only goes to Z=29)
- **Expected:** Warning that Z=50 is out of bounds, or clamping to valid range
- **Actual:** Silently renders an empty grid with header "Z:50 (above +35)"
- **Frequency:** 100%

## 7. Feature or clarity requests

1. **Add ASCII legend to `view` output** — print `. air  ~ water  # soil  % wet soil  @ stone  * root` below the grid
2. **Add axis labels to `view`** — X coordinates across top (every 10), Y coordinates down left side (every 10)
3. **Warn or clamp out-of-bounds Z in `view`**
4. **Fall back to `help` when no terminal is available** instead of panicking
5. **Show a change summary after `tick`** — e.g., material count deltas or "water spread to N new cells"
6. **Add water/wet soil counts to `status`** — total water volume and number of wet-soil cells
7. **Show value ranges in `inspect`** — e.g., "water_level: 115/255" so players understand the scale
8. **Add `--legend` or always-on legend for `view`** so players don't need to memorize symbols

## 8. Brutal bottom line: would I come back tomorrow?

**3/5 — Maybe, with reservations.** The simulation itself is working and the water spreading is genuinely interesting to watch. But the CLI is a bare-minimum interface right now. As an agent player, I spend too much mental effort on: remembering what symbols mean, guessing whether my actions did anything, and figuring out coordinates. The play loop works but doesn't feel smooth. Fix the legend, add axis markers, make `tick` show what changed, and fix the no-terminal crash — then the CLI becomes a real play surface instead of a debugging tool.

### Evaluation Lenses

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 3 | `new` + `view` works, water spring is visible, but no legend = confusion |
| Clarity of cause and effect | 3 | Water spreading is visible after many ticks, but `tick` gives zero feedback |
| Tactile satisfaction | 2 | `place` + `tick` + `view` loop works but feels disconnected — no confirmation of change |
| Beauty/readability | 2 | Raw ASCII grid with no labels or legend is functional but not readable |
| Ecological fantasy delivery | 2 | Water flows, soil gets wet — but no plants, no growth, no ecology yet |
| Desire to keep playing | 3 | I want to see what happens with roots and plants, but current tools make experimenting tedious |
| Friction/confusion | 2 | Multiple usability gaps (no legend, no labels, silent OOB, panic on no-terminal) |
| Trust in the simulation | 4 | Water spread pattern looks organic and believable. Inspect confirms values are changing. |

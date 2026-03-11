# Terraformer Play Session

**Date:** 2026-03-11T18:00:00
**Persona:** Landscape sculptor — terrain shape first, plants second
**Build:** Post-Sprint 3 (placement validation, growth visibility, batch placement)
**Duration:** ~200 ticks across one session
**State file:** terraformer.state

## 1. What the game sold me

Build terrain, then watch how your sculpted landscape shapes where life grows. The `fill` command is your brush. You're composing a canvas.

## 2. What I actually experienced

**Sculpting phase:** Genuinely creative. I built a sunken bowl (carved 3 levels deep), a raised 3-tier plateau with stone retaining walls, a water channel connecting the spring to the bowl, and a terraced step in the middle. This took ~14 `fill` commands. The ASCII output at multiple Z-levels let me see my creation in cross-section, which was surprisingly satisfying.

**Planting phase:** Placed seeds at strategic locations — bowl rim, channel edges, plateau top, terrace row. The seed protection system caught my one accidental overlap (tried to place at a coordinate already seeded). Clean error.

**Growth phase:** Water flowed down the channel into the bowl. Seeds near water grew. The channel-side seeds all sprouted into roots in a neat line. Bowl rim seeds near the channel grew. Plateau seeds stayed dormant until I placed a separate water source up there — then they all grew within 50 ticks. The terrace seeds at Z=16 never grew because water flows past them, not through them.

**The emergent story:** My terrain *did* shape where things grew. The channel became a spine of roots. The bowl collected water. The plateau was a desert until irrigated. This is the core loop working.

## 3. Best moments

- Viewing Z=16 after sculpting and seeing the stone `@` retaining walls framing the plateau like a real structure. That was a "I made this" moment.
- The wet soil `%` diamond spreading from the spring at Z=15 across 200 ticks — watching it grow was like watching an ink blot. Beautiful emergent pattern.
- The moment at tick 50 when I realized my channel seeds had all converted to `*` roots in a line — the water path became a root path. That's ecological cause-and-effect landing.
- At tick 200, the plateau top at Z=18 showed all three rows of seeds converted to roots, with wet soil borders and water fringe. A functioning garden on top of a mesa I built from nothing.

## 4. Confusing moments

- **Coordinate axis confusion:** When using `fill`, I had to think carefully about whether X or Y was horizontal vs vertical in the ASCII view. The first few commands required trial-and-error. The view shows X left-right, Y top-bottom, but when typing `fill soil x1 y1 z1 x2 y2 z2` I had to mentally map "the first number is columns, second is rows."
- **Water doesn't soak into adjacent soil from channels:** I dug a channel at Z=15 (air trench in soil) and put water in it. The water *flowed down* the channel but the adjacent soil walls didn't get wet the way I expected. Water only spreads to soil below, not laterally into soil walls. This broke my "irrigation channel" mental model.
- **No feedback on what I can't do:** I wanted to build a dam or wall out of stone to redirect water, but I had no way to know in advance whether stone blocks water. I just had to try and observe.

## 5. Boring or frustrating moments

- **Terrace seeds never grew:** I placed 8 seeds on a soil terrace at Z=16 and they stayed dormant for 200 ticks. The terrace was surrounded by soil, but water flowed under it (at Z=15) not through it. The seed diagnostics explained why (`no water nearby`), which was helpful, but the solution isn't obvious. Do I need to place water *on* the terrace? That feels wrong for a soil platform.
- **Flat starting terrain is dull:** Before I started sculpting, `view --z 15` showed 60 rows of `#`. No motivation to engage. The game should at least hint at features.
- **Checkerboard water is distracting:** At Z=16 the water spread shows `.~.~.~` alternating pattern. At Z=18 on the plateau it's especially visible. It made me question whether the water was really flowing or glitching.
- **No undo:** After filling a region wrong, there's no way to undo. I had to mentally track what I'd changed and manually `fill` it back. For a terrain sculptor this is painful.

## 6. Bugs

### Bug 1: Checkerboard water artifact
- **Severity:** Major
- **Steps:** Place water source, tick 100+, view Z=16
- **Expected:** Smooth water frontier
- **Actual:** `.~.~.~` alternating wet/dry pattern
- **Frequency:** 100% — every water spread shows it
- **Notes:** Confirmed on both the spring water and my plateau water source. Same pattern at different Z levels.

### Bug 2: Seed placement error message is unhelpful for ranges
- **Severity:** Minor
- **Steps:** `place seed 5 5..25 15` when (5,5,15) is already a seed
- **Expected:** Tells me which coordinates were skipped and which succeeded
- **Actual:** Error on first collision, unclear if remaining coordinates were placed
- **Frequency:** Once
- **Notes:** When using ranges, a partial-success report would be better than error-on-first-collision.

## 7. Feature or clarity requests

### Needs (affect core loop)
- **Water lateral absorption into soil:** Channels don't irrigate adjacent soil walls. This is the #1 gap for the terraformer playstyle. Without it, terrain sculpting is cosmetic — water just falls, it doesn't interact with the terrain I built.
- **Undo/history for fill:** Even a single undo would transform the sculpting experience. Ctrl-Z is muscle memory.
- **Material tooltip/info command:** `help materials` or `info stone` — does stone block water? Light? Can seeds grow on it? I have to experiment for every material, which is slow.

### Wishes (would enhance)
- **Copy/paste regions:** After building a terrace design I liked, I wanted to duplicate it elsewhere.
- **Named waypoints or bookmarks:** Z-levels have no meaning. I'd like to name them: "bowl floor = Z13", "surface = Z15", "plateau top = Z18".
- **Slope/gradient fill:** `fill soil 10 10 15 20 10 17` could auto-slope instead of filling a solid rectangle.
- **Preview before fill:** `fill --dry-run soil 5 5 15 20 25 15` to show what would change without committing.

## Evaluation Scores (1-5)

- **First-impression hook:** 2/5 — Flat `#` wall at Z=15 is lifeless. I had to know what I wanted to build before starting. No affordances.
- **Clarity of cause and effect:** 4/5 — Water flows visibly, seed diagnostics explain dormancy, growth progress is clear (s→S→*). Docked for lack of material property info.
- **Tactile satisfaction:** 4/5 — `fill` feels powerful. Viewing multiple Z-levels to see my structure in 3D cross-section is genuinely fun. Docked for no undo.
- **Beauty/readability:** 3/5 — Stone walls and wet soil patterns look good. Checkerboard water is ugly. Underground air is indistinguishable from surface air.
- **Ecological fantasy delivery:** 3/5 — Water shapes growth along channels. But water doesn't interact with terrain laterally, roots are inert, and there's no resource pressure. Fantasy is "almost there."
- **Desire to keep playing:** 3/5 — I want to see what happens with lateral water interaction. Right now I've explored what I can explore.
- **Friction / confusion:** 3/5 — Coordinate mapping is tricky. No undo. No material info. But `fill` and `inspect` work well.
- **Trust in the simulation:** 3/5 — Seed growth is deterministic and trustworthy. Checkerboard water undermines trust. Water-only-flows-down feels incomplete.

## 8. Brutal bottom line: would I come back tomorrow?

**Yes, but barely.** The sculpting is fun enough to draw me in. Seeing my terrain shape growth is the hook. But without lateral water absorption, my channels are decorative and my terraces are dead. The moment I realize "terrain doesn't actually affect water flow meaningfully," the fantasy collapses. Fix water-soil interaction and I'd play for hours. Without it, I'm just carving ASCII art.

**Score: 3.5/5 — creative potential visible, ecological depth not yet there.**

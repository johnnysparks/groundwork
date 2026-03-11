# Player Feedback — Underground Gardening Session

**Date:** 2026-03-11T14:00:00
**Build:** Tick 268, post-CLI-usability sprint
**Session length:** ~1 hour (CLI non-interactive)

---

## 1. What the game sold me

A living miniature world. Shape soil, water, light, roots above and below ground. Something self-sustaining grows from what I build.

---

## 2. What I actually experienced

Water flows from a spring and slowly wets the surface. I dug an underground shaft and routed water down into a cave. Light followed down the shaft. That moment — a lit, wet underground pocket — was genuinely cool. But then: nothing happened. Two roots sit on the surface doing nothing. No seeds. No growth. The cave is a neat space with nowhere to go.

The diagonal stripe pattern in the water frontier is visually distracting and noticeable by tick 150+. The frontier has clear `.~.~.~` alternating artifacts where the wave should be smooth.

---

## 3. Best moments

**The shaft experiment.** I dug air from Z15 down to Z12, added water at the top, ticked 5 times, and water fell into the underground cave. Then I checked light and it followed the shaft down (light_level ~208 at Z12, matching shaft geometry). This is the first time the vertical world felt real — underground and above-ground connected by deliberate engineering. I wanted to plant something down there immediately.

**Reading the wet soil spread.** At Z15, watching `%` tiles spread outward from the center as water seeped down was genuinely satisfying. The ring of wet soil expanding tick by tick delivers on cause-and-effect.

**Stone dam.** Placed stone at (30,30,16) — water routed around it. The simulation respected it instantly. Trust-building moment.

---

## 4. Confusing moments

**No feedback on roots.** I placed two roots in a wet, lit area. After 10 ticks, nothing changed. Root water_level stayed at 0. The root just sits there. Is it alive? Dead? Decorative? I have no idea what a root is supposed to DO right now. The legend says `* root` but gives no hint it's a thing you build toward, not a thing you place.

**`below -15` label at Z=0.** The depth label says "below -15" at Z=0 which is a stone floor that's clearly the bottom of the world. "bedrock" or "Z:0 (bottom)" would be clearer. "below -15" implies there's more below.

**Water quantity isn't visible.** At the surface (Z15), I can see `%` vs `#` but I don't know how wet. Is water_level 40 enough for roots? 255? The view gives a binary signal (dry / wet) when the underlying data is richer. I'm flying blind on "is this wet enough."

**Light in air shaft reads zero before tick update.** I dug the shaft, then immediately inspected Z15 of the shaft — light_level: 0. Had to run `tick 1` to see light propagate. The "light only updates on tick" mechanic isn't explained anywhere, so the initial zero reading felt like a bug.

---

## 5. Boring or frustrating moments

**Placing voxels one at a time.** Digging the 11-cell tunnel at Z12 required 11 separate commands. It took longer than the interesting part. This is the most painful UX in the game right now. Even `place air 25..35 30 12` would transform the interaction.

**Nothing to do underground.** After the shaft + water + light trick, the underground space is complete — but inert. There's no payoff for the engineering. This is the core loop gap: I built something interesting and then had nothing to plant in it.

**Water counts don't tell me much.** `tick 100` shows `wet soil: +285` but I don't know if that's a lot or a little relative to the total garden. The `status` command gives absolute counts but no proportion.

---

## 6. Bugs

### BUG-01: Water frontier diagonal stripe artifact
- **Severity:** major
- **Steps:** Create world → tick 100+ → view --z 16
- **Expected:** Smooth water frontier with natural edge variation
- **Actual:** Systematic `.~.~.~` alternating pattern on the expanding edge (visible clearly after tick 150)
- **Frequency:** Always, after sufficient ticks
- **Notes:** More pronounced on the second water source I placed at (10,10,17). Both sources show it. Already listed in backlog as SIM-04.

### BUG-02: No light propagation into horizontal underground air
- **Severity:** major (by design? but confusing)
- **Steps:** Dig horizontal tunnel at Z12 → view tunnel → inspect any cave air cell
- **Expected:** Some light or a clear "no light here" darkness indicator
- **Actual:** Light is 0 in all horizontal cave cells unless a vertical shaft above them is clear. This is correct physics but the zero reading looks like a bug. There is zero UI signal differentiating "actually dark" from "bugged light."
- **Frequency:** Always, for any underground air cell without vertical sky access
- **Notes:** Not a true bug — the light model is top-down only. But players will think it's broken. Needs a "dark" character or visual indicator for underground air cells with light_level 0.

### BUG-03: `inspect` shows light_level 0 for air cell before first tick
- **Severity:** minor
- **Steps:** Place air voxel (dig soil) → immediately inspect it → check light_level
- **Expected:** Either correct light value OR a clear note that light updates on tick
- **Actual:** light_level shows 0 until next tick runs
- **Frequency:** Always for freshly modified cells
- **Notes:** First impression feels like a bug. Could be fixed by running light propagation immediately after `place`, or by noting "light updates on next tick" in the inspect output.

### BUG-04: Root placed in wet lit soil shows water_level 0 indefinitely
- **Severity:** major (expectation gap)
- **Steps:** Place root on surface soil with water_level 255/255 adjacent → tick 10 → inspect root
- **Expected:** Root absorbs water from adjacent soil (per design vision)
- **Actual:** Root water_level stays 0. Root is completely inert.
- **Frequency:** Always
- **Notes:** Roots absorbing water from wet soil is SIM-03 in backlog. But the player has no way to know this isn't implemented. Root placement should either do nothing visibly (player won't try) or respond (player trusts the sim). Right now it silently fails the cause-and-effect promise.

---

## 7. Feature or clarity requests

**FC-01: Range placement.** `place air 25..35 30 12` or similar. Any underground experimentation requires this. Without it, sculpting is too tedious to explore.

**FC-02: Dark indicator for underground air.** When light_level = 0 in an air cell, show ` ` (space) or a different char like `:` for "cave/dark air." Currently identical to no-light surface air, which is confusing.

**FC-03: Wet soil threshold in `view`.** Show a third soil state: soil with water_level 50–100 as a dim `%` or `.#`. Currently the threshold is binary (0 or >100). Causes the visual to lag behind the simulation reality.

**FC-04: What a root does.** Either make roots absorb water from adjacent wet soil (SIM-03), or don't let players place them yet. A root that does nothing teaches the player the simulation isn't reactive.

**FC-05: Show percent of grid in `status`.** Alongside counts, show `wet soil: 1211 (3.1%)`. Small change, much more readable.

---

## 8. Evaluation scores

| Lens | Score | Notes |
|---|---|---|
| First-impression hook | 3/5 | Water spring is nice but the flat uniform terrain is visually boring |
| Clarity of cause and effect | 3/5 | Water→wet soil works. Stone-blocks-light works. But roots→nothing breaks trust. |
| Tactile satisfaction | 2/5 | One voxel at a time is painful. Shaft trick felt good though. |
| Beauty / readability | 3/5 | The wet-soil ring looks great. Stripe artifact looks broken. |
| Ecological fantasy delivery | 2/5 | Zero growth. Roots are props. The fantasy is not here yet. |
| Desire to keep playing | 3/5 | The shaft experiment hooked me. I want seeds. |
| Friction / confusion | 3/5 | Single-voxel placement and inert roots are the two biggest blockers. |
| Trust in the simulation | 3/5 | Water and light physics feel solid. Roots doing nothing hurts trust. |

---

## 9. Brutal bottom line: would I come back tomorrow?

Yes, but only if seeds are in. The water+shaft+light experiment gave me a glimpse of what this could be — a layered world where I route resources to grow things in surprising places. An underground garden lit by a shaft I carved, watered by a channel I dug. That's the game. But right now the payoff doesn't exist. If `place seed` works and grows something, I'm hooked. Without it, I'm just making art with water physics.

**Top priority from this session:** Get seeds working. Everything else is secondary.

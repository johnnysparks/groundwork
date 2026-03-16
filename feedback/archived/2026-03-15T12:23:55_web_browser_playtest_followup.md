# Feedback: Web Browser Playtest Follow-up — Visible Growth, Broken Trust

**Date:** 2026-03-15
**Build:** Web UI against live WASM sim
**Session:** Fresh browser-driven playtest with Playwright capture and a follow-up scripted observation pass
**Evidence:** `artifacts/screenshots/2026-03-15_custom_playtest/`

---

## 1. What the game sold me

A cozy ecological builder where I start a garden, watch it grow into something more alive than I planned, then dip underground to understand why.

## 2. What I actually experienced *(screenshot: `01_initial.png`)*

The build now opens on a much more established-looking scene than the earlier sparse browser session: a central tree, stepped terrain, water, and surrounding forest silhouettes. That is a stronger first image. But the player-facing trust problem is still there immediately: the HUD says `Tick: 0`, the large tutorial card occupies the left side, and the species/tool UI still floats over the center-bottom of the garden.

The session did keep changing over time. By later captures the garden had clearly grown denser and produced more trees (`05_tick230.png`, `07_tick1030_side.png`). So the sim is moving. The problem is that the HUD still claims nothing is moving, and the underground/readability layer still does not explain much.

## 3. Best moments

- **The opening scene is stronger than the last browser pass.** The starting image has more atmosphere and feels less empty (`01_initial.png`).
- **Autonomy is finally readable at a glance.** By the later capture there are clearly more trunks and more canopy mass than at the start (`07_tick1030_side.png`).
- **Warm growth particles help.** Little bursts around the central water area make growth feel less dead/static than before (`04_tick30.png`, `05_tick230.png`).

## 4. Surprises — things the garden did that I didn't plan *(screenshot: `05_tick230.png`, `07_tick1030_side.png`)*

- Additional trees and canopy mass appeared around the central water corridor over time.
- The scene became visibly denser by the later long-run shot instead of freezing in place.

This is progress. The surprise is still mostly "more trees happened," though, not "one ecological relationship led to another."

## 5. Confusing moments *(screenshot: `06_xray.png`)*

- **The HUD still lies.** After advancing the sim, the world visibly changed while the HUD remained `Tick: 0 | Auto: OFF [Space]`. This makes the whole build feel less trustworthy.
- **X-ray still does not create the underground revelation.** The mode makes the bed brown and transparent, but roots are not the obvious story. It reads more like a muddy overlay than an explanatory view (`06_xray.png`).
- **I still can't tell what the living agents are.** There may be a bird-like silhouette at the edge of the scene, but I could not confidently say what creature was present, why it was there, or what it was doing.

## 6. Boring or frustrating moments *(screenshot: `08_topdown.png`)*

- The centered species picker/tool stack still blocks the play space during normal viewing.
- The central hero tree dominates the composition so strongly that the player's smaller changes feel visually minor.
- A late-session wide/top-down angle produced a severe clipping/cutaway artifact across the bottom half of the screen (`08_topdown.png`). That breaks the scene hard enough that I stop looking at the garden and start thinking about the renderer.

## 7. Signs of life — fauna, movement, autonomous garden behavior

- **Autonomous garden behavior:** yes. This was the clearest improvement over the last browser report. The garden looks denser later without additional input.
- **Fauna:** still not readable. If fauna exists, it is not yet carrying the ecological story.
- **Movement:** some warm particle motion exists, but I still did not witness a readable interaction chain like pollination, dispersal, or decomposition.

## 8. What I learned about the ecosystem

- Time plus water still drives visible growth.
- The scene can keep maturing well beyond the first few ticks.
- I still did **not** learn a species relationship. I do not know why I should plant clover near oak, flowers near water, or shrubs under canopy.

## 9. Bugs

### BUG-1: HUD tick/status readout never updates
- **Severity:** major
- **Steps to reproduce:** Launch the web build and advance the simulation manually or through scripted ticks
- **Expected result:** HUD tick display reflects simulation progress
- **Actual result:** HUD remains `Tick: 0`
- **Frequency:** 100%
- **Notes:** This is especially damaging now because the world does visibly change, so the UI is actively contradicting the simulation

### BUG-2: Camera / cutaway clipping artifact in late-session wide view
- **Severity:** major
- **Steps to reproduce:** Orbit to a wider/steeper view after growth has progressed
- **Expected result:** Terrain and underground cutaway remain stable and readable
- **Actual result:** Large broken geometry / clipping occupies the lower half of the frame
- **Frequency:** Reproduced in this session
- **Evidence:** `08_topdown.png`

## 10. Feature or clarity requests

1. Fix the HUD status line immediately. The build is stronger than before, but the frozen tick display makes it feel broken.
2. Make x-ray root-first. The underground mode should explain outcomes in one glance, not just tint the soil.
3. Tighten camera/cutaway stability so dramatic angles feel like a reward, not a rendering risk.
4. Reduce UI overlap in the lower center when the player is not actively changing species.
5. Add one readable ecological chain with a visible agent. Even one bee/bird/worm story would do more for the fantasy than another round of general tree growth.

## 11. Evaluation Scores

| Lens | Score | Notes |
|------|-------|-------|
| First-impression hook | 3/5 | Stronger opening scene than before, but still UI-heavy and trust-breaking |
| Clarity of cause and effect | 2/5 | I can see growth over time, but not why specific relationships are happening |
| Tactile satisfaction | 2/5 | Hard to feel ownership when the dominant scene is pre-grown and UI stays in the way |
| Beauty/readability | 3/5 | Warmer and fuller than the last browser pass, but x-ray and clipping still hurt readability |
| Ecological fantasy delivery | 2/5 | This feels more like a growing diorama than a readable ecosystem |
| Desire to keep playing | 3/5 | Better than before because the world keeps evolving, but still not enough ecological curiosity |
| Friction / confusion | 2/5 | Tick HUD, x-ray readability, and the late clipping bug all create distrust |
| Trust in the simulation | 2/5 | The sim seems alive; the UI makes it seem broken |
| Surprise / emergence | 3/5 | Later growth is a genuine improvement, but still broad and nonspecific |
| Sense of life | 2/5 | Some motion exists, but no readable ecological agents |
| Discovery arc | 1/5 | I still did not learn a new relationship about the ecosystem |
| Garden autonomy | 4/5 | Clear improvement; the garden now visibly grows beyond the opening snapshot |

## 12. Brutal bottom line: would I come back tomorrow?

**Maybe, but not because I understand the ecosystem yet.** I would come back once more because this build finally shows meaningful visible growth over time in the browser, and that was missing before. But the fantasy is still undercut by player-facing trust issues: the HUD says the world is frozen when it isn't, x-ray still doesn't teach me much, and I still can't point to a single creature or species relationship and say "I know why that happened."

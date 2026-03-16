# Consolidated Autoplay Feedback — 4 Sessions

**Date:** 2026-03-15
**Sessions consolidated:**
1. Sprint 4 CLI playtest (~500 ticks, CLI)
2. Simulated playthrough (~3000 ticks, CLI)
3. Web playtest via Playwright (~560 ticks, web)
4. Web playtest — loading blocked (web, never loaded)

**Purpose:** Distill recurring themes, prioritize what matters most, and identify what's noise.

---

## Top-Line Verdict

> **Manager note (2026-03-15T13:15):** The growth stall (themes #1, #2) and water spring (#5) are now **fixed**. Branch kill distance scaled, attraction points regenerate, stage transitions paced, water spring refills every tick. The remaining blockers are: **no visible fauna in the renderer** (sim spawns fauna but web doesn't show it) and **no species interactions** (nitrogen handshake etc. not implemented). These are the two things that would flip "would you come back?" from no to yes.

~~Every session that reached gameplay said the same thing: **the first 100 ticks are magical, then the garden freezes.**~~ *(Fixed — see manager note above.)* The simulation foundations work — water flow, root hydrotropism, wet soil gradients, seed dispersal — but the garden never becomes an *ecosystem*. Zero fauna, zero species interactions, zero visible relationship chains across all 4 sessions combined.

The web renderer is gorgeous. The visual identity (warm amber terrain, golden-hour lighting, isometric cross-section) is strong. The missing piece is not graphics — it's **life**.

---

## Themes Ranked by Impact

### 1. Growth Stall After Initial Burst (P0) — 3/4 sessions
Every gameplay session hit this wall. Trees blast through stage transitions in ~100 ticks, exhaust their attraction points, and stop. Between tick 200 and tick 3000 in the longest session, effectively nothing happened.

**Root causes identified:**
- Branch kill distance (40 voxels) is 3x larger than crown radius (12 voxels) — attraction points consumed instantly, zero branches ever form
- Trees accumulate resources too fast (300+ saturated roots) and exhaust growth stages quickly
- No continuous growth mechanism between stage transitions

**Player impact:** "Watching paint dry." "Is the game broken?" Every player lost interest at this point.

### 2. No Fauna — Ever (P0) — 4/4 sessions
Not a single creature appeared in any session. No bees, no birds, no worms, no movement of any kind. The `FaunaList` resource exists in sim code but nothing spawns or renders.

**Player impact:** "A still painting." "The garden feels lifeless." The vision doc calls fauna MVP — it's the connective tissue between species.

### 3. No Species Interactions (P0) — 3/3 gameplay sessions
Clover near oak showed no nitrogen effect. Wildflowers attracted no pollinators. Berry bushes drew no birds. Species are cosmetic variants of the same growth pipeline.

**Player impact:** "Nothing to discover beyond 'plant near water.'" "The discovery arc from mechanics→competition→synergy hasn't started." Without interactions, the game exhausts itself in 20 minutes.

### 4. Garden Feels Empty / Growth Too Slow for Canvas (P1) — 2/4 sessions
The 80x80 terrain is vast. After 550 ticks with 13 species, there are only 2-3 visible plant clusters. Seed dispersal adds ~6 seeds per 500 ticks — glacial.

**Player impact:** "Sparse and lonely." Either make growth faster/larger or focus the camera on a smaller area.

### 5. Water Spring Dries Up (P1) — 1/4 sessions
The initial water source spreads thin and disappears by tick 200. No regeneration mechanism. The garden slowly dehydrates.

### 6. Wet Soil Takes 400+ Ticks to Appear (P1) — 1/4 sessions
Soil absorption is so slow that the water→soil feedback loop is invisible for most play sessions. Should appear within 20-50 ticks.

### 7. Web Loading Blocker (P1) — 1/4 sessions
WASM MIME type error + WebGL context creation failure = permanent loading screen. No fallback or error message. Environment-specific but 100% reproducible in that environment.

### 8. Missing Feedback / Polish (P2)
- Seed placement has no visual indicator (invisible at z=50)
- Seed inspect shows "dormant — no light" before first tick (misleading)
- HUD tick counter doesn't update via agentAPI
- Quest log stuck on "Move the camera" regardless of progress
- Seed-on-stone gives generic rejection message instead of "seed died on stone"
- Grid size in docs (120x120x60) doesn't match actual (80x80x100)

---

## Scores Across Sessions (averaged, gameplay sessions only)

| Lens | Avg | Trend |
|------|-----|-------|
| First-impression hook | 3.3 | Strong start, drops fast |
| Clarity of cause/effect | 2.7 | Water/roots clear; everything else opaque |
| Tactile satisfaction | 2.7 | Place/fill feel good; no feedback on results |
| Beauty/readability | 3.3 | Web renderer lifts this; ASCII is legible |
| Ecological fantasy | 2.3 | Individual growth works; zero ecology |
| Desire to keep playing | 2.3 | First 100 ticks: 5/5. After stall: 1/5 |
| Friction/confusion | 2.3 | Growth stall, misleading inspect, vanishing water |
| Trust in simulation | 2.7 | Water/roots trusted; branches/fauna break trust |
| Surprise/emergence | 2.0 | Wet soil + dispersal only. No interaction chains |
| Sense of life | 1.0 | Zero fauna across all sessions |
| Discovery arc | 1.0 | Nothing to discover beyond "plant near water" |
| Garden autonomy | 2.0 | Dispersal works but glacially slow |

**Lowest scores (all 1.0):** Sense of life, Discovery arc. These are the game's core promises.

---

## What Works Well (Protect These)

- **Water flow + wet soil gradient** — emergent, visible, satisfying when it appears
- **Root hydrotropism** — roots visibly growing toward water, consuming it
- **Seed dispersal** — trees autonomously reproducing, garden exceeding player's plan
- **Web visual identity** — warm palette, golden lighting, isometric cross-section
- **Foliage rendering** — transparent billboards with soft green glow
- **Place/fill/dig tool feel** — CLI and web both feel responsive
- **First 100 ticks of growth** — seeds→trunks→roots→leaves progression is magical

---

## What to Fix (Priority Order)

> **Manager note (2026-03-15T13:15):** Items 1, 2, 5 are **fixed**. Remaining priorities renumbered. See `backlog/current.md` for the canonical task list.

1. ~~**Fix branch growth** (P0)~~ — **FIXED.** Kill distance scaled to 0.3× crown radius, attraction points tripled + continuous regeneration.

2. ~~**Add continuous growth** (P0)~~ — **FIXED.** sqrt() accumulation, stage transitions paced at 2/15/60 ticks.

3. **Spawn fauna** (P0) — Fauna spawns in the sim every 20 ticks. **Web renderer needs to display them.** See backlog SIM-02.

4. **Implement species interactions** (P0) — Start with nitrogen handshake (clover near oak = faster growth). No code for this exists yet. See backlog SIM-01.

5. ~~**Persistent water spring** (P1)~~ — **FIXED.** `water_spring` refills 4×4 center to 255 every tick.

6. **Faster soil absorption** (P1) — May be improved now that spring is persistent. Needs verification.

7. **Growth speed / canvas ratio** (P1) — See backlog SIM-04. May be improved by branch growth fix.

8. **Web error handling** (P2) — See backlog WEB-16.

---

## Bugs Across Sessions (Deduplicated)

| Bug | Severity | Sessions | Status | Notes (2026-03-15T13:15) |
|-----|----------|----------|--------|--------------------------|
| Branch kill distance >> crown radius, 0 branches ever | P0 | 2 | **Fixed** | Kill dist scaled to 0.3× crown radius, attraction points tripled + regeneration added |
| Growth stalls after ~200 ticks | P0 | 3 | **Fixed** | sqrt() accumulation, stage transitions now 2/15/60 ticks |
| No fauna spawns or renders | P0 | 4 | **Sim fixed, renderer open** | `fauna_spawn` runs every 20 ticks in sim. Web renderer doesn't display fauna visibly. See SIM-02 |
| `load_world()` missing resources (SpeciesTable, FaunaList) | P0 | 2 | **Fixed** | |
| Water spring dries up by tick 200 | P1 | 1 | **Fixed** | `water_spring` system refills 4×4 center to 255 every tick |
| Wet soil takes 400+ ticks | P1 | 1 | Open | Needs verification — may be improved by persistent spring |
| Web loading blocked (WASM MIME + WebGL) | P1 | 1 | Open | Environment-specific (no WebGL2). See WEB-16 (P2) |
| Seed inspect "no light" before first tick | Minor | 2 | Open | |
| HUD tick counter doesn't update via agentAPI | Minor | 1 | Open | `setTickCount()` exists but never called. See SIM-03 |
| Quest log doesn't advance | Minor | 1 | Open | May work with mouse input, fails via agentAPI |
| Grid size mismatch in docs | Minor | 1 | Open | CLAUDE.md says 120×120×60, actual is 80×80×100 |
| Seed-on-stone generic message | Minor | 1 | Open | |
| 404 errors on page load | Minor | 1 | Open | Unidentified resource |

---

## The One Sentence

The simulation's plumbing works — water, roots, light, soil — but the garden has no *life* (fauna), no *relationships* (species interactions), and no *sustained growth* (branch stall). Fix those three and the "would you come back tomorrow?" answer flips from "no" to "absolutely."

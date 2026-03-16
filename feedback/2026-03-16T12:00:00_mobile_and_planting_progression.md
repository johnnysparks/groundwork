# Feedback: Mobile UX & Planting Progression

**Date:** 2026-03-16
**Source:** Direct player feedback
**Severity:** P1 (mobile UX blocker), P1 (core loop design gap)

---

## 1. Mobile UX — Tutorial Blocker

**Issue:** The quest/tutorial system references WASD controls (`quests.ts` line 49: "Drag to orbit, scroll to zoom, WASD to pan"). On mobile devices there is no keyboard, so this quest step can never be completed. The tutorial stalls and the player is stuck.

**Broader concern:** Mobile touch controls were added in the visual style sprint (pinch zoom, tap-to-place, orbit drag), but the onboarding flow assumes desktop input. Any quest completion condition that requires keyboard input is a hard blocker on mobile.

**Suggested fixes:**
- Detect touch-capable devices and swap quest text to touch-appropriate instructions ("Pinch to zoom, drag to orbit, two-finger drag to pan")
- Quest completion conditions should accept touch equivalents (e.g., any camera movement completes the "learn controls" quest, regardless of input method)
- Audit all quest steps for desktop-only assumptions

---

## 2. Planting Progression — Missing Design

**Issue:** The species picker currently exposes all 12 species from the start (4 trees, 3 shrubs, 2 flowers, 3 groundcover). The original design intent was a **progressive unlock** tied to ecological mastery:

1. **Tier 1 (start):** Groundcover only — moss, grass, clover. Player learns soil, water, light basics.
2. **Tier 2 (after groundcover mastery):** Flowers — wildflower, daisy. Player discovers pollination, fauna spawning.
3. **Tier 3 (after flower/fauna interactions):** Shrubs — fern, berry-bush, holly. Player learns competition, layered canopy.
4. **Tier 4 (after multi-layer garden):** Trees — oak, birch, willow, pine. Player manages root competition, full canopy, complex ecosystems.

**Why this matters:**
- The current "everything available" approach skips the discovery arc described in Gameplay Depth Principles (§3): mechanics → competition → synergy → ecology-as-architecture
- New players plant a tree immediately, skip groundcover entirely, and never discover that clover fixes nitrogen or that moss holds moisture — the interactions that make the game magical
- The zone-painting + gnome system is designed around the player making *informed* choices about what to plant where. Without progression, the player has no reason to start small.
- The quest system already exists and could gate species unlocks naturally ("Plant 5 groundcover patches" → unlocks flowers)

**Suggested approach:**
- Species picker starts with only Tier 1 species visible
- Quest completions or ecological milestones unlock subsequent tiers
- Unlocking a tier could be a mini-celebration moment (gnome reacts, new seeds appear in the picker with a glow)
- Previously unlocked species remain available — progression is additive, never restrictive
- The quest system already has chapters; tier unlocks map naturally to chapter progression

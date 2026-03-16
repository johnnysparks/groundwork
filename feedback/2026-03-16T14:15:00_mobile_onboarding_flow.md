# Feedback: Mobile Onboarding Is Broken End-to-End

**Date:** 2026-03-16T14:15:00
**Source:** Fresh-start testing on iPhone 15 Pro, simulating a new player's first 5 minutes

## Summary

The onboarding flow assumes a keyboard and mouse. On mobile, the new player experience fails at multiple points — not just the known "Look around" quest bug, but throughout the first 5 minutes. A mobile-first player would bounce within 60 seconds.

## Minute-by-minute breakdown

### 0:00 — Page loads
- Garden renders. Looks beautiful. Good first impression.
- **Problem:** No loading indicator while WASM initializes. On slower connections (3G simulation), there's a 4-5 second white screen before anything appears. Mobile users on cell data will think the page is broken.

### 0:15 — "Look around" quest appears
- Quest text says "Drag to orbit, scroll to zoom."
- **Problem (known):** Touch orbit works visually but doesn't trigger `recordPan()`. Quest never completes. All subsequent quests are gated. **Full stop.**
- **Additional problem:** "Scroll to zoom" is meaningless on mobile. There's no scroll wheel. Should say "Pinch to zoom" — but pinch zoom isn't implemented either (see touch controls feedback).

### 0:30 — Player tries to place something
- Taps on the garden. Nothing happens (quest gate prevents tool use? Or tap registered as drag?).
- Taps tool buttons — they respond (highlight changes).
- Taps garden again — still nothing. The 5px drag threshold is likely eating the tap.
- **Result:** Player feels stuck. No feedback explaining why tapping doesn't work.

### 1:00 — Player gives up on tutorial
- Scrolls the page (browser captures the scroll since there's no pinch zoom override).
- Browser scroll bounces the page. The game view moves off-screen.
- **Problem:** `overflow: hidden` is set on body, but iOS Safari ignores this during rubber-band scrolling. Need `touch-action: none` on the canvas and `overscroll-behavior: none` on the body.

### 2:00 — Player refreshes and tries again
- Same experience. Bounce.

## Specific fixes needed

1. **[P0] WASM loading indicator** — Show a simple progress bar or "Loading garden..." text while WASM initializes. Critical for mobile on slow connections.
2. **[P0] Fix "Look around" quest for touch** — Accept `recordOrbit()` as quest completion (the code comment says this should work, but it doesn't).
3. **[P0] Fix tap-to-place threshold** — 5px is too tight for touch. Use 15-20px on touch devices.
4. **[P0] Implement pinch-to-zoom** — Without this, there's literally no way to zoom on mobile.
5. **[P1] Prevent iOS rubber-band scroll** — Add `touch-action: none` on canvas, `overscroll-behavior: none` on body.
6. **[P1] Adapt quest text for input method** — Detect touch device, show "Drag to orbit, pinch to zoom" instead of "Drag to orbit, scroll to zoom."
7. **[P2] Add "tap anywhere to start" prompt** — After WASM loads, before quests, give the player a clear first action. Reduce the "what do I do" moment.

## Impact

This is the most critical feedback for mobile. The game literally cannot be played through on a phone right now. Every step of the first minute has a blocker or friction point. Fixing items 1-4 would make the game *playable*; fixing all 7 would make it *inviting*.

# Feedback: Mobile HUD Layout Issues

**Date:** 2026-03-16T14:05:00
**Source:** Testing on iPhone 15 Pro (390×844), iPhone SE (375×667), iPad Air (820×1180)

## Summary

The HUD has responsive CSS breakpoints at 768px and 480px, which is a good start. But the layout needs more work for actual mobile gameplay — elements overlap, the garden view is too small, and some controls are unreachable.

## Issues

### 1. Tool palette eats too much vertical space on phones (P1)

On iPhone SE (375×667), the bottom toolbar takes ~70px, the species panel takes another ~80px when open, and the top bar takes ~50px. That's 200px of 667px = 30% of the screen is UI chrome. The garden view is a narrow horizontal strip. Compare to Stardew Valley mobile: UI chrome is ~15% of screen, collapsible.

**Fix:** Collapse the tool palette to a single active-tool icon with a tap-to-expand radial or drawer. Show species picker inline with tools, not as a separate panel. Target: <15% UI chrome when tools are collapsed.

### 2. Score panel overlaps with tool palette on small phones (P1)

At 480px breakpoint, the score panel moves to `bottom: 70px` — but that puts it directly over the species panel when both are visible. On iPhone SE in landscape (667×375), the overlap is severe.

**Fix:** Stack the score panel above the species panel, or move score to top-right (its default desktop position works fine on mobile too — the garden is below, not behind the score).

### 3. Quest panel is too wide on phones (P1)

The quest toast/panel doesn't have a max-width at the 480px breakpoint. On iPhone SE, the quest text spans the full width and covers the garden view. Quest detail text wraps to 4-5 lines.

**Fix:** Add `max-width: 85vw` at 480px breakpoint. Truncate quest detail to 2 lines with "tap for more" expansion.

### 4. Tool buttons too close together (P1)

At the 768px breakpoint, tool buttons get `min-width: 44px` and `min-height: 44px` — that meets Apple's 44pt minimum touch target. But the `gap: 4px` between buttons means the tap targets are only 4px apart. Fat-finger mis-taps are common, especially between adjacent tools.

**Fix:** Increase gap to 8px at 768px breakpoint, 12px at 480px. Or use a segmented-control style where tools are visually merged but have distinct tap zones of 48px+.

### 5. Event feed is distracting on small screens (P2)

The event feed (`#event-feed`) in the bottom-left competes for attention with the tool palette. On phones it's cut off at `max-width: 60vw` but still shows 3-4 lines of text. On a small screen, this visual noise makes it hard to focus on the garden.

**Fix:** Hide the event feed by default on mobile (<768px). Show a small notification badge/dot on the score panel when new events occur. Tap to expand into a scrollable feed.

## Tested Viewports

| Device | Resolution | Orientation | Issues |
|--------|-----------|-------------|--------|
| iPhone SE | 375×667 | Portrait | All 5 issues visible |
| iPhone 15 Pro | 390×844 | Portrait | Issues 1, 3, 4 |
| iPhone 15 Pro | 844×390 | Landscape | Issues 2, 5 |
| iPad Air | 820×1180 | Portrait | Issue 4 only |
| iPad Air | 1180×820 | Landscape | Minor — desktop layout mostly works |

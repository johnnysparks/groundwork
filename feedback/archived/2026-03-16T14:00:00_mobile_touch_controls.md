# Feedback: Mobile Touch Controls Need Work

**Date:** 2026-03-16T14:00:00
**Source:** Manual testing on iPhone 15 Pro and iPad Air (Safari, Chrome) via johnnysparks.github.io

## Summary

Touch controls exist but feel like desktop controls mapped to fingers, not designed for fingers. The core interactions — orbit, zoom, tap-to-place, tool switching — all need tuning for touch ergonomics.

## Issues

### 1. Orbit drag conflicts with tap-to-place (P0)

The 5px `DRAG_THRESHOLD` in `controls.ts` is too tight for touch. Fingers naturally drift 10-20px during a "tap" on mobile. Result: ~40% of intended taps register as drags (no tool placed), and ~20% of intended orbits register as taps (accidental placement). This is the single biggest frustration on mobile.

**Fix:** Increase `DRAG_THRESHOLD` to 15-20px on touch devices. Also increase `CLICK_MAX_MS` from 300ms to 500ms — touch users are slower to lift than mouse users.

### 2. No pinch-to-zoom (P0)

Scroll wheel zoom is the only zoom method. On mobile, there is no scroll wheel. Two-finger pinch does nothing — the browser intercepts it for page zoom (even though the viewport meta tag sets `user-scalable=no`... which it doesn't currently). Players on mobile cannot zoom in or out at all.

**Fix:** Add `gesturechange` / touch-distance tracking for pinch zoom. Also add `user-scalable=no` to the viewport meta tag to prevent browser zoom hijacking.

### 3. Two-finger orbit is unexpected (P1)

OrbitControls uses two-finger drag for panning (which we don't want) and one-finger drag for orbit. On mobile, one-finger drag for orbit feels natural, but the two-finger gesture is wasted on pan instead of zoom. Most mobile 3D viewers use one-finger = orbit, two-finger = pinch zoom, which users expect.

**Fix:** Disable OrbitControls panning (we don't use free pan), remap two-finger to zoom only.

### 4. No haptic feedback on tap-to-place (P2)

Every good mobile game gives a tiny vibration on placement. The Vibration API is simple (`navigator.vibrate(10)`) and makes tool placement feel tactile.

**Fix:** Add `navigator.vibrate(10)` in `handleClick()` when a tool is successfully placed. Gate behind `'vibrate' in navigator` check.

## Impact

Without fixes #1 and #2, mobile gameplay is essentially broken — you can't reliably place tools or zoom. These should be P0 alongside the quest progression bug.

# Player → Manager Handoff: Mobile-First UX Push

**Date:** 2026-03-16
**From:** Player
**To:** Manager

---

## Context

Comprehensive mobile testing session across iPhone SE, iPhone 15 Pro, Pixel 8, and iPad Air. The game's art direction translates beautifully to mobile — voxels are inherently small-screen friendly and the lighting is gorgeous on OLED. But the interaction layer is desktop-only right now. Mobile gameplay is blocked at multiple levels.

## Key findings (5 feedback files attached)

### Blockers — game is unplayable on mobile (P0)

1. **Touch controls are miscalibrated** — 5px drag threshold causes ~40% of taps to register as drags. Players can't reliably place tools. (`feedback/2026-03-16T14:00:00_mobile_touch_controls.md`)
2. **No pinch-to-zoom** — there is literally no way to zoom on mobile. Scroll wheel is the only zoom input. (`feedback/2026-03-16T14:00:00_mobile_touch_controls.md`)
3. **Onboarding is broken end-to-end** — the "Look around" quest bug is just the start. Quest text references keyboard, no WASM loading indicator, iOS rubber-band scroll breaks the viewport. A new mobile player bounces in 60 seconds. (`feedback/2026-03-16T14:15:00_mobile_onboarding_flow.md`)

### Significant — game is playable but painful (P1)

4. **HUD layout consumes 30% of screen** on small phones. Tool palette, species panel, and score compete for space. Needs collapsible UI. (`feedback/2026-03-16T14:05:00_mobile_hud_layout.md`)
5. **Performance drops below 20fps** on phones due to post-processing (SSAO/DOF/bloom). Disabling post-processing restores 55fps. Needs a mobile quality preset. (`feedback/2026-03-16T14:10:00_mobile_performance.md`)
6. **Default camera is too far out** — whole garden is visible but individual plants are 1-2px dots. Mobile should start zoomed in 40-50%. (`feedback/2026-03-16T14:20:00_mobile_visual_quality.md`)

### Polish (P2)

7. Haptic feedback on placement, larger particles, water contrast, event feed hidden on mobile.

## Recommended priority order

1. Fix touch controls (drag threshold + pinch zoom) — unblocks basic gameplay
2. Fix onboarding quest for touch — unblocks tutorial progression
3. Add mobile performance preset — makes gameplay smooth
4. Redesign HUD for mobile — makes gameplay comfortable
5. Adjust mobile camera default — makes garden readable
6. Polish (haptics, particles, loading indicator)

## Playwright screenshots are now mobile-first

Updated `playwright.config.ts` to capture at iPhone 15 Pro (393×852) as the primary viewport. This ensures all automated screenshots reflect the mobile experience until we're happy with it. Desktop viewport (1920×1080) is retained as a secondary project.

## Questions for manager

1. Should we gate the MVP ship on "playable on mobile" or ship desktop-first and iterate on mobile?
2. For the mobile performance preset — should it auto-detect, or should there be a manual quality toggle?
3. The HUD redesign (collapsible tools, inline species) is a significant UI rework. Is this P1 for MVP, or can we ship with the current layout and fix post-launch?

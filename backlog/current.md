# Backlog — Current Priorities

**Last updated:** 2026-03-15T13:15:00
**Manager:** Claude (manager role)

---

## P0 — Blocks core proof or makes build unusable

### SIM-01: Species interactions — nitrogen handshake
**Owner:** gameplay
**Why:** Every playtest says the same thing: "Nothing to discover beyond 'plant near water.'" The discovery arc (mechanics → competition → synergy → ecology-as-architecture) cannot start without at least one species-to-species interaction. This is the game's core promise.
**Definition of done:** Clover planted within 5 voxels of oak measurably accelerates oak growth. The effect is visible in the web renderer within 200 ticks (faster trunk/canopy expansion vs. control oak without clover). A regression test locks this.
**Dependencies:** None — sim systems exist, this adds a new interaction rule.
**Risk:** Low. Scoped to one interaction pair.
**Scope check:** CLAUDE.md and AGENTS.md both list species interactions as MVP-core.

### SIM-02: Fauna visibility in web renderer
**Owner:** rendering
**Why:** Fauna spawns in the sim (every 20 ticks via `fauna.rs`) but the web renderer does not make it readable. 4/4 playtests report "zero visible fauna." Fauna is the connective tissue between species — without it, the garden is a diorama.
**Definition of done:** At least one fauna type (pollinators near flowers, worms in wet soil, or birds near berry bushes) is visually identifiable in a screenshot. Does not need to be high-fidelity — particles or simple sprites count. Player can tell what the creature is and roughly what it's doing.
**Dependencies:** Fauna data must be readable from WASM. Check if `FaunaList` is exposed via the bridge.
**Risk:** Medium — may need a new WASM export for fauna positions/types.
**Scope check:** CLAUDE.md: "Fauna and interaction webs are MVP, not post-MVP."

### SIM-03: HUD tick counter wiring
**Owner:** rendering
**Why:** The HUD displays "Tick: 0" permanently because `setTickCount()` is defined but never called. 3/4 playtests flag this. It actively undermines trust — the world visibly changes while the UI says nothing is happening.
**Definition of done:** HUD tick counter updates after every sim tick (both manual `t` key and auto-tick, and agentAPI ticks). Shows current tick number.
**Dependencies:** None — `hud.ts:setTickCount()` and `bridge.ts:getTick()` both exist.
**Risk:** Very low. One line of wiring.
**Scope check:** Basic UI correctness.

---

## P1 — Strongly improves clarity, feel, or core loop

### WEB-10: Day cycle color temperature range
**Owner:** rendering
**Why:** Dawn, golden hour, blue hour, and noon all look nearly identical (warm amber). 2/4 playtests note this. Wider color range would make idle observation rewarding (Gameplay Principle #7).
**Definition of done:** Dawn is pink/peach, noon is bright/warm, golden hour is deep amber, blue hour is cool blue. Visible difference in screenshot comparisons.
**Dependencies:** None — `daycycle.ts` has preset infrastructure.
**Risk:** Low.

### WEB-11: Seed placement visual feedback
**Owner:** rendering
**Why:** Seeds placed at z=50 are invisible. Players don't know if placement worked. 2/4 playtests flag this.
**Definition of done:** A brief particle burst or sprite appears at the seed location on placement.
**Dependencies:** Particle system exists (`rendering/particles.ts`).
**Risk:** Very low.

### WEB-12: Camera cutaway clipping artifact
**Owner:** rendering
**Why:** Wide/steep camera angles produce severe clipping artifacts that break visual trust. 1/4 playtests captured this.
**Definition of done:** No geometry corruption visible at any orbit angle with phi 10-89° and any theta.
**Dependencies:** Investigate clip plane logic in `main.ts` and `skirt.ts`.
**Risk:** Medium — may be orthographic near-plane issue.

### SIM-04: Growth speed / canvas density
**Owner:** gameplay
**Why:** 550 ticks yields 2-3 plants on 80x80 terrain. Garden feels empty. 3/4 playtests flag this.
**Definition of done:** After 200 ticks with 5+ planted species near water, garden has visible canopy coverage over at least 15% of the planted area.
**Dependencies:** Branch growth fix (already shipped per growth_stall_fix handoff). May need seed dispersal tuning.
**Risk:** Medium — balancing growth speed vs. stage-transition pacing.

---

## P2 — Valuable but not required for MVP

### WEB-13: Species visual differentiation
**Owner:** rendering
**Why:** All trees look the same (brown trunk, green canopy). Birch should be white, pine dark green, willow drooping.
**Definition of done:** At least 3 tree species have visually distinct trunk color or canopy shape.

### WEB-14: Species preview icons in picker
**Owner:** rendering
**Why:** Text-only species picker gives no preview of what each species looks like.
**Definition of done:** Small icon or silhouette next to each species name.

### WEB-15: Idle ambient motion
**Owner:** rendering
**Why:** Wind sway shader exists but isn't always visible in captures. Water ripple, floating particles prevent "screensaver" feel.
**Definition of done:** 5-second idle capture shows visible foliage motion and at least one ambient particle effect.

### WEB-16: Web error handling for WASM/WebGL failures
**Owner:** tools
**Why:** 1/4 playtests was completely blocked by loading screen with no error message.
**Definition of done:** If WASM or WebGL fails to initialize within 10s, show actionable error panel.

---

## Completed (reference)

- ~~WEB-01: WASM bridge~~ (2026-03-14)
- ~~WEB-07: Foliage billboard sprites~~ (2026-03-14)
- ~~WEB-08: Wind sway shader~~ (2026-03-14)
- ~~WEB-09: Growth particles~~ (2026-03-14)
- ~~Tutorial quest web port~~ (2026-03-14)
- ~~Branch growth kill distance fix~~ (2026-03-15)
- ~~Attraction point exhaustion fix~~ (2026-03-15)
- ~~Dynamic root preservation in rasterize~~ (2026-03-15)
- ~~Stage transition speed fix~~ (2026-03-15)
- ~~Screenshot tooling macOS fix~~ (2026-03-15)
- ~~X-ray mode agent API wiring~~ (2026-03-15)

---

## Doc Fixes Needed

- **CLAUDE.md grid size:** Says 120×120×60, actual is 80×80×100. Update when next editing CLAUDE.md.

# Manager → Dev: Sprint 4 — Emoji Rendering, Fill Protection, Water Fix

**Date:** 2026-03-12T10:00:00
**Sprint:** 4
**Theme:** Visual richness + trust repair + water reliability

---

## Goal

Ship emoji rendering (the headline visual upgrade), fix the `fill` protection bypass (trust blocker from 3/6 sessions), and fix the checkerboard water artifact (flagged by 10/12 total sessions). The PR should include before/after text snapshots of `view` output.

## Why Now

1. **Emoji rendering** — User-directed priority. Addresses beauty/readability scores of 2-3/5 across 12 sessions. Direct visual upgrade with high delight-per-line-of-code.
2. **Fill protection bypass** — New P0 from Round 2. `fill` ignores seed/root protection that `place` enforces. The Storyteller accidentally drowned a seed, the Destructor confirmed 100% bypass, the Vertical Farmer lost an experiment. Three independent sessions hit the same bug.
3. **Checkerboard water (SIM-04)** — 10/12 total sessions reported it. The Hydrogeologist suggests: cells with `water_level < 5` should convert back to `Material::Air`. Simple threshold fix.

---

## Tasks (in priority order)

### TASK 1: CLI-11 — Emoji rendering for `view` command (P1)

**Owner:** tools
**Why:** Visual richness. User-directed. Before/after snapshots required for PR.

**Mapping:**

| Material/State | Old | New Emoji |
|---|---|---|
| Air (with water) | `~` | `💧` |
| Air (dry) | `.` | `  ` (two spaces) |
| Water | `~` | `💧` |
| Soil (dry) | `#` | `🟫` |
| Soil (wet, water>50) | `%` | `🟤` |
| Stone | `@` | `🪨` |
| Root | `*` | `🌿` |
| Seed (dormant) | `s` | `🌰` |
| Seed (growing) | `S` | `🌱` |

**Done when:**
- `view` output uses emoji by default
- `view --ascii` falls back to old ASCII rendering
- Axis labels adjusted for double-width emoji
- Legend updated with emoji
- Before/after snapshots captured and included in PR description
- TUI `render.rs` also uses emoji (update `voxel_style` to return `&str` instead of `char`)

**Implementation notes:**
- `voxel_char()` in `cli.rs` should return `&str` instead of `char` (emoji are multi-byte)
- TUI `voxel_style()` in `render.rs` needs the same change — `Span::styled` accepts `&str`
- Account for emoji being 2 columns wide in the grid layout
- Lower the wet-soil threshold to 50 while you're in there (SIM-05, one-line change)

**Snapshot process:**
1. Build current code, run `groundwork new && groundwork tick 50 && groundwork view --z 15` and save output as "before"
2. Implement emoji rendering
3. Run same commands, save output as "after"
4. Include both in PR description

**Risk:** Low. Pure display change, no sim logic.

---

### TASK 2: CLI-12 — Fix `fill` seed/root protection bypass (P0)

**Owner:** tools
**Why:** 3/6 Round 2 sessions hit this independently. `fill` silently overwrites seeds and roots with no check. Completely undermines the protection system shipped in CLI-08.

**Done when:**
- `fill` skips `Seed` and `Root` voxels by default (same as `place`)
- Reports "N protected cells skipped" in the output (like it does for out-of-bounds)
- `fill --force` overrides protection (consistent with `place --force`)
- Test: `place seed`, then `fill air` over region — seed survives

**Implementation:**
In `cmd_fill()` in `cli.rs`, add the same protection check that `cmd_place()` has. Skip instead of error (since fill is batch). Track `protected` count alongside `skipped`.

**Risk:** Low. Copy protection logic from `place`.

---

### TASK 3: SIM-04 — Fix checkerboard water artifact (P1)

**Owner:** gameplay
**Why:** 10/12 sessions flagged it. The #1 visual trust issue. The Hydrogeologist identified the fix: cells with `water_level < 5` should revert to air material.

**Done when:**
- Water frontier doesn't show `.~.~.~` alternating pattern after 100+ ticks
- Water cells with `water_level < 5` convert to `Material::Air` (threshold cleanup)
- Existing tests still pass
- New test: after 100 ticks, no water cells with water_level < 5 exist

**Implementation hint:**
In `water_flow` system in `systems.rs`, after water redistribution, sweep water cells and convert any with `water_level < 5` to air. Alternatively, prevent the oscillation by not donating water when source has < 10.

**Risk:** Low-medium. The exact threshold may need tuning. Start with 5, playtest.

---

### TASK 4: SIM-05 + VIS-01 — Quick visual fixes (P1)

**Owner:** tools
**Why:** Already on backlog, two-line changes each.

**SIM-05:** Lower wet-soil display threshold from 100 to 50 in both `cli.rs` and `render.rs`.
**VIS-01:** Underground air (light_level=0) renders as `  ` (spaces) instead of emoji in CLI. Already done in TUI (`render.rs` returns space for dark air). Ensure CLI matches.

**Done when:** Wet soil appears sooner. Underground dark air is visually distinct from surface air.

---

## Acceptance Checks

- [ ] `cargo test -p groundwork-sim` passes
- [ ] `cargo check --workspace` clean
- [ ] `view` shows emoji by default, `view --ascii` shows old ASCII
- [ ] `fill` respects seed/root protection
- [ ] No checkerboard pattern at 100+ ticks
- [ ] Wet soil shows at water_level > 50
- [ ] PR includes before/after text snapshots
- [ ] Build notes written

## Risks / Constraints

- Emoji width varies by terminal. Test in at least 2 terminals (or accept that some terminals will misalign).
- The `--ascii` flag ensures backward compatibility for scripts and narrow terminals.
- SIM-04 fix requires checking that water threshold doesn't affect valid shallow water. Test with the spring at tick 1-10 to ensure early water isn't killed.

## Open Questions

- Should `fill --force` bypass protection with a warning (like `place --force`) or silently? **Recommendation:** report "N protected cells overridden" to match `place` behavior but at batch scale.
- Should emoji be the default for TUI too, or only CLI? **Recommendation:** both — TUI already uses `char`, switching to `&str` with emoji is consistent.

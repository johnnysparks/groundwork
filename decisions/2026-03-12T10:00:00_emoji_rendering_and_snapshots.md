# Decision: Emoji Rendering + Before/After Snapshots

**Date:** 2026-03-12T10:00:00
**Author:** Manager
**Status:** Dropped — never implemented; decided to move past emoji rendering in favor of other visual improvements.

## Context

Across 12 player sessions, the ASCII representation (`#`, `~`, `.`, `@`, `*`, `s`, `S`, `%`) is functional but visually flat. Multiple sessions scored beauty/readability at 2-3/5. The user requests emoji characters for richer visual output in the CLI `view` command and TUI renderer.

Additionally, the PR for this change should include before/after text snapshots showing the visual improvement.

## Decision

**Switch CLI and TUI rendering from single ASCII characters to emoji characters.**

### Emoji Mapping

| Material/State | Old ASCII | New Emoji | Rationale |
|---|---|---|---|
| Air (surface, lit) | `.` | `  ` (two spaces) | Keep empty space clean |
| Air (underground, dark) | `.` → ` ` (VIS-01) | `  ` (two spaces) | Same — empty is empty |
| Water | `~` | `💧` | Universally recognized |
| Soil (dry) | `#` | `🟫` | Brown square = earth |
| Soil (wet) | `%` | `🟤` | Darker brown = wet earth |
| Stone | `@` | `🪨` | Rock emoji |
| Root | `*` | `🌿` | Green plant/vegetation |
| Seed (dormant) | `s` | `🌰` | Seed/nut |
| Seed (growing) | `S` | `🌱` | Sprouting |

### Width Considerations

Emoji are typically 2 columns wide in monospace terminals. The dev must:
- Account for double-width characters in column spacing
- Adjust axis labels accordingly
- Ensure TUI `ratatui` rendering handles multi-byte characters
- Provide a `--ascii` fallback flag for terminals that don't render emoji well

### Snapshot Requirement

The PR must include:
1. A "before" snapshot: text output of `view --z 15` with old ASCII rendering
2. An "after" snapshot: same world state with new emoji rendering
3. These go in the PR description body for visual comparison

## Risks

- **Terminal compatibility:** Some terminals (especially older ones, Windows cmd) render emoji inconsistently. Mitigated by `--ascii` fallback.
- **Grid width doubles:** Each cell is now 2 columns instead of 1. A 60-cell row becomes 120 columns. This fits most modern terminals (120+ cols) but may clip on narrow ones. Accept this for now.
- **TUI emoji rendering:** ratatui handles Unicode but emoji width calculation varies. Test in at least 2 terminals.

## What This Does NOT Include

- Color changes (TUI already has color via `voxel_style`)
- New materials or visual states
- Changes to `inspect` or `status` output

## Scope Check

This is a visual readability improvement (P1) that directly addresses "beauty/readability" scores of 2-3/5 across sessions. It does not expand MVP scope — same materials, same states, just richer characters.

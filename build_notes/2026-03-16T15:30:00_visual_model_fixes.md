# Build Notes — Visual Model Fixes

**Date:** 2026-03-16T15:30:00
**Focus:** Fix amber foliage bug, boost model visibility and species distinction

## What Changed

### 1. Fixed amber foliage bug (root cause of "everything looks golden")

**File:** `crates/groundwork-web/src/rendering/foliage.ts`

The stress tint system read the water_level byte from leaf voxels to determine health. Leaf voxels in the canopy have water_level=0 (no water above ground), which the code treated as "dead" — applying maximum amber shift to every single leaf in the game. Fixed by treating health=0 as "no data" (healthy) instead of dead.

**Before:** Every leaf got `stressTint = 1.0` → shifted toward amber/brown.
**After:** Leaves with health=0 get `stressTint = 0.0` → render at their species-specific green.

### 2. Wider species foliage color palette

**File:** `crates/groundwork-web/src/rendering/foliage.ts`

Pushed species-specific foliage colors further apart so each species is visually distinct even under warm lighting:
- Oak: deep warm forest green (darker, warmer)
- Birch: bright spring yellow-green (lightest, most distinct)
- Willow: silver-sage with blue tint (cool tone, unique)
- Pine: very dark blue-green (darkest)
- Fern: vibrant emerald (brightest green)
- Others adjusted proportionally

Also increased foliage alpha from 0.92 → 0.96 to reduce warm background bleed-through.

### 3. Gnome scale boost (1.0x → 1.8x)

**File:** `crates/groundwork-web/src/gardener/gardener.ts`

Added `GNOME_BASE_SCALE = 1.8` multiplier. The gnome was 4 units tall in an 80×80 world — invisible at default zoom. Now ~7.2 units tall with the red hat clearly visible among trees.

### 4. Fauna size boost (60-70% increase)

**File:** `crates/groundwork-web/src/rendering/fauna.ts`

- Bee: 1.5 → 2.5 voxels
- Butterfly: 2.0 → 3.5 voxels
- Bird: 3.0 → 5.0 voxels
- Worm: 1.0 → 1.8 voxels
- Beetle: 1.2 → 2.0 voxels

Creatures are now visible as "ecological actors" at default camera zoom.

## Build Verification

- `cargo fmt --check`: pass
- `npx tsc --noEmit`: pass (0 errors)
- `cargo clippy`: pass
- `cargo test -p groundwork-sim`: 5/5 pass
- Screenshots verified across all 4 day cycle phases

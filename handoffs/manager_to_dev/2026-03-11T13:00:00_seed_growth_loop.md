# Manager → Dev Handoff: Seed Growth Loop

**Date:** 2026-03-11T13:00:00
**Context:** Three player sessions completed. CLI usability sprint is done (PR #3). All feedback converges: water is fun, nothing grows. This sprint delivers the core loop.

## Goal

Player places a seed on wet, lit soil → ticks → seed grows into a root. First proof of the "one more seed" fantasy.

## Why Now

This is the single most requested feature across all three player sessions. Without growth, we have a water physics sandbox, not a garden builder. Everything else is polish until this works.

## Tasks (in order)

### Task 1: SIM-01 — Fix material placement state bleed
**Priority:** P0 | **Risk:** low | **Time:** ~15 min

In `crates/groundwork-tui/src/cli.rs`, `cmd_place` (line ~195) sets `voxel.material = mat` but doesn't reset the other fields. Old water/light/nutrient values persist.

**Fix:** After setting material, reset all levels to 0. Then apply material-specific defaults (Water gets water_level=255, which is already handled).

```rust
voxel.material = mat;
voxel.water_level = 0;
voxel.light_level = 0;
voxel.nutrient_level = 0;
if mat == Material::Water {
    voxel.water_level = 255;
}
```

**Acceptance:** Place stone on wet soil → inspect → water_level is 0.

### Task 2: SIM-02 — Fix light attenuation through soil
**Priority:** P0 | **Risk:** low | **Time:** ~15 min

In `crates/groundwork-sim/src/systems.rs`, `light_propagation` (line ~92-121), light is assigned *then* attenuated. This means the first solid layer below air gets nearly full light, and soil attenuates by 60 which is fine — but the assignment order means surface soil gets sky-level light.

**Current behavior (wrong):**
- Z=29 (air): light=255, then attenuate by 2 → light becomes 253
- Z=16 (surface soil): light=253 (assigned!), then attenuate by 60 → light becomes 193
- Z=15 (soil): light=193 (still high!), then attenuate by 60 → 133

Actually looking at the code more carefully: `cell.light_level = light` happens *before* the match. So surface soil gets the full incoming light. The issue is that soil attenuation of 60 is actually creating a gradient — but players reported Z=14 has light=154 which is high but not zero.

**The real issue is:** The player expects underground soil to be dark, but soil only attenuates by 60 per layer. After 4 layers of soil you still have light ~15. This is actually a tuning issue more than a logic bug.

**Recommended fix:** Attenuate *before* assigning for opaque materials (soil, stone, root). This means solid materials get the attenuated value, not the incoming value.

```rust
match cell.material {
    Material::Air => {
        cell.light_level = light;
        light = light.saturating_sub(2);
    }
    Material::Water => {
        cell.light_level = light;
        light = light.saturating_sub(15);
    }
    Material::Soil => {
        light = light.saturating_sub(40);
        cell.light_level = light;
    }
    Material::Root => {
        light = light.saturating_sub(40);
        cell.light_level = light;
    }
    Material::Stone => {
        light = 0;
        cell.light_level = 0;
    }
}
```

This gives: air=full light, surface soil=~213, next soil=~173, next=~133, etc. 5-6 layers of soil before it's dark. Stone blocks completely. This creates a meaningful "plant near the surface or near light shafts" decision.

**Acceptance:** Inspect surface soil → light ~200-215. Inspect 3 layers down in solid soil → light ~90-130. Stone → light 0.

### Task 3: GAME-01 — Add Seed material + growth system
**Priority:** P0 | **Risk:** medium | **Time:** ~45 min

This is the main event. Steps:

**3a. Add Seed material** in `crates/groundwork-sim/src/voxel.rs`:
- Add `Seed = 5` to the Material enum
- Update `from_u8`, `name`, `from_name` (name: "seed")
- Update material tests

**3b. Add seed ASCII char** in `crates/groundwork-tui/src/cli.rs`:
- Add `Material::Seed => 's'` to `voxel_char`
- Update `count_materials` array size from 5 to 6

**3c. Update render.rs** in `crates/groundwork-tui/src/render.rs`:
- Add Seed char to the TUI renderer

**3d. Update save.rs** — Material::from_u8 already handles unknown values by returning None. New material value 5 needs to be recognized. Existing saves with no seeds will load fine.

**3e. Add legend** — Update legend string in cli.rs to include `s seed`.

**3f. Add seed_growth system** in `crates/groundwork-sim/src/systems.rs`:

```rust
pub fn seed_growth(mut grid: ResMut<VoxelGrid>) {
    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                let idx = VoxelGrid::index(x, y, z);
                let cell = &grid.cells()[idx];
                if cell.material != Material::Seed {
                    continue;
                }
                // Check growth conditions
                if cell.water_level >= 30 && cell.light_level >= 30 {
                    // Use nutrient_level as growth counter
                    let growth = cell.nutrient_level;
                    if growth >= 200 {
                        // Seed matures into Root
                        if let Some(cell) = grid.get_mut(x, y, z) {
                            cell.material = Material::Root;
                            cell.nutrient_level = 0;
                        }
                    } else {
                        if let Some(cell) = grid.get_mut(x, y, z) {
                            cell.nutrient_level = cell.nutrient_level.saturating_add(5);
                        }
                    }
                }
            }
        }
    }
}
```

**3g. Register system** in `crates/groundwork-sim/src/lib.rs`:
- Add `seed_growth` to `create_schedule()`, after `soil_absorption` and before any future systems.

**3h. Seed needs water from adjacent soil.** When a seed is placed on soil, it won't have water_level itself unless we add absorption. Simple approach: seed_growth also checks adjacent voxels for water. OR: make seeds absorb water like soil does (add Seed to the soil_absorption system's material check). The simpler path: just check if any adjacent cell has water_level >= 30.

Revised growth condition:
```rust
let has_water = cell.water_level >= 30 || has_adjacent_water(&grid, x, y, z, 30);
let has_light = cell.light_level >= 30;
if has_water && has_light { /* grow */ }
```

**3i. Add test** in `crates/groundwork-sim/src/systems.rs` (or a new test file):
- Create world, place soil with water, place seed above it with light, tick 50 times, verify seed became Root.

**Acceptance:**
1. `place seed 30 30 16` works
2. `view --z 16` shows `s` at position
3. After enough ticks near water + light, seed becomes `*` (root)
4. `cargo test -p groundwork-sim` passes with new test

## Risks / Constraints

- Don't add multiple species yet — one seed type is enough
- Don't add nutrient generation — repurpose nutrient_level as growth counter for now
- Don't add root spreading/branching — just seed→root conversion
- Keep the growth threshold (200) and rate (5/tick) as constants, not config. Tuning comes from playtesting.
- Run `cargo test -p groundwork-sim` and `cargo check --workspace` after

## Open Questions

1. Should seeds absorb water from soil directly, or should we extend soil_absorption to include seeds? Either works. Dev's call — pick whichever is simpler.
2. What happens if a seed is in darkness and wet? It should just not grow (stall at current nutrient_level). Don't kill it.
3. Should growth counter (nutrient_level) reset if conditions become unfavorable? No — keep it simple. Growth pauses but doesn't regress.

## Source Files

- Backlog: `backlog/current.md`
- Player feedback: `feedback/2026-03-11T12:00:00_first_play_session.md`, `feedback/2026-03-11T12:30:00_underground_and_edge_cases.md`
- Sim systems: `crates/groundwork-sim/src/systems.rs`
- Voxel types: `crates/groundwork-sim/src/voxel.rs`
- CLI: `crates/groundwork-tui/src/cli.rs`

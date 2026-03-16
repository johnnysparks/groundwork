//! Actor-view observations — what the player sees.
//!
//! Observations are the text output a human player would get from CLI commands.
//! The actor (scenario or LLM) only sees these; it cannot peek at raw grid state.

use serde::{Deserialize, Serialize};

use groundwork_sim::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};
use groundwork_sim::soil::SoilGrid;
use groundwork_sim::voxel::Material;
use groundwork_sim::Tick;

use bevy_ecs::prelude::*;

/// What the actor sees after an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Human-readable text output (like CLI output).
    pub text: String,
    /// The tick count when this observation was made.
    pub tick: u64,
}

/// Generate a status observation (mirrors `cmd_status`).
pub fn observe_status(world: &World) -> Observation {
    let grid = world.resource::<VoxelGrid>();
    let tick = world.resource::<Tick>().0;

    let mut counts = [0u64; 10];
    let mut wet_soil = 0u64;
    for v in grid.cells() {
        counts[v.material.as_u8() as usize] += 1;
        if v.material == Material::Soil && v.water_level > 50 {
            wet_soil += 1;
        }
    }

    let names = [
        "air", "soil", "stone", "water", "root", "seed", "trunk", "branch", "leaf", "deadwood",
    ];
    let mut lines = vec![
        format!("Tick: {tick}"),
        format!(
            "Grid: {GRID_X}x{GRID_Y}x{GRID_Z} ({} voxels)",
            GRID_X * GRID_Y * GRID_Z
        ),
        "Materials:".to_string(),
    ];
    for i in 0..10 {
        lines.push(format!("  {}: {}", names[i], counts[i]));
    }
    lines.push(format!("  wet soil: {wet_soil}"));

    Observation {
        text: lines.join("\n"),
        tick,
    }
}

/// Generate an inspect observation for a single voxel (mirrors `cmd_inspect`).
pub fn observe_inspect(world: &World, x: usize, y: usize, z: usize) -> Observation {
    let grid = world.resource::<VoxelGrid>();
    let tick = world.resource::<Tick>().0;

    let Some(voxel) = grid.get(x, y, z) else {
        return Observation {
            text: format!("out of bounds: ({x}, {y}, {z})"),
            tick,
        };
    };

    let depth_label = if z > GROUND_LEVEL {
        format!("above +{}", z - GROUND_LEVEL)
    } else if z == GROUND_LEVEL {
        "surface".to_string()
    } else {
        format!("below -{}", GROUND_LEVEL - z)
    };

    let mut lines = vec![
        format!("Voxel at ({x}, {y}, {z}) [{depth_label}]:"),
        format!("  material: {}", voxel.material.name()),
        format!("  water_level: {}/255", voxel.water_level),
        format!("  light_level: {}/255", voxel.light_level),
        format!("  nutrient_level: {}/255", voxel.nutrient_level),
    ];

    if voxel.material == Material::Soil {
        let soil_grid = world.resource::<SoilGrid>();
        if let Some(comp) = soil_grid.get(x, y, z) {
            lines.push(String::new());
            lines.push(format!("  soil type: {}", comp.type_name()));
            lines.push(format!(
                "  sand: {:>3}  clay: {:>3}  organic: {:>3}",
                comp.sand, comp.clay, comp.organic
            ));
        }
    }

    Observation {
        text: lines.join("\n"),
        tick,
    }
}

/// Generate a view observation for a Z-slice.
pub fn observe_view(world: &World, z: usize) -> Observation {
    let grid = world.resource::<VoxelGrid>();
    let tick = world.resource::<Tick>().0;

    if z >= GRID_Z {
        return Observation {
            text: format!("Z={z} out of bounds (max {})", GRID_Z - 1),
            tick,
        };
    }

    let depth_label = if z > GROUND_LEVEL {
        format!("above +{}", z - GROUND_LEVEL)
    } else if z == GROUND_LEVEL {
        "surface".to_string()
    } else {
        format!("below -{}", GROUND_LEVEL - z)
    };

    let mut lines = vec![format!("Z:{z} ({depth_label})  Tick:{tick}")];

    for y in 0..GRID_Y {
        let row: String = (0..GRID_X)
            .map(|x| {
                grid.get(x, y, z)
                    .map(|v| voxel_char(v.material, v.water_level, v.light_level))
                    .unwrap_or(' ')
            })
            .collect();
        lines.push(row);
    }

    Observation {
        text: lines.join("\n"),
        tick,
    }
}

fn voxel_char(mat: Material, water_level: u8, light_level: u8) -> char {
    match mat {
        Material::Air if water_level > 0 => '~',
        Material::Air if light_level == 0 => ' ',
        Material::Air => '.',
        Material::Water => '~',
        Material::Soil if water_level > 50 => '%',
        Material::Soil => '#',
        Material::Stone => '@',
        Material::Root => '*',
        Material::Seed if water_level >= 100 => 'S',
        Material::Seed => 's',
        Material::Trunk => '|',
        Material::Branch => '-',
        Material::Leaf => '&',
        Material::DeadWood => 'X',
    }
}

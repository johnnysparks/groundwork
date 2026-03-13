use std::path::PathBuf;

use groundwork_sim::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};
use groundwork_sim::soil::SoilGrid;
use groundwork_sim::voxel::Material;
use groundwork_sim::{FocusState, ToolState, Tick};

use crate::app::{self, Tool};

const DEFAULT_STATE: &str = "groundwork.state";

fn state_path(args: &[String]) -> PathBuf {
    find_value(args, "--state")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_STATE))
}

fn find_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
}

/// Parse a coordinate argument as either a single value or a range.
/// Accepts: "30" -> [30], "20..40" -> [20, 21, ..., 39] (exclusive end).
fn parse_coord_range(s: &str) -> Result<Vec<usize>, String> {
    if let Some((start_s, end_s)) = s.split_once("..") {
        let start: usize = start_s.parse().map_err(|e| format!("bad range start: {e}"))?;
        let end: usize = end_s.parse().map_err(|e| format!("bad range end: {e}"))?;
        if start >= end {
            return Err(format!("empty range: {start}..{end} (start must be < end)"));
        }
        Ok((start..end).collect())
    } else {
        let v: usize = s.parse().map_err(|e| format!("bad coordinate: {e}"))?;
        Ok(vec![v])
    }
}

fn voxel_char(mat: Material, water_level: u8, light_level: u8, nutrient_level: u8) -> char {
    match mat {
        Material::Air if water_level > 0 => '~',
        Material::Air if light_level == 0 => ' ',
        Material::Air => '.',
        Material::Water => '~',
        Material::Soil if water_level > 50 => '%',
        Material::Soil => '#',
        Material::Stone => '@',
        Material::Root => '*',
        Material::Seed if nutrient_level >= 100 => 'S',
        Material::Seed => 's',
        Material::Trunk => '|',
        Material::Branch => '-',
        Material::Leaf => '&',
        Material::DeadWood => 'X',
    }
}

pub fn cmd_new(args: &[String]) -> std::io::Result<()> {
    let path = state_path(args);
    let world = groundwork_sim::create_world();
    groundwork_sim::save::save_world(&world, &path)?;
    println!("Created new world at {}", path.display());
    println!("Tick: 0  Grid: {GRID_X}x{GRID_Y}x{GRID_Z}");
    Ok(())
}

fn count_materials(grid: &VoxelGrid) -> ([u64; 10], u64) {
    let mut counts = [0u64; 10];
    let mut wet_soil = 0u64;
    for v in grid.cells() {
        counts[v.material.as_u8() as usize] += 1;
        if v.material == Material::Soil && v.water_level > 50 {
            wet_soil += 1;
        }
    }
    (counts, wet_soil)
}

pub fn cmd_tick(args: &[String]) -> std::io::Result<()> {
    let path = state_path(args);
    let n: u64 = args
        .first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let mut world = groundwork_sim::save::load_world(&path)?;
    let mut schedule = groundwork_sim::create_schedule();

    let (before_counts, before_wet) = count_materials(world.resource::<VoxelGrid>());

    for _ in 0..n {
        groundwork_sim::tick(&mut world, &mut schedule);
    }

    let (after_counts, after_wet) = count_materials(world.resource::<VoxelGrid>());

    groundwork_sim::save::save_world(&world, &path)?;
    let tick = world.resource::<Tick>().0;
    println!("Tick: {tick} (+{n})");

    // Show changes
    let mut changes = Vec::new();
    let names = ["air", "soil", "stone", "water", "root", "seed", "trunk", "branch", "leaf", "deadwood"];
    for i in 0..10 {
        let diff = after_counts[i] as i64 - before_counts[i] as i64;
        if diff != 0 {
            changes.push(format!("{}: {:+}", names[i], diff));
        }
    }
    let wet_diff = after_wet as i64 - before_wet as i64;
    if wet_diff != 0 {
        changes.push(format!("wet soil: {:+}", wet_diff));
    }

    if changes.is_empty() {
        println!("  (no material changes)");
    } else {
        println!("  Changes: {}", changes.join(", "));
    }
    Ok(())
}

pub fn cmd_view(args: &[String]) -> std::io::Result<()> {
    let path = state_path(args);
    let world = groundwork_sim::save::load_world(&path)?;
    let grid = world.resource::<VoxelGrid>();
    let tick = world.resource::<Tick>().0;

    let max_z = GRID_Z - 1;
    let mut z: usize = find_value(args, "--z")
        .and_then(|v| v.parse().ok())
        .unwrap_or(GROUND_LEVEL);

    if z >= GRID_Z {
        eprintln!("Warning: Z={z} is out of bounds (max {max_z}), clamping to {max_z}");
        z = max_z;
    }

    let depth_label = if z > GROUND_LEVEL {
        format!("above +{}", z - GROUND_LEVEL)
    } else if z == GROUND_LEVEL {
        "surface".to_string()
    } else {
        format!("below -{}", GROUND_LEVEL - z)
    };

    println!("Z:{z} ({depth_label})  Tick:{tick}  Grid:{GRID_X}x{GRID_Y}x{GRID_Z}");
    println!();

    print!("    ");
    for x in 0..GRID_X {
        if x % 10 == 0 {
            print!("{:<10}", x);
        }
    }
    println!();

    for y in 0..GRID_Y {
        if y % 10 == 0 {
            print!("{:>3} ", y);
        } else {
            print!("    ");
        }

        let row: String = (0..GRID_X)
            .map(|x| {
                grid.get(x, y, z)
                    .map(|v| voxel_char(v.material, v.water_level, v.light_level, v.nutrient_level))
                    .unwrap_or(' ')
            })
            .collect();
        println!("{row}");
    }

    println!();
    println!("Legend: . air  ~ water  # soil  % wet  @ stone  * root  s seed  S sprouting  | trunk  - branch  & leaf  X dead");
    Ok(())
}

fn parse_tool(name: &str) -> std::io::Result<Tool> {
    Tool::from_material_name(name).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unknown material: {name}. Valid: air/dig, seed, water, soil, stone"),
        )
    })
}

pub fn cmd_place(args: &[String]) -> std::io::Result<()> {
    if args.len() < 4 {
        eprintln!("Usage: groundwork place <material> <x> <y> <z> [--state FILE]");
        eprintln!("  Coordinates accept ranges: place soil 20..40 30 15");
        eprintln!("  Materials: air/dig, seed, water, soil, stone");
        eprintln!("  Items with gravity (seed, water, soil) fall through air.");
        eprintln!("  'air' or 'dig' uses the shovel — removes anything.");
        std::process::exit(1);
    }

    let tool = parse_tool(&args[0])?;

    let xs = parse_coord_range(&args[1]).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("x: {e}"))
    })?;
    let ys = parse_coord_range(&args[2]).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("y: {e}"))
    })?;
    let zs = parse_coord_range(&args[3]).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("z: {e}"))
    })?;

    let path = state_path(&args[4..]);
    let mut world = groundwork_sim::save::load_world(&path)?;

    let mut placed = 0u64;
    let mut skipped = 0u64;
    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        for &z in &zs {
            for &y in &ys {
                for &x in &xs {
                    if VoxelGrid::in_bounds(x, y, z) {
                        if app::apply_tool(&mut grid, tool, x, y, z) {
                            placed += 1;
                        } else {
                            skipped += 1;
                        }
                    } else {
                        skipped += 1;
                    }
                }
            }
        }
    }

    if placed == 0 {
        eprintln!("Nothing placed ({skipped} cells skipped — occupied, protected, or out of bounds)");
        return Ok(());
    }

    groundwork_sim::save::save_world(&world, &path)?;

    if placed == 1 && skipped == 0 {
        println!("Used {} at ({}, {}, {})", tool.name(), xs[0], ys[0], zs[0]);
    } else if skipped > 0 {
        println!("Used {} x {} ({} skipped)", tool.name(), placed, skipped);
    } else {
        println!("Used {} x {}", tool.name(), placed);
    }
    Ok(())
}

pub fn cmd_fill(args: &[String]) -> std::io::Result<()> {
    if args.len() < 7 {
        eprintln!("Usage: groundwork fill <material> <x1> <y1> <z1> <x2> <y2> <z2> [--state FILE]");
        eprintln!("  Fills a rectangular region from (x1,y1,z1) to (x2,y2,z2) inclusive.");
        eprintln!("  Materials: air/dig, seed, water, soil, stone");
        eprintln!("  Shovel (air/dig) removes anything. Other tools respect gravity & protection.");
        std::process::exit(1);
    }

    let tool = parse_tool(&args[0])?;

    let parse = |i: usize, label: &str| -> std::io::Result<usize> {
        args[i].parse().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad {label}: {e}"))
        })
    };

    let x1 = parse(1, "x1")?;
    let y1 = parse(2, "y1")?;
    let z1 = parse(3, "z1")?;
    let x2 = parse(4, "x2")?;
    let y2 = parse(5, "y2")?;
    let z2 = parse(6, "z2")?;

    let xlo = x1.min(x2);
    let xhi = x1.max(x2);
    let ylo = y1.min(y2);
    let yhi = y1.max(y2);
    let zlo = z1.min(z2);
    let zhi = z1.max(z2);

    let path = state_path(&args[7..]);
    let mut world = groundwork_sim::save::load_world(&path)?;

    let mut placed = 0u64;
    let mut skipped = 0u64;
    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        for z in zlo..=zhi {
            for y in ylo..=yhi {
                for x in xlo..=xhi {
                    if app::apply_tool(&mut grid, tool, x, y, z) {
                        placed += 1;
                    } else {
                        skipped += 1;
                    }
                }
            }
        }
    }

    if placed == 0 && skipped > 0 {
        eprintln!("Nothing placed ({skipped} cells skipped)");
        return Ok(());
    }

    groundwork_sim::save::save_world(&world, &path)?;

    let mut msg = format!(
        "Used {} x {} from ({},{},{}) to ({},{},{})",
        tool.name(), placed, xlo, ylo, zlo, xhi, yhi, zhi
    );
    if skipped > 0 {
        msg.push_str(&format!(" ({skipped} skipped)"));
    }
    println!("{msg}");
    Ok(())
}

pub fn cmd_inspect(args: &[String]) -> std::io::Result<()> {
    let has_coords = args.len() >= 3 && args[0].parse::<usize>().is_ok();

    let (x, y, z, remaining_args);
    if has_coords {
        x = args[0].parse().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad x: {e}"))
        })?;
        y = args[1].parse().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad y: {e}"))
        })?;
        z = args[2].parse().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad z: {e}"))
        })?;
        remaining_args = &args[3..];
    } else {
        remaining_args = args;
        x = 0; y = 0; z = 0;
    }

    let path = state_path(remaining_args);
    let world = groundwork_sim::save::load_world(&path)?;

    let (x, y, z) = if has_coords {
        (x, y, z)
    } else {
        let focus = world.resource::<FocusState>();
        println!("(inspecting at focus: {}, {}, {})", focus.x, focus.y, focus.z);
        (focus.x, focus.y, focus.z)
    };

    let grid = world.resource::<VoxelGrid>();

    let voxel = grid.get(x, y, z).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("out of bounds: ({x}, {y}, {z})"),
        )
    })?;

    let depth_label = if z > GROUND_LEVEL {
        format!("above +{}", z - GROUND_LEVEL)
    } else if z == GROUND_LEVEL {
        "surface".to_string()
    } else {
        format!("below -{}", GROUND_LEVEL - z)
    };

    println!("Voxel at ({x}, {y}, {z}) [{}]:", depth_label);
    println!("  material: {}", voxel.material.name());
    println!("  water_level: {}/255", voxel.water_level);
    println!("  light_level: {}/255", voxel.light_level);
    println!("  nutrient_level: {}/255", voxel.nutrient_level);

    // Soil composition diagnostics
    if voxel.material == Material::Soil {
        let soil_grid = world.resource::<SoilGrid>();
        if let Some(comp) = soil_grid.get(x, y, z) {
            println!();
            println!("  soil type: {}", comp.type_name());
            println!("  sand: {:>3}  clay: {:>3}  organic: {:>3}", comp.sand, comp.clay, comp.organic);
            println!("  rock: {:>3}  pH: {:.1}  bacteria: {:>3}", comp.rock, comp.ph_value(), comp.bacteria);
            println!("  drainage: {:>3}  retention: {:>3}  nutrients: {:>3}",
                comp.drainage_rate(), comp.water_retention(), comp.nutrient_capacity());
            if comp.is_compacted() {
                println!("  WARNING: compacted — blocks root growth");
            }
        }
    }

    // Seed growth diagnostics
    if voxel.material == Material::Seed {
        let growth = voxel.nutrient_level;
        let growth_max = 200u16;
        let pct = (growth as u16 * 100) / growth_max;
        println!();
        println!("  growth: {growth}/{growth_max} ({pct}%)");

        let mut has_water = voxel.water_level >= 30;
        let mut water_source = String::new();
        if has_water {
            water_source = format!("own: {}", voxel.water_level);
        } else {
            let neighbor_dirs: [(isize, isize, isize, &str); 6] = [
                (-1, 0, 0, "x-1"),
                (1, 0, 0, "x+1"),
                (0, -1, 0, "y-1"),
                (0, 1, 0, "y+1"),
                (0, 0, -1, "below"),
                (0, 0, 1, "above"),
            ];
            for (dx, dy, dz, label) in neighbor_dirs {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                let nz = z as isize + dz;
                if nx < 0 || ny < 0 || nz < 0 {
                    continue;
                }
                let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                if let Some(nv) = grid.get(nx, ny, nz) {
                    if nv.water_level >= 30 {
                        has_water = true;
                        water_source = format!("neighbor {label}: {}/255", nv.water_level);
                        break;
                    }
                }
            }
        }

        if has_water {
            println!("  water: YES ({water_source})");
        } else {
            println!("  water: NO — need adjacent water_level >= 30");
        }

        let has_light = voxel.light_level >= 30;
        if has_light {
            println!("  light: YES ({}/255)", voxel.light_level);
        } else {
            println!("  light: NO — need light_level >= 30");
        }

        if has_water && has_light {
            let remaining = growth_max.saturating_sub(growth as u16);
            let ticks_left = (remaining + 4) / 5;
            println!("  status: growing (+5/tick, ~{ticks_left} ticks to root)");
        } else {
            let mut missing = Vec::new();
            if !has_water {
                missing.push("no water nearby");
            }
            if !has_light {
                missing.push("no light");
            }
            println!("  status: dormant — {}", missing.join(", "));
        }
    }

    Ok(())
}

pub fn cmd_status(args: &[String]) -> std::io::Result<()> {
    let path = state_path(args);
    let world = groundwork_sim::save::load_world(&path)?;
    let grid = world.resource::<VoxelGrid>();
    let tick = world.resource::<Tick>().0;

    let (counts, wet_soil) = count_materials(grid);

    println!("Tick: {tick}");
    println!("Grid: {GRID_X}x{GRID_Y}x{GRID_Z} ({} voxels)", GRID_X * GRID_Y * GRID_Z);
    println!("Materials:");
    for i in 0..10 {
        if let Some(mat) = Material::from_u8(i) {
            println!("  {}: {}", mat.name(), counts[i as usize]);
        }
    }
    println!("  wet soil: {}", wet_soil);

    // Soil composition summary
    let soil_grid = world.resource::<SoilGrid>();
    let mut type_counts: std::collections::HashMap<&str, u64> = std::collections::HashMap::new();
    for z in 0..GRID_Z {
        for y in 0..GRID_Y {
            for x in 0..GRID_X {
                if let Some(v) = grid.get(x, y, z) {
                    if v.material == Material::Soil {
                        if let Some(comp) = soil_grid.get(x, y, z) {
                            *type_counts.entry(comp.type_name()).or_default() += 1;
                        }
                    }
                }
            }
        }
    }
    if !type_counts.is_empty() {
        println!("Soil types:");
        let mut sorted: Vec<_> = type_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        for (name, count) in sorted {
            println!("  {}: {}", name, count);
        }
    }
    Ok(())
}

pub fn cmd_focus(args: &[String]) -> std::io::Result<()> {
    let path = state_path(args);
    let has_coords = args.len() >= 3 && args[0].parse::<usize>().is_ok();

    if !has_coords {
        let world = groundwork_sim::save::load_world(&path)?;
        let focus = world.resource::<FocusState>();
        let grid = world.resource::<VoxelGrid>();

        let depth_label = if focus.z > GROUND_LEVEL {
            format!("above +{}", focus.z - GROUND_LEVEL)
        } else if focus.z == GROUND_LEVEL {
            "surface".to_string()
        } else {
            format!("below -{}", GROUND_LEVEL - focus.z)
        };

        println!("Focus: ({}, {}, {}) [{}]", focus.x, focus.y, focus.z, depth_label);
        if let Some(v) = grid.get(focus.x, focus.y, focus.z) {
            println!("  material: {}", v.material.name());
            println!("  water: {}/255  light: {}/255  nutrient: {}/255",
                v.water_level, v.light_level, v.nutrient_level);
        }
        if let Some(ref tool) = focus.tool {
            println!("  tool: {} from ({}, {}, {})",
                tool.material.name(), tool.start_x, tool.start_y, tool.start_z);
        }
        return Ok(());
    }

    let x: usize = args[0].parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad x: {e}"))
    })?;
    let y: usize = args[1].parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad y: {e}"))
    })?;
    let z: usize = args[2].parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad z: {e}"))
    })?;

    if !VoxelGrid::in_bounds(x, y, z) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("out of bounds: ({x}, {y}, {z}) — max ({}, {}, {})", GRID_X - 1, GRID_Y - 1, GRID_Z - 1),
        ));
    }

    let path = state_path(&args[3..]);
    let mut world = groundwork_sim::save::load_world(&path)?;
    {
        let mut focus = world.resource_mut::<FocusState>();
        focus.x = x;
        focus.y = y;
        focus.z = z;
    }
    groundwork_sim::save::save_world(&world, &path)?;

    let grid = world.resource::<VoxelGrid>();
    let v = grid.get(x, y, z).unwrap();
    println!("Focus set to ({x}, {y}, {z}) — {}", v.material.name());
    Ok(())
}

pub fn cmd_tool_start(args: &[String]) -> std::io::Result<()> {
    if args.is_empty() {
        eprintln!("Usage: groundwork tool-start <material> [--state FILE]");
        eprintln!("  Begins a range operation at the current focus position.");
        eprintln!("  Move focus, then use tool-end to apply.");
        eprintln!("  Materials: air/dig, seed, water, soil, stone");
        std::process::exit(1);
    }

    let tool = parse_tool(&args[0])?;
    let mat = tool.material();
    let path = state_path(&args[1..]);
    let mut world = groundwork_sim::save::load_world(&path)?;

    let (sx, sy, sz);
    {
        let mut focus = world.resource_mut::<FocusState>();
        sx = focus.x;
        sy = focus.y;
        sz = focus.z;
        focus.tool = Some(ToolState {
            material: mat,
            start_x: sx,
            start_y: sy,
            start_z: sz,
        });
    }

    groundwork_sim::save::save_world(&world, &path)?;
    println!("Tool started: {} from ({sx}, {sy}, {sz})", tool.name());
    println!("  Move focus with `groundwork focus <x> <y> <z>`, then `groundwork tool-end` to apply.");
    Ok(())
}

pub fn cmd_tool_end(args: &[String]) -> std::io::Result<()> {
    let path = state_path(args);
    let mut world = groundwork_sim::save::load_world(&path)?;

    let (mat, sx, sy, sz, ex, ey, ez);
    {
        let focus = world.resource::<FocusState>();
        let tool_state = focus.tool.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "no tool in progress — use `groundwork tool-start <material>` first",
            )
        })?;
        mat = tool_state.material;
        sx = tool_state.start_x;
        sy = tool_state.start_y;
        sz = tool_state.start_z;
        ex = focus.x;
        ey = focus.y;
        ez = focus.z;
    }

    let tool = Tool::from_material_name(mat.name()).unwrap_or(Tool::Shovel);

    let xlo = sx.min(ex);
    let xhi = sx.max(ex);
    let ylo = sy.min(ey);
    let yhi = sy.max(ey);
    let zlo = sz.min(ez);
    let zhi = sz.max(ez);

    let mut placed = 0u64;
    let mut skipped = 0u64;
    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        for z in zlo..=zhi {
            for y in ylo..=yhi {
                for x in xlo..=xhi {
                    if app::apply_tool(&mut grid, tool, x, y, z) {
                        placed += 1;
                    } else {
                        skipped += 1;
                    }
                }
            }
        }
    }

    // Clear tool state
    {
        let mut focus = world.resource_mut::<FocusState>();
        focus.tool = None;
    }

    groundwork_sim::save::save_world(&world, &path)?;

    let mut msg = format!(
        "Tool applied: {} x {} from ({},{},{}) to ({},{},{})",
        tool.name(), placed, xlo, ylo, zlo, xhi, yhi, zhi
    );
    if skipped > 0 {
        msg.push_str(&format!(" ({skipped} skipped)"));
    }
    println!("{msg}");
    Ok(())
}

pub fn print_help() {
    println!("GROUNDWORK - ecological voxel garden builder");
    println!();
    println!("Usage: groundwork <command> [args]");
    println!();
    println!("Commands:");
    println!("  new                           Create a new world");
    println!("  tick [N]                      Advance N ticks (default 1)");
    println!("  view [--z Z]                  Print a Z-slice of the grid");
    println!("  place <mat> <x> <y> <z>       Use a tool at coordinates");
    println!("    Coordinates accept ranges:  place soil 20..40 30 15");
    println!("    'air' or 'dig' uses the shovel to remove anything.");
    println!("    Seed/water/soil fall through air (gravity).");
    println!("    Seeds die on stone.");
    println!("  fill <mat> <x1> <y1> <z1> <x2> <y2> <z2>");
    println!("                                Fill a rectangular region");
    println!("  inspect [<x> <y> <z>]         Show voxel details (uses focus if no coords)");
    println!("  status                        Show world summary");
    println!();
    println!("Focus & Tool:");
    println!("  focus [<x> <y> <z>]           Get/set the focus cursor position");
    println!("  tool-start <material>         Begin a range operation at current focus");
    println!("  tool-end                      Apply the tool from start to current focus");
    println!();
    println!("  tui                           Launch interactive terminal UI");
    println!("  help                          Show this help");
    println!();
    println!("Materials: air/dig, seed, water, soil, stone");
    println!("  air/dig = shovel (removes anything)");
    println!("  seed    = seed bag (falls, dies on stone)");
    println!("  water   = watering can (falls, no-op on water)");
    println!("  soil    = soil (falls through air)");
    println!("  stone   = stone (placed directly)");
    println!();
    println!("Options:");
    println!("  --state FILE                  State file (default: groundwork.state)");
}

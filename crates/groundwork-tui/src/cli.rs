use std::path::PathBuf;

use groundwork_sim::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL};
use groundwork_sim::voxel::Material;
use groundwork_sim::Tick;

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
/// Accepts: "30" → [30], "20..40" → [20, 21, ..., 39] (exclusive end).
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
    }
}

/// Return (char, ANSI color escape) for colored CLI output.
/// `below_ground` adjusts root/air presentation for underground layers.
fn voxel_colored(mat: Material, water_level: u8, light_level: u8, nutrient_level: u8, below_ground: bool) -> (char, &'static str) {
    // ANSI 256-color or RGB escape sequences for each material.
    // Using \x1b[38;2;R;G;Bm for true-color foreground.
    match mat {
        Material::Air if water_level > 0 => ('~', "\x1b[38;2;100;160;255m"),   // blue mist
        Material::Air if below_ground    => (' ', "\x1b[0m"),                     // dark void
        Material::Air if light_level == 0 => (' ', "\x1b[0m"),
        Material::Air                    => ('.', "\x1b[38;2;60;60;70m"),        // dim air
        Material::Water => {
            let _intensity = 140 + (water_level as u16 * 115 / 255) as u8;
            // Use a fixed bright blue — terminal can't interpolate per-cell easily
            ('~', "\x1b[38;2;40;120;255m")                                       // blue water
        }
        Material::Soil if water_level > 50 => ('%', "\x1b[38;2;66;40;14m"),     // wet soil — dark brown
        Material::Soil if water_level > 0  => ('#', "\x1b[38;2;101;67;33m"),    // moist soil — medium brown
        Material::Soil                     => ('#', "\x1b[38;2;160;110;60m"),   // dry soil — warm brown
        Material::Stone => ('@', "\x1b[38;2;150;150;155m"),                      // gray rock
        Material::Root if below_ground => ('*', "\x1b[38;2;210;180;140m"),       // tan root underground
        Material::Root                 => ('*', "\x1b[38;2;60;180;60m"),         // green root above ground
        Material::Seed if nutrient_level >= 100 => ('S', "\x1b[38;2;80;220;60m"), // sprouting — green
        Material::Seed => ('s', "\x1b[38;2;200;170;60m"),                        // dormant — yellow-brown
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

fn count_materials(grid: &VoxelGrid) -> ([u64; 6], u64) {
    let mut counts = [0u64; 6];
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
    let names = ["air", "soil", "stone", "water", "root", "seed"];
    for i in 0..6 {
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

    let plain = has_flag(args, "--plain");
    let ascii = has_flag(args, "--ascii");

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

    let below_ground = z < GROUND_LEVEL;

    if plain || ascii {
        // Plain ASCII mode — single-char, no color
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
        println!("Legend: . air  (space) dark air  ~ water  # soil  % wet soil  @ stone  * root  s seed  S growing seed");
    } else {
        // Colored ASCII mode (default) — single-char with ANSI true-color
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

            for x in 0..GRID_X {
                if let Some(v) = grid.get(x, y, z) {
                    let (ch, color) = voxel_colored(v.material, v.water_level, v.light_level, v.nutrient_level, below_ground);
                    print!("{color}{ch}");
                } else {
                    print!(" ");
                }
            }
            print!("\x1b[0m"); // reset color at end of row
            println!();
        }

        println!();
        println!(
            "Legend: \x1b[38;2;60;60;70m.\x1b[0m air  \
             \x1b[38;2;40;120;255m~\x1b[0m water  \
             \x1b[38;2;160;110;60m#\x1b[0m soil  \
             \x1b[38;2;66;40;14m%\x1b[0m wet soil  \
             \x1b[38;2;150;150;155m@\x1b[0m stone  \
             \x1b[38;2;210;180;140m*\x1b[0m root  \
             \x1b[38;2;200;170;60ms\x1b[0m seed  \
             \x1b[38;2;80;220;60mS\x1b[0m sprouting"
        );
    }
    Ok(())
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

pub fn cmd_place(args: &[String]) -> std::io::Result<()> {
    if args.len() < 4 {
        eprintln!("Usage: groundwork place <material> <x> <y> <z> [--force] [--state FILE]");
        eprintln!("  Coordinates accept ranges: place soil 20..40 30 15");
        eprintln!("Materials: air, soil, stone, water, root, seed");
        std::process::exit(1);
    }

    let mat_name = &args[0];
    let mat = Material::from_name(mat_name).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unknown material: {mat_name}. Valid: air, soil, stone, water, root, seed"),
        )
    })?;

    let xs = parse_coord_range(&args[1]).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("x: {e}"))
    })?;
    let ys = parse_coord_range(&args[2]).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("y: {e}"))
    })?;
    let zs = parse_coord_range(&args[3]).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("z: {e}"))
    })?;

    let force = has_flag(&args[4..], "--force");
    let path = state_path(&args[4..]);
    let mut world = groundwork_sim::save::load_world(&path)?;

    let mut placed = 0u64;
    let mut skipped = 0u64;
    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        for &z in &zs {
            for &y in &ys {
                for &x in &xs {
                    if let Some(voxel) = grid.get_mut(x, y, z) {
                        let existing = voxel.material;

                        // Protect seeds and roots from accidental overwriting
                        if !force && (existing == Material::Seed || existing == Material::Root) {
                            eprintln!(
                                "Error: cannot overwrite {} at ({},{},{}). Use --force to override.",
                                existing.name(),
                                x,
                                y,
                                z
                            );
                            std::process::exit(1);
                        }

                        // Warn when overwriting water (but still execute)
                        if !force && existing == Material::Water {
                            eprintln!(
                                "Warning: overwriting water at ({},{},{}). Use --force to skip this warning.",
                                x, y, z
                            );
                        }

                        voxel.set_material(mat);
                        placed += 1;
                    } else {
                        skipped += 1;
                    }
                }
            }
        }
    }

    if placed == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "all coordinates out of bounds",
        ));
    }

    groundwork_sim::save::save_world(&world, &path)?;

    if placed == 1 && skipped == 0 {
        println!("Placed {} at ({}, {}, {})", mat.name(), xs[0], ys[0], zs[0]);
    } else if skipped > 0 {
        println!("Placed {} × {} ({} out of bounds, skipped)", placed, mat.name(), skipped);
    } else {
        println!("Placed {} × {}", placed, mat.name());
    }
    Ok(())
}

pub fn cmd_fill(args: &[String]) -> std::io::Result<()> {
    if args.len() < 7 {
        eprintln!("Usage: groundwork fill <material> <x1> <y1> <z1> <x2> <y2> <z2> [--force] [--state FILE]");
        eprintln!("  Fills a rectangular region from (x1,y1,z1) to (x2,y2,z2) inclusive.");
        eprintln!("  Seeds and roots are protected; use --force to override.");
        eprintln!("Materials: air, soil, stone, water, root, seed");
        std::process::exit(1);
    }

    let mat_name = &args[0];
    let mat = Material::from_name(mat_name).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unknown material: {mat_name}. Valid: air, soil, stone, water, root, seed"),
        )
    })?;

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

    let force = has_flag(&args[7..], "--force");
    let path = state_path(&args[7..]);
    let mut world = groundwork_sim::save::load_world(&path)?;

    let mut placed = 0u64;
    let mut skipped = 0u64;
    let mut protected = 0u64;
    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        for z in zlo..=zhi {
            for y in ylo..=yhi {
                for x in xlo..=xhi {
                    if let Some(voxel) = grid.get_mut(x, y, z) {
                        let existing = voxel.material;

                        // Protect seeds and roots from accidental overwriting
                        if !force && (existing == Material::Seed || existing == Material::Root) {
                            protected += 1;
                            continue;
                        }

                        voxel.set_material(mat);
                        placed += 1;
                    } else {
                        skipped += 1;
                    }
                }
            }
        }
    }

    if placed == 0 && protected == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "all coordinates out of bounds",
        ));
    }

    groundwork_sim::save::save_world(&world, &path)?;

    let mut msg = format!(
        "Filled {} × {} from ({},{},{}) to ({},{},{})",
        placed, mat.name(), xlo, ylo, zlo, xhi, yhi, zhi
    );
    if skipped > 0 {
        msg.push_str(&format!(" ({skipped} out of bounds, skipped)"));
    }
    if protected > 0 {
        msg.push_str(&format!(" ({protected} protected cells skipped)"));
    }
    println!("{msg}");
    Ok(())
}

pub fn cmd_inspect(args: &[String]) -> std::io::Result<()> {
    if args.len() < 3 {
        eprintln!("Usage: groundwork inspect <x> <y> <z> [--state FILE]");
        std::process::exit(1);
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

    let path = state_path(&args[3..]);
    let world = groundwork_sim::save::load_world(&path)?;
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

    // Seed growth diagnostics
    if voxel.material == Material::Seed {
        let growth = voxel.nutrient_level;
        let growth_max = 200u16;
        let pct = (growth as u16 * 100) / growth_max;
        println!();
        println!("  growth: {growth}/{growth_max} ({pct}%)");

        // Check water condition: own water or any neighbor with water_level >= 30
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

        // Check light condition
        let has_light = voxel.light_level >= 30;
        if has_light {
            println!("  light: YES ({}/255)", voxel.light_level);
        } else {
            println!("  light: NO — need light_level >= 30");
        }

        // Status summary
        if has_water && has_light {
            let remaining = growth_max.saturating_sub(growth as u16);
            let ticks_left = (remaining + 4) / 5; // ceiling division
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
    for i in 0..6 {
        if let Some(mat) = Material::from_u8(i) {
            println!("  {}: {}", mat.name(), counts[i as usize]);
        }
    }
    println!("  wet soil: {}", wet_soil);
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
    println!("  view [--z Z] [--plain]         Print slice (colored ASCII default, --plain for uncolored)");
    println!("  place <mat> <x> <y> <z>       Place a voxel (air/soil/stone/water/root/seed)");
    println!("    Coordinates accept ranges:  place soil 20..40 30 15");
    println!("    Rejects overwriting seeds/roots (use --force to override)");
    println!("  fill <mat> <x1> <y1> <z1> <x2> <y2> <z2>");
    println!("                                Fill a rectangular region (inclusive)");
    println!("    Skips seeds/roots by default (use --force to override)");
    println!("  inspect <x> <y> <z>           Show voxel details");
    println!("  status                        Show world summary");
    println!("  tui                           Launch interactive terminal UI");
    println!("  help                          Show this help");
    println!();
    println!("Options:");
    println!("  --state FILE                  State file (default: groundwork.state)");
}

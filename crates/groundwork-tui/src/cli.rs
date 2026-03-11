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

fn voxel_char(mat: Material, water_level: u8) -> char {
    match mat {
        Material::Air if water_level > 0 => '~',
        Material::Air => '.',
        Material::Water => '~',
        Material::Soil if water_level > 100 => '%',
        Material::Soil => '#',
        Material::Stone => '@',
        Material::Root => '*',
        Material::Seed => 's',
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
        if v.material == Material::Soil && v.water_level > 100 {
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

    let max_z = GRID_Z - 1;
    let mut z: usize = find_value(args, "--z")
        .and_then(|v| v.parse().ok())
        .unwrap_or(GROUND_LEVEL + 1);

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

    // X-axis labels (every 10)
    print!("    ");
    for x in 0..GRID_X {
        if x % 10 == 0 {
            print!("{:<10}", x);
        }
    }
    println!();

    for y in 0..GRID_Y {
        // Y-axis label (every 10, or blank)
        if y % 10 == 0 {
            print!("{:>3} ", y);
        } else {
            print!("    ");
        }

        let row: String = (0..GRID_X)
            .map(|x| {
                grid.get(x, y, z)
                    .map(|v| voxel_char(v.material, v.water_level))
                    .unwrap_or(' ')
            })
            .collect();
        println!("{row}");
    }

    // Legend
    println!();
    println!("Legend: . air  ~ water  # soil  % wet soil  @ stone  * root  s seed");
    Ok(())
}

/// Parse a coordinate argument as a single value or an inclusive range.
/// Accepts "N" (single) or "N..M" (range from N to M-1, exclusive end).
fn parse_coord_range(s: &str, label: &str) -> std::io::Result<std::ops::Range<usize>> {
    if let Some((a, b)) = s.split_once("..") {
        let start: usize = a.parse().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad {label} range start: {e}"))
        })?;
        let end: usize = b.parse().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad {label} range end: {e}"))
        })?;
        if start >= end {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{label} range {start}..{end} is empty (start must be < end)"),
            ));
        }
        Ok(start..end)
    } else {
        let v: usize = s.parse().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad {label}: {e}"))
        })?;
        Ok(v..v + 1)
    }
}

pub fn cmd_place(args: &[String]) -> std::io::Result<()> {
    if args.len() < 4 {
        eprintln!("Usage: groundwork place <material> <x> <y> <z> [--state FILE]");
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

    let x_range = parse_coord_range(&args[1], "x")?;
    let y_range = parse_coord_range(&args[2], "y")?;
    let z_range = parse_coord_range(&args[3], "z")?;

    let path = state_path(&args[4..]);
    let mut world = groundwork_sim::save::load_world(&path)?;

    let mut count: u64 = 0;
    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        for z in z_range.clone() {
            for y in y_range.clone() {
                for x in x_range.clone() {
                    let voxel = grid.get_mut(x, y, z).ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("out of bounds: ({x}, {y}, {z})"),
                        )
                    })?;
                    voxel.set_material(mat);
                    count += 1;
                }
            }
        }
    }

    groundwork_sim::save::save_world(&world, &path)?;
    if count == 1 {
        println!(
            "Placed {} at ({}, {}, {})",
            mat.name(),
            x_range.start,
            y_range.start,
            z_range.start
        );
    } else {
        println!("Placed {} x{count} voxels", mat.name());
    }
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
    println!("  view [--z Z]                  Print ASCII slice of the grid");
    println!("  place <material> <x> <y> <z>  Place voxel(s) — coords accept ranges (e.g. 20..40)");
    println!("  inspect <x> <y> <z>           Show voxel details");
    println!("  status                        Show world summary");
    println!("  tui                           Launch interactive terminal UI");
    println!("  help                          Show this help");
    println!();
    println!("Options:");
    println!("  --state FILE                  State file (default: groundwork.state)");
}

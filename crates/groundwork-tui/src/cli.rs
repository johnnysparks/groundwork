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

pub fn cmd_tick(args: &[String]) -> std::io::Result<()> {
    let path = state_path(args);
    let n: u64 = args
        .first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let mut world = groundwork_sim::save::load_world(&path)?;
    let mut schedule = groundwork_sim::create_schedule();

    for _ in 0..n {
        groundwork_sim::tick(&mut world, &mut schedule);
    }

    groundwork_sim::save::save_world(&world, &path)?;
    let tick = world.resource::<Tick>().0;
    println!("Tick: {tick} (+{n})");
    Ok(())
}

pub fn cmd_view(args: &[String]) -> std::io::Result<()> {
    let path = state_path(args);
    let world = groundwork_sim::save::load_world(&path)?;
    let grid = world.resource::<VoxelGrid>();
    let tick = world.resource::<Tick>().0;

    let z: usize = find_value(args, "--z")
        .and_then(|v| v.parse().ok())
        .unwrap_or(GROUND_LEVEL + 1);

    let depth_label = if z > GROUND_LEVEL {
        format!("above +{}", z - GROUND_LEVEL)
    } else if z == GROUND_LEVEL {
        "surface".to_string()
    } else {
        format!("below -{}", GROUND_LEVEL - z)
    };

    println!("Z:{z} ({depth_label})  Tick:{tick}  Grid:{GRID_X}x{GRID_Y}x{GRID_Z}");
    println!();

    for y in 0..GRID_Y {
        let row: String = (0..GRID_X)
            .map(|x| {
                grid.get(x, y, z)
                    .map(|v| voxel_char(v.material, v.water_level))
                    .unwrap_or(' ')
            })
            .collect();
        println!("{row}");
    }
    Ok(())
}

pub fn cmd_place(args: &[String]) -> std::io::Result<()> {
    if args.len() < 4 {
        eprintln!("Usage: groundwork place <material> <x> <y> <z> [--state FILE]");
        eprintln!("Materials: air, soil, stone, water, root");
        std::process::exit(1);
    }

    let mat_name = &args[0];
    let mat = Material::from_name(mat_name).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unknown material: {mat_name}. Valid: air, soil, stone, water, root"),
        )
    })?;

    let x: usize = args[1].parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad x: {e}"))
    })?;
    let y: usize = args[2].parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad y: {e}"))
    })?;
    let z: usize = args[3].parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("bad z: {e}"))
    })?;

    let path = state_path(&args[4..]);
    let mut world = groundwork_sim::save::load_world(&path)?;

    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        let voxel = grid.get_mut(x, y, z).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("out of bounds: ({x}, {y}, {z})"),
            )
        })?;
        voxel.material = mat;
        if mat == Material::Water {
            voxel.water_level = 255;
        }
    }

    groundwork_sim::save::save_world(&world, &path)?;
    println!("Placed {} at ({x}, {y}, {z})", mat.name());
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

    println!("Voxel at ({x}, {y}, {z}):");
    println!("  material: {}", voxel.material.name());
    println!("  water_level: {}", voxel.water_level);
    println!("  light_level: {}", voxel.light_level);
    println!("  nutrient_level: {}", voxel.nutrient_level);
    Ok(())
}

pub fn cmd_status(args: &[String]) -> std::io::Result<()> {
    let path = state_path(args);
    let world = groundwork_sim::save::load_world(&path)?;
    let grid = world.resource::<VoxelGrid>();
    let tick = world.resource::<Tick>().0;

    let mut counts = [0u64; 5];
    for v in grid.cells() {
        counts[v.material.as_u8() as usize] += 1;
    }

    println!("Tick: {tick}");
    println!("Grid: {GRID_X}x{GRID_Y}x{GRID_Z} ({} voxels)", GRID_X * GRID_Y * GRID_Z);
    println!("Materials:");
    for i in 0..5 {
        if let Some(mat) = Material::from_u8(i) {
            println!("  {}: {}", mat.name(), counts[i as usize]);
        }
    }
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
    println!("  place <material> <x> <y> <z>  Place a voxel (air/soil/stone/water/root)");
    println!("  inspect <x> <y> <z>           Show voxel details");
    println!("  status                        Show world summary");
    println!("  tui                           Launch interactive terminal UI");
    println!("  help                          Show this help");
    println!();
    println!("Options:");
    println!("  --state FILE                  State file (default: groundwork.state)");
}

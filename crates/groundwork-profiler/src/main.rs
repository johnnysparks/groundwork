//! Stress-test profiler for groundwork-sim.
//!
//! Usage:
//!   cargo run --release -p groundwork-profiler                    # default: 500 ticks, full schedule
//!   cargo run --release -p groundwork-profiler -- 2000            # custom tick count
//!   cargo run --release -p groundwork-profiler -- --systems 500   # per-system breakdown
//!   cargo flamegraph -p groundwork-profiler -- 1000               # flamegraph (needs perf)
//!
//! Seeds trees across the grid to stress all systems: water flow, soil evolution,
//! light propagation, tree growth, branching, root growth, seed dispersal.

mod per_system;

use std::time::Instant;

use groundwork_sim::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z};
use groundwork_sim::voxel::Material;
use groundwork_sim::{create_world, create_schedule, tick, Tick};

fn parse_ticks(args: &[String], skip_flags: &[&str]) -> u64 {
    args.iter()
        .skip(1)
        .filter(|a| !skip_flags.iter().any(|f| a.as_str() == *f))
        .find_map(|s| s.parse().ok())
        .unwrap_or(500)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let has_flag = |f: &str| args.iter().any(|a| a == f);

    if has_flag("--systems") {
        per_system::run_per_system_profile(parse_ticks(&args, &["--systems"]));
        return;
    }

    if has_flag("--flamegraph") {
        run_with_flamegraph(parse_ticks(&args, &["--flamegraph"]));
        return;
    }

    run_full_profile(parse_ticks(&args, &[]));
}

fn run_full_profile(num_ticks: u64) {
    eprintln!("=== Groundwork Sim Profiler ===");
    eprintln!("Grid: {}x{}x{} = {} voxels", GRID_X, GRID_Y, GRID_Z, GRID_X * GRID_Y * GRID_Z);
    eprintln!("Ticks: {}", num_ticks);
    eprintln!();

    // --- Setup ---
    let setup_start = Instant::now();
    let mut world = create_world();
    let mut schedule = create_schedule();

    // Plant a grid of seeds to stress tree growth, branching, roots, etc.
    // 6x6 = 36 trees spread across the terrain
    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        let spacing_x = GRID_X / 7;
        let spacing_y = GRID_Y / 7;
        let mut planted = 0;
        for row in 1..=6 {
            for col in 1..=6 {
                let x = col * spacing_x;
                let y = row * spacing_y;
                let surface = VoxelGrid::surface_height(x, y);
                let z = surface + 1;
                if z < GRID_Z {
                    if let Some(cell) = grid.get(x, y, z) {
                        if cell.material == Material::Air {
                            if let Some(cell) = grid.get_mut(x, y, z) {
                                cell.material = Material::Seed;
                                planted += 1;
                            }
                        }
                    }
                }
            }
        }
        eprintln!("Planted {} seeds across grid", planted);
    }

    // Also flood a larger water area to stress water_flow
    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        let cx = GRID_X / 2;
        let cy = GRID_Y / 2;
        for dy in (cy.saturating_sub(15))..=(cy + 15).min(GRID_Y - 1) {
            for dx in (cx.saturating_sub(15))..=(cx + 15).min(GRID_X - 1) {
                let surface = VoxelGrid::surface_height(dx, dy);
                let z = surface;
                if let Some(cell) = grid.get_mut(dx, dy, z) {
                    if cell.material == Material::Soil || cell.material == Material::Air {
                        cell.material = Material::Water;
                        cell.water_level = 255;
                    }
                }
            }
        }
        eprintln!("Flooded 30x30 area with water");
    }

    let setup_time = setup_start.elapsed();
    eprintln!("Setup: {:.2}ms", setup_time.as_secs_f64() * 1000.0);
    eprintln!();

    // --- Ticking with per-tick timing ---
    let mut tick_times = Vec::with_capacity(num_ticks as usize);
    let total_start = Instant::now();

    for i in 0..num_ticks {
        let t0 = Instant::now();
        tick(&mut world, &mut schedule);
        let elapsed = t0.elapsed();
        tick_times.push(elapsed);

        // Progress every 100 ticks
        if (i + 1) % 100 == 0 || i == num_ticks - 1 {
            let avg = tick_times[tick_times.len().saturating_sub(100)..]
                .iter()
                .map(|d| d.as_secs_f64())
                .sum::<f64>()
                / tick_times[tick_times.len().saturating_sub(100)..].len() as f64;
            eprintln!(
                "  tick {:>5} | last batch avg: {:>8.2}ms | this tick: {:>8.2}ms",
                i + 1,
                avg * 1000.0,
                elapsed.as_secs_f64() * 1000.0,
            );
        }
    }

    let total_time = total_start.elapsed();
    eprintln!();

    // --- Statistics ---
    let times_ms: Vec<f64> = tick_times.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    let total_ms = total_time.as_secs_f64() * 1000.0;
    let avg_ms = times_ms.iter().sum::<f64>() / times_ms.len() as f64;
    let min_ms = times_ms.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_ms = times_ms.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut sorted = times_ms.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = sorted[sorted.len() / 2];
    let p90 = sorted[(sorted.len() as f64 * 0.9) as usize];
    let p99 = sorted[(sorted.len() as f64 * 0.99) as usize];

    // Variance and stddev
    let variance = times_ms.iter().map(|t| (t - avg_ms).powi(2)).sum::<f64>() / times_ms.len() as f64;
    let stddev = variance.sqrt();

    eprintln!("=== Results ===");
    eprintln!("Total:   {:>10.2}ms ({:.2}s)", total_ms, total_time.as_secs_f64());
    eprintln!("Avg:     {:>10.2}ms/tick", avg_ms);
    eprintln!("Min:     {:>10.2}ms", min_ms);
    eprintln!("Max:     {:>10.2}ms", max_ms);
    eprintln!("Stddev:  {:>10.2}ms", stddev);
    eprintln!("P50:     {:>10.2}ms", p50);
    eprintln!("P90:     {:>10.2}ms", p90);
    eprintln!("P99:     {:>10.2}ms", p99);
    eprintln!("Throughput: {:.1} ticks/sec", num_ticks as f64 / total_time.as_secs_f64());
    eprintln!();

    // --- Material census at end ---
    let grid = world.resource::<VoxelGrid>();
    let mut counts = [0u64; 10];
    for cell in grid.cells() {
        let idx = cell.material.as_u8() as usize;
        if idx < counts.len() {
            counts[idx] += 1;
        }
    }
    let tick_count = world.resource::<Tick>().0;
    eprintln!("=== World State at tick {} ===", tick_count);
    let names = ["air", "soil", "stone", "water", "root", "seed", "trunk", "branch", "leaf", "deadwood"];
    for (i, name) in names.iter().enumerate() {
        if counts[i] > 0 {
            eprintln!("  {:>10}: {:>8} ({:.1}%)", name, counts[i], counts[i] as f64 / (GRID_X * GRID_Y * GRID_Z) as f64 * 100.0);
        }
    }
    eprintln!();

    // --- Tick-by-tick CSV to stdout for graphing ---
    println!("tick,time_ms");
    for (i, t) in times_ms.iter().enumerate() {
        println!("{},{:.4}", i + 1, t);
    }
}

fn setup_world() -> (bevy_ecs::prelude::World, bevy_ecs::prelude::Schedule) {
    let mut world = create_world();
    let schedule = create_schedule();

    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        let spacing_x = GRID_X / 7;
        let spacing_y = GRID_Y / 7;
        for row in 1..=6 {
            for col in 1..=6 {
                let x = col * spacing_x;
                let y = row * spacing_y;
                let surface = VoxelGrid::surface_height(x, y);
                let z = surface + 1;
                if z < GRID_Z {
                    if let Some(cell) = grid.get_mut(x, y, z) {
                        if cell.material == Material::Air {
                            cell.material = Material::Seed;
                        }
                    }
                }
            }
        }
    }
    {
        let mut grid = world.resource_mut::<VoxelGrid>();
        let cx = GRID_X / 2;
        let cy = GRID_Y / 2;
        for dy in (cy.saturating_sub(15))..=(cy + 15).min(GRID_Y - 1) {
            for dx in (cx.saturating_sub(15))..=(cx + 15).min(GRID_X - 1) {
                let surface = VoxelGrid::surface_height(dx, dy);
                if let Some(cell) = grid.get_mut(dx, dy, surface) {
                    if cell.material == Material::Soil || cell.material == Material::Air {
                        cell.material = Material::Water;
                        cell.water_level = 255;
                    }
                }
            }
        }
    }

    (world, schedule)
}

fn run_with_flamegraph(num_ticks: u64) {
    eprintln!("=== Flamegraph Profiling ({} ticks) ===", num_ticks);

    let (mut world, mut schedule) = setup_world();

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(997) // ~1000 Hz sampling, prime to avoid aliasing
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .expect("failed to start profiler");

    for i in 0..num_ticks {
        tick(&mut world, &mut schedule);
        if (i + 1) % 100 == 0 {
            eprintln!("  tick {} done...", i + 1);
        }
    }

    if let Ok(report) = guard.report().build() {
        let svg_path = "profile_flamegraph.svg";
        let file = std::fs::File::create(svg_path).expect("failed to create SVG file");
        report.flamegraph(file).expect("failed to write flamegraph");
        eprintln!("Flamegraph written to {}", svg_path);

        // Also write a collapsed stacks file for further analysis
        let collapsed_path = "profile_collapsed.txt";
        let mut collapsed = std::fs::File::create(collapsed_path).expect("failed to create collapsed file");
        use std::io::Write;
        for (frames, count) in report.data.iter() {
            let symbols: Vec<String> = frames.frames.iter().rev().map(|frame_vec| {
                frame_vec.iter().map(|s| s.name()).collect::<Vec<_>>().join(";")
            }).collect();
            writeln!(collapsed, "{} {}", symbols.join(";"), count).ok();
        }
        eprintln!("Collapsed stacks written to {}", collapsed_path);
    }
}

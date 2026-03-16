//! Per-system instrumented profiling.
//!
//! Runs each ECS system individually and measures wall-clock time,
//! producing a breakdown showing exactly where time is spent per tick.

use std::time::Instant;

use bevy_ecs::prelude::*;

use groundwork_sim::grid::{VoxelGrid, GRID_X, GRID_Y, GRID_Z};
use groundwork_sim::voxel::Material;
use groundwork_sim::{create_world, Tick};

struct SystemProfile {
    name: &'static str,
    schedule: Schedule,
}

impl SystemProfile {
    fn new(name: &'static str, schedule: Schedule) -> Self {
        Self { name, schedule }
    }
}

pub fn run_per_system_profile(num_ticks: u64) {
    eprintln!("=== Per-System Profiling ({} ticks) ===", num_ticks);
    eprintln!(
        "Grid: {}x{}x{} = {} voxels",
        GRID_X,
        GRID_Y,
        GRID_Z,
        GRID_X * GRID_Y * GRID_Z
    );
    eprintln!();

    let mut world = create_world();

    // Plant seeds and flood water (same setup as main profiler)
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

    // Build individual schedules for each system
    use groundwork_sim::systems::*;

    let mut systems: Vec<SystemProfile> = Vec::new();

    macro_rules! add_system {
        ($name:expr, $sys:expr) => {{
            let mut sched = Schedule::default();
            sched.add_systems($sys);
            systems.push(SystemProfile::new($name, sched));
        }};
    }

    add_system!("water_spring", water_spring);
    add_system!("water_flow", water_flow);
    add_system!("soil_absorption", soil_absorption);
    add_system!("root_water_absorption", root_water_absorption);
    add_system!("soil_evolution", soil_evolution);
    add_system!("light_propagation", light_propagation);
    add_system!("seed_growth", seed_growth);
    // ApplyDeferred handled inline
    add_system!("tree_growth", tree_growth);
    add_system!("branch_growth", branch_growth);
    add_system!("self_pruning", self_pruning);
    add_system!("tree_rasterize", tree_rasterize);
    add_system!("root_growth", root_growth);
    add_system!("seed_dispersal", seed_dispersal);

    // tick_counter
    let mut tick_sched = Schedule::default();
    tick_sched.add_systems(|mut t: ResMut<Tick>| {
        t.0 += 1;
    });
    systems.push(SystemProfile::new("tick_counter", tick_sched));

    let num_systems = systems.len();
    // Accumulated times per system
    let mut totals = vec![0.0f64; num_systems];
    let mut maxes = vec![0.0f64; num_systems];

    let total_start = Instant::now();

    for tick_num in 0..num_ticks {
        for (i, sys) in systems.iter_mut().enumerate() {
            let t0 = Instant::now();
            sys.schedule.run(&mut world);
            let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
            totals[i] += elapsed_ms;
            if elapsed_ms > maxes[i] {
                maxes[i] = elapsed_ms;
            }
        }

        if (tick_num + 1) % 100 == 0 {
            eprintln!("  tick {} done...", tick_num + 1);
        }
    }

    let total_time = total_start.elapsed();
    let total_ms = total_time.as_secs_f64() * 1000.0;
    let grand_total_system_ms: f64 = totals.iter().sum();

    eprintln!();
    eprintln!("=== Per-System Breakdown ===");
    eprintln!(
        "{:<25} {:>10} {:>10} {:>10} {:>8}",
        "System", "Total(ms)", "Avg(ms)", "Max(ms)", "% Time"
    );
    eprintln!("{}", "-".repeat(73));

    // Sort by total time descending for the display
    let mut indices: Vec<usize> = (0..num_systems).collect();
    indices.sort_by(|&a, &b| totals[b].partial_cmp(&totals[a]).unwrap());

    for &i in &indices {
        let avg = totals[i] / num_ticks as f64;
        let pct = totals[i] / grand_total_system_ms * 100.0;
        eprintln!(
            "{:<25} {:>10.2} {:>10.3} {:>10.3} {:>7.1}%",
            systems[i].name, totals[i], avg, maxes[i], pct
        );
    }

    eprintln!("{}", "-".repeat(73));
    eprintln!(
        "{:<25} {:>10.2} {:>10.3} {:>10} {:>7}",
        "TOTAL (systems)",
        grand_total_system_ms,
        grand_total_system_ms / num_ticks as f64,
        "",
        "100.0%"
    );
    eprintln!("{:<25} {:>10.2}", "Wall clock", total_ms);
    eprintln!(
        "{:<25} {:>10.2}ms ({:.1}%)",
        "Overhead",
        total_ms - grand_total_system_ms,
        (total_ms - grand_total_system_ms) / total_ms * 100.0
    );
    eprintln!();
    eprintln!(
        "Avg tick: {:.3}ms | Throughput: {:.1} ticks/sec",
        total_ms / num_ticks as f64,
        num_ticks as f64 / total_time.as_secs_f64()
    );

    // CSV output
    println!("system,total_ms,avg_ms,max_ms,pct");
    for &i in &indices {
        let avg = totals[i] / num_ticks as f64;
        let pct = totals[i] / grand_total_system_ms * 100.0;
        println!(
            "{},{:.4},{:.4},{:.4},{:.2}",
            systems[i].name, totals[i], avg, maxes[i], pct
        );
    }
}

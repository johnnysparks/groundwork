//! Scenario runner — deterministic execution of scenarios against the sim.
//!
//! The runner creates a fresh world, executes actions in order, records
//! observations and oracle snapshots, and returns a complete trace.
//! Camera state is tracked alongside sim state so evaluators can assess
//! what the player was looking at.
//!
//! Two entry points:
//! - `run()` — execute a scripted `Scenario` (deterministic action list)
//! - `run_autonomous()` — execute with a `Planner` that decides actions dynamically

use bevy_ecs::prelude::*;

use groundwork_sim::grid::{VoxelGrid, GRID_Z, GROUND_LEVEL};
use groundwork_sim::tree::{SeedSpeciesMap, species_name_to_id};
use groundwork_sim::voxel::Material;

use crate::action::Action;
use crate::evaluator::{Evaluator, Verdict};
use crate::observer::{self, Observation};
use crate::oracle::{self, CameraState};
use crate::planner::{ObservationEntry, Planner};
use crate::scenario::Scenario;
use crate::trace::Trace;

/// Result of running a scenario.
pub struct RunResult {
    /// The complete trace of the run.
    pub trace: Trace,
    /// Verdicts from all evaluators.
    pub verdicts: Vec<Verdict>,
}

impl RunResult {
    /// Whether all evaluators passed.
    pub fn all_passed(&self) -> bool {
        self.verdicts.iter().all(|v| v.passed)
    }

    /// Print a human-readable report.
    pub fn report(&self) -> String {
        let mut lines = vec![format!("=== {} ===", self.trace.scenario_name)];
        lines.push(format!(
            "{} steps, {:.0}ms total",
            self.trace.steps.len(),
            self.trace.total_duration.as_millis()
        ));
        lines.push(String::new());

        for v in &self.verdicts {
            lines.push(format!("  {v}"));
        }

        let passed = self.verdicts.iter().filter(|v| v.passed).count();
        let total = self.verdicts.len();
        lines.push(String::new());
        lines.push(format!(
            "Result: {passed}/{total} passed {}",
            if self.all_passed() { "✓" } else { "✗" }
        ));

        lines.join("\n")
    }
}

/// Run a scenario against a fresh world and return the trace + verdicts.
pub fn run(scenario: &Scenario) -> RunResult {
    let mut world = groundwork_sim::create_world();
    let mut schedule = groundwork_sim::create_schedule();
    let mut camera = CameraState::default();

    let mut trace_builder = Trace::builder(&scenario.name);

    for action in &scenario.actions {
        trace_builder.begin_step();

        let observation = execute_action(&mut world, &mut schedule, &mut camera, action);
        let oracle_snapshot = oracle::snapshot_with_probes(&world, &scenario.probes, &camera);

        trace_builder.record(action.clone(), observation, oracle_snapshot);
    }

    let trace = trace_builder.finish();

    let verdicts: Vec<Verdict> = scenario
        .evaluators
        .iter()
        .map(|e| e.evaluate(&trace))
        .collect();

    RunResult { trace, verdicts }
}

/// Run an autonomous session with a planner that decides actions dynamically.
///
/// The planner sees only `ObservationEntry` (actor-visible text).
/// Oracle snapshots are recorded for evaluators but never shown to the planner.
pub fn run_autonomous(
    planner: &mut dyn Planner,
    probes: &[(usize, usize, usize)],
    evaluators: &[Box<dyn Evaluator>],
    max_steps: usize,
) -> RunResult {
    let mut world = groundwork_sim::create_world();
    let mut schedule = groundwork_sim::create_schedule();
    let mut camera = CameraState::default();

    let scenario_name = format!("autonomous_{}", planner.name());
    let mut trace_builder = Trace::builder(&scenario_name);
    let mut history: Vec<ObservationEntry> = Vec::new();
    let mut step_index: usize = 0;

    loop {
        if step_index >= max_steps || planner.should_stop(&history) {
            break;
        }

        let batch = planner.plan(&history);
        if batch.is_empty() {
            break;
        }

        for action in batch {
            if step_index >= max_steps {
                break;
            }

            trace_builder.begin_step();
            let observation = execute_action(&mut world, &mut schedule, &mut camera, &action);
            let oracle_snapshot = oracle::snapshot_with_probes(&world, probes, &camera);

            // Record in history (actor-visible only — planner sees this)
            history.push(ObservationEntry {
                action: action.clone(),
                observation: observation.clone(),
                step_index,
            });

            // Record in trace (includes oracle — planner never sees this)
            trace_builder.record(action, observation, oracle_snapshot);
            step_index += 1;
        }
    }

    let trace = trace_builder.finish();
    let verdicts: Vec<Verdict> = evaluators.iter().map(|e| e.evaluate(&trace)).collect();

    RunResult { trace, verdicts }
}

/// Execute a single action against the sim and return the actor-visible observation.
fn execute_action(
    world: &mut World,
    schedule: &mut bevy_ecs::schedule::Schedule,
    camera: &mut CameraState,
    action: &Action,
) -> Observation {
    match action {
        Action::Tick { n } => {
            let before = count_materials(world);
            for _ in 0..*n {
                groundwork_sim::tick(world, schedule);
            }
            let after = count_materials(world);
            let tick = world.resource::<groundwork_sim::Tick>().0;

            let mut changes = Vec::new();
            let names = [
                "air", "soil", "stone", "water", "root", "seed", "trunk", "branch", "leaf",
                "deadwood",
            ];
            for i in 0..10 {
                let diff = after[i] as i64 - before[i] as i64;
                if diff != 0 {
                    changes.push(format!("{}: {:+}", names[i], diff));
                }
            }

            let text = if changes.is_empty() {
                format!("Tick: {tick} (+{n})  (no material changes)")
            } else {
                format!("Tick: {tick} (+{n})  Changes: {}", changes.join(", "))
            };

            Observation { text, tick }
        }

        Action::Place {
            tool,
            x,
            y,
            z,
            species,
        } => {
            let tick = world.resource::<groundwork_sim::Tick>().0;
            let name = species.as_deref().unwrap_or(tool.as_str());
            let (mat, species_id) = parse_tool_material(name);

            let mut grid = world.resource_mut::<VoxelGrid>();
            let result = apply_tool(&mut grid, mat, *x, *y, *z);

            let text = match result {
                Some(landing_z) => {
                    drop(grid);
                    if mat == Material::Seed {
                        let mut seed_map = world.resource_mut::<SeedSpeciesMap>();
                        seed_map.map.insert((*x, *y, landing_z), species_id);
                    }
                    if landing_z != *z {
                        format!("Placed {name} at ({x}, {y}, {z}) → landed at z={landing_z}")
                    } else {
                        format!("Placed {name} at ({x}, {y}, {z})")
                    }
                }
                None => format!("Nothing placed at ({x}, {y}, {z}) — skipped"),
            };

            Observation { text, tick }
        }

        Action::Fill {
            tool,
            x1,
            y1,
            z1,
            x2,
            y2,
            z2,
        } => {
            let tick = world.resource::<groundwork_sim::Tick>().0;
            let (mat, species_id) = parse_tool_material(tool);
            let xlo = (*x1).min(*x2);
            let xhi = (*x1).max(*x2);
            let ylo = (*y1).min(*y2);
            let yhi = (*y1).max(*y2);
            let zlo = (*z1).min(*z2);
            let zhi = (*z1).max(*z2);

            let mut placed = 0u64;
            let mut skipped = 0u64;
            let mut seed_landings = Vec::new();
            {
                let mut grid = world.resource_mut::<VoxelGrid>();
                for z in zlo..=zhi {
                    for y in ylo..=yhi {
                        for x in xlo..=xhi {
                            if let Some(lz) = apply_tool(&mut grid, mat, x, y, z) {
                                placed += 1;
                                if mat == Material::Seed {
                                    seed_landings.push((x, y, lz));
                                }
                            } else {
                                skipped += 1;
                            }
                        }
                    }
                }
            }
            if !seed_landings.is_empty() {
                let mut seed_map = world.resource_mut::<SeedSpeciesMap>();
                for pos in seed_landings {
                    seed_map.map.insert(pos, species_id);
                }
            }

            let text = format!(
                "Fill {tool} ({xlo},{ylo},{zlo})→({xhi},{yhi},{zhi}): {placed} placed, {skipped} skipped"
            );
            Observation { text, tick }
        }

        // --- Camera actions ---

        Action::CameraOrbit { theta_deg, phi_deg } => {
            let tick = world.resource::<groundwork_sim::Tick>().0;
            camera.theta_deg = *theta_deg;
            camera.phi_deg = phi_deg.clamp(11.0, 85.0);
            Observation {
                text: format!(
                    "Camera orbit: azimuth {:.0}°, elevation {:.0}°",
                    camera.theta_deg, camera.phi_deg
                ),
                tick,
            }
        }

        Action::CameraPan { x, y, z } => {
            let tick = world.resource::<groundwork_sim::Tick>().0;
            camera.center_x = *x;
            camera.center_y = *y;
            camera.center_z = *z;
            Observation {
                text: format!("Camera pan to ({:.0}, {:.0}, {:.0})", x, y, z),
                tick,
            }
        }

        Action::CameraZoom { level } => {
            let tick = world.resource::<groundwork_sim::Tick>().0;
            camera.zoom = level.clamp(0.3, 4.0);
            Observation {
                text: format!("Camera zoom: {:.1}x", camera.zoom),
                tick,
            }
        }

        Action::CameraCutaway { z } => {
            let tick = world.resource::<groundwork_sim::Tick>().0;
            camera.cutaway_z = z.clamp(0.0, GRID_Z as f64);
            let gl = GROUND_LEVEL as f64;
            let label = if camera.cutaway_z >= GRID_Z as f64 - 0.5 {
                "no cutaway (full view)".to_string()
            } else if camera.cutaway_z >= gl - 0.5 {
                format!("surface + {:.0} above", camera.cutaway_z - gl)
            } else {
                format!("{:.0} below surface", gl - camera.cutaway_z)
            };
            Observation {
                text: format!("Camera cutaway: z={:.0} ({})", camera.cutaway_z, label),
                tick,
            }
        }

        Action::CameraReset => {
            let tick = world.resource::<groundwork_sim::Tick>().0;
            *camera = CameraState::default();
            Observation {
                text: "Camera reset to default diorama view".to_string(),
                tick,
            }
        }

        // --- Observations ---

        Action::Inspect { x, y, z } => observer::observe_inspect(world, *x, *y, *z),

        Action::Status => observer::observe_status(world),

        Action::View { z } => observer::observe_view(world, *z),

        Action::Checkpoint { label } => {
            let tick = world.resource::<groundwork_sim::Tick>().0;
            Observation {
                text: format!("--- {label} ---"),
                tick,
            }
        }
    }
}

/// Count materials in the grid (for tick change reporting).
fn count_materials(world: &World) -> [u64; 10] {
    let grid = world.resource::<VoxelGrid>();
    let mut counts = [0u64; 10];
    for v in grid.cells() {
        counts[v.material.as_u8() as usize] += 1;
    }
    counts
}

/// Parse tool name into Material + species ID.
fn parse_tool_material(name: &str) -> (Material, usize) {
    match name.to_ascii_lowercase().as_str() {
        "air" | "dig" => (Material::Air, 0),
        "water" => (Material::Water, 0),
        "soil" => (Material::Soil, 0),
        "stone" => (Material::Stone, 0),
        "seed" => (Material::Seed, 0),
        other => {
            if let Some(id) = species_name_to_id(other) {
                (Material::Seed, id)
            } else {
                (Material::Seed, 0)
            }
        }
    }
}

/// Apply a tool at coordinates, handling gravity for seeds/water/soil.
/// Returns the landing Z coordinate, or None if the action was skipped.
fn apply_tool(grid: &mut VoxelGrid, mat: Material, x: usize, y: usize, z: usize) -> Option<usize> {
    if !VoxelGrid::in_bounds(x, y, z) {
        return None;
    }

    if mat == Material::Air {
        let voxel = grid.get(x, y, z)?;
        if voxel.material == Material::Air {
            return None;
        }
        grid.get_mut(x, y, z)?.set_material(Material::Air);
        return Some(z);
    }

    let voxel = grid.get(x, y, z)?;
    if voxel.material != Material::Air {
        return None;
    }

    let landing_z = match mat {
        Material::Stone => z,
        _ => grid.find_landing_z(x, y, z),
    };

    let landing_voxel = grid.get(x, y, landing_z)?;
    if landing_voxel.material != Material::Air {
        return None;
    }

    grid.get_mut(x, y, landing_z)?.set_material(mat);
    Some(landing_z)
}

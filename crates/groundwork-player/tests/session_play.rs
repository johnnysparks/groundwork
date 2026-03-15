//! Interactive session test — Claude Code acts as the LLM planner.
//!
//! This test drives the game loop step by step, printing observations
//! so the agent can decide what to do next. It exercises the full
//! Phase 1-3 pipeline: actions → observations → oracle → trace → analysis.

use groundwork_player::action::Action;
use groundwork_player::analysis;
use groundwork_player::evaluator::{
    CameraOrbited, CameraWentUnderground, Evaluator, MaterialMinimum, NoCrash,
};
use groundwork_player::oracle::{self, CameraState};
use groundwork_player::runner;
use groundwork_player::skills;
use groundwork_player::trace::Trace;

/// Full interactive session: Claude Code plays the garden.
#[test]
fn session_play() {
    let mut world = groundwork_sim::create_world();
    let mut schedule = groundwork_sim::create_schedule();
    let mut camera = CameraState::default();
    let mut trace_builder = Trace::builder("claude_session");
    let probes = vec![(40, 40, 41), (40, 40, 38), (36, 40, 41), (44, 40, 41)];

    // Helper closure to execute + record + print
    let mut step = |world: &mut bevy_ecs::prelude::World,
                    schedule: &mut bevy_ecs::schedule::Schedule,
                    camera: &mut CameraState,
                    trace_builder: &mut groundwork_player::trace::TraceBuilder,
                    action: Action| {
        trace_builder.begin_step();
        let obs = runner::execute_action(world, schedule, camera, &action);
        let oracle = oracle::snapshot_with_probes(world, &probes, camera);
        eprintln!("  {action} → {}", obs.text);
        trace_builder.record(action, obs, oracle);
    };

    // ===== ROUND 1: Survey the world =====
    eprintln!("\n=== ROUND 1: Survey ===");
    step(&mut world, &mut schedule, &mut camera, &mut trace_builder, Action::Status);
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::CameraOrbit {
            theta_deg: 0.0,
            phi_deg: 55.0,
        },
    );

    // ===== ROUND 2: Create water features =====
    eprintln!("\n=== ROUND 2: Water ===");
    // Main pond near center
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Fill {
            tool: "water".into(),
            x1: 37,
            y1: 37,
            z1: 45,
            x2: 43,
            y2: 43,
            z2: 45,
        },
    );
    // Small side stream
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Fill {
            tool: "water".into(),
            x1: 44,
            y1: 40,
            z1: 45,
            x2: 50,
            y2: 42,
            z2: 45,
        },
    );
    // Let water settle
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Tick { n: 5 },
    );
    step(&mut world, &mut schedule, &mut camera, &mut trace_builder, Action::Status);

    // ===== ROUND 3: Plant diverse garden =====
    eprintln!("\n=== ROUND 3: Plant ===");
    let seed_z = 50;
    // Trees (spaced out, near water)
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Place {
            tool: "seed".into(),
            x: 35,
            y: 40,
            z: seed_z,
            species: Some("oak".into()),
        },
    );
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Place {
            tool: "seed".into(),
            x: 45,
            y: 40,
            z: seed_z,
            species: Some("birch".into()),
        },
    );
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Place {
            tool: "seed".into(),
            x: 40,
            y: 35,
            z: seed_z,
            species: Some("pine".into()),
        },
    );
    // Shrubs (closer to water)
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Place {
            tool: "seed".into(),
            x: 36,
            y: 38,
            z: seed_z,
            species: Some("fern".into()),
        },
    );
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Place {
            tool: "seed".into(),
            x: 44,
            y: 42,
            z: seed_z,
            species: Some("holly".into()),
        },
    );
    // Groundcover (right at water edge)
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Place {
            tool: "seed".into(),
            x: 38,
            y: 36,
            z: seed_z,
            species: Some("moss".into()),
        },
    );
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Place {
            tool: "seed".into(),
            x: 42,
            y: 44,
            z: seed_z,
            species: Some("grass".into()),
        },
    );
    // Flowers
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Place {
            tool: "seed".into(),
            x: 40,
            y: 45,
            z: seed_z,
            species: Some("wildflower".into()),
        },
    );
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Place {
            tool: "seed".into(),
            x: 40,
            y: 37,
            z: seed_z,
            species: Some("daisy".into()),
        },
    );

    // ===== ROUND 4: Watch it grow =====
    eprintln!("\n=== ROUND 4: Grow phase 1 ===");
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Tick { n: 50 },
    );
    step(&mut world, &mut schedule, &mut camera, &mut trace_builder, Action::Status);

    // Orbit around to see the garden from different angles
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::CameraOrbit {
            theta_deg: 90.0,
            phi_deg: 45.0,
        },
    );
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::CameraZoom { level: 1.5 },
    );
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::CameraPan {
            x: 40.0,
            y: 40.0,
            z: 41.0,
        },
    );

    // Inspect an oak sapling
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Inspect {
            x: 35,
            y: 40,
            z: 41,
        },
    );

    // ===== ROUND 5: More growth =====
    eprintln!("\n=== ROUND 5: Grow phase 2 ===");
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Tick { n: 100 },
    );
    step(&mut world, &mut schedule, &mut camera, &mut trace_builder, Action::Status);

    // ===== ROUND 6: Underground exploration =====
    eprintln!("\n=== ROUND 6: Underground ===");
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::CameraCutaway { z: 35.0 },
    );
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::CameraOrbit {
            theta_deg: 180.0,
            phi_deg: 40.0,
        },
    );
    // See root systems
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Inspect {
            x: 35,
            y: 40,
            z: 38,
        },
    );
    // Go deeper
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::CameraCutaway { z: 25.0 },
    );
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Inspect {
            x: 40,
            y: 40,
            z: 30,
        },
    );

    // ===== ROUND 7: Final growth and survey =====
    eprintln!("\n=== ROUND 7: Final growth ===");
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::CameraCutaway { z: 100.0 },
    ); // back to full view
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::Tick { n: 150 },
    );
    step(&mut world, &mut schedule, &mut camera, &mut trace_builder, Action::Status);

    // Final panoramic orbit
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::CameraOrbit {
            theta_deg: 270.0,
            phi_deg: 60.0,
        },
    );
    step(
        &mut world,
        &mut schedule,
        &mut camera,
        &mut trace_builder,
        Action::CameraZoom { level: 0.8 },
    );

    // ===== EVALUATE =====
    let trace = trace_builder.finish();

    let evaluators: Vec<Box<dyn Evaluator>> = vec![
        Box::new(NoCrash),
        Box::new(MaterialMinimum::new("plant", 5)),
        Box::new(MaterialMinimum::new("water", 1)),
        Box::new(CameraOrbited),
        Box::new(CameraWentUnderground::new()),
    ];

    let verdicts: Vec<_> = evaluators.iter().map(|e| e.evaluate(&trace)).collect();
    let result = groundwork_player::runner::RunResult {
        trace,
        verdicts,
    };

    eprintln!("\n{}", result.report());

    // Run Phase 3 analysis
    let summary = analysis::summarize(&result);
    eprintln!("\n=== ANALYSIS ===");
    eprintln!("Steps: {}", summary.step_count);
    eprintln!("Score: {:.0}%", summary.aggregate_score * 100.0);
    eprintln!("Final plants: {}", summary.final_material_counts.total_plant());
    eprintln!("Final trees: {}", summary.final_material_counts.total_tree());
    eprintln!("Camera actions: {}", summary.camera_coverage.total_camera_actions);
    eprintln!("Went underground: {}", summary.camera_coverage.went_underground);

    // Extract skills
    let extracted = skills::extract_skills(&[summary.clone()], &[result.trace.clone()]);
    eprintln!("\n=== SKILLS EXTRACTED ===");
    for skill in &extracted {
        eprintln!("  {} — {}", skill.name, skill.description);
    }

    // Growth timeline
    eprintln!("\n=== GROWTH TIMELINE ===");
    for (tick, plants) in &summary.growth_timeline {
        eprintln!("  tick {tick}: {plants} plant voxels");
    }

    // Save trace
    let trace_dir = std::path::Path::new("artifacts/traces");
    std::fs::create_dir_all(trace_dir).ok();
    let trace_path = trace_dir.join("claude_session.json");
    std::fs::write(&trace_path, result.trace.to_json()).ok();
    eprintln!("\nTrace saved to {}", trace_path.display());

    // Assert success
    assert!(result.all_passed(), "Session failed:\n{}", result.report());
}

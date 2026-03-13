use std::process::Command;
use tempfile::NamedTempFile;

fn groundwork() -> Command {
    Command::new(env!("CARGO_BIN_EXE_groundwork"))
}

fn new_world() -> NamedTempFile {
    let f = NamedTempFile::new().unwrap();
    let out = groundwork()
        .args(["new", "--state", f.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "new failed: {}", String::from_utf8_lossy(&out.stderr));
    f
}

// ── Shovel removes anything ───────────────────────────────────────────

/// Shovel (place air) removes a seed.
#[test]
fn shovel_removes_seed() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place seed above terrain. Position (30,10) has surface_height=15,
    // so seed lands at z=16.
    let out = groundwork()
        .args(["place", "seed", "30", "10", "20", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "seed placement failed: {}", String::from_utf8_lossy(&out.stderr));

    // z=20 should be air (seed fell)
    let out = groundwork()
        .args(["inspect", "30", "10", "20", "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("air"), "z=20 should be air after seed fell, got: {stdout}");

    // Seed should have landed at z=16 (surface_height+1)
    let out = groundwork()
        .args(["inspect", "30", "10", "16", "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("seed"), "expected seed at z=16, got: {stdout}");

    // Use shovel to remove it
    let out = groundwork()
        .args(["place", "air", "30", "10", "16", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "shovel failed: {}", String::from_utf8_lossy(&out.stderr));

    // Verify it's gone
    let out = groundwork()
        .args(["inspect", "30", "10", "16", "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("air"), "expected air after shovel, got: {stdout}");
}

/// Shovel works with 'dig' alias too.
#[test]
fn dig_alias_works() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Dig out some soil at the surface
    let out = groundwork()
        .args(["place", "dig", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "dig failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("shovel"), "should say shovel, got: {stdout}");
}

// ── Protection without --force ────────────────────────────────────────

/// Non-shovel tools can't overwrite occupied cells.
#[test]
fn cant_place_on_occupied_cell() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Try to place water on soil (z=15 is soil) — should skip
    let out = groundwork()
        .args(["place", "water", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Should report nothing placed (cell is occupied)
    assert!(
        stderr.contains("Nothing placed") || stdout.contains("skipped"),
        "expected skip on occupied cell, stdout: {stdout}, stderr: {stderr}"
    );
}

/// Normal placement into air works cleanly.
#[test]
fn place_into_air_succeeds() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place stone into air well above ground (z=20, away from outcrops)
    let out = groundwork()
        .args(["place", "stone", "20", "20", "20", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("stone"), "should confirm stone placement, got: {stdout}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.is_empty(), "should have no warnings, got: {stderr}");
}

// ── Gravity ───────────────────────────────────────────────────────────

/// Seeds fall through air to land above solid ground.
#[test]
fn seed_falls_through_air() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place seed high up (z=25) at position (30,10) where surface_height=15.
    // Seed lands at z=16 (surface+1).
    let out = groundwork()
        .args(["place", "seed", "30", "10", "25", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "seed placement failed: {}", String::from_utf8_lossy(&out.stderr));

    // Verify z=25 is still air (seed fell)
    let out = groundwork()
        .args(["inspect", "30", "10", "25", "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("air"), "z=25 should still be air, got: {stdout}");

    // Verify seed landed at z=16 (surface_height+1)
    let out = groundwork()
        .args(["inspect", "30", "10", "16", "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("seed"), "seed should have fallen to z=16, got: {stdout}");
}

/// Soil falls through air.
#[test]
fn soil_falls_through_air() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // First, dig out the soil at z=15 to make air there
    let out = groundwork()
        .args(["place", "dig", "5", "5", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());

    // Place soil at z=20 — should fall to z=15 (the gap we just dug)
    let out = groundwork()
        .args(["place", "soil", "5", "5", "20", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());

    // Verify soil is at z=15
    let out = groundwork()
        .args(["inspect", "5", "5", "15", "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("soil"), "soil should have fallen to z=15, got: {stdout}");
}

/// Stone does NOT fall (placed directly).
#[test]
fn stone_does_not_fall() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place stone at z=20 — should stay at z=20 (no gravity)
    let out = groundwork()
        .args(["place", "stone", "10", "10", "20", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = groundwork()
        .args(["inspect", "10", "10", "20", "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("stone"), "stone should stay at z=20, got: {stdout}");
}

// ── Seeds on stone ────────────────────────────────────────────────────

/// Seeds die on stone — can't plant on rock.
#[test]
fn seed_dies_on_stone() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Dig out everything above stone at (5,5). Stone is at z=0..4.
    // Dig z=5..15 (soil layers) to expose stone at z=4.
    for z in 5..=15 {
        let out = groundwork()
            .args(["place", "dig", "5", "5", &z.to_string(), "--state", p])
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    // Now try to place a seed — it should fall and land on stone, then die (not placed)
    let out = groundwork()
        .args(["place", "seed", "5", "5", "20", "--state", p])
        .output()
        .unwrap();
    // Should report nothing placed since seed dies on stone
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Nothing placed") || stdout.contains("0"),
        "seed should die on stone, stdout: {stdout}, stderr: {stderr}"
    );
}

// ── Fill tests ────────────────────────────────────────────────────────

/// Fill air region with soil works.
#[test]
fn fill_air_with_soil() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Fill a 3x3x1 region of air (z=16) with soil
    let out = groundwork()
        .args(["fill", "soil", "0", "0", "16", "2", "2", "16", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Soil has gravity, but z=16 is directly above soil at z=15, so it stays at z=16
    assert!(stdout.contains("soil"), "should confirm soil fill, got: {stdout}");
}

/// Fill with shovel (dig) removes a region.
#[test]
fn fill_dig_removes_region() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Fill-dig a 3x3x1 region of soil at z=15
    let out = groundwork()
        .args(["fill", "dig", "0", "0", "15", "2", "2", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("shovel"), "should say shovel, got: {stdout}");

    // Verify the cells are now air
    let out = groundwork()
        .args(["inspect", "1", "1", "15", "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("air"), "cell should be air after dig, got: {stdout}");
}

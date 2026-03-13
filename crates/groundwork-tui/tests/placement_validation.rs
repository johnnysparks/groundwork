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

/// Query the surface height at (x, y) via `inspect` at GROUND_LEVEL.
/// Returns the z of the first air cell by binary searching upward.
fn find_surface(p: &str, x: u32, y: u32) -> u32 {
    // GROUND_LEVEL is 30 in the 120×120×60 grid. Surface is usually within ±4.
    // Search upward from z=25 to find the first air cell.
    for z in 25..=40 {
        let out = groundwork()
            .args(["inspect", &x.to_string(), &y.to_string(), &z.to_string(), "--state", p])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        if stdout.contains("air") {
            return z - 1; // surface is the last non-air
        }
    }
    30 // fallback
}

// ── Shovel removes anything ───────────────────────────────────────────

/// Shovel (place air) removes a seed.
#[test]
fn shovel_removes_seed() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    let surface = find_surface(p, 60, 20);
    let above = surface + 1;
    let high = surface + 10;

    // Place seed high up — it should fall to just above surface.
    let out = groundwork()
        .args(["place", "seed", "60", "20", &high.to_string(), "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "seed placement failed: {}", String::from_utf8_lossy(&out.stderr));

    // High z should be air (seed fell)
    let out = groundwork()
        .args(["inspect", "60", "20", &high.to_string(), "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("air"), "z={high} should be air after seed fell, got: {stdout}");

    // Seed should have landed at surface+1
    let out = groundwork()
        .args(["inspect", "60", "20", &above.to_string(), "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("seed"), "expected seed at z={above}, got: {stdout}");

    // Use shovel to remove it
    let out = groundwork()
        .args(["place", "air", "60", "20", &above.to_string(), "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "shovel failed: {}", String::from_utf8_lossy(&out.stderr));

    // Verify it's gone
    let out = groundwork()
        .args(["inspect", "60", "20", &above.to_string(), "--state", p])
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

    let surface = find_surface(p, 20, 20);

    // Dig out some soil at the surface
    let out = groundwork()
        .args(["place", "dig", "20", "20", &surface.to_string(), "--state", p])
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

    let surface = find_surface(p, 20, 20);

    // Try to place water on soil (surface is soil) — should skip
    let out = groundwork()
        .args(["place", "water", "20", "20", &surface.to_string(), "--state", p])
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

    let surface = find_surface(p, 40, 40);
    let high = surface + 10; // well above ground

    // Place stone into air well above ground
    let out = groundwork()
        .args(["place", "stone", "40", "40", &high.to_string(), "--state", p])
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

    let surface = find_surface(p, 60, 20);
    let above = surface + 1;
    let high = surface + 15;

    // Place seed high up — should fall to surface+1.
    let out = groundwork()
        .args(["place", "seed", "60", "20", &high.to_string(), "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "seed placement failed: {}", String::from_utf8_lossy(&out.stderr));

    // Verify high z is still air (seed fell)
    let out = groundwork()
        .args(["inspect", "60", "20", &high.to_string(), "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("air"), "z={high} should still be air, got: {stdout}");

    // Verify seed landed at surface+1
    let out = groundwork()
        .args(["inspect", "60", "20", &above.to_string(), "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("seed"), "seed should have fallen to z={above}, got: {stdout}");
}

/// Soil falls through air.
#[test]
fn soil_falls_through_air() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    let surface = find_surface(p, 10, 10);
    let high = surface + 10;

    // First, dig out the soil at the surface to make air there
    let out = groundwork()
        .args(["place", "dig", "10", "10", &surface.to_string(), "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());

    // Place soil high up — should fall to the gap we just dug
    let out = groundwork()
        .args(["place", "soil", "10", "10", &high.to_string(), "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());

    // Verify soil is at the dug-out surface level
    let out = groundwork()
        .args(["inspect", "10", "10", &surface.to_string(), "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("soil"), "soil should have fallen to z={surface}, got: {stdout}");
}

/// Stone does NOT fall (placed directly).
#[test]
fn stone_does_not_fall() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    let surface = find_surface(p, 20, 20);
    let high = surface + 10;

    // Place stone high up — should stay there (no gravity for stone)
    let out = groundwork()
        .args(["place", "stone", "20", "20", &high.to_string(), "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = groundwork()
        .args(["inspect", "20", "20", &high.to_string(), "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("stone"), "stone should stay at z={high}, got: {stdout}");
}

// ── Seeds on stone ────────────────────────────────────────────────────

/// Seeds die on stone — can't plant on rock.
#[test]
fn seed_dies_on_stone() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    let surface = find_surface(p, 10, 10);

    // Dig out everything above stone at (10,10). Stone top is at z=9.
    // Dig surface down through soil layers to expose stone.
    for z in (10..=surface).rev() {
        let out = groundwork()
            .args(["place", "dig", "10", "10", &z.to_string(), "--state", p])
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    // Now try to place a seed — it should fall and land on stone, then die (not placed)
    let high = surface + 5;
    let out = groundwork()
        .args(["place", "seed", "10", "10", &high.to_string(), "--state", p])
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

    let surface = find_surface(p, 0, 0);
    let above = surface + 1;

    // Fill a 3x3x1 region of air just above surface with soil
    let out = groundwork()
        .args(["fill", "soil", "0", "0", &above.to_string(), "2", "2", &above.to_string(), "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Soil has gravity, but just above surface it rests on top
    assert!(stdout.contains("soil"), "should confirm soil fill, got: {stdout}");
}

/// Fill with shovel (dig) removes a region.
#[test]
fn fill_dig_removes_region() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    let surface = find_surface(p, 1, 1);

    // Fill-dig a 3x3x1 region of soil at the surface
    let out = groundwork()
        .args(["fill", "dig", "0", "0", &surface.to_string(), "2", "2", &surface.to_string(), "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("shovel"), "should say shovel, got: {stdout}");

    // Verify the cells are now air
    let out = groundwork()
        .args(["inspect", "1", "1", &surface.to_string(), "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("air"), "cell should be air after dig, got: {stdout}");
}

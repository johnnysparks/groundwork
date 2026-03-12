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

/// Place a seed, then try to overwrite it without --force → should fail.
#[test]
fn place_rejects_overwriting_seed() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place a seed
    let out = groundwork()
        .args(["place", "seed", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "seed placement failed");

    // Try to overwrite with water (should be rejected)
    let out = groundwork()
        .args(["place", "water", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(!out.status.success(), "overwriting seed should fail without --force");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot overwrite seed"), "expected rejection message, got: {stderr}");
    assert!(stderr.contains("--force"), "should suggest --force");
}

/// Place a root, then try to overwrite it without --force → should fail.
#[test]
fn place_rejects_overwriting_root() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place a root
    let out = groundwork()
        .args(["place", "root", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "root placement failed");

    // Try to overwrite with soil (should be rejected)
    let out = groundwork()
        .args(["place", "soil", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(!out.status.success(), "overwriting root should fail without --force");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot overwrite root"), "expected rejection message, got: {stderr}");
}

/// --force bypasses protection on seeds.
#[test]
fn place_force_overrides_seed_protection() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place a seed
    let out = groundwork()
        .args(["place", "seed", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());

    // Overwrite with --force (should succeed)
    let out = groundwork()
        .args(["place", "water", "10", "10", "15", "--force", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "force overwrite should succeed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Placed water"), "should confirm placement");
}

/// Overwriting water emits a warning but succeeds.
#[test]
fn place_warns_on_overwriting_water() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place water
    let out = groundwork()
        .args(["place", "water", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());

    // Overwrite with soil (should warn but succeed)
    let out = groundwork()
        .args(["place", "soil", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "overwriting water should succeed (warn only)");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Warning: overwriting water"), "expected warning, got: {stderr}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Placed soil"), "should confirm placement");
}

/// Normal placement (non-living target) works without warnings.
#[test]
fn place_normal_no_warning() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place soil on air (should work cleanly)
    let out = groundwork()
        .args(["place", "soil", "10", "10", "16", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.is_empty(), "should have no warnings for normal placement, got: {stderr}");
}

// ── Fill protection tests (CLI-12) ─────────────────────────────────────

/// Fill skips seeds by default and reports protected count.
#[test]
fn fill_skips_seeds() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place seeds in the fill region
    for x in 10..=12 {
        let out = groundwork()
            .args(["place", "seed", &x.to_string(), "10", "15", "--state", p])
            .output()
            .unwrap();
        assert!(out.status.success(), "seed placement failed at x={x}");
    }

    // Fill the region with water — seeds should be skipped
    let out = groundwork()
        .args(["fill", "water", "10", "10", "15", "14", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "fill should succeed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("3 protected cells skipped"), "expected 3 protected, got: {stdout}");

    // Verify seeds still exist
    for x in 10..=12 {
        let out = groundwork()
            .args(["inspect", &x.to_string(), "10", "15", "--state", p])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("seed"), "seed at x={x} should still exist, got: {stdout}");
    }
}

/// Fill skips roots by default and reports protected count.
#[test]
fn fill_skips_roots() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place a root
    let out = groundwork()
        .args(["place", "root", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());

    // Fill the region with soil — root should be skipped
    let out = groundwork()
        .args(["fill", "soil", "10", "10", "15", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1 protected cells skipped"), "expected 1 protected, got: {stdout}");

    // Verify root still exists
    let out = groundwork()
        .args(["inspect", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root"), "root should still exist, got: {stdout}");
}

/// Fill --force overrides seed/root protection.
#[test]
fn fill_force_overrides_protection() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Place a seed and a root
    let out = groundwork()
        .args(["place", "seed", "10", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = groundwork()
        .args(["place", "root", "11", "10", "15", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());

    // Fill with --force — should overwrite both
    let out = groundwork()
        .args(["fill", "water", "10", "10", "15", "11", "10", "15", "--force", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success(), "fill --force should succeed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("protected"), "force should not report protected cells, got: {stdout}");
    assert!(stdout.contains("Filled 2"), "should fill both cells, got: {stdout}");

    // Verify both are now water
    for x in ["10", "11"] {
        let out = groundwork()
            .args(["inspect", x, "10", "15", "--state", p])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("water"), "cell at x={x} should be water, got: {stdout}");
    }
}

/// Fill on a region with no protected cells works normally.
#[test]
fn fill_normal_no_protection_message() {
    let state = new_world();
    let p = state.path().to_str().unwrap();

    // Fill air with soil (no seeds/roots in the way)
    let out = groundwork()
        .args(["fill", "soil", "0", "0", "16", "2", "2", "16", "--state", p])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("protected"), "no protection message expected, got: {stdout}");
    assert!(stdout.contains("Filled 9"), "should fill 3x3x1=9 cells, got: {stdout}");
}

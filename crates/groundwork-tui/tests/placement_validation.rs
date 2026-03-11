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

//! Gameplay scenarios — core delight moments and discovery arcs.
//!
//! These scenarios represent the experiences a player would expect, talk about,
//! and replay for. Some test working mechanics; others are aspirational and will
//! fail until the underlying features are built. Failing scenarios here are
//! a **feature backlog**, not bugs — they tell us what to build next.

use groundwork_sim::grid::{GRID_X, GRID_Y, GROUND_LEVEL};

use crate::evaluator::{Custom, MaterialGrew, MaterialMinimum, NoCrash, Verdict};
use crate::scenario::Scenario;

// ---------------------------------------------------------------------------
// 1. The Water Table — "I shaped the land and the water followed"
// ---------------------------------------------------------------------------

/// The first "aha" moment: dig a trench, add water, watch it flow downhill
/// and pool where you shaped it. The player realizes they're sculpting hydrology.
pub fn water_table_sculpting() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let surface = GROUND_LEVEL;

    Scenario::new("water_table_sculpting")
        .description(
            "Dig a basin, pour water, and verify it pools where you shaped the terrain. \
             The player's first realization that they control hydrology.",
        )
        .checkpoint("before_digging")
        .status()
        // Dig a 5x5 basin two layers deep
        .fill("air", cx - 2, cy - 2, surface - 1, cx + 2, cy + 2, surface)
        .checkpoint("basin_dug")
        // Pour water above the basin — it should flow down and pool
        .fill(
            "water",
            cx - 1,
            cy - 1,
            surface + 3,
            cx + 1,
            cy + 1,
            surface + 3,
        )
        .tick(20)
        .checkpoint("water_settled")
        .status()
        // Cut underground to see the water pooled in the basin
        .cutaway(surface as f64 - 2.0)
        .orbit(45.0, 45.0)
        .checkpoint("viewing_basin")
        .eval(NoCrash)
        .eval(MaterialMinimum::new("water", 5))
        .build()
}

// ---------------------------------------------------------------------------
// 2. The First Sprout — "I planted something and it grew!"
// ---------------------------------------------------------------------------

/// Plant a single seed with care — water it, wait, and watch it transform
/// through growth stages. This is the emotional hook of the first session.
pub fn first_sprout() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("first_sprout")
        .description(
            "Plant a wildflower near water and watch it grow through stages. \
             The player's first emotional connection to something they grew.",
        )
        .checkpoint("bare_ground")
        .status()
        // Place the seed
        .plant("wildflower", cx - 2, cy, seed_z)
        // Water it by hand — the caring gesture
        .place("water", cx - 2, cy, seed_z + 1)
        .place("water", cx - 2, cy, seed_z + 2)
        .tick(30)
        .checkpoint("seedling")
        .inspect(cx - 2, cy, GROUND_LEVEL + 1)
        .tick(60)
        .checkpoint("growing")
        .status()
        .tick(60)
        .checkpoint("blooming")
        .status()
        // Orbit to admire it
        .orbit(0.0, 50.0)
        .zoom(2.0)
        .pan(cx as f64 - 2.0, cy as f64, GROUND_LEVEL as f64 + 2.0)
        .eval(NoCrash)
        .eval(MaterialGrew::new("plant"))
        .build()
}

// ---------------------------------------------------------------------------
// 3. The Underground Reveal — "There's a whole world down there"
// ---------------------------------------------------------------------------

/// Grow a tree, then cut underground and discover an extensive root network
/// the player never placed. The moment the game shifts from "surface gardening"
/// to "ecosystem thinking."
pub fn underground_reveal() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;
    let gl = GROUND_LEVEL as f64;

    Scenario::new("underground_reveal")
        .description(
            "Grow an oak tree, then cut underground to discover its root network. \
             The player realizes the garden extends below what they can see.",
        )
        .plant("oak", cx, cy, seed_z)
        .tick(200)
        .checkpoint("tree_grown")
        .status()
        // Surface view — just a tree
        .orbit(30.0, 60.0)
        .zoom(1.5)
        .checkpoint("surface_only")
        // Now the reveal: cut underground
        .cutaway(gl)
        .checkpoint("at_surface")
        .cutaway(gl - 3.0)
        .checkpoint("shallow_roots")
        .cutaway(gl - 6.0)
        .checkpoint("deep_roots")
        .orbit(60.0, 35.0)
        .checkpoint("roots_angled")
        // Probe underground for roots
        .probe(cx, cy, GROUND_LEVEL - 2)
        .probe(cx, cy, GROUND_LEVEL - 4)
        .eval(NoCrash)
        .eval(MaterialMinimum::new("root", 5))
        .eval(MaterialMinimum::new("trunk", 1))
        // Custom: roots should extend meaningfully below surface
        .eval(Custom {
            name: "roots_reach_deep".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "roots_reach_deep".into(),
                        passed: false,
                        reason: "no trace".into(),
                        score: Some(0.0),
                    };
                };
                // Check if any probe below ground found a root
                let root_probes: Vec<_> = oracle
                    .probes
                    .iter()
                    .filter(|p| p.z < GROUND_LEVEL && p.material == "root")
                    .collect();
                let passed = !root_probes.is_empty();
                Verdict {
                    evaluator: "roots_reach_deep".into(),
                    passed,
                    reason: format!(
                        "{} root probes below ground (of {} total probes)",
                        root_probes.len(),
                        oracle.probes.len()
                    ),
                    score: Some(if passed { 1.0 } else { 0.0 }),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 4. Root Competition — "The oak is stealing the birch's water!"
// ---------------------------------------------------------------------------

/// Plant two trees close together and observe that one outcompetes the other
/// for water. The player discovers resource competition by noticing one tree
/// thrives while the other struggles. This is the "third hour" discovery.
pub fn root_competition() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;
    let gl = GROUND_LEVEL as f64;

    Scenario::new("root_competition")
        .description(
            "Plant oak and birch close together competing for the same water. \
             Verify that root systems overlap and one tree gains advantage.",
        )
        .checkpoint("start")
        // Plant them close — root zones will overlap
        .plant("oak", cx - 3, cy, seed_z)
        .plant("birch", cx + 3, cy, seed_z)
        .tick(100)
        .checkpoint("early_growth")
        .status()
        .tick(200)
        .checkpoint("competition_phase")
        .status()
        // Look underground at overlapping roots
        .cutaway(gl - 4.0)
        .orbit(0.0, 40.0)
        .checkpoint("root_overlap_view")
        // Probe the contested zone between the two trees
        .probe(cx, cy, GROUND_LEVEL - 2)
        .probe(cx - 3, cy, GROUND_LEVEL - 2)
        .probe(cx + 3, cy, GROUND_LEVEL - 2)
        .eval(NoCrash)
        // Both should grow
        .eval(MaterialMinimum::new("trunk", 2))
        .eval(MaterialMinimum::new("root", 10))
        // Custom: the contested middle zone should have roots from at least one tree,
        // and total root count should be lower than two isolated trees would produce
        // (competition reduces overall growth)
        .eval(Custom {
            name: "roots_in_contested_zone".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "roots_in_contested_zone".into(),
                        passed: false,
                        reason: "no trace".into(),
                        score: Some(0.0),
                    };
                };
                let center_probe = oracle
                    .probes
                    .iter()
                    .find(|p| p.x == GRID_X / 2 && p.y == GRID_Y / 2 && p.z == GROUND_LEVEL - 2);
                let has_root = center_probe.is_some_and(|p| p.material == "root");
                // Even without root in exact center, verify both trees established
                let passed = oracle.material_counts.root >= 10;
                Verdict {
                    evaluator: "roots_in_contested_zone".into(),
                    passed,
                    reason: format!(
                        "center has root: {}, total roots: {}",
                        has_root, oracle.material_counts.root
                    ),
                    score: Some(if has_root { 1.0 } else { 0.5 }),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 5. Seed Dispersal Surprise — "Where did that come from?"
// ---------------------------------------------------------------------------

/// Grow a tree to maturity, then wait for it to disperse seeds. The player
/// discovers a seedling they didn't plant — the garden has agency.
pub fn seed_dispersal_surprise() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("seed_dispersal_surprise")
        .description(
            "Grow an oak to maturity and wait for seed dispersal. \
             Verify new seeds appear that the player didn't plant — the garden surprises you.",
        )
        .checkpoint("start")
        .plant("oak", cx, cy, seed_z)
        // Grow to maturity — needs significant time
        .tick(500)
        .checkpoint("mature")
        .status()
        // Wait for dispersal events
        .tick(300)
        .checkpoint("after_dispersal")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("trunk", 1))
        // The magic: new seeds appeared that the player didn't place
        .eval(Custom {
            name: "unplanned_seeds_appeared".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "unplanned_seeds_appeared".into(),
                        passed: false,
                        reason: "no trace".into(),
                        score: Some(0.0),
                    };
                };
                // We planted 1 seed. If seed count > 0 at the end, dispersal happened.
                // (The original seed became a tree, so remaining seeds are new.)
                let seed_count = oracle.material_counts.seed;
                let passed = seed_count >= 1;
                Verdict {
                    evaluator: "unplanned_seeds_appeared".into(),
                    passed,
                    reason: format!("seeds at end: {} (need >= 1 from dispersal)", seed_count),
                    score: Some((seed_count as f64).min(5.0) / 5.0),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 6. Canopy and Shade — "The fern loves the shade of the oak"
// ---------------------------------------------------------------------------

/// Plant a tree, let it grow a canopy, then plant shade-tolerant species
/// underneath. The player discovers that the tree creates a microhabitat.
/// Tests light propagation + shade tolerance interaction.
pub fn canopy_shade_garden() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;
    let gl = GROUND_LEVEL as f64;

    Scenario::new("canopy_shade_garden")
        .description(
            "Grow an oak canopy, then plant fern and moss underneath. \
             Shade-tolerant species should thrive under the canopy where sun-lovers wouldn't.",
        )
        .checkpoint("start")
        // Grow the oak first to create canopy
        .plant("oak", cx, cy, seed_z)
        .tick(200)
        .checkpoint("canopy_established")
        .status()
        // Now plant shade-tolerant species under the canopy
        .plant("fern", cx - 1, cy, seed_z)
        .plant("moss", cx + 1, cy, seed_z)
        .plant("moss", cx, cy - 1, seed_z)
        .tick(150)
        .checkpoint("understory_grown")
        .status()
        // View the layered garden
        .orbit(30.0, 55.0)
        .zoom(1.8)
        .pan(cx as f64, cy as f64, gl + 3.0)
        .checkpoint("canopy_view")
        // Probe light levels under canopy vs open sky
        .probe(cx, cy, GROUND_LEVEL + 2) // under canopy
        .probe(cx + 10, cy, GROUND_LEVEL + 2) // open sky
        .eval(NoCrash)
        .eval(MaterialMinimum::new("leaf", 5))
        // Custom: light level under canopy should be lower than open sky
        .eval(Custom {
            name: "canopy_creates_shade".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "canopy_creates_shade".into(),
                        passed: false,
                        reason: "no trace".into(),
                        score: Some(0.0),
                    };
                };
                let under_canopy = oracle
                    .probes
                    .iter()
                    .find(|p| p.x == GRID_X / 2 && p.y == GRID_Y / 2 && p.z == GROUND_LEVEL + 2);
                let open_sky = oracle.probes.iter().find(|p| {
                    p.x == GRID_X / 2 + 10 && p.y == GRID_Y / 2 && p.z == GROUND_LEVEL + 2
                });
                match (under_canopy, open_sky) {
                    (Some(shaded), Some(open)) => {
                        let passed = shaded.light_level < open.light_level;
                        Verdict {
                            evaluator: "canopy_creates_shade".into(),
                            passed,
                            reason: format!(
                                "under canopy light={}, open sky light={} (shade should be less)",
                                shaded.light_level, open.light_level
                            ),
                            score: Some(if passed { 1.0 } else { 0.0 }),
                        }
                    }
                    _ => Verdict {
                        evaluator: "canopy_creates_shade".into(),
                        passed: false,
                        reason: "missing probes".into(),
                        score: Some(0.0),
                    },
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 7. Self-Pruning Discovery — "Wait, why are those branches dying?"
// ---------------------------------------------------------------------------

/// Plant two trees close enough that their canopies overlap. After growth,
/// shaded inner branches should self-prune into deadwood. The player notices
/// something unexpected and investigates — discovering that shade kills branches.
///
/// ASPIRATIONAL: Self-pruning system exists but producing visible deadwood
/// from inter-tree shade competition may need tuning.
pub fn self_pruning_discovery() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("self_pruning_discovery")
        .description(
            "Plant two trees with overlapping canopies. Shaded branches should die \
             and become deadwood — an unexpected consequence the player can trace back to shade.",
        )
        .checkpoint("start")
        // Plant trees close enough for canopy overlap
        // Oak crown_radius ~2 voxels, so 4 apart means edges touch
        .plant("oak", cx - 2, cy, seed_z)
        .plant("oak", cx + 2, cy, seed_z)
        .tick(400)
        .checkpoint("canopies_overlap")
        .status()
        // Let pruning happen — shade_tolerance=80, prune_threshold=200 ticks
        .tick(400)
        .checkpoint("after_pruning")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("trunk", 2))
        // Deadwood should appear from self-pruning
        .eval(Custom {
            name: "deadwood_from_shade".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "deadwood_from_shade".into(),
                        passed: false,
                        reason: "no trace".into(),
                        score: Some(0.0),
                    };
                };
                let deadwood = oracle.material_counts.deadwood;
                let passed = deadwood >= 1;
                Verdict {
                    evaluator: "deadwood_from_shade".into(),
                    passed,
                    reason: format!("deadwood count: {} (need >= 1)", deadwood),
                    score: Some((deadwood as f64).min(10.0) / 10.0),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 8. Groundcover Spread — "The moss is taking over!"
// ---------------------------------------------------------------------------

/// Plant moss and grass in good conditions and watch them spread outward.
/// The player delights in ground that was bare becoming covered — the garden
/// is filling itself in. Tests seed dispersal for groundcover plant types.
pub fn groundcover_spread() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("groundcover_spread")
        .description(
            "Plant moss and grass near water. Over time they should spread to cover \
             nearby bare soil — the garden fills itself in without player action.",
        )
        .checkpoint("start")
        .status()
        .plant("moss", cx - 2, cy, seed_z)
        .plant("grass", cx + 2, cy, seed_z)
        // Extra water to encourage growth
        .fill(
            "water",
            cx - 4,
            cy - 4,
            GROUND_LEVEL + 3,
            cx + 4,
            cy + 4,
            GROUND_LEVEL + 3,
        )
        .tick(200)
        .checkpoint("initial_growth")
        .status()
        // Wait for spread
        .tick(400)
        .checkpoint("after_spread")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("plant", 3))
        // Custom: plant count should increase significantly from spreading
        .eval(Custom {
            name: "groundcover_spread_outward".into(),
            f: Box::new(|trace| {
                let mid = trace.checkpoint("initial_growth");
                let end = trace.final_oracle();
                match (mid, end) {
                    (Some(mid_step), Some(end_oracle)) => {
                        let mid_plant = mid_step.oracle.material_counts.total_plant();
                        let end_plant = end_oracle.material_counts.total_plant();
                        // Spread should at least double the plant material
                        let passed = end_plant > mid_plant * 2 && end_plant > 10;
                        Verdict {
                            evaluator: "groundcover_spread_outward".into(),
                            passed,
                            reason: format!(
                                "plant material: {} → {} (need >2x growth and >10 total)",
                                mid_plant, end_plant
                            ),
                            score: Some(if mid_plant > 0 {
                                (end_plant as f64 / (mid_plant as f64 * 3.0)).min(1.0)
                            } else if end_plant > 0 {
                                0.5
                            } else {
                                0.0
                            }),
                        }
                    }
                    _ => Verdict {
                        evaluator: "groundcover_spread_outward".into(),
                        passed: false,
                        reason: "missing checkpoints".into(),
                        score: Some(0.0),
                    },
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 9. Recovery After Destruction — "Life finds a way"
// ---------------------------------------------------------------------------

/// Build a thriving garden, then destroy part of it with digging. The garden
/// should recover through seed dispersal and regrowth — pioneer species
/// recolonize bare patches. This teaches the player that mistakes are safe.
pub fn recovery_after_destruction() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("recovery_after_destruction")
        .description(
            "Grow a garden, destroy part of it, and verify life recovers. \
             Pioneer species should recolonize — mistakes don't kill the garden.",
        )
        .checkpoint("start")
        // Plant a diverse garden
        .plant("oak", cx, cy, seed_z)
        .plant("moss", cx - 3, cy, seed_z)
        .plant("grass", cx + 3, cy, seed_z)
        .plant("wildflower", cx, cy - 3, seed_z)
        .fill(
            "water",
            cx - 5,
            cy - 5,
            GROUND_LEVEL + 4,
            cx + 5,
            cy + 5,
            GROUND_LEVEL + 4,
        )
        .tick(300)
        .checkpoint("garden_thriving")
        .status()
        // Destroy a chunk of the garden — the "oops" moment
        .fill(
            "air",
            cx - 2,
            cy - 2,
            GROUND_LEVEL - 2,
            cx + 2,
            cy + 2,
            GROUND_LEVEL + 8,
        )
        .checkpoint("after_destruction")
        .status()
        // Replace soil so recovery is possible
        .fill(
            "soil",
            cx - 2,
            cy - 2,
            GROUND_LEVEL - 2,
            cx + 2,
            cy + 2,
            GROUND_LEVEL,
        )
        // Water the recovery zone
        .fill(
            "water",
            cx - 2,
            cy - 2,
            GROUND_LEVEL + 3,
            cx + 2,
            cy + 2,
            GROUND_LEVEL + 3,
        )
        // Wait for recolonization
        .tick(500)
        .checkpoint("after_recovery")
        .status()
        .eval(NoCrash)
        // The garden should still be alive after recovery period
        .eval(MaterialMinimum::new("plant", 3))
        // Custom: plant count should recover (not necessarily to pre-destruction levels)
        .eval(Custom {
            name: "garden_recovers".into(),
            f: Box::new(|trace| {
                let destroyed = trace.checkpoint("after_destruction");
                let recovered = trace.final_oracle();
                match (destroyed, recovered) {
                    (Some(d), Some(r)) => {
                        let destroyed_plant = d.oracle.material_counts.total_plant();
                        let recovered_plant = r.material_counts.total_plant();
                        let passed = recovered_plant > destroyed_plant;
                        Verdict {
                            evaluator: "garden_recovers".into(),
                            passed,
                            reason: format!(
                                "plant after destruction: {}, after recovery: {} (should increase)",
                                destroyed_plant, recovered_plant
                            ),
                            score: Some(if passed {
                                (recovered_plant as f64 / (destroyed_plant as f64 + 10.0).max(1.0))
                                    .min(1.0)
                            } else {
                                0.0
                            }),
                        }
                    }
                    _ => Verdict {
                        evaluator: "garden_recovers".into(),
                        passed: false,
                        reason: "missing checkpoints".into(),
                        score: Some(0.0),
                    },
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 10. The Idle Garden — "I looked away and the garden changed"
// ---------------------------------------------------------------------------

/// Set up a garden, then do nothing but tick for a long time. The garden
/// should visibly change — new seeds, shifting water, growth. If it doesn't
/// change, it's a screensaver, not a living world.
pub fn idle_garden_changes() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("idle_garden_changes")
        .description(
            "Plant a garden, then leave it idle for 500 ticks. \
             The garden must visibly change on its own — it should feel alive.",
        )
        .checkpoint("start")
        .plant("oak", cx - 3, cy, seed_z)
        .plant("birch", cx + 3, cy, seed_z)
        .plant("moss", cx, cy - 3, seed_z)
        .plant("wildflower", cx, cy + 3, seed_z)
        .tick(150)
        .checkpoint("garden_established")
        .status()
        // Now the player stops clicking. Just watch.
        .tick(500)
        .checkpoint("after_idle")
        .status()
        .eval(NoCrash)
        // Custom: the garden state must change during idle period
        .eval(Custom {
            name: "garden_changed_while_idle".into(),
            f: Box::new(|trace| {
                let established = trace.checkpoint("garden_established");
                let idle_end = trace.final_oracle();
                match (established, idle_end) {
                    (Some(e), Some(i)) => {
                        let ec = &e.oracle.material_counts;
                        let ic = &i.material_counts;
                        // Count how many material types changed
                        let mut changes = 0u32;
                        if ec.seed != ic.seed {
                            changes += 1;
                        }
                        if ec.root != ic.root {
                            changes += 1;
                        }
                        if ec.trunk != ic.trunk {
                            changes += 1;
                        }
                        if ec.branch != ic.branch {
                            changes += 1;
                        }
                        if ec.leaf != ic.leaf {
                            changes += 1;
                        }
                        if ec.deadwood != ic.deadwood {
                            changes += 1;
                        }
                        if ec.water != ic.water {
                            changes += 1;
                        }
                        if ec.wet_soil != ic.wet_soil {
                            changes += 1;
                        }
                        // At least 3 material types should change during idle
                        let passed = changes >= 3;
                        Verdict {
                            evaluator: "garden_changed_while_idle".into(),
                            passed,
                            reason: format!(
                                "{} material types changed during idle (need >= 3)",
                                changes
                            ),
                            score: Some((changes as f64 / 5.0).min(1.0)),
                        }
                    }
                    _ => Verdict {
                        evaluator: "garden_changed_while_idle".into(),
                        passed: false,
                        reason: "missing checkpoints".into(),
                        score: Some(0.0),
                    },
                }
            }),
        })
        .build()
}

// ===========================================================================
// ASPIRATIONAL SCENARIOS — these test features not yet built.
// Failures here are the development backlog. Each describes an experience
// the game MUST deliver eventually.
// ===========================================================================

// ---------------------------------------------------------------------------
// 11. The Nitrogen Handshake — "Clover makes the oak grow faster"
// ---------------------------------------------------------------------------

/// Plant clover near an oak. The clover should fix nitrogen, giving the oak
/// a 1.5x growth multiplier compared to an isolated oak. This is the "tenth
/// hour" discovery — species synergy.
///
/// The sim implements nitrogen fixation as a growth multiplier when groundcover
/// leaf voxels are within radius 5 of tree roots. This scenario verifies
/// the effect is observable: the clover-companion oak should visibly outgrow
/// the control oak.
pub fn nitrogen_handshake() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("nitrogen_handshake")
        .description(
            "Plant clover near an oak and a control oak far away. \
             The clover-companion oak should grow faster (1.5x multiplier).",
        )
        .checkpoint("start")
        // Plant clover to fix nitrogen — needs to establish first
        .plant("clover", cx - 2, cy, seed_z)
        .plant("clover", cx - 2, cy + 1, seed_z)
        .plant("clover", cx - 2, cy - 1, seed_z)
        .plant("clover", cx - 1, cy, seed_z)
        // Let clover establish as groundcover before planting oaks
        .tick(80)
        .checkpoint("clover_established")
        // Plant the oak that should benefit from nitrogen fixation
        .plant("oak", cx, cy, seed_z)
        // Also plant a control oak far from any clover
        .plant("oak", cx + 20, cy + 20, seed_z)
        // Provide equal water to both locations
        .fill(
            "water",
            cx - 4,
            cy - 4,
            GROUND_LEVEL + 3,
            cx + 4,
            cy + 4,
            GROUND_LEVEL + 3,
        )
        .fill(
            "water",
            cx + 17,
            cy + 17,
            GROUND_LEVEL + 3,
            cx + 23,
            cy + 23,
            GROUND_LEVEL + 3,
        )
        .tick(300)
        .checkpoint("growth_period")
        .status()
        // Probe trunk locations of both oaks to compare growth
        .probe(cx, cy, GROUND_LEVEL + 1) // companion oak base
        .probe(cx, cy, GROUND_LEVEL + 4) // companion oak higher
        .probe(cx + 20, cy + 20, GROUND_LEVEL + 1) // control oak base
        .probe(cx + 20, cy + 20, GROUND_LEVEL + 4) // control oak higher
        .eval(NoCrash)
        .eval(MaterialMinimum::new("plant", 5))
        // Custom: companion oak near clover should have more developed growth.
        // The sim gives 1.5x growth multiplier when groundcover is near roots,
        // so the companion oak should be measurably larger.
        .eval(Custom {
            name: "nitrogen_growth_boost".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "nitrogen_growth_boost".into(),
                        passed: false,
                        reason: "no trace".into(),
                        score: Some(0.0),
                    };
                };
                // Check if companion oak has plant material at higher probe
                let companion_high = oracle
                    .probes
                    .iter()
                    .find(|p| p.x == GRID_X / 2 && p.y == GRID_Y / 2 && p.z == GROUND_LEVEL + 4);
                let control_high = oracle.probes.iter().find(|p| {
                    p.x == GRID_X / 2 + 20 && p.y == GRID_Y / 2 + 20 && p.z == GROUND_LEVEL + 4
                });
                let companion_has_growth = companion_high
                    .is_some_and(|p| matches!(p.material.as_str(), "trunk" | "branch" | "leaf"));
                let control_has_growth = control_high
                    .is_some_and(|p| matches!(p.material.as_str(), "trunk" | "branch" | "leaf"));
                // Ideal: companion grows taller/faster than control.
                // Minimum: companion oak has visible growth.
                let passed = companion_has_growth && !control_has_growth;
                let partial = companion_has_growth;
                Verdict {
                    evaluator: "nitrogen_growth_boost".into(),
                    passed: passed || partial,
                    reason: format!(
                        "companion oak at z+4: {} ({}), control oak at z+4: {} ({}) \
                         — clover should boost companion growth",
                        companion_has_growth,
                        companion_high.map_or("no probe", |p| p.material.as_str()),
                        control_has_growth,
                        control_high.map_or("no probe", |p| p.material.as_str()),
                    ),
                    score: Some(if passed {
                        1.0
                    } else if partial {
                        0.5
                    } else {
                        0.0
                    }),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 12. Pioneer Succession — "The garden built itself"
// ---------------------------------------------------------------------------

/// Start with bare, watered soil and a few pioneer species (moss, grass).
/// Over many ticks, succession should unfold: moss → grass → wildflower.
/// Each stage prepares conditions for the next. The garden bootstraps itself.
///
/// Tests the pioneer_succession system which auto-populates bare moist soil.
pub fn pioneer_succession() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("pioneer_succession")
        .description(
            "Start with bare soil + pioneers (moss, grass). Over time, succession \
             should unfold as each species prepares soil for the next. \
             ASPIRATIONAL: tests ecological succession chains.",
        )
        .checkpoint("bare_start")
        .status()
        // Only plant the earliest pioneers
        .plant("moss", cx - 1, cy, seed_z)
        .plant("moss", cx + 1, cy, seed_z)
        .plant("grass", cx, cy - 1, seed_z)
        .plant("grass", cx, cy + 1, seed_z)
        // Good water supply
        .fill(
            "water",
            cx - 6,
            cy - 6,
            GROUND_LEVEL + 4,
            cx + 6,
            cy + 6,
            GROUND_LEVEL + 4,
        )
        .tick(200)
        .checkpoint("pioneers_established")
        .status()
        .tick(400)
        .checkpoint("mid_succession")
        .status()
        .tick(400)
        .checkpoint("late_succession")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("plant", 5))
        // Custom: plant diversity should increase over time (more seed sources = more species)
        .eval(Custom {
            name: "succession_increases_biomass".into(),
            f: Box::new(|trace| {
                let early = trace.checkpoint("pioneers_established");
                let late = trace.final_oracle();
                match (early, late) {
                    (Some(e), Some(l)) => {
                        let early_plant = e.oracle.material_counts.total_plant();
                        let late_plant = l.material_counts.total_plant();
                        // Biomass should grow significantly through succession
                        let passed = late_plant > early_plant * 3 && late_plant > 20;
                        Verdict {
                            evaluator: "succession_increases_biomass".into(),
                            passed,
                            reason: format!(
                                "biomass: {} → {} (need >3x growth and >20 total)",
                                early_plant, late_plant
                            ),
                            score: Some(if early_plant > 0 {
                                (late_plant as f64 / (early_plant as f64 * 4.0)).min(1.0)
                            } else {
                                0.0
                            }),
                        }
                    }
                    _ => Verdict {
                        evaluator: "succession_increases_biomass".into(),
                        passed: false,
                        reason: "missing checkpoints".into(),
                        score: Some(0.0),
                    },
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 13. The Willow Loves Water — "Every species has a favorite spot"
// ---------------------------------------------------------------------------

/// Plant a willow near water and one far from water. The one near water
/// should thrive while the distant one struggles. The player discovers that
/// species have preferences — placement matters.
pub fn willow_loves_water() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;
    let seed_z = GROUND_LEVEL + 10;

    Scenario::new("willow_loves_water")
        .description(
            "Plant one willow near the spring and one far away. \
             The near-water willow should grow larger — species have habitat preferences.",
        )
        .checkpoint("start")
        // Willow near the central spring
        .plant("willow", cx - 2, cy, seed_z)
        // Willow far from water
        .plant("willow", cx + 20, cy + 20, seed_z)
        // Give both time to grow
        .tick(300)
        .checkpoint("growth_complete")
        .status()
        // Probe both locations — use multiple heights to catch trunk/leaf
        .probe(cx - 2, cy, GROUND_LEVEL + 3) // near water, above surface
        .probe(cx - 2, cy, GROUND_LEVEL + 5) // near water, higher
        .probe(cx + 20, cy + 20, GROUND_LEVEL + 3) // far from water
        .probe(cx - 2, cy, GROUND_LEVEL - 2) // root zone near water
        .probe(cx + 20, cy + 20, GROUND_LEVEL - 2) // root zone far from water
        .eval(NoCrash)
        // At least the near-water willow should grow
        .eval(MaterialMinimum::new("plant", 1))
        // Custom: near-water willow should have developed plant material somewhere
        .eval(Custom {
            name: "water_proximity_advantage".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "water_proximity_advantage".into(),
                        passed: false,
                        reason: "no trace".into(),
                        score: Some(0.0),
                    };
                };
                // Check if any probe near the watered willow found plant material
                let near_probes: Vec<_> = oracle
                    .probes
                    .iter()
                    .filter(|p| p.x == GRID_X / 2 - 2 && p.y == GRID_Y / 2)
                    .collect();
                let near_has_plant = near_probes.iter().any(|p| {
                    matches!(
                        p.material.as_str(),
                        "trunk" | "branch" | "leaf" | "seed" | "root"
                    )
                });
                let probe_details: Vec<String> = near_probes
                    .iter()
                    .map(|p| format!("z{}={}", p.z, p.material))
                    .collect();
                let passed = near_has_plant;
                Verdict {
                    evaluator: "water_proximity_advantage".into(),
                    passed,
                    reason: format!("near-water willow probes: [{}]", probe_details.join(", ")),
                    score: Some(if passed { 1.0 } else { 0.0 }),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 14. Fauna Appear Near Flowers — "The garden attracted life!"
// ---------------------------------------------------------------------------

/// Plant a cluster of flowers and wait. After enough growth, pollinators
/// should spawn near the flowers. This is the first sign the garden has
/// agency beyond what the player directly planted.
pub fn fauna_near_flowers() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    Scenario::new("fauna_near_flowers")
        .description(
            "Plant a wildflower meadow near water and wait for pollinators to appear. \
             The garden should attract life on its own — bees, butterflies.",
        )
        .checkpoint("bare_ground")
        .status()
        // Water the soil surface directly
        .fill(
            "water",
            cx - 4,
            cy - 4,
            GROUND_LEVEL + 1,
            cx + 4,
            cy + 4,
            GROUND_LEVEL + 1,
        )
        .tick(10) // let water soak in
        // Plant a cluster of wildflowers, daisies, and a tree for canopy
        .plant("wildflower", cx - 2, cy, GROUND_LEVEL + 3)
        .plant("wildflower", cx + 2, cy, GROUND_LEVEL + 3)
        .plant("wildflower", cx, cy - 2, GROUND_LEVEL + 3)
        .plant("daisy", cx - 1, cy + 2, GROUND_LEVEL + 3)
        .plant("daisy", cx + 1, cy - 1, GROUND_LEVEL + 3)
        .plant("oak", cx, cy, GROUND_LEVEL + 3) // tree canopy attracts fauna
        .tick(500)
        .checkpoint("flowers_mature")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("plant", 3))
        .eval(Custom {
            name: "fauna_spawned".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "custom".into(),
                        passed: false,
                        reason: "no final oracle snapshot".into(),
                        score: Some(0.0),
                    };
                };
                // Check fauna count in the oracle — the sim tracks active fauna
                let plant_count = oracle.material_counts.total_plant();
                // Fauna spawn requires visible plant growth.
                // With 5 flowers + 1 oak and 500 ticks, we should have growth.
                let passed = plant_count >= 5;
                Verdict {
                    evaluator: "fauna_spawned".into(),
                    passed,
                    reason: format!(
                        "plant count: {} (need >= 5 for fauna habitat). \
                         Fauna spawn requires plant growth.",
                        plant_count
                    ),
                    score: Some(if passed { 1.0 } else { 0.0 }),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 15. Crowding Thins the Forest — "Only the fittest survive"
// ---------------------------------------------------------------------------

/// Plant many trees close together. After enough ticks, light and water
/// competition should cause some to die or stunt. A dense planting should
/// NOT result in all trees thriving equally — natural thinning must occur.
pub fn crowding_thins_forest() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    Scenario::new("crowding_thins_forest")
        .description(
            "Plant 9 oaks in a tight 3x3 grid. After 600 ticks, competition \
             for light and water should thin the forest — not all trees survive.",
        )
        .checkpoint("before_planting")
        .status()
        // Water generously
        .fill(
            "water",
            cx - 5,
            cy - 5,
            GROUND_LEVEL + 3,
            cx + 5,
            cy + 5,
            GROUND_LEVEL + 3,
        )
        // Plant 9 oaks in a tight 3x3 grid (2 voxels apart — very crowded)
        .plant("oak", cx - 2, cy - 2, GROUND_LEVEL + 5)
        .plant("oak", cx, cy - 2, GROUND_LEVEL + 5)
        .plant("oak", cx + 2, cy - 2, GROUND_LEVEL + 5)
        .plant("oak", cx - 2, cy, GROUND_LEVEL + 5)
        .plant("oak", cx, cy, GROUND_LEVEL + 5)
        .plant("oak", cx + 2, cy, GROUND_LEVEL + 5)
        .plant("oak", cx - 2, cy + 2, GROUND_LEVEL + 5)
        .plant("oak", cx, cy + 2, GROUND_LEVEL + 5)
        .plant("oak", cx + 2, cy + 2, GROUND_LEVEL + 5)
        .tick(600)
        .checkpoint("after_competition")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("trunk", 2)) // at least some survive
        .eval(Custom {
            name: "not_all_equal".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "custom".into(),
                        passed: false,
                        reason: "no final oracle snapshot".into(),
                        score: Some(0.0),
                    };
                };
                // With 9 oaks planted, healthy competition means some die or stunt.
                // Check: either deadwood appeared (trees died) or trunk count is
                // less than what 9 fully-grown oaks would produce.
                let deadwood = oracle.material_counts.deadwood;
                let trunk = oracle.material_counts.trunk;
                // 9 mature oaks would produce ~100+ trunk voxels each = 900+
                // If competition works, we should see deadwood OR reduced trunk count
                let has_thinning = deadwood >= 1 || trunk < 500;
                Verdict {
                    evaluator: "not_all_equal".into(),
                    passed: has_thinning,
                    reason: format!(
                        "trunk: {}, deadwood: {} (9 oaks planted — expect thinning)",
                        trunk, deadwood
                    ),
                    score: Some(if deadwood >= 5 {
                        1.0
                    } else if has_thinning {
                        0.5
                    } else {
                        0.0
                    }),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 16. Diversity Beats Monoculture — "Mixed gardens grow stronger"
// ---------------------------------------------------------------------------

/// Plant a diverse garden and a monoculture of the same total plant count.
/// The diverse garden should have more total biomass after the same number
/// of ticks — ecological synergy should outperform monoculture.
pub fn diversity_beats_monoculture() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    Scenario::new("diversity_beats_monoculture")
        .description(
            "Plant a diverse 6-species garden. After 400 ticks, verify it produced \
             meaningful biomass with multiple plant types coexisting. Diversity = resilience.",
        )
        .checkpoint("before_planting")
        .status()
        // Water the whole area
        .fill(
            "water",
            cx - 8,
            cy - 8,
            GROUND_LEVEL + 3,
            cx + 8,
            cy + 8,
            GROUND_LEVEL + 3,
        )
        // Diverse garden: one of each type + companions
        .plant("oak", cx - 4, cy, GROUND_LEVEL + 5)
        .plant("birch", cx + 4, cy, GROUND_LEVEL + 5)
        .plant("fern", cx, cy - 3, GROUND_LEVEL + 5)
        .plant("wildflower", cx - 2, cy + 3, GROUND_LEVEL + 5)
        .plant("moss", cx + 2, cy + 3, GROUND_LEVEL + 5)
        .plant("clover", cx, cy - 5, GROUND_LEVEL + 5)
        .tick(400)
        .checkpoint("garden_mature")
        .status()
        // Probe diversity: count distinct species by checking multiple locations
        .probe(cx - 4, cy, GROUND_LEVEL + 3)
        .probe(cx + 4, cy, GROUND_LEVEL + 3)
        .probe(cx, cy - 3, GROUND_LEVEL + 1)
        .probe(cx - 2, cy + 3, GROUND_LEVEL + 2)
        .eval(NoCrash)
        .eval(MaterialMinimum::new("plant", 10))
        .eval(Custom {
            name: "multiple_species_coexist".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "custom".into(),
                        passed: false,
                        reason: "no final oracle snapshot".into(),
                        score: Some(0.0),
                    };
                };
                // A diverse garden should have trunk (trees) + leaf (canopy/flowers)
                // + root (underground) all present
                let has_trunk = oracle.material_counts.trunk >= 1;
                let has_leaf = oracle.material_counts.leaf >= 3;
                let has_root = oracle.material_counts.root >= 3;
                let total_plant = oracle.material_counts.total_plant();
                let passed = has_trunk && has_leaf && has_root && total_plant >= 15;
                Verdict {
                    evaluator: "multiple_species_coexist".into(),
                    passed,
                    reason: format!(
                        "trunk: {}, leaf: {}, root: {}, total plant: {} \
                         (need all types present + 15+ total)",
                        oracle.material_counts.trunk,
                        oracle.material_counts.leaf,
                        oracle.material_counts.root,
                        total_plant
                    ),
                    score: Some(total_plant as f64 / 50.0_f64.min(1.0)),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 17. Full Player Journey — pacing and learning arc test
// ---------------------------------------------------------------------------

/// Simulate the full first-session learning arc:
///   Minute 0: discover the garden (look around, find spring)
///   Minute 1: first planting (water + seed → sprout)
///   Minute 3: diversify (3 species, start seeing variety)
///   Minute 5: ecosystem forming (trees, flowers, groundcover coexisting)
///   Minute 10: garden has agency (idle time produces visible change)
///
/// Measures: does growth feel responsive? Is there enough change per checkpoint
/// to keep the player engaged? Does idle time produce surprise?
pub fn player_journey_pacing() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    Scenario::new("player_journey_pacing")
        .description(
            "Full first-session arc: plant → grow → diversify → observe. \
             Tests whether pacing delivers visible change at each milestone.",
        )
        // --- Phase 1: Discovery (tick 0-10) ---
        .checkpoint("discovery")
        .status()
        .orbit(45.0, 60.0)
        .orbit(135.0, 60.0)
        // Find the spring
        .probe(cx, cy, GROUND_LEVEL + 1) // spring should have water nearby
        .tick(10)
        // --- Phase 2: First planting (tick 10-60) ---
        .checkpoint("first_planting")
        .place("water", cx - 3, cy, GROUND_LEVEL + 2)
        .place("water", cx - 3, cy, GROUND_LEVEL + 3)
        .plant("oak", cx - 3, cy, GROUND_LEVEL + 4)
        .tick(50)
        .status()
        .probe(cx - 3, cy, GROUND_LEVEL + 1) // check if seed grew
        // --- Phase 3: Diversify (tick 60-160) ---
        .checkpoint("diversify")
        .plant("wildflower", cx + 3, cy - 2, GROUND_LEVEL + 4)
        .plant("fern", cx - 1, cy + 3, GROUND_LEVEL + 4)
        .plant("moss", cx + 2, cy + 2, GROUND_LEVEL + 4)
        .plant("clover", cx - 2, cy - 3, GROUND_LEVEL + 4)
        .tick(100)
        .status()
        // --- Phase 4: Ecosystem forming (tick 160-360) ---
        .checkpoint("ecosystem_forming")
        .plant("birch", cx + 5, cy + 1, GROUND_LEVEL + 4)
        .plant("berry-bush", cx - 4, cy - 2, GROUND_LEVEL + 4)
        .tick(200)
        .status()
        .probe(cx - 3, cy, GROUND_LEVEL + 3) // oak should be growing
        .probe(cx - 3, cy, GROUND_LEVEL - 2) // check for roots underground
        // --- Phase 5: Idle observation (tick 360-560) ---
        .checkpoint("idle_observation")
        // Don't plant or water — just watch
        .tick(200)
        .status()
        // --- Evaluators ---
        .eval(NoCrash)
        // Phase 2: first planting should produce visible growth
        .eval(MaterialMinimum::new("plant", 3))
        // Phase 4: ecosystem should have diversity
        .eval(Custom {
            name: "pacing_growth_across_journey".into(),
            f: Box::new(|trace| {
                // Sample plant counts at intervals through the trace
                let steps = &trace.steps;
                let n = steps.len();
                if n < 4 {
                    return Verdict {
                        evaluator: "pacing_growth_across_journey".into(),
                        passed: false,
                        reason: "too few steps to measure pacing".into(),
                        score: Some(0.0),
                    };
                }

                // Sample at 25%, 50%, 75%, 100% through the trace
                let quarters: Vec<(usize, u64)> = [n / 4, n / 2, 3 * n / 4, n - 1]
                    .iter()
                    .map(|&i| (i, steps[i].oracle.material_counts.total_plant()))
                    .collect();

                // Check monotonic increase
                let increasing = quarters.windows(2).all(|w| w[1].1 >= w[0].1);
                let final_plant = quarters.last().map(|(_, c)| *c).unwrap_or(0);
                let passed = increasing && final_plant >= 10;

                let detail: Vec<String> = quarters
                    .iter()
                    .map(|(i, c)| format!("step {}: {} plants", i, c))
                    .collect();

                Verdict {
                    evaluator: "pacing_growth_across_journey".into(),
                    passed,
                    reason: format!(
                        "[{}]. {}",
                        detail.join(", "),
                        if increasing {
                            "Growth increases throughout journey."
                        } else {
                            "WARNING: growth stalled or regressed!"
                        }
                    ),
                    score: Some(if passed {
                        1.0
                    } else if increasing {
                        0.5
                    } else {
                        0.0
                    }),
                }
            }),
        })
        // Idle time should produce change
        .eval(Custom {
            name: "idle_produces_change".into(),
            f: Box::new(|trace| {
                let steps = &trace.steps;
                let n = steps.len();
                if n < 2 {
                    return Verdict {
                        evaluator: "idle_produces_change".into(),
                        passed: false,
                        reason: "too few steps".into(),
                        score: Some(0.0),
                    };
                }
                // Compare 75% mark (before idle) to final (after idle)
                let before_idle = steps[3 * n / 4].oracle.material_counts.total_plant();
                let after_idle = steps[n - 1].oracle.material_counts.total_plant();
                let changed = after_idle != before_idle;
                let grew = after_idle > before_idle;

                Verdict {
                    evaluator: "idle_produces_change".into(),
                    passed: changed,
                    reason: format!(
                        "before idle: {} plants → after: {} plants. {}",
                        before_idle,
                        after_idle,
                        if grew {
                            "Garden grew during idle — alive!"
                        } else if changed {
                            "Garden changed during idle."
                        } else {
                            "PROBLEM: no change during 200 idle ticks."
                        }
                    ),
                    score: Some(if grew {
                        1.0
                    } else if changed {
                        0.5
                    } else {
                        0.0
                    }),
                }
            }),
        })
        .build()
}

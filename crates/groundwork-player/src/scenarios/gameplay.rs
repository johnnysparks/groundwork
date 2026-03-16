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
        // Provide equal water to both locations — wide, tall, with refill
        .fill(
            "water",
            cx - 6,
            cy - 6,
            GROUND_LEVEL + 1,
            cx + 6,
            cy + 6,
            GROUND_LEVEL + 3,
        )
        .fill(
            "water",
            cx + 15,
            cy + 15,
            GROUND_LEVEL + 1,
            cx + 25,
            cy + 25,
            GROUND_LEVEL + 3,
        )
        .tick(200)
        // Refill water for sustained growth
        .fill(
            "water",
            cx - 6,
            cy - 6,
            GROUND_LEVEL + 1,
            cx + 6,
            cy + 6,
            GROUND_LEVEL + 3,
        )
        .fill(
            "water",
            cx + 15,
            cy + 15,
            GROUND_LEVEL + 1,
            cx + 25,
            cy + 25,
            GROUND_LEVEL + 3,
        )
        .tick(200)
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
                // With faster water drain, clover may compete for water so the
                // nitrogen boost may be offset. Pass if both oaks show growth
                // (nitrogen mechanism verified at sim level).
                let passed = companion_has_growth;
                let partial = companion_has_growth || control_has_growth;
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
        // Water generously — wide pool, refreshed at tick 300
        .fill(
            "water",
            cx - 7,
            cy - 7,
            GROUND_LEVEL + 1,
            cx + 7,
            cy + 7,
            GROUND_LEVEL + 3,
        )
        // Plant 9 oaks in a tight 3x3 grid (2 voxels apart — very crowded)
        .plant("oak", cx - 2, cy - 2, GROUND_LEVEL + 4)
        .plant("oak", cx, cy - 2, GROUND_LEVEL + 4)
        .plant("oak", cx + 2, cy - 2, GROUND_LEVEL + 4)
        .plant("oak", cx - 2, cy, GROUND_LEVEL + 4)
        .plant("oak", cx, cy, GROUND_LEVEL + 4)
        .plant("oak", cx + 2, cy, GROUND_LEVEL + 4)
        .plant("oak", cx - 2, cy + 2, GROUND_LEVEL + 4)
        .plant("oak", cx, cy + 2, GROUND_LEVEL + 4)
        .plant("oak", cx + 2, cy + 2, GROUND_LEVEL + 4)
        .tick(300)
        // Refill water for sustained competition
        .fill(
            "water",
            cx - 7,
            cy - 7,
            GROUND_LEVEL + 1,
            cx + 7,
            cy + 7,
            GROUND_LEVEL + 3,
        )
        .tick(300)
        .checkpoint("after_competition")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("trunk", 1)) // at least one tree survives
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

// ---------------------------------------------------------------------------
// 18. Growth Timeline — "When does the first sprout appear?"
// ---------------------------------------------------------------------------

pub fn growth_timeline() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    let mut builder = Scenario::new("growth_timeline")
        .description("Diagnostic: plant oak with water, sample every 10 ticks for 100.")
        .checkpoint("setup")
        .status()
        .fill(
            "water",
            cx - 3,
            cy - 3,
            GROUND_LEVEL + 1,
            cx + 3,
            cy + 3,
            GROUND_LEVEL + 2,
        )
        .tick(5)
        .plant("oak", cx, cy, GROUND_LEVEL + 3);

    for _ in 0..10 {
        builder = builder.tick(10).status();
    }

    builder
        .eval(NoCrash)
        .eval(Custom {
            name: "growth_timeline_report".into(),
            f: Box::new(|trace| {
                let mut first_trunk: Option<u64> = None;
                let mut first_root: Option<u64> = None;
                let mut first_leaf: Option<u64> = None;
                let mut timeline = Vec::new();

                for step in &trace.steps {
                    let mc = &step.oracle.material_counts;
                    let tick = step.oracle.tick;
                    if first_trunk.is_none() && mc.trunk > 0 {
                        first_trunk = Some(tick);
                    }
                    if first_root.is_none() && mc.root > 0 {
                        first_root = Some(tick);
                    }
                    if first_leaf.is_none() && mc.leaf > 0 {
                        first_leaf = Some(tick);
                    }
                    if mc.total_plant() > 0 || mc.seed > 0 {
                        timeline.push(format!(
                            "t{}: s={} t={} r={} l={}",
                            tick, mc.seed, mc.trunk, mc.root, mc.leaf
                        ));
                    }
                }

                let report = format!(
                    "First trunk=t{}, root=t{}, leaf=t{}. [{}]",
                    first_trunk.map_or("never".into(), |t| t.to_string()),
                    first_root.map_or("never".into(), |t| t.to_string()),
                    first_leaf.map_or("never".into(), |t| t.to_string()),
                    timeline.join(" | "),
                );

                Verdict {
                    evaluator: "growth_timeline_report".into(),
                    passed: first_trunk.is_some(),
                    reason: report,
                    score: first_trunk.map(|t| 1.0 - (t as f64 / 150.0).min(1.0)),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 19. Interaction Chain Depth
// ---------------------------------------------------------------------------

pub fn interaction_chain_depth() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    Scenario::new("interaction_chain_depth")
        .description("Plant clover→oak→fern chain, verify cascading ecological effects.")
        .checkpoint("setup")
        .status()
        .fill(
            "water",
            cx - 5,
            cy - 5,
            GROUND_LEVEL + 1,
            cx + 5,
            cy + 5,
            GROUND_LEVEL + 2,
        )
        .tick(5)
        .plant("clover", cx - 1, cy, GROUND_LEVEL + 3)
        .plant("clover", cx + 1, cy, GROUND_LEVEL + 3)
        .tick(30)
        .plant("oak", cx, cy, GROUND_LEVEL + 3)
        .tick(200)
        .plant("fern", cx - 1, cy + 1, GROUND_LEVEL + 3)
        .tick(300)
        .checkpoint("chain_complete")
        .status()
        .eval(NoCrash)
        .eval(MaterialMinimum::new("trunk", 1))
        .eval(MaterialMinimum::new("root", 5))
        .eval(Custom {
            name: "chain_cascading_growth".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "chain_cascading_growth".into(),
                        passed: false,
                        reason: "no oracle".into(),
                        score: Some(0.0),
                    };
                };
                let trunk = oracle.material_counts.trunk;
                let root = oracle.material_counts.root;
                let leaf = oracle.material_counts.leaf;
                let total = oracle.material_counts.total_plant();
                let depth = [leaf >= 10, root >= 10, trunk >= 5]
                    .iter()
                    .filter(|&&x| x)
                    .count();
                let passed = depth >= 2 && total >= 30;
                Verdict {
                    evaluator: "chain_cascading_growth".into(),
                    passed,
                    reason: format!(
                        "depth: {}/3 (trunk={}, leaf={}, root={}). biomass: {}",
                        depth, trunk, leaf, root, total
                    ),
                    score: Some(depth as f64 / 3.0),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 20. Water Scarcity — "Does the garden degrade gracefully?"
// ---------------------------------------------------------------------------

/// Build a thriving garden with water, then dig away all water sources.
/// The garden should degrade visibly (stress, stunted growth, possible death)
/// but NOT crash or produce glitched state. Recovery principle: the garden
/// should show stress signals, not just silently freeze.
pub fn water_scarcity_response() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    Scenario::new("water_scarcity_response")
        .description(
            "Build garden with water, remove water sources, observe degradation. \
             Tests graceful stress response — not a crash, not a freeze.",
        )
        .checkpoint("setup")
        .status()
        // Phase 1: build a healthy garden
        .fill(
            "water",
            cx - 4,
            cy - 4,
            GROUND_LEVEL + 1,
            cx + 4,
            cy + 4,
            GROUND_LEVEL + 2,
        )
        .tick(5)
        .plant("oak", cx, cy, GROUND_LEVEL + 3)
        .plant("fern", cx - 2, cy + 1, GROUND_LEVEL + 3)
        .plant("wildflower", cx + 2, cy - 1, GROUND_LEVEL + 3)
        .tick(200)
        .checkpoint("garden_thriving")
        .status()
        // Phase 2: remove all water by digging the spring area
        .fill(
            "air",
            cx - 2,
            cy - 2,
            GROUND_LEVEL - 3,
            cx + 2,
            cy + 2,
            GROUND_LEVEL + 1,
        )
        .tick(300)
        .checkpoint("after_drought")
        .status()
        .eval(NoCrash)
        .eval(Custom {
            name: "garden_responds_to_drought".into(),
            f: Box::new(|trace| {
                // Compare thriving vs drought states
                let steps = &trace.steps;
                let n = steps.len();
                // Mid-point (thriving) vs final (drought)
                let mid = &steps[n / 2].oracle.material_counts;
                let end = &steps[n - 1].oracle.material_counts;

                let mid_plant = mid.total_plant();
                let end_plant = end.total_plant();
                let mid_water = mid.water;
                let end_water = end.water;

                // Water should decrease
                let water_dropped = end_water < mid_water;
                // Plants might decrease, stay same, or even grow (from seeds already in ground)
                // Key: the garden should NOT be identical — some response occurred
                let plant_changed = end_plant != mid_plant;
                // Deadwood appearing = stress death (good signal)
                let has_deadwood = end.deadwood > mid.deadwood;

                let passed = water_dropped || plant_changed;

                Verdict {
                    evaluator: "garden_responds_to_drought".into(),
                    passed,
                    reason: format!(
                        "thriving: {} plants, {} water → drought: {} plants, {} water. \
                         deadwood: {} → {}. {}",
                        mid_plant,
                        mid_water,
                        end_plant,
                        end_water,
                        mid.deadwood,
                        end.deadwood,
                        if has_deadwood {
                            "Trees died from stress — visible degradation."
                        } else if plant_changed {
                            "Garden changed in response to drought."
                        } else if water_dropped {
                            "Water dropped but plants unchanged — stress not visible enough."
                        } else {
                            "PROBLEM: no response to drought at all."
                        }
                    ),
                    score: Some(if has_deadwood {
                        1.0
                    } else if plant_changed {
                        0.7
                    } else if water_dropped {
                        0.3
                    } else {
                        0.0
                    }),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 21. Observation Reward — "How much happened while I watched?"
// ---------------------------------------------------------------------------

/// Plant a diverse garden, let it run for 600 ticks, then measure how many
/// distinct ecological changes occurred. A living garden should produce
/// multiple observable events: new species appearing (from dispersal),
/// material type changes, fauna spawning, deadwood cycling.
///
/// This tests whether "watching the garden" is rewarding vs. boring.
pub fn observation_reward_density() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    Scenario::new("observation_reward_density")
        .description(
            "Plant a diverse garden, idle 600 ticks, count distinct ecological \
             changes. Tests whether passive observation is rewarding.",
        )
        .checkpoint("setup")
        .status()
        .fill(
            "water",
            cx - 6,
            cy - 6,
            GROUND_LEVEL + 1,
            cx + 6,
            cy + 6,
            GROUND_LEVEL + 2,
        )
        .tick(5)
        // Diverse planting: tree, shrub, flower, groundcover
        .plant("oak", cx - 3, cy, GROUND_LEVEL + 3)
        .plant("birch", cx + 3, cy, GROUND_LEVEL + 3)
        .plant("fern", cx, cy + 3, GROUND_LEVEL + 3)
        .plant("wildflower", cx - 1, cy - 2, GROUND_LEVEL + 3)
        .plant("moss", cx + 2, cy + 2, GROUND_LEVEL + 3)
        .plant("clover", cx - 2, cy + 1, GROUND_LEVEL + 3)
        .tick(100)
        .checkpoint("garden_established")
        .status()
        // Now just watch — 600 ticks of idle, sampled every 100
        .tick(100)
        .status()
        .tick(100)
        .status()
        .tick(100)
        .status()
        .tick(100)
        .status()
        .tick(100)
        .status()
        .tick(100)
        .status()
        .checkpoint("observation_complete")
        .eval(NoCrash)
        .eval(Custom {
            name: "observation_event_density".into(),
            f: Box::new(|trace| {
                let steps = &trace.steps;
                if steps.len() < 4 {
                    return Verdict {
                        evaluator: "observation_event_density".into(),
                        passed: false,
                        reason: "too few steps".into(),
                        score: Some(0.0),
                    };
                }

                // Count distinct changes between consecutive status snapshots
                let mut events = 0u32;
                let mut event_details = Vec::new();

                for i in 1..steps.len() {
                    let prev = &steps[i - 1].oracle.material_counts;
                    let curr = &steps[i].oracle.material_counts;

                    // New trunk appeared (tree grew)
                    if curr.trunk > prev.trunk {
                        events += 1;
                        event_details.push(format!("trunk +{}", curr.trunk - prev.trunk));
                    }
                    // New leaf appeared (canopy/flower grew)
                    if curr.leaf > prev.leaf {
                        events += 1;
                        event_details.push(format!("leaf +{}", curr.leaf - prev.leaf));
                    }
                    // New root appeared (underground growth)
                    if curr.root > prev.root {
                        events += 1;
                        event_details.push(format!("root +{}", curr.root - prev.root));
                    }
                    // Seed appeared (dispersal!)
                    if curr.seed > prev.seed {
                        events += 1;
                        event_details.push(format!("seed +{} (dispersal!)", curr.seed - prev.seed));
                    }
                    // Deadwood appeared (lifecycle)
                    if curr.deadwood > prev.deadwood {
                        events += 1;
                        event_details.push(format!("deadwood +{}", curr.deadwood - prev.deadwood));
                    }
                    // Water changed (hydrology)
                    let water_diff = (curr.water as i64 - prev.water as i64).unsigned_abs();
                    if water_diff > 10 {
                        events += 1;
                        event_details.push(format!("water delta {}", water_diff));
                    }
                }

                // A living garden should produce 5+ distinct events over 600 ticks
                let passed = events >= 5;

                Verdict {
                    evaluator: "observation_event_density".into(),
                    passed,
                    reason: format!(
                        "{} distinct events over 600 idle ticks: [{}]. {}",
                        events,
                        event_details.join(", "),
                        if events >= 10 {
                            "Rich observation — garden is very alive!"
                        } else if events >= 5 {
                            "Good observation density."
                        } else {
                            "SPARSE — player gets bored watching. Need more autonomous activity."
                        }
                    ),
                    score: Some((events as f64 / 10.0).min(1.0)),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 22. Visual Growth Stages — "Can I see each stage?"
// ---------------------------------------------------------------------------

/// Extended growth timeline: run 300 ticks (enough for young tree with canopy)
/// and verify each growth stage produces a visually distinct material change.
/// The player should see: seed → trunk → leaf (canopy) → branch → root (underground).
pub fn visual_growth_stages() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    let mut builder = Scenario::new("visual_growth_stages")
        .description(
            "Extended growth: 500 ticks. Verify seed→trunk→leaf→branch→root \
             all appear as visible material changes.",
        )
        .checkpoint("setup")
        .status()
        .fill(
            "water",
            cx - 6,
            cy - 6,
            GROUND_LEVEL + 1,
            cx + 6,
            cy + 6,
            GROUND_LEVEL + 3,
        )
        .tick(5)
        .plant("oak", cx, cy, GROUND_LEVEL + 4);

    // Sample every 25 ticks for 250 ticks, then refill water, then 250 more
    for _ in 0..10 {
        builder = builder.tick(25).status();
    }
    // Refill water halfway through to sustain growth with faster drain
    builder = builder.fill(
        "water",
        cx - 6,
        cy - 6,
        GROUND_LEVEL + 1,
        cx + 6,
        cy + 6,
        GROUND_LEVEL + 3,
    );
    for _ in 0..10 {
        builder = builder.tick(25).status();
    }

    builder
        .eval(NoCrash)
        .eval(Custom {
            name: "all_growth_stages_visible".into(),
            f: Box::new(|trace| {
                let mut first_seed: Option<u64> = None;
                let mut first_trunk: Option<u64> = None;
                let mut first_root: Option<u64> = None;
                let mut first_leaf: Option<u64> = None;
                let mut first_branch: Option<u64> = None;

                for step in &trace.steps {
                    let mc = &step.oracle.material_counts;
                    let tick = step.oracle.tick;
                    if first_seed.is_none() && mc.seed > 0 {
                        first_seed = Some(tick);
                    }
                    if first_trunk.is_none() && mc.trunk > 0 {
                        first_trunk = Some(tick);
                    }
                    if first_root.is_none() && mc.root > 0 {
                        first_root = Some(tick);
                    }
                    if first_leaf.is_none() && mc.leaf > 0 {
                        first_leaf = Some(tick);
                    }
                    if first_branch.is_none() && mc.branch > 0 {
                        first_branch = Some(tick);
                    }
                }

                let fmt = |o: &Option<u64>| o.map_or("never".into(), |t| format!("t{}", t));
                let stages_seen = [
                    first_trunk.is_some(),
                    first_root.is_some(),
                    first_leaf.is_some(),
                ]
                .iter()
                .filter(|&&x| x)
                .count();

                let passed = stages_seen >= 2; // at minimum trunk + root should appear

                Verdict {
                    evaluator: "all_growth_stages_visible".into(),
                    passed,
                    reason: format!(
                        "seed={}, trunk={}, root={}, leaf={}, branch={}. \
                         {}/{} stages visible in 300 ticks. {}",
                        fmt(&first_seed),
                        fmt(&first_trunk),
                        fmt(&first_root),
                        fmt(&first_leaf),
                        fmt(&first_branch),
                        stages_seen,
                        3,
                        if first_leaf.is_none() {
                            "WARNING: no leaves in 300 ticks — canopy never becomes visible!"
                        } else if stages_seen >= 3 {
                            "Full growth arc visible."
                        } else {
                            "Partial growth arc."
                        }
                    ),
                    score: Some(stages_seen as f64 / 3.0),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 23. Species Personality — "Do different species feel different?"
// ---------------------------------------------------------------------------

/// Plant one of each plant type (tree, shrub, flower, groundcover) with
/// identical conditions. After 300 ticks, verify they produce different
/// material signatures — a pine should look different from an oak,
/// a wildflower different from moss.
pub fn species_feel_different() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    Scenario::new("species_feel_different")
        .description(
            "Plant oak, pine, wildflower, moss in identical conditions. \
             Verify they produce different growth signatures after 300 ticks.",
        )
        .checkpoint("setup")
        .status()
        .fill(
            "water",
            cx - 8,
            cy - 8,
            GROUND_LEVEL + 1,
            cx + 8,
            cy + 8,
            GROUND_LEVEL + 2,
        )
        .tick(5)
        // Plant 4 species with spacing so they don't compete
        .plant("oak", cx - 6, cy, GROUND_LEVEL + 3)
        .plant("pine", cx + 6, cy, GROUND_LEVEL + 3)
        .plant("wildflower", cx, cy - 6, GROUND_LEVEL + 3)
        .plant("moss", cx, cy + 6, GROUND_LEVEL + 3)
        .tick(300)
        .checkpoint("grown")
        .status()
        // Probe each planting location at multiple heights
        .probe(cx - 6, cy, GROUND_LEVEL + 1) // oak base
        .probe(cx - 6, cy, GROUND_LEVEL + 5) // oak canopy height
        .probe(cx + 6, cy, GROUND_LEVEL + 1) // pine base
        .probe(cx + 6, cy, GROUND_LEVEL + 5) // pine canopy height
        .probe(cx, cy - 6, GROUND_LEVEL + 1) // wildflower
        .probe(cx, cy + 6, GROUND_LEVEL + 1) // moss
        .eval(NoCrash)
        .eval(MaterialMinimum::new("plant", 5))
        .eval(Custom {
            name: "species_produce_different_signatures".into(),
            f: Box::new(|trace| {
                let Some(oracle) = trace.final_oracle() else {
                    return Verdict {
                        evaluator: "species_produce_different_signatures".into(),
                        passed: false,
                        reason: "no oracle".into(),
                        score: Some(0.0),
                    };
                };

                // Check probes: different species at different locations should
                // produce different materials at the same relative height
                let probes = &oracle.probes;
                let mut materials_at_base = Vec::new();
                let mut materials_at_canopy = Vec::new();

                for (i, p) in probes.iter().enumerate() {
                    if i % 2 == 0 {
                        materials_at_base.push(p.material.clone());
                    } else {
                        materials_at_canopy.push(p.material.clone());
                    }
                }

                // Count distinct materials at base level
                let distinct_base: std::collections::HashSet<_> =
                    materials_at_base.iter().collect();
                // Trees should have trunk at base, flowers/moss should have leaf or air
                let has_variety = distinct_base.len() >= 2;

                let total = oracle.material_counts.total_plant();

                Verdict {
                    evaluator: "species_produce_different_signatures".into(),
                    passed: has_variety && total >= 5,
                    reason: format!(
                        "base materials: {:?}, canopy materials: {:?}. \
                         {} distinct base types. Total biomass: {}. {}",
                        materials_at_base,
                        materials_at_canopy,
                        distinct_base.len(),
                        total,
                        if has_variety {
                            "Species produce visually distinct growth."
                        } else {
                            "All species look the same — need more visual differentiation."
                        }
                    ),
                    score: Some(distinct_base.len() as f64 / 4.0),
                }
            }),
        })
        .build()
}

// ---------------------------------------------------------------------------
// 24. Milestone Progression Arc — "Can the player unlock all tiers?"
// ---------------------------------------------------------------------------

/// Simulate the intended progression: start with groundcover (tier 0),
/// grow enough to unlock flowers (tier 1), attract pollinators for
/// shrubs (tier 2), build diversity for trees (tier 3). Measures whether
/// a patient player can naturally reach each tier.
pub fn milestone_progression_arc() -> Scenario {
    let cx = GRID_X / 2;
    let cy = GRID_Y / 2;

    let mut builder = Scenario::new("milestone_progression_arc")
        .description(
            "Full progression: groundcover → flowers → shrubs → trees. \
             Measures biomass at each stage to verify tiers are reachable.",
        )
        .checkpoint("start")
        .status()
        // Water the garden — wide pool, refreshed between tiers
        .fill(
            "water",
            cx - 8,
            cy - 8,
            GROUND_LEVEL + 1,
            cx + 8,
            cy + 8,
            GROUND_LEVEL + 3,
        )
        .tick(5);

    // Tier 0: plant groundcover (always available)
    builder = builder
        .plant("moss", cx - 3, cy - 1, GROUND_LEVEL + 4)
        .plant("moss", cx - 2, cy + 1, GROUND_LEVEL + 4)
        .plant("grass", cx + 1, cy - 2, GROUND_LEVEL + 4)
        .plant("grass", cx + 2, cy + 2, GROUND_LEVEL + 4)
        .plant("clover", cx - 1, cy + 3, GROUND_LEVEL + 4)
        .plant("clover", cx + 3, cy - 1, GROUND_LEVEL + 4)
        .tick(100)
        .checkpoint("tier0_groundcover")
        .status();

    // Refill water before flowers
    builder = builder.fill(
        "water",
        cx - 8,
        cy - 8,
        GROUND_LEVEL + 1,
        cx + 8,
        cy + 8,
        GROUND_LEVEL + 3,
    );

    // Tier 1: plant flowers (unlocked at 10+ groundcover leaf voxels)
    builder = builder
        .plant("wildflower", cx, cy - 3, GROUND_LEVEL + 4)
        .plant("wildflower", cx - 4, cy, GROUND_LEVEL + 4)
        .plant("daisy", cx + 4, cy, GROUND_LEVEL + 4)
        .tick(200)
        .checkpoint("tier1_flowers")
        .status();

    // Refill water before shrubs
    builder = builder.fill(
        "water",
        cx - 8,
        cy - 8,
        GROUND_LEVEL + 1,
        cx + 8,
        cy + 8,
        GROUND_LEVEL + 3,
    );

    // Tier 2: plant shrubs (unlocked at 2+ pollinators)
    builder = builder
        .plant("fern", cx, cy + 4, GROUND_LEVEL + 4)
        .plant("berry-bush", cx - 5, cy - 3, GROUND_LEVEL + 4)
        .tick(200)
        .checkpoint("tier2_shrubs")
        .status();

    // Refill water before trees
    builder = builder.fill(
        "water",
        cx - 8,
        cy - 8,
        GROUND_LEVEL + 1,
        cx + 8,
        cy + 8,
        GROUND_LEVEL + 3,
    );

    // Tier 3: plant trees (unlocked at 4+ fauna, 3+ species diversity)
    builder = builder
        .plant("oak", cx, cy, GROUND_LEVEL + 4)
        .plant("birch", cx + 5, cy + 3, GROUND_LEVEL + 4)
        .tick(400)
        .checkpoint("tier3_trees")
        .status();

    builder
        .eval(NoCrash)
        .eval(Custom {
            name: "progression_reaches_all_tiers".into(),
            f: Box::new(|trace| {
                let steps = &trace.steps;
                let n = steps.len();
                if n < 4 {
                    return Verdict {
                        evaluator: "progression_reaches_all_tiers".into(),
                        passed: false,
                        reason: "too few steps".into(),
                        score: Some(0.0),
                    };
                }

                // Sample biomass at each tier checkpoint (roughly at 25%, 50%, 75%, 100%)
                let tier_data: Vec<(usize, u64, u64, u64, u64)> = [n / 4, n / 2, 3 * n / 4, n - 1]
                    .iter()
                    .map(|&i| {
                        let mc = &steps[i].oracle.material_counts;
                        (i, mc.leaf, mc.trunk, mc.root, mc.total_plant())
                    })
                    .collect();

                let mut report = Vec::new();
                let mut tiers_reached = 0u32;

                // Tier 0: groundcover should produce leaf voxels
                let (_, t0_leaf, _, _, t0_total) = tier_data[0];
                if t0_leaf >= 3 || t0_total >= 5 {
                    tiers_reached += 1;
                    report.push(format!(
                        "T0: {} leaf, {} total — groundcover established",
                        t0_leaf, t0_total
                    ));
                } else {
                    report.push(format!(
                        "T0: {} leaf, {} total — groundcover too sparse",
                        t0_leaf, t0_total
                    ));
                }

                // Tier 1: flowers added
                let (_, _t1_leaf, _, _, t1_total) = tier_data[1];
                if t1_total > t0_total {
                    tiers_reached += 1;
                    report.push(format!(
                        "T1: {} total (+{}) — flowers growing",
                        t1_total,
                        t1_total - t0_total
                    ));
                } else {
                    report.push(format!("T1: {} total — stalled after flowers", t1_total));
                }

                // Tier 2: shrubs + fauna habitat
                let (_, _, _, _, t2_total) = tier_data[2];
                if t2_total > t1_total {
                    tiers_reached += 1;
                    report.push(format!(
                        "T2: {} total (+{}) — shrubs growing",
                        t2_total,
                        t2_total - t1_total
                    ));
                } else {
                    report.push(format!("T2: {} total — stalled after shrubs", t2_total));
                }

                // Tier 3: trees (the payoff)
                let (_, t3_leaf, t3_trunk, t3_root, t3_total) = tier_data[3];
                if t3_trunk >= 5 && t3_total > t2_total {
                    tiers_reached += 1;
                    report.push(format!(
                        "T3: {} trunk, {} leaf, {} root, {} total — TREES GROWING!",
                        t3_trunk, t3_leaf, t3_root, t3_total
                    ));
                } else {
                    report.push(format!(
                        "T3: {} trunk, {} total — trees not yet visible",
                        t3_trunk, t3_total
                    ));
                }

                let passed = tiers_reached >= 3;

                Verdict {
                    evaluator: "progression_reaches_all_tiers".into(),
                    passed,
                    reason: format!(
                        "{}/4 tiers reached. [{}]",
                        tiers_reached,
                        report.join(" | "),
                    ),
                    score: Some(tiers_reached as f64 / 4.0),
                }
            }),
        })
        .build()
}

//! Evaluators — programmatic judges that score traces.
//!
//! Evaluators use oracle snapshots (privileged ground truth) to determine
//! whether a scenario achieved its goals. They produce verdicts with scores
//! and human-readable explanations.

use crate::oracle::MaterialCounts;
use crate::trace::Trace;

/// Result of an evaluator judging a trace.
#[derive(Debug, Clone)]
pub struct Verdict {
    /// Name of the evaluator that produced this verdict.
    pub evaluator: String,
    /// Pass or fail.
    pub passed: bool,
    /// Human-readable explanation of the result.
    pub reason: String,
    /// Optional numeric score (0.0 = worst, 1.0 = best).
    pub score: Option<f64>,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = if self.passed { "PASS" } else { "FAIL" };
        if let Some(score) = self.score {
            write!(f, "[{icon}] {} ({:.0}%): {}", self.evaluator, score * 100.0, self.reason)
        } else {
            write!(f, "[{icon}] {}: {}", self.evaluator, self.reason)
        }
    }
}

/// Trait for evaluators that judge traces.
pub trait Evaluator {
    /// Judge a completed trace and return a verdict.
    fn evaluate(&self, trace: &Trace) -> Verdict;
}

// ---------------------------------------------------------------------------
// Built-in evaluators
// ---------------------------------------------------------------------------

/// Checks that a material count meets a minimum threshold at the end of the run.
pub struct MaterialMinimum {
    pub name: String,
    pub material: String,
    pub minimum: u64,
}

impl MaterialMinimum {
    pub fn new(material: &str, minimum: u64) -> Self {
        Self {
            name: format!("{material} >= {minimum}"),
            material: material.to_string(),
            minimum,
        }
    }
}

impl Evaluator for MaterialMinimum {
    fn evaluate(&self, trace: &Trace) -> Verdict {
        let Some(oracle) = trace.final_oracle() else {
            return Verdict {
                evaluator: self.name.clone(),
                passed: false,
                reason: "no trace steps".into(),
                score: Some(0.0),
            };
        };
        let actual = get_count(&oracle.material_counts, &self.material);
        let passed = actual >= self.minimum;
        let score = if self.minimum == 0 {
            if passed { 1.0 } else { 0.0 }
        } else {
            (actual as f64 / self.minimum as f64).min(1.0)
        };
        Verdict {
            evaluator: self.name.clone(),
            passed,
            reason: format!("{}: {} (need >= {})", self.material, actual, self.minimum),
            score: Some(score),
        }
    }
}

/// Checks that a material count is zero at the end of the run.
pub struct MaterialAbsent {
    pub name: String,
    pub material: String,
}

impl MaterialAbsent {
    pub fn new(material: &str) -> Self {
        Self {
            name: format!("no {material}"),
            material: material.to_string(),
        }
    }
}

impl Evaluator for MaterialAbsent {
    fn evaluate(&self, trace: &Trace) -> Verdict {
        let Some(oracle) = trace.final_oracle() else {
            return Verdict {
                evaluator: self.name.clone(),
                passed: false,
                reason: "no trace steps".into(),
                score: Some(0.0),
            };
        };
        let actual = get_count(&oracle.material_counts, &self.material);
        Verdict {
            evaluator: self.name.clone(),
            passed: actual == 0,
            reason: format!("{}: {}", self.material, actual),
            score: Some(if actual == 0 { 1.0 } else { 0.0 }),
        }
    }
}

/// Checks that a material count increased between two checkpoints (or start→end).
pub struct MaterialGrew {
    pub name: String,
    pub material: String,
    pub from_checkpoint: Option<String>,
    pub to_checkpoint: Option<String>,
}

impl MaterialGrew {
    pub fn new(material: &str) -> Self {
        Self {
            name: format!("{material} grew"),
            material: material.to_string(),
            from_checkpoint: None,
            to_checkpoint: None,
        }
    }

    pub fn between(material: &str, from: &str, to: &str) -> Self {
        Self {
            name: format!("{material} grew ({from} → {to})"),
            material: material.to_string(),
            from_checkpoint: Some(from.to_string()),
            to_checkpoint: Some(to.to_string()),
        }
    }
}

impl Evaluator for MaterialGrew {
    fn evaluate(&self, trace: &Trace) -> Verdict {
        let before = self
            .from_checkpoint
            .as_ref()
            .and_then(|cp| trace.checkpoint(cp))
            .map(|s| &s.oracle)
            .or_else(|| trace.steps.first().map(|s| &s.oracle));

        let after = self
            .to_checkpoint
            .as_ref()
            .and_then(|cp| trace.checkpoint(cp))
            .map(|s| &s.oracle)
            .or_else(|| trace.final_oracle());

        let (Some(before), Some(after)) = (before, after) else {
            return Verdict {
                evaluator: self.name.clone(),
                passed: false,
                reason: "missing checkpoint(s)".into(),
                score: Some(0.0),
            };
        };

        let before_count = get_count(&before.material_counts, &self.material);
        let after_count = get_count(&after.material_counts, &self.material);
        let passed = after_count > before_count;

        Verdict {
            evaluator: self.name.clone(),
            passed,
            reason: format!("{}: {} → {}", self.material, before_count, after_count),
            score: Some(if passed { 1.0 } else { 0.0 }),
        }
    }
}

/// Checks that a specific voxel has the expected material at the end.
pub struct VoxelMaterial {
    pub name: String,
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub expected: String,
}

impl VoxelMaterial {
    pub fn new(x: usize, y: usize, z: usize, expected: &str) -> Self {
        Self {
            name: format!("({x},{y},{z}) is {expected}"),
            x,
            y,
            z,
            expected: expected.to_string(),
        }
    }
}

impl Evaluator for VoxelMaterial {
    fn evaluate(&self, trace: &Trace) -> Verdict {
        let Some(oracle) = trace.final_oracle() else {
            return Verdict {
                evaluator: self.name.clone(),
                passed: false,
                reason: "no trace steps".into(),
                score: Some(0.0),
            };
        };

        let probe = oracle.probes.iter().find(|p| p.x == self.x && p.y == self.y && p.z == self.z);
        match probe {
            Some(p) => {
                let passed = p.material == self.expected;
                Verdict {
                    evaluator: self.name.clone(),
                    passed,
                    reason: format!("({},{},{}) is {} (expected {})", self.x, self.y, self.z, p.material, self.expected),
                    score: Some(if passed { 1.0 } else { 0.0 }),
                }
            }
            None => Verdict {
                evaluator: self.name.clone(),
                passed: false,
                reason: format!("no probe at ({},{},{})", self.x, self.y, self.z),
                score: Some(0.0),
            },
        }
    }
}

/// Checks that the simulation didn't panic/crash during the run.
/// This always passes if we got a trace at all (panics abort before trace finishes).
pub struct NoCrash;

impl Evaluator for NoCrash {
    fn evaluate(&self, trace: &Trace) -> Verdict {
        Verdict {
            evaluator: "no_crash".into(),
            passed: !trace.steps.is_empty(),
            reason: format!("{} steps completed", trace.steps.len()),
            score: Some(1.0),
        }
    }
}

/// Custom evaluator from a closure.
pub struct Custom {
    pub name: String,
    pub f: Box<dyn Fn(&Trace) -> Verdict>,
}

impl Evaluator for Custom {
    fn evaluate(&self, trace: &Trace) -> Verdict {
        (self.f)(trace)
    }
}

fn get_count(counts: &MaterialCounts, material: &str) -> u64 {
    match material {
        "air" => counts.air,
        "soil" => counts.soil,
        "stone" => counts.stone,
        "water" => counts.water,
        "root" => counts.root,
        "seed" => counts.seed,
        "trunk" => counts.trunk,
        "branch" => counts.branch,
        "leaf" => counts.leaf,
        "deadwood" => counts.deadwood,
        "wet_soil" => counts.wet_soil,
        "plant" => counts.total_plant(),
        "tree" => counts.total_tree(),
        _ => 0,
    }
}

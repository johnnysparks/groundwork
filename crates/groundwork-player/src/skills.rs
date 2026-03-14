//! Skill extraction — learn reusable action patterns from successful traces.
//!
//! Skills are text descriptions of successful strategies that can be injected
//! into the LLM planner's prompt, giving it a library of known-good approaches.

use crate::action::Action;
use crate::analysis::RunSummary;
use crate::trace::Trace;

/// A reusable action pattern extracted from a successful trace.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Short name (e.g. "plant_near_water").
    pub name: String,
    /// Natural language description for LLM consumption.
    pub description: String,
    /// When to use this skill.
    pub preconditions: String,
    /// The action sequence.
    pub actions: Vec<Action>,
    /// What should happen when this skill is applied.
    pub expected_outcome: String,
    /// Which scenario this was learned from.
    pub source_scenario: String,
    /// How often this pattern led to success (0.0–1.0).
    pub success_rate: f64,
}

/// Extract skills from successful run summaries and their traces.
pub fn extract_skills(summaries: &[RunSummary], traces: &[Trace]) -> Vec<Skill> {
    let mut skills = Vec::new();

    for (summary, trace) in summaries.iter().zip(traces.iter()) {
        if !summary.passed {
            continue;
        }

        // Extract water placement patterns
        if let Some(skill) = extract_water_basin_skill(trace, summary) {
            skills.push(skill);
        }

        // Extract planting patterns
        if let Some(skill) = extract_planting_skill(trace, summary) {
            skills.push(skill);
        }

        // Extract camera exploration patterns
        if let Some(skill) = extract_camera_skill(trace, summary) {
            skills.push(skill);
        }
    }

    // Deduplicate by name, keeping highest success rate
    deduplicate_skills(&mut skills);
    skills
}

/// Format skills as text for injection into an LLM prompt.
pub fn skills_to_prompt(skills: &[Skill]) -> String {
    if skills.is_empty() {
        return String::new();
    }

    let mut lines = vec!["## Learned Skills (from successful sessions)".to_string()];

    for skill in skills {
        lines.push(format!("\n### {}", skill.name));
        lines.push(skill.description.clone());
        lines.push(format!("When: {}", skill.preconditions));
        lines.push(format!("Expected: {}", skill.expected_outcome));
        lines.push(format!(
            "Success rate: {:.0}% (from {})",
            skill.success_rate * 100.0,
            skill.source_scenario
        ));
    }

    lines.join("\n")
}

fn extract_water_basin_skill(trace: &Trace, summary: &RunSummary) -> Option<Skill> {
    // Look for Fill actions with water tool
    let water_fills: Vec<&Action> = trace
        .steps
        .iter()
        .filter(|s| matches!(&s.action, Action::Fill { tool, .. } if tool == "water"))
        .map(|s| &s.action)
        .collect();

    if water_fills.is_empty() {
        return None;
    }

    let final_water = summary.final_material_counts.water;
    if final_water == 0 {
        return None;
    }

    Some(Skill {
        name: "water_basin".into(),
        description: format!(
            "Fill a region with water to create a growing zone. Used {} fill action(s), resulting in {} water voxels.",
            water_fills.len(),
            final_water
        ),
        preconditions: "Start of session, before planting seeds.".into(),
        actions: water_fills.into_iter().cloned().collect(),
        expected_outcome: "Water persists at or near surface level, providing moisture for nearby seeds.".into(),
        source_scenario: trace.scenario_name.clone(),
        success_rate: 1.0,
    })
}

fn extract_planting_skill(trace: &Trace, summary: &RunSummary) -> Option<Skill> {
    let plant_actions: Vec<&Action> = trace
        .steps
        .iter()
        .filter(|s| {
            matches!(&s.action, Action::Place { tool, .. } if tool == "seed")
        })
        .map(|s| &s.action)
        .collect();

    if plant_actions.is_empty() {
        return None;
    }

    let final_plants = summary.final_material_counts.total_plant();
    if final_plants == 0 {
        return None;
    }

    // Count unique species
    let species: std::collections::HashSet<String> = plant_actions
        .iter()
        .filter_map(|a| match a {
            Action::Place { species, .. } => species.clone(),
            _ => None,
        })
        .collect();

    let diversity = if species.len() >= 4 {
        "diverse planting"
    } else if species.len() >= 2 {
        "mixed planting"
    } else {
        "single-species planting"
    };

    Some(Skill {
        name: format!("{}_near_water", diversity.replace(' ', "_")),
        description: format!(
            "Plant {} seed(s) of {} species near water. Resulted in {} plant voxels.",
            plant_actions.len(),
            species.len(),
            final_plants
        ),
        preconditions: "Water already placed nearby. Soil present at or below planting z-level.".into(),
        actions: plant_actions.into_iter().cloned().collect(),
        expected_outcome: format!(
            "Seeds germinate and grow into plants. Species: {}.",
            species.into_iter().collect::<Vec<_>>().join(", ")
        ),
        source_scenario: trace.scenario_name.clone(),
        success_rate: 1.0,
    })
}

fn extract_camera_skill(trace: &Trace, summary: &RunSummary) -> Option<Skill> {
    if summary.camera_coverage.total_camera_actions < 3 {
        return None;
    }

    let camera_actions: Vec<&Action> = trace
        .steps
        .iter()
        .filter(|s| {
            matches!(
                &s.action,
                Action::CameraOrbit { .. }
                    | Action::CameraPan { .. }
                    | Action::CameraZoom { .. }
                    | Action::CameraCutaway { .. }
                    | Action::CameraReset
            )
        })
        .map(|s| &s.action)
        .collect();

    let went_underground = summary.camera_coverage.went_underground;
    let name = if went_underground {
        "full_exploration"
    } else {
        "surface_exploration"
    };

    Some(Skill {
        name: name.into(),
        description: format!(
            "Explore the garden with {} camera actions. {}.",
            camera_actions.len(),
            if went_underground {
                "Includes underground cutaway to see root systems"
            } else {
                "Surface-level exploration only"
            }
        ),
        preconditions: "After planting and ticking, to observe the results.".into(),
        actions: camera_actions.into_iter().cloned().collect(),
        expected_outcome: "Comprehensive visual survey of the garden from multiple angles.".into(),
        source_scenario: trace.scenario_name.clone(),
        success_rate: 1.0,
    })
}

fn deduplicate_skills(skills: &mut Vec<Skill>) {
    // Keep highest success_rate for duplicate names
    let mut best: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut to_remove = Vec::new();

    for (i, skill) in skills.iter().enumerate() {
        if let Some(&existing_idx) = best.get(&skill.name) {
            if skill.success_rate > skills[existing_idx].success_rate {
                to_remove.push(existing_idx);
                best.insert(skill.name.clone(), i);
            } else {
                to_remove.push(i);
            }
        } else {
            best.insert(skill.name.clone(), i);
        }
    }

    to_remove.sort_unstable();
    to_remove.dedup();
    for idx in to_remove.into_iter().rev() {
        skills.remove(idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::summarize;
    use crate::runner;
    use crate::scenarios::ecosystem;

    #[test]
    fn extract_skills_from_passing_scenario() {
        let scenario = ecosystem::diverse_garden();
        let result = runner::run(&scenario);
        let summary = summarize(&result);
        let skills = extract_skills(&[summary], &[result.trace]);

        // Should extract at least water_basin and planting skills
        assert!(!skills.is_empty());
        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"water_basin"), "Expected water_basin skill, got: {names:?}");
    }

    #[test]
    fn skills_to_prompt_formats_correctly() {
        let skill = Skill {
            name: "test_skill".into(),
            description: "A test skill.".into(),
            preconditions: "Always.".into(),
            actions: vec![],
            expected_outcome: "Good things.".into(),
            source_scenario: "test".into(),
            success_rate: 0.85,
        };
        let prompt = skills_to_prompt(&[skill]);
        assert!(prompt.contains("test_skill"));
        assert!(prompt.contains("85%"));
    }
}

//! Agent skills: built-in skill definitions and tools_description builder.
//! See docs/AGENT_ASSISTANT_PLAN.md §3.
//!
//! Skills are named bundles (description, tools) that define what an agent can do.
//! Default learning-path skills: development, creative, office, home.
//! Hard skill: project_init (every agent can init a DAL project).

/// Default skill set for the DAL assistant (smart agent assist).
/// When an agent has no explicit skills (e.g. no mold or mold has no skills), use these.
pub const DEFAULT_LEARNING_PATH_SKILLS: &[&str] = &["development", "creative", "office", "home"];

/// Hard skill: always available to every agent. Used when building tools description.
pub const PROJECT_INIT_SKILL: &str = "project_init";

/// Short description for each built-in skill (for the prompt).
fn skill_description(name: &str) -> &'static str {
    match name {
        "project_init" => "Initialize and set up a DAL project (dal.toml + entry file); run dal init or create layout.",
        "development" => "Coding, scripts, DAL projects: read/edit files, dal check/run, debug, explain code.",
        "creative" => "Writing, design, ideation: drafts, outlines, copy, storytelling, tone, brainstorm.",
        "office" => "Tasks, scheduling, docs, meetings: summarize, action lists, email/note drafts, coordination.",
        "home" => "Personal tasks, routines, health, family, hobbies: reminders, recipes, how-tos, lists.",
        _ => "",
    }
}

/// Base tools (all agents): reply, run, search, ask_user, dal_init.
const BASE_TOOLS: &str = "reply, run (shell), search, ask_user, dal_init";
/// Development skill tools: read_file, write_file, list_dir, dal_check, dal_run.
const DEVELOPMENT_TOOLS: &str = "read_file, write_file, list_dir, dal_check, dal_run";

/// Build the tools_description string for the agent prompt from the given skill names.
/// Always includes project_init (hard skill). If skills is empty, uses DEFAULT_LEARNING_PATH_SKILLS.
/// Development skill adds read_file, write_file, list_dir, dal_check, dal_run.
pub fn tools_description_for_skills(skills: &[String]) -> String {
    let mut effective: Vec<String> = skills.iter().map(String::clone).collect();
    if effective.is_empty() {
        effective = DEFAULT_LEARNING_PATH_SKILLS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
    }
    // Hard skill: project_init always in effect for tools description
    if !effective.iter().any(|s| s == PROJECT_INIT_SKILL) {
        effective.insert(0, PROJECT_INIT_SKILL.to_string());
    }

    let skill_descriptions: Vec<String> = effective
        .iter()
        .filter_map(|name| {
            let d = skill_description(name);
            if d.is_empty() {
                None
            } else {
                Some(format!("- {}: {}", name, d))
            }
        })
        .collect();

    let skills_block = if skill_descriptions.is_empty() {
        "You are a helpful assistant.".to_string()
    } else {
        format!(
            "You are a DAL assistant serving your principal. Skills:\n{}",
            skill_descriptions.join("\n")
        )
    };

    let has_development = effective.iter().any(|s| s == "development");
    let tools_list = if has_development {
        format!("{}; {}", BASE_TOOLS, DEVELOPMENT_TOOLS)
    } else {
        BASE_TOOLS.to_string()
    };

    let json_tools = if has_development {
        r#"Reply with JSON: {"action":"reply","text":"..."} or {"action":"run","cmd":"..."} or {"action":"search","query":"..."} or {"action":"ask_user","message":"..."} or {"action":"dal_init","template":"general"|"chain"|"iot"|"agent"} or {"action":"read_file","path":"..."} or {"action":"write_file","path":"...","contents":"..."} or {"action":"list_dir","path":"."} or {"action":"dal_check","path":"file.dal"} or {"action":"dal_run","path":"file.dal"}. Paths are relative to working directory."#
    } else {
        r#"Reply with JSON: {"action":"reply","text":"..."} or {"action":"run","cmd":"..."} or {"action":"search","query":"..."} or {"action":"ask_user","message":"..."} or {"action":"dal_init","template":"general"|"chain"|"iot"|"agent"} to initialize a DAL project (omit template for general)."#
    };

    format!(
        "{} Available tools: {}. {} When the objective is satisfied, use reply with your final answer. Respect the constraints below.",
        skills_block, tools_list, json_tools
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_skills_produce_non_empty_description() {
        let desc = tools_description_for_skills(&[]);
        assert!(desc.contains("assistant"));
        assert!(desc.contains("development"));
        assert!(desc.contains("creative"));
        assert!(desc.contains("office"));
        assert!(desc.contains("home"));
        assert!(desc.contains("project_init"));
        assert!(desc.contains("reply"));
        assert!(desc.contains("ask_user"));
    }

    #[test]
    fn explicit_skills_used() {
        let desc = tools_description_for_skills(&["development".to_string()]);
        assert!(desc.contains("development"));
        assert!(desc.contains("project_init"));
    }
}

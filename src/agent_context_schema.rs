//! Canonical agent context schema for P0 (Structured Prompt Context).
//!
//! All entry points (HTTP, CLI, DAL) that send prompts to the LLM should build
//! this same shape and use `build_prompt_for_llm` so the model receives
//! consistent context. See docs/AGENT_CONTEXT_SCHEMA.md and
//! docs/COMPREHENSIVE_AGENT_AND_MOLD_PLANS.md §2.3.

/// One turn in the conversation (user or assistant).
#[derive(Debug, Clone, Default)]
pub struct ConversationTurn {
    pub role: String,
    pub content: String,
}

/// Optional agent state included in context (capabilities, trust, working context).
#[derive(Debug, Clone, Default)]
pub struct AgentStateBlock {
    pub capabilities: Vec<String>,
    pub trust_level: String,
    pub working_context: Option<String>,
}

/// A block of injected context (e.g. evolve summary, DAL summary, retrieved doc).
#[derive(Debug, Clone, Default)]
pub struct ContextBlock {
    /// Source label for A/B or debugging (e.g. "evolve_summary", "dal_summary", "doc").
    pub source: String,
    /// Content to include in the prompt.
    pub content: String,
}

/// Canonical shape of what the agent "sees" each turn.
/// All entry points build this and pass it to `build_prompt_for_llm`.
#[derive(Debug, Clone, Default)]
pub struct AgentContextSchema {
    /// Current user goal or objective (e.g. latest message or explicit task).
    pub objective: String,
    /// Recent conversation turns (user/assistant). P1 will fill from evolve.
    pub conversation: Vec<ConversationTurn>,
    /// Description of available tools (e.g. reply, run, search) and how to use them.
    pub tools_description: String,
    /// Optional: agent capabilities, trust level, working context.
    pub agent_state: Option<AgentStateBlock>,
    /// Optional: safety or policy (e.g. shell trust, forbidden patterns).
    pub constraints: Option<String>,
    /// Optional: injected context blocks (evolve summary, DAL summary, docs). Rendered as "## Context".
    pub context_blocks: Vec<ContextBlock>,
    /// If true, render Objective first (for A/B testing). Default false = tools/state/constraints then conversation then objective.
    pub objective_first: bool,
    /// Optional sub-tasks for the objective (P2). Rendered under Objective so the model can reason about progress.
    pub sub_tasks: Option<Vec<String>>,
    /// P5: Completion criteria and when to involve the user. Rendered as "## Completion and when to ask human".
    pub completion_and_ask_guidance: Option<String>,
}

impl AgentContextSchema {
    /// Build a minimal schema with only objective and tools (e.g. for DAL `ai::respond_with_tools`).
    pub fn minimal(objective: impl Into<String>, tools_description: impl Into<String>) -> Self {
        Self {
            objective: objective.into(),
            conversation: Vec::new(),
            tools_description: tools_description.into(),
            agent_state: None,
            constraints: None,
            context_blocks: Vec::new(),
            objective_first: false,
            sub_tasks: None,
            completion_and_ask_guidance: None,
        }
    }
}

/// Renders the schema into the single prompt string sent to the LLM.
/// All entry points use this so the model always sees the same structure.
/// When `objective_first` is true, Objective is rendered first (for A/B testing).
pub fn build_prompt_for_llm(schema: &AgentContextSchema) -> String {
    let mut out = String::new();

    fn append_objective(out: &mut String, objective: &str, sub_tasks: Option<&[String]>) {
        out.push_str("## Objective\n\n");
        out.push_str(objective.trim());
        out.push_str("\n\n");
        if let Some(tasks) = sub_tasks {
            if !tasks.is_empty() {
                out.push_str("Sub-tasks:\n");
                for (i, t) in tasks.iter().enumerate() {
                    out.push_str(&format!("{}. {}\n", i + 1, t.trim()));
                }
                out.push('\n');
            }
        }
    }

    fn append_tools(out: &mut String, tools_description: &str) {
        if !tools_description.is_empty() {
            out.push_str("## Tools\n\n");
            out.push_str(tools_description.trim());
            out.push_str("\n\n");
        }
    }

    fn append_agent_state(out: &mut String, state: &AgentStateBlock) {
        out.push_str("## Agent state\n\n");
        if !state.capabilities.is_empty() {
            out.push_str("- Capabilities: ");
            out.push_str(&state.capabilities.join(", "));
            out.push_str("\n");
        }
        if !state.trust_level.is_empty() {
            out.push_str("- Trust level: ");
            out.push_str(&state.trust_level);
            out.push_str("\n");
        }
        if let Some(ref ctx) = state.working_context {
            if !ctx.is_empty() {
                out.push_str("- Working context: ");
                out.push_str(ctx);
                out.push_str("\n");
            }
        }
        out.push('\n');
    }

    fn append_constraints(out: &mut String, constraints: &str) {
        if !constraints.is_empty() {
            out.push_str("## Constraints\n\n");
            out.push_str(constraints.trim());
            out.push_str("\n\n");
        }
    }

    fn append_completion_guidance(out: &mut String, guidance: &str) {
        if !guidance.trim().is_empty() {
            out.push_str("## Completion and when to ask human\n\n");
            out.push_str(guidance.trim());
            out.push_str("\n\n");
        }
    }

    fn append_context_blocks(out: &mut String, blocks: &[ContextBlock]) {
        if blocks.is_empty() {
            return;
        }
        out.push_str("## Context\n\n");
        for block in blocks {
            if !block.content.trim().is_empty() {
                if !block.source.is_empty() {
                    out.push_str(&format!("[{}]\n", block.source));
                }
                out.push_str(block.content.trim());
                out.push_str("\n\n");
            }
        }
    }

    fn append_conversation(out: &mut String, conversation: &[ConversationTurn]) {
        if !conversation.is_empty() {
            out.push_str("## Conversation\n\n");
            for turn in conversation {
                out.push_str(&format!("**{}**:\n{}\n\n", turn.role, turn.content.trim()));
            }
        }
    }

    if schema.objective_first {
        append_objective(&mut out, &schema.objective, schema.sub_tasks.as_deref());
    }

    append_tools(&mut out, &schema.tools_description);

    if let Some(ref state) = schema.agent_state {
        append_agent_state(&mut out, state);
    }

    if let Some(ref c) = schema.constraints {
        append_constraints(&mut out, c);
    }

    if let Some(ref g) = schema.completion_and_ask_guidance {
        append_completion_guidance(&mut out, g);
    }

    append_context_blocks(&mut out, &schema.context_blocks);

    append_conversation(&mut out, &schema.conversation);

    if !schema.objective_first {
        append_objective(&mut out, &schema.objective, schema.sub_tasks.as_deref());
    }

    out.push_str("Respond according to the tools and constraints above.");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_schema_builds_prompt() {
        let schema = AgentContextSchema::minimal("What is 2+2?", "Reply or run a command.");
        let prompt = build_prompt_for_llm(&schema);
        assert!(prompt.contains("## Tools"));
        assert!(prompt.contains("Reply or run a command"));
        assert!(prompt.contains("## Objective"));
        assert!(prompt.contains("What is 2+2?"));
    }

    #[test]
    fn schema_with_state_and_constraints() {
        let schema = AgentContextSchema {
            objective: "Run ls".to_string(),
            conversation: vec![ConversationTurn {
                role: "user".to_string(),
                content: "List files".to_string(),
            }],
            tools_description: "reply, run, search".to_string(),
            agent_state: Some(AgentStateBlock {
                capabilities: vec!["task_execution".to_string()],
                trust_level: "sandboxed".to_string(),
                working_context: None,
            }),
            constraints: Some("Shell: sandboxed only.".to_string()),
            context_blocks: Vec::new(),
            objective_first: false,
            sub_tasks: None,
            completion_and_ask_guidance: None,
        };
        let prompt = build_prompt_for_llm(&schema);
        assert!(prompt.contains("Capabilities: task_execution"));
        assert!(prompt.contains("Trust level: sandboxed"));
        assert!(prompt.contains("Shell: sandboxed only"));
        assert!(prompt.contains("**user**:"));
        assert!(prompt.contains("List files"));
        assert!(prompt.contains("Run ls"));
    }

    #[test]
    fn objective_first_renders_objective_at_start() {
        let schema = AgentContextSchema {
            objective: "What is 2+2?".to_string(),
            conversation: Vec::new(),
            tools_description: "Reply only.".to_string(),
            agent_state: None,
            constraints: None,
            context_blocks: Vec::new(),
            objective_first: true,
            sub_tasks: None,
            completion_and_ask_guidance: None,
        };
        let prompt = build_prompt_for_llm(&schema);
        let obj_pos = prompt.find("## Objective").unwrap_or(0);
        let tools_pos = prompt.find("## Tools").unwrap_or(0);
        assert!(
            obj_pos < tools_pos,
            "objective should appear before tools when objective_first"
        );
        assert!(prompt.contains("What is 2+2?"));
    }

    #[test]
    fn context_blocks_rendered_in_context_section() {
        let schema = AgentContextSchema {
            objective: "Summarize.".to_string(),
            conversation: Vec::new(),
            tools_description: "Reply.".to_string(),
            agent_state: None,
            constraints: None,
            context_blocks: vec![
                ContextBlock {
                    source: "evolve_summary".to_string(),
                    content: "User asked about DAL. Agent explained syntax.".to_string(),
                },
                ContextBlock {
                    source: "dal_summary".to_string(),
                    content: "Services: Foo, Bar. Functions: main.".to_string(),
                },
            ],
            objective_first: false,
            sub_tasks: None,
            completion_and_ask_guidance: None,
        };
        let prompt = build_prompt_for_llm(&schema);
        assert!(prompt.contains("## Context"));
        assert!(prompt.contains("[evolve_summary]"));
        assert!(prompt.contains("User asked about DAL"));
        assert!(prompt.contains("[dal_summary]"));
        assert!(prompt.contains("Services: Foo, Bar"));
    }

    #[test]
    fn sub_tasks_rendered_under_objective() {
        let schema = AgentContextSchema {
            objective: "Build a DAL script.".to_string(),
            conversation: Vec::new(),
            tools_description: "Reply.".to_string(),
            agent_state: None,
            constraints: None,
            context_blocks: Vec::new(),
            objective_first: false,
            sub_tasks: Some(vec![
                "Parse the file.".to_string(),
                "Add chain::balance call.".to_string(),
            ]),
            completion_and_ask_guidance: None,
        };
        let prompt = build_prompt_for_llm(&schema);
        assert!(prompt.contains("## Objective"));
        assert!(prompt.contains("Build a DAL script."));
        assert!(prompt.contains("Sub-tasks:"));
        assert!(prompt.contains("1. Parse the file."));
        assert!(prompt.contains("2. Add chain::balance call."));
    }

    #[test]
    fn completion_and_ask_guidance_rendered() {
        let schema = AgentContextSchema {
            objective: "Run ls.".to_string(),
            conversation: Vec::new(),
            tools_description: "Reply.".to_string(),
            agent_state: None,
            constraints: None,
            context_blocks: Vec::new(),
            objective_first: false,
            sub_tasks: None,
            completion_and_ask_guidance: Some(
                "When done, reply with the outcome. Ask user when stuck.".to_string(),
            ),
        };
        let prompt = build_prompt_for_llm(&schema);
        assert!(prompt.contains("## Completion and when to ask human"));
        assert!(prompt.contains("When done, reply with the outcome"));
        assert!(prompt.contains("Ask user when stuck"));
    }
}

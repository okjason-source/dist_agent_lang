//! IDE agent runner: a first-class IDE agent that runs the DAL model-turn loop
//! with the workspace as its surface.  Spawned as `AgentType::IDE` so it
//! participates in the agent registry, uses `ResourceBudget` for guard-rails,
//! and is terminated on completion.
//!
//! Tool execution goes through the same logic as the IDE API (run_command,
//! read_file, write_file) so activity events (file_written, run_started,
//! run_stopped, agent_step) are emitted.

use crate::agent_context_schema::{AgentContextSchema, ContextBlock, ConversationTurn};
use crate::ide::run_backend;
use crate::stdlib::agent::{self, AgentConfig, AgentType, ResourceBudget};
use crate::stdlib::ai::{
    completion_and_ask_guidance_for_tool_loop, execute_fetch_url_result, generate_agent_model_turn,
    max_tool_result_chars, max_tool_steps_from_env, model_turn_to_outcome, run_web_search,
    MultiStepResult, ToolOutcome, TurnUsage, TOOLS_SYSTEM_WITH_SCRIPTING,
};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::broadcast;

// ── Shared helpers ──────────────────────────────────────────────────

pub(crate) fn emit_activity(
    tx: &broadcast::Sender<String>,
    event_type: &str,
    payload: serde_json::Value,
) {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let event = serde_json::json!({
        "type": event_type,
        "timestamp": timestamp,
        "payload": payload
    });
    let _ = serde_json::to_string(&event).map(|s| tx.send(s));
}

pub(crate) use crate::stdlib::fs::resolve_path_under_root;

fn truncate_result(s: &str) -> String {
    let cap = max_tool_result_chars();
    if s.len() <= cap {
        s.to_string()
    } else {
        format!("{}\n... (truncated)", &s[..cap])
    }
}

// ── Loop-local guard state ──────────────────────────────────────────
// The *limits* live on AgentConfig.resource_budget; this struct tracks
// per-invocation runtime counters.

#[derive(Debug, Clone, Default)]
struct ToolLoopState {
    tool_type_counts: std::collections::HashMap<String, u32>,
    last_tool_signature: Option<String>,
    repeated_identical_invocations: u32,
    last_result_fingerprint: Option<u64>,
    consecutive_no_progress: u32,
    total_tokens: u64,
    total_cost_microusd: u64,
}

fn env_u64(name: &str) -> Option<u64> {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
}

fn env_u32(name: &str) -> Option<u32> {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
}

fn env_bool(name: &str) -> Option<bool> {
    std::env::var(name).ok().map(|v| {
        let n = v.trim().to_ascii_lowercase();
        matches!(n.as_str(), "1" | "true" | "yes" | "on")
    })
}

/// Build a `ResourceBudget` from environment variables (same env vars as
/// the old `IdeGuardConfig`, now unified with the agent framework).
fn resource_budget_from_env() -> ResourceBudget {
    let strict_mode = env_bool("DAL_AGENT_GUARDS_STRICT_MODE").unwrap_or(false);
    ResourceBudget {
        max_wall_clock_ms: env_u64("DAL_AGENT_MAX_WALL_CLOCK_MS"),
        max_tool_calls_per_type: env_u32("DAL_AGENT_MAX_TOOL_CALLS_PER_TYPE")
            .or_else(|| strict_mode.then_some(8)),
        max_repeated_identical_invocations: env_u32("DAL_AGENT_MAX_REPEATED_IDENTICAL_INVOCATIONS")
            .or_else(|| strict_mode.then_some(2)),
        max_consecutive_no_progress: env_u32("DAL_AGENT_MAX_CONSECUTIVE_NO_PROGRESS")
            .or_else(|| strict_mode.then_some(1)),
        max_total_tokens: env_u64("DAL_AGENT_MAX_TOTAL_TOKENS"),
        max_cost_microusd: env_u64("DAL_AGENT_MAX_COST_MICROUSD"),
    }
}

fn fingerprint_str(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

fn apply_usage_budget(
    usage: &TurnUsage,
    budget: &ResourceBudget,
    state: &mut ToolLoopState,
) -> Option<String> {
    if let Some(tokens) = usage.total_tokens {
        state.total_tokens = state.total_tokens.saturating_add(tokens);
        if let Some(limit) = budget.max_total_tokens {
            if state.total_tokens > limit {
                return Some("max_total_tokens".to_string());
            }
        }
    }
    if let Some(cost) = usage.estimated_cost_microusd {
        state.total_cost_microusd = state.total_cost_microusd.saturating_add(cost);
        if let Some(limit) = budget.max_cost_microusd {
            if state.total_cost_microusd > limit {
                return Some("max_cost_microusd".to_string());
            }
        }
    }
    None
}

fn tool_signature(outcome: &ToolOutcome) -> Option<(String, String)> {
    match outcome {
        ToolOutcome::Run(cmd) => Some(("run".to_string(), format!("run:{}", cmd.trim()))),
        ToolOutcome::Search(query) => {
            Some(("search".to_string(), format!("search:{}", query.trim())))
        }
        ToolOutcome::FetchUrl(url) => {
            Some(("fetch_url".to_string(), format!("fetch_url:{}", url.trim())))
        }
        ToolOutcome::DalInit(template) => Some((
            "dal_init".to_string(),
            format!(
                "dal_init:{}",
                template.clone().unwrap_or_else(|| "general".to_string())
            ),
        )),
        ToolOutcome::ReadFile(path) => Some((
            "read_file".to_string(),
            format!("read_file:{}", path.trim()),
        )),
        ToolOutcome::WriteFile(path, contents) => Some((
            "write_file".to_string(),
            format!("write_file:{}:{}", path.trim(), fingerprint_str(contents)),
        )),
        ToolOutcome::ListDir(path) => {
            Some(("list_dir".to_string(), format!("list_dir:{}", path.trim())))
        }
        ToolOutcome::DalCheck(path) => Some((
            "dal_check".to_string(),
            format!("dal_check:{}", path.trim()),
        )),
        ToolOutcome::DalRun(path) => {
            Some(("dal_run".to_string(), format!("dal_run:{}", path.trim())))
        }
        ToolOutcome::ShowUrl(url) => {
            Some(("show_url".to_string(), format!("show_url:{}", url.trim())))
        }
        ToolOutcome::ShowContent(content, title) => Some((
            "show_content".to_string(),
            format!(
                "show_content:{}:{}",
                title.clone().unwrap_or_default(),
                fingerprint_str(content)
            ),
        )),
        _ => None,
    }
}

fn check_invocation_guard(
    state: &mut ToolLoopState,
    budget: &ResourceBudget,
    tool_name: &str,
    signature: &str,
) -> Option<String> {
    let count = state
        .tool_type_counts
        .entry(tool_name.to_string())
        .or_insert(0);
    *count = count.saturating_add(1);

    if let Some(limit) = budget.max_tool_calls_per_type {
        if *count > limit {
            return Some("max_tool_calls_per_type".to_string());
        }
    }

    if state.last_tool_signature.as_deref() == Some(signature) {
        state.repeated_identical_invocations =
            state.repeated_identical_invocations.saturating_add(1);
    } else {
        state.repeated_identical_invocations = 1;
    }

    if let Some(limit) = budget.max_repeated_identical_invocations {
        if state.repeated_identical_invocations > limit {
            return Some("max_repeated_identical_invocations".to_string());
        }
    }
    None
}

fn check_result_guard(
    state: &mut ToolLoopState,
    budget: &ResourceBudget,
    signature: &str,
    result: &str,
) -> Option<String> {
    let current_fingerprint = fingerprint_str(result);
    if state.last_tool_signature.as_deref() == Some(signature)
        && state.last_result_fingerprint == Some(current_fingerprint)
    {
        state.consecutive_no_progress = state.consecutive_no_progress.saturating_add(1);
    } else {
        state.consecutive_no_progress = 0;
    }
    state.last_tool_signature = Some(signature.to_string());
    state.last_result_fingerprint = Some(current_fingerprint);
    if let Some(limit) = budget.max_consecutive_no_progress {
        if state.consecutive_no_progress >= limit {
            return Some("max_consecutive_no_progress".to_string());
        }
    }
    None
}

// ── Tool execution ──────────────────────────────────────────────────
// Each function returns the tool result string (or error message).

/// True if `path` stays under `workspace_root` (IDE jail).
fn path_is_within_workspace(workspace_root: &Path, path: &Path) -> bool {
    if let (Ok(root_c), Ok(path_c)) = (workspace_root.canonicalize(), path.canonicalize()) {
        return path_c.starts_with(&root_c);
    }
    path.starts_with(workspace_root)
}

/// Resolve `cd` target: empty or `~` → workspace root; otherwise relative to `job_cwd`
/// with `/` segments and `..` (single-token components only; no `..` inside a name).
fn apply_cd(workspace_root: &Path, job_cwd: &Path, arg: &str) -> Result<PathBuf, String> {
    let arg = arg.trim();
    if arg.is_empty() || arg == "~" {
        return workspace_root
            .canonicalize()
            .or_else(|_| Ok(workspace_root.to_path_buf()));
    }
    let mut cur = job_cwd.to_path_buf();
    for part in arg.split('/').filter(|p| !p.is_empty() && *p != ".") {
        if part == ".." {
            cur.pop();
            if !path_is_within_workspace(workspace_root, &cur) {
                return Err("cd: would leave workspace".to_string());
            }
        } else if part.contains("..") {
            return Err("cd: invalid path component".to_string());
        } else {
            cur.push(part);
        }
    }
    if !path_is_within_workspace(workspace_root, &cur) {
        return Err("cd: path escapes workspace".to_string());
    }
    if !cur.exists() {
        return Err(format!("cd: no such file or directory: {}", arg));
    }
    if !cur.is_dir() {
        return Err(format!("cd: not a directory: {}", arg));
    }
    cur.canonicalize().map_err(|e| format!("cd: {}", e))
}

/// Recognize a lone `cd` or `cd <single relative path>` (no spaces in target). Other forms
/// fall through to normal execution (e.g. `sh -c 'cd a && …'`).
fn try_parse_cd_command(cmd: &str) -> Option<&str> {
    let cmd = cmd.trim();
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.first().copied() != Some("cd") {
        return None;
    }
    if parts.len() > 2 {
        return None;
    }
    if parts.len() == 1 {
        return Some("");
    }
    Some(parts[1])
}

fn exec_run(
    workspace_root: &Path,
    job_cwd: &mut PathBuf,
    cmd: &str,
    events_tx: &broadcast::Sender<String>,
    job_id: &str,
) -> String {
    let cmd = cmd.trim();
    if let Some(cd_arg) = try_parse_cd_command(cmd) {
        let old = job_cwd.clone();
        match apply_cd(workspace_root, job_cwd.as_path(), cd_arg) {
            Ok(new_cwd) => {
                let old_canon = old.canonicalize().ok().unwrap_or_else(|| old.clone());
                let new_canon = new_cwd
                    .canonicalize()
                    .ok()
                    .unwrap_or_else(|| new_cwd.clone());
                *job_cwd = new_cwd;
                if old_canon != new_canon {
                    emit_activity(
                        events_tx,
                        "agent_scope",
                        serde_json::json!({
                            "job_id": job_id,
                            "effective_root": workspace_root.to_string_lossy(),
                            "effective_cwd": job_cwd.to_string_lossy(),
                            "reason": "cd",
                        }),
                    );
                }
                format!("Changed directory to {}", job_cwd.display())
            }
            Err(e) => e,
        }
    } else {
        let (cmd_str, args) = if cmd.is_empty() {
            ("true".to_string(), vec![])
        } else {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            (
                parts
                    .first()
                    .map(|s| (*s).to_string())
                    .unwrap_or_else(|| "true".to_string()),
                parts
                    .get(1..)
                    .map(|s| s.iter().map(|x| (*x).to_string()).collect())
                    .unwrap_or_default(),
            )
        };
        match run_backend::run_command_blocking(&cmd_str, &args, Some(job_cwd.as_path())) {
            Ok((stdout, stderr, code)) => {
                let mut out = format!("Exit code: {}\n", code);
                if !stdout.is_empty() {
                    out.push_str("stdout:\n");
                    out.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    out.push_str("\nstderr:\n");
                    out.push_str(&stderr);
                }
                truncate_result(&out)
            }
            Err(e) => format!("Command failed: {}", e),
        }
    }
}

fn exec_search(query: &str) -> String {
    let result = run_web_search(query.trim()).unwrap_or_else(|e| format!("Search failed: {}", e));
    truncate_result(&result)
}

fn exec_fetch_url(url: &str) -> String {
    truncate_result(&execute_fetch_url_result(url.trim()))
}

fn exec_read_file(root: &Path, path: &str) -> String {
    match resolve_path_under_root(root, path) {
        Err(e) => format!("read_file failed: {}", e),
        Ok(p) => {
            if !p.is_file() {
                "read_file failed: not a file".to_string()
            } else {
                let r = std::fs::read_to_string(&p)
                    .unwrap_or_else(|e| format!("read_file failed: {}", e));
                truncate_result(&r)
            }
        }
    }
}

fn exec_write_file(
    root: &Path,
    path: &str,
    contents: &str,
    files_changed: &mut Vec<String>,
    events_tx: &broadcast::Sender<String>,
) -> String {
    match resolve_path_under_root(root, path) {
        Err(e) => format!("write_file failed: {}", e),
        Ok(p) => {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&p, contents) {
                Ok(()) => {
                    let path_display = p.to_string_lossy().to_string();
                    if !files_changed.contains(&path_display) {
                        files_changed.push(path_display.clone());
                    }
                    emit_activity(
                        events_tx,
                        "file_written",
                        serde_json::json!({ "path": path_display }),
                    );
                    format!("Wrote {} ({} bytes).", p.display(), contents.len())
                }
                Err(e) => format!("write_file failed: {}", e),
            }
        }
    }
}

fn exec_list_dir(root: &Path, path: &str) -> String {
    match resolve_path_under_root(root, path) {
        Err(e) => format!("list_dir failed: {}", e),
        Ok(p) => {
            if !p.is_dir() {
                "list_dir failed: not a directory".to_string()
            } else {
                match std::fs::read_dir(&p) {
                    Ok(entries) => {
                        let mut names: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| {
                                let name = e.file_name().to_string_lossy().into_owned();
                                if e.path().is_dir() {
                                    format!("{}/", name)
                                } else {
                                    name
                                }
                            })
                            .collect();
                        names.sort();
                        names.join("\n")
                    }
                    Err(e) => format!("list_dir failed: {}", e),
                }
            }
        }
    }
}

fn exec_dal_command(root: &Path, subcommand: &str, path: &str) -> String {
    match resolve_path_under_root(root, path) {
        Err(e) => format!("dal_{} failed: {}", subcommand, e),
        Ok(p) => {
            if !p.is_file() {
                return format!("dal_{} failed: path is not a file", subcommand);
            }
            let path_str = p.to_string_lossy().into_owned();
            let exe =
                run_backend::dal_binary_path().unwrap_or_else(|_| std::path::PathBuf::from("dal"));
            match run_backend::run_command_blocking(
                exe.to_string_lossy().as_ref(),
                &[subcommand.to_string(), path_str],
                Some(root),
            ) {
                Ok((stdout, stderr, code)) => {
                    let mut s = format!("Exit code: {}\n", code);
                    if !stdout.is_empty() {
                        s.push_str("stdout:\n");
                        s.push_str(&stdout);
                    }
                    if !stderr.is_empty() {
                        s.push_str("\nstderr:\n");
                        s.push_str(&stderr);
                    }
                    truncate_result(&s)
                }
                Err(e) => format!("dal {} failed: {}", subcommand, e),
            }
        }
    }
}

// ── Step recording ──────────────────────────────────────────────────
// The unified pattern: check result guard → emit agent_step → push
// conversation turns → increment step counter.

struct StepContext<'a> {
    events_tx: &'a broadcast::Sender<String>,
    job_id: &'a str,
    schema: &'a mut AgentContextSchema,
    steps_used: &'a mut u32,
    max_steps: u32,
    loop_state: &'a mut ToolLoopState,
    budget: &'a ResourceBudget,
    /// Current job working directory (`run` tool cwd; updated on `cd`).
    working_directory_display: &'a str,
}

/// Record a completed tool step. Returns `Some(reason)` if a guard or
/// step limit fires and the loop should break.
fn record_step(
    ctx: &mut StepContext<'_>,
    tool_name: &str,
    args_sanitized: &str,
    result: &str,
    assistant_event: &str,
    pending_signature: &Option<(String, String)>,
) -> Option<String> {
    if let Some((_, sig)) = pending_signature.as_ref() {
        if let Some(reason) = check_result_guard(ctx.loop_state, ctx.budget, sig, result) {
            return Some(reason);
        }
    }
    emit_activity(
        ctx.events_tx,
        "agent_step",
        serde_json::json!({
            "job_id": ctx.job_id,
            "step": *ctx.steps_used + 1,
            "tool": tool_name,
            "args_sanitized": args_sanitized,
            "result_summary": truncate_result(result),
            "working_directory": ctx.working_directory_display
        }),
    );
    ctx.schema.conversation.push(ConversationTurn {
        role: "assistant".to_string(),
        content: assistant_event.to_string(),
    });
    ctx.schema.conversation.push(ConversationTurn {
        role: "user".to_string(),
        content: format!("[Tool result]\n{}", result),
    });
    *ctx.steps_used += 1;
    if *ctx.steps_used >= ctx.max_steps {
        return Some("max_steps_reached".to_string());
    }
    None
}

fn guard_break(reason: &str) -> MultiStepResult {
    MultiStepResult {
        final_text: format!("Stopped by guard: {}.", reason),
        is_ask_user: false,
        steps_used: 0, // caller patches this
        max_steps_reached: reason == "max_steps_reached",
        executed_tools: vec![],
        last_tool_success: None,
    }
}

// ── Main entry point ────────────────────────────────────────────────

/// Run the DAL agent loop as a first-class IDE agent.
///
/// 1. Builds an `AgentConfig` of type `IDE` with capabilities and resource
///    budget sourced from env.
/// 2. Calls `agent::spawn` to register in the global agent registry.
/// 3. Runs the model-turn loop with tool dispatch, emitting activity events.
/// 4. Terminates the agent on completion.
pub fn run_agent_prompt_sync(
    workspace_root: &Path,
    prompt: String,
    context: Option<String>,
    job_id: String,
    events_tx: broadcast::Sender<String>,
) -> Result<MultiStepResult, String> {
    // ── 1. Spawn as IDE agent ───────────────────────────────────────
    let budget = resource_budget_from_env();
    let config = AgentConfig::new(format!("ide-agent-{}", job_id), AgentType::IDE)
        .with_role("ide_tool_loop".to_string())
        .with_resource_budget(budget.clone());

    let agent_ctx =
        agent::spawn(config).map_err(|e| format!("Failed to spawn IDE agent: {}", e))?;
    let agent_id = agent_ctx.agent_id.clone();

    // ── 2. Build conversation schema ────────────────────────────────
    let mut schema = AgentContextSchema::minimal(prompt.trim(), TOOLS_SYSTEM_WITH_SCRIPTING);
    schema.completion_and_ask_guidance = Some(completion_and_ask_guidance_for_tool_loop());
    if let Some(ref ctx) = context {
        if !ctx.trim().is_empty() {
            schema.context_blocks.push(ContextBlock {
                source: "workspace".to_string(),
                content: ctx.clone(),
            });
        }
    }

    let root = workspace_root.to_path_buf();
    let root_display = root.to_string_lossy().into_owned();
    let mut job_cwd = root.canonicalize().unwrap_or_else(|_| root.clone());
    let max_steps = max_tool_steps_from_env();
    let mut steps_used: u32 = 0;
    let mut files_changed: Vec<String> = Vec::new();
    let mut loop_state = ToolLoopState::default();
    let loop_started_at = Instant::now();
    let mut termination_reason = "completed".to_string();
    let mut guard_stopped = false;

    emit_activity(
        &events_tx,
        "agent_started",
        serde_json::json!({
            "job_id": job_id,
            "agent_id": agent_id,
            "prompt": schema.objective
        }),
    );

    emit_activity(
        &events_tx,
        "agent_scope",
        serde_json::json!({
            "job_id": job_id,
            "effective_root": root_display.as_str(),
            "effective_cwd": job_cwd.to_string_lossy(),
            "reason": "job_start"
        }),
    );

    let include_scripting_tools = true;

    // ── 3. Model-turn loop ──────────────────────────────────────────
    let result = loop {
        // Wall-clock guard
        if let Some(limit_ms) = budget.max_wall_clock_ms {
            if loop_started_at.elapsed().as_millis() > u128::from(limit_ms) {
                termination_reason = "max_wall_clock_ms".to_string();
                guard_stopped = true;
                break Ok(MultiStepResult {
                    final_text: "Stopped: max wall-clock exceeded.".to_string(),
                    is_ask_user: false,
                    steps_used,
                    max_steps_reached: false,
                    executed_tools: vec![],
                    last_tool_success: None,
                });
            }
        }

        let turn = generate_agent_model_turn(&schema, include_scripting_tools)?;

        // Token / cost guard
        if let Some(reason) = apply_usage_budget(&turn.usage, &budget, &mut loop_state) {
            termination_reason = reason.clone();
            guard_stopped = true;
            break Ok(MultiStepResult {
                final_text: format!("Stopped by guard: {}.", reason),
                is_ask_user: false,
                steps_used,
                max_steps_reached: false,
                executed_tools: vec![],
                last_tool_success: None,
            });
        }

        let parsed = model_turn_to_outcome(&turn);
        let outcome = parsed.outcome;
        let assistant_event = parsed.assistant_event;
        let pending_signature = tool_signature(&outcome);

        // Pre-invocation guard (tool type counts, repeated calls)
        if let Some((ref tool_name, ref signature)) = pending_signature {
            if let Some(reason) =
                check_invocation_guard(&mut loop_state, &budget, tool_name, signature)
            {
                termination_reason = reason.clone();
                guard_stopped = true;
                break Ok(guard_break(&reason));
            }

            // Capability check: agent must have the tool in its capabilities
            if !agent_ctx.is_capable(tool_name) {
                let msg = format!(
                    "IDE agent lacks capability '{}'. Available: {:?}",
                    tool_name, agent_ctx.config.capabilities
                );
                log::warn!("{}", msg);
                // Feed error back to model rather than hard-stopping
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\nError: {}", msg),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                        executed_tools: vec![],
                        last_tool_success: None,
                    });
                }
                continue;
            }
        }

        // ── Execute tool ────────────────────────────────────────────
        let (tool_name, args_sanitized, result_str): (String, String, String) = match outcome {
            ToolOutcome::Reply(text) => {
                break Ok(MultiStepResult {
                    final_text: text,
                    is_ask_user: false,
                    steps_used,
                    max_steps_reached: false,
                    executed_tools: vec![],
                    last_tool_success: None,
                });
            }
            ToolOutcome::AskUser(message) => {
                break Ok(MultiStepResult {
                    final_text: message,
                    is_ask_user: true,
                    steps_used,
                    max_steps_reached: false,
                    executed_tools: vec![],
                    last_tool_success: None,
                });
            }
            ToolOutcome::ParseFail(raw) => {
                break Ok(MultiStepResult {
                    final_text: raw,
                    is_ask_user: false,
                    steps_used,
                    max_steps_reached: false,
                    executed_tools: vec![],
                    last_tool_success: None,
                });
            }
            ToolOutcome::Run(cmd) => {
                let cmd_trimmed = cmd.trim().to_string();
                let run_job_id = format!("{}-run-{}", job_id, steps_used);
                emit_activity(
                    &events_tx,
                    "run_started",
                    serde_json::json!({ "job_id": run_job_id, "cmd": cmd_trimmed }),
                );
                let r = exec_run(&root, &mut job_cwd, &cmd_trimmed, &events_tx, &job_id);
                emit_activity(
                    &events_tx,
                    "run_stopped",
                    serde_json::json!({ "job_id": run_job_id }),
                );
                ("run".into(), cmd_trimmed, r)
            }
            ToolOutcome::Search(query) => {
                let q = query.trim().to_string();
                ("search".into(), q.clone(), exec_search(&q))
            }
            ToolOutcome::FetchUrl(url) => {
                let u = url.trim().to_string();
                ("fetch_url".into(), u.clone(), exec_fetch_url(&u))
            }
            ToolOutcome::ReadFile(path) => {
                let p = path.trim().to_string();
                ("read_file".into(), p.clone(), exec_read_file(&root, &p))
            }
            ToolOutcome::WriteFile(path, contents) => {
                let p = path.trim().to_string();
                let r = exec_write_file(&root, &p, &contents, &mut files_changed, &events_tx);
                ("write_file".into(), p, r)
            }
            ToolOutcome::ListDir(path) => {
                let p = path.trim().to_string();
                ("list_dir".into(), p.clone(), exec_list_dir(&root, &p))
            }
            ToolOutcome::DalCheck(path) => {
                let p = path.trim().to_string();
                (
                    "dal_check".into(),
                    p.clone(),
                    exec_dal_command(&root, "check", &p),
                )
            }
            ToolOutcome::DalRun(path) => {
                let p = path.trim().to_string();
                (
                    "dal_run".into(),
                    p.clone(),
                    exec_dal_command(&root, "run", &p),
                )
            }
            ToolOutcome::ShowUrl(url) => {
                let u = url.trim().to_string();
                emit_activity(&events_tx, "show_url", serde_json::json!({ "url": u }));
                (
                    "show_url".into(),
                    u,
                    "URL displayed in workspace.".to_string(),
                )
            }
            ToolOutcome::ShowContent(content, title) => {
                emit_activity(
                    &events_tx,
                    "show_content",
                    serde_json::json!({ "content": content, "title": title }),
                );
                let sanitized = title.as_deref().unwrap_or("(content)").to_string();
                (
                    "show_content".into(),
                    sanitized,
                    "Content displayed in workspace.".to_string(),
                )
            }
            ToolOutcome::DalInit(template) => {
                let t = template.as_deref().unwrap_or("general").to_string();
                let r = match crate::project_init::run_init(&t, &root) {
                    Ok(msg) => msg,
                    Err(e) => format!("dal_init failed: {}", e),
                };
                ("dal_init".into(), t, r)
            }
        };

        // ── Record step (guard + emit + conversation + counter) ─────
        let wd_display = job_cwd.to_string_lossy();
        let mut step_ctx = StepContext {
            events_tx: &events_tx,
            job_id: &job_id,
            schema: &mut schema,
            steps_used: &mut steps_used,
            max_steps,
            loop_state: &mut loop_state,
            budget: &budget,
            working_directory_display: wd_display.as_ref(),
        };
        if let Some(reason) = record_step(
            &mut step_ctx,
            &tool_name,
            &args_sanitized,
            &result_str,
            &assistant_event,
            &pending_signature,
        ) {
            termination_reason = reason.clone();
            guard_stopped = reason != "max_steps_reached";
            let mut res = guard_break(&reason);
            res.steps_used = steps_used;
            break Ok(res);
        }
    };

    // ── 4. Emit completion, terminate agent ──────────────────────────
    emit_activity(
        &events_tx,
        "agent_stopped",
        serde_json::json!({ "job_id": job_id, "agent_id": agent_id }),
    );

    if !guard_stopped {
        termination_reason = match &result {
            Ok(res) if res.max_steps_reached => "max_steps_reached".to_string(),
            Ok(res) if res.is_ask_user => "ask_user".to_string(),
            Ok(_) => "completed".to_string(),
            Err(_) => "error".to_string(),
        };
    }

    if let Ok(ref res) = result {
        emit_activity(
            &events_tx,
            "completion_summary",
            serde_json::json!({
                "job_id": job_id,
                "agent_id": agent_id,
                "summary": res.final_text,
                "steps_count": res.steps_used,
                "files_changed": files_changed,
                "termination_reason": termination_reason,
                "guard_stopped": guard_stopped,
                "usage_totals": {
                    "total_tokens": loop_state.total_tokens,
                    "estimated_cost_microusd": loop_state.total_cost_microusd
                }
            }),
        );
    }

    // Terminate the IDE agent in the registry
    if let Err(e) = agent::terminate(&agent_id) {
        log::warn!("Failed to terminate IDE agent {}: {}", agent_id, e);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn apply_cd_subdir_and_back() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path().to_path_buf();
        fs::create_dir_all(root.join("a")).unwrap();
        let start = root.canonicalize().unwrap();
        let mut cur = start.clone();
        let r = apply_cd(&start, &cur, "a");
        assert!(r.is_ok());
        cur = r.unwrap();
        assert!(cur.ends_with("a"));
        let r2 = apply_cd(&start, &cur, "..");
        assert!(r2.is_ok());
        assert_eq!(r2.unwrap(), start);
    }

    #[test]
    fn apply_cd_rejects_escape() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path().canonicalize().unwrap();
        let r = apply_cd(&root, &root, "..");
        assert!(r.is_err());
    }
}

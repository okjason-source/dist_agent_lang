//! IDE agent runner: run DAL agent loop with workspace as surface and emit activity.
//! Uses dist_agent_lang (DAL) agent API for model turn parsing.
//! Tool execution goes through the same logic as the IDE API (run_command, read_file, write_file)
//! so activity events (file_written, run_started, run_stopped, agent_step) are emitted.

use crate::agent_context_schema::{AgentContextSchema, ContextBlock, ConversationTurn};
use crate::ide::run_backend;
use crate::stdlib::ai::{
    generate_agent_model_turn, max_tool_steps_from_env, model_turn_to_outcome, run_web_search,
    MultiStepResult, ToolOutcome, COMPLETION_AND_ASK_GUIDANCE, TOOLS_SYSTEM_WITH_SCRIPTING,
};
use std::path::Path;
use tokio::sync::broadcast;

const MAX_TOOL_RESULT_LEN: usize = 4000;

fn emit_activity(tx: &broadcast::Sender<String>, event_type: &str, payload: serde_json::Value) {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let event = serde_json::json!({
        "type": event_type,
        "timestamp": timestamp,
        "payload": payload
    });
    let _ = serde_json::to_string(&event).map(|s| tx.send(s));
}

/// Resolve path relative to root; reject path traversal.
fn resolve_path_under_root(root: &Path, path: &str) -> Result<std::path::PathBuf, String> {
    let path = path.trim();
    if path.is_empty() {
        return Ok(root.to_path_buf());
    }
    if path.contains("..") {
        return Err("Path traversal (..) not allowed".to_string());
    }
    if path.starts_with('/') || (path.len() >= 2 && path.get(..2) == Some("\\\\")) {
        return Err("Absolute paths not allowed".to_string());
    }
    let root_canonical = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => root.to_path_buf(),
    };
    let joined = root_canonical.join(path);
    if joined.exists() {
        let canonical = joined.canonicalize().map_err(|e| e.to_string())?;
        if !canonical.starts_with(&root_canonical) {
            return Err("Path escapes working directory".to_string());
        }
        Ok(canonical)
    } else {
        if !joined.starts_with(&root_canonical) {
            return Err("Path escapes working directory".to_string());
        }
        Ok(joined)
    }
}

fn truncate_result(s: &str) -> String {
    if s.len() <= MAX_TOOL_RESULT_LEN {
        s.to_string()
    } else {
        format!("{}\n... (truncated)", &s[..MAX_TOOL_RESULT_LEN])
    }
}

/// Run the DAL agent loop with IDE-backed tool execution. Emits agent_started, agent_step,
/// file_written (on write_file), run_started/run_stopped (on run), and on completion
/// agent_stopped and completion_summary. Called from the IDE server (e.g. in spawn_blocking).
pub fn run_agent_prompt_sync(
    workspace_root: &Path,
    prompt: String,
    context: Option<String>,
    job_id: String,
    events_tx: broadcast::Sender<String>,
) -> Result<MultiStepResult, String> {
    let mut schema = AgentContextSchema::minimal(prompt.trim(), TOOLS_SYSTEM_WITH_SCRIPTING);
    schema.completion_and_ask_guidance = Some(COMPLETION_AND_ASK_GUIDANCE.to_string());
    if let Some(ref ctx) = context {
        if !ctx.trim().is_empty() {
            schema.context_blocks.push(ContextBlock {
                source: "workspace".to_string(),
                content: ctx.clone(),
            });
        }
    }

    let root = workspace_root.to_path_buf();
    let max_steps = max_tool_steps_from_env();
    let mut steps_used: u32 = 0;
    let mut files_changed: Vec<String> = Vec::new();

    emit_activity(
        &events_tx,
        "agent_started",
        serde_json::json!({ "job_id": job_id, "prompt": schema.objective }),
    );

    let include_scripting_tools = true;
    let result = loop {
        let turn = generate_agent_model_turn(&schema, include_scripting_tools)?;
        let parsed = model_turn_to_outcome(&turn);
        let outcome = parsed.outcome;
        let assistant_event = parsed.assistant_event;

        match outcome {
            ToolOutcome::Reply(text) => {
                break Ok(MultiStepResult {
                    final_text: text,
                    is_ask_user: false,
                    steps_used,
                    max_steps_reached: false,
                });
            }
            ToolOutcome::AskUser(message) => {
                break Ok(MultiStepResult {
                    final_text: message,
                    is_ask_user: true,
                    steps_used,
                    max_steps_reached: false,
                });
            }
            ToolOutcome::ParseFail(raw) => {
                break Ok(MultiStepResult {
                    final_text: raw,
                    is_ask_user: false,
                    steps_used,
                    max_steps_reached: false,
                });
            }
            ToolOutcome::Run(cmd) => {
                let cmd = cmd.trim();
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
                let run_job_id = format!("{}-run-{}", job_id, steps_used);
                emit_activity(
                    &events_tx,
                    "run_started",
                    serde_json::json!({ "job_id": run_job_id, "cmd": cmd }),
                );
                let exec_result =
                    run_backend::run_command_blocking(&cmd_str, &args, Some(root.as_path()));
                emit_activity(
                    &events_tx,
                    "run_stopped",
                    serde_json::json!({ "job_id": run_job_id }),
                );
                let result = match &exec_result {
                    Ok((stdout, stderr, code)) => {
                        let mut out = format!("Exit code: {}\n", code);
                        if !stdout.is_empty() {
                            out.push_str("stdout:\n");
                            out.push_str(stdout);
                        }
                        if !stderr.is_empty() {
                            out.push_str("\nstderr:\n");
                            out.push_str(stderr);
                        }
                        if out.len() > MAX_TOOL_RESULT_LEN {
                            out.truncate(MAX_TOOL_RESULT_LEN);
                            out.push_str("\n... (truncated)");
                        }
                        out
                    }
                    Err(e) => format!("Command failed: {}", e),
                };
                emit_activity(
                    &events_tx,
                    "agent_step",
                    serde_json::json!({
                        "job_id": job_id,
                        "step": steps_used + 1,
                        "tool": "run",
                        "args_sanitized": cmd,
                        "result_summary": truncate_result(&result)
                    }),
                );
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::Search(query) => {
                let result = run_web_search(query.trim())
                    .unwrap_or_else(|e| format!("Search failed: {}", e));
                let result = truncate_result(&result);
                emit_activity(
                    &events_tx,
                    "agent_step",
                    serde_json::json!({
                        "job_id": job_id,
                        "step": steps_used + 1,
                        "tool": "search",
                        "args_sanitized": query,
                        "result_summary": result
                    }),
                );
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::ReadFile(path) => {
                let result = match resolve_path_under_root(&root, &path) {
                    Err(e) => format!("read_file failed: {}", e),
                    Ok(p) => {
                        if !p.is_file() {
                            "read_file failed: not a file".to_string()
                        } else {
                            std::fs::read_to_string(&p)
                                .unwrap_or_else(|e| format!("read_file failed: {}", e))
                        }
                    }
                };
                let result = truncate_result(&result);
                emit_activity(
                    &events_tx,
                    "agent_step",
                    serde_json::json!({
                        "job_id": job_id,
                        "step": steps_used + 1,
                        "tool": "read_file",
                        "args_sanitized": path,
                        "result_summary": result
                    }),
                );
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::WriteFile(path, contents) => {
                let result = match resolve_path_under_root(&root, &path) {
                    Err(e) => format!("write_file failed: {}", e),
                    Ok(p) => {
                        if let Some(parent) = p.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        match std::fs::write(&p, &contents) {
                            Ok(()) => {
                                let path_display = p.to_string_lossy().to_string();
                                if !files_changed.contains(&path_display) {
                                    files_changed.push(path_display.clone());
                                }
                                emit_activity(
                                    &events_tx,
                                    "file_written",
                                    serde_json::json!({ "path": path_display }),
                                );
                                format!("Wrote {} ({} bytes).", p.display(), contents.len())
                            }
                            Err(e) => format!("write_file failed: {}", e),
                        }
                    }
                };
                emit_activity(
                    &events_tx,
                    "agent_step",
                    serde_json::json!({
                        "job_id": job_id,
                        "step": steps_used + 1,
                        "tool": "write_file",
                        "args_sanitized": path,
                        "result_summary": truncate_result(&result)
                    }),
                );
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::ListDir(path) => {
                let result = match resolve_path_under_root(&root, &path) {
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
                };
                emit_activity(
                    &events_tx,
                    "agent_step",
                    serde_json::json!({
                        "job_id": job_id,
                        "step": steps_used + 1,
                        "tool": "list_dir",
                        "args_sanitized": path,
                        "result_summary": truncate_result(&result)
                    }),
                );
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::DalCheck(path) => {
                let result = match resolve_path_under_root(&root, &path) {
                    Err(e) => format!("dal_check failed: {}", e),
                    Ok(p) => {
                        if !p.is_file() {
                            "dal_check failed: path is not a file".to_string()
                        } else {
                            let path_str = p.to_string_lossy().into_owned();
                            let exe = run_backend::dal_binary_path()
                                .unwrap_or_else(|_| std::path::PathBuf::from("dal"));
                            match run_backend::run_command_blocking(
                                exe.to_string_lossy().as_ref(),
                                &["check".to_string(), path_str],
                                Some(root.as_path()),
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
                                    if s.len() > MAX_TOOL_RESULT_LEN {
                                        s.truncate(MAX_TOOL_RESULT_LEN);
                                        s.push_str("\n... (truncated)");
                                    }
                                    s
                                }
                                Err(e) => format!("dal check failed: {}", e),
                            }
                        }
                    }
                };
                emit_activity(
                    &events_tx,
                    "agent_step",
                    serde_json::json!({
                        "job_id": job_id,
                        "step": steps_used + 1,
                        "tool": "dal_check",
                        "args_sanitized": path,
                        "result_summary": truncate_result(&result)
                    }),
                );
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::DalRun(path) => {
                let result = match resolve_path_under_root(&root, &path) {
                    Err(e) => format!("dal_run failed: {}", e),
                    Ok(p) => {
                        if !p.is_file() {
                            "dal_run failed: path is not a file".to_string()
                        } else {
                            let path_str = p.to_string_lossy().into_owned();
                            let exe = run_backend::dal_binary_path()
                                .unwrap_or_else(|_| std::path::PathBuf::from("dal"));
                            match run_backend::run_command_blocking(
                                exe.to_string_lossy().as_ref(),
                                &["run".to_string(), path_str],
                                Some(root.as_path()),
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
                                    if s.len() > MAX_TOOL_RESULT_LEN {
                                        s.truncate(MAX_TOOL_RESULT_LEN);
                                        s.push_str("\n... (truncated)");
                                    }
                                    s
                                }
                                Err(e) => format!("dal run failed: {}", e),
                            }
                        }
                    }
                };
                emit_activity(
                    &events_tx,
                    "agent_step",
                    serde_json::json!({
                        "job_id": job_id,
                        "step": steps_used + 1,
                        "tool": "dal_run",
                        "args_sanitized": path,
                        "result_summary": truncate_result(&result)
                    }),
                );
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::ShowUrl(url) => {
                let url = url.trim().to_string();
                emit_activity(&events_tx, "show_url", serde_json::json!({ "url": url }));
                emit_activity(
                    &events_tx,
                    "agent_step",
                    serde_json::json!({
                        "job_id": job_id,
                        "step": steps_used + 1,
                        "tool": "show_url",
                        "args_sanitized": url,
                        "result_summary": "URL displayed in workspace."
                    }),
                );
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: "[Tool result]\nURL displayed in workspace.".to_string(),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::ShowContent(content, title) => {
                let content = content.to_string();
                let title = title.clone();
                emit_activity(
                    &events_tx,
                    "show_content",
                    serde_json::json!({
                        "content": content,
                        "title": title
                    }),
                );
                emit_activity(
                    &events_tx,
                    "agent_step",
                    serde_json::json!({
                        "job_id": job_id,
                        "step": steps_used + 1,
                        "tool": "show_content",
                        "args_sanitized": title.as_deref().unwrap_or("(content)"),
                        "result_summary": "Content displayed in workspace."
                    }),
                );
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: "[Tool result]\nContent displayed in workspace.".to_string(),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::DalInit(template) => {
                let t = template.as_deref().unwrap_or("general");
                let result = match crate::project_init::run_init(t, &root) {
                    Ok(msg) => msg,
                    Err(e) => format!("dal_init failed: {}", e),
                };
                emit_activity(
                    &events_tx,
                    "agent_step",
                    serde_json::json!({
                        "job_id": job_id,
                        "step": steps_used + 1,
                        "tool": "dal_init",
                        "args_sanitized": t,
                        "result_summary": truncate_result(&result)
                    }),
                );
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    break Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
        }
    };

    emit_activity(
        &events_tx,
        "agent_stopped",
        serde_json::json!({ "job_id": job_id }),
    );

    if let Ok(ref res) = result {
        let summary = res.final_text.clone();
        emit_activity(
            &events_tx,
            "completion_summary",
            serde_json::json!({
                "job_id": job_id,
                "summary": summary,
                "steps_count": res.steps_used,
                "files_changed": files_changed
            }),
        );
    }

    result
}

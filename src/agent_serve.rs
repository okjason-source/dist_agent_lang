//! Agent local server: `dal agent serve [name] [--port N] [--mold path]`
//! One agent per process; HTTP API for status, message, messages, task, tasks, health.
//! See docs/development/AGENT_LOCAL_SERVER_DESIGN.md.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dist_agent_lang::agent_context_schema::{AgentContextSchema, AgentStateBlock, ContextBlock};
use dist_agent_lang::ffi::interface::value_to_json;
use dist_agent_lang::stdlib::agent::{self, AgentContext};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Shared state for the agent server: the single agent's context.
/// When prompt_only is true (no behavior script), the server responds to each message via the AI and posts the reply.
struct AgentServeState {
    ctx: AgentContext,
    prompt_only: bool,
}

/// Max lines of evolve content to include in prompt (P1: evolve in context).
const EVOLVE_RECENT_LINES: i64 = 300;

/// Load recent evolve content as a context block when available (P1).
fn evolve_context_block(agent_name: &str) -> Vec<ContextBlock> {
    match dist_agent_lang::stdlib::evolve::load_recent(Some(agent_name), EVOLVE_RECENT_LINES) {
        Ok(s) if !s.trim().is_empty() => vec![ContextBlock {
            source: "evolve".to_string(),
            content: s,
        }],
        _ => Vec::new(),
    }
}

/// Heuristic: true if objective looks code-related (P3). Used to optionally include DAL summary.
fn objective_seems_code_related(objective: &str) -> bool {
    let lower = objective.to_lowercase();
    let keywords = [
        "dal", "script", "service", "function", "fn ", "code", "explain", "parse", "import",
        "chain::", "agent", "mold",
    ];
    keywords.iter().any(|k| lower.contains(k))
}

/// Build DAL summary context block when requested (P3). Tries path override, then main.dal, then agent.dal in cwd.
fn dal_summary_context_blocks(include: bool, dal_file_override: Option<&str>) -> Vec<ContextBlock> {
    if !include {
        return Vec::new();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let candidates: Vec<std::path::PathBuf> = if let Some(p) = dal_file_override {
        vec![cwd.join(p)]
    } else {
        vec![cwd.join("main.dal"), cwd.join("agent.dal")]
    };
    for path in candidates {
        if path.exists() {
            if let Ok(summary) = dist_agent_lang::dal_summary::summary_from_path(&path) {
                let content = dist_agent_lang::dal_summary::to_context_string(&summary);
                if !content.is_empty() {
                    return vec![ContextBlock {
                        source: "dal_summary".to_string(),
                        content,
                    }];
                }
            }
            break; // only use first candidate when override is set
        }
    }
    Vec::new()
}

/// Tools description for message handler (P4 + P5: explicit tool list, ask_user).
const SERVE_MESSAGE_TOOLS: &str = "You are a helpful assistant. Available tools: reply, run (shell), search, ask_user. Reply with JSON: {\"action\":\"reply\",\"text\":\"...\"} or {\"action\":\"run\",\"cmd\":\"...\"} or {\"action\":\"search\",\"query\":\"...\"} or {\"action\":\"ask_user\",\"message\":\"...\"} when you need human input. When the objective is satisfied, use reply with your final answer. Respect the constraints below.";
/// Tools description for task handler (P4 + P5).
const SERVE_TASK_TOOLS: &str = "You are a helpful assistant. Available tools: reply, run (shell), search, ask_user. Complete the task and use reply with the result when done; use ask_user only when you need human input. Respect the constraints below.";

/// P5: Completion criteria and when to involve the user. Shown in prompt so the agent knows task done vs need user.
const COMPLETION_AND_ASK_GUIDANCE: &str = "Completion: When the objective is met, reply with a clear final answer or outcome; do not ask \"what next?\". When to involve the user: You must ask for confirmation before sensitive actions (e.g. if shell trust is confirmed). You should ask when stuck (ambiguous goal, retries exhausted). Otherwise proceed and only reply when done or with one concrete question. On tool failure, consider one retry or alternative before asking the user.";

/// Build canonical agent context schema for serve (P0). P1: context_blocks; P2: sub_tasks; P4: constraints; P5: completion guidance.
fn build_serve_schema(
    ctx: &AgentContext,
    objective: &str,
    tools_description: &str,
    context_blocks: Vec<ContextBlock>,
    sub_tasks: Option<Vec<String>>,
) -> AgentContextSchema {
    let agent_state = Some(AgentStateBlock {
        capabilities: ctx.config.capabilities.clone(),
        trust_level: ctx.config.agent_type.to_string(),
        working_context: None,
    });
    let constraints = Some(dist_agent_lang::stdlib::sh::constraints_description_for_prompt(
        &dist_agent_lang::stdlib::sh::load_sh_config(),
    ));
    AgentContextSchema {
        objective: objective.to_string(),
        conversation: Vec::new(),
        tools_description: tools_description.to_string(),
        agent_state,
        constraints,
        context_blocks,
        objective_first: false,
        sub_tasks,
        completion_and_ask_guidance: Some(COMPLETION_AND_ASK_GUIDANCE.to_string()),
    }
}

/// GET /status — agent id, name, type, status
async fn handle_status(State(state): State<Arc<AgentServeState>>) -> impl IntoResponse {
    let ctx = &state.ctx;
    let body = serde_json::json!({
        "id": ctx.agent_id,
        "name": ctx.config.name,
        "type": ctx.config.agent_type.to_string(),
        "status": ctx.status.to_string(),
    });
    (StatusCode::OK, Json(body))
}

/// POST /message — body: { sender_id, content } or { sender_id?, message_type?, content }; optional objective, sub_tasks (P2), include_dal_summary (P3).
#[derive(Deserialize)]
struct MessageBody {
    sender_id: String,
    #[serde(default)]
    message_type: String,
    content: String,
    /// Optional explicit objective; if absent, content is used as objective.
    #[serde(default)]
    objective: Option<String>,
    /// Optional sub-tasks for the objective (P2).
    #[serde(default)]
    sub_tasks: Option<Vec<String>>,
    /// Include DAL project summary in context when true; heuristic used if absent (P3).
    #[serde(default)]
    include_dal_summary: Option<bool>,
    /// Optional path to DAL file (relative to cwd); default main.dal / agent.dal (P3).
    #[serde(default)]
    dal_file: Option<String>,
}

async fn handle_message(
    State(state): State<Arc<AgentServeState>>,
    Json(body): Json<MessageBody>,
) -> impl IntoResponse {
    let agent_id = &state.ctx.agent_id;
    let message_type = if body.message_type.is_empty() {
        "user".to_string()
    } else {
        body.message_type
    };
    let content_for_reply = body.content.clone();
    let user_id_for_reply = body.sender_id.clone();
    let message_id = format!(
        "msg_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let msg = agent::create_agent_message(
        message_id,
        body.sender_id,
        agent_id.clone(),
        message_type,
        dist_agent_lang::runtime::values::Value::String(body.content),
    );
    let sender = msg.sender_id.clone();
    match agent::communicate(sender.as_str(), agent_id.as_str(), msg) {
        Ok(_) => {
            if state.prompt_only {
                let agent_id = state.ctx.agent_id.clone();
                let user_id = user_id_for_reply;
                let mut context_blocks = evolve_context_block(state.ctx.config.name.as_str());
                let objective = body
                    .objective
                    .as_deref()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or(&content_for_reply);
                let include_dal = body.include_dal_summary.unwrap_or_else(|| objective_seems_code_related(objective));
                context_blocks.extend(dal_summary_context_blocks(include_dal, body.dal_file.as_deref()));
                let sub_tasks = body.sub_tasks.clone();
                let schema = build_serve_schema(
                    &state.ctx,
                    objective,
                    SERVE_MESSAGE_TOOLS,
                    context_blocks,
                    sub_tasks,
                );
                let content_for_evolve = content_for_reply.clone();
                let agent_name = state.ctx.config.name.clone();
                let max_steps = dist_agent_lang::stdlib::ai::max_tool_steps_from_env();
                tokio::task::spawn_blocking(move || {
                    let mut schema = schema;
                    match dist_agent_lang::stdlib::ai::run_multi_step_tool_loop(
                        &mut schema,
                        max_steps,
                        Some(agent_name.as_str()),
                    ) {
                        Ok(result) => {
                            let reply_trimmed = result.final_text.trim();
                            let _ = dist_agent_lang::stdlib::evolve::append_conversation(
                                &content_for_evolve,
                                reply_trimmed,
                                Some(agent_name.as_str()),
                            );
                            let reply_msg = agent::create_agent_message(
                                format!(
                                    "msg_reply_{}",
                                    SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis()
                                ),
                                agent_id.clone(),
                                user_id.clone(),
                                "assistant".to_string(),
                                dist_agent_lang::runtime::values::Value::String(result.final_text),
                            );
                            let _ = agent::communicate(agent_id.as_str(), user_id.as_str(), reply_msg);
                        }
                        Err(e) => {
                            let _ = dist_agent_lang::stdlib::evolve::append_conversation(
                                &content_for_evolve,
                                &format!("Error: {}", e),
                                Some(agent_name.as_str()),
                            );
                            let reply_msg = agent::create_agent_message(
                                format!(
                                    "msg_reply_{}",
                                    SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis()
                                ),
                                agent_id.clone(),
                                user_id.clone(),
                                "assistant".to_string(),
                                dist_agent_lang::runtime::values::Value::String(format!("Error: {}", e)),
                            );
                            let _ = agent::communicate(agent_id.as_str(), user_id.as_str(), reply_msg);
                        }
                    }
                });
            }
            (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

/// GET /messages — receive (and consume) messages for this agent
async fn handle_messages(State(state): State<Arc<AgentServeState>>) -> impl IntoResponse {
    let messages = agent::receive_messages(state.ctx.agent_id.as_str());
    let arr: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "message_id": m.message_id,
                "sender_id": m.sender_id,
                "message_type": m.message_type,
                "content": value_to_json(&m.content),
            })
        })
        .collect();
    (StatusCode::OK, Json(arr))
}

/// POST /task — body: { task_id?, description, priority?, requester_id? }; optional sub_tasks (P2), include_dal_summary (P3).
/// In prompt-only mode, requester_id (optional) is used to send the task result back as a message.
#[derive(Deserialize)]
struct TaskBody {
    #[serde(default)]
    task_id: String,
    description: String,
    #[serde(default)]
    priority: String,
    #[serde(default)]
    requester_id: String,
    /// Optional sub-tasks for the task (P2).
    #[serde(default)]
    sub_tasks: Option<Vec<String>>,
    /// Include DAL project summary in context when true (P3).
    #[serde(default)]
    include_dal_summary: Option<bool>,
    /// Optional path to DAL file for summary (P3).
    #[serde(default)]
    dal_file: Option<String>,
}

fn task_priority_from_str(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "low" => "low",
        "high" => "high",
        "critical" => "critical",
        "normal" => "medium",
        _ => "medium",
    }
}

async fn handle_task(
    State(state): State<Arc<AgentServeState>>,
    Json(body): Json<TaskBody>,
) -> impl IntoResponse {
    let agent_id = &state.ctx.agent_id;
    let task_id = if body.task_id.is_empty() {
        format!(
            "task_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        )
    } else {
        body.task_id
    };
    let priority = task_priority_from_str(&body.priority);
    let task_opt = agent::create_agent_task(task_id.clone(), body.description.clone(), priority);
    match task_opt {
        Some(task) => {
            if let Err(e) = agent::coordinate(agent_id.as_str(), task, "task_distribution") {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                );
            }
            if state.prompt_only {
                let agent_id = state.ctx.agent_id.clone();
                let requester_id = if body.requester_id.trim().is_empty() {
                    "task_requester".to_string()
                } else {
                    body.requester_id
                };
                let task_description = body.description.clone();
                let mut context_blocks = evolve_context_block(state.ctx.config.name.as_str());
                let include_dal = body.include_dal_summary.unwrap_or_else(|| objective_seems_code_related(&task_description));
                context_blocks.extend(dal_summary_context_blocks(include_dal, body.dal_file.as_deref()));
                let objective = format!(
                    "Complete the following task. Reply with the result or answer only.\n\nTask: {}",
                    task_description
                );
                let schema = build_serve_schema(
                    &state.ctx,
                    &objective,
                    SERVE_TASK_TOOLS,
                    context_blocks,
                    body.sub_tasks.clone(),
                );
                let task_for_evolve = task_description.clone();
                let agent_name = state.ctx.config.name.clone();
                let max_steps = dist_agent_lang::stdlib::ai::max_tool_steps_from_env();
                tokio::task::spawn_blocking(move || {
                    let pending = agent::receive_pending_tasks(agent_id.as_str());
                    if let Some(_t) = pending.into_iter().next() {
                        let mut schema = schema;
                        match dist_agent_lang::stdlib::ai::run_multi_step_tool_loop(
                            &mut schema,
                            max_steps,
                            Some(agent_name.as_str()),
                        ) {
                            Ok(result) => {
                                let result_trimmed = result.final_text.trim();
                                let user_turn = format!("Task: {}", task_for_evolve);
                                let _ = dist_agent_lang::stdlib::evolve::append_conversation(
                                    &user_turn,
                                    result_trimmed,
                                    Some(agent_name.as_str()),
                                );
                                let reply_msg = agent::create_agent_message(
                                    format!(
                                        "msg_task_{}",
                                        SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_millis()
                                    ),
                                    agent_id.clone(),
                                    requester_id.clone(),
                                    "task_result".to_string(),
                                    dist_agent_lang::runtime::values::Value::String(result.final_text),
                                );
                                let _ = agent::communicate(
                                    agent_id.as_str(),
                                    requester_id.as_str(),
                                    reply_msg,
                                );
                            }
                            Err(e) => {
                                let user_turn = format!("Task: {}", task_for_evolve);
                                let _ = dist_agent_lang::stdlib::evolve::append_conversation(
                                    &user_turn,
                                    &format!("Error: {}", e),
                                    Some(agent_name.as_str()),
                                );
                                let reply_msg = agent::create_agent_message(
                                    format!(
                                        "msg_task_{}",
                                        SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_millis()
                                    ),
                                    agent_id.clone(),
                                    requester_id.clone(),
                                    "task_result".to_string(),
                                    dist_agent_lang::runtime::values::Value::String(format!("Error: {}", e)),
                                );
                                let _ = agent::communicate(
                                    agent_id.as_str(),
                                    requester_id.as_str(),
                                    reply_msg,
                                );
                            }
                        }
                    }
                });
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({ "ok": true, "task_id": task_id })),
            )
        }
        None => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid priority; use low, normal, high, critical"
            })),
        ),
    }
}

/// GET /tasks — receive (and consume) pending tasks for this agent
async fn handle_tasks(State(state): State<Arc<AgentServeState>>) -> impl IntoResponse {
    let tasks = agent::receive_pending_tasks(state.ctx.agent_id.as_str());
    let arr: Vec<serde_json::Value> = tasks
        .iter()
        .map(|t| {
            serde_json::json!({
                "task_id": t.task_id,
                "description": t.description,
                "priority": format!("{:?}", t.priority).to_lowercase(),
                "status": format!("{:?}", t.status).to_lowercase(),
                "assigned_at": t.assigned_at,
            })
        })
        .collect();
    (StatusCode::OK, Json(arr))
}

/// GET /health — liveness
async fn handle_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

fn build_router(state: Arc<AgentServeState>) -> Router {
    Router::new()
        .route("/status", get(handle_status))
        .route("/message", post(handle_message))
        .route("/messages", get(handle_messages))
        .route("/task", post(handle_task))
        .route("/tasks", get(handle_tasks))
        .route("/health", get(handle_health))
        .with_state(state)
}

/// Spawn agent: from mold if mold_path is Some, else default config with name and type "worker".
fn spawn_agent(name: &str, mold_path: Option<&str>) -> Result<AgentContext, String> {
    if let Some(source) = mold_path {
        let base = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        dist_agent_lang::mold::create_from_mold_source(source, &base, Some(name), None)
    } else {
        let config =
            agent::create_agent_config(name.to_string(), "worker", "Agent serve".to_string())
                .ok_or_else(|| "Invalid agent type".to_string())?;
        agent::spawn(config)
    }
}

/// Run the agent HTTP server. Blocks until the server stops.
/// If behavior_path is Some, run that DAL file first; the script must spawn an agent and call agent::set_serve_agent(agent_id). Otherwise spawn from name/mold.
/// When behavior_path is None and prompt_only is true, the server responds to each message by calling the AI and posting the reply (prompt directions only, no DAL script).
pub fn run_agent_serve(
    name: &str,
    port: u16,
    mold_path: Option<&str>,
    behavior_path: Option<&str>,
    prompt_only: bool,
) -> Result<(), String> {
    let ctx = if let Some(path) = behavior_path {
        dist_agent_lang::execute_dal_file(path)
            .map_err(|e| format!("Behavior script error: {}", e))?;
        agent::get_serve_agent_context().ok_or_else(|| {
            "Behavior script did not set serve agent. In your script, spawn an agent and call agent::set_serve_agent(agent_id).".to_string()
        })?
    } else {
        spawn_agent(name, mold_path)?
    };
    let display_name = ctx.config.name.clone();
    let prompt_only = prompt_only && behavior_path.is_none();
    let state = Arc::new(AgentServeState { ctx, prompt_only });
    let app = build_router(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!(
        "Agent \"{}\" listening on http://localhost:{}/",
        display_name.as_str(),
        port
    );
    println!("  GET  /status   — agent id, name, type, status");
    println!("  POST /message  — send message (body: sender_id, content)");
    println!("  GET  /messages — receive messages");
    println!("  POST /task     — assign task (body: description, optional task_id, priority)");
    println!("  GET  /tasks    — receive pending tasks");
    println!("  GET  /health   — liveness");
    if prompt_only {
        println!("  (prompt-only mode: replies via AI for each message)");
    }
    println!("Press Ctrl+C to stop");

    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;
        axum::serve(listener, app)
            .await
            .map_err(|e| format!("Server error: {}", e))
    })
}

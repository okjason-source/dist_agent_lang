//! IDE HTTP server: orchestration API, run backend, agent API.
//! Serves the IDE backend for web deployment; desktop can use in-process.

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Path, Query, State};
use axum::http::{Method, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Default parent directory for `dal new` when no cwd is provided. Uses $HOME (Unix) or %USERPROFILE% (Windows); falls back to ".".
fn projects_root() -> PathBuf {
    let s = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(s)
}
use std::time::Duration;
use tokio::sync::{broadcast, oneshot, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::normalize_path::NormalizePathLayer;

use super::diagnostics;
use super::lsp_bridge;
use super::lsp_client::{self, SecondLsp};
use super::orchestration::{discover_workspace, OrchestrationResponse};
use super::run_backend::{
    resolve_run_config, run_command_blocking, spawn_run_streaming, ReadFileRequest,
    RunCommandRequest, WriteFileRequest,
};
use super::symbols;

struct JobEntry {
    output_tx: broadcast::Sender<String>,
    kill_tx: oneshot::Sender<()>,
}

/// Activity event for the events stream (run_started, run_stopped, file_written, command).
#[derive(Debug, Clone, Serialize)]
struct ActivityEvent {
    #[serde(rename = "type")]
    event_type: String,
    timestamp: String,
    payload: serde_json::Value,
}

fn emit_activity(
    events_tx: &broadcast::Sender<String>,
    event_type: &str,
    payload: serde_json::Value,
) {
    let timestamp = Utc::now().to_rfc3339();
    let event = ActivityEvent {
        event_type: event_type.to_string(),
        timestamp,
        payload,
    };
    let _ = serde_json::to_string(&event).map(|s| events_tx.send(s));
}

#[derive(Clone)]
struct AppState {
    workspace_root: PathBuf,
    orchestration: Arc<RwLock<Option<OrchestrationResponse>>>,
    jobs: Arc<RwLock<std::collections::HashMap<String, JobEntry>>>,
    events_tx: broadcast::Sender<String>,
    /// Second LSP (rust-analyzer) per Cargo root; key = directory containing Cargo.toml.
    second_lsp_by_cargo_root: Arc<std::sync::Mutex<HashMap<PathBuf, SecondLsp>>>,
}

/// Query params for orchestration.
#[derive(Debug, Deserialize)]
pub struct WorkspaceQuery {
    #[serde(default)]
    pub workspace: Option<String>,
}

/// Query params for list_files.
#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

/// POST /api/search — workspace text search (optional path scope).
#[derive(Debug, Deserialize)]
struct SearchRequest {
    /// Search string (plain text; case-insensitive by default).
    query: String,
    /// Optional subpath relative to workspace to limit search (e.g. "src" or "" for whole workspace).
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    case_sensitive: bool,
}

#[derive(Debug, Serialize)]
struct SearchResultItem {
    path: String,
    line_number: u32,
    line: String,
}

fn search_workspace_blocking(
    root: &std::path::Path,
    subpath: &str,
    query: &str,
    case_sensitive: bool,
) -> Vec<SearchResultItem> {
    let dir = if subpath.is_empty() {
        root.to_path_buf()
    } else {
        root.join(subpath)
    };
    if !dir.exists() || !dir.is_dir() {
        return Vec::new();
    }
    let query_lower = if case_sensitive {
        String::new()
    } else {
        query.to_lowercase()
    };
    let query = query.as_bytes();
    let mut results = Vec::new();
    let skip_dirs = ["node_modules", "target", ".git", ".venv", "venv", "dist"];
    let text_extensions = [
        "dal", "rs", "js", "ts", "jsx", "tsx", "json", "toml", "md", "txt", "html", "css", "yml",
        "yaml", "sh", "py",
    ];
    let mut stack = vec![(dir, subpath.to_string())];
    while let Some((dir_path, rel_prefix)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir_path) else {
            continue;
        };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') && name != ".dal" {
                continue;
            }
            let meta = match e.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let rel = if rel_prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", rel_prefix, name)
            };
            if meta.is_dir() {
                if skip_dirs.contains(&name.as_str()) {
                    continue;
                }
                stack.push((dir_path.join(&name), rel));
                continue;
            }
            let ext = std::path::Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !text_extensions.contains(&ext) {
                continue;
            }
            let full = dir_path.join(&name);
            let content = match std::fs::read(&full) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let Ok(text) = std::str::from_utf8(&content) else {
                continue;
            };
            for (i, line) in text.lines().enumerate() {
                let line_number = (i + 1) as u32;
                let matched = if case_sensitive {
                    line.contains(std::str::from_utf8(query).unwrap_or(""))
                } else {
                    line.to_lowercase().contains(&query_lower)
                };
                if matched {
                    results.push(SearchResultItem {
                        path: rel.clone(),
                        line_number,
                        line: line.to_string(),
                    });
                }
            }
        }
    }
    results
}

/// GET /api/orchestration — project info, run configs, scripts, workflows.
async fn get_orchestration(
    State(state): State<AppState>,
    Query(query): Query<WorkspaceQuery>,
) -> impl IntoResponse {
    let root = match &query.workspace {
        Some(ws) if !ws.trim().is_empty() => {
            let p = PathBuf::from(ws.trim());
            if p.is_absolute() && p.exists() {
                p
            } else {
                state.workspace_root.join(ws.trim())
            }
        }
        _ => state.workspace_root.clone(),
    };

    if !root.exists() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Workspace path does not exist"})),
        )
            .into_response();
    }

    let resp = discover_workspace(&root);
    *state.orchestration.write().await = Some(resp.clone());
    let root_uri = lsp_client::path_to_file_uri(&root);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "root_uri": root_uri,
            "projects": resp.projects,
            "run_configs": resp.run_configs,
            "scripts": resp.scripts,
            "workflows": resp.workflows,
            "agent_evolve": resp.agent_evolve
        })),
    )
        .into_response()
}

/// GET /api/files — list directory contents (path relative to workspace).
async fn get_files(
    State(state): State<AppState>,
    Query(query): Query<ListFilesQuery>,
) -> impl IntoResponse {
    let root = match &query.workspace {
        Some(ws) => {
            let p = PathBuf::from(ws);
            if p.is_absolute() {
                p
            } else {
                state.workspace_root.join(ws)
            }
        }
        None => state.workspace_root.clone(),
    };

    let subpath = query.path.as_deref().unwrap_or("");
    let dir = if subpath.is_empty() {
        root
    } else {
        root.join(subpath)
    };

    if !dir.exists() || !dir.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Path does not exist or is not a directory"})),
        )
            .into_response();
    }

    let mut entries = Vec::new();
    match std::fs::read_dir(&dir) {
        Ok(rd) => {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') && name != ".dal" {
                    continue; // Skip hidden except .dal
                }
                if ["node_modules", "target", ".git", ".venv", "venv"].contains(&name.as_str()) {
                    continue; // Skip common build/cache dirs
                }
                let meta = match e.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let is_dir = meta.is_dir();
                let rel = if subpath.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", subpath, name)
                };
                entries.push(serde_json::json!({
                    "name": name,
                    "path": rel,
                    "is_dir": is_dir,
                }));
            }
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }

    entries.sort_by(|a, b| {
        let a_dir = a["is_dir"].as_bool().unwrap_or(false);
        let b_dir = b["is_dir"].as_bool().unwrap_or(false);
        match (a_dir, b_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a["name"]
                .as_str()
                .unwrap_or("")
                .cmp(b["name"].as_str().unwrap_or("")),
        }
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({"entries": entries})),
    )
        .into_response()
}

/// POST /api/search — workspace text search.
async fn post_search(
    State(state): State<AppState>,
    Query(query): Query<WorkspaceQuery>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let root = query
        .workspace
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| state.workspace_root.clone());
    if !root.exists() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Workspace path does not exist"})),
        )
            .into_response();
    }
    let q = req.query.trim().to_string();
    if q.is_empty() {
        return (StatusCode::OK, Json(serde_json::json!({"results": []}))).into_response();
    }
    let subpath = req.path.clone().unwrap_or_default();
    let root_clone = root.clone();
    let case_sensitive = req.case_sensitive;
    let results = match tokio::task::spawn_blocking(move || {
        search_workspace_blocking(&root_clone, &subpath, &q, case_sensitive)
    })
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };
    (
        StatusCode::OK,
        Json(serde_json::json!({"results": results})),
    )
        .into_response()
}

/// POST /api/run — run a config by id (blocking, returns full output).
#[derive(Debug, Deserialize)]
struct RunConfigRequest {
    config_id: String,
}

/// POST /api/run/stream — start a config, returns job_id. Connect to GET /api/run/stream/:job_id for SSE output.
async fn post_run_stream(
    State(state): State<AppState>,
    Json(req): Json<RunConfigRequest>,
) -> impl IntoResponse {
    let orch = state.orchestration.read().await;
    let orch = match orch.as_ref() {
        Some(o) => o,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Orchestration not loaded"})),
            )
                .into_response()
        }
    };
    let (cmd, args, cwd) = match resolve_run_config(&req.config_id, &orch.run_configs) {
        Some(x) => x,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Config not found"})),
            )
                .into_response()
        }
    };

    let cmd = if cmd == "dal" {
        super::run_backend::dal_binary_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "dal".to_string())
    } else {
        cmd
    };

    let cwd_path = Some(std::path::Path::new(&cwd));
    match spawn_run_streaming(&cmd, &args, cwd_path) {
        Ok((output_tx, kill_tx)) => {
            let job_id = format!(
                "run-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
            state
                .jobs
                .write()
                .await
                .insert(job_id.clone(), JobEntry { output_tx, kill_tx });
            emit_activity(
                &state.events_tx,
                "run_started",
                serde_json::json!({ "job_id": job_id, "config_id": req.config_id }),
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "job_id": job_id,
                    "config_id": req.config_id
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/run/stream/:job_id — SSE stream of job output.
async fn get_run_stream(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    let rx = {
        let jobs = state.jobs.read().await;
        match jobs.get(&job_id) {
            Some(entry) => entry.output_tx.subscribe(),
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Job not found"})),
                )
                    .into_response()
            }
        }
    };

    enum StreamState {
        Receiving(broadcast::Receiver<String>),
        Done,
    }
    let stream = futures_util::stream::unfold(StreamState::Receiving(rx), |state| async move {
        match state {
            StreamState::Done => None,
            StreamState::Receiving(rx) => {
                let mut rx = rx;
                match rx.recv().await {
                    Ok(msg) => {
                        let event = Event::default().data(msg.clone());
                        let next = if msg == "[DONE]" {
                            StreamState::Done
                        } else {
                            StreamState::Receiving(rx)
                        };
                        Some((Ok::<_, std::convert::Infallible>(event), next))
                    }
                    Err(_) => None,
                }
            }
        }
    });

    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        )
        .into_response()
}

/// POST /api/run/stop — stop a running job.
#[derive(Debug, Deserialize)]
struct StopJobRequest {
    job_id: String,
}

async fn post_run_stop(
    State(state): State<AppState>,
    Json(req): Json<StopJobRequest>,
) -> impl IntoResponse {
    let killed = {
        let mut jobs = state.jobs.write().await;
        if let Some(entry) = jobs.remove(&req.job_id) {
            let _ = entry.kill_tx.send(());
            true
        } else {
            false
        }
    };
    if killed {
        emit_activity(
            &state.events_tx,
            "run_stopped",
            serde_json::json!({ "job_id": req.job_id }),
        );
        (
            StatusCode::OK,
            Json(serde_json::json!({"ok": true, "message": "Job stopped"})),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Job not found"})),
        )
            .into_response()
    }
}

async fn post_run(
    State(state): State<AppState>,
    Json(req): Json<RunConfigRequest>,
) -> impl IntoResponse {
    let orch = state.orchestration.read().await;
    let orch = match orch.as_ref() {
        Some(o) => o,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Orchestration not loaded"})),
            )
                .into_response()
        }
    };
    let (cmd, args, cwd) = match resolve_run_config(&req.config_id, &orch.run_configs) {
        Some(x) => x,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Config not found"})),
            )
                .into_response()
        }
    };

    // Resolve "dal" to actual binary
    let cmd = if cmd == "dal" {
        super::run_backend::dal_binary_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "dal".to_string())
    } else {
        cmd
    };

    let cwd_path = Some(std::path::Path::new(&cwd));
    match run_command_blocking(&cmd, &args, cwd_path) {
        Ok((stdout, stderr, code)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "job_id": format!("run-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
                "exit_code": code,
                "stdout": stdout,
                "stderr": stderr
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// POST /api/agent/run_command — run arbitrary CLI command (agent API).
async fn post_agent_run_command(
    State(state): State<AppState>,
    Json(req): Json<RunCommandRequest>,
) -> impl IntoResponse {
    let (cmd, args) = if req.args.is_empty() {
        // Parse "dal run foo.dal" style
        let parts: Vec<&str> = req.cmd.split_whitespace().collect();
        if parts.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Empty command"})),
            );
        }
        let cmd = parts[0].to_string();
        let args: Vec<String> = parts[1..].iter().map(|s| (*s).to_string()).collect();
        (cmd, args)
    } else {
        (req.cmd, req.args)
    };

    let cwd = req.cwd.as_ref().and_then(|s| {
        let s = s.trim();
        if s.is_empty() {
            None
        } else {
            Some(PathBuf::from(s))
        }
    });
    let cwd = cwd.unwrap_or_else(|| {
        if cmd == "dal" && args.first().map(|a| a.as_str()) == Some("new") {
            projects_root()
        } else {
            state.workspace_root.clone()
        }
    });
    let cwd_path = if cwd.exists() {
        Some(cwd.as_path())
    } else {
        None
    };

    match run_command_blocking(&cmd, &args, cwd_path) {
        Ok((stdout, stderr, code)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "exit_code": code,
                "stdout": stdout,
                "stderr": stderr
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        ),
    }
}

/// POST /api/lsp/diagnostics — get parse/lex diagnostics; route .rs to second LSP (e.g. rust-analyzer).
#[derive(Debug, Deserialize)]
struct LspDiagnosticsRequest {
    contents: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    workspace: Option<String>,
}

async fn post_lsp_diagnostics(
    State(state): State<AppState>,
    Json(req): Json<LspDiagnosticsRequest>,
) -> impl IntoResponse {
    let path = req.path.as_deref().unwrap_or("").trim();
    let use_rust = path.ends_with(".rs");
    if use_rust && !path.is_empty() {
        let root = resolve_workspace_root(&state, req.workspace.as_deref());
        let path_buf = std::path::Path::new(path);
        let full_path = if path_buf.is_absolute() {
            path_buf.to_path_buf()
        } else {
            root.join(path)
        };
        let cargo_root = find_cargo_root(&full_path, &state.workspace_root);
        let uri = lsp_client::path_to_file_uri(&full_path);
        let contents = req.contents.clone();
        let state_clone = state.clone();
        match tokio::task::spawn_blocking(move || {
            let mut guard = state_clone.second_lsp_by_cargo_root.lock().unwrap();
            if !guard.contains_key(&cargo_root) {
                if let Some(lsp) = SecondLsp::spawn(&cargo_root) {
                    guard.insert(cargo_root.clone(), lsp);
                }
            }
            guard
                .get(&cargo_root)
                .and_then(|lsp| lsp.request_diagnostics(uri, contents))
        })
        .await
        {
            Ok(Some(diags)) => {
                let out: Vec<serde_json::Value> = diags
                    .into_iter()
                    .map(|d| {
                        serde_json::json!({
                            "line": d.line,
                            "column": d.column,
                            "end_line": d.end_line,
                            "end_column": d.end_column,
                            "message": d.message,
                            "severity": d.severity
                        })
                    })
                    .collect();
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({ "diagnostics": out })),
                )
                    .into_response();
            }
            _ => {}
        }
    }
    let diags = diagnostics::diagnostics_from_source(&req.contents);
    (
        StatusCode::OK,
        Json(serde_json::json!({ "diagnostics": diags })),
    )
        .into_response()
}

/// WebSocket upgrade for full LSP (dal lsp over stdio bridged to client).
/// Connect to GET /api/lsp/stream; requires `dal lsp` on PATH (build with --features lsp).
async fn get_lsp_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    let workspace_root = state.workspace_root.clone();
    ws.on_upgrade(move |socket| lsp_bridge::run_lsp_bridge(socket, workspace_root))
}

fn resolve_workspace_root(state: &AppState, workspace: Option<&str>) -> PathBuf {
    match workspace {
        Some(ws) if !ws.trim().is_empty() => {
            let p = PathBuf::from(ws.trim());
            if p.is_absolute() && p.exists() {
                p
            } else {
                state.workspace_root.join(ws.trim())
            }
        }
        _ => state.workspace_root.clone(),
    }
}

/// Resolve the Cargo project root (directory containing Cargo.toml) for a Rust file.
/// Walks up from the file's directory; stops at workspace_root or when Cargo.toml is found.
/// Returns workspace_root if no Cargo.toml is found in between.
fn find_cargo_root(file_path: &std::path::Path, workspace_root: &std::path::Path) -> PathBuf {
    let mut dir = file_path
        .parent()
        .filter(|p: &&std::path::Path| !p.as_os_str().is_empty())
        .unwrap_or_else(|| file_path);
    loop {
        if dir.join("Cargo.toml").exists() {
            return dir.to_path_buf();
        }
        if dir == workspace_root {
            return workspace_root.to_path_buf();
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return workspace_root.to_path_buf(),
        }
    }
}

/// POST /api/lsp/hover — hover at position; route .rs to second LSP.
#[derive(Debug, Deserialize)]
struct LspHoverRequest {
    contents: String,
    line: u32,
    character: u32,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    workspace: Option<String>,
}

async fn post_lsp_hover(
    State(state): State<AppState>,
    Json(req): Json<LspHoverRequest>,
) -> impl IntoResponse {
    let path = req.path.as_deref().unwrap_or("").trim();
    let use_rust = path.ends_with(".rs");
    if use_rust && !path.is_empty() {
        let root = resolve_workspace_root(&state, req.workspace.as_deref());
        let path_buf = std::path::Path::new(path);
        let full_path = if path_buf.is_absolute() {
            path_buf.to_path_buf()
        } else {
            root.join(path)
        };
        let cargo_root = find_cargo_root(&full_path, &state.workspace_root);
        let uri = lsp_client::path_to_file_uri(&full_path);
        let contents = req.contents.clone();
        let line = req.line;
        let character = req.character;
        let state_clone = state.clone();
        match tokio::task::spawn_blocking(move || {
            let mut guard = state_clone.second_lsp_by_cargo_root.lock().unwrap();
            if !guard.contains_key(&cargo_root) {
                if let Some(lsp) = SecondLsp::spawn(&cargo_root) {
                    guard.insert(cargo_root.clone(), lsp);
                }
            }
            guard
                .get(&cargo_root)
                .and_then(|lsp| lsp.request_hover(uri, contents, line, character))
        })
        .await
        {
            Ok(Some(hover_content)) => {
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({ "contents": hover_content })),
                )
                    .into_response();
            }
            _ => {}
        }
    }
    let content = diagnostics::hover_at_position(&req.contents, req.line, req.character);
    (
        StatusCode::OK,
        Json(serde_json::json!({ "contents": content })),
    )
        .into_response()
}

/// POST /api/lsp/completion — completion at position; route .rs to second LSP.
#[derive(Debug, Deserialize)]
struct LspCompletionRequest {
    contents: String,
    line: u32,
    character: u32,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    workspace: Option<String>,
}

async fn post_lsp_completion(
    State(state): State<AppState>,
    Json(req): Json<LspCompletionRequest>,
) -> impl IntoResponse {
    let path = req.path.as_deref().unwrap_or("").trim();
    let use_rust = path.ends_with(".rs");
    if use_rust && !path.is_empty() {
        let root = resolve_workspace_root(&state, req.workspace.as_deref());
        let path_buf = std::path::Path::new(path);
        let full_path = if path_buf.is_absolute() {
            path_buf.to_path_buf()
        } else {
            root.join(path)
        };
        let cargo_root = find_cargo_root(&full_path, &state.workspace_root);
        let uri = lsp_client::path_to_file_uri(&full_path);
        let contents = req.contents.clone();
        let line = req.line;
        let character = req.character;
        let state_clone = state.clone();
        match tokio::task::spawn_blocking(move || {
            let mut guard = state_clone.second_lsp_by_cargo_root.lock().unwrap();
            if !guard.contains_key(&cargo_root) {
                if let Some(lsp) = SecondLsp::spawn(&cargo_root) {
                    guard.insert(cargo_root.clone(), lsp);
                }
            }
            guard
                .get(&cargo_root)
                .and_then(|lsp| lsp.request_completion(uri, contents, line, character))
        })
        .await
        {
            Ok(Some(items)) => {
                let out: Vec<serde_json::Value> = items
                    .into_iter()
                    .map(|c| {
                        serde_json::json!({
                            "label": c.label,
                            "kind": c.kind,
                            "detail": c.detail,
                            "insertText": c.insert_text
                        })
                    })
                    .collect();
                return (StatusCode::OK, Json(serde_json::json!({ "items": out }))).into_response();
            }
            _ => {}
        }
    }
    let items = diagnostics::completion_at_position(&req.contents, req.line, req.character);
    (StatusCode::OK, Json(serde_json::json!({ "items": items }))).into_response()
}

/// POST /api/lsp/document_symbols — symbols (functions, services, methods) with location.
#[derive(Debug, Deserialize)]
struct DocumentSymbolsRequest {
    #[serde(default)]
    contents: String,
}

async fn post_lsp_document_symbols(Json(req): Json<DocumentSymbolsRequest>) -> impl IntoResponse {
    let symbols = symbols::document_symbols_from_source(&req.contents);
    (
        StatusCode::OK,
        Json(serde_json::json!({ "symbols": symbols })),
    )
        .into_response()
}

/// POST /api/lsp/references — call sites of a symbol name in the source.
#[derive(Debug, Deserialize)]
struct ReferencesRequest {
    #[serde(default)]
    contents: String,
    name: String,
}

async fn post_lsp_references(Json(req): Json<ReferencesRequest>) -> impl IntoResponse {
    let references = symbols::references_in_source(&req.contents, req.name.trim());
    (
        StatusCode::OK,
        Json(serde_json::json!({ "references": references })),
    )
        .into_response()
}

/// POST /api/agent/read_file — read file contents (agent API).
async fn post_agent_read_file(
    State(state): State<AppState>,
    Json(req): Json<ReadFileRequest>,
) -> impl IntoResponse {
    let path_str = req.path.trim();
    if path_str.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "path is required"})),
        )
            .into_response();
    }
    let root = match &req.workspace {
        Some(ws) => {
            let ws = ws.trim();
            if ws.is_empty() {
                state.workspace_root.clone()
            } else {
                let p = PathBuf::from(ws);
                if p.is_absolute() && p.exists() {
                    p
                } else {
                    state.workspace_root.join(ws)
                }
            }
        }
        None => state.workspace_root.clone(),
    };
    let path = if std::path::Path::new(path_str).is_absolute() {
        PathBuf::from(path_str)
    } else {
        root.join(path_str)
    };

    if !path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "file not found", "path": path.to_string_lossy()})),
        )
            .into_response();
    }
    if path.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({"error": "path is a directory", "path": path.to_string_lossy()}),
            ),
        )
            .into_response();
    }

    match std::fs::read_to_string(&path) {
        Ok(contents) => (
            StatusCode::OK,
            Json(serde_json::json!({"ok": true, "contents": contents})),
        )
            .into_response(),
        Err(e) => {
            let code = if e.kind() == std::io::ErrorKind::NotFound {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (code, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// POST /api/agent/write_file — create or overwrite file (agent API).
async fn post_agent_write_file(
    State(state): State<AppState>,
    Json(req): Json<WriteFileRequest>,
) -> impl IntoResponse {
    let path = if std::path::Path::new(&req.path).is_absolute() {
        PathBuf::from(&req.path)
    } else {
        state.workspace_root.join(&req.path)
    };

    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to create parent dir: {}", e)})),
            );
        }
    }

    match std::fs::write(&path, &req.contents) {
        Ok(()) => {
            emit_activity(
                &state.events_tx,
                "file_written",
                serde_json::json!({ "path": req.path }),
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({"ok": true, "path": path.to_string_lossy()})),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

/// POST /api/command — submit a command/prompt (emits activity event; does not run CLI).
#[derive(Debug, Deserialize)]
struct CommandRequest {
    text: String,
    #[serde(default)]
    context: Option<String>,
}

async fn post_command(
    State(state): State<AppState>,
    Json(req): Json<CommandRequest>,
) -> impl IntoResponse {
    let payload = serde_json::json!({
        "text": req.text,
        "context": req.context,
    });
    emit_activity(&state.events_tx, "command", payload);
    (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response()
}

/// GET /api/config — IDE config (e.g. default parent path for `dal new`).
async fn get_config() -> impl IntoResponse {
    let root = projects_root().to_string_lossy().to_string();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "projects_root": root })),
    )
        .into_response()
}

/// GET /api/events/stream — SSE stream of activity events (run_started, run_stopped, file_written, command).
async fn get_events_stream(State(state): State<AppState>) -> impl IntoResponse {
    let rx = state.events_tx.subscribe();
    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(msg) => {
                let event = Event::default().data(msg);
                Some((Ok::<_, std::convert::Infallible>(event), rx))
            }
            Err(_) => None,
        }
    });
    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        )
        .into_response()
}

/// Build the IDE API router.
pub fn build_router(workspace_root: PathBuf) -> Router {
    let (events_tx, _) = broadcast::channel(256);
    let state = AppState {
        workspace_root: workspace_root.clone(),
        orchestration: Arc::new(RwLock::new(Some(discover_workspace(&workspace_root)))),
        jobs: Arc::new(RwLock::new(std::collections::HashMap::new())),
        events_tx,
        second_lsp_by_cargo_root: Arc::new(std::sync::Mutex::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    Router::new()
        .route("/api/config", get(get_config))
        .route("/api/orchestration", get(get_orchestration))
        .route("/api/files", get(get_files))
        .route("/api/search", post(post_search))
        .route("/api/run", post(post_run))
        .route("/api/run/stream", post(post_run_stream))
        .route("/api/run/stream/:job_id", get(get_run_stream))
        .route("/api/run/stop", post(post_run_stop))
        .route("/api/lsp/diagnostics", post(post_lsp_diagnostics))
        .route("/api/lsp/hover", post(post_lsp_hover))
        .route("/api/lsp/completion", post(post_lsp_completion))
        .route("/api/lsp/document_symbols", post(post_lsp_document_symbols))
        .route(
            "/api/lsp/document_symbols/",
            post(post_lsp_document_symbols),
        )
        .route("/api/lsp/references", post(post_lsp_references))
        .route("/api/lsp/stream", get(get_lsp_ws))
        .route("/api/agent/run_command", post(post_agent_run_command))
        .route("/api/agent/write_file", post(post_agent_write_file))
        .route("/api/agent/read_file", post(post_agent_read_file))
        .route("/api/command", post(post_command))
        .route("/api/events/stream", get(get_events_stream))
        .route("/health", get(|| async { "OK" }))
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(cors)
        .with_state(state)
}

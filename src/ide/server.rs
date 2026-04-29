//! IDE HTTP server: orchestration API, run backend, agent API.
//! Serves the IDE backend for web deployment; desktop can use in-process.

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Path, Query, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{middleware, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Sanitize a user-supplied relative path so it cannot escape a base directory.
/// Resolves "." and ".."; returns None if path is absolute or ".." would escape.
fn sanitize_relative_subpath(path: &str) -> Option<PathBuf> {
    let path = path.trim();
    if path.is_empty() {
        return Some(PathBuf::new());
    }
    let p = std::path::Path::new(path);
    if p.is_absolute() {
        return None;
    }
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            std::path::Component::Normal(c) => {
                out.push(c);
            }
            _ => return None,
        }
    }
    Some(out)
}

/// Resolve a user-supplied path under a base directory. Returns None if the path
/// would escape the base (path traversal) or is absolute.
fn path_under_base(base: &std::path::Path, user_path: &str) -> Option<PathBuf> {
    let sanitized = sanitize_relative_subpath(user_path)?;
    Some(base.join(sanitized))
}

/// Resolve a user-supplied path (relative or absolute) to a path under root.
/// Absolute paths are allowed only if they canonicalize to under root.
fn resolve_path_under_root(root: &std::path::Path, user_path: &str) -> Option<PathBuf> {
    let user_path = user_path.trim();
    if user_path.is_empty() {
        return None;
    }
    let p = std::path::Path::new(user_path);
    if p.is_absolute() {
        let canon_root = root.canonicalize().ok()?;
        let canon_p = p.canonicalize().ok()?;
        if canon_p.starts_with(&canon_root) {
            Some(canon_p)
        } else {
            None
        }
    } else {
        path_under_base(root, user_path)
    }
}

/// Best-effort guard that a path is contained by root.
/// For existing paths, compares canonical paths to account for symlinks.
/// For non-existing paths (e.g. writes), falls back to lexical prefix check.
fn path_is_within_root(root: &std::path::Path, path: &std::path::Path) -> bool {
    if let (Ok(canon_root), Ok(canon_path)) = (root.canonicalize(), path.canonicalize()) {
        return canon_path.starts_with(&canon_root);
    }
    path.starts_with(root)
}

/// Default parent directory for `dal new` when no cwd is provided. Uses $HOME (Unix) or %USERPROFILE% (Windows); falls back to ".".
fn projects_root() -> PathBuf {
    let s = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(s)
}

fn image_mime_for_path(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        Some("avif") => "image/avif",
        _ => "application/octet-stream",
    }
}
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, oneshot, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::normalize_path::NormalizePathLayer;

use super::agent_runner;
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
    kill_tx: Arc<std::sync::Mutex<Option<oneshot::Sender<()>>>>,
    replay_history: Arc<std::sync::Mutex<std::collections::VecDeque<ReplayRecord>>>,
}

use super::agent_runner::emit_activity;

#[derive(Clone)]
struct AppState {
    workspace_root: PathBuf,
    orchestration: Arc<RwLock<Option<OrchestrationResponse>>>,
    jobs: Arc<RwLock<std::collections::HashMap<String, JobEntry>>>,
    events_tx: broadcast::Sender<String>,
    events_replay_history: Arc<std::sync::Mutex<std::collections::VecDeque<ReplayRecord>>>,
    client_active_streams: Arc<std::sync::Mutex<HashMap<String, usize>>>,
    client_stream_establishes:
        Arc<std::sync::Mutex<HashMap<String, std::collections::VecDeque<Instant>>>>,
    sse_phase0: IdeSsePhase0Config,
    /// Second LSP (rust-analyzer) per Cargo root; key = directory containing Cargo.toml.
    second_lsp_by_cargo_root: Arc<std::sync::Mutex<HashMap<PathBuf, SecondLsp>>>,
}

#[derive(Clone, Debug)]
struct ReplayRecord {
    seq: u64,
    msg: String,
}

#[derive(Clone, Debug)]
struct IdeSsePhase0Config {
    structured_envelope: bool,
    replay_enabled: bool,
    replay_capacity: usize,
    keepalive_secs: u64,
    job_retention_secs: u64,
    max_chunk_bytes: usize,
    max_streams_per_client: usize,
    max_establish_per_minute: usize,
    max_stream_lifetime_secs: u64,
    idle_timeout_secs: u64,
    max_header_bytes: usize,
    max_body_bytes: usize,
    cors_allow_any: bool,
    cors_allow_origin: Option<String>,
    sse_auth_token: Option<String>,
    version: String,
}

#[derive(Serialize)]
struct SseEnvelope {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    timestamp: String,
    payload: serde_json::Value,
    version: String,
}

impl IdeSsePhase0Config {
    fn from_env() -> Self {
        Self {
            structured_envelope: env_flag("DAL_IDE_SSE_STRUCTURED"),
            replay_enabled: env_flag("DAL_IDE_SSE_REPLAY"),
            replay_capacity: env_usize("DAL_IDE_SSE_REPLAY_CAP", 512),
            keepalive_secs: env_u64("DAL_IDE_SSE_KEEPALIVE_SECS", 15),
            job_retention_secs: env_u64("DAL_IDE_SSE_JOB_RETENTION_SECS", 120),
            max_chunk_bytes: env_usize("DAL_IDE_SSE_MAX_CHUNK_BYTES", 16384),
            max_streams_per_client: env_usize("DAL_IDE_SSE_MAX_STREAMS_PER_CLIENT", 8),
            max_establish_per_minute: env_usize("DAL_IDE_SSE_MAX_ESTABLISH_PER_MINUTE", 120),
            max_stream_lifetime_secs: env_u64("DAL_IDE_SSE_MAX_STREAM_LIFETIME_SECS", 3600),
            idle_timeout_secs: env_u64("DAL_IDE_SSE_IDLE_TIMEOUT_SECS", 300),
            max_header_bytes: env_usize("DAL_IDE_SSE_MAX_HEADER_BYTES", 16384),
            max_body_bytes: env_usize("DAL_IDE_MAX_BODY_BYTES", 1048576),
            cors_allow_any: env_flag("DAL_IDE_CORS_ALLOW_ANY"),
            cors_allow_origin: std::env::var("DAL_IDE_CORS_ALLOW_ORIGIN")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            sse_auth_token: std::env::var("DAL_IDE_SSE_AUTH_TOKEN")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            version: std::env::var("DAL_IDE_SSE_VERSION")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "sse.v1".to_string()),
        }
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(default)
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn parse_stream_terminal_marker(msg: &str) -> Option<(&'static str, Option<i64>)> {
    if msg == "[DONE]" {
        return Some(("done", None));
    }
    if msg == "[CANCELLED]" {
        return Some(("cancelled", None));
    }
    msg.strip_prefix("[ERROR:")
        .and_then(|s| s.strip_suffix(']'))
        .map(|code| ("error", code.trim().parse::<i64>().ok()))
}

fn to_structured_event(
    id: u64,
    event_type: &str,
    timestamp: String,
    payload: serde_json::Value,
    version: &str,
) -> Event {
    let body = serde_json::to_string(&SseEnvelope {
        id: id.to_string(),
        event_type: event_type.to_string(),
        timestamp,
        payload,
        version: version.to_string(),
    })
    .unwrap_or_else(|_| "{}".to_string());
    let _ = event_type;
    Event::default().id(id.to_string()).data(body)
}

fn decode_activity_event(msg: &str) -> (String, String, serde_json::Value) {
    match serde_json::from_str::<serde_json::Value>(msg) {
        Ok(v) => {
            let event_type = v
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("message")
                .to_string();
            let timestamp = v
                .get("timestamp")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(now_rfc3339);
            let payload = v
                .get("payload")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            (event_type, timestamp, payload)
        }
        Err(_) => (
            "message".to_string(),
            now_rfc3339(),
            serde_json::json!({ "text": msg }),
        ),
    }
}

fn parse_last_event_id(headers: &HeaderMap) -> Option<u64> {
    headers
        .get("last-event-id")
        .or_else(|| headers.get("Last-Event-ID"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
}

fn stream_client_id(headers: &HeaderMap) -> String {
    if let Some(v) = headers
        .get("x-client-id")
        .or_else(|| headers.get("x-request-id"))
        .or_else(|| headers.get("x-real-ip"))
        .or_else(|| headers.get("x-forwarded-for"))
    {
        if let Ok(s) = v.to_str() {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    "unknown".to_string()
}

#[derive(Debug)]
struct ClientStreamGuard {
    client_id: String,
    active_streams: Arc<std::sync::Mutex<HashMap<String, usize>>>,
}

impl Drop for ClientStreamGuard {
    fn drop(&mut self) {
        let mut guard = self
            .active_streams
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(count) = guard.get_mut(&self.client_id) {
            if *count <= 1 {
                guard.remove(&self.client_id);
            } else {
                *count -= 1;
            }
        }
    }
}

fn try_acquire_client_stream(
    active_streams: Arc<std::sync::Mutex<HashMap<String, usize>>>,
    client_id: &str,
    max_streams_per_client: usize,
) -> Result<ClientStreamGuard, String> {
    let mut guard = active_streams.lock().unwrap_or_else(|e| e.into_inner());
    let count = guard.entry(client_id.to_string()).or_insert(0);
    if *count >= max_streams_per_client {
        return Err(format!(
            "client stream concurrency exceeded: max={} active={}",
            max_streams_per_client, count
        ));
    }
    *count += 1;
    drop(guard);
    Ok(ClientStreamGuard {
        client_id: client_id.to_string(),
        active_streams,
    })
}

fn enforce_stream_establish_rate_limit(
    stream_establishes: &Arc<
        std::sync::Mutex<HashMap<String, std::collections::VecDeque<Instant>>>,
    >,
    client_id: &str,
    max_establish_per_minute: usize,
) -> bool {
    let now = Instant::now();
    let window_start = now.checked_sub(Duration::from_secs(60)).unwrap_or(now);
    let mut guard = stream_establishes.lock().unwrap_or_else(|e| e.into_inner());
    let history = guard
        .entry(client_id.to_string())
        .or_insert_with(std::collections::VecDeque::new);
    while let Some(t) = history.front() {
        if *t < window_start {
            let _ = history.pop_front();
        } else {
            break;
        }
    }
    if history.len() >= max_establish_per_minute {
        return false;
    }
    history.push_back(now);
    true
}

#[derive(Clone, Debug)]
struct TruncatedChunk {
    text: String,
    truncated: bool,
    original_bytes: usize,
}

fn truncate_chunk_text(msg: &str, max_chunk_bytes: usize) -> TruncatedChunk {
    let original_bytes = msg.len();
    if original_bytes <= max_chunk_bytes {
        return TruncatedChunk {
            text: msg.to_string(),
            truncated: false,
            original_bytes,
        };
    }
    let mut cut = max_chunk_bytes.min(original_bytes);
    while cut > 0 && !msg.is_char_boundary(cut) {
        cut -= 1;
    }
    TruncatedChunk {
        text: msg[..cut].to_string(),
        truncated: true,
        original_bytes,
    }
}

fn headers_size_bytes(headers: &HeaderMap) -> usize {
    headers
        .iter()
        .map(|(name, value)| name.as_str().len().saturating_add(value.as_bytes().len()))
        .sum()
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))?
        .to_str()
        .ok()?
        .trim();
    let (scheme, token) = raw.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let token = token.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn stream_auth_ok(cfg: &IdeSsePhase0Config, headers: &HeaderMap, query: &SseResumeQuery) -> bool {
    let Some(expected) = cfg.sse_auth_token.as_ref() else {
        return true;
    };
    if let Some(tok) = extract_bearer_token(headers) {
        return tok == *expected;
    }
    if let Some(tok) = query.access_token.as_ref() {
        return tok == expected;
    }
    false
}

#[derive(Clone, Copy, Debug)]
struct StreamRecvErrorMeta {
    code: &'static str,
    recoverable: bool,
    dropped_count: u64,
    should_close: bool,
}

fn classify_stream_recv_error(
    err: &tokio::sync::broadcast::error::RecvError,
) -> StreamRecvErrorMeta {
    match err {
        tokio::sync::broadcast::error::RecvError::Lagged(n) => StreamRecvErrorMeta {
            code: "channel_lagged",
            recoverable: true,
            dropped_count: *n,
            should_close: false,
        },
        tokio::sync::broadcast::error::RecvError::Closed => StreamRecvErrorMeta {
            code: "channel_closed",
            recoverable: false,
            dropped_count: 0,
            should_close: true,
        },
    }
}

fn replay_gap_payload(
    history: &std::collections::VecDeque<ReplayRecord>,
    resume_after: Option<u64>,
) -> Option<(u64, serde_json::Value)> {
    let after = resume_after?;
    let oldest = history.front()?.seq;
    let requested_from = after.saturating_add(1);
    if oldest > requested_from {
        let gap_id = requested_from;
        Some((
            gap_id,
            serde_json::json!({
                "reason": "replay_window_exceeded",
                "requested_from": requested_from,
                "available_from": oldest,
                "dropped_count": oldest.saturating_sub(requested_from),
                "recoverable": true
            }),
        ))
    } else {
        None
    }
}

fn append_replay_record(
    endpoint: crate::observability::IdeSseEndpoint,
    history: &std::sync::Mutex<std::collections::VecDeque<ReplayRecord>>,
    cap: usize,
    seq: u64,
    msg: String,
) {
    let mut guard = history.lock().unwrap_or_else(|e| e.into_inner());
    guard.push_back(ReplayRecord { seq, msg });
    let mut evicted = 0u64;
    while guard.len() > cap {
        let _ = guard.pop_front();
        evicted = evicted.saturating_add(1);
    }
    if evicted > 0 {
        crate::observability::ide_sse_replay_evictions(endpoint, evicted);
    }
}

fn spawn_events_replay_collector(
    tx: broadcast::Sender<String>,
    history: Arc<std::sync::Mutex<std::collections::VecDeque<ReplayRecord>>>,
    cap: usize,
) {
    if cap == 0 {
        return;
    }
    std::thread::spawn(move || {
        let mut rx = tx.subscribe();
        let mut seq = 1u64;
        loop {
            match rx.blocking_recv() {
                Ok(msg) => {
                    append_replay_record(
                        crate::observability::IdeSseEndpoint::Events,
                        &history,
                        cap,
                        seq,
                        msg,
                    );
                    seq = seq.saturating_add(1);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn spawn_job_replay_collector(
    tx: broadcast::Sender<String>,
    history: Arc<std::sync::Mutex<std::collections::VecDeque<ReplayRecord>>>,
    cap: usize,
) {
    if cap == 0 {
        return;
    }
    tokio::spawn(async move {
        let mut rx = tx.subscribe();
        let mut seq = 1u64;
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    append_replay_record(
                        crate::observability::IdeSseEndpoint::Run,
                        &history,
                        cap,
                        seq,
                        msg.clone(),
                    );
                    seq = seq.saturating_add(1);
                    if msg == "[DONE]" {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn spawn_job_terminal_and_cleanup_watcher(
    job_id: String,
    tx: broadcast::Sender<String>,
    jobs: Arc<RwLock<std::collections::HashMap<String, JobEntry>>>,
    retention_secs: u64,
    events_tx: broadcast::Sender<String>,
) {
    tokio::spawn(async move {
        let mut rx = tx.subscribe();
        let mut terminal = "done".to_string();
        let mut error_code: Option<i64> = None;
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if msg == "[CANCELLED]" {
                        terminal = "cancelled".to_string();
                        break;
                    }
                    if let Some(code_str) = msg
                        .strip_prefix("[ERROR:")
                        .and_then(|s| s.strip_suffix(']'))
                    {
                        terminal = "error".to_string();
                        error_code = code_str.trim().parse::<i64>().ok();
                        break;
                    }
                    if msg == "[DONE]" {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }

        emit_activity(
            &events_tx,
            "run_stream_terminal",
            serde_json::json!({
                "job_id": job_id.clone(),
                "terminal": terminal,
                "exit_code": error_code
            }),
        );

        tokio::time::sleep(Duration::from_secs(retention_secs.max(1))).await;
        let _ = jobs.write().await.remove(&job_id);
    });
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

/// Query params for SSE resume.
#[derive(Debug, Deserialize, Default)]
struct SseResumeQuery {
    #[serde(default)]
    last_event_id: Option<u64>,
    #[serde(default)]
    access_token: Option<String>,
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
        match path_under_base(root, subpath) {
            Some(d) => d,
            None => return Vec::new(),
        }
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
            let ws = ws.trim();
            let p = PathBuf::from(ws);
            if p.is_absolute() {
                if let (Ok(canon_root), Ok(canon_p)) =
                    (state.workspace_root.canonicalize(), p.canonicalize())
                {
                    if canon_p.starts_with(&canon_root) {
                        canon_p
                    } else {
                        state.workspace_root.clone()
                    }
                } else {
                    state.workspace_root.clone()
                }
            } else {
                path_under_base(&state.workspace_root, ws)
                    .unwrap_or_else(|| state.workspace_root.clone())
            }
        }
        _ => state.workspace_root.clone(),
    };

    if !path_is_within_root(&state.workspace_root, &root) || !root.exists() {
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
        Some(ws) if !ws.trim().is_empty() => {
            let ws = ws.trim();
            let p = PathBuf::from(ws);
            if p.is_absolute() {
                if let (Ok(canon_root), Ok(canon_p)) =
                    (state.workspace_root.canonicalize(), p.canonicalize())
                {
                    if canon_p.starts_with(&canon_root) {
                        canon_p
                    } else {
                        state.workspace_root.clone()
                    }
                } else {
                    state.workspace_root.clone()
                }
            } else {
                path_under_base(&state.workspace_root, ws)
                    .unwrap_or_else(|| state.workspace_root.clone())
            }
        }
        _ => state.workspace_root.clone(),
    };

    let subpath = query.path.as_deref().unwrap_or("");
    let dir = if subpath.is_empty() {
        root.clone()
    } else {
        match path_under_base(&root, subpath) {
            Some(d) => d,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Invalid path"})),
                )
                    .into_response();
            }
        }
    };

    if !path_is_within_root(&root, &dir) || !dir.exists() || !dir.is_dir() {
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
        .filter(|s| !s.trim().is_empty())
        .and_then(|ws| {
            let ws = ws.trim();
            let p = PathBuf::from(ws);
            if p.is_absolute() {
                let (cr, cp) = (
                    state.workspace_root.canonicalize().ok()?,
                    p.canonicalize().ok()?,
                );
                if cp.starts_with(&cr) {
                    Some(cp)
                } else {
                    None
                }
            } else {
                path_under_base(&state.workspace_root, ws)
            }
        })
        .unwrap_or_else(|| state.workspace_root.clone());
    if !path_is_within_root(&state.workspace_root, &root) || !root.exists() {
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
            state.jobs.write().await.insert(
                job_id.clone(),
                JobEntry {
                    output_tx: output_tx.clone(),
                    kill_tx: Arc::new(std::sync::Mutex::new(Some(kill_tx))),
                    replay_history: Arc::new(std::sync::Mutex::new(
                        std::collections::VecDeque::new(),
                    )),
                },
            );
            if state.sse_phase0.replay_enabled {
                let history = {
                    let jobs = state.jobs.read().await;
                    jobs.get(&job_id).map(|j| j.replay_history.clone())
                };
                if let Some(history) = history {
                    spawn_job_replay_collector(
                        output_tx.clone(),
                        history,
                        state.sse_phase0.replay_capacity,
                    );
                }
            }
            spawn_job_terminal_and_cleanup_watcher(
                job_id.clone(),
                output_tx.clone(),
                state.jobs.clone(),
                state.sse_phase0.job_retention_secs,
                state.events_tx.clone(),
            );
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
    Query(query): Query<SseResumeQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let stream_id = uuid::Uuid::new_v4().to_string();
    let client_id = stream_client_id(&headers);
    let resume_after = query
        .last_event_id
        .or_else(|| parse_last_event_id(&headers));
    if headers_size_bytes(&headers) > state.sse_phase0.max_header_bytes {
        return (
            StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            Json(serde_json::json!({"error": "request headers too large"})),
        )
            .into_response();
    }
    if !stream_auth_ok(&state.sse_phase0, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "stream authentication required (set Authorization: Bearer <token> or access_token query)"
            })),
        )
            .into_response();
    }
    if !enforce_stream_establish_rate_limit(
        &state.client_stream_establishes,
        &client_id,
        state.sse_phase0.max_establish_per_minute,
    ) {
        tracing::warn!(
            target: "dal_stream",
            endpoint = "run",
            stream_id = %stream_id,
            client_id = %client_id,
            phase = "rate_limit",
            max_establish_per_minute = state.sse_phase0.max_establish_per_minute,
            "sse_stream_establish_rejected"
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "stream establish rate limit exceeded",
                "client_id": client_id
            })),
        )
            .into_response();
    }
    let client_guard = match try_acquire_client_stream(
        state.client_active_streams.clone(),
        &client_id,
        state.sse_phase0.max_streams_per_client,
    ) {
        Ok(guard) => guard,
        Err(e) => {
            tracing::warn!(
                target: "dal_stream",
                endpoint = "run",
                stream_id = %stream_id,
                client_id = %client_id,
                phase = "concurrency_limit",
                max_streams_per_client = state.sse_phase0.max_streams_per_client,
                error = %e,
                "sse_stream_establish_rejected"
            );
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": e,
                    "client_id": client_id
                })),
            )
                .into_response();
        }
    };
    let (rx, replay_history) = {
        let jobs = state.jobs.read().await;
        match jobs.get(&job_id) {
            Some(entry) => (entry.output_tx.subscribe(), entry.replay_history.clone()),
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
        Receiving {
            rx: broadcast::Receiver<String>,
            next_id: u64,
        },
        Done,
    }
    let cfg = state.sse_phase0.clone();
    let replay_queue = if cfg.structured_envelope && cfg.replay_enabled {
        let after = resume_after.unwrap_or(0);
        let guard = replay_history.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .iter()
            .filter(|r| r.seq > after)
            .cloned()
            .collect::<std::collections::VecDeque<_>>()
    } else {
        std::collections::VecDeque::new()
    };
    let replay_gap = if cfg.structured_envelope && cfg.replay_enabled {
        let guard = replay_history.lock().unwrap_or_else(|e| e.into_inner());
        replay_gap_payload(&guard, resume_after)
    } else {
        None
    };
    let active_guard =
        crate::observability::ide_sse_stream_open(crate::observability::IdeSseEndpoint::Run);
    let stream_span = tracing::info_span!(
        target: "dal_stream",
        "sse_stream_lifecycle",
        endpoint = "run",
        stream_id = %stream_id,
        client_id = %client_id,
        job_id = %job_id
    );
    let events_tx = state.events_tx.clone();
    let stream_job_id = job_id.clone();
    tracing::info!(
        target: "dal_stream",
        endpoint = "run",
        stream_id = %stream_id,
        client_id = %client_id,
        job_id = %job_id,
        resume_after = ?resume_after,
        "sse_stream_open"
    );
    tracing::info!(parent: &stream_span, phase = "open", resume_after = ?resume_after, "sse_stream_lifecycle");
    emit_activity(
        &events_tx,
        "run_stream_connected",
        serde_json::json!({
            "job_id": job_id.clone(),
            "structured": cfg.structured_envelope,
            "resume_after": resume_after
        }),
    );
    if let Some(after) = resume_after {
        crate::observability::ide_sse_resume(crate::observability::IdeSseEndpoint::Run);
        tracing::info!(parent: &stream_span, phase = "resume", last_event_id = after, "sse_stream_lifecycle");
        emit_activity(
            &events_tx,
            "run_stream_resumed",
            serde_json::json!({
                "job_id": job_id.clone(),
                "last_event_id": after
            }),
        );
    }
    if let Some((_gap_id, payload)) = replay_gap.clone() {
        crate::observability::ide_sse_gap(crate::observability::IdeSseEndpoint::Run);
        tracing::warn!(parent: &stream_span, phase = "replay_gap", code = "replay_window_exceeded", "sse_stream_lifecycle");
        emit_activity(
            &events_tx,
            "run_stream_gap",
            serde_json::json!({
                "job_id": job_id.clone(),
                "gap": payload
            }),
        );
    }
    let stream = futures_util::stream::unfold(
        (
            StreamState::Receiving { rx, next_id: 1 },
            replay_queue,
            resume_after.unwrap_or(0).saturating_add(1),
            replay_gap,
            active_guard,
            client_guard,
            Instant::now(),
        ),
        move |state| {
            let cfg = cfg.clone();
            let stream_job_id = stream_job_id.clone();
            let stream_id = stream_id.clone();
            let client_id = client_id.clone();
            let stream_span = stream_span.clone();
            async move {
                let (
                    stream_state,
                    mut replay_queue,
                    mut next_id_hint,
                    mut replay_gap,
                    active_guard,
                    client_guard,
                    opened_at,
                ) = state;
                if opened_at.elapsed().as_secs() >= cfg.max_stream_lifetime_secs {
                    tracing::info!(
                        parent: &stream_span,
                        phase = "close",
                        reason = "max_stream_lifetime_exceeded",
                        "sse_stream_lifecycle"
                    );
                    let _ = active_guard;
                    let _ = client_guard;
                    return None;
                }
                if let Some((gap_id, payload)) = replay_gap.take() {
                    let event =
                        to_structured_event(gap_id, "gap", now_rfc3339(), payload, &cfg.version);
                    next_id_hint = next_id_hint.max(gap_id.saturating_add(1));
                    return Some((
                        Ok::<_, std::convert::Infallible>(event),
                        (
                            stream_state,
                            replay_queue,
                            next_id_hint,
                            replay_gap,
                            active_guard,
                            client_guard,
                            opened_at,
                        ),
                    ));
                }
                if let Some(record) = replay_queue.pop_front() {
                    let msg = record.msg.clone();
                    let event = if cfg.structured_envelope {
                        if let Some((terminal, exit_code)) = parse_stream_terminal_marker(&msg) {
                            to_structured_event(
                                record.seq,
                                terminal,
                                now_rfc3339(),
                                serde_json::json!({ "terminal": terminal, "exit_code": exit_code }),
                                &cfg.version,
                            )
                        } else {
                            let limited = truncate_chunk_text(&msg, cfg.max_chunk_bytes);
                            to_structured_event(
                                record.seq,
                                "chunk",
                                now_rfc3339(),
                                serde_json::json!({
                                    "text": limited.text,
                                    "truncated": limited.truncated,
                                    "original_bytes": limited.original_bytes
                                }),
                                &cfg.version,
                            )
                        }
                    } else {
                        if parse_stream_terminal_marker(&msg).is_some() {
                            Event::default().data(msg)
                        } else {
                            let limited = truncate_chunk_text(&msg, cfg.max_chunk_bytes);
                            if limited.truncated {
                                Event::default().data(format!(
                                    "{}\n[TRUNCATED original_bytes={} emitted_bytes={}]",
                                    limited.text,
                                    limited.original_bytes,
                                    limited.text.len()
                                ))
                            } else {
                                Event::default().data(limited.text)
                            }
                        }
                    };
                    next_id_hint = record.seq.saturating_add(1);
                    return Some((
                        Ok::<_, std::convert::Infallible>(event),
                        (
                            stream_state,
                            replay_queue,
                            next_id_hint,
                            replay_gap,
                            active_guard,
                            client_guard,
                            opened_at,
                        ),
                    ));
                }

                match stream_state {
                    StreamState::Done => {
                        tracing::info!(
                            parent: &stream_span,
                            phase = "close",
                            reason = "terminal_done",
                            "sse_stream_lifecycle"
                        );
                        let _ = active_guard;
                        let _ = client_guard;
                        None
                    }
                    StreamState::Receiving { rx, next_id } => {
                        let mut rx = rx;
                        loop {
                            let recv = tokio::time::timeout(
                                Duration::from_secs(cfg.idle_timeout_secs),
                                rx.recv(),
                            )
                            .await;
                            match recv {
                                Err(_) => {
                                    tracing::info!(
                                        parent: &stream_span,
                                        phase = "close",
                                        reason = "idle_timeout_exceeded",
                                        idle_timeout_secs = cfg.idle_timeout_secs,
                                        "sse_stream_lifecycle"
                                    );
                                    let _ = active_guard;
                                    let _ = client_guard;
                                    return None;
                                }
                                Ok(Ok(msg)) => {
                                    let effective_id = next_id.max(next_id_hint);
                                    let (event, next_id_after) = if cfg.structured_envelope {
                                        if let Some((terminal, exit_code)) =
                                            parse_stream_terminal_marker(&msg)
                                        {
                                            tracing::info!(
                                                parent: &stream_span,
                                                phase = "terminal",
                                                terminal = terminal,
                                                exit_code = exit_code.unwrap_or_default(),
                                                "sse_stream_lifecycle"
                                            );
                                            (
                                                to_structured_event(
                                                    effective_id,
                                                    terminal,
                                                    now_rfc3339(),
                                                    serde_json::json!({
                                                        "terminal": terminal,
                                                        "exit_code": exit_code
                                                    }),
                                                    &cfg.version,
                                                ),
                                                effective_id.saturating_add(1),
                                            )
                                        } else {
                                            let limited =
                                                truncate_chunk_text(&msg, cfg.max_chunk_bytes);
                                            (
                                                to_structured_event(
                                                    effective_id,
                                                    "chunk",
                                                    now_rfc3339(),
                                                    serde_json::json!({
                                                        "text": limited.text,
                                                        "truncated": limited.truncated,
                                                        "original_bytes": limited.original_bytes
                                                    }),
                                                    &cfg.version,
                                                ),
                                                effective_id.saturating_add(1),
                                            )
                                        }
                                    } else {
                                        if parse_stream_terminal_marker(&msg).is_some() {
                                            (Event::default().data(msg.clone()), next_id)
                                        } else {
                                            let limited =
                                                truncate_chunk_text(&msg, cfg.max_chunk_bytes);
                                            let legacy_text = if limited.truncated {
                                                format!(
                                                    "{}\n[TRUNCATED original_bytes={} emitted_bytes={}]",
                                                    limited.text,
                                                    limited.original_bytes,
                                                    limited.text.len()
                                                )
                                            } else {
                                                limited.text
                                            };
                                            (Event::default().data(legacy_text), next_id)
                                        }
                                    };
                                    let next_stream =
                                        if parse_stream_terminal_marker(&msg).is_some() {
                                            StreamState::Done
                                        } else {
                                            StreamState::Receiving {
                                                rx,
                                                next_id: next_id_after,
                                            }
                                        };
                                    return Some((
                                        Ok::<_, std::convert::Infallible>(event),
                                        (
                                            next_stream,
                                            replay_queue,
                                            next_id_after,
                                            replay_gap,
                                            active_guard,
                                            client_guard,
                                            opened_at,
                                        ),
                                    ));
                                }
                                Ok(Err(err)) => {
                                    let meta = classify_stream_recv_error(&err);
                                    if meta.dropped_count > 0 {
                                        crate::observability::ide_sse_lagged(
                                            crate::observability::IdeSseEndpoint::Run,
                                            meta.dropped_count,
                                        );
                                    }
                                    tracing::warn!(
                                        parent: &stream_span,
                                        phase = "recv_error",
                                        error_code = meta.code,
                                        recoverable = meta.recoverable,
                                        dropped_count = meta.dropped_count,
                                        stream_id = %stream_id,
                                        client_id = %client_id,
                                        job_id = %stream_job_id,
                                        "sse_stream_recv_error"
                                    );
                                    if meta.should_close {
                                        crate::observability::ide_sse_recv_closed(
                                            crate::observability::IdeSseEndpoint::Run,
                                        );
                                        tracing::info!(
                                            parent: &stream_span,
                                            phase = "close",
                                            reason = meta.code,
                                            "sse_stream_lifecycle"
                                        );
                                        let _ = active_guard;
                                        let _ = client_guard;
                                        return None;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    );

    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(state.sse_phase0.keepalive_secs))
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
    let kill_handle = {
        let jobs = state.jobs.read().await;
        jobs.get(&req.job_id).map(|entry| entry.kill_tx.clone())
    };
    let killed = if let Some(kill_handle) = kill_handle {
        let mut guard = match kill_handle.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(kill_tx) = guard.take() {
            let _ = kill_tx.send(());
            true
        } else {
            false
        }
    } else {
        false
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
            let p = PathBuf::from(s);
            if p.is_absolute() {
                state
                    .workspace_root
                    .canonicalize()
                    .ok()
                    .and_then(|cr| p.canonicalize().ok().filter(|cp| cp.starts_with(&cr)))
            } else {
                path_under_base(&state.workspace_root, s)
            }
        }
    });
    let cwd = cwd.unwrap_or_else(|| {
        if cmd == "dal" && args.first().map(|a| a.as_str()) == Some("new") {
            projects_root()
        } else {
            state.workspace_root.clone()
        }
    });
    let cwd_path = if path_is_within_root(&state.workspace_root, &cwd) && cwd.exists() {
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

/// POST /api/agent/run_command_stream — run command in background, stream output (does not block).
/// Use GET /api/run/stream/:job_id for output. Long-lived servers (e.g. dal serve) will not block the IDE.
async fn post_agent_run_command_stream(
    State(state): State<AppState>,
    Json(req): Json<RunCommandRequest>,
) -> impl IntoResponse {
    let cmd_string = req.cmd.clone();
    let (cmd, args) = if req.args.is_empty() {
        let parts: Vec<&str> = req.cmd.split_whitespace().collect();
        if parts.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Empty command"})),
            )
                .into_response();
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
            let p = PathBuf::from(s);
            if p.is_absolute() {
                state
                    .workspace_root
                    .canonicalize()
                    .ok()
                    .and_then(|cr| p.canonicalize().ok().filter(|cp| cp.starts_with(&cr)))
            } else {
                path_under_base(&state.workspace_root, s)
            }
        }
    });
    let cwd = cwd.unwrap_or_else(|| state.workspace_root.clone());
    let cwd_path = if path_is_within_root(&state.workspace_root, &cwd) && cwd.exists() {
        Some(cwd.as_path())
    } else {
        None
    };

    let cmd_resolved = if cmd == "dal" {
        super::run_backend::dal_binary_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "dal".to_string())
    } else {
        cmd
    };

    match spawn_run_streaming(&cmd_resolved, &args, cwd_path) {
        Ok((output_tx, kill_tx)) => {
            let job_id = format!(
                "run-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
            state.jobs.write().await.insert(
                job_id.clone(),
                JobEntry {
                    output_tx: output_tx.clone(),
                    kill_tx: Arc::new(std::sync::Mutex::new(Some(kill_tx))),
                    replay_history: Arc::new(std::sync::Mutex::new(
                        std::collections::VecDeque::new(),
                    )),
                },
            );
            if state.sse_phase0.replay_enabled {
                let history = {
                    let jobs = state.jobs.read().await;
                    jobs.get(&job_id).map(|j| j.replay_history.clone())
                };
                if let Some(history) = history {
                    spawn_job_replay_collector(
                        output_tx.clone(),
                        history,
                        state.sse_phase0.replay_capacity,
                    );
                }
            }
            spawn_job_terminal_and_cleanup_watcher(
                job_id.clone(),
                output_tx.clone(),
                state.jobs.clone(),
                state.sse_phase0.job_retention_secs,
                state.events_tx.clone(),
            );
            emit_activity(
                &state.events_tx,
                "run_started",
                serde_json::json!({ "job_id": job_id, "cmd": cmd_string }),
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "job_id": job_id,
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
        let full_path = match resolve_path_under_root(&root, path) {
            Some(p) => p,
            None => {
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({ "diagnostics": [] })),
                )
                    .into_response();
            }
        };
        let cargo_root = find_cargo_root(&full_path, &state.workspace_root);
        let uri = lsp_client::path_to_file_uri(&full_path);
        let contents = req.contents.clone();
        let state_clone = state.clone();
        if let Ok(Some(diags)) = tokio::task::spawn_blocking(move || {
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

/// Resolve workspace root from optional user input. Ensures the result is under
/// state.workspace_root to prevent path traversal (absolute paths are canonicalized and checked).
fn resolve_workspace_root(state: &AppState, workspace: Option<&str>) -> PathBuf {
    match workspace {
        Some(ws) if !ws.trim().is_empty() => {
            let ws = ws.trim();
            let p = PathBuf::from(ws);
            if p.is_absolute() {
                if let (Ok(canon_root), Ok(canon_p)) =
                    (state.workspace_root.canonicalize(), p.canonicalize())
                {
                    if canon_p.starts_with(&canon_root) {
                        return canon_p;
                    }
                }
                state.workspace_root.clone()
            } else {
                path_under_base(&state.workspace_root, ws)
                    .unwrap_or_else(|| state.workspace_root.clone())
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
        .unwrap_or(file_path);
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if path_is_within_root(workspace_root, &cargo_toml) && cargo_toml.exists() {
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
        let full_path = match resolve_path_under_root(&root, path) {
            Some(p) => p,
            None => {
                return (StatusCode::OK, Json(serde_json::json!({ "contents": [] })))
                    .into_response();
            }
        };
        let cargo_root = find_cargo_root(&full_path, &state.workspace_root);
        let uri = lsp_client::path_to_file_uri(&full_path);
        let contents = req.contents.clone();
        let line = req.line;
        let character = req.character;
        let state_clone = state.clone();
        if let Ok(Some(hover_content)) = tokio::task::spawn_blocking(move || {
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
            return (
                StatusCode::OK,
                Json(serde_json::json!({ "contents": hover_content })),
            )
                .into_response();
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
        let full_path = match resolve_path_under_root(&root, path) {
            Some(p) => p,
            None => {
                return (StatusCode::OK, Json(serde_json::json!({ "items": [] }))).into_response();
            }
        };
        let cargo_root = find_cargo_root(&full_path, &state.workspace_root);
        let uri = lsp_client::path_to_file_uri(&full_path);
        let contents = req.contents.clone();
        let line = req.line;
        let character = req.character;
        let state_clone = state.clone();
        if let Ok(Some(items)) = tokio::task::spawn_blocking(move || {
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

#[derive(Debug, Deserialize)]
struct FileAssetQuery {
    path: String,
    #[serde(default)]
    workspace: Option<String>,
}

/// GET /api/agent/file_asset — read file bytes for browser previews (e.g. images).
async fn get_agent_file_asset(
    State(state): State<AppState>,
    Query(query): Query<FileAssetQuery>,
) -> impl IntoResponse {
    let path_str = query.path.trim();
    if path_str.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "path is required"})),
        )
            .into_response();
    }
    let root = match &query.workspace {
        Some(ws) if !ws.trim().is_empty() => {
            let ws = ws.trim();
            let p = PathBuf::from(ws);
            if p.is_absolute() {
                if let (Ok(cr), Ok(cp)) = (state.workspace_root.canonicalize(), p.canonicalize()) {
                    if cp.starts_with(&cr) {
                        cp
                    } else {
                        state.workspace_root.clone()
                    }
                } else {
                    state.workspace_root.clone()
                }
            } else {
                path_under_base(&state.workspace_root, ws)
                    .unwrap_or_else(|| state.workspace_root.clone())
            }
        }
        _ => state.workspace_root.clone(),
    };
    let path = match resolve_path_under_root(&root, path_str) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid path"})),
            )
                .into_response();
        }
    };

    if !path_is_within_root(&root, &path) || !path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "file not found", "path": path.to_string_lossy()})),
        )
            .into_response();
    }
    if !path_is_within_root(&root, &path) || path.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({"error": "path is a directory", "path": path.to_string_lossy()}),
            ),
        )
            .into_response();
    }

    if !path_is_within_root(&root, &path) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid path"})),
        )
            .into_response();
    }
    match std::fs::read(&path) {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static(image_mime_for_path(&path)),
            );
            (StatusCode::OK, headers, bytes).into_response()
        }
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
        Some(ws) if !ws.trim().is_empty() => {
            let ws = ws.trim();
            let p = PathBuf::from(ws);
            if p.is_absolute() {
                if let (Ok(cr), Ok(cp)) = (state.workspace_root.canonicalize(), p.canonicalize()) {
                    if cp.starts_with(&cr) {
                        cp
                    } else {
                        state.workspace_root.clone()
                    }
                } else {
                    state.workspace_root.clone()
                }
            } else {
                path_under_base(&state.workspace_root, ws)
                    .unwrap_or_else(|| state.workspace_root.clone())
            }
        }
        _ => state.workspace_root.clone(),
    };
    let path = match resolve_path_under_root(&root, path_str) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid path"})),
            )
                .into_response();
        }
    };

    if !path_is_within_root(&root, &path) || !path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "file not found", "path": path.to_string_lossy()})),
        )
            .into_response();
    }
    if !path_is_within_root(&root, &path) || path.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({"error": "path is a directory", "path": path.to_string_lossy()}),
            ),
        )
            .into_response();
    }

    if !path_is_within_root(&root, &path) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid path"})),
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
    let root = match &req.workspace {
        Some(ws) if !ws.trim().is_empty() => {
            let ws = ws.trim();
            let p = PathBuf::from(ws);
            if p.is_absolute() {
                if let (Ok(cr), Ok(cp)) = (state.workspace_root.canonicalize(), p.canonicalize()) {
                    if cp.starts_with(&cr) {
                        cp
                    } else {
                        state.workspace_root.clone()
                    }
                } else {
                    state.workspace_root.clone()
                }
            } else {
                path_under_base(&state.workspace_root, ws)
                    .unwrap_or_else(|| state.workspace_root.clone())
            }
        }
        _ => state.workspace_root.clone(),
    };
    let path = match resolve_path_under_root(&root, &req.path) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid path"})),
            )
                .into_response();
        }
    };

    if !path_is_within_root(&root, &path) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid path"})),
        )
            .into_response();
    }
    if let Some(parent) = path.parent() {
        if !path_is_within_root(&root, parent) {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid path"})),
            )
                .into_response();
        }
        if let Err(e) = std::fs::create_dir_all(parent) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to create parent dir: {}", e)})),
            )
                .into_response();
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
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/agent/prompt — run DAL agent on prompt; returns job_id and streams activity (agent_started, agent_step, completion_summary).
#[derive(Debug, Deserialize)]
struct AgentPromptRequest {
    text: String,
    #[serde(default)]
    context: Option<String>,
    /// Optional workspace path (relative to server workspace or absolute). Defaults to server workspace.
    #[serde(default)]
    workspace: Option<String>,
}

async fn post_agent_prompt(
    State(state): State<AppState>,
    Json(req): Json<AgentPromptRequest>,
) -> impl IntoResponse {
    let text = req.text.trim().to_string();
    if text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Prompt text is required"})),
        )
            .into_response();
    }
    let workspace_root = match req.workspace.as_deref() {
        Some(ws) if !ws.trim().is_empty() => {
            let ws = ws.trim();
            let p = PathBuf::from(ws);
            if p.is_absolute() {
                if let (Ok(cr), Ok(cp)) = (state.workspace_root.canonicalize(), p.canonicalize()) {
                    if cp.starts_with(&cr) {
                        cp
                    } else {
                        state.workspace_root.clone()
                    }
                } else {
                    state.workspace_root.clone()
                }
            } else {
                path_under_base(&state.workspace_root, ws)
                    .unwrap_or_else(|| state.workspace_root.clone())
            }
        }
        _ => state.workspace_root.clone(),
    };
    let job_id = uuid::Uuid::new_v4().to_string();
    let job_id_response = job_id.clone();
    let context = req.context.clone();
    let events_tx = state.events_tx.clone();
    tokio::task::spawn_blocking(move || {
        let _ =
            agent_runner::run_agent_prompt_sync(&workspace_root, text, context, job_id, events_tx);
    });
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "job_id": job_id_response
        })),
    )
        .into_response()
}

/// POST /api/agent/chat — single-turn chat using DAL's LLM (same as `dal agent chat`). Returns reply or error.
#[derive(Debug, Deserialize)]
struct AgentChatRequest {
    text: String,
}

async fn post_agent_chat(
    State(state): State<AppState>,
    Json(req): Json<AgentChatRequest>,
) -> impl IntoResponse {
    let text = req.text.trim().to_string();
    if text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "text is required"})),
        )
            .into_response();
    }
    let events_tx = state.events_tx.clone();
    emit_activity(&events_tx, "command", serde_json::json!({ "text": text }));
    let result = tokio::task::spawn_blocking(move || crate::stdlib::ai::generate_text(text)).await;
    match result {
        Ok(Ok(reply)) => {
            let reply_trimmed = reply.trim().to_string();
            emit_activity(
                &events_tx,
                "chat_reply",
                serde_json::json!({ "reply": reply_trimmed }),
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({ "reply": reply_trimmed })),
            )
                .into_response()
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
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
async fn get_events_stream(
    State(state): State<AppState>,
    Query(query): Query<SseResumeQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let stream_id = uuid::Uuid::new_v4().to_string();
    let client_id = stream_client_id(&headers);
    let resume_after = query
        .last_event_id
        .or_else(|| parse_last_event_id(&headers));
    if headers_size_bytes(&headers) > state.sse_phase0.max_header_bytes {
        return (
            StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            Json(serde_json::json!({"error": "request headers too large"})),
        )
            .into_response();
    }
    if !stream_auth_ok(&state.sse_phase0, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "stream authentication required (set Authorization: Bearer <token> or access_token query)"
            })),
        )
            .into_response();
    }
    if !enforce_stream_establish_rate_limit(
        &state.client_stream_establishes,
        &client_id,
        state.sse_phase0.max_establish_per_minute,
    ) {
        tracing::warn!(
            target: "dal_stream",
            endpoint = "events",
            stream_id = %stream_id,
            client_id = %client_id,
            phase = "rate_limit",
            max_establish_per_minute = state.sse_phase0.max_establish_per_minute,
            "sse_stream_establish_rejected"
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "stream establish rate limit exceeded",
                "client_id": client_id
            })),
        )
            .into_response();
    }
    let client_guard = match try_acquire_client_stream(
        state.client_active_streams.clone(),
        &client_id,
        state.sse_phase0.max_streams_per_client,
    ) {
        Ok(guard) => guard,
        Err(e) => {
            tracing::warn!(
                target: "dal_stream",
                endpoint = "events",
                stream_id = %stream_id,
                client_id = %client_id,
                phase = "concurrency_limit",
                max_streams_per_client = state.sse_phase0.max_streams_per_client,
                error = %e,
                "sse_stream_establish_rejected"
            );
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": e,
                    "client_id": client_id
                })),
            )
                .into_response();
        }
    };
    let rx = state.events_tx.subscribe();
    let replay_records = if state.sse_phase0.structured_envelope && state.sse_phase0.replay_enabled
    {
        let after = resume_after.unwrap_or(0);
        let guard = state
            .events_replay_history
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        guard
            .iter()
            .filter(|r| r.seq > after)
            .cloned()
            .collect::<std::collections::VecDeque<_>>()
    } else {
        std::collections::VecDeque::new()
    };
    let replay_gap = if state.sse_phase0.structured_envelope && state.sse_phase0.replay_enabled {
        let guard = state
            .events_replay_history
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        replay_gap_payload(&guard, resume_after)
    } else {
        None
    };
    let active_guard =
        crate::observability::ide_sse_stream_open(crate::observability::IdeSseEndpoint::Events);
    let stream_span = tracing::info_span!(
        target: "dal_stream",
        "sse_stream_lifecycle",
        endpoint = "events",
        stream_id = %stream_id,
        client_id = %client_id
    );
    tracing::info!(
        target: "dal_stream",
        endpoint = "events",
        stream_id = %stream_id,
        client_id = %client_id,
        resume_after = ?resume_after,
        "sse_stream_open"
    );
    tracing::info!(parent: &stream_span, phase = "open", resume_after = ?resume_after, "sse_stream_lifecycle");
    if resume_after.is_some() {
        crate::observability::ide_sse_resume(crate::observability::IdeSseEndpoint::Events);
        tracing::info!(parent: &stream_span, phase = "resume", "sse_stream_lifecycle");
    }
    if replay_gap.is_some() {
        crate::observability::ide_sse_gap(crate::observability::IdeSseEndpoint::Events);
        tracing::warn!(parent: &stream_span, phase = "replay_gap", code = "replay_window_exceeded", "sse_stream_lifecycle");
    }
    let cfg = state.sse_phase0.clone();
    let start_id = resume_after.unwrap_or(0).saturating_add(1);
    let stream = futures_util::stream::unfold(
        (
            rx,
            start_id,
            replay_records,
            replay_gap,
            active_guard,
            client_guard,
            Instant::now(),
        ),
        move |state| {
            let cfg = cfg.clone();
            let stream_id = stream_id.clone();
            let client_id = client_id.clone();
            let stream_span = stream_span.clone();
            async move {
                let (
                    mut rx,
                    next_id,
                    mut replay_records,
                    mut replay_gap,
                    active_guard,
                    client_guard,
                    opened_at,
                ) = state;
                if opened_at.elapsed().as_secs() >= cfg.max_stream_lifetime_secs {
                    tracing::info!(
                        parent: &stream_span,
                        phase = "close",
                        reason = "max_stream_lifetime_exceeded",
                        "sse_stream_lifecycle"
                    );
                    let _ = active_guard;
                    let _ = client_guard;
                    return None;
                }
                if let Some((gap_id, payload)) = replay_gap.take() {
                    let event =
                        to_structured_event(gap_id, "gap", now_rfc3339(), payload, &cfg.version);
                    return Some((
                        Ok::<_, std::convert::Infallible>(event),
                        (
                            rx,
                            next_id.max(gap_id.saturating_add(1)),
                            replay_records,
                            replay_gap,
                            active_guard,
                            client_guard,
                            opened_at,
                        ),
                    ));
                }
                if let Some(record) = replay_records.pop_front() {
                    let event = if cfg.structured_envelope {
                        let (event_type, timestamp, payload) = decode_activity_event(&record.msg);
                        to_structured_event(
                            record.seq,
                            &event_type,
                            timestamp,
                            payload,
                            &cfg.version,
                        )
                    } else {
                        Event::default().data(record.msg)
                    };
                    return Some((
                        Ok::<_, std::convert::Infallible>(event),
                        (
                            rx,
                            record.seq.saturating_add(1),
                            replay_records,
                            replay_gap,
                            active_guard,
                            client_guard,
                            opened_at,
                        ),
                    ));
                }
                loop {
                    let recv =
                        tokio::time::timeout(Duration::from_secs(cfg.idle_timeout_secs), rx.recv())
                            .await;
                    match recv {
                        Err(_) => {
                            tracing::info!(
                                parent: &stream_span,
                                phase = "close",
                                reason = "idle_timeout_exceeded",
                                idle_timeout_secs = cfg.idle_timeout_secs,
                                "sse_stream_lifecycle"
                            );
                            let _ = active_guard;
                            let _ = client_guard;
                            return None;
                        }
                        Ok(Ok(msg)) => {
                            let event = if cfg.structured_envelope {
                                let (event_type, timestamp, payload) = decode_activity_event(&msg);
                                to_structured_event(
                                    next_id,
                                    &event_type,
                                    timestamp,
                                    payload,
                                    &cfg.version,
                                )
                            } else {
                                Event::default().data(msg)
                            };
                            let next = if cfg.structured_envelope {
                                next_id.saturating_add(1)
                            } else {
                                next_id
                            };
                            return Some((
                                Ok::<_, std::convert::Infallible>(event),
                                (
                                    rx,
                                    next,
                                    replay_records,
                                    replay_gap,
                                    active_guard,
                                    client_guard,
                                    opened_at,
                                ),
                            ));
                        }
                        Ok(Err(err)) => {
                            let meta = classify_stream_recv_error(&err);
                            if meta.dropped_count > 0 {
                                crate::observability::ide_sse_lagged(
                                    crate::observability::IdeSseEndpoint::Events,
                                    meta.dropped_count,
                                );
                            }
                            tracing::warn!(
                                parent: &stream_span,
                                phase = "recv_error",
                                error_code = meta.code,
                                recoverable = meta.recoverable,
                                dropped_count = meta.dropped_count,
                                stream_id = %stream_id,
                                client_id = %client_id,
                                "sse_stream_recv_error"
                            );
                            if meta.should_close {
                                crate::observability::ide_sse_recv_closed(
                                    crate::observability::IdeSseEndpoint::Events,
                                );
                                tracing::info!(
                                    parent: &stream_span,
                                    phase = "close",
                                    reason = meta.code,
                                    "sse_stream_lifecycle"
                                );
                                let _ = active_guard;
                                let _ = client_guard;
                                return None;
                            }
                        }
                    }
                }
            }
        },
    );
    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(state.sse_phase0.keepalive_secs))
                .text("keepalive"),
        )
        .into_response()
}

/// Build the IDE API router.
pub fn build_router(workspace_root: PathBuf) -> Router {
    let sse_phase0 = IdeSsePhase0Config::from_env();
    let sse_cfg = sse_phase0.clone();
    let (events_tx, _) = broadcast::channel(256);
    let events_replay_history = Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
    if sse_phase0.replay_enabled {
        spawn_events_replay_collector(
            events_tx.clone(),
            events_replay_history.clone(),
            sse_phase0.replay_capacity,
        );
    }
    let state = AppState {
        workspace_root: workspace_root.clone(),
        orchestration: Arc::new(RwLock::new(Some(discover_workspace(&workspace_root)))),
        jobs: Arc::new(RwLock::new(std::collections::HashMap::new())),
        events_tx,
        events_replay_history,
        client_active_streams: Arc::new(std::sync::Mutex::new(HashMap::new())),
        client_stream_establishes: Arc::new(std::sync::Mutex::new(HashMap::new())),
        sse_phase0,
        second_lsp_by_cargo_root: Arc::new(std::sync::Mutex::new(HashMap::new())),
    };

    let security_preset = std::env::var("DAL_SERVE_SECURITY_PRESET")
        .unwrap_or_else(|_| "legacy".to_string())
        .trim()
        .to_ascii_lowercase();
    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);
    if sse_cfg.cors_allow_any || security_preset == "legacy" {
        cors = cors.allow_origin(Any);
    } else if let Some(origin) = sse_cfg.cors_allow_origin.as_deref() {
        if let Ok(value) = HeaderValue::from_str(origin) {
            cors = cors.allow_origin(value);
        }
    }

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
        .route(
            "/api/agent/run_command_stream",
            post(post_agent_run_command_stream),
        )
        .route("/api/agent/file_asset", get(get_agent_file_asset))
        .route("/api/agent/write_file", post(post_agent_write_file))
        .route("/api/agent/read_file", post(post_agent_read_file))
        .route("/api/agent/prompt", post(post_agent_prompt))
        .route("/api/agent/chat", post(post_agent_chat))
        .route("/api/command", post(post_command))
        .route("/api/events/stream", get(get_events_stream))
        .route("/health", get(|| async { "OK" }))
        .route("/metrics", get(crate::observability::metrics_http_response))
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(middleware::from_fn(
            crate::observability::http_observability_middleware,
        ))
        .layer(RequestBodyLimitLayer::new(sse_cfg.max_body_bytes))
        .layer(cors)
        .with_state(state)
}

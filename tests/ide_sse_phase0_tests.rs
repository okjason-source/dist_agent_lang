use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt as _;
use serde_json::Value;
use serial_test::serial;
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
use tower::ServiceExt;

use dist_agent_lang::ide::server::build_router;

struct EnvGuard {
    key: &'static str,
    original: Option<String>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let original = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(v) = &self.original {
            std::env::set_var(self.key, v);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

fn setup_workspace() -> tempfile::TempDir {
    let workspace = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(workspace.path().join("src")).expect("create src dir");
    std::fs::write(workspace.path().join("src/main.dal"), "print(\"ok\")").expect("write file");
    workspace
}

async fn post_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("build request");

    let res = app.oneshot(req).await.expect("response");
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .expect("read body");
    let parsed = serde_json::from_slice::<Value>(&bytes).unwrap_or_else(|_| serde_json::json!({}));
    (status, parsed)
}

async fn read_sse_text(uri: &str, app: axum::Router, max_frames: usize, timeout_ms: u64) -> String {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("build request");
    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);

    let mut body = res.into_body();
    let mut out = String::new();
    for _ in 0..max_frames {
        let next = tokio::time::timeout(Duration::from_millis(timeout_ms), body.frame()).await;
        match next {
            Ok(Some(Ok(frame))) => {
                if let Ok(data) = frame.into_data() {
                    out.push_str(&String::from_utf8_lossy(&data));
                }
            }
            Ok(Some(Err(_))) | Ok(None) | Err(_) => break,
        }
    }
    out
}

fn parse_sse_envelopes(raw: &str) -> Vec<Value> {
    raw.lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .filter_map(|payload| serde_json::from_str::<Value>(payload).ok())
        .collect()
}

fn envelope_key_set(v: &Value) -> BTreeSet<String> {
    v.as_object()
        .map(|m| m.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default()
}

fn assert_sse_envelope_shape(v: &Value) {
    let keys = envelope_key_set(v);
    let expected = BTreeSet::from([
        "id".to_string(),
        "type".to_string(),
        "timestamp".to_string(),
        "payload".to_string(),
        "version".to_string(),
    ]);
    assert_eq!(keys, expected, "unexpected envelope shape: {:?}", v);
}

async fn get_status(app: axum::Router, uri: &str) -> StatusCode {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("build request");
    let res = app.oneshot(req).await.expect("response");
    res.status()
}

async fn get_status_with_headers(
    app: axum::Router,
    uri: &str,
    headers: &[(&str, &str)],
) -> StatusCode {
    let mut req = Request::builder().method("GET").uri(uri);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let req = req.body(Body::empty()).expect("build request");
    let res = app.oneshot(req).await.expect("response");
    res.status()
}

async fn get_response_with_headers(
    app: axum::Router,
    uri: &str,
    headers: &[(&str, &str)],
) -> axum::response::Response {
    let mut req = Request::builder().method("GET").uri(uri);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let req = req.body(Body::empty()).expect("build request");
    app.oneshot(req).await.expect("response")
}

#[tokio::test]
#[serial]
async fn run_stream_structured_resume_replays_after_last_event_id() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "64");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "printf hello"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id");

    let first = read_sse_text(&format!("/api/run/stream/{}", job_id), app.clone(), 24, 200).await;
    let envelopes = parse_sse_envelopes(&first);
    assert!(
        !envelopes.is_empty(),
        "expected structured envelopes: {}",
        first
    );

    let ids: Vec<u64> = envelopes
        .iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();
    assert!(!ids.is_empty(), "expected envelope ids");
    let first_id = ids[0];
    assert!(
        envelopes
            .iter()
            .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("done")),
        "expected done terminal event"
    );

    let resumed = read_sse_text(
        &format!("/api/run/stream/{}?last_event_id={}", job_id, first_id),
        app.clone(),
        24,
        200,
    )
    .await;
    let resumed_env = parse_sse_envelopes(&resumed);
    assert!(
        !resumed_env.is_empty(),
        "expected resumed envelopes after last_event_id: {}",
        resumed
    );
    let resumed_ids: Vec<u64> = resumed_env
        .iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();
    assert!(
        resumed_ids.iter().all(|id| *id > first_id),
        "expected resumed ids > {} got {:?}",
        first_id,
        resumed_ids
    );
}

#[tokio::test]
#[serial]
async fn events_stream_resume_uses_last_event_id_query() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "64");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (s1, _) = post_json(
        app.clone(),
        "/api/command",
        serde_json::json!({ "text": "first" }),
    )
    .await;
    assert_eq!(s1, StatusCode::OK);
    let (s2, _) = post_json(
        app.clone(),
        "/api/command",
        serde_json::json!({ "text": "second" }),
    )
    .await;
    assert_eq!(s2, StatusCode::OK);

    tokio::time::sleep(Duration::from_millis(120)).await;

    let first = read_sse_text("/api/events/stream", app.clone(), 16, 200).await;
    let env = parse_sse_envelopes(&first);
    assert!(!env.is_empty(), "expected replayed activity envelopes");
    let ids: Vec<u64> = env
        .iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();
    assert!(!ids.is_empty(), "expected ids in activity envelopes");

    let resume_after = ids[0];
    let resumed = read_sse_text(
        &format!("/api/events/stream?last_event_id={}", resume_after),
        app.clone(),
        16,
        200,
    )
    .await;
    let resumed_env = parse_sse_envelopes(&resumed);
    assert!(
        !resumed_env.is_empty(),
        "expected resumed activity envelopes: {}",
        resumed
    );
    let resumed_ids: Vec<u64> = resumed_env
        .iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();
    assert!(
        resumed_ids.iter().all(|id| *id > resume_after),
        "expected resumed ids > {} got {:?}",
        resume_after,
        resumed_ids
    );
}

#[tokio::test]
#[serial]
async fn events_stream_emits_gap_event_when_resume_window_exceeded() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "2");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    for i in 0..4 {
        let (status, _) = post_json(
            app.clone(),
            "/api/command",
            serde_json::json!({ "text": format!("cmd-{}", i) }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    tokio::time::sleep(Duration::from_millis(120)).await;

    let raw = read_sse_text("/api/events/stream?last_event_id=0", app.clone(), 20, 200).await;
    let env = parse_sse_envelopes(&raw);
    let gap = env
        .iter()
        .find(|v| v.get("type").and_then(|t| t.as_str()) == Some("gap"));
    assert!(gap.is_some(), "expected gap envelope, got {}", raw);
    let gap = gap.unwrap();
    let dropped = gap
        .get("payload")
        .and_then(|p| p.get("dropped_count"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    assert!(dropped > 0, "expected dropped_count > 0 in gap payload");
}

#[tokio::test]
#[serial]
async fn run_stream_emits_gap_event_when_resume_window_exceeded() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "1");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "printf hello"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id");

    tokio::time::sleep(Duration::from_millis(220)).await;

    let raw = read_sse_text(
        &format!("/api/run/stream/{}?last_event_id=0", job_id),
        app.clone(),
        20,
        200,
    )
    .await;
    let env = parse_sse_envelopes(&raw);
    let gap = env
        .iter()
        .find(|v| v.get("type").and_then(|t| t.as_str()) == Some("gap"));
    assert!(gap.is_some(), "expected gap envelope, got {}", raw);
    let dropped = gap
        .and_then(|v| v.get("payload"))
        .and_then(|p| p.get("dropped_count"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    assert!(dropped > 0, "expected run-stream dropped_count > 0");
}

#[tokio::test]
#[serial]
async fn backend_emits_run_stream_status_activity_events() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "128");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "printf hello"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id");

    // First connect (connected + terminal likely emitted), then reconnect with resume token.
    let _ = read_sse_text(&format!("/api/run/stream/{}", job_id), app.clone(), 16, 200).await;
    let _ = read_sse_text(
        &format!("/api/run/stream/{}?last_event_id=1", job_id),
        app.clone(),
        16,
        200,
    )
    .await;

    tokio::time::sleep(Duration::from_millis(120)).await;

    let events_raw = read_sse_text("/api/events/stream", app.clone(), 64, 200).await;
    let envelopes = parse_sse_envelopes(&events_raw);
    let types: Vec<&str> = envelopes
        .iter()
        .filter_map(|v| v.get("type").and_then(|t| t.as_str()))
        .collect();

    assert!(
        types.contains(&"run_stream_connected"),
        "expected run_stream_connected in activity stream, got {:?}",
        types
    );
    assert!(
        types.contains(&"run_stream_resumed"),
        "expected run_stream_resumed in activity stream, got {:?}",
        types
    );
}

#[tokio::test]
#[serial]
async fn sse_contract_includes_headers_keepalive_and_envelope_format() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_KEEPALIVE_SECS", "1");
    let _g4 = EnvGuard::set("DAL_IDE_SSE_VERSION", "sse.v1");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    // Idle stream should still emit keepalive frames.
    let idle_raw = read_sse_text("/api/events/stream", app.clone(), 4, 1300).await;
    assert!(
        idle_raw.contains("keepalive"),
        "expected keepalive frame text in idle activity stream, got {}",
        idle_raw
    );

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "printf 'hello\\n' && sleep 2"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id");

    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/run/stream/{}", job_id))
        .body(Body::empty())
        .expect("build request");
    let res = app.clone().oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);
    let content_type = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.starts_with("text/event-stream"),
        "expected SSE content-type, got {}",
        content_type
    );

    drop(res);

    let raw = read_sse_text(&format!("/api/run/stream/{}", job_id), app.clone(), 40, 300).await;
    let envelopes = parse_sse_envelopes(&raw);
    let chunk_or_done = envelopes.iter().find(|v| {
        matches!(
            v.get("type").and_then(|t| t.as_str()),
            Some("chunk") | Some("done")
        )
    });
    assert!(
        chunk_or_done.is_some(),
        "expected structured chunk/done envelope, got {}",
        raw
    );
    let sample = chunk_or_done.unwrap();
    assert_sse_envelope_shape(sample);
    assert!(
        sample.get("id").is_some(),
        "missing id in envelope: {:?}",
        sample
    );
    assert!(
        sample.get("timestamp").is_some(),
        "missing timestamp in envelope: {:?}",
        sample
    );
    assert!(
        sample.get("payload").is_some(),
        "missing payload in envelope: {:?}",
        sample
    );
    assert_eq!(
        sample.get("version").and_then(|v| v.as_str()),
        Some("sse.v1")
    );
}

#[tokio::test]
#[serial]
async fn sse_envelope_schema_drift_guard_for_run_and_activity_streams() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "128");
    let _g4 = EnvGuard::set("DAL_IDE_SSE_VERSION", "sse.v1");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "printf hello"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id");

    let run_raw = read_sse_text(&format!("/api/run/stream/{}", job_id), app.clone(), 24, 220).await;
    let run_env = parse_sse_envelopes(&run_raw);
    assert!(!run_env.is_empty(), "expected run envelopes: {}", run_raw);
    for v in &run_env {
        assert_sse_envelope_shape(v);
    }

    let events_raw =
        read_sse_text("/api/events/stream?last_event_id=0", app.clone(), 40, 220).await;
    let events_env = parse_sse_envelopes(&events_raw);
    assert!(
        !events_env.is_empty(),
        "expected activity envelopes: {}",
        events_raw
    );
    for v in &events_env {
        assert_sse_envelope_shape(v);
    }
}

#[tokio::test]
#[serial]
async fn run_stop_cleans_up_job_lifecycle_state() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_JOB_RETENTION_SECS", "1");
    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "sleep 3"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id").to_string();

    let (stop_status, _) = post_json(
        app.clone(),
        "/api/run/stop",
        serde_json::json!({ "job_id": job_id }),
    )
    .await;
    assert_eq!(stop_status, StatusCode::OK);

    let (stop_again_status, _) = post_json(
        app.clone(),
        "/api/run/stop",
        serde_json::json!({ "job_id": job_id }),
    )
    .await;
    assert_eq!(stop_again_status, StatusCode::NOT_FOUND);

    let stream_status_before_cleanup =
        get_status(app.clone(), &format!("/api/run/stream/{}", job_id)).await;
    assert_eq!(stream_status_before_cleanup, StatusCode::OK);

    let cleanup_deadline = Instant::now() + Duration::from_secs(4);
    let mut stream_status_after_cleanup =
        get_status(app.clone(), &format!("/api/run/stream/{}", job_id)).await;
    while stream_status_after_cleanup != StatusCode::NOT_FOUND && Instant::now() < cleanup_deadline
    {
        tokio::time::sleep(Duration::from_millis(100)).await;
        stream_status_after_cleanup =
            get_status(app.clone(), &format!("/api/run/stream/{}", job_id)).await;
    }
    assert_eq!(stream_status_after_cleanup, StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial]
async fn run_stream_handles_burst_output_without_losing_terminal_event() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "1024");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "i=0; while [ $i -lt 400 ]; do printf \"line-%s\\n\" \"$i\"; i=$((i+1)); done"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id");

    let raw = read_sse_text(
        &format!("/api/run/stream/{}", job_id),
        app.clone(),
        1200,
        350,
    )
    .await;
    let envelopes = parse_sse_envelopes(&raw);
    assert!(
        envelopes
            .iter()
            .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("done")),
        "expected done terminal event under burst output, got {}",
        raw
    );
    assert!(
        envelopes
            .iter()
            .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("chunk")),
        "expected at least one chunk event under burst output"
    );
}

#[tokio::test]
#[serial]
async fn run_stream_truncates_oversized_chunk_with_metadata() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "128");
    let _g4 = EnvGuard::set("DAL_IDE_SSE_MAX_CHUNK_BYTES", "8");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());
    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "printf 'abcdefghijklmnop'"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id");

    let raw = read_sse_text(&format!("/api/run/stream/{}", job_id), app.clone(), 24, 220).await;
    let env = parse_sse_envelopes(&raw);
    let chunk = env
        .iter()
        .find(|v| v.get("type").and_then(|t| t.as_str()) == Some("chunk"))
        .expect("chunk event");
    let payload = chunk.get("payload").expect("payload");
    assert_eq!(
        payload.get("truncated").and_then(|t| t.as_bool()),
        Some(true),
        "expected chunk truncation metadata in {}",
        raw
    );
    let original_bytes = payload
        .get("original_bytes")
        .and_then(|n| n.as_u64())
        .expect("original_bytes");
    let emitted_bytes = payload
        .get("text")
        .and_then(|t| t.as_str())
        .map(|t| t.len() as u64)
        .expect("payload.text");
    assert!(
        original_bytes > emitted_bytes,
        "expected original_bytes > emitted_bytes in {}",
        raw
    );
}

#[tokio::test]
#[serial]
async fn events_stream_enforces_per_client_concurrency_limit() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_MAX_STREAMS_PER_CLIENT", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_MAX_ESTABLISH_PER_MINUTE", "100");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let req1 = Request::builder()
        .method("GET")
        .uri("/api/events/stream")
        .header("x-client-id", "client-a")
        .body(Body::empty())
        .expect("build request");
    let res1 = app.clone().oneshot(req1).await.expect("first response");
    assert_eq!(res1.status(), StatusCode::OK);

    let second = get_status_with_headers(
        app.clone(),
        "/api/events/stream",
        &[("x-client-id", "client-a")],
    )
    .await;
    assert_eq!(second, StatusCode::TOO_MANY_REQUESTS);

    drop(res1);
}

#[tokio::test]
#[serial]
async fn events_stream_enforces_establish_rate_limit() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_MAX_STREAMS_PER_CLIENT", "8");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_MAX_ESTABLISH_PER_MINUTE", "1");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let first = get_status_with_headers(
        app.clone(),
        "/api/events/stream",
        &[("x-client-id", "client-rate")],
    )
    .await;
    assert_eq!(first, StatusCode::OK);

    let second = get_status_with_headers(
        app.clone(),
        "/api/events/stream",
        &[("x-client-id", "client-rate")],
    )
    .await;
    assert_eq!(second, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
#[serial]
async fn events_stream_requires_auth_token_when_configured() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_AUTH_TOKEN", "local-dev-token");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_MAX_ESTABLISH_PER_MINUTE", "100");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let unauthorized = get_status(app.clone(), "/api/events/stream").await;
    assert_eq!(unauthorized, StatusCode::UNAUTHORIZED);

    let authorized_header = get_status_with_headers(
        app.clone(),
        "/api/events/stream",
        &[("authorization", "Bearer local-dev-token")],
    )
    .await;
    assert_eq!(authorized_header, StatusCode::OK);

    let authorized_query = get_status(
        app.clone(),
        "/api/events/stream?access_token=local-dev-token",
    )
    .await;
    assert_eq!(authorized_query, StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn cors_policy_legacy_allows_any_origin() {
    let _g1 = EnvGuard::set("DAL_SERVE_SECURITY_PRESET", "legacy");
    let _g2 = EnvGuard::set("DAL_IDE_CORS_ALLOW_ANY", "0");
    let _g3 = EnvGuard::set("DAL_IDE_CORS_ALLOW_ORIGIN", "");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());
    let res = get_response_with_headers(
        app.clone(),
        "/health",
        &[("origin", "https://example.invalid")],
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let origin = res
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok());
    assert_eq!(origin, Some("*"));
}

#[tokio::test]
#[serial]
async fn cors_policy_strict_respects_explicit_origin() {
    let _g1 = EnvGuard::set("DAL_SERVE_SECURITY_PRESET", "strict");
    let _g2 = EnvGuard::set("DAL_IDE_CORS_ALLOW_ANY", "0");
    let _g3 = EnvGuard::set("DAL_IDE_CORS_ALLOW_ORIGIN", "https://allowed.example");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());
    let res = get_response_with_headers(
        app.clone(),
        "/health",
        &[("origin", "https://allowed.example")],
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let origin = res
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok());
    assert_eq!(origin, Some("https://allowed.example"));
}

#[tokio::test]
#[serial]
async fn stream_auth_does_not_block_health_or_metrics() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_AUTH_TOKEN", "local-dev-token");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());
    let health = get_status(app.clone(), "/health").await;
    assert_eq!(health, StatusCode::OK);
    let metrics = get_status(app.clone(), "/metrics").await;
    assert!(
        metrics != StatusCode::UNAUTHORIZED && metrics != StatusCode::FORBIDDEN,
        "expected /metrics not to be blocked by stream auth token policy"
    );
}

#[tokio::test]
#[serial]
async fn run_stream_reconnects_after_interruption_mid_run() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "256");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "i=1; while [ $i -le 18 ]; do printf \"line-%s\\n\" \"$i\"; sleep 0.05; i=$((i+1)); done"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id");

    // First connection reads only an initial window, then disconnects mid-run.
    let first = read_sse_text(&format!("/api/run/stream/{}", job_id), app.clone(), 8, 220).await;
    let first_env = parse_sse_envelopes(&first);
    let last_seen = first_env
        .iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .filter_map(|s| s.parse::<u64>().ok())
        .max()
        .expect("last seen id");

    let resumed = read_sse_text(
        &format!("/api/run/stream/{}?last_event_id={}", job_id, last_seen),
        app.clone(),
        64,
        280,
    )
    .await;
    let resumed_env = parse_sse_envelopes(&resumed);
    let resumed_ids: Vec<u64> = resumed_env
        .iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();
    assert!(
        resumed_ids.iter().all(|id| *id > last_seen),
        "expected resumed ids > {} got {:?}",
        last_seen,
        resumed_ids
    );
    assert!(
        resumed_env
            .iter()
            .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("done")),
        "expected resumed stream to reach done terminal event: {}",
        resumed
    );
}

#[tokio::test]
#[serial]
async fn run_stream_tolerates_burst_reconnects_until_completion() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "512");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "i=1; while [ $i -le 60 ]; do printf \"burst-%s\\n\" \"$i\"; sleep 0.01; i=$((i+1)); done"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id");

    let mut cursor = 0u64;
    for _ in 0..5 {
        let raw = read_sse_text(
            &format!("/api/run/stream/{}?last_event_id={}", job_id, cursor),
            app.clone(),
            6,
            180,
        )
        .await;
        let env = parse_sse_envelopes(&raw);
        if let Some(max_id) = env
            .iter()
            .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
            .filter_map(|s| s.parse::<u64>().ok())
            .max()
        {
            cursor = max_id;
        }
        tokio::time::sleep(Duration::from_millis(40)).await;
    }

    let final_raw = read_sse_text(
        &format!("/api/run/stream/{}?last_event_id={}", job_id, cursor),
        app.clone(),
        200,
        260,
    )
    .await;
    let final_env = parse_sse_envelopes(&final_raw);
    assert!(
        final_env
            .iter()
            .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("done")),
        "expected done after burst reconnects, got {}",
        final_raw
    );
}

#[tokio::test]
#[serial]
async fn forced_stop_emits_run_stopped_activity_event() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "256");

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "i=0; while [ $i -lt 1000 ]; do printf \"tick-%s\\n\" \"$i\"; sleep 0.05; i=$((i+1)); done"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id").to_string();

    // Ensure stream was active before forced stop.
    let _ = read_sse_text(&format!("/api/run/stream/{}", job_id), app.clone(), 4, 150).await;

    let (stop_status, _) = post_json(
        app.clone(),
        "/api/run/stop",
        serde_json::json!({ "job_id": job_id.clone() }),
    )
    .await;
    assert_eq!(stop_status, StatusCode::OK);

    tokio::time::sleep(Duration::from_millis(120)).await;
    let events_raw =
        read_sse_text("/api/events/stream?last_event_id=0", app.clone(), 80, 220).await;
    let env = parse_sse_envelopes(&events_raw);
    let run_stopped = env.iter().find(|v| {
        v.get("type").and_then(|t| t.as_str()) == Some("run_stopped")
            && v.get("payload")
                .and_then(|p| p.get("job_id"))
                .and_then(|id| id.as_str())
                == Some(job_id.as_str())
    });
    let run_terminal_cancelled = env.iter().find(|v| {
        v.get("type").and_then(|t| t.as_str()) == Some("run_stream_terminal")
            && v.get("payload")
                .and_then(|p| p.get("job_id"))
                .and_then(|id| id.as_str())
                == Some(job_id.as_str())
            && v.get("payload")
                .and_then(|p| p.get("terminal"))
                .and_then(|t| t.as_str())
                == Some("cancelled")
    });
    assert!(
        run_stopped.is_some(),
        "expected run_stopped event for forced stop, got {}",
        events_raw
    );
    assert!(
        run_terminal_cancelled.is_some(),
        "expected run_stream_terminal(cancelled) event for forced stop, got {}",
        events_raw
    );
}

#[tokio::test]
#[serial]
#[ignore = "long-running soak harness (set DAL_IDE_SSE_SOAK_SECS, default 1800)"]
async fn soak_run_stream_stability_for_configured_duration() {
    let _g1 = EnvGuard::set("DAL_IDE_SSE_STRUCTURED", "1");
    let _g2 = EnvGuard::set("DAL_IDE_SSE_REPLAY", "1");
    let _g3 = EnvGuard::set("DAL_IDE_SSE_REPLAY_CAP", "4096");
    let _g4 = EnvGuard::set("DAL_IDE_SSE_KEEPALIVE_SECS", "1");

    let soak_secs = std::env::var("DAL_IDE_SSE_SOAK_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(1800);

    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app.clone(),
        "/api/agent/run_command_stream",
        serde_json::json!({
            "cmd": "sh",
            "args": ["-c", "i=0; while [ $i -lt 1000000 ]; do printf \"soak-%s\\n\" \"$i\"; sleep 0.2; i=$((i+1)); done"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let job_id = body["job_id"].as_str().expect("job_id").to_string();

    let deadline = Instant::now() + Duration::from_secs(soak_secs);
    let mut cursor = 0u64;
    let mut reconnect_windows = 0usize;
    let mut chunk_events = 0usize;
    let mut saw_gap = false;

    while Instant::now() < deadline {
        reconnect_windows += 1;
        let raw = read_sse_text(
            &format!("/api/run/stream/{}?last_event_id={}", job_id, cursor),
            app.clone(),
            14,
            1200,
        )
        .await;
        let env = parse_sse_envelopes(&raw);
        for e in &env {
            if e.get("type").and_then(|t| t.as_str()) == Some("gap") {
                saw_gap = true;
            }
            if e.get("type").and_then(|t| t.as_str()) == Some("chunk") {
                chunk_events += 1;
            }
            if let Some(id) = e
                .get("id")
                .and_then(|id| id.as_str())
                .and_then(|s| s.parse::<u64>().ok())
            {
                cursor = cursor.max(id);
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    assert!(
        reconnect_windows > 1,
        "expected multiple reconnect windows during soak"
    );
    assert!(
        chunk_events > 0,
        "expected at least one chunk event in soak"
    );
    assert!(cursor > 0, "expected cursor to advance during soak");
    assert!(
        !saw_gap,
        "soak harness should avoid replay gap in stable conditions"
    );

    let (stop_status, _) = post_json(
        app.clone(),
        "/api/run/stop",
        serde_json::json!({ "job_id": job_id }),
    )
    .await;
    assert_eq!(stop_status, StatusCode::OK);
}

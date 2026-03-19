use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use dist_agent_lang::ide::server::build_router;
use serde_json::{json, Value};
use std::fs;
use tempfile::TempDir;
use tower::ServiceExt;

fn setup_workspace() -> TempDir {
    let workspace = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(workspace.path().join("src")).expect("create src dir");
    fs::write(workspace.path().join("src/main.dal"), "print(\"ok\")").expect("write file");
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
    let body_bytes = to_bytes(res.into_body(), usize::MAX)
        .await
        .expect("read body");
    let parsed = serde_json::from_slice::<Value>(&body_bytes).unwrap_or_else(|_| json!({}));
    (status, parsed)
}

fn canonical_string(path: &std::path::Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

#[tokio::test]
async fn read_file_rejects_parent_traversal() {
    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app,
        "/api/agent/read_file",
        json!({
            "path": "../outside.txt"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap_or("").contains("Invalid path"));
}

#[tokio::test]
async fn read_file_rejects_absolute_path_outside_workspace() {
    let workspace = setup_workspace();
    let outside = tempfile::NamedTempFile::new().expect("outside temp file");
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app,
        "/api/agent/read_file",
        json!({
            "path": outside.path().to_string_lossy()
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap_or("").contains("Invalid path"));
}

#[tokio::test]
async fn write_file_allows_nonexistent_in_root_and_creates_parents() {
    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let rel_path = "nested/new/file.txt";
    let target = workspace.path().join(rel_path);
    assert!(!target.exists());

    let (status, body) = post_json(
        app,
        "/api/agent/write_file",
        json!({
            "path": rel_path,
            "contents": "hello"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));
    assert_eq!(fs::read_to_string(&target).expect("read created file"), "hello");
}

#[tokio::test]
async fn write_file_rejects_nonexistent_parent_traversal() {
    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app,
        "/api/agent/write_file",
        json!({
            "path": "../escape/new.txt",
            "contents": "nope"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap_or("").contains("Invalid path"));
}

#[tokio::test]
async fn list_files_rejects_parent_traversal_subpath() {
    let workspace = setup_workspace();
    let app = build_router(workspace.path().to_path_buf());

    let req = Request::builder()
        .method("GET")
        .uri("/api/files?path=../")
        .body(Body::empty())
        .expect("build request");

    let res = app.oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[cfg(unix)]
#[tokio::test]
async fn read_file_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let workspace = setup_workspace();
    let outside_dir = tempfile::tempdir().expect("outside dir");
    let outside_file = outside_dir.path().join("outside.txt");
    fs::write(&outside_file, "secret").expect("write outside file");

    let link_path = workspace.path().join("link_out");
    symlink(outside_dir.path(), &link_path).expect("create symlink");

    let app = build_router(workspace.path().to_path_buf());
    let (status, _body) = post_json(
        app,
        "/api/agent/read_file",
        json!({
            "path": "link_out/outside.txt"
        }),
    )
    .await;

    assert_ne!(status, StatusCode::OK);
    assert!(status.is_client_error());
}

#[tokio::test]
async fn read_file_allows_absolute_workspace_inside_root() {
    let workspace = setup_workspace();
    let nested_ws = workspace.path().join("nested_ws");
    fs::create_dir_all(nested_ws.join("sub")).expect("create nested workspace");
    fs::write(nested_ws.join("sub/file.txt"), "inside").expect("write nested file");

    let app = build_router(workspace.path().to_path_buf());
    let (status, body) = post_json(
        app,
        "/api/agent/read_file",
        json!({
            "workspace": nested_ws.to_string_lossy(),
            "path": "sub/file.txt"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));
    assert_eq!(body["contents"], json!("inside"));
}

#[tokio::test]
async fn read_file_workspace_outside_root_falls_back_to_server_root() {
    let workspace = setup_workspace();
    let outside = tempfile::tempdir().expect("outside workspace");
    fs::write(outside.path().join("src/main.dal"), "outside").ok();

    let app = build_router(workspace.path().to_path_buf());
    let (status, body) = post_json(
        app,
        "/api/agent/read_file",
        json!({
            "workspace": outside.path().to_string_lossy(),
            "path": "src/main.dal"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["contents"], json!("print(\"ok\")"));
}

#[cfg(unix)]
#[tokio::test]
async fn run_command_outside_cwd_falls_back_to_workspace_root() {
    let workspace = setup_workspace();
    let outside = tempfile::tempdir().expect("outside cwd");
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app,
        "/api/agent/run_command",
        json!({
            "cmd": "sh",
            "args": ["-c", "pwd"],
            "cwd": outside.path().to_string_lossy()
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let stdout = body["stdout"].as_str().unwrap_or("").trim();
    assert_eq!(stdout, canonical_string(workspace.path()));
}

#[cfg(unix)]
#[tokio::test]
async fn run_command_inside_cwd_uses_requested_workspace_subdir() {
    let workspace = setup_workspace();
    let inside = workspace.path().join("src");
    let app = build_router(workspace.path().to_path_buf());

    let (status, body) = post_json(
        app,
        "/api/agent/run_command",
        json!({
            "cmd": "sh",
            "args": ["-c", "pwd"],
            "cwd": inside.to_string_lossy()
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let stdout = body["stdout"].as_str().unwrap_or("").trim();
    assert_eq!(stdout, canonical_string(&inside));
}

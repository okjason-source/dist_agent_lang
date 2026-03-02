//! DAL package registry server. Production-ready: auth, file storage, index API.
//!
//! Endpoints:
//! - GET /v1/packages/:name     -> index JSON (versions + url per version)
//! - GET /v1/packages/:name/versions/:version/tarball -> tarball bytes
//! - PUT /v1/packages/:name/versions/:version -> publish (body = tarball, Authorization: Bearer <token>)
//!
//! Env: REGISTRY_STORAGE_PATH, REGISTRY_TOKEN (auth), REGISTRY_PUBLIC_URL (for index urls), PORT.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, put},
    Router,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

fn storage_path() -> PathBuf {
    std::env::var("REGISTRY_STORAGE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./registry-storage"))
}

fn auth_token() -> Option<String> {
    std::env::var("REGISTRY_TOKEN")
        .or_else(|_| std::env::var("DAL_REGISTRY_TOKEN"))
        .ok()
}

fn public_url() -> String {
    std::env::var("REGISTRY_PUBLIC_URL").unwrap_or_else(|_| "http://localhost:8787".to_string())
}

#[derive(Clone)]
struct AppState {
    storage: PathBuf,
    public_url: String,
    auth_token: Option<String>,
}

fn sanitize_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '/' || c == '@' || c == '-' || c == '_' || c == '.')
}

fn sanitize_version(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
}

/// Map package name to filesystem-safe dir name (e.g. @dal/testing -> _at_dal_slash_testing).
fn name_to_storage_dir(name: &str) -> String {
    name.replace('@', "_at_").replace('/', "_slash_")
}

async fn get_package_index(State(state): State<Arc<AppState>>, Path(name): Path<String>) -> Response {
    if !sanitize_name(&name) {
        return (StatusCode::BAD_REQUEST, "invalid package name").into_response();
    }
    let dir = name_to_storage_dir(&name);
    let index_path = state.storage.join("packages").join(dir).join("index.json");
    match tokio::fs::read_to_string(&index_path).await {
        Ok(s) => {
            let v: serde_json::Value = match serde_json::from_str(&s) {
                Ok(j) => j,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "invalid index.json").into_response(),
            };
            (StatusCode::OK, Json(v)).into_response()
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            (StatusCode::NOT_FOUND, format!("package not found: {}", name)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "read error").into_response(),
    }
}

async fn get_tarball(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
) -> Response {
    if !sanitize_name(&name) || !sanitize_version(&version) {
        return (StatusCode::BAD_REQUEST, "invalid name or version").into_response();
    }
    let dir = name_to_storage_dir(&name);
    let tgz_path = state
        .storage
        .join("packages")
        .join(dir)
        .join("versions")
        .join(format!("{}.tgz", version));
    match tokio::fs::read(&tgz_path).await {
        Ok(bytes) => (
            StatusCode::OK,
            [("Content-Type", "application/gzip")],
            bytes,
        )
            .into_response(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (
            StatusCode::NOT_FOUND,
            format!("version not found: {}@{}", name, version),
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "read error").into_response(),
    }
}

async fn put_version(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
    request: Request<Body>,
) -> Response {
    if !sanitize_name(&name) || !sanitize_version(&version) {
        return (StatusCode::BAD_REQUEST, "invalid name or version").into_response();
    }
    if let Some(ref token) = state.auth_token {
        let auth = request
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !auth.starts_with("Bearer ") || auth.strip_prefix("Bearer ").unwrap_or("") != token.as_str() {
            return (StatusCode::UNAUTHORIZED, "invalid or missing token").into_response();
        }
    }
    const MAX_TARBALL: usize = 50 * 1024 * 1024; // 50 MiB
    let body = match axum::body::to_bytes(request.into_body(), MAX_TARBALL).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_REQUEST, "body error").into_response(),
    };
    let dir = name_to_storage_dir(&name);
    let pkg_dir = state.storage.join("packages").join(dir);
    let versions_dir = pkg_dir.join("versions");
    if let Err(e) = tokio::fs::create_dir_all(&versions_dir).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("create dir: {}", e),
        )
            .into_response();
    }
    let tgz_path = versions_dir.join(format!("{}.tgz", version));
    if let Err(e) = tokio::fs::write(&tgz_path, body.as_ref()).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("write: {}", e),
        )
            .into_response();
    }
    let index_path = pkg_dir.join("index.json");
    let base = state.public_url.trim_end_matches('/');
    let url = format!("{}/v1/packages/{}/versions/{}/tarball", base, name, version);
    let mut index: serde_json::Map<String, serde_json::Value> = match tokio::fs::read_to_string(&index_path).await {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| {
            let mut m = serde_json::Map::new();
            m.insert("name".to_string(), json!(&name));
            m.insert("versions".to_string(), json!({}));
            m
        }),
        Err(_) => {
            let mut m = serde_json::Map::new();
            m.insert("name".to_string(), json!(&name));
            m.insert("versions".to_string(), json!({}));
            m
        }
    };
    let versions = index
        .get_mut("versions")
        .and_then(|v| v.as_object_mut())
        .expect("versions object");
    versions.insert(version.clone(), json!({ "url": url }));
    let index_bytes = serde_json::to_vec_pretty(&index).expect("json");
    if let Err(e) = tokio::fs::write(&index_path, index_bytes).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("write index: {}", e),
        )
            .into_response();
    }
    (StatusCode::CREATED, Json(json!({ "ok": true, "name": name, "version": version }))).into_response()
}

#[tokio::main]
async fn main() {
    let storage = storage_path();
    let public_url = public_url();
    let auth_token = auth_token();
    if auth_token.is_none() {
        eprintln!("⚠️  REGISTRY_TOKEN (or DAL_REGISTRY_TOKEN) not set; publish is unauthenticated.");
    }
    if let Err(e) = std::fs::create_dir_all(&storage) {
        eprintln!("Failed to create storage dir {:?}: {}", storage, e);
        std::process::exit(1);
    }
    let state = Arc::new(AppState {
        storage,
        public_url,
        auth_token,
    });
    let app = Router::new()
        .route("/v1/packages/:name", get(get_package_index))
        .route(
            "/v1/packages/:name/versions/:version/tarball",
            get(get_tarball),
        )
        .route("/v1/packages/:name/versions/:version", put(put_version))
        .layer(CorsLayer::permissive())
        .with_state(state);
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8787);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("DAL registry listening on http://{} (storage: {:?})", addr, storage_path());
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

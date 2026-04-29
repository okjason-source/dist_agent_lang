use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use dist_agent_lang::http_server_integration::{
    apply_standard_http_layers, dal_serve_basic_auth_configured, ServeSecurityOptions,
    ServeSecurityPreset,
};
use serial_test::serial;
use tower::ServiceExt;

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

    fn unset(key: &'static str) -> Self {
        let original = std::env::var(key).ok();
        std::env::remove_var(key);
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

/// Avoid flaky tests when the shell has `DAL_HTTP_*` Basic Auth vars set.
fn clear_dal_serve_http_auth_env() -> (EnvGuard, EnvGuard, EnvGuard) {
    (
        EnvGuard::unset("DAL_HTTP_USER"),
        EnvGuard::unset("DAL_HTTP_PASSWORD"),
        EnvGuard::unset("DAL_HTTP_PASSWORD_HASH"),
    )
}

fn test_router() -> Router {
    Router::new()
        .route("/protected", get(|| async { "ok" }))
        .route("/upload", post(|| async { "uploaded" }))
        .route("/health", get(|| async { "healthy" }))
}

fn strict_options() -> ServeSecurityOptions {
    ServeSecurityOptions {
        preset: ServeSecurityPreset::Strict,
        enable_security_headers: true,
        enable_rate_limit: true,
        enable_request_size_limit: true,
        enable_input_validation: true,
        enable_auth_middleware: true,
        rate_limit_requests_per_minute: 60,
        rate_limit_window_seconds: 60,
        max_body_bytes: 512_000,
        max_header_bytes: 8_192,
        max_url_length: 2_048,
        public_paths_without_auth: vec!["/metrics".to_string(), "/health".to_string()],
        public_paths_without_input_validation: vec!["/metrics".to_string()],
    }
}

#[test]
#[serial]
fn test_serve_security_from_env_defaults_to_legacy() {
    let _g1 = EnvGuard::unset("DAL_SERVE_SECURITY_PRESET");
    let _g2 = EnvGuard::unset("DAL_SERVE_ENABLE_AUTH");
    let _g3 = EnvGuard::unset("DAL_SERVE_ENABLE_INPUT_VALIDATION");
    let _g4 = EnvGuard::unset("DAL_SERVE_RATE_LIMIT_RPM");
    let _g5 = EnvGuard::unset("DAL_SERVE_MAX_BODY_BYTES");

    let opts = ServeSecurityOptions::from_env();
    assert_eq!(opts.preset, ServeSecurityPreset::Legacy);
    assert!(!opts.enable_auth_middleware);
    assert!(!opts.enable_input_validation);
    assert!(!opts.enable_rate_limit);
    assert!(!opts.enable_request_size_limit);
}

#[test]
#[serial]
fn test_serve_security_from_env_balanced_and_overrides() {
    let _g1 = EnvGuard::set("DAL_SERVE_SECURITY_PRESET", "balanced");
    let _g2 = EnvGuard::set("DAL_SERVE_ENABLE_AUTH", "TRUE");
    let _g3 = EnvGuard::set("DAL_SERVE_ENABLE_INPUT_VALIDATION", "1");
    let _g4 = EnvGuard::set("DAL_SERVE_RATE_LIMIT_RPM", "777");
    let _g5 = EnvGuard::set("DAL_SERVE_MAX_BODY_BYTES", "12345");

    let opts = ServeSecurityOptions::from_env();
    assert_eq!(opts.preset, ServeSecurityPreset::Balanced);
    assert!(opts.enable_security_headers);
    assert!(opts.enable_rate_limit);
    assert!(opts.enable_request_size_limit);
    assert!(opts.enable_auth_middleware);
    assert!(opts.enable_input_validation);
    assert_eq!(opts.rate_limit_requests_per_minute, 777);
    assert_eq!(opts.max_body_bytes, 12_345);
}

#[test]
#[serial]
fn test_serve_security_from_env_strict_baseline() {
    let _g1 = EnvGuard::set("DAL_SERVE_SECURITY_PRESET", "strict");
    let _g2 = EnvGuard::unset("DAL_SERVE_ENABLE_AUTH");
    let _g3 = EnvGuard::unset("DAL_SERVE_ENABLE_INPUT_VALIDATION");

    let opts = ServeSecurityOptions::from_env();
    assert_eq!(opts.preset, ServeSecurityPreset::Strict);
    assert!(opts.enable_auth_middleware);
    assert!(opts.enable_input_validation);
    assert!(opts.enable_rate_limit);
    assert!(opts.enable_request_size_limit);
}

#[tokio::test]
#[serial]
async fn test_legacy_allows_protected_route_without_auth() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let options = ServeSecurityOptions::default();
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn test_strict_requires_auth_for_protected_route() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let options = strict_options();
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_strict_allows_options_preflight_without_auth() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let options = strict_options();
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/protected")
                .header("Origin", "https://example.com")
                .header("Access-Control-Request-Method", "GET")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_strict_metrics_auth_exemption_works() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let _g = EnvGuard::set("DAL_METRICS", "1");
    let options = strict_options();
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn test_strict_metrics_input_validation_exemption_works() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let _g = EnvGuard::set("DAL_METRICS", "1");
    let options = strict_options();
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics?q=javascript:alert(1)")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn test_balanced_default_does_not_enable_input_validation() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let options = ServeSecurityOptions {
        preset: ServeSecurityPreset::Balanced,
        enable_security_headers: true,
        enable_rate_limit: true,
        enable_request_size_limit: true,
        enable_input_validation: false,
        enable_auth_middleware: false,
        rate_limit_requests_per_minute: 120,
        rate_limit_window_seconds: 60,
        max_body_bytes: 1_000_000,
        max_header_bytes: 8_192,
        max_url_length: 2_048,
        public_paths_without_auth: vec!["/metrics".to_string(), "/health".to_string()],
        public_paths_without_input_validation: vec!["/metrics".to_string()],
    };
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected?q=javascript:alert(1)")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn test_rate_limit_enforces_second_request() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let options = ServeSecurityOptions {
        enable_rate_limit: true,
        rate_limit_requests_per_minute: 1,
        rate_limit_window_seconds: 60,
        ..ServeSecurityOptions::default()
    };
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
#[serial]
async fn test_request_size_limit_rejects_large_body_header() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let options = ServeSecurityOptions {
        enable_request_size_limit: true,
        max_body_bytes: 10,
        ..ServeSecurityOptions::default()
    };
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/upload")
                .header("Content-Length", "1000")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
#[serial]
async fn test_security_headers_enabled_adds_expected_headers() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let options = ServeSecurityOptions {
        enable_security_headers: true,
        ..ServeSecurityOptions::default()
    };
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert!(headers.get("content-security-policy").is_some());
    assert!(headers.get("x-frame-options").is_some());
    assert!(headers.get("x-content-type-options").is_some());
}

#[tokio::test]
#[serial]
async fn test_dal_serve_basic_auth_when_env_set() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let _u = EnvGuard::set("DAL_HTTP_USER", "alice");
    let _p = EnvGuard::set("DAL_HTTP_PASSWORD", "secret");
    let options = ServeSecurityOptions::default();
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);

    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(health.status(), StatusCode::OK);

    let tok = B64.encode(b"alice:secret");
    let ok = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("Authorization", format!("Basic {}", tok))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(ok.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn test_dal_serve_basic_auth_bcrypt_hash() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let _u = EnvGuard::set("DAL_HTTP_USER", "alice");
    let hash = bcrypt::hash("secret", 4).expect("bcrypt hash");
    let _h = EnvGuard::set("DAL_HTTP_PASSWORD_HASH", &hash);
    let options = ServeSecurityOptions::default();
    let app = apply_standard_http_layers(test_router(), "*", &options);

    let tok = B64.encode(b"alice:secret");
    let ok = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("Authorization", format!("Basic {}", tok))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(ok.status(), StatusCode::OK);
}

#[test]
fn test_serve_security_preset_as_str_roundtrip() {
    assert_eq!(ServeSecurityPreset::Legacy.as_str(), "legacy");
    assert_eq!(ServeSecurityPreset::Balanced.as_str(), "balanced");
    assert_eq!(ServeSecurityPreset::Strict.as_str(), "strict");
}

/// Zero / invalid numeric env must not override preset defaults (`n > 0` guards).
#[test]
#[serial]
fn test_serve_security_from_env_ignores_zero_rate_and_body_limits() {
    let _g0 = EnvGuard::set("DAL_SERVE_SECURITY_PRESET", "strict");
    let _g1 = EnvGuard::set("DAL_SERVE_RATE_LIMIT_RPM", "0");
    let _g2 = EnvGuard::set("DAL_SERVE_MAX_BODY_BYTES", "0");
    let _g3 = EnvGuard::unset("DAL_SERVE_ENABLE_AUTH");
    let _g4 = EnvGuard::unset("DAL_SERVE_ENABLE_INPUT_VALIDATION");

    let opts = ServeSecurityOptions::from_env();
    assert_eq!(opts.preset, ServeSecurityPreset::Strict);
    assert_eq!(opts.rate_limit_requests_per_minute, 60);
    assert_eq!(opts.max_body_bytes, 512_000);
}

#[test]
#[serial]
fn test_serve_security_from_env_parse_bool_false_overrides_strict_auth() {
    let _g0 = EnvGuard::set("DAL_SERVE_SECURITY_PRESET", "strict");
    let _g1 = EnvGuard::set("DAL_SERVE_ENABLE_AUTH", "false");
    let _g2 = EnvGuard::unset("DAL_SERVE_ENABLE_INPUT_VALIDATION");

    let opts = ServeSecurityOptions::from_env();
    assert_eq!(opts.preset, ServeSecurityPreset::Strict);
    assert!(
        !opts.enable_auth_middleware,
        "explicit false must disable auth even under strict preset"
    );
}

#[test]
#[serial]
fn test_serve_security_balanced_shape_all_fields() {
    let _g1 = EnvGuard::set("DAL_SERVE_SECURITY_PRESET", "balanced");
    let _g2 = EnvGuard::unset("DAL_SERVE_ENABLE_AUTH");
    let _g3 = EnvGuard::unset("DAL_SERVE_ENABLE_INPUT_VALIDATION");
    let _g4 = EnvGuard::unset("DAL_SERVE_RATE_LIMIT_RPM");
    let _g5 = EnvGuard::unset("DAL_SERVE_MAX_BODY_BYTES");

    let o = ServeSecurityOptions::from_env();
    assert_eq!(o.preset, ServeSecurityPreset::Balanced);
    assert!(o.enable_security_headers);
    assert!(o.enable_rate_limit);
    assert!(o.enable_request_size_limit);
    assert!(!o.enable_input_validation);
    assert!(!o.enable_auth_middleware);
    assert_eq!(o.rate_limit_requests_per_minute, 120);
    assert_eq!(o.rate_limit_window_seconds, 60);
    assert_eq!(o.max_body_bytes, 1_000_000);
    assert_eq!(o.max_header_bytes, 8_192);
    assert_eq!(o.max_url_length, 2_048);
}

#[test]
#[serial]
fn test_serve_security_strict_shape_all_fields() {
    let _g1 = EnvGuard::set("DAL_SERVE_SECURITY_PRESET", "strict");
    let _g2 = EnvGuard::unset("DAL_SERVE_ENABLE_AUTH");
    let _g3 = EnvGuard::unset("DAL_SERVE_ENABLE_INPUT_VALIDATION");
    let _g4 = EnvGuard::unset("DAL_SERVE_RATE_LIMIT_RPM");
    let _g5 = EnvGuard::unset("DAL_SERVE_MAX_BODY_BYTES");

    let o = ServeSecurityOptions::from_env();
    assert_eq!(o.preset, ServeSecurityPreset::Strict);
    assert!(o.enable_security_headers);
    assert!(o.enable_rate_limit);
    assert!(o.enable_request_size_limit);
    assert!(o.enable_input_validation);
    assert!(o.enable_auth_middleware);
    assert_eq!(o.rate_limit_requests_per_minute, 60);
    assert_eq!(o.rate_limit_window_seconds, 60);
    assert_eq!(o.max_body_bytes, 512_000);
    assert_eq!(o.max_header_bytes, 8_192);
    assert_eq!(o.max_url_length, 2_048);
}

#[test]
#[serial]
fn test_dal_serve_basic_auth_not_configured_with_user_only() {
    let (_auth_u, _auth_p, _auth_h) = clear_dal_serve_http_auth_env();
    let _u = EnvGuard::set("DAL_HTTP_USER", "alice");
    assert!(
        !dal_serve_basic_auth_configured(),
        "must require password or hash when user is set"
    );
}

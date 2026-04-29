// HTTP Server Integration
// Integrates middleware and handlers into the HTTP server router

use crate::http_server::WebServerState;
use crate::http_server_converters::error_response;
use crate::http_server_handlers::handle_with_middleware;
use crate::http_server_security::{security_headers_middleware, RateLimiter, RequestSizeLimiter};
use crate::http_server_security_middleware::{
    auth_middleware_with_exemptions, dal_serve_basic_auth_middleware,
    input_validation_middleware_with_exemptions, rate_limit_middleware, request_size_middleware,
    AuthExemptPaths, InputValidationExemptPaths,
};
use crate::stdlib::web::HttpServer;
use axum::{
    body::Body,
    extract::{Request, State},
    http::Method,
    middleware,
    response::Response,
    routing::{get, post},
    Extension, Router,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServeSecurityPreset {
    Legacy,
    Balanced,
    Strict,
}

impl ServeSecurityPreset {
    pub fn from_env() -> Self {
        let raw = std::env::var("DAL_SERVE_SECURITY_PRESET")
            .unwrap_or_else(|_| "legacy".to_string())
            .to_lowercase();
        match raw.as_str() {
            "balanced" => Self::Balanced,
            "strict" => Self::Strict,
            _ => Self::Legacy,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Balanced => "balanced",
            Self::Strict => "strict",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServeSecurityOptions {
    pub preset: ServeSecurityPreset,
    pub enable_security_headers: bool,
    pub enable_rate_limit: bool,
    pub enable_request_size_limit: bool,
    pub enable_input_validation: bool,
    pub enable_auth_middleware: bool,
    pub rate_limit_requests_per_minute: usize,
    pub rate_limit_window_seconds: u64,
    pub max_body_bytes: usize,
    pub max_header_bytes: usize,
    pub max_url_length: usize,
    pub public_paths_without_auth: Vec<String>,
    pub public_paths_without_input_validation: Vec<String>,
}

impl Default for ServeSecurityOptions {
    fn default() -> Self {
        // Default = legacy for compatibility-first rollout.
        Self {
            preset: ServeSecurityPreset::Legacy,
            enable_security_headers: false,
            enable_rate_limit: false,
            enable_request_size_limit: false,
            enable_input_validation: false,
            enable_auth_middleware: false,
            rate_limit_requests_per_minute: 120,
            rate_limit_window_seconds: 60,
            max_body_bytes: 1_000_000,
            max_header_bytes: 8_192,
            max_url_length: 2_048,
            public_paths_without_auth: vec!["/metrics".to_string(), "/health".to_string()],
            public_paths_without_input_validation: vec!["/metrics".to_string()],
        }
    }
}

impl ServeSecurityOptions {
    pub fn from_env() -> Self {
        let preset = ServeSecurityPreset::from_env();
        let mut opts = match preset {
            ServeSecurityPreset::Legacy => Self {
                preset,
                ..Self::default()
            },
            ServeSecurityPreset::Balanced => Self {
                preset,
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
                ..Self::default()
            },
            ServeSecurityPreset::Strict => Self {
                preset,
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
                ..Self::default()
            },
        };

        if let Ok(v) = std::env::var("DAL_SERVE_ENABLE_AUTH") {
            opts.enable_auth_middleware = parse_bool_flag(&v);
        }
        if let Ok(v) = std::env::var("DAL_SERVE_ENABLE_INPUT_VALIDATION") {
            opts.enable_input_validation = parse_bool_flag(&v);
        }
        if let Ok(v) = std::env::var("DAL_SERVE_RATE_LIMIT_RPM") {
            if let Ok(n) = v.parse::<usize>() {
                if n > 0 {
                    opts.rate_limit_requests_per_minute = n;
                }
            }
        }
        if let Ok(v) = std::env::var("DAL_SERVE_MAX_BODY_BYTES") {
            if let Ok(n) = v.parse::<usize>() {
                if n > 0 {
                    opts.max_body_bytes = n;
                }
            }
        }
        opts
    }
}

fn parse_bool_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// True when optional HTTP Basic Auth should be active (`dal serve` layer).
pub fn dal_serve_basic_auth_configured() -> bool {
    let user_ok = std::env::var("DAL_HTTP_USER")
        .ok()
        .filter(|s| !s.is_empty())
        .is_some();
    if !user_ok {
        return false;
    }
    let hash_ok = std::env::var("DAL_HTTP_PASSWORD_HASH")
        .ok()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let pass_ok = std::env::var("DAL_HTTP_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty())
        .is_some();
    hash_ok || pass_ok
}

pub fn apply_standard_http_layers(
    router: Router,
    cors_origin: &str,
    options: &ServeSecurityOptions,
) -> Router {
    let cors = if cors_origin == "*" {
        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ])
            .allow_origin(Any)
            .allow_headers(Any)
    } else {
        let origin = cors_origin
            .parse::<axum::http::HeaderValue>()
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("*"));
        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ])
            .allow_origin(origin)
            .allow_headers(Any)
    };

    let mut app = router
        .route("/metrics", get(crate::observability::metrics_http_response))
        .layer(middleware::from_fn(
            crate::observability::http_observability_middleware,
        ))
        .layer(cors);

    if options.enable_security_headers {
        app = app.layer(middleware::from_fn(security_headers_middleware));
    }

    if options.enable_rate_limit {
        let limiter = Arc::new(RateLimiter::new(
            options.rate_limit_requests_per_minute,
            options.rate_limit_window_seconds,
        ));
        app = app
            .layer(middleware::from_fn(rate_limit_middleware))
            // Extension must be added last so middleware sees it first.
            .layer(Extension(limiter));
    }

    if options.enable_request_size_limit {
        let size = Arc::new(RequestSizeLimiter {
            max_body_size: options.max_body_bytes,
            max_header_size: options.max_header_bytes,
            max_url_length: options.max_url_length,
        });
        app = app
            .layer(middleware::from_fn(request_size_middleware))
            .layer(Extension(size));
    }

    if options.enable_input_validation {
        let exempt_paths: HashSet<String> = options
            .public_paths_without_input_validation
            .iter()
            .cloned()
            .collect();
        app = app
            .layer(middleware::from_fn(
                input_validation_middleware_with_exemptions,
            ))
            .layer(Extension(InputValidationExemptPaths(Arc::new(
                exempt_paths,
            ))));
    }

    if options.enable_auth_middleware {
        let exempt_paths: HashSet<String> =
            options.public_paths_without_auth.iter().cloned().collect();
        app = app
            .layer(middleware::from_fn(auth_middleware_with_exemptions))
            .layer(Extension(AuthExemptPaths(Arc::new(exempt_paths))));
    }

    // Optional HTTP Basic Auth when DAL_HTTP_USER + (DAL_HTTP_PASSWORD_HASH or DAL_HTTP_PASSWORD).
    // Independent of DAL_SERVE_ENABLE_AUTH / JWT middleware.
    if dal_serve_basic_auth_configured() {
        app = app.layer(middleware::from_fn(dal_serve_basic_auth_middleware));
    }

    app
}

/// Create router with middleware support
pub fn create_router_with_middleware(server: HttpServer) -> Router {
    create_router_with_runtime_factory(server, || crate::runtime::engine::Runtime::new())
}

/// Create router with custom runtime factory (for E2E tests that need stub handlers).
pub fn create_router_with_runtime_factory<F>(server: HttpServer, factory: F) -> Router
where
    F: Fn() -> crate::runtime::engine::Runtime + Send + Sync + 'static,
{
    create_router_with_options(server, factory, None)
}

/// Create router with runtime factory and optional scope writeback for shared state.
pub fn create_router_with_options<F>(
    server: HttpServer,
    factory: F,
    scope_writeback: Option<Arc<Box<dyn Fn(&crate::runtime::scope::Scope) + Send + Sync>>>,
) -> Router
where
    F: Fn() -> crate::runtime::engine::Runtime + Send + Sync + 'static,
{
    let runtime_factory = Arc::new(
        Box::new(factory) as Box<dyn Fn() -> crate::runtime::engine::Runtime + Send + Sync>
    );

    let state = Arc::new(RwLock::new(WebServerState {
        server: server.clone(),
        handlers: std::collections::HashMap::new(),
        runtime_factory: Some(runtime_factory),
        scope_writeback,
    }));

    let mut router = Router::new();

    // Add routes from server configuration
    for (route_key, route) in &server.routes {
        // Parse route key (format: "METHOD:/path")
        if let Some((method_str, path)) = route_key.split_once(':') {
            let method = method_str.to_uppercase();
            let handler_name = route.handler.clone();

            // Create handler closure
            let state_clone = state.clone();
            let handler = move |request: Request| {
                let state = state_clone.clone();
                let handler_name = handler_name.clone();
                async move { handle_with_middleware(State(state), request, &handler_name).await }
            };

            // Add route based on method
            match method.as_str() {
                "GET" => {
                    router = router.route(path, get(handler));
                }
                "POST" => {
                    router = router.route(path, post(handler));
                }
                "PUT" => {
                    router = router.route(path, axum::routing::put(handler));
                }
                "DELETE" => {
                    router = router.route(path, axum::routing::delete(handler));
                }
                _ => {
                    // Default to GET
                    router = router.route(path, get(handler));
                }
            }
        }
    }

    // Add default routes if server has no routes configured
    if server.routes.is_empty() {
        router = router
            .route("/", get(home_handler))
            .route("/health", get(health_handler));
    }

    router.with_state(state)
}

/// Default home handler
async fn home_handler() -> Response<Body> {
    error_response(200, "Welcome to dist_agent_lang HTTP Server")
}

/// Default health handler
async fn health_handler() -> Response<Body> {
    error_response(200, "OK")
}

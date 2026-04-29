# Spec: dist_agent_lang `dal serve` Security Parity Patch

Status: Draft patch proposal  
Owner: Jason  
Last updated: 2026-04-04

## Goal

Make `dal serve` use the same baseline security middleware stack as the other HTTP server path, while preserving backward compatibility.

Current issue:

- `dal serve` route path currently applies CORS + observability/metrics at that layer.
- Full security middleware (rate limit, request-size, input validation, optional auth) is available in other server wiring, but not consistently applied in `dal serve`.

Desired outcome:

- One shared HTTP-layer builder used by `dal serve`.
- Preset-driven rollout (`legacy`, `balanced`, `strict`).
- Safe defaults for production while allowing legacy behavior.
- Preserve browser preflight (`OPTIONS`) and keep `/metrics` reachable in strict mode.

---

## Proposed patch (copy-ready)

## 1) Patch `src/http_server_integration.rs`

Add a reusable security-layer builder and env-driven options.

```rust
// add imports near top
use crate::http_server_security::{security_headers_middleware, RateLimiter, RequestSizeLimiter};
use crate::http_server_security_middleware::{
    auth_middleware_with_exemptions, input_validation_middleware_with_exemptions,
    rate_limit_middleware, request_size_middleware,
};
use axum::{extract::State, http::Method, middleware, routing::get, Extension};
use std::collections::HashSet;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
```

```rust
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
            "legacy" => Self::Legacy,
            "strict" => Self::Strict,
            _ => Self::Balanced,
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
        // default = legacy (safe rollout / behavior parity)
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
            public_paths_without_auth: vec!["/metrics".into(), "/health".into()],
            public_paths_without_input_validation: vec!["/metrics".into()],
        }
    }
}

impl ServeSecurityOptions {
    pub fn from_env() -> Self {
        let preset = ServeSecurityPreset::from_env();

        let mut opts = match preset {
            ServeSecurityPreset::Legacy => Self {
                preset,
                enable_security_headers: false,
                enable_rate_limit: false,
                enable_request_size_limit: false,
                enable_input_validation: false,
                enable_auth_middleware: false,
                ..Self::default()
            },
            ServeSecurityPreset::Balanced => Self {
                preset,
                enable_security_headers: true,
                enable_rate_limit: true,
                enable_request_size_limit: true,
                // Keep balanced compatible with common browser traffic by default.
                // Validation can be enabled explicitly per deployment.
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
            },
        };

        if let Ok(v) = std::env::var("DAL_SERVE_ENABLE_AUTH") {
            opts.enable_auth_middleware = matches!(v.as_str(), "1" | "true" | "TRUE" | "yes");
        }

        if let Ok(v) = std::env::var("DAL_SERVE_ENABLE_INPUT_VALIDATION") {
            opts.enable_input_validation = matches!(v.as_str(), "1" | "true" | "TRUE" | "yes");
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
```

```rust
// add new helper in this file
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

    let auth_exempt = Arc::new(
        options
            .public_paths_without_auth
            .iter()
            .cloned()
            .collect::<HashSet<_>>(),
    );
    let validation_exempt = Arc::new(
        options
            .public_paths_without_input_validation
            .iter()
            .cloned()
            .collect::<HashSet<_>>(),
    );

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

    if options.enable_auth_middleware {
        app = app.layer(Extension(auth_exempt));
        app = app.layer(middleware::from_fn(auth_middleware_with_exemptions));
    }

    if options.enable_input_validation {
        app = app.layer(Extension(validation_exempt));
        app = app.layer(middleware::from_fn(input_validation_middleware_with_exemptions));
    }

    app
}
```

## 1b) Patch `src/http_server_security_middleware.rs`

Add wrappers that preserve browser preflight and allow public endpoint exemptions.

```rust
use axum::http::Method;
use std::collections::HashSet;
```

```rust
pub async fn auth_middleware_with_exemptions(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // Never block CORS preflight.
    if method == Method::OPTIONS {
        return next.run(request).await;
    }

    // Allow explicitly public routes (e.g., /metrics, /health).
    let exempt = request
        .extensions()
        .get::<Arc<HashSet<String>>>()
        .cloned();
    if exempt.as_ref().map(|set| set.contains(&path)).unwrap_or(false) {
        return next.run(request).await;
    }

    auth_middleware(request, next).await
}

pub async fn input_validation_middleware_with_exemptions(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // Preflight should pass through unvalidated.
    if method == Method::OPTIONS {
        return next.run(request).await;
    }

    let exempt = request
        .extensions()
        .get::<Arc<HashSet<String>>>()
        .cloned();
    if exempt.as_ref().map(|set| set.contains(&path)).unwrap_or(false) {
        return next.run(request).await;
    }

    input_validation_middleware(request, next).await
}
```

---

## 2) Patch `src/main.rs` (`run_serve`)

Replace local CORS/metrics layer setup with shared layer helper.

### Remove current block

Remove the current section that builds CORS and then does:

```rust
let app = app
    .route("/metrics", ...)
    .layer(middleware::from_fn(...))
    .layer(cors);
```

### Add

```rust
let serve_security = dist_agent_lang::http_server_integration::ServeSecurityOptions::from_env();
let app = dist_agent_lang::http_server_integration::apply_standard_http_layers(
    app,
    cors_origin,
    &serve_security,
);

println!("✅ Serve security preset: {}", serve_security.preset.as_str());
if serve_security.enable_auth_middleware {
    println!("🔐 Auth middleware enabled (JWT_SECRET required)");
}
if serve_security.enable_input_validation {
    println!("🧪 Input validation middleware enabled");
}
```

This keeps `run_serve` behavior centralized and consistent.

---

## 3) Patch docs (`docs/CONFIG.md`)

Add env vars section:

```md
| **`dal serve` security presets** | `DAL_SERVE_SECURITY_PRESET`, `DAL_SERVE_ENABLE_AUTH`, `DAL_SERVE_ENABLE_INPUT_VALIDATION`, `DAL_SERVE_RATE_LIMIT_RPM`, `DAL_SERVE_MAX_BODY_BYTES` | `DAL_SERVE_SECURITY_PRESET=legacy|balanced|strict` (default `legacy`). `legacy` keeps old behavior (CORS + observability only). `balanced` enables security headers + rate-limit + request-size by default, with input validation opt-in. `strict` also enables auth middleware by default (requires `JWT_SECRET`). |
```

---

## Compatibility matrix (recommended defaults)

## Preset compatibility matrix

| Preset | Middleware stack | Backward compatibility | Recommended use |
|---|---|---|---|
| `legacy` | CORS + observability + metrics | Highest | Existing deployments that need zero behavior change |
| `balanced` | `legacy` + security headers + rate limit + request size (`input_validation` opt-in) | High | Recommended step-up after legacy rollout |
| `strict` | `balanced` + auth middleware | Medium (requires token/JWT config) | Internet-exposed endpoints with explicit auth policy |

## Behavior compatibility notes

| Area | `legacy` | `balanced` | `strict` |
|---|---|---|---|
| Existing clients without auth header | Works | Works | Fails with `401` unless auth disabled |
| Large request bodies | Allowed per current behavior | Rejected above limit | Rejected above tighter limit |
| High burst traffic | Unthrottled | Throttled by RPM | Throttled more aggressively |
| Invalid/suspicious query/header payloads | Often allowed | Allowed by default; rejected with `400` when validation enabled | Rejected with `400` when validation enabled |
| Security headers | Not added | Added | Added |
| Browser preflight (`OPTIONS`) | Available | Available | Available (explicit pass-through) |
| Metrics endpoint | Available | Available | Available (auth exempt) |

## Recommended production defaults

- Default preset: `balanced`
- For internet-facing environments: move to `strict` after auth token flows are validated.
- Keep override controls:
  - `DAL_SERVE_SECURITY_PRESET=legacy` for emergency rollback
  - `DAL_SERVE_ENABLE_AUTH=0` if strict auth rollout must be delayed

Suggested values:

```bash
# baseline production-like
DAL_SERVE_SECURITY_PRESET=legacy
DAL_SERVE_RATE_LIMIT_RPM=120
DAL_SERVE_MAX_BODY_BYTES=1000000
# optional hardening toggle
# DAL_SERVE_ENABLE_INPUT_VALIDATION=1

# hardened internet-facing
DAL_SERVE_SECURITY_PRESET=strict
DAL_SERVE_ENABLE_AUTH=1
JWT_SECRET=<strong-random-secret>
DAL_SERVE_RATE_LIMIT_RPM=60
DAL_SERVE_MAX_BODY_BYTES=512000
DAL_SERVE_ENABLE_INPUT_VALIDATION=1
```

---

## Rollout guidance

1. Release with default `legacy` for compatibility.
2. Monitor:
   - `401/403` spikes
   - `400` validation rejects
   - `413` payload-too-large rejects
   - `429` rate-limit rejects
3. Move selected environments to `balanced`.
4. After client compatibility confirmed, promote exposed deployments to `strict`.

---

## Validation checklist

- [ ] `dal serve` with no env behaves as `legacy`.
- [ ] `DAL_SERVE_SECURITY_PRESET=legacy` reproduces old behavior.
- [ ] `DAL_SERVE_SECURITY_PRESET=strict` rejects unauthenticated requests.
- [ ] `OPTIONS` preflight requests succeed in `strict`.
- [ ] `/metrics` remains available when `DAL_METRICS=1`.
- [ ] Existing route handlers (`@route`) still execute unchanged.

---

## Notes

- This patch intentionally separates security posture from DAL language semantics.
- It preserves compatibility while making secure operation the practical default.

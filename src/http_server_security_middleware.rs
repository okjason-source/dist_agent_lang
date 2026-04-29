// HTTP Server Security Middleware
// Axum middleware wrappers for security controls

use crate::http_server_security::{
    AuthValidator, DalServeBasicAuthBruteForce, InputValidator, RateLimiter, RequestSizeLimiter,
    SecurityLogger,
};
use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::STANDARD as B64_ENGINE, Engine as _};
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;

lazy_static::lazy_static! {
    static ref DAL_SERVE_BASIC_AUTH_BF: DalServeBasicAuthBruteForce =
        DalServeBasicAuthBruteForce::from_env();
}

#[derive(Clone, Debug)]
pub struct AuthExemptPaths(pub Arc<HashSet<String>>);

#[derive(Clone, Debug)]
pub struct InputValidationExemptPaths(pub Arc<HashSet<String>>);

/// Rate limiting middleware
pub async fn rate_limit_middleware(request: Request, next: Next) -> Response {
    // Get rate limiter from extensions or use default
    let rate_limiter = request
        .extensions()
        .get::<Arc<RateLimiter>>()
        .cloned()
        .unwrap_or_else(|| Arc::new(RateLimiter::new(100, 60))); // Default: 100 req/min

    // Extract IP address
    let ip = extract_ip(&request);

    // Check rate limit
    match rate_limiter.check_rate_limit(ip).await {
        Ok(_) => next.run(request).await,
        Err(status) => {
            SecurityLogger::log_rate_limit(&ip.to_string());
            Response::builder()
                .status(status)
                .body(axum::body::Body::from("Too many requests"))
                .unwrap()
        }
    }
}

/// Request size limiting middleware
pub async fn request_size_middleware(request: Request, next: Next) -> Response {
    let size_limiter = request
        .extensions()
        .get::<Arc<RequestSizeLimiter>>()
        .cloned()
        .unwrap_or_else(|| Arc::new(RequestSizeLimiter::default()));

    let headers = request.headers();
    let uri = request.uri();
    let url_length = uri.to_string().len();

    // Get body size (approximate - actual body reading happens later)
    // For now, check Content-Length header
    let body_size = headers
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    match size_limiter.validate_request(headers, body_size, url_length) {
        Ok(_) => next.run(request).await,
        Err(status) => {
            let ip = extract_ip(&request);
            SecurityLogger::log_event(
                "REQUEST_SIZE_LIMIT",
                &format!("Body: {} bytes, URL: {} chars", body_size, url_length),
                Some(&ip.to_string()),
            );
            Response::builder()
                .status(status)
                .body(axum::body::Body::from("Request too large"))
                .unwrap()
        }
    }
}

/// Authentication middleware (optional - can be applied to specific routes)
pub async fn auth_middleware(request: Request, next: Next) -> Response {
    let headers = request.headers();

    // Extract token
    if let Some(token) = AuthValidator::extract_token(headers) {
        // Create validator instance and validate token
        let validator = AuthValidator::default();
        match validator.validate_api_key(&token) {
            Ok(claims) => {
                let mut request = request;
                request.extensions_mut().insert(claims);
                next.run(request).await
            }
            Err(e) => {
                let ip = extract_ip(&request);
                SecurityLogger::log_auth_failure(&ip.to_string(), &e);
                Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(axum::body::Body::from("Unauthorized"))
                    .unwrap()
            }
        }
    } else {
        // No token provided
        let ip = extract_ip(&request);
        SecurityLogger::log_auth_failure(&ip.to_string(), "Missing Authorization header");
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(axum::body::Body::from("Unauthorized"))
            .unwrap()
    }
}

/// Optional HTTP Basic Auth for `dal serve` when **`DAL_HTTP_USER`** is set and either
/// **`DAL_HTTP_PASSWORD_HASH`** (bcrypt, preferred for internet-facing) or **`DAL_HTTP_PASSWORD`**
/// (plaintext, dev only) is set. If **`DAL_HTTP_PASSWORD_HASH`** is set, it is the only verifier
/// (bcrypt **`$2a$` / `$2b$`**).
///
/// Brute-force: see **`DalServeBasicAuthBruteForce`** (`DAL_HTTP_AUTH_MAX_FAILS_PER_IP`, etc.).
///
/// Exempt paths: built-in list always includes `/health`, `/metrics`, and PWA shell paths
/// (`/manifest.webmanifest`, favicons, `/sw.js`). **`DAL_HTTP_AUTH_EXEMPT`** adds more paths
/// (comma-separated); it does **not** remove the built-ins, so a line like `/health,/metrics` in
/// `.env` no longer drops manifest exemption.
/// **`OPTIONS`** requests pass through (CORS preflight).
///
/// **Internet / 24×7:** put TLS in front (reverse proxy or tunnel); Basic sends credentials
/// base64-encoded; HTTPS is required on untrusted networks.
///
/// Browsers prompt once per origin; scripts use `curl -u user:pass` or Basic in `fetch`.
pub async fn dal_serve_basic_auth_middleware(request: Request, next: Next) -> Response {
    let user = match std::env::var("DAL_HTTP_USER") {
        Ok(s) if !s.is_empty() => s,
        _ => return next.run(request).await,
    };
    let pass_hash = std::env::var("DAL_HTTP_PASSWORD_HASH")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let pass_plain = std::env::var("DAL_HTTP_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty());
    if pass_hash.is_none() && pass_plain.is_none() {
        return next.run(request).await;
    }

    if request.method() == Method::OPTIONS {
        return next.run(request).await;
    }

    let path = request.uri().path().to_string();
    if dal_serve_basic_auth_exempt_path(&path) {
        return next.run(request).await;
    }

    let ip = extract_ip(&request);
    if DAL_SERVE_BASIC_AUTH_BF.is_locked_out(ip).await {
        return dal_serve_basic_auth_too_many();
    }

    let headers = request.headers();
    let auth_hdr = match headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        Some(h) => h,
        None => {
            DAL_SERVE_BASIC_AUTH_BF.record_failure(ip).await;
            return dal_serve_basic_auth_unauthorized();
        }
    };
    let rest = match auth_hdr.strip_prefix("Basic ") {
        Some(r) => r.trim(),
        None => {
            DAL_SERVE_BASIC_AUTH_BF.record_failure(ip).await;
            return dal_serve_basic_auth_unauthorized();
        }
    };
    let decoded = match B64_ENGINE.decode(rest) {
        Ok(b) => b,
        Err(_) => {
            DAL_SERVE_BASIC_AUTH_BF.record_failure(ip).await;
            return dal_serve_basic_auth_unauthorized();
        }
    };
    let combined = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => {
            DAL_SERVE_BASIC_AUTH_BF.record_failure(ip).await;
            return dal_serve_basic_auth_unauthorized();
        }
    };
    let (given_user, given_pass) = match combined.split_once(':') {
        Some((u, p)) => (u, p),
        None => {
            DAL_SERVE_BASIC_AUTH_BF.record_failure(ip).await;
            return dal_serve_basic_auth_unauthorized();
        }
    };

    if !ct_eq_bytes(given_user.as_bytes(), user.as_bytes()) {
        DAL_SERVE_BASIC_AUTH_BF.record_failure(ip).await;
        return dal_serve_basic_auth_unauthorized();
    }
    if !dal_serve_basic_auth_password_ok(given_pass, &pass_hash, &pass_plain) {
        DAL_SERVE_BASIC_AUTH_BF.record_failure(ip).await;
        return dal_serve_basic_auth_unauthorized();
    }

    DAL_SERVE_BASIC_AUTH_BF.clear(ip).await;
    next.run(request).await
}

fn dal_serve_basic_auth_password_ok(
    given: &str,
    hash: &Option<String>,
    plain: &Option<String>,
) -> bool {
    if let Some(h) = hash {
        let t = h.trim();
        if !t.is_empty() {
            return bcrypt::verify(given, t).unwrap_or(false);
        }
    }
    if let Some(p) = plain {
        return ct_eq_bytes(given.as_bytes(), p.as_bytes());
    }
    false
}

fn dal_serve_basic_auth_exempt_path(path: &str) -> bool {
    // Automation POSTs use JSON `token` (DAL_COO_API_TOKEN) in server.dal — not Basic. Scripts cannot
    // send bcrypt; curl would need a plaintext Basic password. Exempt these routes from Basic so
    // launchd_wake.sh / wake_poll.sh work; token check in DAL still applies.
    const DEFAULT_EXEMPT: &str = "/health,/metrics,/manifest.webmanifest,/favicon.svg,/favicon.ico,/sw.js,/api/wake,/api/tasks/run-due";
    let extra = std::env::var("DAL_HTTP_AUTH_EXEMPT").unwrap_or_default();
    // Merge defaults with optional env (env used to replace defaults and broke PWA fetches).
    for raw in [DEFAULT_EXEMPT, extra.as_str()] {
        for part in raw.split(',') {
            let p = part.trim();
            if !p.is_empty() && path == p {
                return true;
            }
        }
    }
    false
}

fn ct_eq_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn dal_serve_basic_auth_unauthorized() -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, r#"Basic realm="dal serve""#)
        .body(Body::from("Unauthorized"))
        .unwrap()
}

fn dal_serve_basic_auth_too_many() -> Response {
    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .body(Body::from("Too many failed authentication attempts"))
        .unwrap()
}

/// Authentication middleware with route/method exemptions.
pub async fn auth_middleware_with_exemptions(request: Request, next: Next) -> Response {
    if request.method() == Method::OPTIONS {
        return next.run(request).await;
    }
    if let Some(paths) = request.extensions().get::<AuthExemptPaths>() {
        if paths.0.contains(request.uri().path()) {
            return next.run(request).await;
        }
    }
    auth_middleware(request, next).await
}

/// Input validation middleware (validates query params and headers)
pub async fn input_validation_middleware(request: Request, next: Next) -> Response {
    let headers = request.headers();
    let uri = request.uri();
    let ip = extract_ip(&request);

    // Validate query parameters
    if let Some(query) = uri.query() {
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                // Validate key and value
                if let Err(e) = InputValidator::validate_string(key, 100) {
                    SecurityLogger::log_invalid_input(
                        &ip.to_string(),
                        &format!("Query key: {}", e),
                    );
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(axum::body::Body::from("Invalid query parameter"))
                        .unwrap();
                }

                if let Err(e) = InputValidator::validate_string(value, 1000) {
                    SecurityLogger::log_invalid_input(
                        &ip.to_string(),
                        &format!("Query value: {}", e),
                    );
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(axum::body::Body::from("Invalid query parameter"))
                        .unwrap();
                }
            }
        }
    }

    // Validate header values (basic check)
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            if let Err(e) = InputValidator::validate_string(value_str, 1000) {
                SecurityLogger::log_invalid_input(
                    &ip.to_string(),
                    &format!("Header {}: {}", name, e),
                );
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(axum::body::Body::from("Invalid header"))
                    .unwrap();
            }
        }
    }

    next.run(request).await
}

/// Input validation middleware with route/method exemptions.
pub async fn input_validation_middleware_with_exemptions(request: Request, next: Next) -> Response {
    if request.method() == Method::OPTIONS {
        return next.run(request).await;
    }
    if let Some(paths) = request.extensions().get::<InputValidationExemptPaths>() {
        if paths.0.contains(request.uri().path()) {
            return next.run(request).await;
        }
    }
    input_validation_middleware(request, next).await
}

/// Extract IP address from request
fn extract_ip(request: &Request) -> IpAddr {
    // Try to get from X-Forwarded-For header (for proxies)
    if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip_str) = forwarded_str.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    // Try to get from X-Real-IP header
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // Fallback to localhost (for testing)
    // In production, this should come from connection info
    "127.0.0.1".parse().unwrap()
}

// Note: Combined security middleware is not needed since we apply layers separately
// Each middleware handles its own errors and returns appropriate responses

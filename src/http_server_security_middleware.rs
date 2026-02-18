// HTTP Server Security Middleware
// Axum middleware wrappers for security controls

use crate::http_server_security::{
    AuthValidator, InputValidator, RateLimiter, RequestSizeLimiter, SecurityLogger,
};
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use std::net::IpAddr;
use std::sync::Arc;

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
            Ok(_claims) => {
                // Token is valid with claims, continue
                // TODO: Can attach claims to request extensions
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

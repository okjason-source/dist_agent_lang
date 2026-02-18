// HTTP Server Request/Response Converters
// Converts between Axum HTTP types and dist_agent_lang types

use axum::{
    extract::Request,
    http::StatusCode,
    response::Response,
    body::Body,
};
use crate::stdlib::web::{HttpRequest, HttpResponse};
use std::collections::HashMap;

/// Simple percent decoding (for query parameters)
fn percent_decode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let mut hex = String::new();
            if let Some(c1) = chars.next() {
                hex.push(c1);
                if let Some(c2) = chars.next() {
                    hex.push(c2);
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                        continue;
                    }
                }
            }
            // If decoding failed, keep the %
            result.push('%');
            result.push_str(&hex);
        } else if ch == '+' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Convert Axum Request to HttpRequest
pub async fn axum_request_to_http_request(mut request: Request) -> HttpRequest {
    let method = request.method().to_string();
    let uri = request.uri();
    let path = uri.path().to_string();
    
    // Extract headers
    let mut headers = HashMap::new();
    for (name, value) in request.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }
    
    // Extract query parameters
    let mut query_params = HashMap::new();
    if let Some(query) = uri.query() {
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                // Simple URL decoding (percent-encoded)
                let decoded_key = percent_decode(key);
                let decoded_value = percent_decode(value);
                query_params.insert(decoded_key, decoded_value);
            }
        }
    }
    
    // Extract path parameters (would be set by route extractors)
    let path_params = HashMap::new();
    
    // Extract cookies
    let mut cookies = HashMap::new();
    if let Some(cookie_header) = request.headers().get("Cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                if let Some((key, value)) = cookie.trim().split_once('=') {
                    cookies.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }
    
    // Extract body (using axum's body extractor pattern)
    let body = extract_body(&mut request).await.unwrap_or_default();
    
    // Session and user are empty initially (set by middleware)
    let session = HashMap::new();
    let user = None;
    
    HttpRequest {
        method,
        path,
        headers,
        body,
        query_params,
        path_params,
        cookies,
        session,
        user,
    }
}

/// Max body size when reading request body (10MB). Security middleware may reject earlier via Content-Length.
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Extract body from Axum Request
async fn extract_body(request: &mut Request) -> Result<String, String> {
    use axum::body::{to_bytes, Body};

    let body = std::mem::replace(request.body_mut(), Body::empty());
    let bytes = to_bytes(body, MAX_BODY_SIZE)
        .await
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// Convert HttpResponse to Axum Response
pub fn http_response_to_axum_response(response: HttpResponse) -> Response<Body> {
    let status = StatusCode::from_u16(response.status as u16)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    
    // Build response with status
    let mut axum_response = Response::builder()
        .status(status);
    
    // Add headers
    for (key, value) in &response.headers {
        if let (Ok(header_name), Ok(header_value)) = (
            axum::http::HeaderName::from_bytes(key.as_bytes()),
            axum::http::HeaderValue::from_str(value),
        ) {
            axum_response = axum_response.header(header_name, header_value);
        }
    }
    
    // Add cookies
    for cookie in &response.cookies {
        let cookie_value = format!(
            "{}={}; Path={}{}{}",
            cookie.name,
            cookie.value,
            cookie.path,
            if let Some(ref domain) = cookie.domain {
                format!("; Domain={}", domain)
            } else {
                String::new()
            },
            if cookie.secure { "; Secure" } else { "" },
        );
        
        if let Ok(header_value) = axum::http::HeaderValue::from_str(&cookie_value) {
            axum_response = axum_response.header("Set-Cookie", header_value);
        }
    }
    
    // Handle redirect
    if let Some(ref redirect_url) = response.redirect_url {
        if let Ok(header_value) = axum::http::HeaderValue::from_str(redirect_url) {
            axum_response = axum_response.header("Location", header_value);
        }
    }
    
    // Set body
    axum_response
        .body(Body::from(response.body))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal server error"))
                .unwrap()
        })
}

/// Create error response
pub fn error_response(status: u16, message: &str) -> Response<Body> {
    let status_code = StatusCode::from_u16(status)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(format!(
            r#"{{"error": "{}", "status": {}}}"#,
            message, status
        )))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal server error"))
                .unwrap()
        })
}

/// Create JSON response
pub fn json_response(data: serde_json::Value) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&data).unwrap_or_default()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal server error"))
                .unwrap()
        })
}


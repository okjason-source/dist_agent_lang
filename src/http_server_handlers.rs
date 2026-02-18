// HTTP Server Handlers with Middleware Support
// Generic handler that executes middleware chain and route handlers

use axum::{
    extract::{Request, State},
    response::Response,
    body::Body,
};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::http_server::WebServerState;
use crate::runtime::engine::Runtime;
use crate::http_server_middleware::{execute_middleware_chain, execute_route_handler};
use crate::http_server_converters::{axum_request_to_http_request, http_response_to_axum_response, error_response};

/// Generic handler that executes middleware and route handler.
/// Uses spawn_blocking for DAL execution so the async runtime stays responsive.
pub async fn handle_with_middleware(
    State(state): State<Arc<RwLock<WebServerState>>>,
    request: Request,
    handler_name: &str,
) -> Response<Body> {
    // Convert Axum request to HttpRequest (async body extraction)
    let mut http_request = axum_request_to_http_request(request).await;

    // Get server state and populate path_params when route uses a pattern (e.g. /users/:id)
    let state_guard = state.read().await;
    if let Some((_, params)) = get_route_handler_name_and_params(
        &state_guard.server,
        &http_request.method,
        &http_request.path,
    ) {
        http_request.path_params = params;
    }
    let middleware_list = state_guard.server.middleware.clone();
    let handler_name = handler_name.to_string();

    // Clone what we need for spawn_blocking (Runtime is not Send, so we run DAL entirely in blocking task)
    let runtime_factory = state_guard.runtime_factory.clone();
    let scope_writeback = state_guard.scope_writeback.clone();
    drop(state_guard);

    // Run DAL execution on threadpool - keeps async runtime responsive
    let response = tokio::task::spawn_blocking(move || {
        let mut runtime = if let Some(ref factory) = runtime_factory {
            (factory.as_ref().deref())()
        } else {
            Runtime::new()
        };

        // Execute middleware chain
        let processed_request = match execute_middleware_chain(
            &mut runtime,
            &middleware_list,
            http_request,
        ) {
            Ok(req) => req,
            Err(e) => {
                return Err(format!("Middleware error: {}", e));
            }
        };

        // Execute route handler
        let result = execute_route_handler(&mut runtime, &handler_name, processed_request)
            .map_err(|e| format!("Handler error: {}", e));

        // Persist scope changes (shared state across requests)
        if let Some(ref writeback) = scope_writeback {
            writeback(&runtime.scope);
        }

        result
    })
    .await
    .map_err(|e| format!("Blocking task join error: {}", e));

    match response {
        Ok(Ok(resp)) => http_response_to_axum_response(resp),
        Ok(Err(e)) => error_response(500, &e),
        Err(e) => error_response(500, &e),
    }
}

/// Check if a route pattern (e.g. "/users/:id") matches a request path (e.g. "/users/123").
fn path_pattern_matches(pattern: &str, path: &str) -> bool {
    let pattern_segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if pattern_segments.len() != path_segments.len() {
        return false;
    }
    for (p, v) in pattern_segments.iter().zip(path_segments.iter()) {
        if !(*p == *v || (p.starts_with(':') && !v.is_empty())) {
            return false;
        }
    }
    true
}

/// Extract path params from a request path using a route pattern (e.g. "/users/:id" and "/users/123" -> {"id": "123"}).
fn path_params_from_match(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if pattern_segments.len() != path_segments.len() {
        return None;
    }
    let mut params = HashMap::new();
    for (p, v) in pattern_segments.iter().zip(path_segments.iter()) {
        if p.starts_with(':') && !v.is_empty() {
            let param_name = p.trim_start_matches(':');
            params.insert(param_name.to_string(), (*v).to_string());
        } else if *p != *v {
            return None;
        }
    }
    Some(params)
}

/// Get route handler name and path params from method and path. Supports exact match and path-param patterns.
pub fn get_route_handler_name_and_params(
    server: &crate::stdlib::web::HttpServer,
    method: &str,
    path: &str,
) -> Option<(String, HashMap<String, String>)> {
    let method_upper = method.to_uppercase();
    let route_key = format!("{}:{}", method_upper, path);

    // Exact match: no path params
    if let Some(route) = server.routes.get(&route_key) {
        return Some((route.handler.clone(), HashMap::new()));
    }

    // Match path patterns (e.g. GET:/users/:id for path /users/123)
    for (key, route) in &server.routes {
        if let Some((route_method, route_path)) = key.split_once(':') {
            if route_method.eq_ignore_ascii_case(method)
                && path_pattern_matches(route_path, path)
            {
                let params = path_params_from_match(route_path, path).unwrap_or_default();
                return Some((route.handler.clone(), params));
            }
        }
    }
    None
}

/// Get route handler name from path and method. Supports exact match and path-param patterns (e.g. "/users/:id").
pub fn get_route_handler_name(
    server: &crate::stdlib::web::HttpServer,
    method: &str,
    path: &str,
) -> Option<String> {
    get_route_handler_name_and_params(server, method, path).map(|(name, _)| name)
}


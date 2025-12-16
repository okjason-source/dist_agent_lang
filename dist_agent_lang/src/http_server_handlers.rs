// HTTP Server Handlers with Middleware Support
// Generic handler that executes middleware chain and route handlers

use axum::{
    extract::{Request, State},
    response::Response,
    body::Body,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::http_server::WebServerState;
use crate::runtime::engine::Runtime;
use crate::http_server_middleware::{execute_middleware_chain, execute_route_handler, MiddlewareError};
use crate::http_server_converters::{axum_request_to_http_request, http_response_to_axum_response, error_response};
use crate::stdlib::web::HttpRequest;

/// Generic handler that executes middleware and route handler
pub async fn handle_with_middleware(
    State(state): State<Arc<RwLock<WebServerState>>>,
    request: Request,
    handler_name: &str,
) -> Response<Body> {
    // Convert Axum request to HttpRequest
    let http_request = axum_request_to_http_request(request).await;
    
    // Get server state
    let state_guard = state.read().await;
    let middleware_list = &state_guard.server.middleware;
    
    // Create new runtime instance (Runtime is not thread-safe)
    let mut runtime = if let Some(ref factory) = state_guard.runtime_factory {
        factory()
    } else {
        Runtime::new()
    };
    
    // Execute middleware chain
    let processed_request = match execute_middleware_chain(
        &mut runtime,
        middleware_list,
        http_request,
    ) {
        Ok(req) => req,
        Err(e) => {
            // Middleware error - reject request
            return error_response(403, &format!("Middleware error: {}", e));
        }
    };
    
    // Execute route handler
    let response = match execute_route_handler(
        &mut runtime,
        handler_name,
        processed_request,
    ) {
        Ok(resp) => resp,
        Err(e) => {
            // Handler error - return 500
            return error_response(500, &format!("Handler error: {}", e));
        }
    };
    
    // Convert HttpResponse to Axum Response
    http_response_to_axum_response(response)
}

/// Get route handler name from path and method
pub fn get_route_handler_name(
    server: &crate::stdlib::web::HttpServer,
    method: &str,
    path: &str,
) -> Option<String> {
    // Look up route in server.routes
    // Format: "METHOD:/path" -> handler name
    let route_key = format!("{}:{}", method, path);
    
    // Try exact match first
    if let Some(route) = server.routes.get(&route_key) {
        return Some(route.handler.clone());
    }
    
    // Try to match with path parameters (simplified - would need proper routing)
    // For now, return None if not found
    None
}


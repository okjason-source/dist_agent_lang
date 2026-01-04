// HTTP Server Integration
// Integrates middleware and handlers into the HTTP server router

use axum::{
    extract::{Request, State},
    response::Response,
    body::Body,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::http_server::WebServerState;
use crate::http_server_handlers::handle_with_middleware;
use crate::http_server_converters::error_response;
use crate::stdlib::web::HttpServer;

/// Create router with middleware support
pub fn create_router_with_middleware(server: HttpServer) -> Router<Arc<RwLock<WebServerState>>> {
    let runtime_factory: Box<dyn Fn() -> crate::runtime::engine::Runtime + Send + Sync> = 
        Box::new(|| crate::runtime::engine::Runtime::new());
    
    let state = Arc::new(RwLock::new(WebServerState {
        server: server.clone(),
        handlers: std::collections::HashMap::new(),
        runtime_factory: Some(runtime_factory),
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
                async move {
                    handle_with_middleware(
                        State(state),
                        request,
                        &handler_name,
                    ).await
                }
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


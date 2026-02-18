// HTTP Server Middleware Execution
// Bridges Rust HTTP server with dist_agent_lang middleware handlers

use crate::runtime::engine::Runtime;
use crate::runtime::functions::RuntimeError;
use crate::runtime::values::Value;
use crate::stdlib::web::{HttpRequest, HttpResponse, Middleware};
use std::collections::HashMap;

/// Execute middleware chain for a request
pub fn execute_middleware_chain(
    runtime: &mut Runtime,
    middleware_list: &[Middleware],
    request: HttpRequest,
) -> Result<HttpRequest, MiddlewareError> {
    let mut current_request = request;

    // Execute middleware in priority order (already sorted)
    for middleware in middleware_list {
        current_request = execute_middleware(runtime, middleware, current_request)?;
    }

    Ok(current_request)
}

/// Execute a single middleware handler
fn execute_middleware(
    runtime: &mut Runtime,
    middleware: &Middleware,
    request: HttpRequest,
) -> Result<HttpRequest, MiddlewareError> {
    // Convert HttpRequest to Value for runtime call
    let request_value = http_request_to_value(&request);

    // Call the handler function by name
    let args = vec![request_value];

    match runtime.call_function(&middleware.handler, &args) {
        Ok(result) => {
            // Convert result back to HttpRequest
            value_to_http_request(result).map_err(|e| {
                MiddlewareError::ExecutionError(format!("Failed to convert response: {}", e))
            })
        }
        Err(e) => {
            // If function not found, try as service method
            if let RuntimeError::FunctionNotFound(_) = e {
                // Try calling as service method: service_name::handler_name
                // For now, we'll return an error - this needs service context
                Err(MiddlewareError::HandlerNotFound(middleware.handler.clone()))
            } else {
                Err(MiddlewareError::ExecutionError(format!("{}", e)))
            }
        }
    }
}

/// Convert HttpRequest to Value for runtime
fn http_request_to_value(request: &HttpRequest) -> Value {
    let mut map = HashMap::new();

    map.insert("method".to_string(), Value::String(request.method.clone()));
    map.insert("path".to_string(), Value::String(request.path.clone()));

    // Convert headers
    let mut headers_map = HashMap::new();
    for (k, v) in &request.headers {
        headers_map.insert(k.clone(), Value::String(v.clone()));
    }
    map.insert("headers".to_string(), Value::Map(headers_map));

    map.insert("body".to_string(), Value::String(request.body.clone()));

    // Convert query params
    let mut query_map = HashMap::new();
    for (k, v) in &request.query_params {
        query_map.insert(k.clone(), Value::String(v.clone()));
    }
    map.insert("query_params".to_string(), Value::Map(query_map));

    // Convert path params
    let mut path_map = HashMap::new();
    for (k, v) in &request.path_params {
        path_map.insert(k.clone(), Value::String(v.clone()));
    }
    map.insert("path_params".to_string(), Value::Map(path_map));

    // Convert cookies
    let mut cookies_map = HashMap::new();
    for (k, v) in &request.cookies {
        cookies_map.insert(k.clone(), Value::String(v.clone()));
    }
    map.insert("cookies".to_string(), Value::Map(cookies_map));

    // Convert session (already HashMap<String, Value>, wrap in Value::Map)
    map.insert("session".to_string(), Value::Map(request.session.clone()));

    // User (if present)
    if let Some(ref user) = request.user {
        // Convert user to Value (simplified)
        let mut user_map = HashMap::new();
        user_map.insert("id".to_string(), Value::String(user.id.clone()));
        user_map.insert("email".to_string(), Value::String(user.email.clone()));
        map.insert("user".to_string(), Value::Map(user_map));
    } else {
        map.insert("user".to_string(), Value::Null);
    }

    Value::Map(map)
}

/// Convert Value back to HttpRequest. Public for mutation testing (Value::Map match arm).
pub fn value_to_http_request(value: Value) -> Result<HttpRequest, String> {
    match value {
        Value::Map(map) => {
            let method = map
                .get("method")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| "Missing method".to_string())?;

            let path = map
                .get("path")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| "Missing path".to_string())?;

            // Extract headers
            let headers = map
                .get("headers")
                .and_then(|v| {
                    if let Value::Map(m) = v {
                        let mut h = HashMap::new();
                        for (k, v) in m {
                            if let Value::String(s) = v {
                                h.insert(k.clone(), s.clone());
                            }
                        }
                        Some(h)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let body = map
                .get("body")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            // Extract query params
            let query_params = map
                .get("query_params")
                .and_then(|v| {
                    if let Value::Map(m) = v {
                        let mut q = HashMap::new();
                        for (k, v) in m {
                            if let Value::String(s) = v {
                                q.insert(k.clone(), s.clone());
                            }
                        }
                        Some(q)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            // Extract path params
            let path_params = map
                .get("path_params")
                .and_then(|v| {
                    if let Value::Map(m) = v {
                        let mut p = HashMap::new();
                        for (k, v) in m {
                            if let Value::String(s) = v {
                                p.insert(k.clone(), s.clone());
                            }
                        }
                        Some(p)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            // Extract cookies
            let cookies = map
                .get("cookies")
                .and_then(|v| {
                    if let Value::Map(m) = v {
                        let mut c = HashMap::new();
                        for (k, v) in m {
                            if let Value::String(s) = v {
                                c.insert(k.clone(), s.clone());
                            }
                        }
                        Some(c)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            // Extract session
            let session = map
                .get("session")
                .and_then(|v| {
                    if let Value::Map(m) = v {
                        Some(m.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            // Extract user (simplified)
            let user = map.get("user").and_then(|v| {
                if let Value::Map(m) = v {
                    let id = m
                        .get("id")
                        .and_then(|v| {
                            if let Value::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    let username = m
                        .get("username")
                        .and_then(|v| {
                            if let Value::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    let email = m
                        .get("email")
                        .and_then(|v| {
                            if let Value::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    let roles = m
                        .get("roles")
                        .and_then(|v| {
                            if let Value::List(l) = v {
                                Some(
                                    l.iter()
                                        .filter_map(|v| {
                                            if let Value::String(s) = v {
                                                Some(s.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect(),
                                )
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    Some(crate::stdlib::web::User {
                        id,
                        username,
                        email,
                        roles,
                    })
                } else {
                    None
                }
            });

            Ok(HttpRequest {
                method,
                path,
                headers,
                body,
                query_params,
                path_params,
                cookies,
                session,
                user,
            })
        }
        _ => Err("Expected Map value".to_string()),
    }
}

/// Middleware execution errors
#[derive(Debug)]
pub enum MiddlewareError {
    HandlerNotFound(String),
    ExecutionError(String),
    ConversionError(String),
}

impl std::fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MiddlewareError::HandlerNotFound(name) => {
                write!(f, "Middleware handler not found: {}", name)
            }
            MiddlewareError::ExecutionError(msg) => {
                write!(f, "Middleware execution error: {}", msg)
            }
            MiddlewareError::ConversionError(msg) => write!(f, "Value conversion error: {}", msg),
        }
    }
}

impl std::error::Error for MiddlewareError {}

/// Execute route handler
pub fn execute_route_handler(
    runtime: &mut Runtime,
    handler_name: &str,
    request: HttpRequest,
) -> Result<HttpResponse, MiddlewareError> {
    // Convert request to Value
    let request_value = http_request_to_value(&request);
    let args = vec![request_value];

    // Call handler function
    match runtime.call_function(handler_name, &args) {
        Ok(result) => {
            // Convert result to HttpResponse
            value_to_http_response(result).map_err(|e| {
                MiddlewareError::ExecutionError(format!("Failed to convert response: {}", e))
            })
        }
        Err(e) => Err(MiddlewareError::ExecutionError(format!("{}", e))),
    }
}

/// Convert Value to HttpResponse. Public for mutation testing (Value::Map and Value::String match arms).
pub fn value_to_http_response(value: Value) -> Result<HttpResponse, String> {
    match value {
        Value::Map(map) => {
            let status = map
                .get("status")
                .and_then(|v| {
                    if let Value::Int(i) = v {
                        Some(*i)
                    } else {
                        None
                    }
                })
                .unwrap_or(200);

            // Extract headers
            let headers = map
                .get("headers")
                .and_then(|v| {
                    if let Value::Map(m) = v {
                        let mut h = HashMap::new();
                        for (k, v) in m {
                            if let Value::String(s) = v {
                                h.insert(k.clone(), s.clone());
                            }
                        }
                        Some(h)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let body = map
                .get("body")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            // Extract cookies (simplified - would need proper Cookie struct)
            let cookies = vec![];

            // Extract redirect_url
            let redirect_url = map.get("redirect_url").and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            });

            Ok(HttpResponse {
                status,
                headers,
                body,
                cookies,
                redirect_url,
            })
        }
        Value::String(s) => {
            // If just a string, treat as body with 200 status
            Ok(HttpResponse {
                status: 200,
                headers: HashMap::new(),
                body: s,
                cookies: vec![],
                redirect_url: None,
            })
        }
        _ => Err("Invalid response type".to_string()),
    }
}

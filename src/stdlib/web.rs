use crate::runtime::values::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// Web Development Framework - Phase 2
// Comprehensive web development capabilities including:
// - HTTP Server with routing
// - Frontend framework with HTML generation
// - RESTful API framework
// - Middleware system
// - WebSocket support

// === ENHANCED HTTP SERVER FRAMEWORK ===

#[derive(Debug, Clone)]
pub struct HttpServer {
    pub port: i64,
    pub routes: HashMap<String, Route>, // path -> route
    pub middleware: Vec<Middleware>,
    pub static_files: HashMap<String, String>, // path -> file content
    pub config: ServerConfig,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub method: HttpMethod,
    pub path: String,
    pub handler: String,         // function name
    pub middleware: Vec<String>, // middleware names
}

#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
}

#[derive(Debug, Clone)]
pub struct Middleware {
    pub name: String,
    pub handler: String, // function name
    pub priority: i64,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub max_connections: i64,
    pub timeout_seconds: i64,
    pub cors_enabled: bool,
    pub ssl_enabled: bool,
    pub static_path: String,
}

// === ENHANCED HTTP STRUCTURES ===

#[derive(Debug, Clone)]
pub struct HttpClient {
    pub base_url: String,
    pub headers: HashMap<String, String>,
    pub timeout: i64,
    pub retry_count: i64,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub query_params: HashMap<String, String>,
    pub path_params: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    pub session: HashMap<String, Value>,
    pub user: Option<User>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: i64,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub cookies: Vec<Cookie>,
    pub redirect_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: String,
    pub domain: Option<String>,
    pub expires: Option<String>,
    pub secure: bool,
    pub http_only: bool,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

// === FRONTEND FRAMEWORK ===

#[derive(Debug, Clone)]
pub struct HtmlElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<HtmlElement>,
    pub text: Option<String>,
    pub event_handlers: HashMap<String, String>, // event -> handler
}

#[derive(Debug, Clone)]
pub struct HtmlPage {
    pub title: String,
    pub meta: HashMap<String, String>,
    pub styles: Vec<String>,  // CSS files/links
    pub scripts: Vec<String>, // JS files/links
    pub head_elements: Vec<HtmlElement>,
    pub body: HtmlElement,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub content: String,
    pub variables: HashMap<String, Value>,
    pub includes: Vec<String>, // other template names
}

// === API FRAMEWORK ===

#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    pub path: String,
    pub method: HttpMethod,
    pub handler: String,
    pub input_schema: JsonSchema,
    pub output_schema: JsonSchema,
    pub auth_required: bool,
    pub rate_limit: Option<RateLimit>,
}

#[derive(Debug, Clone)]
pub struct JsonSchema {
    pub schema_type: String, // "object", "array", etc.
    pub properties: HashMap<String, PropertySchema>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PropertySchema {
    pub property_type: String, // "string", "number", etc.
    pub description: String,
    pub validation: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub rule_type: String, // "min", "max", "pattern", etc.
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_minute: i64,
    pub burst_limit: i64,
}

// === WEBSOCKET SUPPORT ===

#[derive(Debug, Clone)]
pub struct WebSocketServer {
    pub port: i64,
    pub connections: HashMap<String, WebSocketConnection>, // connection_id -> connection
    pub rooms: HashMap<String, Vec<String>>,               // room_name -> connection_ids
}

#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub user_id: Option<String>,
    pub rooms: Vec<String>,
    pub last_ping: String, // timestamp
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct WebSocketMessage {
    pub message_type: String, // "text", "binary", "ping", "pong"
    pub data: String,
    pub from: String,       // connection_id
    pub to: Option<String>, // connection_id or room_name
    pub timestamp: String,
}

// === ENHANCED HTTP SERVER FUNCTIONS ===

pub fn create_server(port: i64) -> HttpServer {
    let message = format!("Creating enhanced HTTP server on port {}", port);
    crate::stdlib::log::info(
        &message,
        {
            let mut data = std::collections::HashMap::new();
            data.insert("port".to_string(), Value::Int(port));
            data.insert("message".to_string(), Value::String(message.clone()));
            data
        },
        Some("web"),
    );

    HttpServer {
        port,
        routes: HashMap::new(),
        middleware: Vec::new(),
        static_files: HashMap::new(),
        config: ServerConfig {
            max_connections: 1000,
            timeout_seconds: 30,
            cors_enabled: true,
            ssl_enabled: false,
            static_path: "./static".to_string(),
        },
    }
}

pub fn add_route(server: &mut HttpServer, method: String, path: String, handler: String) {
    let http_method = match method.to_uppercase().as_str() {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PUT" => HttpMethod::PUT,
        "DELETE" => HttpMethod::DELETE,
        "PATCH" => HttpMethod::PATCH,
        "OPTIONS" => HttpMethod::OPTIONS,
        "HEAD" => HttpMethod::HEAD,
        _ => HttpMethod::GET, // Default
    };

    let route = Route {
        method: http_method,
        path: path.clone(),
        handler,
        middleware: Vec::new(),
    };

    // Use "METHOD:/path" format as key to match create_router_with_middleware expectations
    let route_key = format!("{}:{}", method.to_uppercase(), path);
    let route_path = route.path.clone(); // Store path before move
    server.routes.insert(route_key, route);

    crate::stdlib::log::info(
        "Route added",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("method".to_string(), Value::String(method));
            data.insert("path".to_string(), Value::String(route_path));
            data.insert(
                "message".to_string(),
                Value::String("Route added".to_string()),
            );
            data
        },
        Some("web"),
    );
}

pub fn add_middleware(server: &mut HttpServer, name: String, handler: String, priority: i64) {
    let middleware = Middleware {
        name: name.clone(),
        handler,
        priority,
    };

    server.middleware.push(middleware);
    server
        .middleware
        .sort_by(|a, b| a.priority.cmp(&b.priority));

    crate::stdlib::log::info(
        "Middleware added",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("middleware".to_string(), Value::String(name));
            data.insert("priority".to_string(), Value::Int(priority));
            data.insert(
                "message".to_string(),
                Value::String("Middleware added".to_string()),
            );
            data
        },
        Some("web"),
    );
}

pub fn configure_cors(server: &mut HttpServer, enabled: bool, origins: Vec<String>) {
    server.config.cors_enabled = enabled;

    crate::stdlib::log::info(
        "CORS configured",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("cors_enabled".to_string(), Value::Bool(enabled));
            data.insert(
                "origins_count".to_string(),
                Value::Int(origins.len() as i64),
            );
            data.insert(
                "message".to_string(),
                Value::String("CORS configured".to_string()),
            );
            data
        },
        Some("web"),
    );
}

pub fn serve_static_files(server: &mut HttpServer, path: String, content: String) {
    server.static_files.insert(path.clone(), content);

    crate::stdlib::log::info(
        "Static file added",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("path".to_string(), Value::String(path));
            data.insert(
                "message".to_string(),
                Value::String("Static file added".to_string()),
            );
            data
        },
        Some("web"),
    );
}

pub fn start_server(server: &HttpServer) -> Result<String, String> {
    crate::stdlib::log::info(
        "Starting HTTP server",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("port".to_string(), Value::Int(server.port));
            data.insert(
                "routes_count".to_string(),
                Value::Int(server.routes.len() as i64),
            );
            data.insert(
                "middleware_count".to_string(),
                Value::Int(server.middleware.len() as i64),
            );
            data.insert(
                "message".to_string(),
                Value::String("Starting HTTP server".to_string()),
            );
            data
        },
        Some("web"),
    );

    // Start actual HTTP server
    let server_clone = server.clone();
    tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create runtime: {}", e))?
        .block_on(async {
            crate::http_server::start_http_server(server_clone)
                .await
                .map_err(|e| format!("Failed to start server: {}", e))
        })
        .map(|_| {
            format!(
                "Server started on port {} with {} routes",
                server.port,
                server.routes.len()
            )
        })
}

pub fn create_client(base_url: String) -> HttpClient {
    crate::stdlib::log::info(
        "web",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("base_url".to_string(), Value::String(base_url.clone()));
            data.insert(
                "message".to_string(),
                Value::String(format!("Creating enhanced HTTP client for {}", base_url)),
            );
            data
        },
        Some("web"),
    );

    HttpClient {
        base_url,
        headers: HashMap::new(),
        // Timeout in milliseconds (30000ms = 30 seconds)
        // Standardized to milliseconds for consistency with HTTP libraries (reqwest, etc.)
        timeout: 30000,
        retry_count: 3,
    }
}

pub fn render_template(template: String, data: HashMap<String, Value>) -> String {
    crate::stdlib::log::info(
        "web",
        {
            let mut log_data = std::collections::HashMap::new();
            log_data.insert("template".to_string(), Value::String(template.clone()));
            log_data.insert(
                "message".to_string(),
                Value::String(format!("Rendering template: {}", template)),
            );
            log_data
        },
        Some("web"),
    );

    // Simple template rendering
    let mut result = template.clone();

    for (key, value) in data {
        let placeholder = format!("{{{{{}}}}}", key);
        let replacement = value.to_string();
        result = result.replace(&placeholder, &replacement);
    }

    result
}

pub fn get_request(url: String) -> Result<HttpResponse, String> {
    crate::stdlib::log::info(
        "web",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("url".to_string(), Value::String(url.clone()));
            data.insert(
                "message".to_string(),
                Value::String(format!("Making GET request to {}", url)),
            );
            data
        },
        Some("web"),
    );

    // Simulated HTTP GET request
    Ok(HttpResponse {
        status: 200,
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
        body: format!(
            "{{\"url\": \"{}\", \"method\": \"GET\", \"status\": \"success\"}}",
            url
        ),
        cookies: Vec::new(),
        redirect_url: None,
    })
}

pub fn post_request(url: String, data: HashMap<String, Value>) -> Result<HttpResponse, String> {
    crate::stdlib::log::info(
        "web",
        {
            let mut log_data = std::collections::HashMap::new();
            log_data.insert("url".to_string(), Value::String(url.clone()));
            log_data.insert(
                "message".to_string(),
                Value::String(format!("Making POST request to {}", url)),
            );
            log_data
        },
        Some("web"),
    );

    // Simulated HTTP POST request
    Ok(HttpResponse {
        status: 201,
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
        body: format!(
            "{{\"url\": \"{}\", \"method\": \"POST\", \"data\": {:?}, \"status\": \"created\"}}",
            url, data
        ),
        cookies: Vec::new(),
        redirect_url: None,
    })
}

pub fn create_html_element(tag: String, attributes: HashMap<String, String>) -> HtmlElement {
    HtmlElement {
        tag,
        attributes,
        children: Vec::new(),
        text: None,
        event_handlers: HashMap::new(),
    }
}

pub fn add_child(parent: &mut HtmlElement, child: HtmlElement) {
    parent.children.push(child);
}

pub fn set_text(element: &mut HtmlElement, text: String) {
    element.text = Some(text);
}

pub fn render_html(element: &HtmlElement) -> String {
    let mut html = String::new();

    // Start tag
    html.push_str(&format!("<{}", element.tag));

    // Attributes
    for (key, value) in &element.attributes {
        html.push_str(&format!(" {}=\"{}\"", key, value));
    }

    html.push('>');

    // Text content
    if let Some(text) = &element.text {
        html.push_str(text);
    }

    // Children
    for child in &element.children {
        html.push_str(&render_html(child));
    }

    // End tag
    html.push_str(&format!("</{}>", element.tag));

    html
}

pub fn parse_url(url: String) -> HashMap<String, String> {
    let mut params = HashMap::new();

    if let Some(query_start) = url.find('?') {
        let query_string = &url[query_start + 1..];

        for pair in query_string.split('&') {
            if let Some(equal_pos) = pair.find('=') {
                let key = &pair[..equal_pos];
                let value = &pair[equal_pos + 1..];
                params.insert(key.to_string(), value.to_string());
            }
        }
    }

    params
}

pub fn set_header(client: &mut HttpClient, key: String, value: String) {
    client.headers.insert(key, value);
}

pub fn get_header<'a>(request: &'a HttpRequest, key: &str) -> Option<&'a String> {
    request.headers.get(key)
}

pub fn set_cookie(
    response: &mut HttpResponse,
    name: String,
    value: String,
    options: HashMap<String, String>,
) {
    let mut cookie_value = format!("{}={}", name, value);

    for (key, val) in options {
        cookie_value.push_str(&format!("; {}={}", key, val));
    }

    response
        .headers
        .insert("Set-Cookie".to_string(), cookie_value);
}

pub fn get_cookie(request: &HttpRequest, name: &str) -> Option<String> {
    if let Some(cookie_header) = request.headers.get("Cookie") {
        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if let Some(equal_pos) = cookie.find('=') {
                let cookie_name = &cookie[..equal_pos];
                let cookie_value = &cookie[equal_pos + 1..];

                if cookie_name == name {
                    return Some(cookie_value.to_string());
                }
            }
        }
    }
    None
}

pub fn redirect(response: &mut HttpResponse, url: String) {
    response.status = 302;
    response.headers.insert("Location".to_string(), url);
}

pub fn json_response(data: HashMap<String, Value>) -> HttpResponse {
    HttpResponse {
        status: 200,
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
        body: format!("{:?}", data), // Simplified JSON serialization
        cookies: Vec::new(),
        redirect_url: None,
    }
}

pub fn html_response(html: String) -> HttpResponse {
    HttpResponse {
        status: 200,
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "text/html".to_string());
            h
        },
        body: html,
        cookies: Vec::new(),
        redirect_url: None,
    }
}

pub fn error_response(status: i64, message: String) -> HttpResponse {
    HttpResponse {
        status,
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
        body: format!("{{\"error\": \"{}\"}}", message),
        cookies: Vec::new(),
        redirect_url: None,
    }
}

// === FRONTEND FRAMEWORK FUNCTIONS ===

pub fn create_html_page(title: String) -> HtmlPage {
    crate::stdlib::log::info(
        "Creating HTML page",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("title".to_string(), Value::String(title.clone()));
            data.insert(
                "message".to_string(),
                Value::String("Creating HTML page".to_string()),
            );
            data
        },
        Some("web"),
    );

    HtmlPage {
        title,
        meta: HashMap::new(),
        styles: Vec::new(),
        scripts: Vec::new(),
        head_elements: Vec::new(),
        body: HtmlElement {
            tag: "body".to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
            text: None,
            event_handlers: HashMap::new(),
        },
    }
}

pub fn add_css_file(page: &mut HtmlPage, css_path: String) {
    page.styles.push(css_path.clone());

    crate::stdlib::log::info(
        "CSS file added",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("css_path".to_string(), Value::String(css_path));
            data.insert(
                "message".to_string(),
                Value::String("CSS file added".to_string()),
            );
            data
        },
        Some("web"),
    );
}

pub fn add_js_file(page: &mut HtmlPage, js_path: String) {
    page.scripts.push(js_path.clone());

    crate::stdlib::log::info(
        "JavaScript file added",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("js_path".to_string(), Value::String(js_path));
            data.insert(
                "message".to_string(),
                Value::String("JavaScript file added".to_string()),
            );
            data
        },
        Some("web"),
    );
}

pub fn create_element(tag: String, text: Option<String>) -> HtmlElement {
    HtmlElement {
        tag,
        attributes: HashMap::new(),
        children: Vec::new(),
        text,
        event_handlers: HashMap::new(),
    }
}

pub fn add_attribute(element: &mut HtmlElement, key: String, value: String) {
    element.attributes.insert(key, value);
}

pub fn add_event_handler(element: &mut HtmlElement, event: String, handler: String) {
    element.event_handlers.insert(event, handler);
}

pub fn append_child(parent: &mut HtmlElement, child: HtmlElement) {
    parent.children.push(child);
}

pub fn render_html_page(page: &HtmlPage) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n");

    // Head section
    html.push_str("<head>\n");
    html.push_str(&format!("  <title>{}</title>\n", page.title));

    // Meta tags
    for (key, value) in &page.meta {
        html.push_str(&format!("  <meta {}=\"{}\">\n", key, value));
    }

    // CSS files
    for css in &page.styles {
        html.push_str(&format!("  <link rel=\"stylesheet\" href=\"{}\">\n", css));
    }

    // Head elements
    for element in &page.head_elements {
        html.push_str(&format!("  {}\n", render_html_element(element)));
    }

    html.push_str("</head>\n");

    // Body section
    html.push_str(&render_html_element(&page.body));

    // JavaScript files
    for js in &page.scripts {
        html.push_str(&format!("  <script src=\"{}\"></script>\n", js));
    }

    html.push_str("</html>");
    html
}

pub fn render_html_element(element: &HtmlElement) -> String {
    let mut html = String::new();

    // Opening tag
    html.push('<');
    html.push_str(&element.tag);

    // Attributes
    for (key, value) in &element.attributes {
        html.push_str(&format!(" {}=\"{}\"", key, value));
    }

    // Event handlers (as data attributes)
    for (event, handler) in &element.event_handlers {
        html.push_str(&format!(" data-{}=\"{}\"", event, handler));
    }

    html.push('>');

    // Content
    if let Some(text) = &element.text {
        html.push_str(text);
    }

    // Children
    for child in &element.children {
        html.push_str(&render_html_element(child));
    }

    // Closing tag
    html.push_str(&format!("</{}>", element.tag));

    html
}

pub fn create_form(action: String, method: String) -> HtmlElement {
    let mut form = create_element("form".to_string(), None);
    add_attribute(&mut form, "action".to_string(), action);
    add_attribute(&mut form, "method".to_string(), method);
    form
}

pub fn create_input(input_type: String, name: String, placeholder: String) -> HtmlElement {
    let mut input = create_element("input".to_string(), None);
    add_attribute(&mut input, "type".to_string(), input_type);
    add_attribute(&mut input, "name".to_string(), name);
    add_attribute(&mut input, "placeholder".to_string(), placeholder);
    input
}

pub fn create_button(text: String, button_type: String) -> HtmlElement {
    let mut button = create_element("button".to_string(), Some(text));
    add_attribute(&mut button, "type".to_string(), button_type);
    button
}

// === API FRAMEWORK FUNCTIONS ===

pub fn create_api_endpoint(path: String, method: String, handler: String) -> ApiEndpoint {
    let http_method = match method.to_uppercase().as_str() {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PUT" => HttpMethod::PUT,
        "DELETE" => HttpMethod::DELETE,
        "PATCH" => HttpMethod::PATCH,
        _ => HttpMethod::GET,
    };

    crate::stdlib::log::info(
        "API endpoint created",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("path".to_string(), Value::String(path.clone()));
            data.insert("method".to_string(), Value::String(method));
            data.insert(
                "message".to_string(),
                Value::String("API endpoint created".to_string()),
            );
            data
        },
        Some("web"),
    );

    ApiEndpoint {
        path,
        method: http_method,
        handler,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
        },
        output_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
        },
        auth_required: false,
        rate_limit: None,
    }
}

pub fn add_auth_requirement(endpoint: &mut ApiEndpoint, required: bool) {
    endpoint.auth_required = required;

    crate::stdlib::log::info(
        "Auth requirement updated",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("path".to_string(), Value::String(endpoint.path.clone()));
            data.insert("auth_required".to_string(), Value::Bool(required));
            data.insert(
                "message".to_string(),
                Value::String("Auth requirement updated".to_string()),
            );
            data
        },
        Some("web"),
    );
}

pub fn add_rate_limit(endpoint: &mut ApiEndpoint, requests_per_minute: i64, burst_limit: i64) {
    endpoint.rate_limit = Some(RateLimit {
        requests_per_minute,
        burst_limit,
    });

    crate::stdlib::log::info(
        "Rate limit added",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("path".to_string(), Value::String(endpoint.path.clone()));
            data.insert("rpm".to_string(), Value::Int(requests_per_minute));
            data.insert("burst".to_string(), Value::Int(burst_limit));
            data.insert(
                "message".to_string(),
                Value::String("Rate limit added".to_string()),
            );
            data
        },
        Some("web"),
    );
}

/// Parse request body as JSON and validate against schema (required fields + simple type check).
/// Returns Ok(true) if valid, Err with message on parse or validation failure.
pub fn validate_json_request(request: &HttpRequest, schema: &JsonSchema) -> Result<bool, String> {
    crate::stdlib::log::info(
        "Validating JSON request",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("path".to_string(), Value::String(request.path.clone()));
            data.insert(
                "schema_type".to_string(),
                Value::String(schema.schema_type.clone()),
            );
            data.insert(
                "message".to_string(),
                Value::String("Validating JSON request".to_string()),
            );
            data
        },
        Some("web"),
    );

    let body: serde_json::Value =
        serde_json::from_str(&request.body).map_err(|e| format!("Invalid JSON: {}", e))?;

    let obj = body.as_object().ok_or("Expected JSON object")?;

    for key in &schema.required {
        if !obj.contains_key(key) {
            return Err(format!("Missing required field: {}", key));
        }
    }

    for (key, prop) in &schema.properties {
        if let Some(v) = obj.get(key) {
            if !json_value_matches_type(v, &prop.property_type) {
                return Err(format!(
                    "Field '{}' has wrong type (expected {})",
                    key, prop.property_type
                ));
            }
        }
    }

    Ok(true)
}

fn json_value_matches_type(v: &serde_json::Value, typ: &str) -> bool {
    match typ.to_lowercase().as_str() {
        "string" => v.is_string(),
        "number" => v.is_number(),
        "integer" => v.is_i64() || v.is_u64(),
        "boolean" | "bool" => v.is_boolean(),
        "object" => v.is_object(),
        "array" => v.is_array(),
        "null" => v.is_null(),
        _ => true,
    }
}

// === WEBSOCKET FUNCTIONS ===

pub fn create_websocket_server(port: i64) -> WebSocketServer {
    crate::stdlib::log::info(
        "Creating WebSocket server",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("port".to_string(), Value::Int(port));
            data.insert(
                "message".to_string(),
                Value::String("Creating WebSocket server".to_string()),
            );
            data
        },
        Some("web"),
    );

    WebSocketServer {
        port,
        connections: HashMap::new(),
        rooms: HashMap::new(),
    }
}

pub fn add_websocket_connection(
    server: &mut WebSocketServer,
    connection_id: String,
    user_id: Option<String>,
) {
    let connection = WebSocketConnection {
        id: connection_id.clone(),
        user_id,
        rooms: Vec::new(),
        last_ping: "2024-01-01T00:00:00Z".to_string(), // Simplified timestamp
        metadata: HashMap::new(),
    };

    server.connections.insert(connection_id.clone(), connection);

    crate::stdlib::log::info(
        "WebSocket connection added",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("connection_id".to_string(), Value::String(connection_id));
            data.insert(
                "message".to_string(),
                Value::String("WebSocket connection added".to_string()),
            );
            data
        },
        Some("web"),
    );
}

pub fn join_room(
    server: &mut WebSocketServer,
    connection_id: String,
    room_name: String,
) -> Result<bool, String> {
    if let Some(connection) = server.connections.get_mut(&connection_id) {
        connection.rooms.push(room_name.clone());

        server
            .rooms
            .entry(room_name.clone())
            .or_insert_with(Vec::new)
            .push(connection_id.clone());

        crate::stdlib::log::info(
            "Joined room",
            {
                let mut data = std::collections::HashMap::new();
                data.insert("connection_id".to_string(), Value::String(connection_id));
                data.insert("room".to_string(), Value::String(room_name));
                data.insert(
                    "message".to_string(),
                    Value::String("Joined room".to_string()),
                );
                data
            },
            Some("web"),
        );

        Ok(true)
    } else {
        Err("Connection not found".to_string())
    }
}

fn ws_pending_messages() -> std::sync::MutexGuard<'static, HashMap<String, Vec<String>>> {
    static REG: OnceLock<Mutex<HashMap<String, Vec<String>>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
}

/// Pending messages queued for each connection by broadcast_to_room. The WS transport layer
/// should call this to drain and send; real delivery requires integration with axum/tokio-tungstenite.
pub fn get_and_clear_connection_pending_messages(connection_id: &str) -> Vec<String> {
    let mut reg = ws_pending_messages();
    reg.remove(connection_id).unwrap_or_default()
}

pub fn broadcast_to_room(
    server: &WebSocketServer,
    room_name: String,
    message: String,
) -> Result<i64, String> {
    if let Some(connection_ids) = server.rooms.get(&room_name) {
        let mut reg = ws_pending_messages();
        for id in connection_ids {
            reg.entry(id.clone()).or_default().push(message.clone());
        }
        let message_count = connection_ids.len() as i64;

        crate::stdlib::log::info(
            "Broadcasting to room",
            {
                let mut data = std::collections::HashMap::new();
                data.insert("room".to_string(), Value::String(room_name));
                data.insert("connections".to_string(), Value::Int(message_count));
                data.insert(
                    "message".to_string(),
                    Value::String("Broadcasting to room".to_string()),
                );
                data
            },
            Some("web"),
        );

        Ok(message_count)
    } else {
        Err("Room not found".to_string())
    }
}

// === TEMPLATE ENGINE FUNCTIONS ===

pub fn create_template(name: String, content: String) -> Template {
    crate::stdlib::log::info(
        "Template created",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("template_name".to_string(), Value::String(name.clone()));
            data.insert(
                "message".to_string(),
                Value::String("Template created".to_string()),
            );
            data
        },
        Some("web"),
    );

    Template {
        name,
        content,
        variables: HashMap::new(),
        includes: Vec::new(),
    }
}

pub fn add_template_variable(template: &mut Template, key: String, value: Value) {
    template.variables.insert(key.clone(), value);

    crate::stdlib::log::info(
        "Template variable added",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("template".to_string(), Value::String(template.name.clone()));
            data.insert("variable".to_string(), Value::String(key));
            data.insert(
                "message".to_string(),
                Value::String("Template variable added".to_string()),
            );
            data
        },
        Some("web"),
    );
}

pub fn render_advanced_template(template: &Template) -> String {
    let mut result = template.content.clone();

    // Replace variables
    for (key, value) in &template.variables {
        let placeholder = format!("{{{{{}}}}}", key);
        let replacement = match value {
            Value::String(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => "".to_string(),
        };
        result = result.replace(&placeholder, &replacement);
    }

    crate::stdlib::log::info(
        "Template rendered",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("template".to_string(), Value::String(template.name.clone()));
            data.insert(
                "variables_count".to_string(),
                Value::Int(template.variables.len() as i64),
            );
            data.insert(
                "message".to_string(),
                Value::String("Template rendered".to_string()),
            );
            data
        },
        Some("web"),
    );

    result
}

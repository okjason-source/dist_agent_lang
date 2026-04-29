//! Provider-agnostic graph API primitives.
//!
//! This module provides stateful graph clients for APIs such as Microsoft Graph
//! (and other graph-style REST APIs) with bearer auth, OAuth client-credentials,
//! JSON request/response handling, and retry/backoff behavior.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone)]
pub struct GraphClient {
    pub base_url: String,
    pub bearer_token: String,
    pub timeout_secs: u64,
    pub default_headers: HashMap<String, String>,
    pub max_retries: u32,
    pub retry_backoff_ms: u64,
}

#[derive(Debug, Clone)]
pub struct GraphResponse {
    pub status: i64,
    pub ok: bool,
    pub body: serde_json::Value,
    pub body_text: String,
    pub request_id: Option<String>,
    pub retries: i64,
}

fn clients() -> &'static Mutex<HashMap<String, GraphClient>> {
    static CLIENTS: OnceLock<Mutex<HashMap<String, GraphClient>>> = OnceLock::new();
    CLIENTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_client_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    format!("graph_client_{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn validate_base_url(base_url: &str) -> Result<String, String> {
    let base = base_url.trim();
    if base.is_empty() {
        return Err("graph::connect requires a non-empty base_url".to_string());
    }
    if base.starts_with("https://") {
        return Ok(base.trim_end_matches('/').to_string());
    }
    if base.starts_with("http://") && env_flag("DAL_GRAPH_ALLOW_INSECURE_HTTP") {
        return Ok(base.trim_end_matches('/').to_string());
    }
    Err(
        "graph::connect base_url must use https:// (or set DAL_GRAPH_ALLOW_INSECURE_HTTP=1 for local-only http)"
            .to_string(),
    )
}

fn is_protected_header(key: &str) -> bool {
    let k = key.trim().to_ascii_lowercase();
    k == "authorization" || k == "content-length" || k == "host"
}

pub fn create_client(
    base_url: &str,
    bearer_token: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    let id = next_client_id();
    let client = GraphClient {
        base_url: validate_base_url(base_url)?,
        bearer_token: bearer_token.trim().to_string(),
        timeout_secs: timeout_secs.max(1),
        default_headers: HashMap::new(),
        max_retries: 2,
        retry_backoff_ms: 250,
    };
    let mut guard = clients().lock().map_err(|e| e.to_string())?;
    guard.insert(id.clone(), client);
    Ok(id)
}

pub fn set_header(client_id: &str, key: &str, value: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("graph::set_header key cannot be empty".to_string());
    }
    if is_protected_header(key) {
        return Err(format!(
            "graph::set_header '{}' is protected (use graph::set_token for auth)",
            key
        ));
    }
    let mut guard = clients().lock().map_err(|e| e.to_string())?;
    let c = guard
        .get_mut(client_id)
        .ok_or_else(|| format!("graph client '{}' not found", client_id))?;
    c.default_headers.insert(key.to_string(), value.to_string());
    Ok(())
}

pub fn set_token(client_id: &str, bearer_token: &str) -> Result<(), String> {
    let mut guard = clients().lock().map_err(|e| e.to_string())?;
    let c = guard
        .get_mut(client_id)
        .ok_or_else(|| format!("graph client '{}' not found", client_id))?;
    c.bearer_token = bearer_token.trim().to_string();
    Ok(())
}

pub fn set_retry_policy(
    client_id: &str,
    max_retries: u32,
    retry_backoff_ms: u64,
) -> Result<(), String> {
    let mut guard = clients().lock().map_err(|e| e.to_string())?;
    let c = guard
        .get_mut(client_id)
        .ok_or_else(|| format!("graph client '{}' not found", client_id))?;
    c.max_retries = max_retries.min(10);
    c.retry_backoff_ms = retry_backoff_ms.max(1);
    Ok(())
}

pub fn remove_client(client_id: &str) -> Result<bool, String> {
    let mut guard = clients().lock().map_err(|e| e.to_string())?;
    Ok(guard.remove(client_id).is_some())
}

#[cfg(feature = "http-interface")]
pub fn connect_client_credentials(
    base_url: &str,
    token_endpoint: &str,
    client_id: &str,
    client_secret: &str,
    scope: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    let token_endpoint = token_endpoint.trim();
    if token_endpoint.is_empty() {
        return Err("graph::connect_client_credentials requires token_endpoint".to_string());
    }
    if !token_endpoint.starts_with("https://") {
        return Err(
            "graph::connect_client_credentials token_endpoint must use https://".to_string(),
        );
    }
    let timeout_secs = timeout_secs.max(1);
    let http = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| e.to_string())?;
    let mut form: Vec<(&str, &str)> = vec![
        ("grant_type", "client_credentials"),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];
    let scope = scope.trim();
    if !scope.is_empty() {
        form.push(("scope", scope));
    }
    let resp = http
        .post(token_endpoint)
        .header("content-type", "application/x-www-form-urlencoded")
        .form(&form)
        .send()
        .map_err(|e| format!("graph oauth token request failed: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!(
            "graph oauth token request failed with status {}",
            resp.status()
        ));
    }
    let payload: serde_json::Value = resp
        .json()
        .map_err(|e| format!("graph oauth token decode failed: {}", e))?;
    let token = payload
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "graph oauth token response missing access_token".to_string())?;
    create_client(base_url, token, timeout_secs)
}

#[cfg(not(feature = "http-interface"))]
pub fn connect_client_credentials(
    _base_url: &str,
    _token_endpoint: &str,
    _client_id: &str,
    _client_secret: &str,
    _scope: &str,
    _timeout_secs: u64,
) -> Result<String, String> {
    Err("graph oauth requires the 'http-interface' feature".to_string())
}

#[cfg(feature = "http-interface")]
fn perform_request(
    client: &GraphClient,
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
) -> Result<GraphResponse, String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("graph request path cannot be empty".to_string());
    }
    let url = if path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else if path.starts_with('/') {
        format!("{}{}", client.base_url, path)
    } else {
        format!("{}/{}", client.base_url, path)
    };
    if url.starts_with("http://") && !env_flag("DAL_GRAPH_ALLOW_INSECURE_HTTP") {
        return Err(
            "graph request blocked: non-HTTPS URL (set DAL_GRAPH_ALLOW_INSECURE_HTTP=1 for local-only)"
                .to_string(),
        );
    }

    let http = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(client.timeout_secs))
        .build()
        .map_err(|e| e.to_string())?;
    let method = reqwest::Method::from_bytes(method.trim().to_ascii_uppercase().as_bytes())
        .map_err(|_| format!("unsupported HTTP method '{}'", method))?;

    let mut attempt = 0u32;
    loop {
        let mut req = http.request(method.clone(), &url);
        if !client.bearer_token.is_empty() {
            req = req.bearer_auth(&client.bearer_token);
        }
        for (k, v) in &client.default_headers {
            req = req.header(k, v);
        }
        if let Some(payload) = body {
            req = req.json(payload);
        }

        let resp = req.send().map_err(|e| e.to_string())?;
        let status = resp.status();
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|s| s * 1000);
        let request_id = resp
            .headers()
            .get("request-id")
            .or_else(|| resp.headers().get("x-ms-request-id"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let text = resp.text().map_err(|e| e.to_string())?;
        let parsed = serde_json::from_str::<serde_json::Value>(&text)
            .unwrap_or_else(|_| serde_json::json!(text));

        let retryable = status.as_u16() == 429 || status.is_server_error();
        if retryable && attempt < client.max_retries {
            let backoff = retry_after.unwrap_or_else(|| {
                let mult = 2u64.saturating_pow(attempt);
                client.retry_backoff_ms.saturating_mul(mult)
            });
            std::thread::sleep(std::time::Duration::from_millis(backoff));
            attempt = attempt.saturating_add(1);
            continue;
        }

        return Ok(GraphResponse {
            status: status.as_u16() as i64,
            ok: status.is_success(),
            body: parsed,
            body_text: text,
            request_id,
            retries: attempt as i64,
        });
    }
}

#[cfg(not(feature = "http-interface"))]
fn perform_request(
    _client: &GraphClient,
    _method: &str,
    _path: &str,
    _body: Option<&serde_json::Value>,
) -> Result<GraphResponse, String> {
    Err("graph requests require the 'http-interface' feature".to_string())
}

pub fn request(
    client_id: &str,
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
) -> Result<GraphResponse, String> {
    let guard = clients().lock().map_err(|e| e.to_string())?;
    let c = guard
        .get(client_id)
        .ok_or_else(|| format!("graph client '{}' not found", client_id))?;
    perform_request(c, method, path, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        old: Option<String>,
    }

    impl EnvGuard {
        fn set_allow_http(value: Option<&str>) -> Self {
            let old = std::env::var("DAL_GRAPH_ALLOW_INSECURE_HTTP").ok();
            if let Some(v) = value {
                std::env::set_var("DAL_GRAPH_ALLOW_INSECURE_HTTP", v);
            } else {
                std::env::remove_var("DAL_GRAPH_ALLOW_INSECURE_HTTP");
            }
            Self { old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(v) = &self.old {
                std::env::set_var("DAL_GRAPH_ALLOW_INSECURE_HTTP", v);
            } else {
                std::env::remove_var("DAL_GRAPH_ALLOW_INSECURE_HTTP");
            }
        }
    }

    #[test]
    fn create_and_remove_client() {
        let _l = env_lock().lock().expect("lock");
        let _g = EnvGuard::set_allow_http(None);
        let id = create_client("https://graph.microsoft.com/v1.0", "token", 30).expect("client");
        assert!(id.starts_with("graph_client_"));
        assert!(remove_client(&id).expect("remove"));
    }

    #[test]
    fn create_client_rejects_insecure_http_by_default() {
        let _l = env_lock().lock().expect("lock");
        let _g = EnvGuard::set_allow_http(None);
        let e = create_client("http://localhost:8080", "token", 30).expect_err("invalid");
        assert!(e.contains("https://"));
    }

    #[test]
    fn create_client_allows_http_with_env_toggle() {
        let _l = env_lock().lock().expect("lock");
        let _g = EnvGuard::set_allow_http(Some("1"));
        let id = create_client("http://localhost:8080", "token", 10).expect("client");
        assert!(remove_client(&id).expect("remove"));
    }

    #[test]
    fn set_header_rejects_authorization_override() {
        let _l = env_lock().lock().expect("lock");
        let _g = EnvGuard::set_allow_http(None);
        let id = create_client("https://graph.microsoft.com/v1.0", "token", 30).expect("client");
        let err = set_header(&id, "Authorization", "Bearer bad").expect_err("must fail");
        assert!(err.contains("protected"));
        let _ = remove_client(&id);
    }
}

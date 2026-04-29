//! MCP bridge lifecycle + tool invocation primitives.
//!
//! This namespace is runtime-facing and complements the CLI `dal mcp-bridge`.
//! It can:
//! - start/stop/list status for MCP bridge processes,
//! - invoke known bridge tools against DAL agent HTTP endpoints.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpTransport {
    Stdio,
    HttpStream,
}

impl McpTransport {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stdio => "stdio",
            Self::HttpStream => "http-stream",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BridgeStatus {
    pub id: String,
    pub pid: i64,
    pub running: bool,
    pub exit_code: Option<i64>,
    pub base_url: String,
    pub transport: String,
}

struct BridgeProc {
    id: String,
    base_url: String,
    transport: McpTransport,
    child: Child,
}

#[derive(Debug, Clone)]
pub struct InvokeResult {
    pub status: i64,
    pub ok: bool,
    pub body: serde_json::Value,
    pub body_text: String,
}

fn bridges() -> &'static Mutex<HashMap<String, BridgeProc>> {
    static BRIDGES: OnceLock<Mutex<HashMap<String, BridgeProc>>> = OnceLock::new();
    BRIDGES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_bridge_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    format!("mcp_bridge_{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

pub fn resolve_transport(raw: Option<&str>) -> Result<McpTransport, String> {
    let from_env = std::env::var("DAL_MCP_TRANSPORT").ok();
    let selected = raw
        .map(|s| s.trim().to_string())
        .or(from_env)
        .unwrap_or_else(|| "stdio".to_string())
        .to_ascii_lowercase();
    match selected.as_str() {
        "stdio" => Ok(McpTransport::Stdio),
        "http-stream" | "http_stream" | "httpstream" => Ok(McpTransport::HttpStream),
        other => Err(format!(
            "mcp::bridge_start unsupported transport `{}`. Use `stdio` or `http-stream`",
            other
        )),
    }
}

pub fn resolve_base_url(raw: Option<&str>) -> String {
    raw.map(String::from)
        .or_else(|| std::env::var("DAL_AGENT_HTTP_BASE").ok())
        .or_else(|| std::env::var("DAL_MCP_HTTP_BASE").ok())
        .or_else(|| std::env::var("DAL_COO_BASE_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:4040".to_string())
        .trim_end_matches('/')
        .to_string()
}

pub fn find_bridge_script() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("DAL_MCP_BRIDGE_SCRIPT") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "mcp::bridge_start DAL_MCP_BRIDGE_SCRIPT missing: {}",
            path.display()
        ));
    }
    if let Ok(p) = std::env::var("DAL_COO_MCP_SCRIPT") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "mcp::bridge_start DAL_COO_MCP_SCRIPT missing: {}",
            path.display()
        ));
    }
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = cwd;
        for _ in 0..12 {
            let candidate = dir.join("COO/mcp/src/server.js");
            if candidate.is_file() {
                return Ok(candidate);
            }
            if !dir.pop() {
                break;
            }
        }
    }
    let compile_time_candidates = [
        Path::new(env!("CARGO_MANIFEST_DIR")).join("COO/mcp/src/server.js"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../COO/mcp/src/server.js"),
    ];
    for candidate in compile_time_candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err("mcp::bridge_start could not find MCP bridge script (COO/mcp/src/server.js)".to_string())
}

pub fn bridge_start(base_url: Option<&str>, transport: Option<&str>) -> Result<String, String> {
    let transport = resolve_transport(transport)?;
    let base = resolve_base_url(base_url);
    let script = find_bridge_script()?;
    let mcp_root = script
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| format!("invalid MCP script path: {}", script.display()))?;
    let node_modules = mcp_root.join("node_modules");
    if !node_modules.is_dir() {
        return Err(format!(
            "mcp::bridge_start missing {} — run `cd {} && npm install`",
            node_modules.display(),
            mcp_root.display()
        ));
    }

    let child = std::process::Command::new("node")
        .current_dir(mcp_root)
        .arg(&script)
        .env("DAL_AGENT_HTTP_BASE", &base)
        .env("DAL_MCP_HTTP_BASE", &base)
        .env("DAL_COO_BASE_URL", &base)
        .env("DAL_MCP_TRANSPORT", transport.as_str())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("mcp::bridge_start failed to spawn node: {}", e))?;

    let id = next_bridge_id();
    let proc = BridgeProc {
        id: id.clone(),
        base_url: base,
        transport,
        child,
    };
    let mut map = bridges().lock().map_err(|e| e.to_string())?;
    map.insert(id.clone(), proc);
    Ok(id)
}

pub fn bridge_stop(bridge_id: &str) -> Result<bool, String> {
    let mut map = bridges().lock().map_err(|e| e.to_string())?;
    let Some(mut proc) = map.remove(bridge_id) else {
        return Ok(false);
    };
    let _ = proc.child.kill();
    let _ = proc.child.wait();
    Ok(true)
}

pub fn bridge_status(bridge_id: &str) -> Result<Option<BridgeStatus>, String> {
    let mut map = bridges().lock().map_err(|e| e.to_string())?;
    let Some(proc) = map.get_mut(bridge_id) else {
        return Ok(None);
    };
    let state = proc.child.try_wait().map_err(|e| e.to_string())?;
    let (running, exit_code) = match state {
        Some(status) => (false, status.code().map(|c| c as i64)),
        None => (true, None),
    };
    Ok(Some(BridgeStatus {
        id: proc.id.clone(),
        pid: proc.child.id() as i64,
        running,
        exit_code,
        base_url: proc.base_url.clone(),
        transport: proc.transport.as_str().to_string(),
    }))
}

pub fn bridge_list() -> Result<Vec<BridgeStatus>, String> {
    let ids = {
        let map = bridges().lock().map_err(|e| e.to_string())?;
        map.keys().cloned().collect::<Vec<_>>()
    };
    let mut out = Vec::new();
    for id in ids {
        if let Some(s) = bridge_status(&id)? {
            out.push(s);
        }
    }
    Ok(out)
}

fn endpoint_for_tool(tool_name: &str) -> Option<&'static str> {
    match tool_name {
        "dal_agent_message" => Some("/api/message"),
        "dal_agent_task" => Some("/api/task"),
        "dal_agent_run" => Some("/api/agents/run"),
        "dal_agent_workflow" => Some("/api/workflow"),
        _ => None,
    }
}

#[cfg(feature = "http-interface")]
pub fn invoke_tool(
    tool_name: &str,
    input: &serde_json::Value,
    base_url: Option<&str>,
) -> Result<InvokeResult, String> {
    let path = endpoint_for_tool(tool_name)
        .ok_or_else(|| format!("mcp::invoke unsupported tool `{}`", tool_name))?;
    let base = resolve_base_url(base_url);
    let url = format!("{}{}", base, path);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(&url)
        .header("content-type", "application/json")
        .json(input)
        .send()
        .map_err(|e| format!("mcp::invoke request failed: {}", e))?;
    let status = resp.status();
    let text = resp.text().map_err(|e| e.to_string())?;
    let parsed = serde_json::from_str::<serde_json::Value>(&text)
        .unwrap_or_else(|_| serde_json::json!(text));
    Ok(InvokeResult {
        status: status.as_u16() as i64,
        ok: status.is_success(),
        body: parsed,
        body_text: text,
    })
}

#[cfg(not(feature = "http-interface"))]
pub fn invoke_tool(
    _tool_name: &str,
    _input: &serde_json::Value,
    _base_url: Option<&str>,
) -> Result<InvokeResult, String> {
    Err("mcp::invoke requires the 'http-interface' feature".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_resolution_supports_aliases() {
        assert_eq!(
            resolve_transport(Some("stdio")).expect("transport"),
            McpTransport::Stdio
        );
        assert_eq!(
            resolve_transport(Some("http_stream")).expect("transport"),
            McpTransport::HttpStream
        );
    }

    #[test]
    fn endpoint_map_contains_expected_tools() {
        assert_eq!(endpoint_for_tool("dal_agent_message"), Some("/api/message"));
        assert_eq!(endpoint_for_tool("dal_agent_task"), Some("/api/task"));
        assert_eq!(endpoint_for_tool("dal_agent_run"), Some("/api/agents/run"));
        assert_eq!(
            endpoint_for_tool("dal_agent_workflow"),
            Some("/api/workflow")
        );
        assert_eq!(endpoint_for_tool("unknown_tool"), None);
    }
}

use crate::runtime::values::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Admin ABI - Interface for administrative operations
///
/// Process start times: when wired to a real process manager, call
/// `register_process_start_time(process_id, unix_secs)` so `get_process_info` / `list_processes`
/// use actual start times; otherwise a runtime-derived value is used.

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub process_id: String,
    pub name: String,
    pub status: String,
    pub start_time: i64,
    pub resource_usage: HashMap<String, Value>,
}

fn process_start_times() -> std::sync::MutexGuard<'static, HashMap<String, i64>> {
    static REG: OnceLock<Mutex<HashMap<String, i64>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
}

/// Register process start time (Unix seconds). Used when wired to a real process manager.
pub fn register_process_start_time(process_id: &str, start_time_unix_secs: i64) {
    process_start_times().insert(process_id.to_string(), start_time_unix_secs);
}

fn fallback_start_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

impl ProcessInfo {
    pub fn new(process_id: String, name: String) -> Self {
        let start_time = process_start_times()
            .get(&process_id)
            .copied()
            .unwrap_or_else(fallback_start_time);
        Self {
            process_id,
            name,
            status: "running".to_string(),
            start_time,
            resource_usage: HashMap::new(),
        }
    }
    
    pub fn with_status(mut self, status: String) -> Self {
        self.status = status;
        self
    }
    
    pub fn with_resource_usage(mut self, resource: String, usage: Value) -> Self {
        self.resource_usage.insert(resource, usage);
        self
    }
}

/// Terminate process or agent
pub fn kill(process_id: &str, reason: &str) -> Result<bool, String> {
    // MOCK: in a real system this would terminate actual processes
    match process_id {
        "agent_123" => {
            if reason.is_empty() {
                Err("Kill reason is required".to_string())
            } else {
                Ok(true)
            }
        }
        "process_456" => {
            if reason == "resource_violation" || reason == "security_breach" {
                Ok(true)
            } else {
                Err("Invalid kill reason".to_string())
            }
        }
        "system_process" => {
            Err("Cannot kill system processes".to_string())
        }
        _ => Err(format!("Process not found: {}", process_id))
    }
}

/// Get process information
pub fn get_process_info(process_id: &str) -> Result<ProcessInfo, String> {
    // MOCK: in a real system this would query process status
    match process_id {
        "agent_123" => {
            let mut process = ProcessInfo::new("agent_123".to_string(), "data_processor".to_string());
            process = process.with_resource_usage("cpu".to_string(), Value::Int(45));
            process = process.with_resource_usage("memory".to_string(), Value::Int(1024));
            Ok(process)
        }
        "process_456" => {
            let mut process = ProcessInfo::new("process_456".to_string(), "web_server".to_string());
            process = process.with_resource_usage("cpu".to_string(), Value::Int(30));
            process = process.with_resource_usage("memory".to_string(), Value::Int(2048));
            Ok(process)
        }
        _ => Err(format!("Process not found: {}", process_id))
    }
}

/// List all running processes
pub fn list_processes() -> Vec<ProcessInfo> {
    // MOCK: in a real system this would query system processes
    vec![
        ProcessInfo::new("agent_123".to_string(), "data_processor".to_string()),
        ProcessInfo::new("process_456".to_string(), "web_server".to_string()),
        ProcessInfo::new("agent_789".to_string(), "monitor".to_string()),
    ]
}

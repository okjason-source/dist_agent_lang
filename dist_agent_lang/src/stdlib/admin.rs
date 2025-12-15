use crate::runtime::values::Value;
use std::collections::HashMap;

/// Admin ABI - Interface for administrative operations
/// 
/// This provides a namespace-based approach to admin operations:
/// - admin::kill(process_id, reason) - Terminate processes/agents
/// - admin::get_process_info(process_id) - Get process information
/// - admin::list_processes() - List running processes

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub process_id: String,
    pub name: String,
    pub status: String,
    pub start_time: i64,
    pub resource_usage: HashMap<String, Value>,
}

impl ProcessInfo {
    pub fn new(process_id: String, name: String) -> Self {
        Self {
            process_id,
            name,
            status: "running".to_string(),
            start_time: 1756744707, // Mock timestamp
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
    // Mock implementation - in real system this would terminate actual processes
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
    // Mock implementation - in real system this would query process status
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
    // Mock implementation - in real system this would query system processes
    vec![
        ProcessInfo::new("agent_123".to_string(), "data_processor".to_string()),
        ProcessInfo::new("process_456".to_string(), "web_server".to_string()),
        ProcessInfo::new("agent_789".to_string(), "monitor".to_string()),
    ]
}

use crate::runtime::values::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Log namespace for logging and audit functionality
/// Provides info and audit logging capabilities

/// Log entry structure
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: LogLevel,
    pub message: String,
    pub data: HashMap<String, Value>,
    pub source: String,
}

/// Log levels
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Audit,
    Debug,
}

lazy_static::lazy_static! {
    static ref LOG_STORAGE: Mutex<Vec<LogEntry>> = Mutex::new(Vec::new());
}

/// Log an informational message
/// 
/// # Arguments
/// * `message` - The message to log
/// * `data` - Additional data to include with the log
/// * `source` - Optional source identifier (defaults to "system" if None)
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::log;
/// use dist_agent_lang::runtime::values::Value;
/// use std::collections::HashMap;
/// let mut data1 = HashMap::new();
/// data1.insert("user_id".to_string(), Value::String("123".to_string()));
/// data1.insert("ip".to_string(), Value::String("192.168.1.1".to_string()));
/// log::info("User logged in", data1, Some("auth_service"));
/// let data2 = HashMap::new();
/// log::info("System message", data2, None); // Uses default "system" source
/// ```
pub fn info(message: &str, data: HashMap<String, Value>, source: Option<&str>) {
    let source_str = source.unwrap_or("system").to_string();
    log_message(LogLevel::Info, message, data, source_str);
}

/// Log a warning message
/// 
/// # Arguments
/// * `message` - The warning message
/// * `data` - Additional data to include with the warning
/// * `source` - Optional source identifier (defaults to "system" if None)
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::log;
/// use dist_agent_lang::runtime::values::Value;
/// use std::collections::HashMap;
/// let mut data = HashMap::new();
/// data.insert("usage".to_string(), Value::String("85%".to_string()));
/// data.insert("threshold".to_string(), Value::String("80%".to_string()));
/// log::warning("High memory usage detected", data, Some("monitor"));
/// ```
pub fn warning(message: &str, data: HashMap<String, Value>, source: Option<&str>) {
    let source_str = source.unwrap_or("system").to_string();
    log_message(LogLevel::Warning, message, data, source_str);
}

/// Log an error message
/// 
/// # Arguments
/// * `message` - The error message
/// * `data` - Additional data to include with the error
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::log;
/// use dist_agent_lang::runtime::values::Value;
/// use std::collections::HashMap;
/// let mut data = HashMap::new();
/// data.insert("error_code".to_string(), Value::String("CONN_001".to_string()));
/// data.insert("retry_count".to_string(), Value::Int(3));
/// log::error("Database connection failed", data, None);
/// ```
pub fn error(message: &str, data: HashMap<String, Value>, source: Option<&str>) {
    let source_str = source.unwrap_or("system").to_string();
    log_message(LogLevel::Error, message, data, source_str);
}

/// Log an audit event
/// 
/// # Arguments
/// * `event` - The audit event name
/// * `data` - Additional data to include with the audit
/// * `source` - Optional source identifier (defaults to "audit" if None)
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::log;
/// use dist_agent_lang::runtime::values::Value;
/// use std::collections::HashMap;
/// let mut data = HashMap::new();
/// data.insert("user_id".to_string(), Value::String("123".to_string()));
/// data.insert("ip".to_string(), Value::String("192.168.1.1".to_string()));
/// data.insert("success".to_string(), Value::Bool(true));
/// log::audit("user_login", data, Some("auth"));
/// ```
pub fn audit(event: &str, data: HashMap<String, Value>, source: Option<&str>) {
    let source_str = source.unwrap_or("audit").to_string();
    log_message(LogLevel::Audit, event, data, source_str);
}

/// Log a debug message
/// 
/// # Arguments
/// * `message` - The debug message
/// * `data` - Additional data to include with the debug info
/// * `source` - Optional source identifier (defaults to "debug" if None)
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::log;
/// use dist_agent_lang::runtime::values::Value;
/// use std::collections::HashMap;
/// let mut data = HashMap::new();
/// data.insert("function".to_string(), Value::String("process_data".to_string()));
/// data.insert("duration_ms".to_string(), Value::Int(150));
/// log::debug("Function execution time", data, Some("performance"));
/// ```
pub fn debug(message: &str, data: HashMap<String, Value>, source: Option<&str>) {
    let source_str = source.unwrap_or("debug").to_string();
    log_message(LogLevel::Debug, message, data, source_str);
}

/// Internal function to log messages
fn log_message(level: LogLevel, message: &str, data: HashMap<String, Value>, source: String) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    let entry = LogEntry {
        timestamp,
        level: level.clone(),
        message: message.to_string(),
        data: data.clone(),
        source: source.clone(),
    };
    
    // Store in log storage
    if let Ok(mut storage) = LOG_STORAGE.lock() {
        storage.push(entry.clone());
        
        // Keep only last 1000 entries (mock implementation)
        if storage.len() > 1000 {
            storage.remove(0);
        }
    }
    
    // Print to console for development (mock implementation)
    let level_str = match level {
        LogLevel::Info => "INFO",
        LogLevel::Warning => "WARN",
        LogLevel::Error => "ERROR",
        LogLevel::Audit => "AUDIT",
        LogLevel::Debug => "DEBUG",
    };
    
    println!("[{}] {}: {} - {:?}", level_str, source, message, data);
}

/// Get all log entries
/// 
/// # Returns
/// * `Vec<LogEntry>` - All stored log entries
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::log;
/// let entries = log::get_entries();
/// ```
pub fn get_entries() -> Vec<LogEntry> {
    if let Ok(storage) = LOG_STORAGE.lock() {
        storage.clone()
    } else {
        Vec::new()
    }
}

/// Get log entries by level
/// 
/// # Arguments
/// * `level` - The log level to filter by
/// 
/// # Returns
/// * `Vec<LogEntry>` - Filtered log entries
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::log;
/// use dist_agent_lang::stdlib::log::LogLevel;
/// let audit_entries = log::get_entries_by_level(LogLevel::Audit);
/// ```
pub fn get_entries_by_level(level: LogLevel) -> Vec<LogEntry> {
    get_entries().into_iter()
        .filter(|entry| entry.level == level)
        .collect()
}

/// Get log entries by source
/// 
/// # Arguments
/// * `source` - The source to filter by
/// 
/// # Returns
/// * `Vec<LogEntry>` - Filtered log entries
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::log;
/// let audit_logs = log::get_entries_by_source("audit");
/// ```
pub fn get_entries_by_source(source: &str) -> Vec<LogEntry> {
    get_entries().into_iter()
        .filter(|entry| entry.source == source)
        .collect()
}

/// Clear all log entries
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::log;
/// log::clear();
/// ```
pub fn clear() {
    if let Ok(mut storage) = LOG_STORAGE.lock() {
        storage.clear();
    }
}

/// Get log statistics
/// 
/// # Returns
/// * `HashMap<String, Value>` - Log statistics
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::log;
/// let stats = log::get_stats();
/// ```
pub fn get_stats() -> HashMap<String, Value> {
    let entries = get_entries();
    let mut stats = HashMap::new();
    
    stats.insert("total_entries".to_string(), Value::Int(entries.len() as i64));
    
    // Count by level
    let mut level_counts = HashMap::new();
    for entry in &entries {
        let level_str = match entry.level {
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            LogLevel::Audit => "audit",
            LogLevel::Debug => "debug",
        };
        
        let count = level_counts.entry(level_str.to_string()).or_insert(0);
        *count += 1;
    }
    
    for (level, count) in level_counts {
        stats.insert(format!("count_{}", level), Value::Int(count));
    }
    
    // Count by source
    let mut source_counts = HashMap::new();
    for entry in &entries {
        let count = source_counts.entry(entry.source.clone()).or_insert(0);
        *count += 1;
    }
    
    for (source, count) in source_counts {
        stats.insert(format!("source_{}", source), Value::Int(count));
    }
    
    stats
}

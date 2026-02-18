use crate::runtime::values::Value;
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Log namespace for logging and audit functionality.
/// Configurable via env:
/// - LOG_MAX_ENTRIES (default 1000) - In-memory log limit
/// - LOG_SINK (console|none|file|both) - Where to output logs
/// - LOG_LEVEL (debug|info|warning|error|audit) - Minimum log level
/// - LOG_FILE (path) - File path for persistent logging (default: ./logs/audit.log)
/// - LOG_DIR (path) - Directory for log files (default: ./logs)
/// - LOG_ROTATE_SIZE (bytes) - Rotate log file when it exceeds this size (default: 10MB)
/// - LOG_RETENTION_DAYS (days) - Keep log files for this many days (default: 30)

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
    static ref LOG_FILE_HANDLE: Mutex<Option<File>> = Mutex::new(None);
}

/// Initialize persistent file logging
/// This should be called once at application startup
pub fn initialize_file_logging() -> Result<(), String> {
    if !should_log_to_file() {
        return Ok(());
    }

    let log_file_path = get_log_file_path();
    
    // Create log directory if it doesn't exist
    if let Some(parent) = log_file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create log directory: {}", e))?;
    }

    // Close current handle if exists (for rotation)
    {
        let mut handle = LOG_FILE_HANDLE.lock().unwrap();
        if handle.is_some() {
            *handle = None; // Close current handle
        }
    }
    
    // Check if rotation is needed before opening
    rotate_logs_if_needed()?;
    
    // Open log file in append mode (after potential rotation)
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .map_err(|e| format!("Failed to open log file {}: {}", log_file_path.display(), e))?;

    *LOG_FILE_HANDLE.lock().unwrap() = Some(file);
    
    Ok(())
}

/// Get the log file path from environment or use default
fn get_log_file_path() -> PathBuf {
    if let Ok(log_file) = env::var("LOG_FILE") {
        PathBuf::from(log_file)
    } else {
        let log_dir = get_log_directory();
        log_dir.join("audit.log")
    }
}

/// Get the log directory from environment or use default
fn get_log_directory() -> PathBuf {
    if let Ok(log_dir) = env::var("LOG_DIR") {
        PathBuf::from(log_dir)
    } else {
        PathBuf::from("./logs")
    }
}

/// Check if we should log to file
fn should_log_to_file() -> bool {
    match env::var("LOG_SINK").as_deref() {
        Ok("file") | Ok("both") => true,
        _ => false,
    }
}

/// Get maximum log file size before rotation (in bytes)
fn max_log_file_size() -> u64 {
    env::var("LOG_ROTATE_SIZE")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(10 * 1024 * 1024) // Default: 10MB
}

/// Get log retention period in days
fn log_retention_days() -> u64 {
    env::var("LOG_RETENTION_DAYS")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(30) // Default: 30 days
}

/// Rotate log files if current file exceeds size limit
fn rotate_logs_if_needed() -> Result<(), String> {
    let log_file_path = get_log_file_path();
    
    if !log_file_path.exists() {
        // File doesn't exist yet, just clean up old logs
        cleanup_old_logs()?;
        return Ok(());
    }

    let metadata = std::fs::metadata(&log_file_path)
        .map_err(|e| format!("Failed to get log file metadata: {}", e))?;
    
    if metadata.len() >= max_log_file_size() {
        rotate_log_file(&log_file_path)?;
    }
    
    // Clean up old log files
    cleanup_old_logs()?;
    
    Ok(())
}

/// Rotate the current log file by appending timestamp
fn rotate_log_file(log_file_path: &Path) -> Result<(), String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let rotated_name = format!("{}.{}", log_file_path.display(), timestamp);
    std::fs::rename(log_file_path, &rotated_name)
        .map_err(|e| format!("Failed to rotate log file: {}", e))?;
    
    Ok(())
}

/// Clean up log files older than retention period
fn cleanup_old_logs() -> Result<(), String> {
    let log_dir = get_log_directory();
    let retention_days = log_retention_days();
    let cutoff_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        - (retention_days * 24 * 60 * 60);

    let entries = std::fs::read_dir(&log_dir)
        .map_err(|e| format!("Failed to read log directory: {}", e))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
                if let Ok(metadata) = path.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(modified_secs) = modified.duration_since(UNIX_EPOCH) {
                            if modified_secs.as_secs() < cutoff_time {
                                let _ = std::fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Write log entry to file
fn write_to_file(entry: &LogEntry) -> Result<(), io::Error> {
    if !should_log_to_file() {
        return Ok(());
    }

    // Initialize file handle if needed
    {
        let handle = LOG_FILE_HANDLE.lock().unwrap();
        if handle.is_none() {
            // Try to initialize (best effort, don't fail if it doesn't work)
            let _ = initialize_file_logging();
        }
    }

    let mut handle_guard = LOG_FILE_HANDLE.lock().unwrap();
    if let Some(ref mut file) = *handle_guard {
        // Format log entry as JSON for easy parsing
        let json_entry = format!(
            r#"{{"timestamp":{},"level":"{:?}","message":"{}","source":"{}","data":{}}}\n"#,
            entry.timestamp,
            entry.level,
            entry.message.replace('"', "\\\""),
            entry.source.replace('"', "\\\""),
            format_data_as_json(&entry.data)
        );
        
        file.write_all(json_entry.as_bytes())?;
        file.flush()?;
    }
    
    Ok(())
}

/// Format log data HashMap as JSON string
fn format_data_as_json(data: &HashMap<String, Value>) -> String {
    let mut parts = Vec::new();
    for (key, value) in data {
        let value_str = match value {
            Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => format!("\"{:?}\"", value),
        };
        parts.push(format!("\"{}\":{}", key.replace('"', "\\\""), value_str));
    }
    format!("{{{}}}", parts.join(","))
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

fn log_level_order(level: &LogLevel) -> u8 {
    match level {
        LogLevel::Debug => 0,
        LogLevel::Info | LogLevel::Audit => 1,
        LogLevel::Warning => 2,
        LogLevel::Error => 3,
    }
}

fn max_entries() -> usize {
    env::var("LOG_MAX_ENTRIES")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1000)
}

fn sink_console() -> bool {
    match env::var("LOG_SINK").as_deref() {
        Ok("none") | Ok("file") => false, // Only file, no console
        Ok("both") => true, // Both console and file
        _ => true, // Default: console only
    }
}

fn min_level_order() -> u8 {
    match env::var("LOG_LEVEL").as_deref() {
        Ok("debug") => 0,
        Ok("info") | Ok("audit") => 1,
        Ok("warning") | Ok("warn") => 2,
        Ok("error") => 3,
        _ => 0,
    }
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

    // Store in memory
    if let Ok(mut storage) = LOG_STORAGE.lock() {
        storage.push(entry.clone());
        let cap = max_entries();
        while storage.len() > cap {
            storage.remove(0);
        }
    }

    // Write to file (especially important for audit logs)
    if level == LogLevel::Audit || should_log_to_file() {
        let _ = write_to_file(&entry);
    }

    // Output to console
    if sink_console() && log_level_order(&level) >= min_level_order() {
        let level_str = match level {
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Audit => "AUDIT",
            LogLevel::Debug => "DEBUG",
        };
        println!("[{}] {}: {} - {:?}", level_str, source, message, data);
    }
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

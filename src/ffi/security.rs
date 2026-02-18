// FFI Security Module
// Provides security controls for FFI operations

use crate::runtime::values::Value;
use std::time::{Duration, Instant};

/// Resource limits for FFI execution
#[derive(Debug, Clone)]
pub struct FFIResourceLimits {
    pub max_execution_time: Duration,
    pub max_memory_bytes: usize,
    pub max_stack_depth: usize,
    pub max_recursion_depth: usize,
    pub max_input_size: usize,
}

impl Default for FFIResourceLimits {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            max_memory_bytes: 100_000_000, // 100MB
            max_stack_depth: 1000,
            max_recursion_depth: 100,
            max_input_size: 10_000_000, // 10MB
        }
    }
}

/// Input validator for FFI
pub struct FFIInputValidator;

impl FFIInputValidator {
    /// Validate source code input
    pub fn validate_source(source: &str, limits: &FFIResourceLimits) -> Result<(), String> {
        // Check input size
        if source.len() > limits.max_input_size {
            return Err(format!(
                "Input too large: {} bytes (max: {} bytes)",
                source.len(),
                limits.max_input_size
            ));
        }

        // Check for null bytes (potential security issue)
        if source.contains('\0') {
            return Err("Input contains null bytes".to_string());
        }

        // Check for extremely long lines (potential DoS)
        for line in source.lines() {
            if line.len() > 1_000_000 {
                return Err("Line too long (potential DoS)".to_string());
            }
        }

        Ok(())
    }

    /// Validate value inputs
    pub fn validate_value(value: &Value, limits: &FFIResourceLimits) -> Result<(), String> {
        match value {
            Value::String(s) => {
                if s.len() > limits.max_input_size {
                    return Err("String value too large".to_string());
                }
                if s.contains('\0') {
                    return Err("String contains null bytes".to_string());
                }
            }
            Value::List(list) => {
                if list.len() > 100_000 {
                    return Err("List too large".to_string());
                }
                for item in list {
                    Self::validate_value(item, limits)?;
                }
            }
            Value::Array(arr) => {
                if arr.len() > 100_000 {
                    return Err("Array too large".to_string());
                }
                for item in arr {
                    Self::validate_value(item, limits)?;
                }
            }
            Value::Map(map) => {
                if map.len() > 100_000 {
                    return Err("Map too large".to_string());
                }
                for (k, v) in map {
                    if k.len() > 10_000 {
                        return Err("Map key too long".to_string());
                    }
                    Self::validate_value(v, limits)?;
                }
            }
            _ => {} // Other types are safe
        }

        Ok(())
    }

    /// Sanitize string input
    pub fn sanitize_string(input: &str) -> String {
        // Remove null bytes
        input.chars().filter(|c| *c != '\0').collect()
    }
}

/// Execution monitor for FFI operations
pub struct FFIExecutionMonitor {
    start_time: Instant,
    limits: FFIResourceLimits,
}

impl FFIExecutionMonitor {
    pub fn new(limits: FFIResourceLimits) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
        }
    }

    /// Check if execution time limit exceeded
    pub fn check_timeout(&self) -> Result<(), String> {
        if self.start_time.elapsed() > self.limits.max_execution_time {
            return Err("Execution timeout exceeded".to_string());
        }
        Ok(())
    }

    /// Check remaining time
    pub fn remaining_time(&self) -> Duration {
        self.limits
            .max_execution_time
            .saturating_sub(self.start_time.elapsed())
    }
}

/// Sandbox configuration for FFI execution
#[derive(Debug, Clone)]
pub struct FFISandbox {
    pub allow_file_access: bool,
    pub allow_network_access: bool,
    pub allow_system_calls: bool,
    pub allowed_paths: Vec<String>,
}

impl Default for FFISandbox {
    fn default() -> Self {
        Self {
            allow_file_access: false,
            allow_network_access: false,
            allow_system_calls: false,
            allowed_paths: vec![],
        }
    }
}

impl FFISandbox {
    /// Check if a file path is allowed
    pub fn is_path_allowed(&self, path: &str) -> bool {
        if !self.allow_file_access {
            return false;
        }

        if self.allowed_paths.is_empty() {
            return true; // All paths allowed if list is empty
        }

        // Check if path is in allowed list
        for allowed in &self.allowed_paths {
            if path.starts_with(allowed) {
                return true;
            }
        }

        false
    }

    /// Check if network access is allowed
    pub fn is_network_allowed(&self) -> bool {
        self.allow_network_access
    }

    /// Check if system calls are allowed
    pub fn is_system_calls_allowed(&self) -> bool {
        self.allow_system_calls
    }
}

/// Security context for FFI operations
#[derive(Debug, Clone)]
pub struct FFISecurityContext {
    pub limits: FFIResourceLimits,
    pub sandbox: FFISandbox,
    pub enable_validation: bool,
}

impl Default for FFISecurityContext {
    fn default() -> Self {
        Self {
            limits: FFIResourceLimits::default(),
            sandbox: FFISandbox::default(),
            enable_validation: true,
        }
    }
}

impl FFISecurityContext {
    /// Create a strict security context (maximum security)
    pub fn strict() -> Self {
        Self {
            limits: FFIResourceLimits {
                max_execution_time: Duration::from_secs(10),
                max_memory_bytes: 10_000_000, // 10MB
                max_stack_depth: 100,
                max_recursion_depth: 10,
                max_input_size: 1_000_000, // 1MB
            },
            sandbox: FFISandbox::default(), // No access by default
            enable_validation: true,
        }
    }

    /// Create a permissive security context (for trusted code)
    pub fn permissive() -> Self {
        Self {
            limits: FFIResourceLimits {
                max_execution_time: Duration::from_secs(300), // 5 minutes
                max_memory_bytes: 1_000_000_000,              // 1GB
                max_stack_depth: 10000,
                max_recursion_depth: 1000,
                max_input_size: 100_000_000, // 100MB
            },
            sandbox: FFISandbox {
                allow_file_access: true,
                allow_network_access: true,
                allow_system_calls: true,
                allowed_paths: vec![],
            },
            enable_validation: false,
        }
    }
}

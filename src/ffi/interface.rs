// Unified interface abstraction for HTTP and FFI
use crate::runtime::values::Value;
use std::collections::HashMap;
#[cfg(feature = "http-interface")]
use log;

/// Interface type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceType {
    /// HTTP/REST API only
    HTTP,
    /// FFI (Foreign Function Interface) only
    FFI,
    /// Both HTTP and FFI available
    Both,
}

/// Service interface abstraction
pub trait ServiceInterface: Send + Sync {
    /// Call a service function
    fn call_function(
        &self,
        service_name: &str,
        function_name: &str,
        args: &[Value],
    ) -> Result<Value, String>;

    /// Get interface type
    fn interface_type(&self) -> InterfaceType;

    /// Check if interface is available
    fn is_available(&self) -> bool;
}

/// FFI Interface manager
pub struct FFIInterface {
    config: crate::ffi::FFIConfig,
    http_interface: Option<Box<dyn ServiceInterface>>,
    ffi_interface: Option<Box<dyn ServiceInterface>>,
}

impl FFIInterface {
    pub fn new(config: crate::ffi::FFIConfig) -> Self {
        let mut http_interface = None;
        let mut ffi_interface = None;

        if config.enable_http {
            http_interface = Some(Box::new(HttpInterface::new()) as Box<dyn ServiceInterface>);
        }

        if config.enable_ffi {
            if config.rust_enabled {
                ffi_interface = Some(Box::new(RustFFIInterface::new()) as Box<dyn ServiceInterface>);
            }
        }

        Self {
            config,
            http_interface,
            ffi_interface,
        }
    }

    /// Call function using the appropriate interface
    /// If no explicit preference, automatically chooses based on heuristics
    pub fn call(
        &self,
        service_name: &str,
        function_name: &str,
        args: &[Value],
        prefer_ffi: Option<bool>,
    ) -> Result<Value, String> {
        // Auto-detect if no preference specified
        let should_use_ffi = prefer_ffi.unwrap_or_else(|| {
            self.auto_detect_interface(service_name, function_name, args)
        });

        match self.config.interface_type {
            InterfaceType::HTTP => {
                if let Some(ref http) = self.http_interface {
                    http.call_function(service_name, function_name, args)
                } else {
                    Err("HTTP interface not available".to_string())
                }
            }
            InterfaceType::FFI => {
                if let Some(ref ffi) = self.ffi_interface {
                    ffi.call_function(service_name, function_name, args)
                } else {
                    Err("FFI interface not available".to_string())
                }
            }
            InterfaceType::Both => {
                // Auto-select based on heuristics or preference
                if should_use_ffi {
                    // Try FFI first
                    if let Some(ref ffi) = self.ffi_interface {
                        if ffi.is_available() {
                            match ffi.call_function(service_name, function_name, args) {
                                Ok(result) => return Ok(result),
                                Err(e) => {
                                    // FFI failed, fallback to HTTP
                                    #[cfg(feature = "http-interface")]
                                    log::warn!("FFI call failed, falling back to HTTP: {}", e);
                                    // Continue to HTTP fallback
                                }
                            }
                        }
                    }
                }

                // Use HTTP (or FFI if HTTP not available)
                if let Some(ref http) = self.http_interface {
                    http.call_function(service_name, function_name, args)
                } else if let Some(ref ffi) = self.ffi_interface {
                    ffi.call_function(service_name, function_name, args)
                } else {
                    Err("No interface available".to_string())
                }
            }
        }
    }

    /// Auto-detect which interface to use based on heuristics
    fn auto_detect_interface(
        &self,
        service_name: &str,
        function_name: &str,
        args: &[Value],
    ) -> bool {
        // Heuristic 1: Check if this is a local call (same process)
        // If we're in the same process, prefer FFI
        if self.ffi_interface.is_some() {
            // Heuristic 2: Function name patterns that suggest high-frequency operations
            let ffi_preferred_patterns = [
                "hash", "sign", "verify", "encrypt", "decrypt",
                "compute", "calculate", "process", "transform",
                "batch_", "parallel_", "fast_",
            ];
            
            for pattern in &ffi_preferred_patterns {
                if function_name.contains(pattern) {
                    return true; // Prefer FFI for compute-intensive operations
                }
            }

            // Heuristic 3: Small argument size suggests local operation
            // Large arguments might benefit from HTTP's serialization
            let total_arg_size: usize = args.iter()
                .map(|v| self.estimate_value_size(v))
                .sum();
            
            if total_arg_size < 1024 {
                // Small data - prefer FFI (low serialization overhead)
                return true;
            }
        }

        // Heuristic 4: Network-bound operations prefer HTTP
        let http_preferred_patterns = [
            "chain::", "database::", "network_", "remote_",
            "fetch", "request", "api_", "http_",
        ];
        
        for pattern in &http_preferred_patterns {
            if function_name.contains(pattern) || service_name.contains(pattern) {
                return false; // Prefer HTTP for network operations
            }
        }

        // Default: Prefer FFI if available (better performance)
        // Fallback to HTTP if FFI not available
        self.ffi_interface.is_some() && self.ffi_interface.as_ref().unwrap().is_available()
    }

    /// Estimate the size of a value in bytes (rough approximation)
    fn estimate_value_size(&self, value: &Value) -> usize {
        match value {
            Value::Int(_) => 8,
            Value::Float(_) => 8,
            Value::Bool(_) => 1,
            Value::Null => 0,
            Value::String(s) => s.len(),
            Value::List(arr) => arr.iter().map(|v| self.estimate_value_size(v)).sum(),
            Value::Array(arr) => arr.iter().map(|v| self.estimate_value_size(v)).sum(),
            Value::Map(map) => {
                map.iter()
                    .map(|(k, v)| k.len() + self.estimate_value_size(v))
                    .sum()
            }
            Value::Result(ok_val, err_val) => {
                self.estimate_value_size(ok_val) + self.estimate_value_size(err_val)
            }
            Value::Option(opt) => {
                opt.as_ref().map(|v| self.estimate_value_size(v)).unwrap_or(0)
            }
            Value::Set(set) => set.len() * 8, // Rough estimate
            Value::Struct(_, fields) => {
                fields.iter().map(|(k, v)| k.len() + self.estimate_value_size(v)).sum()
            }
            Value::Closure(id) => id.len() + 8,
        }
    }
}

/// HTTP Interface implementation
#[cfg(feature = "http-interface")]
struct HttpInterface {
    base_url: String,
}

#[cfg(feature = "http-interface")]
impl HttpInterface {
    fn new() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
        }
    }
}

#[cfg(feature = "http-interface")]
impl ServiceInterface for HttpInterface {
    fn call_function(
        &self,
        service_name: &str,
        function_name: &str,
        args: &[Value],
    ) -> Result<Value, String> {
        // Serialize arguments to JSON
        let json_args: Vec<serde_json::Value> = args
            .iter()
            .map(|v| value_to_json(v))
            .collect();

        // Make HTTP request
        let url = format!("{}/api/{}/{}", self.base_url, service_name, function_name);
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&url)
            .json(&json_args)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let result: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Convert JSON back to Value
        json_to_value(&result)
    }

    fn interface_type(&self) -> InterfaceType {
        InterfaceType::HTTP
    }

    fn is_available(&self) -> bool {
        true // HTTP is always available if server is running
    }
}

// Stub when HTTP interface is not available
#[cfg(not(feature = "http-interface"))]
struct HttpInterface;

#[cfg(not(feature = "http-interface"))]
impl HttpInterface {
    fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "http-interface"))]
impl ServiceInterface for HttpInterface {
    fn call_function(
        &self,
        _service_name: &str,
        _function_name: &str,
        _args: &[Value],
    ) -> Result<Value, String> {
        Err("HTTP interface not available (compile with 'http-interface' feature)".to_string())
    }

    fn interface_type(&self) -> InterfaceType {
        InterfaceType::HTTP
    }

    fn is_available(&self) -> bool {
        false
    }
}

/// Rust FFI Interface implementation
struct RustFFIInterface;

impl RustFFIInterface {
    fn new() -> Self {
        Self
    }
}

impl ServiceInterface for RustFFIInterface {
    fn call_function(
        &self,
        _service_name: &str,
        _function_name: &str,
        _args: &[Value],
    ) -> Result<Value, String> {
        // Direct function call - zero overhead
        // This would call the actual runtime directly
        // Note: This is a simplified implementation
        // In a real implementation, you'd maintain a runtime instance
        // For now, return an error indicating FFI needs runtime integration
        Err("FFI interface requires runtime integration - use execute_source or execute_program".to_string())
    }

    fn interface_type(&self) -> InterfaceType {
        InterfaceType::FFI
    }

    fn is_available(&self) -> bool {
        true // Native FFI is always available
    }
}

// Helper functions for JSON conversion
pub fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0)),
        ),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Null => serde_json::Value::Null,
        Value::List(arr) => {
            serde_json::Value::Array(arr.iter().map(value_to_json).collect())
        }
        Value::Map(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                json_map.insert(k.clone(), value_to_json(v));
            }
            serde_json::Value::Object(json_map)
        }
        Value::Result(ok_val, err_val) => {
            // Serialize Result as object
            let mut json_map = serde_json::Map::new();
            json_map.insert("ok".to_string(), value_to_json(ok_val));
            json_map.insert("err".to_string(), value_to_json(err_val));
            serde_json::Value::Object(json_map)
        }
        Value::Option(opt) => {
            match opt {
                Some(v) => value_to_json(v),
                None => serde_json::Value::Null,
            }
        }
        Value::Set(set) => {
            serde_json::Value::Array(set.iter().map(|s| serde_json::Value::String(s.clone())).collect())
        }
        Value::Struct(name, fields) => {
            let mut json_map = serde_json::Map::new();
            json_map.insert("_type".to_string(), serde_json::Value::String(name.clone()));
            for (k, v) in fields {
                json_map.insert(k.clone(), value_to_json(v));
            }
            serde_json::Value::Object(json_map)
        }
        Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(value_to_json).collect())
        }
        Value::Closure(id) => serde_json::Value::String(format!("<closure {}>", id)),
    }
}

/// Convert serde_json::Value to DAL Value. Used by json::parse.
pub fn json_to_value(json: &serde_json::Value) -> Result<Value, String> {
    match json {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err("Invalid number".to_string())
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s.clone())),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Null => Ok(Value::Null),
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<Value>, String> =
                arr.iter().map(json_to_value).collect();
            Ok(Value::List(values?))
        }
        serde_json::Value::Object(map) => {
            // Check if it's a Result or Struct
            if let Some(type_val) = map.get("_type") {
                if let Some(type_str) = type_val.as_str() {
                    let mut fields = HashMap::new();
                    for (k, v) in map {
                        if k != "_type" {
                            fields.insert(k.clone(), json_to_value(v)?);
                        }
                    }
                    return Ok(Value::Struct(type_str.to_string(), fields));
                }
            }
            
            // Regular map
            let mut value_map = HashMap::new();
            for (k, v) in map {
                value_map.insert(k.clone(), json_to_value(v)?);
            }
            Ok(Value::Map(value_map))
        }
    }
}

// Rust FFI bindings (C-compatible)
// Provides direct function calls for Rust codebases

use crate::runtime::values::Value;
use crate::runtime::engine::Runtime;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Rust FFI runtime handle
pub struct RustFFIRuntime {
    runtime: Runtime,
}

impl RustFFIRuntime {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    /// Call a service function
    pub fn call_function(
        &mut self,
        _service_name: &str,
        _function_name: &str,
        _args: &[Value],
    ) -> Result<Value, String> {
        // Note: Runtime doesn't have execute_function method
        // This would need to be implemented via execute_source or execute_program
        Err("FFI call_function requires runtime integration - use execute_source instead".to_string())
    }

    /// Execute dist_agent_lang source code with security checks
    pub fn execute(&mut self, source: &str) -> Result<Value, String> {
        // Apply security checks
        use crate::ffi::security::{FFIInputValidator, FFIResourceLimits};
        let limits = FFIResourceLimits::default();
        
        // Validate input
        FFIInputValidator::validate_source(source, &limits)?;
        
        // Parse and execute source
        use crate::parse_source;
        let program = parse_source(source)
            .map_err(|e| format!("Parse error: {}", e))?;
        self.runtime
            .execute_program(program)
            .map_err(|e| format!("Execution error: {}", e))?;
        Ok(Value::Null) // Return null for now - would need to capture return value
    }
}

// C-compatible FFI functions
#[no_mangle]
pub extern "C" fn dist_agent_lang_runtime_new() -> *mut RustFFIRuntime {
    Box::into_raw(Box::new(RustFFIRuntime::new()))
}

#[no_mangle]
pub extern "C" fn dist_agent_lang_runtime_free(ptr: *mut RustFFIRuntime) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn dist_agent_lang_hash(
    data: *const u8,
    data_len: usize,
    algorithm: *const c_char,
    output: *mut c_char,
    output_len: usize,
) -> c_int {
    unsafe {
        if data.is_null() || algorithm.is_null() || output.is_null() {
            return -1;
        }

        // Security check: validate input size
        use crate::ffi::security::FFIResourceLimits;
        let limits = FFIResourceLimits::default();
        if data_len > limits.max_input_size {
            return -4; // Input too large
        }

        let data_slice = std::slice::from_raw_parts(data, data_len);
        let algo_str = CStr::from_ptr(algorithm)
            .to_str()
            .unwrap_or("SHA256");

        match crate::stdlib::crypto::hash_bytes(data_slice, algo_str) {
            Ok(hash) => {
                let hash_bytes = hash.as_bytes();
                if hash_bytes.len() >= output_len {
                    return -2; // Buffer too small
                }
                std::ptr::copy_nonoverlapping(
                    hash_bytes.as_ptr(),
                    output as *mut u8,
                    hash_bytes.len().min(output_len - 1),
                );
                *output.add(hash_bytes.len()) = 0; // Null terminator
                0
            }
            Err(_) => -3,
        }
    }
}

#[no_mangle]
pub extern "C" fn dist_agent_lang_sign(
    data: *const u8,
    data_len: usize,
    private_key: *const u8,
    key_len: usize,
    signature: *mut u8,
    sig_len: usize,
) -> c_int {
    unsafe {
        if data.is_null() || private_key.is_null() || signature.is_null() {
            return -1;
        }

        // Security check: validate input sizes
        use crate::ffi::security::FFIResourceLimits;
        let limits = FFIResourceLimits::default();
        if data_len > limits.max_input_size || key_len > limits.max_input_size {
            return -4; // Input too large
        }

        let data_slice = std::slice::from_raw_parts(data, data_len);
        let key_slice = std::slice::from_raw_parts(private_key, key_len);

        let key_str = std::str::from_utf8(key_slice).unwrap_or("");
        match crate::stdlib::crypto_signatures::sign(data_slice, key_str) {
            Ok(sig) => {
                if sig.len() > sig_len {
                    return -2; // Buffer too small
                }
                std::ptr::copy_nonoverlapping(sig.as_ptr(), signature, sig.len());
                sig.len() as c_int
            }
            Err(_) => -3,
        }
    }
}

#[no_mangle]
pub extern "C" fn dist_agent_lang_verify(
    data: *const u8,
    data_len: usize,
    signature: *const u8,
    sig_len: usize,
    public_key: *const u8,
    key_len: usize,
) -> c_int {
    unsafe {
        if data.is_null() || signature.is_null() || public_key.is_null() {
            return -1;
        }

        // Security check: validate input sizes
        use crate::ffi::security::FFIResourceLimits;
        let limits = FFIResourceLimits::default();
        if data_len > limits.max_input_size || 
           sig_len > limits.max_input_size || 
           key_len > limits.max_input_size {
            return -4; // Input too large
        }

        let data_slice = std::slice::from_raw_parts(data, data_len);
        let sig_slice = std::slice::from_raw_parts(signature, sig_len);
        let key_slice = std::slice::from_raw_parts(public_key, key_len);

        let sig_str = std::str::from_utf8(sig_slice).unwrap_or("");
        let key_str = std::str::from_utf8(key_slice).unwrap_or("");
        match crate::stdlib::crypto_signatures::verify(data_slice, sig_str, key_str) {
            Ok(valid) => if valid { 1 } else { 0 },
            Err(_) => -2,
        }
    }
}

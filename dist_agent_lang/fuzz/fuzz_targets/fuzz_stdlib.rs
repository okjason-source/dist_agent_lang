#![no_main]

use libfuzzer_sys::fuzz_target;
use dist_agent_lang::runtime::values::Value;

fuzz_target!(|data: &[u8]| {
    // Fuzz standard library functions with arbitrary input
    // Test string operations
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = Value::String(s.to_string());
    }
    
    // Test numeric operations with arbitrary bytes
    if data.len() >= 8 {
        // Try to interpret as i64
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[..8]);
        let num = i64::from_le_bytes(bytes);
        let _ = Value::Int(num);
    }
    
    // Test with various data sizes
    if !data.is_empty() {
        let _ = Value::String(format!("{:?}", data));
    }
});


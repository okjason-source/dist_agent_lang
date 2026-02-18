// Rust FFI Example - Native Integration
// Demonstrates using dist_agent_lang via FFI from Rust code

use dist_agent_lang::ffi::rust::RustFFIRuntime;
use dist_agent_lang::runtime::values::Value;

fn main() {
    // Example 1: Direct FFI calls (zero overhead)
    let mut runtime = RustFFIRuntime::new();

    // Call service function directly
    let args = vec![Value::String("Hello, World!".to_string())];

    match runtime.call_function("CryptoService", "hash_data", &args) {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 2: Execute dist_agent_lang source
    let source = r#"
        fn compute(x: int) -> int {
            return x * 2;
        }
        compute(42)
    "#;

    match runtime.execute(source) {
        Ok(result) => println!("Computed: {:?}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 3: C-compatible FFI functions
    use std::ffi::CString;

    let data = b"Hello, World!";
    let algorithm = CString::new("SHA256").unwrap();
    let mut output = vec![0u8; 64];

    unsafe {
        let result = dist_agent_lang::ffi::rust::dist_agent_lang_hash(
            data.as_ptr(),
            data.len(),
            algorithm.as_ptr(),
            output.as_mut_ptr() as *mut i8,
            output.len(),
        );

        if result == 0 {
            let hash_str = CString::from_vec_unchecked(output)
                .to_string_lossy()
                .to_string();
            println!("Hash: {}", hash_str);
        } else {
            eprintln!("Hash failed with code: {}", result);
        }
    }
}

// Example 4: Performance comparison
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_ffi_performance() {
        let mut runtime = RustFFIRuntime::new();
        let args = vec![Value::String("test_data".to_string())];
        let iterations = 1_000_000;

        let start = Instant::now();
        for _ in 0..iterations {
            runtime
                .call_function("CryptoService", "hash_data", &args)
                .unwrap();
        }
        let duration = start.elapsed();

        println!(
            "FFI: {} ops in {:?} ({:.0} ops/sec)",
            iterations,
            duration,
            iterations as f64 / duration.as_secs_f64()
        );
    }
}

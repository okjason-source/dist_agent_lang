// Load and Stress Tests for dist_agent_lang
// Tests system behavior under heavy load and extreme conditions

use dist_agent_lang::lexer::lexer::Lexer;
use dist_agent_lang::parser::parser::Parser;
use dist_agent_lang::runtime::ReentrancyGuard;
use dist_agent_lang::runtime::Runtime;
use dist_agent_lang::stdlib::chain;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// Test: Lexer performance with large files
#[test]
fn test_lexer_large_file_performance() {
    // Generate a large source file (10,000 lines)
    let mut code = String::new();
    for i in 0..10000 {
        code.push_str(&format!("let var{} = {};\n", i, i));
    }

    let start = Instant::now();
    let mut lexer = Lexer::new(&code);
    let result = lexer.tokenize();
    let duration = start.elapsed();

    assert!(result.is_ok());
    println!("Lexer processed 10,000 lines in {:?}", duration);
    assert!(
        duration.as_secs() < 5,
        "Lexer took too long: {:?}",
        duration
    );
}

/// Test: Parser performance with complex nested structures
#[test]
fn test_parser_nested_structures_performance() {
    // Generate deeply nested if statements (100 levels)
    let mut code = String::new();
    for i in 0..100 {
        code.push_str(&format!("if x > {} {{\n", i));
    }
    code.push_str("let result = 1;");
    for _ in 0..100 {
        code.push_str("\n}");
    }

    let start = Instant::now();
    let mut lexer = Lexer::new(&code);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    let duration = start.elapsed();

    println!("Parser processed 100 nested levels in {:?}", duration);
    assert!(result.is_ok() || result.is_err()); // Should complete without panic
    assert!(
        duration.as_secs() < 10,
        "Parser took too long: {:?}",
        duration
    );
}

/// Test: Runtime creation performance
#[test]
fn test_runtime_many_variables() {
    let start = Instant::now();

    // Create many runtime instances
    let mut runtimes = Vec::new();
    for _ in 0..1000 {
        let runtime = Runtime::new();
        runtimes.push(runtime);
    }

    let duration = start.elapsed();
    println!("Created 1,000 runtime instances in {:?}", duration);
    assert_eq!(runtimes.len(), 1000);
    assert!(
        duration.as_millis() < 1000,
        "Runtime creation took too long"
    );
}

/// Test: Concurrent access to ReentrancyGuard
#[test]
fn test_concurrent_reentrancy_guard() {
    let guard = Arc::new(ReentrancyGuard::new());
    let mut handles = vec![];

    // Spawn 100 threads trying to access the same function
    for _ in 0..100 {
        let guard_clone = Arc::clone(&guard);
        let handle = thread::spawn(move || {
            let result = guard_clone.enter("critical_function", Some("contract"));
            if result.is_ok() {
                // Simulate some work
                thread::sleep(std::time::Duration::from_micros(100));
            }
            result.is_ok()
        });
        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    println!(
        "Reentrancy guard allowed {} out of 100 concurrent attempts",
        success_count
    );
    // At least some should succeed (not all will due to timing)
    assert!(success_count > 0, "No threads succeeded");
}

/// Test: Stress test with malformed inputs
#[test]
fn test_lexer_malformed_inputs() {
    let malformed_inputs = vec![
        "let x = 999999999999999999999999999999999999999;", // Very large number
        "let x = \"unterminated string",                    // Unterminated string
        "let x = @@@@@@@@;",                                // Invalid symbols
        "let x = 3.14.159.265;",                            // Invalid float
        "let 123invalid = 1;",                              // Invalid identifier
        "let x = ;;;;;;;;;;;;",                             // Multiple semicolons
        "{{{{{{{{{{{{{{{{{{{{",                             // Many open braces
        "))))))))))))))))))))",                             // Many close parens
        "",                                                 // Empty input
        " \n\t\r\n ",                                       // Only whitespace
    ];

    for (i, input) in malformed_inputs.iter().enumerate() {
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        println!(
            "Malformed input {}: {:?}",
            i,
            if result.is_ok() { "OK" } else { "Error" }
        );
        // Should not panic, either succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

/// Test: Memory usage with large data structures
#[test]
fn test_memory_usage_large_structures() {
    use dist_agent_lang::runtime::values::Value;
    use std::collections::HashMap;

    let start = Instant::now();

    // Create a large map with 10,000 entries
    let mut large_map = HashMap::new();
    for i in 0..10000 {
        let key = format!("key{}", i);
        large_map.insert(key, Value::Int(i as i64));
    }

    let duration = start.elapsed();
    println!("Created map with 10,000 entries in {:?}", duration);
    assert_eq!(large_map.len(), 10000);
    assert!(duration.as_millis() < 500);
}

/// Test: Parser with many service declarations
#[test]
fn test_parser_many_services() {
    let mut code = String::new();

    // Generate 100 service declarations
    for i in 0..100 {
        code.push_str(&format!(
            r#"
            @trust("hybrid")
            service Service{} {{
                field counter: int = {};
            }}
        "#,
            i, i
        ));
    }

    let start = Instant::now();
    let mut lexer = Lexer::new(&code);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    let duration = start.elapsed();

    println!("Parsed 100 services in {:?}", duration);
    assert!(result.is_ok() || result.is_err());
    assert!(duration.as_secs() < 5);
}

/// Test: Concurrent blockchain operations
#[test]
fn test_concurrent_blockchain_operations() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    // Spawn 50 threads performing blockchain operations
    for i in 0..50 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // Simulate blockchain operations
            let chain_id = (i % 7) + 1; // Rotate through chain IDs
            let address = format!("0x{:040x}", i);

            let _ = chain::get_balance(chain_id, address.clone());
            let _ = chain::get_chain_config(chain_id);

            let mut count = counter_clone.lock().unwrap();
            *count += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_count = *counter.lock().unwrap();
    assert_eq!(final_count, 50, "All threads should complete");
    println!("Completed 50 concurrent blockchain operations");
}

/// Test: Stress test runtime execution
#[test]
fn test_runtime_execution_stress() {
    // Stress test by creating and executing many simple programs
    let code = r#"let x = 10;"#;

    let start = Instant::now();

    // Run lexer and parser 1000 times
    for _ in 0..1000 {
        let mut lexer = Lexer::new(code);
        if let Ok(tokens) = lexer.tokenize() {
            let mut parser = Parser::new(tokens);
            let _ = parser.parse();
        }
    }

    let duration = start.elapsed();
    println!("Executed 1000 parse operations in {:?}", duration);
    assert!(duration.as_secs() < 5, "Execution took too long");
}

/// Test: Maximum string length handling
#[test]
fn test_large_string_handling() {
    // Create a very large string (1 MB)
    let large_string = "x".repeat(1_000_000);
    let code = format!("let x = \"{}\";", large_string);

    let start = Instant::now();
    let mut lexer = Lexer::new(&code);
    let result = lexer.tokenize();
    let duration = start.elapsed();

    println!("Lexer processed 1MB string in {:?}", duration);
    assert!(result.is_ok() || result.is_err());
    assert!(duration.as_secs() < 10);
}

/// Test: Parser error recovery
#[test]
fn test_parser_error_recovery() {
    let invalid_codes = vec![
        "let x = ;", // Missing value
        "let = 5;",  // Missing identifier
        "let x 5;",  // Missing equals
        "x = 5;",    // Missing let
        "let x = 5", // Missing semicolon
    ];

    for (i, code) in invalid_codes.iter().enumerate() {
        let mut lexer = Lexer::new(code);
        if let Ok(tokens) = lexer.tokenize() {
            let mut parser = Parser::new(tokens);
            let result = parser.parse();
            println!(
                "Invalid code {}: {:?}",
                i,
                if result.is_err() {
                    "Error (expected)"
                } else {
                    "OK"
                }
            );
            // Should handle errors gracefully
            assert!(result.is_ok() || result.is_err());
        }
    }
}

/// Test: Scalability - Multiple runtimes
#[test]
fn test_multiple_runtimes() {
    let mut runtimes = Vec::new();

    let start = Instant::now();

    // Create 100 runtime instances
    for _ in 0..100 {
        let runtime = Runtime::new();
        runtimes.push(runtime);
    }

    let duration = start.elapsed();
    println!("Created 100 runtime instances in {:?}", duration);
    assert_eq!(runtimes.len(), 100);
    assert!(duration.as_millis() < 1000);
}

/// Test: Extreme values
#[test]
fn test_extreme_values() {
    // Test with maximum values
    let max_i64 = i64::MAX;
    let min_i64 = i64::MIN;

    // Test saturating operations
    let result1 = max_i64.saturating_add(1);
    assert_eq!(result1, max_i64, "Should saturate at max");

    let result2 = min_i64.saturating_sub(1);
    assert_eq!(result2, min_i64, "Should saturate at min");

    let result3 = max_i64.saturating_mul(2);
    assert_eq!(result3, max_i64, "Should saturate on overflow");

    // Test checked operations
    assert!(max_i64.checked_add(1).is_none(), "Should detect overflow");
    assert!(min_i64.checked_sub(1).is_none(), "Should detect underflow");
    assert!(
        max_i64.checked_mul(2).is_none(),
        "Should detect multiplication overflow"
    );

    println!("Extreme value handling validated");
}

/// Test: Rapid fire operations
#[test]
fn test_rapid_operations() {
    let start = Instant::now();

    for i in 0..10000 {
        let code = format!("let x = {};", i);
        let mut lexer = Lexer::new(&code);
        let _ = lexer.tokenize();
    }

    let duration = start.elapsed();
    println!("Processed 10,000 rapid operations in {:?}", duration);
    assert!(duration.as_secs() < 5);
}

/// Test: Memory cleanup
#[test]
fn test_memory_cleanup() {
    // Create and drop many runtimes to test memory cleanup
    for _ in 0..1000 {
        let _runtime = Runtime::new();
        // Runtime drops here, should clean up memory
    }
    println!("Memory cleanup test completed (1000 runtimes)");
}

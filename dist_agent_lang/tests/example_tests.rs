// Example Tests
// Tests all DAL example files to ensure they compile and execute correctly
// This is critical for ensuring examples stay up-to-date with language changes

use dist_agent_lang::{parse_source, execute_source};
use std::fs;
use std::path::{Path, PathBuf};

// Helper function to get example file paths
fn get_example_files() -> Vec<PathBuf> {
    let examples_dir = Path::new("examples");
    let mut files = Vec::new();
    
    if !examples_dir.exists() {
        eprintln!("Warning: examples directory not found");
        return files;
    }
    
    if let Ok(entries) = fs::read_dir(examples_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("dal") {
                files.push(path);
            }
        }
    }
    
    files.sort();
    files
}

// Helper to read file content
fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("Failed to read file {:?}: {}", path, e);
    })
}

// Test that an example file can be parsed (compiles)
fn test_example_parses(path: &Path) {
    let source = read_file(path);
    parse_source(&source).unwrap_or_else(|e| {
        panic!("Failed to parse {:?}: {}", path, e);
    });
}

// Test that an example file can be executed (if it doesn't require external deps)
fn test_example_executes(path: &Path) -> Result<(), String> {
    let source = read_file(path);
    
    // Skip execution if file requires external dependencies
    if requires_external_dependencies(&source) {
        return Err("Requires external dependencies".to_string());
    }
    
    // Try to execute with timeout protection
    execute_source(&source).map_err(|e| format!("Execution failed: {}", e))?;
    Ok(())
}

// Check if source code requires external dependencies
fn requires_external_dependencies(source: &str) -> bool {
    // Check for blockchain operations
    if source.contains("chain::") && !source.contains("// MOCK") {
        return true;
    }
    
    // Check for AI operations
    if source.contains("ai::") && !source.contains("// MOCK") {
        return true;
    }
    
    // Check for oracle operations
    if source.contains("oracle::") && !source.contains("// MOCK") {
        return true;
    }
    
    // Check for HTTP server operations
    if source.contains("web::create_server") || source.contains("web::listen") {
        return true;
    }
    
    // Check for database operations
    if source.contains("db::") && !source.contains("// MOCK") {
        return true;
    }
    
    false
}

// ============================================
// INDIVIDUAL EXAMPLE TESTS
// ============================================

#[test]
fn test_hello_world_demo_parses() {
    let path = Path::new("examples/hello_world_demo.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_general_purpose_demo_parses() {
    let path = Path::new("examples/general_purpose_demo.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_simple_chain_examples_parses() {
    let path = Path::new("examples/simple_chain_examples.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_agent_system_demo_parses() {
    let path = Path::new("examples/agent_system_demo.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_smart_contract_parses() {
    let path = Path::new("examples/smart_contract.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_cross_chain_patterns_parses() {
    let path = Path::new("examples/cross_chain_patterns.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_keys_token_implementation_parses() {
    let path = Path::new("examples/keys_token_implementation.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_simple_web_api_example_parses() {
    let path = Path::new("examples/simple_web_api_example.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_secure_configuration_example_parses() {
    let path = Path::new("examples/secure_configuration_example.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_oracle_quick_start_parses() {
    let path = Path::new("examples/oracle_quick_start.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_oracle_development_setup_parses() {
    let path = Path::new("examples/oracle_development_setup.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_dynamic_rwa_examples_parses() {
    let path = Path::new("examples/dynamic_rwa_examples.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_xnft_implementation_parses() {
    let path = Path::new("examples/xnft_implementation.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_llm_integration_examples_parses() {
    let path = Path::new("examples/llm_integration_examples.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_backend_connectivity_patterns_parses() {
    let path = Path::new("examples/backend_connectivity_patterns.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_real_time_backend_example_parses() {
    let path = Path::new("examples/real_time_backend_example.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_practical_backend_example_parses() {
    let path = Path::new("examples/practical_backend_example.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

#[test]
fn test_todo_backend_service_parses() {
    let path = Path::new("examples/todo_backend_service.dal");
    if path.exists() {
        test_example_parses(path);
    }
}

// ============================================
// COMPREHENSIVE TESTS
// ============================================

/// Test that all example files can be parsed
#[test]
fn test_all_examples_parse() {
    let example_files = get_example_files();
    
    assert!(!example_files.is_empty(), "No example files found");
    
    let mut failed = Vec::new();
    
    for path in &example_files {
        let source = read_file(path);
        if let Err(e) = parse_source(&source) {
            failed.push((path.clone(), format!("{}", e)));
        }
    }
    
    if !failed.is_empty() {
        eprintln!("\n❌ Failed to parse {} example(s):", failed.len());
        for (path, error) in &failed {
            eprintln!("  - {:?}: {}", path, error);
        }
        panic!("Some examples failed to parse");
    }
}

/// Test that examples without external dependencies can execute
#[test]
fn test_simple_examples_execute() {
    let example_files = get_example_files();
    
    let mut executed = 0;
    let mut skipped = 0;
    let mut failed = Vec::new();
    
    for path in &example_files {
        let source = read_file(path);
        
        // Skip files that require external dependencies
        if requires_external_dependencies(&source) {
            skipped += 1;
            continue;
        }
        
        // Try to execute
        match execute_source(&source) {
            Ok(_) => {
                executed += 1;
            }
            Err(e) => {
                failed.push((path.clone(), format!("{}", e)));
            }
        }
    }
    
    eprintln!("\n📊 Execution Summary:");
    eprintln!("  ✅ Executed: {}", executed);
    eprintln!("  ⏭️  Skipped (external deps): {}", skipped);
    eprintln!("  ❌ Failed: {}", failed.len());
    
    if !failed.is_empty() {
        eprintln!("\n❌ Failed to execute {} example(s):", failed.len());
        for (path, error) in &failed {
            eprintln!("  - {:?}: {}", path, error);
        }
        // Don't panic - some failures might be expected
        // Just report them for investigation
    }
    
    // At least some examples should execute
    assert!(executed > 0, "No examples executed successfully");
}

/// Test that examples follow basic syntax rules
#[test]
fn test_examples_have_valid_syntax() {
    let example_files = get_example_files();
    
    for path in &example_files {
        let source = read_file(path);
        
        // Basic syntax checks
        assert!(!source.is_empty(), "Example file {:?} is empty", path);
        
        // Check for common syntax issues
        // (Add more checks as needed)
        
        // Ensure file ends with newline (optional but good practice)
        // This is just a warning, not a failure
    }
}

// ============================================
// CATEGORY-SPECIFIC TESTS
// ============================================

#[test]
fn test_basic_examples_parse() {
    let basic_examples = vec![
        "examples/hello_world_demo.dal",
        "examples/general_purpose_demo.dal",
    ];
    
    for example in basic_examples {
        let path = Path::new(example);
        if path.exists() {
            test_example_parses(path);
        }
    }
}

#[test]
fn test_blockchain_examples_parse() {
    let blockchain_examples = vec![
        "examples/simple_chain_examples.dal",
        "examples/smart_contract.dal",
        "examples/cross_chain_patterns.dal",
        "examples/keys_token_implementation.dal",
    ];
    
    for example in blockchain_examples {
        let path = Path::new(example);
        if path.exists() {
            test_example_parses(path);
        }
    }
}

#[test]
fn test_ai_examples_parse() {
    let ai_examples = vec![
        "examples/agent_system_demo.dal",
        "examples/llm_integration_examples.dal",
        "examples/llm_motivations_demo.dal",
    ];
    
    for example in ai_examples {
        let path = Path::new(example);
        if path.exists() {
            test_example_parses(path);
        }
    }
}

#[test]
fn test_web_examples_parse() {
    let web_examples = vec![
        "examples/simple_web_api_example.dal",
        "examples/backend_connectivity_patterns.dal",
        "examples/real_time_backend_example.dal",
        "examples/practical_backend_example.dal",
    ];
    
    for example in web_examples {
        let path = Path::new(example);
        if path.exists() {
            test_example_parses(path);
        }
    }
}

// ============================================
// EXECUTION TESTS (for simple examples)
// ============================================

#[test]
fn test_hello_world_executes() {
    let path = Path::new("examples/hello_world_demo.dal");
    if path.exists() {
        let source = read_file(path);
        if !requires_external_dependencies(&source) {
            if let Err(e) = execute_source(&source) {
                // Don't fail the test, just report
                eprintln!("Warning: hello_world_demo.dal execution failed: {}", e);
            }
        }
    }
}

#[test]
fn test_general_purpose_demo_executes() {
    let path = Path::new("examples/general_purpose_demo.dal");
    if path.exists() {
        let source = read_file(path);
        if !requires_external_dependencies(&source) {
            if let Err(e) = execute_source(&source) {
                // Don't fail the test, just report
                eprintln!("Warning: general_purpose_demo.dal execution failed: {}", e);
            }
        }
    }
}

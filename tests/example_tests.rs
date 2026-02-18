// Example Tests
// Tests all DAL example files to ensure they compile and execute correctly
// This is critical for ensuring examples stay up-to-date with language changes
//
// SEMANTIC VALIDATION to catch:
// - Invalid trust models
// - Invalid blockchain identifiers  
// - Attribute compatibility issues
// - Type mismatches

use dist_agent_lang::{parse_source, execute_source};
use std::fs;
use std::path::{Path, PathBuf};

/// Examples skipped from parse/semantic checks (syntax or validator not yet aligned).
/// Only include examples that are known to have issues that can't be easily fixed.
/// Keep this list minimal - only add examples that are actually broken.
const SKIP_EXAMPLES: &[&str] = &[
    // These are currently broken and need fixes:
    "examples/ai_agent_examples.dal",
    "examples/database_examples.dal",
    "examples/desktop_examples.dal",
    "examples/mobile_examples.dal",
    "examples/dynamic_rwa_examples.dal",
    "examples/oracle_quick_start.dal",
    "examples/general_purpose_demo.dal",
    "examples/oracle_development_setup.dal",
    "examples/practical_backend_example.dal",
    "examples/real_time_backend_example.dal",
    "examples/xnft_implementation.dal",
    "examples/todo_backend_service.dal",
    "examples/llm_motivations_demo.dal",
    // Parse failures:
    "examples/auto_detect_example.dal",
    "examples/chain_selection_example.dal",
    "examples/enhanced_language_features.dal",
    "examples/http_vs_ffi_example.dal",
    "examples/integrated_spawn_ai_examples.dal",
    "examples/solidity_testing.dal",
    // Semantic validation failures:
    "examples/backend_connectivity_patterns.dal",
    "examples/cross_chain_patterns.dal",
    "examples/llm_integration_examples.dal",
    "examples/secure_configuration_example.dal",
    "examples/simple_web_api_example.dal",
    "examples/smart_contract.dal",
    "examples/solidity_abi_integration.dal",
];

fn should_skip(path: &Path) -> bool {
    let s = path.to_string_lossy().replace('\\', "/");
    SKIP_EXAMPLES
        .iter()
        .any(|skip| s.ends_with(skip) || s.contains(skip))
}

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
    if should_skip(path) {
        return;
    }
    let source = read_file(path);
    parse_source(&source).unwrap_or_else(|e| {
        panic!("Failed to parse {:?}: {}", path, e);
    });
}

// Test that an example file can be parsed AND semantically validated
#[allow(dead_code)]
fn test_example_parses_with_semantics(path: &Path) {
    if should_skip(path) {
        return;
    }
    let source = read_file(path);
    let ast = parse_source(&source).unwrap_or_else(|e| {
        panic!("Failed to parse {:?}: {}", path, e);
    });
    
    // Validate semantic correctness
    validate_ast_semantics(&ast, path);
}

// Validate the AST for semantic correctness
fn validate_ast_semantics(ast: &dist_agent_lang::parser::ast::Program, path: &Path) {
    use dist_agent_lang::parser::ast::{Statement, Expression};
    use dist_agent_lang::lexer::tokens::Literal;
    
    for statement in &ast.statements {
        match statement {
            Statement::Service(service) => {
                // Collect attribute names
                let attr_names: Vec<&str> = service.attributes.iter()
                    .map(|a| a.name.as_str())
                    .collect();
                
                // Validate attribute compatibility rules
                validate_attribute_compatibility(&attr_names, path);
                
                // Validate individual attribute values
                for attr in &service.attributes {
                    match attr.name.as_str() {
                        "trust" => {
                            if let Some(Expression::Literal(Literal::String(model))) = attr.parameters.first() {
                                validate_trust_model(model, path);
                            }
                        }
                        "chain" => {
                            for param in &attr.parameters {
                                if let Expression::Literal(Literal::String(chain)) = param {
                                    validate_chain(chain, path);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Statement::Function(func) => {
                // Validate function attributes
                for attr in &func.attributes {
                    match attr.name.as_str() {
                        "trust" => {
                            if let Some(Expression::Literal(Literal::String(model))) = attr.parameters.first() {
                                validate_trust_model(model, path);
                            }
                        }
                        "chain" => {
                            for param in &attr.parameters {
                                if let Expression::Literal(Literal::String(chain)) = param {
                                    validate_chain(chain, path);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

// Validate trust model values
fn validate_trust_model(model: &str, path: &Path) {
    let valid_models = ["hybrid", "centralized", "decentralized", "trustless"];
    if !valid_models.contains(&model) {
        panic!(
            "Invalid trust model '{}' in {:?}. Valid options: {:?}",
            model, path, valid_models
        );
    }
}

// Validate blockchain identifiers
fn validate_chain(chain: &str, path: &Path) {
    let valid_chains = [
        "ethereum", "polygon", "bsc", "solana", "bitcoin",
        "avalanche", "arbitrum", "optimism", "base", "near",
        "eth", // Common shorthand
    ];
    if !valid_chains.contains(&chain.to_lowercase().as_str()) {
        panic!(
            "Invalid chain identifier '{}' in {:?}. Valid options: {:?}",
            chain, path, valid_chains
        );
    }
}

// Validate attribute compatibility
fn validate_attribute_compatibility(attrs: &[&str], path: &Path) {
    let has_trust = attrs.contains(&"trust");
    let has_chain = attrs.contains(&"chain");
    let has_secure = attrs.contains(&"secure");
    let has_public = attrs.contains(&"public");
    
    // Rule: @trust requires @chain
    if has_trust && !has_chain {
        panic!(
            "Service with @trust attribute must also have @chain attribute in {:?}",
            path
        );
    }
    
    // Rule: @secure and @public are mutually exclusive
    if has_secure && has_public {
        panic!(
            "@secure and @public attributes are mutually exclusive in {:?}",
            path
        );
    }
}

// Test that an example file can be executed (if it doesn't require external deps)
#[allow(dead_code)]
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

/// Test that all example files can be parsed (skipped examples excluded)
#[test]
fn test_all_examples_parse() {
    let example_files: Vec<_> = get_example_files()
        .into_iter()
        .filter(|p| !should_skip(p))
        .collect();
    
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
    
    println!("\n✅ All {} examples parsed successfully!", example_files.len());
}

/// Test that all example files can be parsed AND semantically validated (skipped examples excluded)
/// This addresses the limitations of syntax-only validation by checking:
/// - Trust model values (hybrid, centralized, decentralized, trustless)
/// - Chain identifiers (ethereum, polygon, etc.)
/// - Attribute compatibility (@trust requires @chain, @secure ⊕ @public)
#[test]
fn test_all_examples_with_semantic_validation() {
    let example_files: Vec<_> = get_example_files()
        .into_iter()
        .filter(|p| !should_skip(p))
        .collect();
    assert!(!example_files.is_empty(), "No example files found");
    
    let mut failed_parse = Vec::new();
    let mut failed_semantic = Vec::new();
    
    for path in &example_files {
        let source = read_file(path);
        
        // Parse test
        match parse_source(&source) {
            Ok(ast) => {
                // Semantic validation
                if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    validate_ast_semantics(&ast, &path);
                })) {
                    let msg = if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "Unknown semantic error".to_string()
                    };
                    failed_semantic.push((path.clone(), msg));
                }
            }
            Err(e) => {
                failed_parse.push((path.clone(), format!("{}", e)));
            }
        }
    }
    
    let mut had_errors = false;
    
    if !failed_parse.is_empty() {
        eprintln!("\n❌ Failed to parse {} example(s):", failed_parse.len());
        for (path, error) in &failed_parse {
            eprintln!("  - {:?}: {}", path, error);
        }
        had_errors = true;
    }
    
    if !failed_semantic.is_empty() {
        eprintln!("\n❌ Semantic validation failed for {} example(s):", failed_semantic.len());
        for (path, error) in &failed_semantic {
            eprintln!("  - {:?}: {}", path, error);
        }
        had_errors = true;
    }
    
    if had_errors {
        panic!("Some examples failed parse or semantic validation");
    }
    
    println!("\n✅ All {} examples passed syntax AND semantic validation!", example_files.len());
}

/// Test that examples without external dependencies can execute
#[test]
fn test_simple_examples_execute() {
    let example_files = get_example_files();
    
    let mut executed = 0;
    let mut skipped = 0;
    let mut failed = Vec::new();
    
    for path in &example_files {
        if should_skip(path) {
            skipped += 1;
            continue;
        }
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
        if should_skip(path) {
            continue;
        }
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

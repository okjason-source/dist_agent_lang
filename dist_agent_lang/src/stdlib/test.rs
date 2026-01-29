// Test utilities for Hardhat-style testing in DAL
// Provides describe, it, expect, beforeEach, afterEach, etc.

use crate::runtime::values::Value;
use crate::runtime::engine::Runtime;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Test context shared across test functions
#[derive(Debug, Clone)]
pub struct TestContext {
    pub current_suite: Option<String>,
    pub current_test: Option<String>,
    pub services: HashMap<String, String>, // service_name -> instance_id
    pub variables: HashMap<String, Value>,
    pub setup_hooks: Vec<String>, // Code to run before each test
    pub teardown_hooks: Vec<String>, // Code to run after each test
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            current_suite: None,
            current_test: None,
            services: HashMap::new(),
            variables: HashMap::new(),
            setup_hooks: Vec::new(),
            teardown_hooks: Vec::new(),
        }
    }
}

// Global test context (thread-safe)
lazy_static::lazy_static! {
    static ref TEST_CONTEXT: Arc<Mutex<TestContext>> = Arc::new(Mutex::new(TestContext::new()));
    static ref TEST_RESULTS: Arc<Mutex<Vec<TestResult>>> = Arc::new(Mutex::new(Vec::new()));
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub suite_name: String,
    pub test_name: String,
    pub passed: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Test suite structure
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub tests: Vec<TestDefinition>,
    pub before_each: Option<String>,
    pub after_each: Option<String>,
    pub before_all: Option<String>,
    pub after_all: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestDefinition {
    pub name: String,
    pub code: String,
    pub skipped: bool,
}

impl TestSuite {
    pub fn new(name: String) -> Self {
        Self {
            name,
            tests: Vec::new(),
            before_each: None,
            after_each: None,
            before_all: None,
            after_all: None,
        }
    }
}

// Global test suites registry
lazy_static::lazy_static! {
    static ref TEST_SUITES: Arc<Mutex<Vec<TestSuite>>> = Arc::new(Mutex::new(Vec::new()));
    static ref CURRENT_SUITE: Arc<Mutex<Option<TestSuite>>> = Arc::new(Mutex::new(None));
}

/// Register a test suite (called by describe())
pub fn register_suite(name: String) {
    let mut suites = TEST_SUITES.lock().unwrap();
    let suite = TestSuite::new(name);
    suites.push(suite.clone());
    *CURRENT_SUITE.lock().unwrap() = Some(suite);
}

/// Add a test to the current suite (called by it())
pub fn add_test(name: String, code: String) {
    let mut current = CURRENT_SUITE.lock().unwrap();
    if let Some(ref mut suite) = *current {
        suite.tests.push(TestDefinition {
            name,
            code,
            skipped: false,
        });
    }
}

/// Set before_each hook for current suite
pub fn set_before_each(code: String) {
    let mut current = CURRENT_SUITE.lock().unwrap();
    if let Some(ref mut suite) = *current {
        suite.before_each = Some(code);
    }
}

/// Set after_each hook for current suite
pub fn set_after_each(code: String) {
    let mut current = CURRENT_SUITE.lock().unwrap();
    if let Some(ref mut suite) = *current {
        suite.after_each = Some(code);
    }
}

/// Set before_all hook for current suite
pub fn set_before_all(code: String) {
    let mut current = CURRENT_SUITE.lock().unwrap();
    if let Some(ref mut suite) = *current {
        suite.before_all = Some(code);
    }
}

/// Set after_all hook for current suite
pub fn set_after_all(code: String) {
    let mut current = CURRENT_SUITE.lock().unwrap();
    if let Some(ref mut suite) = *current {
        suite.after_all = Some(code);
    }
}

/// Expect assertion functions
pub fn expect_eq(actual: Value, expected: Value) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("Expected {:?}, but got {:?}", expected, actual))
    }
}

pub fn expect_ne(actual: Value, expected: Value) -> Result<(), String> {
    if actual != expected {
        Ok(())
    } else {
        Err(format!("Expected not {:?}, but got {:?}", expected, actual))
    }
}

pub fn expect_true(value: Value) -> Result<(), String> {
    match value {
        Value::Bool(true) => Ok(()),
        _ => Err(format!("Expected true, but got {:?}", value)),
    }
}

pub fn expect_false(value: Value) -> Result<(), String> {
    match value {
        Value::Bool(false) => Ok(()),
        _ => Err(format!("Expected false, but got {:?}", value)),
    }
}

pub fn expect_nil(value: Value) -> Result<(), String> {
    match value {
        Value::Null => Ok(()),
        _ => Err(format!("Expected null, but got {:?}", value)),
    }
}

pub fn expect_not_nil(value: Value) -> Result<(), String> {
    match value {
        Value::Null => Err("Expected not null, but got null".to_string()),
        _ => Ok(()),
    }
}

pub fn expect_throws(code: &str, expected_error: &str) -> Result<(), String> {
    // This would need to execute code and check for errors
    // For now, return a placeholder
    Err("expect_throws not yet implemented".to_string())
}

/// Deploy a service for testing (similar to Hardhat's deployContract)
pub fn deploy_service(service_name: String, constructor_args: Vec<Value>) -> Result<String, String> {
    let mut context = TEST_CONTEXT.lock().unwrap();
    
    // Generate instance ID
    let instance_id = format!("test_{}_{}", service_name, context.services.len());
    
    // Store the service instance
    context.services.insert(service_name.clone(), instance_id.clone());
    
    Ok(instance_id)
}

/// Get a deployed service instance
pub fn get_service(service_name: String) -> Result<String, String> {
    let context = TEST_CONTEXT.lock().unwrap();
    context.services.get(&service_name)
        .cloned()
        .ok_or_else(|| format!("Service {} not deployed", service_name))
}

/// Call a service method (similar to Hardhat's contract.method())
pub fn call_service_method(instance_id: String, method_name: String, args: Vec<Value>) -> Result<Value, String> {
    // This would call the actual service method through the runtime
    // For now, return a placeholder
    Err("call_service_method requires runtime integration".to_string())
}

/// Set a test variable
pub fn set_var(name: String, value: Value) {
    let mut context = TEST_CONTEXT.lock().unwrap();
    context.variables.insert(name, value);
}

/// Get a test variable
pub fn get_var(name: String) -> Result<Value, String> {
    let context = TEST_CONTEXT.lock().unwrap();
    context.variables.get(&name)
        .cloned()
        .ok_or_else(|| format!("Variable {} not found", name))
}

/// Reset test context (called between tests)
pub fn reset_context() {
    let mut context = TEST_CONTEXT.lock().unwrap();
    context.variables.clear();
    // Keep services but reset variables
}

/// Get all test suites
pub fn get_test_suites() -> Vec<TestSuite> {
    TEST_SUITES.lock().unwrap().clone()
}

/// Clear all test suites (for fresh test runs)
pub fn clear_test_suites() {
    let mut suites = TEST_SUITES.lock().unwrap();
    suites.clear();
    *CURRENT_SUITE.lock().unwrap() = None;
}

/// Record a test result
pub fn record_result(result: TestResult) {
    let mut results = TEST_RESULTS.lock().unwrap();
    results.push(result);
}

/// Get all test results
pub fn get_results() -> Vec<TestResult> {
    TEST_RESULTS.lock().unwrap().clone()
}

/// Clear test results
pub fn clear_results() {
    let mut results = TEST_RESULTS.lock().unwrap();
    results.clear();
}

// ============================================================================
// SEMANTIC VALIDATION LAYER
// ============================================================================

/// Validate that a trust model is valid
pub fn expect_valid_trust_model(model: &str) -> Result<(), String> {
    let valid_models = ["hybrid", "centralized", "decentralized", "trustless"];
    if valid_models.contains(&model) {
        Ok(())
    } else {
        Err(format!(
            "Invalid trust model: '{}'. Valid options: {:?}", 
            model, 
            valid_models
        ))
    }
}

/// Validate that a blockchain chain identifier is valid
pub fn expect_valid_chain(chain: &str) -> Result<(), String> {
    let valid_chains = [
        "ethereum", "polygon", "bsc", "solana", "bitcoin", 
        "avalanche", "arbitrum", "optimism", "base", "near"
    ];
    if valid_chains.contains(&chain) {
        Ok(())
    } else {
        Err(format!(
            "Invalid chain: '{}'. Valid options: {:?}", 
            chain, 
            valid_chains
        ))
    }
}

/// Validate that a value matches an expected type
pub fn expect_type(value: &Value, expected_type: &str) -> Result<(), String> {
    let actual_type = match value {
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Bool(_) => "bool",
        Value::Map(_) => "map",
        Value::Vector(_) => "vector",
        Value::Null => "null",
        Value::Function { .. } => "function",
        _ => "unknown",
    };
    
    if actual_type == expected_type {
        Ok(())
    } else {
        Err(format!(
            "Type mismatch: expected '{}', but got '{}'", 
            expected_type, 
            actual_type
        ))
    }
}

/// Validate that a number is within a range
pub fn expect_in_range(value: Value, min: f64, max: f64) -> Result<(), String> {
    match value {
        Value::Number(n) => {
            if n >= min && n <= max {
                Ok(())
            } else {
                Err(format!(
                    "Value {} is out of range [{}, {}]", 
                    n, min, max
                ))
            }
        }
        _ => Err(format!("Expected number, got {:?}", value)),
    }
}

/// Validate that a string matches a pattern (basic contains check)
pub fn expect_contains(haystack: &str, needle: &str) -> Result<(), String> {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(format!(
            "String '{}' does not contain '{}'", 
            haystack, 
            needle
        ))
    }
}

/// Validate that a string starts with a prefix
pub fn expect_starts_with(string: &str, prefix: &str) -> Result<(), String> {
    if string.starts_with(prefix) {
        Ok(())
    } else {
        Err(format!(
            "String '{}' does not start with '{}'", 
            string, 
            prefix
        ))
    }
}

/// Validate that a collection has an expected length
pub fn expect_length(value: Value, expected_len: usize) -> Result<(), String> {
    let actual_len = match &value {
        Value::String(s) => s.len(),
        Value::Vector(v) => v.len(),
        Value::Map(m) => m.len(),
        _ => return Err(format!("Value {:?} does not have a length", value)),
    };
    
    if actual_len == expected_len {
        Ok(())
    } else {
        Err(format!(
            "Length mismatch: expected {}, got {}", 
            expected_len, 
            actual_len
        ))
    }
}

/// Validate that a collection is not empty
pub fn expect_not_empty(value: Value) -> Result<(), String> {
    let is_empty = match &value {
        Value::String(s) => s.is_empty(),
        Value::Vector(v) => v.is_empty(),
        Value::Map(m) => m.is_empty(),
        _ => return Err(format!("Value {:?} is not a collection", value)),
    };
    
    if !is_empty {
        Ok(())
    } else {
        Err("Collection is empty".to_string())
    }
}

/// Validate that a map contains a specific key
pub fn expect_has_key(map: Value, key: &str) -> Result<(), String> {
    match map {
        Value::Map(m) => {
            if m.contains_key(key) {
                Ok(())
            } else {
                Err(format!("Map does not contain key '{}'", key))
            }
        }
        _ => Err(format!("Expected map, got {:?}", map)),
    }
}

/// Validate that an attribute exists (for semantic AST validation)
/// Note: This would require AST access, placeholder for now
pub fn expect_has_attribute(service_name: &str, attr_name: &str) -> Result<(), String> {
    // This would need AST access from the runtime
    // For now, store this as a semantic validation requirement
    Err(format!(
        "Attribute validation not yet implemented. Would check if '{}' has @{}", 
        service_name, 
        attr_name
    ))
}

/// Validate attribute compatibility rules
pub fn expect_compatible_attributes(attributes: Vec<&str>) -> Result<(), String> {
    // Rule: @trust requires @chain
    let has_trust = attributes.contains(&"trust");
    let has_chain = attributes.contains(&"chain");
    
    if has_trust && !has_chain {
        return Err("Services with @trust attribute must also have @chain attribute".to_string());
    }
    
    // Rule: @secure and @public are mutually exclusive
    let has_secure = attributes.contains(&"secure");
    let has_public = attributes.contains(&"public");
    
    if has_secure && has_public {
        return Err("@secure and @public attributes are mutually exclusive".to_string());
    }
    
    Ok(())
}

/// Validate that a function signature matches expectations
pub fn expect_function_signature(
    function_name: &str, 
    param_count: usize, 
    has_return: bool
) -> Result<(), String> {
    // This would need runtime/AST integration
    // Placeholder for semantic validation
    Err(format!(
        "Function signature validation not yet implemented for '{}'", 
        function_name
    ))
}

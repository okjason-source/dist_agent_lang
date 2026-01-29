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

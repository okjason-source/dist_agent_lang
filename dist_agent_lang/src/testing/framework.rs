use std::collections::HashMap;
use std::time::Duration;
use crate::runtime::values::Value;

/// Test result status
#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed(String),
    Skipped(String),
    Error(String),
}

/// Individual test case
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub description: Option<String>,
    pub source_code: String,
    pub expected_result: Option<Value>,
    pub expected_error: Option<String>,
    pub timeout: Option<Duration>,
    pub tags: Vec<String>,
    pub setup_code: Option<String>,
    pub teardown_code: Option<String>,
}

impl TestCase {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            source_code: String::new(),
            expected_result: None,
            expected_error: None,
            timeout: None,
            tags: Vec::new(),
            setup_code: None,
            teardown_code: None,
        }
    }
    
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    pub fn with_source_code(mut self, source_code: &str) -> Self {
        self.source_code = source_code.to_string();
        self
    }
    
    pub fn expect_result(mut self, expected: Value) -> Self {
        self.expected_result = Some(expected);
        self
    }
    
    pub fn expect_error(mut self, error_message: &str) -> Self {
        self.expected_error = Some(error_message.to_string());
        self
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
    
    pub fn with_setup(mut self, setup_code: &str) -> Self {
        self.setup_code = Some(setup_code.to_string());
        self
    }
    
    pub fn with_teardown(mut self, teardown_code: &str) -> Self {
        self.teardown_code = Some(teardown_code.to_string());
        self
    }
}

/// Test result with timing and metadata
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_case: TestCase,
    pub status: TestStatus,
    pub duration: Duration,
    pub actual_result: Option<Value>,
    pub error_message: Option<String>,
    pub coverage: Option<TestCoverage>,
    pub metadata: HashMap<String, Value>,
}

impl TestResult {
    pub fn new(test_case: TestCase, status: TestStatus, duration: Duration) -> Self {
        Self {
            test_case,
            status,
            duration,
            actual_result: None,
            error_message: None,
            coverage: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_result(mut self, result: Value) -> Self {
        self.actual_result = Some(result);
        self
    }
    
    pub fn with_error(mut self, error: &str) -> Self {
        self.error_message = Some(error.to_string());
        self
    }
    
    pub fn with_coverage(mut self, coverage: TestCoverage) -> Self {
        self.coverage = Some(coverage);
        self
    }
    
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
    
    pub fn is_passed(&self) -> bool {
        matches!(self.status, TestStatus::Passed)
    }
    
    pub fn is_failed(&self) -> bool {
        matches!(self.status, TestStatus::Failed(_))
    }
}

/// Test coverage information
#[derive(Debug, Clone)]
pub struct TestCoverage {
    pub lines_covered: usize,
    pub total_lines: usize,
    pub functions_called: Vec<String>,
    pub branches_covered: usize,
    pub total_branches: usize,
}

impl TestCoverage {
    pub fn new() -> Self {
        Self {
            lines_covered: 0,
            total_lines: 0,
            functions_called: Vec::new(),
            branches_covered: 0,
            total_branches: 0,
        }
    }
    
    pub fn coverage_percentage(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.lines_covered as f64 / self.total_lines as f64) * 100.0
        }
    }
    
    pub fn branch_coverage_percentage(&self) -> f64 {
        if self.total_branches == 0 {
            0.0
        } else {
            (self.branches_covered as f64 / self.total_branches as f64) * 100.0
        }
    }
}

/// Test suite containing multiple test cases
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub description: Option<String>,
    pub test_cases: Vec<TestCase>,
    pub setup_code: Option<String>,
    pub teardown_code: Option<String>,
    pub tags: Vec<String>,
}

impl TestSuite {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            test_cases: Vec::new(),
            setup_code: None,
            teardown_code: None,
            tags: Vec::new(),
        }
    }
    
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    pub fn add_test(mut self, test_case: TestCase) -> Self {
        self.test_cases.push(test_case);
        self
    }
    
    pub fn with_setup(mut self, setup_code: &str) -> Self {
        self.setup_code = Some(setup_code.to_string());
        self
    }
    
    pub fn with_teardown(mut self, teardown_code: &str) -> Self {
        self.teardown_code = Some(teardown_code.to_string());
        self
    }
    
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
    
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&TestCase> {
        self.test_cases.iter()
            .filter(|test| test.tags.contains(&tag.to_string()))
            .collect()
    }
}

/// Assertion functions for testing
pub trait Assertions {
    fn assert_eq(&self, actual: &Value, expected: &Value) -> Result<(), String>;
    fn assert_ne(&self, actual: &Value, expected: &Value) -> Result<(), String>;
    fn assert_true(&self, value: &Value) -> Result<(), String>;
    fn assert_false(&self, value: &Value) -> Result<(), String>;
    fn assert_nil(&self, value: &Value) -> Result<(), String>;
    fn assert_not_nil(&self, value: &Value) -> Result<(), String>;
    fn assert_contains(&self, container: &Value, item: &Value) -> Result<(), String>;
    fn assert_greater(&self, actual: &Value, expected: &Value) -> Result<(), String>;
    fn assert_less(&self, actual: &Value, expected: &Value) -> Result<(), String>;
    fn assert_throws(&self, code: &str, expected_error: &str) -> Result<(), String>;
}

impl Assertions for TestCase {
    fn assert_eq(&self, actual: &Value, expected: &Value) -> Result<(), String> {
        if actual == expected {
            Ok(())
        } else {
            Err(format!("Expected {:?}, but got {:?}", expected, actual))
        }
    }
    
    fn assert_ne(&self, actual: &Value, expected: &Value) -> Result<(), String> {
        if actual != expected {
            Ok(())
        } else {
            Err(format!("Expected not {:?}, but got {:?}", expected, actual))
        }
    }
    
    fn assert_true(&self, value: &Value) -> Result<(), String> {
        match value {
            Value::Bool(true) => Ok(()),
            _ => Err(format!("Expected true, but got {:?}", value)),
        }
    }
    
    fn assert_false(&self, value: &Value) -> Result<(), String> {
        match value {
            Value::Bool(false) => Ok(()),
            _ => Err(format!("Expected false, but got {:?}", value)),
        }
    }
    
    fn assert_nil(&self, value: &Value) -> Result<(), String> {
        match value {
            Value::Null => Ok(()),
            _ => Err(format!("Expected null, but got {:?}", value)),
        }
    }
    
    fn assert_not_nil(&self, value: &Value) -> Result<(), String> {
        match value {
            Value::Null => Err("Expected not null, but got null".to_string()),
            _ => Ok(()),
        }
    }
    
    fn assert_contains(&self, _container: &Value, _item: &Value) -> Result<(), String> {
        // This is a simplified implementation
        // In a real implementation, you'd check if item is in container
        Err("assert_contains not implemented".to_string())
    }
    
    fn assert_greater(&self, actual: &Value, expected: &Value) -> Result<(), String> {
        match (actual, expected) {
            (Value::Int(a), Value::Int(b)) if a > b => Ok(()),
            _ => Err(format!("Expected {:?} > {:?}", actual, expected)),
        }
    }
    
    fn assert_less(&self, actual: &Value, expected: &Value) -> Result<(), String> {
        match (actual, expected) {
            (Value::Int(a), Value::Int(b)) if a < b => Ok(()),
            _ => Err(format!("Expected {:?} < {:?}", actual, expected)),
        }
    }
    
    fn assert_throws(&self, _code: &str, _expected_error: &str) -> Result<(), String> {
        // This would require parsing and executing the code
        // For now, return a placeholder
        Err("assert_throws not implemented".to_string())
    }
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub verbose: bool,
    pub stop_on_failure: bool,
    pub parallel: bool,
    pub timeout: Option<Duration>,
    pub filter_tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub coverage_enabled: bool,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Text,
    Json,
    Xml,
    Html,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            stop_on_failure: false,
            parallel: false,
            timeout: Some(Duration::from_secs(30)),
            filter_tags: Vec::new(),
            exclude_tags: Vec::new(),
            coverage_enabled: false,
            output_format: OutputFormat::Text,
        }
    }
}

/// Test statistics
#[derive(Debug, Clone)]
pub struct TestStats {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub errors: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub coverage_percentage: f64,
}

impl TestStats {
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            errors: 0,
            total_duration: Duration::ZERO,
            average_duration: Duration::ZERO,
            coverage_percentage: 0.0,
        }
    }
    
    pub fn update_from_results(&mut self, results: &[TestResult]) {
        self.total_tests = results.len();
        self.passed = results.iter().filter(|r| r.is_passed()).count();
        self.failed = results.iter().filter(|r| r.is_failed()).count();
        self.skipped = results.iter().filter(|r| matches!(r.status, TestStatus::Skipped(_))).count();
        self.errors = results.iter().filter(|r| matches!(r.status, TestStatus::Error(_))).count();
        
        self.total_duration = results.iter()
            .map(|r| r.duration)
            .sum();
        
        if self.total_tests > 0 {
            self.average_duration = self.total_duration / self.total_tests as u32;
        }
        
        // Calculate coverage
        let coverage_results: Vec<&TestCoverage> = results.iter()
            .filter_map(|r| r.coverage.as_ref())
            .collect();
        
        if !coverage_results.is_empty() {
            let total_coverage: f64 = coverage_results.iter()
                .map(|c| c.coverage_percentage())
                .sum();
            self.coverage_percentage = total_coverage / coverage_results.len() as f64;
        }
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total_tests as f64) * 100.0
        }
    }
}

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::runtime::Runtime;
use crate::testing::framework::*;
use crate::testing::mock::*;
use std::time::Instant;

/// Test runner that executes test suites
pub struct TestRunner {
    pub config: TestConfig,
    pub mock_registry: MockRegistry,
    pub results: Vec<TestResult>,
    pub stats: TestStats,
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
            mock_registry: MockRegistry::new(),
            results: Vec::new(),
            stats: TestStats::new(),
        }
    }

    pub fn with_config(mut self, config: TestConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_mock(mut self, mock: MockFunction) -> Self {
        self.mock_registry.register(mock);
        self
    }

    pub fn run_suite(&mut self, suite: TestSuite) -> TestStats {
        println!("Running test suite: {}", suite.name);
        if let Some(desc) = &suite.description {
            println!("Description: {}", desc);
        }
        println!("Total tests: {}", suite.test_cases.len());
        println!("{}", "=".repeat(50));

        let start_time = Instant::now();

        // Run setup if provided
        if let Some(setup_code) = &suite.setup_code {
            if let Err(e) = self.execute_code(setup_code) {
                println!("❌ Setup failed: {}", e);
                return self.stats.clone();
            }
        }

        // Filter tests based on tags
        let filtered_tests = self.filter_tests(&suite.test_cases);

        // Run tests
        for test_case in filtered_tests {
            let result = self.run_test(test_case);
            self.results.push(result);

            // Stop on failure if configured
            if self.config.stop_on_failure && self.results.last().unwrap().is_failed() {
                break;
            }
        }

        // Run teardown if provided
        if let Some(teardown_code) = &suite.teardown_code {
            if let Err(e) = self.execute_code(teardown_code) {
                println!("⚠️  Teardown failed: {}", e);
            }
        }

        // Update statistics
        self.stats.update_from_results(&self.results);
        self.stats.total_duration = start_time.elapsed();

        // Verify mocks
        if let Err(e) = self.mock_registry.verify_all() {
            println!("❌ Mock verification failed: {}", e);
        }

        self.print_summary();
        self.stats.clone()
    }

    pub fn run_test(&mut self, test_case: TestCase) -> TestResult {
        let start_time = Instant::now();
        let test_case_clone = test_case.clone();

        println!("Running test: {}", test_case_clone.name);
        if let Some(desc) = &test_case_clone.description {
            println!("  Description: {}", desc);
        }

        // Run setup if provided
        if let Some(setup_code) = &test_case_clone.setup_code {
            if let Err(e) = self.execute_code(setup_code) {
                let duration = start_time.elapsed();
                return TestResult::new(test_case, TestStatus::Error(e), duration);
            }
        }

        // Execute the test
        let result = match self.execute_test(&test_case_clone) {
            Ok(actual_result) => {
                // Verify expected result
                if let Some(expected) = &test_case_clone.expected_result {
                    if &actual_result == expected {
                        TestResult::new(test_case, TestStatus::Passed, start_time.elapsed())
                            .with_result(actual_result)
                    } else {
                        let error = format!("Expected {:?}, but got {:?}", expected, actual_result);
                        TestResult::new(test_case, TestStatus::Failed(error), start_time.elapsed())
                            .with_result(actual_result)
                    }
                } else {
                    TestResult::new(test_case, TestStatus::Passed, start_time.elapsed())
                        .with_result(actual_result)
                }
            }
            Err(error) => {
                // Check if error was expected
                if let Some(expected_error) = &test_case_clone.expected_error {
                    if error.contains(expected_error) {
                        TestResult::new(test_case, TestStatus::Passed, start_time.elapsed())
                            .with_error(&error)
                    } else {
                        TestResult::new(test_case, TestStatus::Failed(error), start_time.elapsed())
                    }
                } else {
                    TestResult::new(test_case, TestStatus::Error(error), start_time.elapsed())
                }
            }
        };

        // Run teardown if provided
        if let Some(teardown_code) = &test_case_clone.teardown_code {
            if let Err(e) = self.execute_code(teardown_code) {
                println!("⚠️  Test teardown failed: {}", e);
            }
        }

        // Print result
        match &result.status {
            TestStatus::Passed => println!("  ✅ PASSED"),
            TestStatus::Failed(reason) => println!("  ❌ FAILED: {}", reason),
            TestStatus::Skipped(reason) => println!("  ⏭️  SKIPPED: {}", reason),
            TestStatus::Error(reason) => println!("  💥 ERROR: {}", reason),
        }

        result
    }

    fn execute_test(
        &mut self,
        test_case: &TestCase,
    ) -> Result<crate::runtime::values::Value, String> {
        // Check timeout
        if let Some(_timeout) = test_case.timeout {
            // In a real implementation, you'd run this in a separate thread with timeout
            return self.execute_code(&test_case.source_code);
        }

        self.execute_code(&test_case.source_code)
    }

    fn execute_code(&mut self, source_code: &str) -> Result<crate::runtime::values::Value, String> {
        // Lexical analysis
        let mut lexer = Lexer::new(source_code);
        let tokens = lexer
            .tokenize()
            .map_err(|e| format!("Lexer error: {}", e))?;

        // Parsing
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| format!("Parser error: {}", e))?;

        // Runtime execution
        let mut runtime = Runtime::new();

        // Execute the AST
        match runtime.execute(&ast) {
            Ok(result) => Ok(result),
            Err(e) => Err(format!("Runtime error: {}", e)),
        }
    }

    fn filter_tests(&self, test_cases: &[TestCase]) -> Vec<TestCase> {
        test_cases
            .iter()
            .filter(|test| {
                // Apply tag filters
                if !self.config.filter_tags.is_empty() {
                    let has_required_tag = self
                        .config
                        .filter_tags
                        .iter()
                        .any(|tag| test.tags.contains(tag));
                    if !has_required_tag {
                        return false;
                    }
                }

                // Apply exclude filters
                if !self.config.exclude_tags.is_empty() {
                    let has_excluded_tag = self
                        .config
                        .exclude_tags
                        .iter()
                        .any(|tag| test.tags.contains(tag));
                    if has_excluded_tag {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    fn print_summary(&self) {
        println!("\n{}", "=".repeat(50));
        println!("Test Summary");
        println!("{}", "=".repeat(50));
        println!("Total tests: {}", self.stats.total_tests);
        println!("Passed: {}", self.stats.passed);
        println!("Failed: {}", self.stats.failed);
        println!("Skipped: {}", self.stats.skipped);
        println!("Errors: {}", self.stats.errors);
        println!("Success rate: {:.1}%", self.stats.success_rate());
        println!("Total duration: {:?}", self.stats.total_duration);
        println!("Average duration: {:?}", self.stats.average_duration);

        if self.config.coverage_enabled {
            println!("Coverage: {:.1}%", self.stats.coverage_percentage);
        }

        // Print failed tests
        if self.stats.failed > 0 {
            println!("\nFailed Tests:");
            for result in &self.results {
                if result.is_failed() {
                    println!("  - {}: {:?}", result.test_case.name, result.status);
                }
            }
        }

        // Print error tests
        if self.stats.errors > 0 {
            println!("\nError Tests:");
            for result in &self.results {
                if let TestStatus::Error(_) = result.status {
                    println!("  - {}: {:?}", result.test_case.name, result.status);
                }
            }
        }
    }

    pub fn generate_report(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Text => self.generate_text_report(),
            OutputFormat::Json => self.generate_json_report(),
            OutputFormat::Xml => self.generate_xml_report(),
            OutputFormat::Html => self.generate_html_report(),
        }
    }

    fn generate_text_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Test Report\n");
        report.push_str("===========\n\n");

        report.push_str("Summary:\n");
        report.push_str(&format!("  Total: {}\n", self.stats.total_tests));
        report.push_str(&format!("  Passed: {}\n", self.stats.passed));
        report.push_str(&format!("  Failed: {}\n", self.stats.failed));
        report.push_str(&format!("  Skipped: {}\n", self.stats.skipped));
        report.push_str(&format!("  Errors: {}\n", self.stats.errors));
        report.push_str(&format!(
            "  Success Rate: {:.1}%\n",
            self.stats.success_rate()
        ));
        report.push_str(&format!("  Duration: {:?}\n", self.stats.total_duration));

        report.push_str("\nDetailed Results:\n");
        for result in &self.results {
            report.push_str(&format!(
                "  {}: {:?}\n",
                result.test_case.name, result.status
            ));
        }

        report
    }

    fn generate_json_report(&self) -> String {
        // Simplified JSON report
        format!(
            r#"{{"summary":{{"total":{},"passed":{},"failed":{},"skipped":{},"errors":{},"success_rate":{:.1},"duration":"{:?}"}},"results":[]}}"#,
            self.stats.total_tests,
            self.stats.passed,
            self.stats.failed,
            self.stats.skipped,
            self.stats.errors,
            self.stats.success_rate(),
            self.stats.total_duration
        )
    }

    fn generate_xml_report(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="dist_agent_lang" tests="{}" failures="{}" errors="{}" skipped="{}" time="{:.3}">
    <properties/>
  </testsuite>
</testsuites>"#,
            self.stats.total_tests,
            self.stats.failed,
            self.stats.errors,
            self.stats.skipped,
            self.stats.total_duration.as_secs_f64()
        )
    }

    fn generate_html_report(&self) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f5f5f5; padding: 15px; border-radius: 5px; }}
        .passed {{ color: green; }}
        .failed {{ color: red; }}
        .error {{ color: orange; }}
    </style>
</head>
<body>
    <h1>Test Report</h1>
    <div class="summary">
        <h2>Summary</h2>
        <p>Total: {}</p>
        <p class="passed">Passed: {}</p>
        <p class="failed">Failed: {}</p>
        <p class="error">Errors: {}</p>
        <p>Skipped: {}</p>
        <p>Success Rate: {:.1}%</p>
        <p>Duration: {:?}</p>
    </div>
</body>
</html>"#,
            self.stats.total_tests,
            self.stats.passed,
            self.stats.failed,
            self.stats.errors,
            self.stats.skipped,
            self.stats.success_rate(),
            self.stats.total_duration
        )
    }
}

/// Convenience functions for common test scenarios
pub mod test_helpers {
    use super::*;

    pub fn create_basic_test(name: &str, source_code: &str) -> TestCase {
        TestCase::new(name).with_source_code(source_code)
    }

    pub fn create_arithmetic_test() -> TestCase {
        TestCase::new("arithmetic_test")
            .with_description("Test basic arithmetic operations")
            .with_source_code("let x = 10 + 5; x")
            .expect_result(crate::runtime::values::Value::Int(15))
    }

    pub fn create_function_test() -> TestCase {
        TestCase::new("function_test")
            .with_description("Test function definition and call")
            .with_source_code(
                "
                fn add(a, b) {
                    return a + b;
                }
                add(3, 4)
            ",
            )
            .expect_result(crate::runtime::values::Value::Int(7))
    }

    pub fn create_error_test() -> TestCase {
        TestCase::new("error_test")
            .with_description("Test error handling")
            .with_source_code("let x = undefined_variable;")
            .expect_error("undefined variable")
    }

    pub fn create_chain_test() -> TestCase {
        TestCase::new("chain_test")
            .with_description("Test chain namespace functions")
            .with_source_code(
                "
                let asset = chain::mint(\"TestNFT\", {\"description\": \"Test asset\"});
                asset
            ",
            )
            .expect_result(crate::runtime::values::Value::Int(12345)) // Mock value
    }

    pub fn create_oracle_test() -> TestCase {
        TestCase::new("oracle_test")
            .with_description("Test oracle namespace functions")
            .with_source_code(
                "
                let price = oracle::fetch(\"price_feed\", oracle::create_query(\"btc_price\"));
                price
            ",
            )
            .expect_result(crate::runtime::values::Value::String(
                "mock_price_data".to_string(),
            ))
    }

    pub fn create_comprehensive_suite() -> TestSuite {
        TestSuite::new("comprehensive_tests")
            .with_description("Comprehensive test suite for dist_agent_lang")
            .add_test(create_arithmetic_test())
            .add_test(create_function_test())
            .add_test(create_error_test())
            .add_test(create_chain_test())
            .add_test(create_oracle_test())
            .with_tag("comprehensive")
            .with_tag("basic")
    }
}

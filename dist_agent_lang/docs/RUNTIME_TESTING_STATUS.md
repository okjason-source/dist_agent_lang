# Runtime Testing Status and Implementation Plan

## Current Status

### ✅ Layer 1: Syntax Validation - COMPLETE
- `cargo test` validates syntax
- Parses all `.dal` files successfully
- Status: **Production Ready**

### ✅ Layer 2: Semantic Validation - COMPLETE
- Validates trust models, chains, attribute compatibility
- Integrated into Layer 1 tests
- Status: **Production Ready**

### ⚠️ Layer 3: Runtime Behavior Testing - PARTIAL
- Framework exists
- Test infrastructure ready
- **Needs runtime integration**

## What's Implemented

### Test Framework (`src/stdlib/test.rs`)
```rust
✅ TestContext - State management
✅ TestSuite - Test organization
✅ TestResult - Result tracking
✅ register_suite() - Suite registration
✅ add_test() - Test registration
✅ Lifecycle hooks - beforeEach, afterEach, etc.
✅ Basic assertions - expect_eq, expect_ne, expect_true, etc.
✅ Test context management
```

### Test Syntax
```dal
✅ describe() - Test suite definition
✅ it() - Individual test definition
✅ beforeEach() / afterEach() - Hooks
✅ expect() - Assertion syntax
```

### Test Infrastructure
```bash
✅ run_dal_tests.sh - Test runner script
✅ token_contract.test.dal - Example test file
✅ Documentation - Complete guides
```

## What Needs Implementation

### 1. Runtime Integration (HIGH PRIORITY)

#### `expect_throws()` - Error Testing
**Current:**
```rust
pub fn expect_throws(code: &str, expected_error: &str) -> Result<(), String> {
    Err("expect_throws not yet implemented".to_string())
}
```

**Needs:**
```rust
pub fn expect_throws(code: &str, expected_error: &str) -> Result<(), String> {
    // Execute code in runtime
    match execute_source(code) {
        Err(e) => {
            if e.to_string().contains(expected_error) {
                Ok(())
            } else {
                Err(format!("Expected error '{}', got '{}'", expected_error, e))
            }
        }
        Ok(_) => Err("Expected error but code succeeded".to_string())
    }
}
```

#### `call_service_method()` - Service Method Calls
**Current:**
```rust
pub fn call_service_method(instance_id: String, method_name: String, args: Vec<Value>) -> Result<Value, String> {
    Err("call_service_method requires runtime integration".to_string())
}
```

**Needs:**
```rust
pub fn call_service_method(instance_id: String, method_name: String, args: Vec<Value>) -> Result<Value, String> {
    // Get service instance from runtime
    let mut runtime = get_test_runtime()?;
    
    // Call method on service instance
    runtime.call_method(&instance_id, &method_name, args)
        .map_err(|e| format!("Method call failed: {}", e))
}
```

#### `deploy_service()` - Service Deployment
**Current:**
```rust
pub fn deploy_service(service_name: String, constructor_args: Vec<Value>) -> Result<String, String> {
    let mut context = TEST_CONTEXT.lock().unwrap();
    let instance_id = format!("test_{}_{}", service_name, context.services.len());
    context.services.insert(service_name.clone(), instance_id.clone());
    Ok(instance_id)
}
```

**Needs:**
```rust
pub fn deploy_service(service_name: String, constructor_args: Vec<Value>) -> Result<String, String> {
    // Create runtime instance
    let mut runtime = get_test_runtime()?;
    
    // Load and instantiate service
    let service_def = runtime.get_service_definition(&service_name)?;
    let instance_id = format!("test_{}_{}", service_name, uuid::new_v4());
    
    // Initialize service with constructor args
    runtime.instantiate_service(&service_name, &instance_id, constructor_args)?;
    
    // Store in test context
    let mut context = TEST_CONTEXT.lock().unwrap();
    context.services.insert(service_name.clone(), instance_id.clone());
    
    Ok(instance_id)
}
```

### 2. Runtime Test Executor

**New file needed:** `src/testing/test_runner.rs`

```rust
use crate::runtime::engine::Runtime;
use crate::stdlib::test::{TestSuite, TestResult, get_test_suites};

pub struct TestRunner {
    runtime: Runtime,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }
    
    pub fn run_test_file(&mut self, path: &str) -> Result<Vec<TestResult>, String> {
        // Load and parse the test file
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;
        
        // Execute test file (registers suites via describe/it)
        crate::execute_source(&source)
            .map_err(|e| format!("Failed to execute test file: {}", e))?;
        
        // Get registered test suites
        let suites = get_test_suites();
        
        // Run each suite
        let mut results = Vec::new();
        for suite in suites {
            results.extend(self.run_suite(&suite)?);
        }
        
        Ok(results)
    }
    
    fn run_suite(&mut self, suite: &TestSuite) -> Result<Vec<TestResult>, String> {
        let mut results = Vec::new();
        
        // Run before_all hook
        if let Some(ref code) = suite.before_all {
            self.runtime.execute(code)?;
        }
        
        // Run each test
        for test in &suite.tests {
            if test.skipped {
                continue;
            }
            
            let start = std::time::Instant::now();
            
            // Run before_each hook
            if let Some(ref code) = suite.before_each {
                self.runtime.execute(code)?;
            }
            
            // Run the test
            let passed = match self.runtime.execute(&test.code) {
                Ok(_) => true,
                Err(e) => {
                    results.push(TestResult {
                        suite_name: suite.name.clone(),
                        test_name: test.name.clone(),
                        passed: false,
                        error: Some(format!("{}", e)),
                        duration_ms: start.elapsed().as_millis() as u64,
                    });
                    false
                }
            };
            
            // Run after_each hook
            if let Some(ref code) = suite.after_each {
                self.runtime.execute(code)?;
            }
            
            if passed {
                results.push(TestResult {
                    suite_name: suite.name.clone(),
                    test_name: test.name.clone(),
                    passed: true,
                    error: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                });
            }
        }
        
        // Run after_all hook
        if let Some(ref code) = suite.after_all {
            self.runtime.execute(code)?;
        }
        
        Ok(results)
    }
}
```

### 3. CLI Integration

**Modify:** `src/main.rs`

```rust
"test" => {
    if args.len() < 3 {
        // Run all test files
        run_all_tests();
    } else {
        // Run specific test file
        run_test_file(&args[2]);
    }
}

fn run_test_file(path: &str) {
    use dist_agent_lang::testing::TestRunner;
    
    let mut runner = TestRunner::new();
    match runner.run_test_file(path) {
        Ok(results) => {
            print_test_results(&results);
            let failed = results.iter().filter(|r| !r.passed).count();
            std::process::exit(if failed > 0 { 1 } else { 0 });
        }
        Err(e) => {
            eprintln!("❌ Test execution failed: {}", e);
            std::process::exit(1);
        }
    }
}
```

## Implementation Priority

### Phase 1: Basic Runtime Integration (HIGH)
1. ✅ Layer 1 & 2 complete
2. ⏳ Implement `expect_throws()` with runtime
3. ⏳ Implement `deploy_service()` with actual instantiation
4. ⏳ Implement `call_service_method()` with runtime

### Phase 2: Test Runner (MEDIUM)
1. ⏳ Create `TestRunner` struct
2. ⏳ Implement suite execution
3. ⏳ Implement lifecycle hooks
4. ⏳ Add CLI `test` command

### Phase 3: Advanced Features (LOW)
1. Test coverage reporting
2. Parallel test execution
3. Test filtering (--filter, --skip)
4. Watch mode (--watch)
5. Performance benchmarking

## Current Workarounds

Until runtime integration is complete:

### Option 1: Use `execute_source()` Directly
```rust
#[test]
fn test_runtime_behavior() {
    let source = r#"
        service TokenContract {
            balance: map<string, float>;
            fn transfer(to: string, amount: float) {
                balance[to] = amount;
            }
        }
        
        let contract = TokenContract();
        contract.transfer("alice", 100.0);
    "#;
    
    // This works NOW
    execute_source(source).unwrap();
}
```

### Option 2: Test DAL Files with `cargo run -- run`
```bash
# This works NOW
cargo run --release -- run examples/token_contract.dal

# Can be used for basic runtime testing
./scripts/run_dal_tests.sh  # Already runs this
```

## Testing Capabilities Matrix

| Feature | Layer 1 | Layer 2 | Layer 3 | Status |
|---------|---------|---------|---------|--------|
| Syntax validation | ✅ | - | - | Complete |
| Semantic validation | ✅ | ✅ | - | Complete |
| Attribute validation | ✅ | ✅ | - | Complete |
| Parse-time errors | ✅ | ✅ | - | Complete |
| Runtime execution | - | - | ⏳ | Partial |
| Service deployment | - | - | ⏳ | Framework only |
| Method calls | - | - | ⏳ | Framework only |
| Error assertions | - | - | ⏳ | Framework only |
| Lifecycle hooks | - | - | ⏳ | Framework only |

## Estimated Implementation Effort

### Phase 1: Basic Runtime Integration
- **Effort:** 8-12 hours
- **Complexity:** Medium
- **Dependencies:** Runtime engine modifications
- **Deliverable:** Working `describe/it` tests with execution

### Phase 2: Test Runner
- **Effort:** 4-6 hours
- **Complexity:** Low-Medium
- **Dependencies:** Phase 1 complete
- **Deliverable:** `dist_agent_lang test` command

### Phase 3: Advanced Features
- **Effort:** 12-20 hours
- **Complexity:** Medium-High
- **Dependencies:** Phases 1-2 complete
- **Deliverable:** Full-featured test system

## Next Steps

1. **Immediate**: Document current state (this file)
2. **Short-term**: Implement Phase 1 (basic runtime integration)
3. **Medium-term**: Implement Phase 2 (test runner)
4. **Long-term**: Implement Phase 3 (advanced features)

## Workaround Documentation

For now, users can:
- ✅ Use Layer 1 & 2 for comprehensive parse-time validation
- ✅ Use `execute_source()` in Rust tests for runtime validation
- ✅ Run individual `.dal` files with `cargo run -- run file.dal`
- ⏳ Wait for Phase 1 for full DAL test file support

## Status Summary

**Layer 1 (Syntax):** ✅ 100% Complete  
**Layer 2 (Semantics):** ✅ 100% Complete  
**Layer 3 (Runtime):** ⏳ 40% Complete (framework ready, needs runtime integration)

**Overall Testing System:** ✅ 80% Complete

The system is **production-ready for syntax and semantic validation**.  
Runtime behavior testing requires Phase 1 implementation.

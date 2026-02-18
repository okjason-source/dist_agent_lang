# Testing Framework Proposal for DAL Examples

## Current State

### How DAL Files Are Currently Processed

1. **Lexer** (`Lexer::new(source).tokenize()`) - Tokenizes the source code
2. **Parser** (`Parser::new(tokens).parse()`) - Generates an Abstract Syntax Tree (AST)
3. **Runtime** (`Runtime::new().execute_program(ast)`) - Executes the program

### Current Testing Approach

- **Rust Unit Tests**: Test the language implementation itself (lexer, parser, runtime)
- **Manual Example Running**: `dist_agent_lang run examples/hello_world_demo.dal`
- **No Dedicated Test Framework**: No way to write tests in DAL itself

## What's Needed to Test Examples

### Basic Compilation Testing

To verify examples compile correctly, we need:

1. **Syntax Validation**: Ensure the DAL file can be tokenized and parsed
2. **Type Checking**: Verify types are correct (if type checking is implemented)
3. **Semantic Analysis**: Check that references are valid, services are properly defined, etc.
4. **Execution Testing**: Run the code and verify it doesn't crash

### Current Capabilities

The library already provides:
- `parse_source(source: &str)` - Parses DAL code and returns AST or error
- `execute_source(source: &str)` - Executes DAL code and returns result or error

### What's Missing

1. **Test File Format**: No standard way to write tests in DAL
2. **Assertion Library**: No built-in assertions (`assert_eq`, `assert_true`, etc.)
3. **Test Runner**: No framework to discover and run test files
4. **Mocking/Stubbing**: No way to mock external dependencies (blockchain, AI, HTTP)
5. **Test Fixtures**: No way to set up test data
6. **Test Reports**: No structured output of test results

## Proposed: Hardhat-Like Testing Framework

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    DAL Test Framework                    │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Test Runner  │  │ Test Parser  │  │ Test Reporter │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│         │                  │                  │          │
│         └──────────────────┼──────────────────┘          │
│                            │                             │
│  ┌──────────────────────────────────────────────────┐   │
│  │         DAL Runtime (with Test Extensions)       │   │
│  │  - Assertion functions                           │   │
│  │  - Mock registry                                 │   │
│  │  - Test fixtures                                 │   │
│  │  - Snapshot testing                              │   │
│  └──────────────────────────────────────────────────┘   │
│                            │                             │
│  ┌──────────────────────────────────────────────────┐   │
│  │         Standard Library Extensions               │   │
│  │  - test::assert_eq()                             │   │
│  │  - test::mock()                                  │   │
│  │  - test::fixture()                               │   │
│  │  - test::before_each()                           │   │
│  │  - test::after_each()                            │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 1. Test File Structure

Tests would be written in DAL with a special test syntax:

```dal
// test_example.dal
@test
fn test_basic_math() {
    let result = add(2, 3);
    test::assert_eq(result, 5, "2 + 3 should equal 5");
}

@test
fn test_string_operations() {
    let text = "Hello, World!";
    test::assert_true(text.contains("World"), "Should contain 'World'");
    test::assert_eq(text.length(), 13, "Length should be 13");
}

@test
fn test_service_initialization() {
    let service = MyService::new();
    service.initialize();
    test::assert_eq(service.get_count(), 0, "Initial count should be 0");
}

// Test with setup/teardown
@test
@before_each(setup_test_data)
@after_each(cleanup_test_data)
fn test_with_fixtures() {
    let data = load_test_data();
    test::assert_not_null(data, "Test data should be loaded");
}
```

### 2. Test Runner CLI

```bash
# Run all tests in a directory
dal test

# Run specific test file
dal test tests/my_test.dal

# Run tests matching a pattern
dal test --match "test_math"

# Run with verbose output
dal test --verbose

# Run tests in parallel
dal test --parallel

# Generate coverage report
dal test --coverage
```

### 3. Assertion Library

```dal
// Built-in test module
module test {
    // Basic assertions
    fn assert_true(condition: bool, message: string);
    fn assert_false(condition: bool, message: string);
    fn assert_eq<T>(actual: T, expected: T, message: string);
    fn assert_ne<T>(actual: T, expected: T, message: string);
    fn assert_null(value: any, message: string);
    fn assert_not_null(value: any, message: string);
    
    // Type assertions
    fn assert_type<T>(value: any, message: string);
    
    // Collection assertions
    fn assert_empty<T>(collection: List<T>, message: string);
    fn assert_not_empty<T>(collection: List<T>, message: string);
    fn assert_contains<T>(collection: List<T>, item: T, message: string);
    
    // Error assertions
    fn assert_throws(fn: Function, error_type: string, message: string);
    
    // Approximate equality (for floats)
    fn assert_approx_eq(actual: float, expected: float, epsilon: float, message: string);
}
```

### 4. Mocking System

```dal
@test
fn test_with_mocks() {
    // Mock blockchain calls
    let mock_chain = test::mock("chain");
    mock_chain.expect("get_balance")
        .with_args("0x123...", "ETH")
        .returns(100.0);
    
    // Mock AI calls
    let mock_ai = test::mock("ai");
    mock_ai.expect("create_agent")
        .returns({ "id": "agent-123", "status": "active" });
    
    // Run code that uses mocks
    let service = MyService::new();
    let balance = service.get_user_balance("0x123...");
    
    test::assert_eq(balance, 100.0, "Should return mocked balance");
    mock_chain.verify(); // Ensure all expectations were met
}
```

### 5. Fixtures and Setup/Teardown

```dal
// fixtures.dal
fn setup_test_environment() {
    // Create test database
    // Initialize test blockchain
    // Set up test accounts
}

fn cleanup_test_environment() {
    // Clean up test data
    // Reset state
}

// test_file.dal
@test
@before_all(setup_test_environment)
@after_all(cleanup_test_environment)
@before_each(setup_test_data)
@after_each(cleanup_test_data)
fn test_with_fixtures() {
    // Test code here
}
```

### 6. Snapshot Testing

```dal
@test
fn test_output_snapshot() {
    let result = complex_calculation();
    test::assert_snapshot(result, "complex_calculation_output");
}
```

### 7. Integration with Blockchain Testing

```dal
@test
@chain("local")  // Use local testnet
fn test_smart_contract() {
    // Deploy contract to testnet
    let contract = deploy_contract("MyContract.dal");
    
    // Interact with contract
    contract.call("setValue", [42]);
    let value = contract.call("getValue", []);
    
    test::assert_eq(value, 42, "Value should be set correctly");
}
```

## Implementation Plan

### Phase 1: Basic Test Framework

1. **Test Parser**: Extend parser to recognize `@test` attribute
2. **Test Runner**: Create CLI command `dal test`
3. **Basic Assertions**: Implement `test::assert_*` functions
4. **Test Discovery**: Find all `*_test.dal` files in `tests/` directory
5. **Test Execution**: Run tests and collect results
6. **Basic Reporting**: Print test results to console

### Phase 2: Advanced Features

1. **Mocking System**: Implement mock registry and expectations
2. **Fixtures**: Add `@before_each`, `@after_each`, etc.
3. **Parallel Execution**: Run tests in parallel
4. **Test Coverage**: Track which code is tested

### Phase 3: Integration Testing

1. **Blockchain Testing**: Integration with local testnet
2. **HTTP Mocking**: Mock HTTP requests/responses
3. **Database Fixtures**: Test database setup/teardown
4. **Snapshot Testing**: Compare outputs to saved snapshots

## Example Test File Structure

```
project/
├── src/
│   └── my_service.dal
├── tests/
│   ├── my_service_test.dal
│   ├── integration_test.dal
│   └── fixtures.dal
└── dal.toml  # Test configuration
```

## Comparison to Hardhat

| Feature | Hardhat | Proposed DAL Framework |
|---------|---------|------------------------|
| Test Discovery | Automatic (`test/` directory) | Automatic (`tests/` directory) |
| Assertions | Chai.js | Built-in `test::assert_*` |
| Mocking | Hardhat network, sinon | `test::mock()` |
| Fixtures | `beforeEach`, `afterEach` | `@before_each`, `@after_each` |
| Blockchain Testing | Hardhat Network | Local testnet integration |
| Parallel Execution | Yes | Yes (Phase 2) |
| Coverage Reports | Yes | Yes (Phase 2) |
| Snapshot Testing | No | Yes (Phase 3) |

## Usage Example

```bash
# Run all tests
$ dal test

Running tests...
✅ test_basic_math (0.001s)
✅ test_string_operations (0.002s)
✅ test_service_initialization (0.005s)
❌ test_with_mocks (0.010s)
   Error: Expected balance 100.0, got 0.0

Tests: 4 total, 3 passed, 1 failed
Time: 0.018s
```

## Benefits

1. **Familiar Pattern**: Developers familiar with Hardhat/Jest will feel at home
2. **Type Safety**: Leverages DAL's type system
3. **Integrated**: Works seamlessly with DAL's blockchain/AI features
4. **Fast**: Can run tests without deploying to real networks
5. **Comprehensive**: Supports unit, integration, and end-to-end tests

## Next Steps

1. **Design Test Syntax**: Finalize the exact syntax for test files
2. **Implement Test Parser**: Extend parser to handle test attributes
3. **Build Test Runner**: Create CLI command and test execution engine
4. **Add Assertions**: Implement assertion library in standard library
5. **Create Examples**: Write example test files for each example in `examples/`

# Example Testing Guide

## Overview

This guide explains how to test DAL examples to ensure they compile and work correctly, and outlines the plan for building a Hardhat-like testing framework.

## Current State

### What We Have

1. **DAL Compiler/Runtime**: The `dist_agent_lang` binary can:
   - Tokenize DAL source code
   - Parse DAL source code into an AST
   - Execute DAL programs

2. **Library Functions**: The `dist_agent_lang` library provides:
   - `parse_source(source: &str)` - Parses DAL code, returns AST or error
   - `execute_source(source: &str)` - Executes DAL code, returns result or error

3. **Basic CLI Commands**:
   - `dist_agent_lang run <file.dal>` - Runs a DAL file
   - `dist_agent_lang test` - Runs Rust unit tests (not DAL tests)

4. **Rust Unit Tests**: Comprehensive test suite in `tests/` directory:
   - `tests/example_tests.rs` - Tests all example files compile and execute
   - `tests/core_language_tests.rs` - Tests language features
   - `tests/integration/` - Integration tests

### What's Missing

1. **Parse-Only Command**: No `dist_agent_lang parse <file.dal>` command yet
2. **Test Framework**: No way to write tests in DAL itself
3. **Assertion Library**: No built-in test assertions
4. **Test Discovery**: No automatic test file discovery
5. **Mocking**: No way to mock external dependencies

## Testing Examples Right Now

### Option 1: Rust Unit Tests (RECOMMENDED) ⭐

**This is the proper way to test examples!** Use Rust unit tests that leverage the library functions.

```bash
# Run all example tests
cargo test example_tests

# Run specific test
cargo test test_hello_world_demo_parses

# Run all tests (including examples)
cargo test
```

**Benefits**:
- ✅ Runs automatically with `cargo test`
- ✅ Integrates with CI/CD pipelines
- ✅ Provides structured test results
- ✅ Can test compilation separately from execution
- ✅ Can skip tests that require external dependencies
- ✅ Fails fast if examples break

**Test File**: `tests/example_tests.rs` contains:
- Individual tests for each example file
- Comprehensive test that checks all examples parse
- Execution tests for simple examples (skips ones requiring external deps)
- Category-specific tests (basic, blockchain, AI, web)

### Option 2: Manual Testing (For Quick Checks)

```bash
# Test if a file compiles and runs
dist_agent_lang run examples/hello_world_demo.dal

# If it runs without errors, it compiles correctly
# If it errors, check the error message
```

**Limitations**:
- Requires manual execution of each file
- No structured test results
- Hard to automate
- Can't test specific functions

### Option 3: Using the Test Script (Alternative)

A test script has been created at `scripts/test_examples.sh`:

```bash
# Run all examples
./scripts/test_examples.sh

# Only test compilation (faster)
./scripts/test_examples.sh --compile-only

# Only test execution
./scripts/test_examples.sh --execute-only

# Custom examples directory
./scripts/test_examples.sh --examples-dir my_examples
```

**Note**: This script currently uses `dist_agent_lang parse` which doesn't exist yet. However, **Rust unit tests are preferred** as they integrate better with the development workflow.

### Option 3: Using Rust Tests (For Library Code)

The language implementation itself is tested with Rust unit tests:

```bash
# Run all Rust tests
cargo test

# Run specific test
cargo test test_lexer_basic_tokens
```

## What's Already Done ✅

### Rust Unit Tests for Examples

A comprehensive test suite has been created in `tests/example_tests.rs`:

1. **Individual Tests**: Each example file has its own test function
2. **Comprehensive Test**: `test_all_examples_parse()` checks all examples at once
3. **Execution Tests**: Tests simple examples that don't require external dependencies
4. **Category Tests**: Groups examples by type (basic, blockchain, AI, web)

**Run the tests**:
```bash
cargo test example_tests
```

## What Could Be Added (Optional)

### Optional: Parse Command

Add a `parse` command to `src/main.rs` for quick CLI checks:

```rust
"parse" => {
    if args.len() < 3 {
        eprintln!("Usage: dist_agent_lang parse <file.dal>");
        std::process::exit(1);
    }
    parse_dal_file(&args[2]);
}
```

**Note**: This is optional since Rust unit tests already cover this functionality.

### Future: DAL Test Framework

For writing tests in DAL itself (like Hardhat), see `TESTING_FRAMEWORK_PROPOSAL.md`. This would be useful for:
- Writing tests alongside DAL code
- Testing DAL-specific features
- Integration with DAL development workflow

However, **Rust unit tests are sufficient and recommended** for testing examples.

## Testing Requirements by Example Type

### 1. Basic Examples (e.g., `hello_world_demo.dal`)

**What to Test**:
- ✅ Compiles (parses successfully)
- ✅ Executes without errors
- ✅ Produces expected output (if deterministic)

**How to Test**:
```bash
dist_agent_lang run examples/hello_world_demo.dal
```

### 2. Service Examples (e.g., `smart_contract.dal`)

**What to Test**:
- ✅ Compiles (syntax is correct)
- ✅ Service definitions are valid
- ⚠️ May require blockchain connection for full execution

**How to Test**:
```bash
# Test compilation
dist_agent_lang parse examples/smart_contract.dal

# Test execution (may fail if blockchain not available)
dist_agent_lang run examples/smart_contract.dal
```

### 3. Blockchain Examples (e.g., `cross_chain_patterns.dal`)

**What to Test**:
- ✅ Compiles
- ⚠️ Requires blockchain RPC connection
- ⚠️ May require testnet or mock blockchain

**How to Test**:
```bash
# Test compilation only
dist_agent_lang parse examples/cross_chain_patterns.dal

# Full execution requires:
# - RPC URL configured
# - Testnet or local blockchain
# - Test tokens/accounts
```

### 4. AI Examples (e.g., `llm_integration_examples.dal`)

**What to Test**:
- ✅ Compiles
- ⚠️ Requires AI API key
- ⚠️ May incur API costs

**How to Test**:
```bash
# Test compilation only
dist_agent_lang parse examples/llm_integration_examples.dal

# Full execution requires:
# - DIST_AGENT_AI_API_KEY environment variable
# - Valid API key
```

### 5. Web Examples (e.g., `simple_web_api_example.dal`)

**What to Test**:
- ✅ Compiles
- ✅ Can start HTTP server (if implemented)
- ⚠️ Requires HTTP client to test endpoints

**How to Test**:
```bash
# Test compilation
dist_agent_lang parse examples/simple_web_api_example.dal

# Test execution (starts server)
dist_agent_lang web examples/simple_web_api_example.dal
```

## Testing Checklist

For each example file, verify:

- [ ] **Syntax**: File parses without syntax errors
- [ ] **Types**: All types are valid (if type checking is implemented)
- [ ] **References**: All function/service references are valid
- [ ] **Execution**: Code runs without runtime errors (if dependencies available)
- [ ] **Output**: Produces expected output (if deterministic)

## Example Test Results Format

```
Testing examples/hello_world_demo.dal...
  ✅ Compilation: PASSED
  ✅ Execution: PASSED
  ✅ Output matches expected

Testing examples/smart_contract.dal...
  ✅ Compilation: PASSED
  ⏭️  Execution: SKIPPED (requires blockchain connection)

Testing examples/broken_example.dal...
  ❌ Compilation: FAILED
     Error: Syntax error at line 42, column 15
     Expected: ';', found: '}'
```

## Next Steps

1. ✅ **Rust Unit Tests Created**: `tests/example_tests.rs` is ready
2. **Run Tests**: Execute `cargo test example_tests` to verify all examples compile
3. **Fix Any Failures**: Update examples that fail to parse
4. **Add More Tests**: Add execution tests for more examples as features are implemented
5. **CI Integration**: Ensure tests run in CI/CD pipeline

## Why Rust Unit Tests Are Important

1. **Automation**: Run automatically with `cargo test`
2. **CI/CD Integration**: Easy to integrate into GitHub Actions, GitLab CI, etc.
3. **Fast Feedback**: Developers get immediate feedback when examples break
4. **Regression Prevention**: Prevents examples from breaking when language changes
5. **Documentation**: Tests serve as documentation of what works
6. **Reliability**: More reliable than manual testing

## Comparison: Current vs. Proposed

| Feature | Current | Proposed (Hardhat-like) |
|---------|---------|--------------------------|
| Test Syntax | None | `@test` attribute |
| Assertions | None | `test::assert_*` functions |
| Test Discovery | Manual | Automatic (`tests/` directory) |
| Mocking | None | `test::mock()` |
| Fixtures | None | `@before_each`, `@after_each` |
| Test Runner | Manual script | `dal test` command |
| Reports | Console output | Structured JSON/HTML |
| Parallel Execution | No | Yes |

## Resources

- **Testing Framework Proposal**: `docs/TESTING_FRAMEWORK_PROPOSAL.md`
- **Test Script**: `scripts/test_examples.sh`
- **Rust Tests**: `tests/` directory
- **Examples**: `examples/` directory

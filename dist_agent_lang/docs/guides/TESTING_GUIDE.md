# Comprehensive Testing Guide for DAL

This guide covers all testing approaches in the Dist Agent Language ecosystem.

## Quick Start

```bash
# Layer 1: Run Rust unit tests (syntax validation)
cd dist_agent_lang
cargo test

# Layer 2: Semantic validation (used within tests)
# See examples in tests/example_tests.rs

# Layer 3: Run DAL test files (runtime behavior)
./scripts/run_dal_tests.sh
```

## Testing Philosophy

DAL uses a **three-layer testing strategy** that provides comprehensive validation at different levels:

1. **Layer 1**: Fast syntax validation (Rust unit tests)
2. **Layer 2**: Semantic validation (validation helpers)
3. **Layer 3**: Runtime behavior testing (DAL test files)

For complete details, see [`THREE_LAYER_TESTING.md`](../THREE_LAYER_TESTING.md).

## Layer 1: Rust Unit Tests

### Purpose
Validate that all DAL code parses correctly and has valid syntax.

### Location
`/dist_agent_lang/tests/example_tests.rs`

### Running Tests

```bash
# Run all tests
cargo test

# Run only example tests
cargo test --test example_tests

# Run specific test
cargo test test_hello_world_demo_parses

# Run with verbose output
cargo test -- --nocapture

# Run tests for a category
cargo test test_blockchain_examples_parse
```

### What Gets Tested

✅ All `.dal` files in `examples/` directory
✅ Syntax correctness
✅ Parse-time validation
✅ Attribute syntax
✅ Statement structure

### Adding New Tests

Tests automatically include any `.dal` file in the `examples/` directory. To add a new test:

1. Add your `.dal` file to `examples/`
2. Run `cargo test`
3. The test suite will automatically pick it up

### Example Output

```
running 25 tests
test test_hello_world_demo_parses ... ok
test test_cross_chain_patterns_parses ... ok
test test_token_contract_parses ... ok
test test_all_examples_parse ... ok

test result: ok. 25 passed; 0 failed; 0 ignored
```

## Layer 2: Semantic Validators

### Purpose
Validate semantic meaning beyond syntax (attribute values, types, rules).

### Location
`/dist_agent_lang/src/stdlib/test.rs`

### Available Functions

#### Attribute Validation

```rust
// Validate trust models
test::expect_valid_trust_model("hybrid")        // ✓
test::expect_valid_trust_model("centralized")   // ✓
test::expect_valid_trust_model("decentralized") // ✓
test::expect_valid_trust_model("invalid")       // ✗ Error

// Validate blockchains
test::expect_valid_chain("ethereum")  // ✓
test::expect_valid_chain("polygon")   // ✓
test::expect_valid_chain("solana")    // ✓
test::expect_valid_chain("fake")      // ✗ Error

// Validate attribute compatibility
test::expect_compatible_attributes(["trust", "chain"])   // ✓
test::expect_compatible_attributes(["trust"])            // ✗ Needs @chain
test::expect_compatible_attributes(["secure", "public"]) // ✗ Exclusive
```

#### Type Validation

```rust
// Validate types
let value = Value::Number(42.0);
test::expect_type(&value, "number")  // ✓
test::expect_type(&value, "string")  // ✗ Error

// Validate ranges
test::expect_in_range(Value::Number(50.0), 0.0, 100.0)  // ✓
test::expect_in_range(Value::Number(150.0), 0.0, 100.0) // ✗ Out of range
```

#### Collection Validation

```rust
// Validate length
test::expect_length(Value::String("hello"), 5)  // ✓

// Validate not empty
test::expect_not_empty(Value::Vector(vec![...]))  // ✓

// Validate map keys
test::expect_has_key(map, "key")  // ✓
```

#### String Validation

```rust
test::expect_contains("hello world", "world")  // ✓
test::expect_starts_with("0x123", "0x")        // ✓
```

### Using in Rust Tests

```rust
#[test]
fn test_semantic_validation() {
    let source = r#"
        @trust("hybrid")
        @chain("ethereum")
        service MyService {}
    "#;
    
    let ast = parse_source(source).unwrap();
    
    if let Statement::Service(service) = &ast.statements[0] {
        // Validate trust model
        let trust_attr = service.attributes.iter()
            .find(|a| a.name == "trust")
            .unwrap();
        
        if let Expression::Literal(Literal::String(model)) = &trust_attr.parameters[0] {
            expect_valid_trust_model(model).unwrap();
        }
    }
}
```

### Using in DAL Tests

```dal
describe("Semantic Validation", fn() {
    it("should validate trust model", fn() {
        test::expect_valid_trust_model("hybrid");
        test::expect_valid_chain("ethereum");
    });
});
```

## Layer 3: DAL Test Files

### Purpose
Test runtime behavior and integration, written in DAL itself.

### File Convention
- Filename: `*.test.dal`
- Location: Alongside implementation files in `examples/`
- Runner: `./scripts/run_dal_tests.sh`

### Syntax (Hardhat-style)

```dal
describe("Test Suite Name", fn() {
    // Setup
    let contract;
    
    beforeEach(fn() {
        // Runs before each test
        contract = deploy_service("MyService", {});
    });
    
    afterEach(fn() {
        // Runs after each test
        reset_context();
    });
    
    it("should do something", fn() {
        // Test implementation
        let result = contract.some_method();
        expect(result).to_equal(expected);
    });
});
```

### Assertions

```dal
// Equality
expect(value).to_equal(expected);
expect(value).not_to_equal(other);

// Booleans
expect(value).to_be_true();
expect(value).to_be_false();

// Null checks
expect(value).to_be_nil();
expect(value).not_to_be_nil();

// Exceptions
expect_throws(fn() {
    some_error_producing_code();
}, "error message");

// Layer 2 semantic validation
test::expect_type(&value, "number");
test::expect_in_range(value, 0.0, 100.0);
test::expect_valid_trust_model("hybrid");
```

### Lifecycle Hooks

```dal
beforeAll(fn() {
    // Runs once before all tests in suite
});

beforeEach(fn() {
    // Runs before each test
});

afterEach(fn() {
    // Runs after each test
});

afterAll(fn() {
    // Runs once after all tests in suite
});
```

### Example Test File

See [`token_contract.test.dal`](../../examples/token_contract.test.dal) for a complete example.

### Running DAL Tests

```bash
# Run all test files
./scripts/run_dal_tests.sh

# Run specific test file
cargo run --release -- run examples/token_contract.test.dal

# Run with verbose output
cargo run --release -- run examples/token_contract.test.dal --verbose
```

## Testing Workflow

### 1. During Development

```bash
# Watch mode for Rust tests
cargo watch -x test

# Quick syntax check
cargo test --test example_tests
```

### 2. Before Commit

```bash
# Full test suite
cargo test

# Run DAL tests
./scripts/run_dal_tests.sh
```

### 3. In CI/CD

```yaml
# .github/workflows/test.yml
- name: Run Rust tests
  run: cargo test

- name: Run DAL tests
  run: ./scripts/run_dal_tests.sh
```

## Testing Best Practices

### 1. Test Pyramid

```
        ╱╲
       ╱  ╲     Few: Integration tests (Layer 3)
      ╱────╲
     ╱      ╲   Medium: Semantic tests (Layer 2)
    ╱────────╲
   ╱          ╲ Many: Unit tests (Layer 1)
  ╱────────────╲
```

- **Many** fast unit tests (Layer 1)
- **Medium** semantic validation (Layer 2)
- **Few** comprehensive integration tests (Layer 3)

### 2. Test Naming

```rust
// Rust tests
#[test]
fn test_<feature>_<scenario>() { }

// Example:
fn test_token_transfer_succeeds_with_valid_amount() { }
```

```dal
// DAL tests
describe("Feature", fn() {
    it("should do something specific", fn() { });
});

// Example:
describe("TokenContract", fn() {
    it("should transfer tokens with valid amount", fn() { });
});
```

### 3. Arrange-Act-Assert

```dal
it("should update balance after transfer", fn() {
    // Arrange
    let initial_balance = contract.balance_of("alice");
    let amount = 100.0;
    
    // Act
    contract.transfer("bob", amount);
    
    // Assert
    let final_balance = contract.balance_of("alice");
    expect(final_balance).to_equal(initial_balance - amount);
});
```

### 4. Test Independence

```dal
describe("Tests", fn() {
    beforeEach(fn() {
        // Fresh state for each test
        contract = deploy_service("MyService", {});
    });
    
    it("test 1", fn() { /* ... */ });
    it("test 2", fn() { /* ... */ }); // Independent of test 1
});
```

### 5. Edge Cases

```dal
describe("Edge Cases", fn() {
    it("should handle zero values", fn() { /* ... */ });
    it("should handle maximum values", fn() { /* ... */ });
    it("should handle empty inputs", fn() { /* ... */ });
    it("should reject invalid inputs", fn() { /* ... */ });
});
```

## Common Testing Patterns

### Pattern 1: Attribute Validation

```rust
#[test]
fn test_service_attributes() {
    let source = "@trust(\"hybrid\") @chain(\"ethereum\") service S {}";
    let ast = parse_source(source).unwrap();
    
    // Extract attributes
    if let Statement::Service(s) = &ast.statements[0] {
        let attrs: Vec<&str> = s.attributes.iter()
            .map(|a| a.name.as_str())
            .collect();
        
        // Validate
        expect_compatible_attributes(attrs).unwrap();
    }
}
```

### Pattern 2: State Changes

```dal
it("should update state correctly", fn() {
    let before = contract.get_state();
    
    contract.modify_state();
    
    let after = contract.get_state();
    expect(after).not_to_equal(before);
});
```

### Pattern 3: Error Handling

```dal
it("should reject invalid input", fn() {
    expect_throws(fn() {
        contract.risky_operation(-1);
    }, "must be positive");
});
```

### Pattern 4: Integration Testing

```dal
describe("Multi-service integration", fn() {
    let token;
    let escrow;
    
    beforeEach(fn() {
        token = deploy_service("Token", {});
        escrow = deploy_service("Escrow", { "token": token });
    });
    
    it("should integrate services", fn() {
        token.approve(escrow, 100.0);
        escrow.deposit(100.0);
        
        expect(token.balance_of(escrow)).to_equal(100.0);
    });
});
```

## Debugging Failed Tests

### 1. Rust Test Failures

```bash
# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name -- --nocapture

# Show full backtraces
RUST_BACKTRACE=1 cargo test
```

### 2. DAL Test Failures

```bash
# Run with verbose mode
cargo run --release -- run test.dal --verbose

# Add print statements
print("Debug: value =", value);

# Check parsed AST
cargo run --release -- parse test.dal
```

### 3. Common Issues

**Issue**: Test fails with "Parse error"
**Solution**: Run Layer 1 tests first to check syntax

**Issue**: Semantic validation fails
**Solution**: Check attribute values match allowed options

**Issue**: Runtime behavior unexpected
**Solution**: Add debug prints, check service state

## Performance Testing

```dal
describe("Performance", fn() {
    it("should complete in reasonable time", fn() {
        let start = time::now();
        
        // Operation to test
        for i in 0..1000 {
            contract.operation();
        }
        
        let duration = time::now() - start;
        test::expect_in_range(duration, 0.0, 1000.0); // < 1 second
    });
});
```

## Coverage Reporting

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

## Continuous Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run Layer 1 tests
        run: cargo test
      
      - name: Run Layer 3 tests
        run: ./scripts/run_dal_tests.sh
```

## Summary

| Layer | Tool | Speed | Purpose |
|-------|------|-------|---------|
| **Layer 1** | `cargo test` | ⚡ Fast | Syntax validation |
| **Layer 2** | `test::expect_*()` | 🚀 Medium | Semantic validation |
| **Layer 3** | `./scripts/run_dal_tests.sh` | 🐌 Slower | Runtime behavior |

**Best Practice**: Start with Layer 1, add Layer 2 for semantic rules, and Layer 3 for critical paths.

For more details, see:
- [`THREE_LAYER_TESTING.md`](../THREE_LAYER_TESTING.md) - Complete strategy overview
- [`TESTING_ATTRIBUTES.md`](../TESTING_ATTRIBUTES.md) - Attribute testing deep dive
- [`WHY_RUST_UNIT_TESTS.md`](../WHY_RUST_UNIT_TESTS.md) - Rationale for Rust tests

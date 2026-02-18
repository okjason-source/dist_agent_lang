# Testing Quick Reference

## Three-Layer Testing Strategy - Quick Reference Card

```
┌─────────────────────────────────────────────────────────────┐
│                    LAYER 1: SYNTAX                          │
│              Rust Unit Tests (milliseconds)                  │
├─────────────────────────────────────────────────────────────┤
│ Command:    cargo test                                      │
│ Location:   tests/example_tests.rs                          │
│ Tests:      Syntax, parsing, AST construction               │
│ Coverage:   All .dal files in examples/                     │
└─────────────────────────────────────────────────────────────┘
                            ↓ Pass
┌─────────────────────────────────────────────────────────────┐
│                   LAYER 2: SEMANTICS                        │
│            Validation Functions (seconds)                    │
├─────────────────────────────────────────────────────────────┤
│ Location:   src/stdlib/test.rs                              │
│ Usage:      test::expect_*()                                │
│ Tests:      Attributes, types, rules, constraints           │
│ Coverage:   Domain-specific validation                      │
└─────────────────────────────────────────────────────────────┘
                            ↓ Pass
┌─────────────────────────────────────────────────────────────┐
│                   LAYER 3: BEHAVIOR                         │
│           DAL Test Files (seconds to minutes)                │
├─────────────────────────────────────────────────────────────┤
│ Command:    ./scripts/run_dal_tests.sh                      │
│ Files:      examples/*.test.dal                              │
│ Tests:      Runtime behavior, integration, logic            │
│ Coverage:   User-defined critical paths                     │
└─────────────────────────────────────────────────────────────┘
```

## Layer 1: Rust Unit Tests

### Run Tests
```bash
cargo test                         # All tests (syntax + semantics!)
cargo test --test example_tests    # Example tests only
cargo test test_all_examples_with_semantic_validation  # Semantic validation
cargo test test_name               # Specific test
cargo test -- --nocapture          # With output
```

### What It Checks
✅ Token recognition  
✅ Syntax correctness  
✅ AST construction  
✅ Attribute syntax  
✅ Statement structure  
✅ **Semantic validation** (NEW!)  
✅ **Trust model values** (NEW!)  
✅ **Chain identifiers** (NEW!)  
✅ **Attribute compatibility** (NEW!)  

## Layer 2: Semantic Validators

### Available Functions
```rust
// Attributes
test::expect_valid_trust_model("hybrid")
test::expect_valid_chain("ethereum")
test::expect_compatible_attributes(["trust", "chain"])

// Types
test::expect_type(&value, "number")
test::expect_in_range(value, 0.0, 100.0)

// Strings
test::expect_contains("hello world", "world")
test::expect_starts_with("0x123", "0x")

// Collections
test::expect_length(value, 5)
test::expect_not_empty(value)
test::expect_has_key(map, "key")
```

### What It Checks
✅ Trust models: hybrid, centralized, decentralized, trustless  
✅ Chains: ethereum, polygon, bsc, solana, bitcoin, etc.  
✅ Types: number, string, bool, map, vector, null, function  
✅ Ranges: min/max validation  
✅ Compatibility: @trust requires @chain, @secure ⊕ @public  

## Layer 3: DAL Test Files

### Create Test File
**Filename:** `my_feature.test.dal`

```dal
describe("Feature Name", fn() {
    let contract;
    
    beforeEach(fn() {
        contract = deploy_service("MyService", {});
    });
    
    it("should do something", fn() {
        // Act
        contract.method();
        
        // Assert - Runtime
        expect(contract.state()).to_equal(expected);
        
        // Assert - Semantic
        test::expect_type(&result, "number");
        test::expect_in_range(result, 0.0, 100.0);
    });
});
```

### Run Tests
```bash
./scripts/run_dal_tests.sh                      # All tests
cargo run --release -- run file.test.dal        # Specific file
```

### Available Assertions
```dal
// Equality
expect(value).to_equal(expected)
expect(value).not_to_equal(other)

// Booleans
expect(value).to_be_true()
expect(value).to_be_false()

// Null
expect(value).to_be_nil()
expect(value).not_to_be_nil()

// Exceptions
expect_throws(fn() { code(); }, "error message")

// Semantic (from Layer 2)
test::expect_valid_trust_model("hybrid")
test::expect_type(&value, "number")
test::expect_in_range(value, 0.0, 100.0)
```

### Lifecycle Hooks
```dal
beforeAll(fn() { /* once before all */ })
beforeEach(fn() { /* before each test */ })
afterEach(fn() { /* after each test */ })
afterAll(fn() { /* once after all */ })
```

## Common Patterns

### Pattern 1: Attribute Validation
```rust
#[test]
fn test_attributes() {
    let source = "@trust(\"hybrid\") @chain(\"ethereum\") service S {}";
    let ast = parse_source(source).unwrap();
    
    if let Statement::Service(s) = &ast.statements[0] {
        // Extract attribute names
        let attrs: Vec<&str> = s.attributes.iter()
            .map(|a| a.name.as_str())
            .collect();
        
        // Validate compatibility
        expect_compatible_attributes(attrs).unwrap();
        
        // Validate specific values
        for attr in &s.attributes {
            match attr.name.as_str() {
                "trust" => {
                    if let Expression::Literal(Literal::String(m)) = &attr.parameters[0] {
                        expect_valid_trust_model(m).unwrap();
                    }
                }
                "chain" => {
                    if let Expression::Literal(Literal::String(c)) = &attr.parameters[0] {
                        expect_valid_chain(c).unwrap();
                    }
                }
                _ => {}
            }
        }
    }
}
```

### Pattern 2: Runtime Behavior
```dal
it("should update state correctly", fn() {
    // Arrange
    let before = contract.get_value();
    
    // Act
    contract.set_value(42.0);
    
    // Assert
    let after = contract.get_value();
    expect(after).to_equal(42.0);
    expect(after).not_to_equal(before);
    
    // Semantic validation
    test::expect_type(&after, "number");
    test::expect_in_range(after, 0.0, 100.0);
})
```

### Pattern 3: Error Handling
```dal
it("should reject invalid input", fn() {
    expect_throws(fn() {
        contract.risky_operation(-1);
    }, "must be positive");
})
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
    
    it("should integrate", fn() {
        token.approve(escrow, 100.0);
        escrow.deposit(100.0);
        expect(token.balance_of(escrow)).to_equal(100.0);
    });
})
```

## Debugging

### Layer 1 Failures
```bash
# Show full output
cargo test -- --nocapture

# Show backtraces
RUST_BACKTRACE=1 cargo test

# Run specific test
cargo test test_name -- --nocapture
```

### Layer 2 Failures
Check semantic validation functions are being called with correct parameters.

### Layer 3 Failures
```bash
# Add debug prints
print("Debug:", value);

# Check parsed AST
cargo run --release -- parse file.dal

# Run with verbose mode
cargo run --release -- run file.test.dal --verbose
```

## CI/CD Integration

```yaml
# .github/workflows/test.yml
- name: Layer 1 - Rust Unit Tests
  run: cargo test

- name: Layer 2 - Semantic Validation
  run: cargo test --test example_tests

- name: Layer 3 - DAL Test Files
  run: ./scripts/run_dal_tests.sh
```

## Documentation

- **Full Strategy**: [`THREE_LAYER_TESTING.md`](THREE_LAYER_TESTING.md)
- **Comprehensive Guide**: [`guides/TESTING_GUIDE.md`](guides/TESTING_GUIDE.md)
- **Attribute Testing**: [`TESTING_ATTRIBUTES.md`](TESTING_ATTRIBUTES.md)
- **Semantic Validation**: [`SEMANTIC_VALIDATION_FEATURE.md`](SEMANTIC_VALIDATION_FEATURE.md)
- **Rust Tests Rationale**: [`WHY_RUST_UNIT_TESTS.md`](WHY_RUST_UNIT_TESTS.md)
- **Implementation Summary**: [`IMPLEMENTATION_SUMMARY.md`](IMPLEMENTATION_SUMMARY.md)

## Summary Table

| Layer | Command | Speed | Purpose | Coverage |
|-------|---------|-------|---------|----------|
| **1** | `cargo test` | ⚡ Fast | Syntax + Semantics | 100% examples |
| **2** | `test::expect_*()` | 🚀 Medium | Standalone validators | Rules |
| **3** | `./scripts/run_dal_tests.sh` | 🐌 Slower | Behavior | Critical paths |

**Note**: Layer 2 validators are now integrated into Layer 1 tests!

## Quick Checklist

Adding new features:
- [ ] Add `.dal` file to `examples/`
- [ ] Run `cargo test` (Layer 1)
- [ ] Add semantic validators if needed (Layer 2)
- [ ] Write `.test.dal` for critical logic (Layer 3)
- [ ] Update attribute compatibility rules
- [ ] Document in README/docs

---

**Status: ✅ Ready to Use**

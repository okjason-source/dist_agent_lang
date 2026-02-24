# Three-Layer Testing Strategy for DAL

This document describes the comprehensive three-layer testing strategy for the Dist Agent Language (DAL).

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Rust Unit Tests (example_tests.rs)                │
│ Purpose: Syntax & Parse-time Validation                     │
│ Tools: cargo test, parse_source(), execute_source()         │
│ Runs: CI/CD, pre-commit, development                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Semantic Validators (stdlib/test.rs)              │
│ Purpose: Attribute, Type & Rule Validation                  │
│ Tools: expect_valid_trust_model(), expect_type(), etc.      │
│ Runs: Within DAL tests, programmatic validation             │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: DAL Test Files (*.test.dal)                       │
│ Purpose: Runtime Behavior & Integration Testing             │
│ Tools: describe(), it(), expect(), deploy_service()         │
│ Runs: dist_agent_lang test, user-written tests              │
└─────────────────────────────────────────────────────────────┘
```

## Layer 1: Rust Unit Tests

**Location**: `/dist_agent_lang/tests/example_tests.rs`

### Purpose
Validate that all DAL example files can be parsed and executed without errors.

### What It Tests
- ✅ **Syntax correctness**: All tokens are valid
- ✅ **Parse-time validation**: AST can be constructed
- ✅ **Attribute syntax**: `@` attributes are properly formatted
- ✅ **Structure validity**: Functions, services, statements are well-formed

### Example

```rust
#[test]
fn test_token_contract_parses() {
    let source = r#"
        @trust("hybrid")
        @chain("ethereum")
        service TokenContract {
            balance: map<string, float>;
            
            fn transfer(to: string, amount: float) {
                balance[to] = balance[to] + amount;
            }
        }
    "#;
    
    // Layer 1: Parse validation
    parse_source(&source).unwrap();
}
```

### When It Runs
- `cargo test` (local development)
- GitHub Actions CI/CD
- Pre-commit hooks (optional)

### Validation Capabilities

**Syntax Validation** (Parse-time)
- ✅ Token recognition and lexical analysis
- ✅ Grammar and structure validation
- ✅ AST construction and integrity
- ✅ Attribute syntax and positioning

**Semantic Validation** (Parse-time)
- ✅ Trust model validation (hybrid, centralized, decentralized, trustless)
- ✅ Blockchain identifier validation (ethereum, polygon, bsc, solana, etc.)
- ✅ Attribute mutual exclusivity (@secure and @public)
- ✅ Domain constraint enforcement

**Runtime Behavior** (Layer 3)
- For runtime behavior testing, use Layer 3 DAL test files (`.test.dal`)
- This separation ensures fast feedback for syntax/semantic issues

**Key Feature**: The `test_all_examples_with_semantic_validation()` test combines syntax and semantic validation in a single fast test run!

---

## Layer 2: Semantic Validators

**Location**: `/dist_agent_lang/src/stdlib/test.rs`

### Purpose
Provide semantic validation helpers that go beyond syntax checking.

### What It Tests
- ✅ **Attribute semantics**: Valid trust models, chain names, etc.
- ✅ **Type validation**: Values match expected types
- ✅ **Rule enforcement**: Attribute compatibility, constraints
- ✅ **Domain logic**: Business rules, value ranges

### Available Functions

#### Attribute Validation
```rust
// Validate trust model
expect_valid_trust_model("hybrid")       // ✅ Pass
expect_valid_trust_model("invalid")      // ❌ Fail

// Validate blockchain
expect_valid_chain("ethereum")           // ✅ Pass
expect_valid_chain("fake_chain")         // ❌ Fail

// Check attribute compatibility
expect_compatible_attributes(vec!["trust"])            // ✅ Pass
expect_compatible_attributes(vec!["trust", "chain"])  // ✅ Pass
expect_compatible_attributes(vec!["secure", "public"]) // ❌ Fail (exclusive)
```

#### Type Validation
```rust
// Validate value type
let balance = Value::Number(100.0);
expect_type(&balance, "number")          // ✅ Pass
expect_type(&balance, "string")          // ❌ Fail

// Validate range
let amount = Value::Number(50.0);
expect_in_range(amount, 0.0, 100.0)      // ✅ Pass
expect_in_range(amount, 200.0, 300.0)    // ❌ Fail
```

#### Collection Validation
```rust
// Validate length
let name = Value::String("Alice".to_string());
expect_length(name, 5)                   // ✅ Pass

// Validate not empty
let items = Value::Vector(vec![Value::Number(1.0)]);
expect_not_empty(items)                  // ✅ Pass

// Validate map keys
let config = Value::Map(hashmap!{"host" => Value::String("localhost")});
expect_has_key(config, "host")           // ✅ Pass
```

#### String Validation
```rust
expect_contains("hello world", "world")  // ✅ Pass
expect_starts_with("0x123abc", "0x")     // ✅ Pass
```

### Example Usage in Rust Tests

```rust
#[test]
fn test_token_contract_semantics() {
    let source = r#"
        @trust("hybrid")
        @chain("ethereum")
        service TokenContract { }
    "#;
    
    // Layer 1: Parse validation
    let ast = parse_source(&source).unwrap();
    
    // Layer 2: Semantic validation
    if let Statement::Service(service) = &ast.statements[0] {
        let attrs: Vec<&str> = service.attributes.iter()
            .map(|a| a.name.as_str())
            .collect();
        
        // Validate attribute compatibility
        expect_compatible_attributes(attrs).unwrap();
        
        // Validate specific attribute values
        for attr in &service.attributes {
            if attr.name == "trust" {
                if let Expression::Literal(Literal::String(model)) = &attr.parameters[0] {
                    expect_valid_trust_model(model).unwrap();
                }
            }
            if attr.name == "chain" {
                if let Expression::Literal(Literal::String(chain)) = &attr.parameters[0] {
                    expect_valid_chain(chain).unwrap();
                }
            }
        }
    }
}
```

### When It Runs
- Within Rust unit tests (Layer 1)
- Within DAL test files (Layer 3)
- Programmatic validation tools

---

## Layer 3: DAL Test Files

**Location**: `/dist_agent_lang/examples/*.test.dal`

### Purpose
Runtime behavior testing and integration testing, written in DAL itself.

### What It Tests
- ✅ **Runtime behavior**: Actual execution results
- ✅ **Service interactions**: Method calls, state changes
- ✅ **Integration**: Multiple services working together
- ✅ **Business logic**: Application-level correctness

### File Naming Convention
- `*.test.dal` - Test files
- Located alongside implementation files
- Run with `dist_agent_lang test`

### Example Test File

**File**: `token_contract.test.dal`

```dal
// Import the service to test
use token_contract::TokenContract;

// Test suite using Hardhat-style syntax
describe("TokenContract", fn() {
    let contract;
    let owner = "alice";
    let recipient = "bob";
    
    beforeEach(fn() {
        // Deploy a fresh contract for each test
        contract = deploy_service("TokenContract", {
            "initial_supply": 1000.0
        });
    });
    
    it("should initialize with correct supply", fn() {
        let supply = contract.total_supply();
        
        // Layer 2: Semantic validation
        expect_type(&supply, "number");
        expect_in_range(supply, 0.0, 1000000.0);
        
        // Layer 3: Runtime behavior
        expect(supply).to_equal(1000.0);
    });
    
    it("should transfer tokens correctly", fn() {
        // Arrange
        let initial_balance = contract.balance_of(recipient);
        let transfer_amount = 100.0;
        
        // Act
        contract.transfer(recipient, transfer_amount);
        
        // Assert - Runtime behavior
        let new_balance = contract.balance_of(recipient);
        expect(new_balance).to_equal(initial_balance + transfer_amount);
        
        // Assert - Semantic validation
        expect_in_range(new_balance, 0.0, 1000.0);
    });
    
    it("should validate trust model attribute", fn() {
        // Layer 2: Semantic validation
        expect_valid_trust_model("hybrid");
        expect_valid_chain("ethereum");
        
        // Verify attribute compatibility
        expect_compatible_attributes(["trust", "chain"]);
    });
    
    it("should reject invalid transfers", fn() {
        expect_throws(fn() {
            contract.transfer(recipient, -100.0);
        }, "negative amounts not allowed");
    });
    
    it("should handle edge cases", fn() {
        // Test boundary conditions
        let zero_transfer = 0.0;
        expect_in_range(zero_transfer, 0.0, 1000.0);
        
        contract.transfer(recipient, zero_transfer);
        expect(contract.balance_of(recipient)).to_equal(zero_transfer);
    });
});

describe("TokenContract attribute validation", fn() {
    it("should have required attributes", fn() {
        // Semantic validation
        expect_has_attribute("TokenContract", "trust");
        expect_has_attribute("TokenContract", "chain");
    });
    
    it("should enforce attribute rules", fn() {
        // These should pass
        expect_compatible_attributes(["trust", "chain"]);
        
        // This should fail
        expect_throws(fn() {
            expect_compatible_attributes(["secure", "public"]);
        }, "mutually exclusive");
    });
});
```

### Running DAL Tests

```bash
# Run all test files
dist_agent_lang test

# Run specific test file
dist_agent_lang test token_contract.test.dal

# Run with verbose output
dist_agent_lang test --verbose

# Run with coverage
dist_agent_lang test --coverage
```

### When It Runs
- `dist_agent_lang test` command
- CI/CD for integration tests
- Manual testing during development

---

## Complete Testing Flow

### Example: Testing a New Feature

**1. Write the implementation**
```dal
// token_contract.dal
@trust("hybrid")
@chain("ethereum")
service TokenContract {
    balance: map<string, float>;
    total_supply: float;
    
    fn transfer(to: string, amount: float) {
        if amount < 0.0 {
            error("negative amounts not allowed");
        }
        balance[to] = balance[to] + amount;
    }
}
```

**2. Layer 1: Add to example tests (automatic)**
```bash
# Runs automatically if file is in examples/
cargo test
```

**3. Layer 2: Add semantic validation (if needed)**
```rust
// In tests/example_tests.rs
#[test]
fn test_token_contract_semantics() {
    let source = read_file("examples/token_contract.dal");
    let ast = parse_source(&source).unwrap();
    
    // Extract and validate attributes
    // (see Layer 2 examples above)
}
```

**4. Layer 3: Write DAL test file**
```bash
# Create token_contract.test.dal
# (see Layer 3 example above)

# Run tests
dist_agent_lang test token_contract.test.dal
```

---

## Testing Checklist

When adding new DAL features, ensure:

### ✅ Layer 1: Rust Unit Tests
- [ ] Example file added to `examples/`
- [ ] File parses without errors (`cargo test`)
- [ ] Attributes have correct syntax
- [ ] All statements are well-formed

### ✅ Layer 2: Semantic Validation
- [ ] New attribute values added to validators (if applicable)
- [ ] Compatibility rules defined
- [ ] Type constraints specified
- [ ] Domain rules documented

### ✅ Layer 3: DAL Test Files
- [ ] Test file created (`*.test.dal`)
- [ ] Happy path tested
- [ ] Edge cases covered
- [ ] Error cases validated
- [ ] Integration scenarios tested

---

## Benefits of Three-Layer Testing

### 🚀 **Speed**
- Layer 1: Fast syntax validation (milliseconds)
- Layer 2: Medium semantic checks (seconds)
- Layer 3: Slower integration tests (seconds to minutes)

### 🎯 **Precision**
- Layer 1: Catches syntax errors immediately
- Layer 2: Catches semantic errors early
- Layer 3: Catches logic errors before production

### 🔄 **Feedback Loop**
- Layer 1: Instant feedback in IDE/CI
- Layer 2: Quick validation during development
- Layer 3: Comprehensive validation before release

### 📊 **Coverage**
- Layer 1: 100% of examples syntactically valid
- Layer 2: 100% of semantic rules enforced
- Layer 3: Critical paths integration tested

---

## Future Enhancements

### Planned Features

1. **AST-aware semantic validation**
   - Direct AST access in `expect_has_attribute()`
   - Automatic attribute extraction
   - Cross-reference validation

2. **Test coverage reporting**
   - Line coverage for DAL code
   - Attribute usage coverage
   - Service interaction coverage

3. **Property-based testing**
   - Fuzzing with semantic constraints
   - Generative test cases
   - Invariant checking

4. **Visual test reporting**
   - HTML test reports
   - Coverage visualizations
   - Performance metrics

---

## Summary

| Layer | Purpose | Tools | Speed | Coverage |
|-------|---------|-------|-------|----------|
| **1. Rust Unit Tests** | Syntax validation | `cargo test`, `parse_source()` | ⚡ Fast | Syntax |
| **2. Semantic Validators** | Rule enforcement | `expect_*()` functions | 🚀 Medium | Semantics |
| **3. DAL Test Files** | Runtime behavior | `describe()`, `it()` | 🐌 Slower | Logic |

**Key Principle**: Each layer builds on the previous one, providing increasingly sophisticated validation while maintaining fast feedback for common errors.

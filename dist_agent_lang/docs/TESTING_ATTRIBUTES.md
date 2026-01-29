# Testing DAL Attributes with Rust Unit Tests

## Overview

When Rust unit tests call `parse_source()` on DAL code, they validate attributes through the parser's multi-stage process.

## How It Works

### 1. Tokenization Phase

```rust
// Input DAL:
@trust("hybrid")
@chain("ethereum")
service MyContract { }

// Tokenized as:
Token::Punctuation(At)           // @
Token::Identifier("trust")        // trust
Token::Punctuation(LeftParen)    // (
Token::Literal(String("hybrid")) // "hybrid"
Token::Punctuation(RightParen)   // )
Token::Punctuation(At)           // @
Token::Identifier("chain")        // chain
// ... and so on
```

### 2. Parsing Phase

The parser recognizes the `@` symbol and calls `parse_attribute()`:

```rust
fn parse_attribute(&self, position: usize) -> Result<(usize, Attribute), ParserError> {
    // 1. Consume '@'
    // 2. Parse attribute name
    // 3. Parse parameters (if any)
    // 4. Return Attribute struct
}
```

### 3. What Gets Validated

When `parse_source()` succeeds, it means:

#### ✅ Syntax Validation
- Attribute names are valid identifiers
- Parameters are properly formatted
- Parentheses are balanced
- Attributes are in valid positions

#### ✅ Structure Validation
- Attributes appear before their targets (functions/services)
- Attribute parameters are valid expressions
- Multiple attributes can be chained

#### ✅ Position Validation
- Attributes are attached to the correct statement type
- Service attributes vs function attributes are distinguished

## Example Test Flow

### Input DAL Code
```dal
@trust("hybrid")
@chain("ethereum", "polygon")
@secure
service TokenContract {
    balance: map<string, float>;
    
    @txn
    @limit(1000)
    fn transfer(to: string, amount: float) {
        // implementation
    }
}
```

### What `parse_source()` Returns

```rust
Ok(Program {
    statements: [
        Statement::Service(ServiceStatement {
            name: "TokenContract",
            attributes: [
                Attribute {
                    name: "trust",
                    parameters: [Expression::Literal("hybrid")],
                    target: AttributeTarget::Function,
                },
                Attribute {
                    name: "chain",
                    parameters: [
                        Expression::Literal("ethereum"),
                        Expression::Literal("polygon")
                    ],
                    target: AttributeTarget::Function,
                },
                Attribute {
                    name: "secure",
                    parameters: [],
                    target: AttributeTarget::Function,
                }
            ],
            fields: [...],
            methods: [
                FunctionStatement {
                    name: "transfer",
                    attributes: [
                        Attribute { name: "txn", parameters: [], ... },
                        Attribute { name: "limit", parameters: [1000], ... }
                    ],
                    ...
                }
            ],
            ...
        })
    ]
})
```

## What Gets Caught by Tests

### ❌ Invalid Syntax
```dal
@trust(hybrid)  // FAIL: Missing quotes around string
@chain["eth"]   // FAIL: Wrong bracket type
@@secure        // FAIL: Double @@
trust("hybrid") // FAIL: Missing @ symbol
```

### ❌ Invalid Position
```dal
fn myFunc() {
    @secure  // FAIL: Attribute inside function body
    let x = 5;
}
```

### ❌ Malformed Parameters
```dal
@limit(1000, )     // FAIL: Trailing comma
@chain("eth",)     // FAIL: Trailing comma
@trust("hybrid"    // FAIL: Missing closing paren
```

## Testing Different Attribute Types

### 1. Simple Attributes (No Parameters)
```dal
@secure
@async
@public
```
**Parsed as**: `Attribute { name: "secure", parameters: [], ... }`

### 2. Single Parameter
```dal
@trust("hybrid")
@limit(1000)
```
**Parsed as**: `Attribute { name: "trust", parameters: [Expr::String("hybrid")], ... }`

### 3. Multiple Parameters
```dal
@chain("ethereum", "polygon", "bsc")
@allowed_roles("admin", "moderator")
```
**Parsed as**: `Attribute { name: "chain", parameters: [Expr::String(...), ...], ... }`

### 4. Complex Parameters
```dal
@config({ "timeout": 30, "retry": 3 })
@rate_limit(100, "per_minute")
```
**Parsed as**: Attribute with Expression::Map or Expression::Object parameters

## Runtime vs Parse-Time Validation

### Parse Time (What Tests Check)
- ✅ Syntax correctness
- ✅ Structure validity
- ✅ Token recognition
- ✅ AST construction

### Runtime (Not Checked by Parse Tests)
- ❌ Semantic meaning (e.g., is "hybrid" a valid trust model?)
- ❌ Attribute effects (e.g., does @secure actually secure?)
- ❌ Parameter values (e.g., is 1000 a reasonable limit?)
- ❌ Attribute combinations (e.g., conflicting attributes)

## Advanced Testing: Semantic Validation

To test semantic meaning, you'd need additional tests:

```rust
#[test]
fn test_attribute_semantics() {
    let source = r#"
        @trust("hybrid")
        service Test {}
    "#;
    
    let ast = parse_source(source).unwrap();
    
    // Extract service
    if let Statement::Service(service) = &ast.statements[0] {
        // Find trust attribute
        let trust_attr = service.attributes
            .iter()
            .find(|a| a.name == "trust")
            .expect("Should have trust attribute");
        
        // Validate parameter
        assert_eq!(trust_attr.parameters.len(), 1);
        
        // Check value is valid trust model
        if let Expression::Literal(Literal::String(model)) = &trust_attr.parameters[0] {
            assert!(
                ["hybrid", "centralized", "decentralized"].contains(&model.as_str()),
                "Invalid trust model: {}", model
            );
        }
    }
}
```

## Common Test Patterns

### 1. Test Attribute Exists
```rust
#[test]
fn test_service_has_trust_attribute() {
    let source = "@trust(\"hybrid\") service Test {}";
    let ast = parse_source(source).unwrap();
    // Verify attribute exists in AST
}
```

### 2. Test Attribute Count
```rust
#[test]
fn test_multiple_attributes() {
    let source = "@trust(\"hybrid\") @chain(\"eth\") @secure service Test {}";
    let ast = parse_source(source).unwrap();
    // Verify 3 attributes parsed
}
```

### 3. Test Attribute Order
```rust
#[test]
fn test_attribute_order_preserved() {
    let source = "@first @second @third service Test {}";
    let ast = parse_source(source).unwrap();
    // Verify order is preserved
}
```

## Summary

**Rust unit tests validate attributes by:**

1. **Parse-time checking**: Syntax, structure, positioning
2. **AST construction**: Attributes are properly attached to targets
3. **Error detection**: Invalid syntax causes `parse_source()` to fail

**What's NOT tested automatically:**
- Semantic meaning of attribute values
- Runtime behavior of attributes
- Attribute interaction/conflicts

**To test semantics**, you need additional tests that:
- Extract attributes from the AST
- Validate parameter values
- Check attribute combinations
- Verify runtime behavior

The current `example_tests.rs` validates that examples **parse correctly**, which includes all attribute syntax validation. Semantic validation would require additional runtime tests.

# Using `@txn` Attribute in DAL

**Version:** 1.0.5  
**Feature:** Transaction Attribute for Automatic ACID Wrapping

---

## Overview

The `@txn` attribute provides **automatic transaction wrapping** for DAL functions, ensuring ACID guarantees without manual begin/commit/rollback calls.

**Status in v1.0.5:**
- ✅ **Lexer support**: `@txn` is recognized as `Keyword::Txn` (line 519 in `lexer.rs`)
- ✅ **Parser support**: Parsed as `Attribute { name: "txn", ... }` (line 1497 in `parser.rs`)
- ⚠️ **Runtime support**: Not yet wired to `TransactionManager` (requires Phase 5 implementation)

---

## Syntax

### Basic Usage

```dal
@trust("hybrid")
@chain("ethereum")
service PaymentProcessor {
    
    @txn
    fn transfer(from: String, to: String, amount: Int) -> Result<Unit, Error> {
        // Automatically wrapped in transaction:
        // database::begin_transaction("read_committed", 30000)
        
        let balance_from = database::tx_read("balance:" + from);
        let balance_to = database::tx_read("balance:" + to);
        
        if (balance_from < amount) {
            return Err(Error::new("InsufficientFunds", "Not enough balance"));
            // Automatic rollback on error
        }
        
        database::tx_write("balance:" + from, balance_from - amount);
        database::tx_write("balance:" + to, balance_to + amount);
        
        return Ok(Unit);
        // Automatic commit on success
    }
}
```

### With Custom Isolation Level

```dal
@txn("serializable")
fn critical_operation() -> Result<Unit, Error> {
    // Uses Serializable isolation instead of default ReadCommitted
    database::tx_write("counter", database::tx_read("counter") + 1);
    return Ok(Unit);
}
```

### With Timeout

```dal
@txn("read_committed", 5000)  // 5 second timeout
fn quick_operation() -> Result<Unit, Error> {
    // Automatic rollback if exceeds 5 seconds
    database::tx_write("key", "value");
    return Ok(Unit);
}
```

---

## How It Works (Implementation Details)

### Phase 1: Parser Recognition (✅ Implemented)

When the parser encounters `@txn` before a function:

1. Lexer tokenizes `@` as `Punctuation::At`
2. Lexer tokenizes `txn` as `Keyword::Txn`
3. Parser calls `parse_attribute()` which:
   - Consumes the `@` and `txn` keyword
   - Parses optional parameters: `("serializable", 5000)`
   - Creates `Attribute { name: "txn", parameters: [...], target: Function }`
4. Attribute is attached to the function's AST node

### Phase 2: Runtime Transformation (⚠️ TODO)

When the runtime executes a function with `@txn`:

```rust
// Pseudo-code for runtime behavior
fn execute_function_with_attributes(func: &UserFunction, args: Vec<Value>) -> Result<Value, RuntimeError> {
    // Check if function has @txn attribute
    let has_txn = func.attributes.iter().any(|a| a.name == "txn");
    
    if has_txn {
        // Parse isolation level and timeout from attribute parameters
        let isolation_level = parse_isolation_from_attr(&func.attributes);
        let timeout_ms = parse_timeout_from_attr(&func.attributes);
        
        // Begin transaction
        let tx_id = runtime.begin_transaction(isolation_level, timeout_ms)?;
        
        // Execute function body
        let result = runtime.execute_function_body(func, args);
        
        // Commit or rollback based on result
        match result {
            Ok(value) => {
                runtime.commit_transaction()?;
                Ok(value)
            }
            Err(error) => {
                runtime.rollback_transaction()?;
                Err(error)
            }
        }
    } else {
        // Execute without transaction
        runtime.execute_function_body(func, args)
    }
}
```

---

## Current Status & Usage

### ✅ Available Now (v1.0.5)

**Manual transaction API:**
```dal
fn transfer(from: String, to: String, amount: Int) -> Result<Unit, Error> {
    database::begin_transaction("serializable", 30000);
    
    let balance_from = database::tx_read("balance:" + from);
    if (balance_from < amount) {
        database::rollback();
        return Err(Error::new("InsufficientFunds", "Not enough balance"));
    }
    
    database::tx_write("balance:" + from, balance_from - amount);
    database::tx_write("balance:" + to, database::tx_read("balance:" + to) + amount);
    
    database::commit();
    return Ok(Unit);
}
```

### ⚠️ Future (Phase 5)

**Automatic with `@txn`:**
```dal
@txn("serializable", 30000)
fn transfer(from: String, to: String, amount: Int) -> Result<Unit, Error> {
    // No manual begin/commit/rollback needed!
    let balance_from = database::tx_read("balance:" + from);
    if (balance_from < amount) {
        return Err(Error::new("InsufficientFunds", "Not enough balance"));
    }
    
    database::tx_write("balance:" + from, balance_from - amount);
    database::tx_write("balance:" + to, database::tx_read("balance:" + to) + amount);
    
    return Ok(Unit);
}
```

---

## Implementation Roadmap

### Current Implementation (v1.0.5)
- ✅ Lexer: Tokenizes `@txn`
- ✅ Parser: Parses as `Attribute` on functions
- ✅ AST: Stores in `UserFunction.attributes`
- ❌ Runtime: Does NOT automatically wrap in transactions yet

### Required for Full Support (Phase 5)

**Step 1: Detect `@txn` in Runtime**
```rust
// In src/runtime/engine.rs, when calling a user function:
impl Runtime {
    pub fn call_user_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let func = self.user_functions.get(name)
            .ok_or_else(|| RuntimeError::function_not_found(name))?
            .clone();
        
        // Check for @txn attribute
        let txn_attr = func.attributes.iter().find(|a| a.name == "txn");
        
        if let Some(attr) = txn_attr {
            self.execute_with_transaction(&func, args, attr)
        } else {
            self.execute_function_body(&func, args)
        }
    }
}
```

**Step 2: Parse Attribute Parameters**
```rust
fn parse_txn_attribute(attr: &Attribute) -> (IsolationLevel, Option<u64>) {
    let isolation = if let Some(Expression::Literal(Literal::String(iso))) = attr.parameters.get(0) {
        match iso.as_str() {
            "read_uncommitted" => IsolationLevel::ReadUncommitted,
            "read_committed" => IsolationLevel::ReadCommitted,
            "repeatable_read" => IsolationLevel::RepeatableRead,
            "serializable" => IsolationLevel::Serializable,
            _ => IsolationLevel::ReadCommitted, // Default
        }
    } else {
        IsolationLevel::ReadCommitted // Default
    };
    
    let timeout = if let Some(Expression::Literal(Literal::Number(n))) = attr.parameters.get(1) {
        Some(*n as u64)
    } else {
        None
    };
    
    (isolation, timeout)
}
```

**Step 3: Execute with Transaction Wrapper**
```rust
fn execute_with_transaction(
    &mut self, 
    func: &UserFunction, 
    args: Vec<Value>,
    txn_attr: &Attribute
) -> Result<Value, RuntimeError> {
    let (isolation, timeout) = parse_txn_attribute(txn_attr);
    
    // Begin transaction
    let tx_id = self.begin_transaction(isolation, timeout)?;
    
    // Execute function body
    let result = self.execute_function_body(func, args);
    
    // Commit or rollback
    match result {
        Ok(value) => {
            self.commit_transaction()?;
            Ok(value)
        }
        Err(error) => {
            let _ = self.rollback_transaction(); // Best effort
            Err(error)
        }
    }
}
```

---

## Testing

### Current Test (v1.0.5)

Test that `@txn` is **parsed correctly**:

```rust
#[test]
fn test_txn_attribute_parsing() {
    let source = r#"
        @txn
        fn transfer() -> Unit {}
    "#;
    
    let ast = parse_source(source).unwrap();
    // Verify attribute is present in AST
}
```

### Future Test (Phase 5)

Test that `@txn` **executes with transaction**:

```rust
#[test]
fn test_txn_attribute_execution() {
    let source = r#"
        @txn
        fn increment_counter() -> Unit {
            let count = database::tx_read("counter");
            database::tx_write("counter", count + 1);
        }
    "#;
    
    let mut runtime = Runtime::new();
    runtime.execute(source).unwrap();
    
    // Call the function
    runtime.call_function("increment_counter", vec![]).unwrap();
    
    // Verify transaction was committed
    runtime.begin_transaction(IsolationLevel::ReadCommitted, None).unwrap();
    let count = runtime.transaction_read("counter").unwrap();
    assert_eq!(count, Some(Value::Int(1)));
    runtime.commit_transaction().unwrap();
}
```

---

## Examples from Codebase

### Example 1: Simple Transfer (smart_contract.dal)

```dal
@trust("hybrid")
@chain("ethereum")
service TokenContract {
    @txn
    fn transfer(to: String, amount: Int) -> Result<Unit, Error> {
        // Automatically wrapped in transaction
        let sender_balance = database::tx_read("balance:" + msg.sender);
        let receiver_balance = database::tx_read("balance:" + to);
        
        if (sender_balance < amount) {
            return Err(Error::new("InsufficientBalance", "Not enough tokens"));
        }
        
        database::tx_write("balance:" + msg.sender, sender_balance - amount);
        database::tx_write("balance:" + to, receiver_balance + amount);
        
        return Ok(Unit);
    }
}
```

### Example 2: With Isolation Level (multi_chain_operations.dal)

```dal
@txn("serializable")
fn atomic_swap(token_a: String, token_b: String, amount: Int) -> Result<Unit, Error> {
    // High isolation for critical operations
    let balance_a = database::tx_read("balance:" + token_a);
    let balance_b = database::tx_read("balance:" + token_b);
    
    database::tx_write("balance:" + token_a, balance_a - amount);
    database::tx_write("balance:" + token_b, balance_b + amount);
    
    return Ok(Unit);
}
```

---

## Migration Path

### From Manual to `@txn`

**Before (v1.0.5 - Manual):**
```dal
fn transfer(from: String, to: String, amount: Int) -> Result<Unit, Error> {
    database::begin_transaction("serializable", 30000);
    
    // ... business logic ...
    
    if (error_condition) {
        database::rollback();
        return Err(error);
    }
    
    database::commit();
    return Ok(Unit);
}
```

**After (Phase 5 - Automatic):**
```dal
@txn("serializable", 30000)
fn transfer(from: String, to: String, amount: Int) -> Result<Unit, Error> {
    // ... business logic (same code, no begin/commit/rollback) ...
    
    if (error_condition) {
        return Err(error);  // Automatic rollback
    }
    
    return Ok(Unit);  // Automatic commit
}
```

---

## Benefits of `@txn`

1. **Less Boilerplate**: No manual begin/commit/rollback
2. **Safer**: Automatic rollback on errors (can't forget)
3. **Cleaner**: Business logic not mixed with transaction management
4. **Declarative**: Transaction semantics visible at function signature
5. **Composable**: Works with other attributes (`@secure`, `@limit`, etc.)

---

## Limitations

### What `@txn` Doesn't Do

- ❌ **Nested transactions**: If a `@txn` function calls another `@txn` function, behavior is TBD (likely reuse same transaction or error)
- ❌ **Partial commits**: Can't commit mid-function (use savepoints instead)
- ❌ **Conditional transactions**: Can't dynamically enable/disable (use manual API if needed)

### When to Use Manual API

Use `database::begin_transaction()` / `commit()` / `rollback()` when:
- You need **savepoints** for partial rollback
- You need **conditional transactions** (only some code paths need atomicity)
- You need **manual control** of commit timing
- You want to **span multiple function calls** in one transaction

---

## See Also

- **Manual Transaction API**: `docs/development/implementation/TRANSACTIONAL_SCOPE_DESIGN.md`
- **Implementation Plan**: `docs/development/implementation/TRANSACTION_IMPLEMENTATION_PLAN.md` (Phase 5)
- **Advanced Features**: `docs/development/implementation/TRANSACTION_ADVANCED_FEATURES_PLAN.md`
- **Examples**: `examples/smart_contract.dal`, `examples/multi_chain_operations.dal`

---

**Current Recommendation (v1.0.5):** Use the **manual transaction API** (`database::begin_transaction`, etc.) until Phase 5 implements automatic `@txn` wrapping. The attribute is recognized by the lexer/parser but not yet acted upon by the runtime.

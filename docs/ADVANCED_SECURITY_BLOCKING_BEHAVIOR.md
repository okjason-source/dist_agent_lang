# `@advanced_security` Blocking Behavior

## Yes, It Blocks Unprotected Execution Code

The `@advanced_security` attribute **DOES block** execution code that:
1. Contains MEV-related patterns (arbitrage, frontrun, sandwich, etc.)
2. **AND** is execution code (not monitoring)
3. **AND** lacks protection patterns

---

## Blocking Logic

### Decision Tree

```
MEV Pattern Detected?
├─ Yes → Is it monitoring code? (find_*, detect_*, analyze_*)
│   ├─ Yes → ✅ ALLOW (monitoring is OK)
│   └─ No → Has protection patterns? (commit-reveal, slippage, oracle)
│       ├─ Yes → ✅ ALLOW (already protected)
│       └─ No → ❌ BLOCK (unprotected execution)
└─ No → ✅ ALLOW
```

---

## Examples

### ✅ Example 1: Monitoring Code (NOT Blocked)

```dal
@advanced_security
service Service {
    fn find_arbitrage_opportunities() {
        // ✅ ALLOWED
        // Reason: "find_" prefix detected = monitoring code
        // Monitoring code is always allowed
    }
    
    fn detect_price_differences() {
        // ✅ ALLOWED
        // Reason: "detect_" prefix detected = monitoring code
    }
}
```

**Result**: ✅ Executes successfully

---

### ✅ Example 2: Protected Execution (NOT Blocked)

```dal
@advanced_security
service DeFiService {
    fn execute_swap(token_in: string, token_out: string, amount: float) {
        // ✅ ALLOWED
        // Reason: Protection patterns detected:
        
        // Protection 1: Commit-reveal
        let commitment_hash = crypto::hash(...);
        
        // Protection 2: Slippage protection
        let min_amount_out = amount * 0.99;  // 1% slippage limit
        
        // Protection 3: Oracle validation
        let oracle_price = self.get_oracle_price(...);
        
        // System detects these patterns and allows execution
    }
}
```

**Result**: ✅ Executes successfully (protection detected)

---

### ❌ Example 3: Unprotected Execution (BLOCKED)

```dal
@advanced_security
service VulnerableService {
    fn execute_swap(token_in: string, token_out: string, amount: float) {
        // ❌ BLOCKED
        // Reason: 
        // 1. Contains MEV pattern (swap = potential arbitrage)
        // 2. Is execution code (not monitoring)
        // 3. No protection patterns detected
        
        // Unprotected swap code
        chain::swap(token_in, token_out, amount);
    }
}
```

**Error Message**:
```
Runtime error: Potential MEV attack detected: arbitrage. 
Consider adding protection patterns (commit-reveal, slippage checks, oracle validation)
```

**Result**: ❌ Execution blocked

---

### ❌ Example 4: Explicit MEV Pattern (BLOCKED)

```dal
@advanced_security
service Service {
    fn arbitrage_trade(token_a: string, token_b: string) {
        // ❌ BLOCKED
        // Reason:
        // 1. Contains "arbitrage" keyword
        // 2. Is execution code (not monitoring - no "find_" prefix)
        // 3. No protection patterns detected
        
        // Unprotected arbitrage code
    }
}
```

**Error Message**:
```
Runtime error: Potential MEV attack detected: arbitrage. 
Consider adding protection patterns (commit-reveal, slippage checks, oracle validation)
```

**Result**: ❌ Execution blocked

---

## How to Fix Blocked Code

### Option 1: Add Protection Patterns

```dal
@advanced_security
service Service {
    fn execute_swap(...) {
        // Add protection patterns:
        
        // 1. Commit-reveal
        let commitment_hash = crypto::hash(...);
        
        // 2. Slippage protection
        let min_amount_out = ...;
        if (actual_out < min_amount_out) {
            throw "Slippage too high";
        }
        
        // 3. Oracle validation
        let oracle_price = self.get_oracle_price(...);
        
        // Now protected - will execute ✅
    }
}
```

### Option 2: Rename to Monitoring Function

```dal
@advanced_security
service Service {
    // Change from execution to monitoring
    fn find_swap_opportunities() {  // ✅ Allowed (monitoring)
        // Monitoring code - no protection needed
    }
    
    // Separate protected execution
    fn execute_protected_swap() {  // ✅ Allowed (has protection)
        // Has protection patterns
    }
}
```

### Option 3: Remove `@advanced_security`

```dal
// No @advanced_security = no MEV detection
service Service {
    fn execute_swap(...) {
        // No MEV detection runs
        // You're responsible for your own protection
    }
}
```

---

## Protection Pattern Detection

The system recognizes these patterns as "protection":

### Commit-Reveal
- `commit_reveal`, `commit-reveal`
- `commitment_hash`, `commitment`
- `reveal_swap`, `commit_swap`

### Slippage Protection
- `slippage`, `min_amount_out`, `max_slippage`
- `slippage_protection`, `slippage_check`

### Oracle Validation
- `oracle_price`, `get_oracle_price`
- `price_oracle`, `oracle_validation`

### Fair Batching
- `fair_batch`, `fair_ordering`
- `batch_pool`, `shuffle`

### Time Delays
- `time_delay`, `delayed_execution`
- `execute_delayed`

---

## Summary

| Code Type | Has Protection? | Result |
|-----------|----------------|--------|
| Monitoring (`find_*`, `detect_*`) | N/A | ✅ **ALLOWED** |
| Execution | ✅ Yes | ✅ **ALLOWED** |
| Execution | ❌ No | ❌ **BLOCKED** |

**Answer**: Yes, it **DOES block** unprotected execution code that contains MEV-related patterns.

**Purpose**: Prevent developers from accidentally deploying vulnerable code without MEV protection.

---

**Last Updated**: February 2026

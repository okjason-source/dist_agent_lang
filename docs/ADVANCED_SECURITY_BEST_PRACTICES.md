# `@advanced_security` Best Practices

## Overview

The `@advanced_security` attribute provides **intelligent MEV detection** that:
- ✅ **Allows monitoring code** (find_*, detect_*, analyze_*)
- ✅ **Detects protection patterns** (commit-reveal, slippage, etc.)
- ✅ **Only blocks unprotected execution code**
- ✅ **Provides helpful suggestions**

---

## How It Works

### Smart Detection

The system distinguishes between:

1. **Monitoring Code** ✅ Allowed
   - Functions: `find_*`, `detect_*`, `analyze_*`, `monitor_*`, `get_*`
   - Read-only operations
   - Analytics and reporting

2. **Protected Execution** ✅ Allowed
   - Has protection patterns: commit-reveal, slippage checks, oracle validation
   - Already implements MEV protection

3. **Unprotected Execution** ⚠️ Blocked (if @advanced_security enabled)
   - Execution code without protection patterns
   - High-risk transactions

---

## Usage Examples

### ✅ Example 1: Monitoring Code (Always Allowed)

```dal
@advanced_security
service AnalyticsService {
    fn find_price_differences() -> list<map<string, any>> {
        // ✅ Allowed - monitoring function
        // System detects "find_" prefix = monitoring code
        return self.analyze_prices();
    }
    
    fn detect_arbitrage_opportunities() -> list<map<string, any>> {
        // ✅ Allowed - monitoring function
        // System detects "detect_" prefix = monitoring code
        return [];
    }
    
    fn monitor_liquidity_health() -> map<string, any> {
        // ✅ Allowed - monitoring function
        return {};
    }
}
```

### ✅ Example 2: Protected Execution (Allowed)

```dal
@advanced_security
service ProtectedDeFiService {
    fn execute_protected_swap(...) {
        // ✅ Allowed - protection patterns detected:
        // - commit-reveal pattern
        // - slippage protection
        // - oracle price validation
        
        let commitment_hash = crypto::hash(...);  // Protection detected
        let min_amount_out = ...;  // Slippage protection detected
        let oracle_price = self.get_oracle_price(...);  // Oracle detected
    }
}
```

### ⚠️ Example 3: Unprotected Execution (Blocked)

```dal
@advanced_security
service VulnerableService {
    fn execute_swap(token_in: string, token_out: string, amount: float) {
        // ❌ Blocked: "MEV pattern detected. Consider adding protection patterns"
        // No protection patterns found
        
        // Unprotected swap code
    }
}
```

**Fix**: Add protection patterns (see manual MEV protection guide)

---

## Best Practices

### 1. **Use Descriptive Function Names**

**✅ Good** (Monitoring):
```dal
fn find_price_differences()  // ✅ Detected as monitoring
fn detect_opportunities()   // ✅ Detected as monitoring
fn analyze_market()         // ✅ Detected as monitoring
```

**⚠️ Avoid** (Ambiguous):
```dal
fn arbitrage()              // ❌ Looks like execution
fn trade()                  // ❌ Looks like execution
```

### 2. **Add Protection Patterns**

**For execution code, always add protection:**

```dal
@advanced_security
service DeFiService {
    fn execute_swap(...) {
        // ✅ Protection patterns:
        // 1. Commit-reveal (for large swaps)
        // 2. Slippage protection
        // 3. Oracle price validation
        
        let commitment_hash = crypto::hash(...);
        let min_amount_out = self.calculate_min_output(...);
        let oracle_price = self.get_oracle_price(...);
    }
}
```

### 3. **Separate Monitoring from Execution**

**✅ Good Pattern**:
```dal
@advanced_security
service Service {
    // Monitoring (allowed)
    fn find_opportunities() -> list<any> {
        return [];
    }
    
    // Execution (needs protection)
    fn execute_protected_swap(...) {
        // Has protection patterns
    }
}
```

---

## When to Use `@advanced_security`

### ✅ **Use When:**
- Building DeFi/swap services
- Handling high-value transactions
- Need MEV protection guidance
- Want security monitoring

### ❌ **Don't Use When:**
- Pure analytics/monitoring services (unnecessary)
- Services with manual protection already (redundant)
- Public read-only APIs (not needed)

---

## Protection Pattern Recognition

The system automatically detects these protection patterns:

### ✅ **Commit-Reveal Patterns**
- `commit_reveal`, `commit-reveal`
- `commitment_hash`, `commitment`
- `reveal_swap`, `commit_swap`

### ✅ **Slippage Protection**
- `slippage`, `min_amount_out`, `max_slippage`
- `slippage_protection`, `slippage_check`

### ✅ **Oracle Validation**
- `oracle_price`, `get_oracle_price`
- `price_oracle`, `oracle_validation`

### ✅ **Fair Batching**
- `fair_batch`, `fair_ordering`
- `batch_pool`, `shuffle`

### ✅ **Time Delays**
- `time_delay`, `delayed_execution`
- `execute_delayed`

---

## Configuration

### Default Behavior

```dal
@advanced_security  // Monitor mode (warn only, allow monitoring)
```

**Behavior**:
- ✅ Allows monitoring code
- ✅ Allows protected execution
- ⚠️ Blocks unprotected execution
- 💡 Provides suggestions

### Disable for Specific Services

```dal
// No @advanced_security = no MEV detection
service AnalyticsService {
    fn find_opportunities() {
        // No MEV detection runs
    }
}
```

---

## Migration Guide

### From Old (Blocking) to New (Smart)

**Before** (Blocked everything):
```dal
@advanced_security
service Service {
    fn find_arbitrage() {  // ❌ Blocked
    }
}
```

**After** (Smart detection):
```dal
@advanced_security
service Service {
    fn find_price_differences() {  // ✅ Allowed (monitoring)
    }
    
    fn execute_protected_swap() {  // ✅ Allowed (has protection)
    }
    
    fn execute_unprotected_swap() {  // ⚠️ Blocked (needs protection)
    }
}
```

---

## Summary

**Best Design**: **Monitor, Warn, Guide - Don't Block Legitimate Code**

1. ✅ **Monitoring code** → Always allowed
2. ✅ **Protected execution** → Allowed (protection detected)
3. ⚠️ **Unprotected execution** → Blocked with helpful suggestions
4. 💡 **Guidance** → Suggests protection patterns

**Result**: Developers can write monitoring/analytics code freely, while still getting protection for actual execution code.

---

**Last Updated**: February 2026
**Status**: Implemented

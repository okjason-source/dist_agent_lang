# `@advanced_security` Design: Monitoring vs Blocking

## Current Problem

The current MEV detection implementation:
- ❌ **Blocks execution** on keyword detection (too aggressive)
- ❌ **Scans entire program AST** (not just transaction data)
- ❌ **Doesn't distinguish** between monitoring code vs actual execution
- ❌ **False positives** for legitimate monitoring/analytics code

## Best Design for Developers

### Principle: **Monitor, Warn, Guide - Don't Block Legitimate Code**

Developers need:
1. ✅ **Monitoring** - Detect potential issues
2. ✅ **Warnings** - Alert but don't block
3. ✅ **Guidance** - Suggest protection patterns
4. ✅ **Control** - Configure sensitivity levels
5. ✅ **Context-Aware** - Distinguish monitoring vs execution

---

## Proposed Design: Three-Tier System

### Tier 1: **Monitoring Mode** (Default)
**Purpose**: Detect and warn, but allow execution

```dal
@advanced_security("monitor")  // or just @advanced_security
service MyService {
    fn find_price_differences() {
        // ✅ Allowed - monitoring code
        // ⚠️ Warning logged: "MEV-related keyword detected in monitoring function"
    }
}
```

**Behavior**:
- ✅ Scans code for MEV-related patterns
- ✅ Logs warnings to audit log
- ✅ Provides suggestions for protection
- ✅ **Does NOT block execution**

### Tier 2: **Advisory Mode**
**Purpose**: Warn and suggest protection patterns

```dal
@advanced_security("advisory")
service DeFiService {
    fn swap(token_in: string, token_out: string, amount: float) {
        // ⚠️ Warning: "Swap function detected - consider adding slippage protection"
        // Suggestion: Use commit-reveal or slippage checks
        // ✅ Still executes
    }
}
```

**Behavior**:
- ✅ Detects potentially vulnerable patterns
- ✅ Suggests protection mechanisms
- ✅ Logs recommendations
- ✅ **Does NOT block execution**

### Tier 3: **Strict Mode** (Opt-in)
**Purpose**: Block actual suspicious transaction patterns

```dal
@advanced_security("strict")
service CriticalDeFiService {
    fn execute_swap(...) {
        // Only blocks if:
        // 1. Actual transaction execution (not monitoring)
        // 2. No protection mechanisms detected
        // 3. High-value transaction without safeguards
    }
}
```

**Behavior**:
- ✅ Only blocks **actual transaction execution**
- ✅ Checks for protection mechanisms (commit-reveal, slippage, etc.)
- ✅ Context-aware (monitoring vs execution)
- ✅ Blocks only high-risk patterns

---

## Implementation Strategy

### 1. **Context-Aware Detection**

**Distinguish between:**
- **Monitoring Code**: Functions that analyze/detect but don't execute
  - `find_*`, `detect_*`, `analyze_*`, `monitor_*`
  - Read-only operations
  - Returns data without state changes

- **Execution Code**: Functions that actually execute transactions
  - `execute_*`, `swap`, `transfer`, `trade`
  - State-changing operations
  - External calls

**Example**:
```dal
@advanced_security("monitor")
service Service {
    // ✅ Monitoring - only warns
    fn find_price_differences() {
        // Warning logged, but executes
    }
    
    // ⚠️ Execution - suggests protection
    fn execute_swap(...) {
        // Warning: "Consider adding slippage protection"
        // Suggestion: Use commit-reveal pattern
        // Still executes
    }
}
```

### 2. **Protection Pattern Detection**

**Check for existing protection** before warning/blocking:

```dal
@advanced_security("advisory")
service Service {
    fn swap(...) {
        // ✅ Detects: commit-reveal pattern present
        // ✅ Detects: slippage protection present
        // ✅ Detects: oracle price validation present
        // Result: No warning (already protected)
    }
    
    fn unprotected_swap(...) {
        // ⚠️ Warning: "No MEV protection detected"
        // Suggestion: Add commit-reveal or slippage protection
        // Still executes (advisory mode)
    }
}
```

### 3. **Configurable Sensitivity**

```dal
@advanced_security("monitor", sensitivity: "low")    // Only obvious patterns
@advanced_security("monitor", sensitivity: "medium")  // Default
@advanced_security("monitor", sensitivity: "high")    // All patterns
```

---

## Recommended Implementation

### Phase 1: Smart Monitoring (Immediate)

**Change MEV detection to:**
1. **Warn instead of block** for monitoring code
2. **Check function names** - `find_*`, `detect_*` = monitoring
3. **Check for protection patterns** - if present, no warning
4. **Provide helpful suggestions**

**Code Changes**:
```rust
pub fn analyze_transaction(&mut self, transaction_data: &str, mode: SecurityMode) -> Result<Vec<SecurityWarning>, RuntimeError> {
    let mut warnings = Vec::new();
    
    // Check for MEV-related keywords
    let suspicious_patterns = ["arbitrage", "frontrun", "sandwich", ...];
    
    for pattern in &suspicious_patterns {
        if transaction_data.contains(pattern) {
            // Check if it's monitoring code
            if self.is_monitoring_code(transaction_data) {
                warnings.push(SecurityWarning::Info(format!(
                    "MEV-related keyword '{}' detected in monitoring code - this is normal"
                )));
            } else {
                warnings.push(SecurityWarning::Advisory(format!(
                    "MEV-related pattern '{}' detected - consider adding protection: {}",
                    pattern,
                    self.suggest_protection(pattern)
                )));
            }
        }
    }
    
    // Only block in strict mode AND if no protection detected
    if mode == SecurityMode::Strict && !self.has_protection_patterns(transaction_data) {
        return Err(RuntimeError::General("MEV protection required in strict mode"));
    }
    
    Ok(warnings)
}
```

### Phase 2: Context-Aware Detection

**Detect function context:**
- Monitoring functions: `find_*`, `detect_*`, `analyze_*`, `monitor_*`
- Execution functions: `execute_*`, `swap`, `transfer`, `trade`

**Example**:
```rust
fn is_monitoring_code(code: &str) -> bool {
    let monitoring_patterns = [
        "fn find_", "fn detect_", "fn analyze_", "fn monitor_",
        "fn get_", "fn check_", "fn query_"
    ];
    
    monitoring_patterns.iter().any(|pattern| code.contains(pattern))
}

fn is_execution_code(code: &str) -> bool {
    let execution_patterns = [
        "fn execute_", "fn swap", "fn transfer", "fn trade",
        "fn buy", "fn sell", "chain::execute"
    ];
    
    execution_patterns.iter().any(|pattern| code.contains(pattern))
}
```

### Phase 3: Protection Pattern Detection

**Check for existing protection:**
```rust
fn has_protection_patterns(code: &str) -> bool {
    let protection_patterns = [
        "commit_reveal", "commit-reveal", "commitment_hash",
        "slippage", "min_amount_out", "oracle_price",
        "fair_batch", "time_delay"
    ];
    
    protection_patterns.iter().any(|pattern| code.contains(pattern))
}
```

---

## Usage Examples

### Example 1: Monitoring Service (No Blocking)

```dal
@advanced_security("monitor")
service AnalyticsService {
    fn find_price_differences() -> list<map<string, any>> {
        // ✅ Executes normally
        // ⚠️ Warning logged: "MEV keyword detected in monitoring function - OK"
        return self.analyze_prices();
    }
    
    fn detect_arbitrage_opportunities() -> list<map<string, any>> {
        // ✅ Executes normally
        // ⚠️ Info logged: "Monitoring function - no action needed"
        return [];
    }
}
```

### Example 2: DeFi Service with Protection (Advisory Mode)

```dal
@advanced_security("advisory")
service DeFiService {
    fn execute_swap(token_in: string, token_out: string, amount: float) {
        // Has protection patterns:
        // ✅ commit-reveal pattern detected
        // ✅ slippage protection detected
        // ✅ oracle price validation detected
        // Result: No warnings (already protected)
        
        let commitment_hash = crypto::hash(...);
        let min_amount_out = self.calculate_min_output(...);
        let oracle_price = self.get_oracle_price(...);
        // ... protected swap logic
    }
}
```

### Example 3: Unprotected Swap (Advisory Mode)

```dal
@advanced_security("advisory")
service VulnerableService {
    fn swap(token_in: string, token_out: string, amount: float) {
        // ⚠️ Warning: "Swap function detected without MEV protection"
        // 💡 Suggestion: "Consider adding:
        //    - Commit-reveal pattern for large swaps
        //    - Slippage protection (min_amount_out)
        //    - Oracle price validation"
        // ✅ Still executes (advisory mode)
        
        // ... unprotected swap
    }
}
```

### Example 4: Strict Mode (Blocks Unprotected)

```dal
@advanced_security("strict")
service CriticalService {
    fn execute_swap(...) {
        // ❌ Blocked: "MEV protection required in strict mode"
        // Must add protection patterns first
    }
    
    fn execute_protected_swap(...) {
        // ✅ Allowed: Protection patterns detected
        // - commit-reveal ✓
        // - slippage protection ✓
    }
}
```

---

## Configuration Options

### Attribute Syntax

```dal
// Default: Monitor mode (warn only)
@advanced_security

// Explicit monitor mode
@advanced_security("monitor")

// Advisory mode (suggestions)
@advanced_security("advisory")

// Strict mode (block unprotected)
@advanced_security("strict")

// With sensitivity
@advanced_security("monitor", sensitivity: "low")
@advanced_security("monitor", sensitivity: "high")

// With whitelist
@advanced_security("monitor", whitelist: ["find_price_differences"])
```

---

## Benefits for Developers

### ✅ **No False Positives**
- Monitoring code doesn't trigger blocks
- Context-aware detection
- Protection pattern recognition

### ✅ **Helpful Guidance**
- Suggests protection patterns
- Points to documentation
- Provides code examples

### ✅ **Flexible Control**
- Choose monitoring level
- Configure sensitivity
- Whitelist functions

### ✅ **Production Ready**
- Strict mode for critical services
- Advisory mode for development
- Monitor mode for analytics

---

## Migration Path

### Current → Proposed

**Step 1**: Change default to "monitor" mode (warn only)
**Step 2**: Add context-aware detection (monitoring vs execution)
**Step 3**: Add protection pattern detection
**Step 4**: Add advisory and strict modes
**Step 5**: Update documentation

---

## Recommended Default Behavior

**For `@advanced_security` (no parameters)**:
- ✅ **Monitor mode** (warn only)
- ✅ **Context-aware** (distinguish monitoring vs execution)
- ✅ **Protection-aware** (check for existing protection)
- ✅ **Helpful suggestions** (guide developers)

**Result**: Developers can write monitoring/analytics code without false positives, while still getting helpful guidance for actual execution code.

---

## Conclusion

**Best Design**: **Monitor, Warn, Guide - Don't Block Legitimate Code**

1. **Default**: Monitor mode (warn only)
2. **Context-Aware**: Distinguish monitoring vs execution
3. **Protection-Aware**: Check for existing protection patterns
4. **Helpful**: Provide suggestions and guidance
5. **Flexible**: Allow configuration and strict mode

This approach:
- ✅ Allows legitimate monitoring code
- ✅ Provides helpful guidance
- ✅ Prevents false positives
- ✅ Still protects when needed (strict mode)
- ✅ Best developer experience

---

**Recommendation**: Implement smart monitoring as default, with optional strict mode for critical services.

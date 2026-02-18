# Function-Level `@secure` and MEV Protection Analysis

## Executive Summary

This document analyzes:
1. **Implementation at the function-level `@secure`?** (Currently only service-level is enforced)
2. **How to protect against MEV attacks** using existing infrastructure and new mechanisms

---

## Part 1: Function-Level `@secure` Implementation

### Current State

**What Works:**
- ✅ Function attributes are **parsed** (`src/parser/parser.rs:186-227`)
- ✅ Attributes stored in `FunctionStatement.attributes` (`src/parser/ast.rs:60-68`)
- ✅ Documentation mentions function-level `@secure` (`docs/attributes.md:140-149`)
- ✅ **Runtime now checks function-level `@secure` attributes** (`src/runtime/engine.rs:4457-4470`)
- ✅ **Parser validates function-level attribute mutual exclusivity** (`src/parser/parser.rs:2356-2385`)
- ✅ **Function-level attributes override service-level** with proper precedence rules

**Implementation Status:**
- ✅ **IMPLEMENTED** - Function-level `@secure` is now fully supported
- ✅ Runtime enforcement checks both service and function attributes
- ✅ Parser validates that `@secure` and `@public` are mutually exclusive at function level

### How it works

#### ✅ **Implementation**

1. **Granular Security Control**
   ```dal
   service MixedService {
       fn public_read() -> data {
           // No @secure - public access
       }
       
       @secure
       fn admin_write(data: any) {
           // Only this method requires auth
       }
   }
   ```

2. **Matches Documentation**
   - Documentation already describes function-level `@secure`
   - Users expect it to work based on docs

3. **Flexibility**
   - Allows mixed public/secure methods in same service
   - Better than requiring separate services

4. **Common Pattern**
   - Many frameworks support method-level security
   - Aligns with developer expectations

### **Clear Rules**

**Implementation Priority**: 🟡 **MEDIUM** (Nice-to-have, not critical)

**Precedence Rules:**
1. **Function-level `@secure` overrides service-level**
2. **Function-level `@public` overrides service-level `@secure`**
3. **If neither function nor service has `@secure`, no protection**

**Example:**
```dal
@secure  // Service-level: all methods secure by default
service MixedService {
    fn secure_method() {
        // ✅ Protected (inherits from service)
    }
    
    @secure
    fn explicitly_secure() {
        // ✅ Protected (explicit function-level)
    }
    
    @public
    fn public_method() {
        // ✅ Public (function-level overrides service)
    }
}
```

---

## Part 2: MEV Protection Integration

### Current MEV Protection Infrastructure

**Existing Components** (`src/runtime/advanced_security.rs`):

1. **MEVProtectionManager** ✅
   - Commit-reveal scheme
   - Time-delayed transactions
   - Fair batch ordering
   - Transaction analysis

2. **Protection Types**:
   - `CommitReveal`: Hide transaction details until reveal phase
   - `TimeDelay`: Delay execution to prevent front-running
   - `FairBatch`: Random/fair ordering of transactions

3. **Current Integration**:
   - MEV analysis called in `execute_program()` (`src/runtime/engine.rs:258`)
   - **NOT integrated with `@secure`**

### MEV Attack Vectors

#### 1. **Front-Running**
**Attack**: Attacker sees pending transaction, submits higher gas price to execute first
**Example**: 
- User submits swap: ETH → USDC at price X
- Attacker sees it, front-runs with higher gas
- Attacker buys ETH, price increases
- User's swap executes at worse price

#### 2. **Back-Running**
**Attack**: Attacker sees transaction, submits transaction immediately after
**Example**:
- User submits large trade
- Attacker back-runs to profit from price impact

#### 3. **Sandwich Attack**
**Attack**: Front-run + back-run combination
**Example**:
- User wants to swap 100 ETH → USDC
- Attacker front-runs: buys ETH (pushes price up)
- User's swap executes (at higher price)
- Attacker back-runs: sells ETH (pushes price down)
- Attacker profits from spread

#### 4. **Time-Based Manipulation**
**Attack**: Exploiting predictable execution times
**Example**:
- Service executes at fixed intervals
- Attacker times transactions to exploit

### How to Protect Against MEV

#### Strategy 1: **Commit-Reveal Scheme**

**How It Works**:
1. **Commit Phase**: User submits hash of transaction (hidden)
2. **Reveal Phase**: User reveals actual transaction data
3. **Execution**: Transaction executes after reveal

**Implementation**:
```dal
@secure
@mev_protection("commit_reveal")
service DeFiService {
    fn swap(from: string, to: string, amount: int) {
        // Transaction details hidden until reveal
        // Prevents front-running
    }
}
```

**Pros**:
- ✅ Strong protection against front-running
- ✅ Hides transaction intent

**Cons**:
- ❌ Two-phase process (slower)
- ❌ User must reveal or forfeit

#### Strategy 2: **Time-Delayed Execution**

**How It Works**:
1. Transaction submitted to pool
2. Waits for time window (e.g., 1 block)
3. Executes in batch with other transactions

**Implementation**:
```dal
@secure
@mev_protection("time_delay", delay_blocks: 2)
service DeFiService {
    fn swap(from: string, to: string, amount: int) {
        // Executes after 2 blocks
        // Prevents immediate front-running
    }
}
```

**Pros**:
- ✅ Simple to implement
- ✅ Reduces front-running window

**Cons**:
- ❌ Still vulnerable if attacker can predict execution
- ❌ Slower execution

#### Strategy 3: **Fair Batch Ordering**

**How It Works**:
1. Collect transactions in time window
2. Randomly shuffle order (using VRF or block hash)
3. Execute in shuffled order

**Implementation**:
```dal
@secure
@mev_protection("fair_batch", batch_size: 10)
service DeFiService {
    fn swap(from: string, to: string, amount: int) {
        // Executes in random order within batch
        // Prevents predictable ordering
    }
}
```

**Pros**:
- ✅ Fair ordering prevents manipulation
- ✅ Can use VRF for true randomness

**Cons**:
- ❌ Requires batching (delayed execution)
- ❌ Randomness source must be secure

#### Strategy 4: **Private Mempool / Encrypted Transactions**

**How It Works**:
1. Transactions encrypted before submission
2. Only decrypted at execution time
3. Prevents mempool observation

**Implementation**:
```dal
@secure
@mev_protection("private_mempool")
service DeFiService {
    fn swap(from: string, to: string, amount: int) {
        // Transaction encrypted until execution
        // Prevents mempool front-running
    }
}
```

**Pros**:
- ✅ Strong protection
- ✅ No delay needed

**Cons**:
- ❌ Requires encryption infrastructure
- ❌ More complex

### Recommended Approach: **Multi-Layer Protection**

**Combine Strategies**:
```dal
@secure
@mev_protection("commit_reveal")  // Primary: hide intent
@mev_protection("fair_batch", batch_size: 20)  // Secondary: fair ordering
service DeFiService {
    fn swap(from: string, to: string, amount: int) {
        // Protected by:
        // 1. Commit-reveal (hides details)
        // 2. Fair batching (random order)
        // 3. @secure (authentication)
    }
}
```

---

## Part 3: Implementation Plan

### Phase 1: Function-Level `@secure` (Medium Priority)

**Steps**:
1. **Modify `execute_service_method()`** to check function attributes
2. **Implement precedence rules**:
   - Function-level overrides service-level
   - `@public` on function overrides `@secure` on service
3. **Update documentation** with precedence rules
4. **Add tests** for function-level `@secure`

**Code Changes**:
```rust
// In src/runtime/engine.rs:execute_service_method()
let has_secure_attr = {
    // Check function-level first (overrides service)
    let func_has_secure = method.attributes.iter().any(|attr| attr.name == "secure");
    let func_has_public = method.attributes.iter().any(|attr| attr.name == "public");
    
    if func_has_public {
        false  // Function-level @public overrides
    } else if func_has_secure {
        true   // Function-level @secure
    } else {
        // Fall back to service-level
        service_attrs.iter().any(|attr| attr == "@secure")
    }
};
```

**Estimated Effort**: 2-3 days

### Phase 2: MEV Protection Integration (High Priority for DeFi)

**Steps**:
1. **Add `@mev_protection` attribute** to parser
2. **Integrate with `@secure`**:
   - When `@secure` + `@mev_protection` present, use MEV protection
3. **Implement attribute parsing**:
   ```dal
   @mev_protection("commit_reveal")
   @mev_protection("fair_batch", batch_size: 20)
   ```
4. **Wire up to MEVProtectionManager**:
   - Check for `@mev_protection` in `execute_service_method()`
   - Route to appropriate protection mechanism
5. **Add tests** for MEV protection

**Code Changes**:
```rust
// In src/runtime/engine.rs:execute_service_method()
let mev_protection = method.attributes.iter()
    .find(|attr| attr.name == "mev_protection")
    .and_then(|attr| {
        // Parse protection type and parameters
        parse_mev_protection(attr)
    });

if let Some(protection) = mev_protection {
    // Submit to MEV protection manager
    self.advanced_security.submit_protected_transaction(
        caller.clone(),
        method_name.to_string(),
        args.to_vec(),
        protection,
    )?;
}
```

**Estimated Effort**: 1-2 weeks

### Phase 3: Enhanced MEV Protection (Future)

**Additional Features**:
1. **VRF Integration**: True randomness for fair ordering
2. **Private Mempool**: Encrypted transaction submission
3. **MEV Detection**: Heuristic-based attack detection
4. **Gas Price Limits**: Prevent gas price manipulation

**Estimated Effort**: 2-3 weeks

---

## Part 4: Usage Examples

### Example 1: Function-Level `@secure`

```dal
// Service with mixed security
service UserService {
    // Public method (no @secure)
    fn get_public_profile(user_id: string) -> map<string, any> {
        return {"name": "Public", "id": user_id};
    }
    
    // Secure method (function-level @secure)
    @secure
    fn update_email(user_id: string, email: string) {
        // Requires authentication
    }
    
    // Another secure method
    @secure
    fn delete_account(user_id: string) {
        // Requires authentication
    }
}
```

### Example 2: MEV Protection for DeFi

```dal
@secure
@mev_protection("commit_reveal")
@trust("decentralized")
@chain("ethereum")
service DEXService {
    fn swap(
        token_in: string,
        token_out: string,
        amount_in: int,
        min_amount_out: int
    ) -> Result<int, string> {
        // Protected from front-running via commit-reveal
        // Transaction details hidden until reveal phase
    }
}
```

### Example 3: Combined Protection

```dal
@secure  // Service-level: default secure
@trust("decentralized")
@chain("ethereum")
service AdvancedDeFiService {
    // Public read (function-level @public overrides)
    @public
    fn get_price(token: string) -> int {
        return get_current_price(token);
    }
    
    // Secure swap with MEV protection
    @secure  // Explicit (redundant but clear)
    @mev_protection("fair_batch", batch_size: 10)
    fn swap(token_in: string, token_out: string, amount: int) {
        // Protected from:
        // 1. Unauthenticated access (@secure)
        // 2. Front-running (fair batch ordering)
    }
    
    // Admin function with time delay
    @secure
    @mev_protection("time_delay", delay_blocks: 5)
    fn set_fee_rate(rate: int) {
        // Protected from:
        // 1. Unauthenticated access
        // 2. Time-based manipulation (5 block delay)
    }
}
```

---

## Part 5: Recommendations Summary

### Function-Level `@secure`

**Recommendation**: ✅ **YES, implement with clear precedence rules**

**Priority**: 🟡 **MEDIUM**
- Not critical, but improves developer experience
- Matches documentation expectations
- Provides flexibility

**Implementation**:
- Function-level overrides service-level
- `@public` on function overrides `@secure` on service
- Clear error messages for conflicts

### MEV Protection

**Recommendation**: ✅ **YES, integrate with `@secure`**

**Priority**: 🔴 **HIGH** (for DeFi applications)

**Implementation**:
- Add `@mev_protection` attribute
- Integrate with existing `MEVProtectionManager`
- Support multiple protection types:
  - `commit_reveal`
  - `time_delay`
  - `fair_batch`
  - `private_mempool` (future)

**Best Practices**:
- Use commit-reveal for high-value transactions
- Use fair batching for DEX swaps
- Use time delays for admin operations
- Combine multiple strategies for critical operations

---

## Part 6: Security Considerations

### Function-Level `@secure` Security

**Risks**:
- ⚠️ Precedence confusion could lead to security holes
- ⚠️ Developers might forget to add `@secure` to functions

**Mitigations**:
- Clear documentation and examples
- Linter warnings for missing `@secure` on write operations
- Default to secure if service has `@secure`

### MEV Protection Security

**Risks**:
- ⚠️ Commit-reveal: Users might forget to reveal
- ⚠️ Time delays: Still vulnerable to predictable execution
- ⚠️ Fair batching: Randomness source must be secure

**Mitigations**:
- Automatic reveal after deadline
- Use VRF for true randomness
- Monitor for MEV attack patterns
- Rate limiting on high-value transactions

---

## Conclusion

1. **Function-Level `@secure`**: Implement with clear precedence rules (Medium priority)
2. **MEV Protection**: Integrate with `@secure` using existing infrastructure (High priority for DeFi)
3. **Combined Approach**: Use both for comprehensive security

**Next Steps**:
1. Review and approve implementation plan
2. Prioritize based on use cases (DeFi = high priority for MEV)
3. Implement function-level `@secure` first (simpler)
4. Then implement MEV protection integration

---

**Last Updated**: February 2026
**Status**: Proposal for Review

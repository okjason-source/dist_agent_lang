# Complete Scope of `@secure` Attribute

## Overview

This document provides a comprehensive analysis of the entire scope of the `@secure` attribute in DAL, including what it protects, what it doesn't protect, and how it interacts with the runtime system.

---

## 1. Application Scope

### Where `@secure` Can Be Applied

**Service-Level Only** (Primary Usage):
```dal
@secure
service MyService {
    fn method1() { }
    fn method2() { }
    // All methods inherit @secure protection
}
```

**Current Implementation**: `@secure` is checked at the **service level** in `execute_service_method()` (`src/runtime/engine.rs:4447`). The attribute is read from the service's attributes list.

**Function-Level** (Documented but Not Implemented):
- Documentation mentions `@secure` can be applied to functions (`docs/attributes.md:140-149`)
- **Current runtime implementation does NOT check function-level `@secure` attributes**
- Only service-level `@secure` is enforced

---

## 2. Protection Scope

### What `@secure` Protects

#### 2.1 **All Methods in the Service**

When `@secure` is applied to a service, **ALL methods** in that service are protected:

```dal
@secure
service SecureService {
    fn public_method() {
        // ✅ Protected by @secure
    }
    
    fn private_helper() {
        // ✅ Also protected by @secure
    }
    
    fn admin_operation() {
        // ✅ Also protected by @secure
    }
}
```

**Implementation**: The check happens in `execute_service_method()` before any method body executes (`src/runtime/engine.rs:4442-4455`).

#### 2.2 **Per-Method Execution**

Each method call gets its own:
- **Reentrancy token**: Prevents re-entry into the same `instance_id::method_name` combination
- **Authentication check**: Verifies `current_caller` is set
- **Audit log entry**: Records access attempt

#### 2.3 **Reentrancy Protection Scope**

**Protected Against**:
- ✅ Re-entry into the **same method** on the **same service instance**
- ✅ Format: `{instance_id}::{method_name}` (e.g., `"service_123::transfer"`)

**Example**:
```dal
@secure
service BankService {
    fn transfer(from: string, to: string, amount: int) {
        // If transfer() calls transfer() recursively → ❌ ReentrancyDetected
        // If transfer() calls withdraw() → ✅ Allowed (different method)
    }
    
    fn withdraw(account: string, amount: int) {
        // Different method, can be called even if transfer() is active
    }
}
```

**Implementation**: `src/runtime/reentrancy.rs:23-53`
- Call key format: `"{instance_id}::{method_name}"`
- Same instance + same method = reentrancy detected
- Different methods on same instance = allowed
- Same method on different instances = allowed

#### 2.4 **Authentication Scope**

**Protected Against**:
- ✅ Unauthenticated access (no `current_caller` set)
- ✅ Null/default caller addresses (`0x0000...`)

**Implementation**: `src/runtime/engine.rs:4485-4508`
```rust
let is_authenticated = self.current_caller.as_ref()
    .map(|caller| {
        caller != "0x0000000000000000000000000000000000000000" && !caller.is_empty()
    })
    .unwrap_or(false);
```

#### 2.5 **Audit Logging Scope**

**All Events Logged**:
1. ✅ **Successful authenticated access**: `secure_service_access`
   - Service ID, method name, caller address, result="allowed"

2. ✅ **Unauthorized access attempts**: `secure_service_access_denied`
   - Service ID, method name, caller (or "unauthenticated"), result="denied"

3. ✅ **Reentrancy attempts**: `reentrancy_attempt`
   - Service ID, method name, caller, result="reentrancy_detected", call stack

**Implementation**: `src/runtime/engine.rs:4466-4518`

---

## 3. What `@secure` Does NOT Protect

### 3.1 **Cross-Method Calls Within Same Service**

**Not Protected**:
- ✅ Method A calling Method B on the same service instance is **allowed**
- ✅ This is intentional - different methods can call each other

```dal
@secure
service Service {
    fn method_a() {
        self.method_b();  // ✅ Allowed - different method
    }
    
    fn method_b() {
        // Can be called from method_a
    }
}
```

**Why**: Reentrancy protection only prevents re-entry into the **same** method, not cross-method calls.

### 3.2 **Calls to Different Service Instances**

**Not Protected**:
- ✅ Same method name on different service instances is **allowed**

```dal
@secure
service Service {
    fn transfer() { }
}

// Instance 1: "service_123"
// Instance 2: "service_456"

// Both can have transfer() active simultaneously ✅
```

**Why**: Reentrancy key includes `instance_id`, so different instances are tracked separately.

### 3.3 **Internal Function Calls**

**Not Protected**:
- ✅ Top-level DAL functions (not service methods) are **not protected** by `@secure`
- ✅ Only service method calls go through `execute_service_method()`

```dal
@secure
service Service {
    fn method() {
        helper_function();  // ✅ Not protected by @secure
    }
}

fn helper_function() {
    // Not a service method - no @secure protection
}
```

### 3.4 **Nested Service Calls**

**Protected**:
- ✅ If Service A (with `@secure`) calls Service B (with `@secure`), both are protected
- ✅ Each service method gets its own reentrancy token

```dal
@secure
service ServiceA {
    fn call_b() {
        let b = ServiceB::new();
        b.method();  // ✅ ServiceB's @secure is checked separately
    }
}

@secure
service ServiceB {
    fn method() {
        // Protected by ServiceB's @secure
    }
}
```

### 3.5 **State Modifications**

**Not Protected**:
- ❌ `@secure` does **NOT** protect against:
  - Race conditions between different methods
  - Concurrent access from different callers
  - State corruption from logic errors
  - Integer overflow/underflow (use `@safe_math`)

**Only Protects**:
- ✅ Authentication (who can call)
- ✅ Reentrancy (same method re-entry)
- ✅ Audit logging (what was called)

---

## 4. Inheritance and Scope Rules

### 4.1 **Service-Level Inheritance**

**All methods inherit `@secure`**:
```dal
@secure
service Service {
    fn method1() { }  // ✅ Protected
    fn method2() { }  // ✅ Protected
    fn method3() { }  // ✅ Protected
}
```

**No selective protection**: You cannot have some methods secure and others not within the same service.

### 4.2 **Mutual Exclusivity**

**Cannot Combine**:
- ❌ `@secure` and `@public` are **mutually exclusive**
- ✅ Parser enforces this (`src/parser/parser.rs:2298-2309`)

```dal
@secure
@public  // ❌ Parser error: mutually exclusive
service InvalidService { }
```

### 4.3 **No Override Mechanism**

**Current Limitation**:
- ❌ Cannot override `@secure` at method level
- ❌ Cannot have `@public` method in `@secure` service
- ✅ Must use separate services for different access levels

---

## 5. Runtime Execution Flow

### 5.1 **Method Call Sequence**

When a method on a `@secure` service is called:

```
1. execute_service_method() called
   ↓
2. Check service attributes for "@secure"
   ↓
3. [IF @secure] Enter reentrancy guard
   ├─ Check if method already active
   ├─ If yes → Return ReentrancyDetected error
   └─ If no → Acquire token, add to active calls
   ↓
4. [IF @secure] Check authentication
   ├─ Verify current_caller is set and not null
   ├─ If no → Drop token, return AccessDenied error
   └─ If yes → Log successful access
   ↓
5. Execute method body
   ├─ Reentrancy token held during execution
   └─ All nested calls checked separately
   ↓
6. Method returns
   └─ Reentrancy token dropped (automatic via Drop trait)
```

### 5.2 **Token Lifetime**

**Reentrancy Token Scope**:
- **Acquired**: Before authentication check
- **Held**: For entire method execution
- **Released**: When method returns (via `Drop` trait)

**Critical**: Token must live for entire execution to prevent reentrancy during method body execution.

---

## 6. Interaction with Other Attributes

### 6.1 **Compatible Attributes**

**Can Combine With**:
- ✅ `@trust("hybrid"|"centralized"|"decentralized")`
- ✅ `@chain("ethereum", ...)`
- ✅ `@txn` (transaction support)
- ✅ `@limit(n)` (resource limits)
- ✅ `@compile_target(...)`
- ✅ `@interface(...)`
- ✅ `@ai` (AI capabilities)

### 6.2 **Incompatible Attributes**

**Cannot Combine**:
- ❌ `@public` (mutually exclusive)

### 6.3 **Attribute Order**

**No Order Dependency**:
```dal
@secure @trust("hybrid") @chain("ethereum")  // ✅ Works
@trust("hybrid") @secure @chain("ethereum")  // ✅ Also works
```

---

## 7. Security Boundaries

### 7.1 **What `@secure` Guarantees**

✅ **Authentication**: Only authenticated callers can invoke methods
✅ **Reentrancy**: Same method cannot be re-entered while executing
✅ **Audit Trail**: All access attempts are logged
✅ **Caller Identity**: `current_caller` must be set and valid

### 7.2 **What `@secure` Does NOT Guarantee**

❌ **Authorization**: Does not check if caller has permission for specific operations
❌ **Rate Limiting**: Does not prevent excessive calls (use `@limit`)
❌ **Input Validation**: Does not validate method parameters
❌ **State Consistency**: Does not prevent race conditions between methods
❌ **Overflow Protection**: Does not prevent integer overflow (use `@safe_math`)
❌ **Cross-Service Attacks**: Does not protect against malicious service calls

---

## 8. Code References

### Key Implementation Files

1. **Service Attribute Check**: `src/runtime/engine.rs:4442-4455`
2. **Reentrancy Protection**: `src/runtime/engine.rs:4462-4483`
3. **Authentication Check**: `src/runtime/engine.rs:4485-4508`
4. **Audit Logging**: `src/runtime/engine.rs:4466-4518`
5. **ReentrancyGuard**: `src/runtime/reentrancy.rs`
6. **Parser Validation**: `src/parser/parser.rs:2298-2309`

### Key Data Structures

- **ReentrancyGuard**: `src/runtime/reentrancy.rs:9-84`
- **ReentrancyToken**: `src/runtime/reentrancy.rs:88-97`
- **Call Key Format**: `"{instance_id}::{method_name}"`

---

## 9. Examples

### Example 1: Basic Service Protection

```dal
@secure
service UserService {
    fn update_profile(user_id: string, data: map<string, any>) {
        // Protected: requires authentication
        // Protected: cannot be re-entered
        // Logged: all access attempts
    }
}
```

### Example 2: Cross-Method Calls

```dal
@secure
service BankService {
    fn transfer(from: string, to: string, amount: int) {
        self.validate_account(from);  // ✅ Allowed - different method
        self.update_balance(from, -amount);  // ✅ Allowed - different method
    }
    
    fn validate_account(account: string) {
        // ✅ Can be called from transfer()
    }
    
    fn update_balance(account: string, delta: int) {
        // ✅ Can be called from transfer()
    }
}
```

### Example 3: Reentrancy Prevention

```dal
@secure
service VulnerableService {
    fn withdraw(amount: int) {
        // ❌ This would fail if withdraw() called itself:
        // self.withdraw(amount);  // ReentrancyDetected error
        
        // ✅ But this is allowed:
        self.process_withdrawal(amount);  // Different method
    }
    
    fn process_withdrawal(amount: int) {
        // Different method - allowed
    }
}
```

---

## 10. Summary

### Scope Summary

| Aspect | Scope |
|--------|-------|
| **Application** | Service-level only |
| **Inheritance** | All methods in service |
| **Reentrancy** | Per `instance_id::method_name` |
| **Authentication** | Per method call |
| **Audit Logging** | All access attempts |
| **Token Lifetime** | Entire method execution |
| **Cross-Method** | Allowed (different methods) |
| **Cross-Instance** | Allowed (different instances) |

### Protection Summary

**Protected**:
- ✅ Unauthenticated access
- ✅ Same-method reentrancy
- ✅ Access logging

**Not Protected**:
- ❌ Cross-method calls
- ❌ Authorization (permissions)
- ❌ Input validation
- ❌ State race conditions
- ❌ Integer overflow

---

**Last Updated**: February 2026
**Implementation Version**: Current (with reentrancy protection)

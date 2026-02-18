# Reentrancy Protection and `@secure` - Clarification

## Summary

**`@secure` provides BOTH authentication/authorization AND reentrancy protection.** This was implemented to provide comprehensive security for secure services.

## What `@secure` Does

Based on the current implementation (`src/runtime/engine.rs:4457-4524`), `@secure` provides:

1. **Reentrancy Protection**: Prevents re-entry into the same method/contract combination
2. **Authentication Enforcement**: Checks if `current_caller` is set and not null/default
3. **Audit Logging**: Automatically logs access attempts (both allowed and denied) and reentrancy attempts
4. **Access Control**: Returns `RuntimeError::AccessDenied` if caller is not authenticated
5. **Reentrancy Detection**: Returns `RuntimeError::ReentrancyDetected` if re-entry is attempted

### Code Reference

```rust
// From src/runtime/engine.rs:4457-4524
// Enforce @secure attribute: require authentication AND reentrancy protection
if has_secure_attr {
    // 1. REENTRANCY PROTECTION: Check and enter reentrancy guard
    let guard = self.reentrancy_guard.clone();
    let token = guard.enter(method_name, Some(instance_id))?;
    // Token held for entire method execution, released on return
    
    // 2. AUTHENTICATION: Check if caller is authenticated
    let is_authenticated = self.current_caller.as_ref()
        .map(|caller| {
            caller != "0x0000000000000000000000000000000000000000" && !caller.is_empty()
        })
        .unwrap_or(false);

    if !is_authenticated {
        drop(token); // Release guard before error
        return Err(RuntimeError::AccessDenied);
    }
    
    // Log successful authenticated access
    // ... audit logging ...
}
```

## Reentrancy Protection System

The codebase DOES have a reentrancy protection system:

- **Location**: `src/runtime/reentrancy.rs`
- **Component**: `ReentrancyGuard` struct
- **Mechanism**: Tracks active calls using `HashSet<String>` and call stack
- **Protection**: Prevents re-entry into the same function/contract combination

### How ReentrancyGuard Works

```rust
// From src/runtime/reentrancy.rs:22-53
pub fn enter(&self, function_name: &str, contract_address: Option<&str>) -> Result<ReentrancyToken, RuntimeError> {
    let call_key = match contract_address {
        Some(addr) => format!("{}::{}", addr, function_name),
        None => function_name.to_string(),
    };

    // Check for re-entrancy
    if active_calls.contains(&call_key) {
        return Err(RuntimeError::ReentrancyDetected(format!(
            "Re-entrancy detected in function: {} (call stack: {:?})",
            call_key, *call_stack
        )));
    }

    // Add to active calls and call stack
    active_calls.insert(call_key.clone());
    call_stack.push(call_key.clone());
    // ...
}
```

### Current Status

**The `ReentrancyGuard` is now integrated with `@secure`:**

- ✅ It IS automatically invoked when `@secure` is used
- ✅ The guard is called in `execute_service_method` before authentication checks
- ✅ Reentrancy tokens are held for the entire method execution
- ✅ Reentrancy attempts are logged for audit purposes

## The Confusion

The Solidity converter (`src/solidity_converter/security.rs:18-20`) suggests BOTH attributes for reentrancy protection:

```rust
if self.has_reentrancy_risk(contract) {
    patterns.push("@secure".to_string());
    patterns.push("@reentrancy_guard".to_string());
}
```

This suggests that:
- `@secure` is recommended for reentrancy protection (as guidance)
- `@reentrancy_guard` should also be used (but this attribute doesn't appear to be implemented)

## Usage

### For Reentrancy Protection

Simply use `@secure` on your service - it now provides both authentication and reentrancy protection automatically:

```dal
@secure
service MySecureService {
    fn transfer(from: string, to: string, amount: int) {
        // Automatically protected from:
        // 1. Unauthenticated access
        // 2. Reentrancy attacks
        // 3. All access is audit logged
    }
}
```

### Current Best Practice

Based on the current implementation:

- Use `@secure` for **authentication/authorization AND reentrancy protection**
- Reentrancy protection is **automatically provided** by `@secure`
- The `ReentrancyGuard` is integrated and invoked automatically
- No additional attributes needed for basic reentrancy protection

## Code Locations

- **`@secure` enforcement**: `src/runtime/engine.rs:4446-4490`
- **ReentrancyGuard implementation**: `src/runtime/reentrancy.rs`
- **ReentrancyGuard instance**: `src/runtime/engine.rs:60,125`
- **Solidity converter suggestion**: `src/solidity_converter/security.rs:18-20`

## Conclusion

**`@secure` = Authentication + Reentrancy Protection + Audit Logging**

The `@secure` attribute now provides comprehensive security:
- ✅ Authentication/Authorization enforcement
- ✅ Automatic reentrancy protection
- ✅ Complete audit logging of all access attempts and reentrancy attempts
- ✅ Single attribute for all security needs

**Implementation Date**: February 2026
**Code Location**: `src/runtime/engine.rs:4457-4524`

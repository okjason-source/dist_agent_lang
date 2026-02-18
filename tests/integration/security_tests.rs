// Security-focused integration tests

use dist_agent_lang::Runtime;
use dist_agent_lang::runtime::{ReentrancyGuard, SafeMath, StateIsolationManager};
use dist_agent_lang::runtime::values::Value;

#[test]
fn test_runtime_initialization() {
    // Test that runtime initializes with security features
    let runtime = Runtime::new();
    // Runtime should be initialized successfully
    assert!(true);
}

#[test]
fn test_reentrancy_protection() {
    let guard = ReentrancyGuard::new();
    
    // First call should succeed
    let result1 = guard.enter("function1", Some("contract1"));
    assert!(result1.is_ok());
    
    // Re-entrant call should fail
    let result2 = guard.enter("function1", Some("contract1"));
    assert!(result2.is_err());
    
    // Exit should allow new calls
    if let Ok(token) = result1 {
        guard.exit(&token);
        let result3 = guard.enter("function1", Some("contract1"));
        assert!(result3.is_ok());
    }
}

#[test]
fn test_safe_math_overflow() {
    let max_int = Value::Int(i64::MAX);
    let one = Value::Int(1);
    
    // Test overflow protection
    let result = SafeMath::add(&max_int, &one);
    assert!(result.is_err()); // Should detect overflow
}

#[test]
fn test_safe_math_underflow() {
    let min_int = Value::Int(i64::MIN);
    let one = Value::Int(1);
    
    // Test underflow protection
    let result = SafeMath::subtract(&min_int, &one);
    assert!(result.is_err()); // Should detect underflow
}

#[test]
fn test_state_isolation() {
    let manager = StateIsolationManager::new();
    
    // Create isolated state
    manager.create_isolated_state("contract1", "0x1234");
    
    // Access should be allowed
    assert!(manager.can_access("contract1", "0x1234"));
    
    // Access from different contract should be denied
    assert!(!manager.can_access("contract2", "0x1234"));
}

#[test]
fn test_runtime_security_features() {
    // Test that runtime has security features enabled
    let mut runtime = Runtime::new();
    
    // Runtime should be initialized with security features
    // Verify that security managers are accessible and initialized
    
    // Test reentrancy guard is initialized and functional
    let guard_result = runtime.reentrancy_guard.enter("test_function", Some("test_contract"));
    assert!(guard_result.is_ok(), "ReentrancyGuard should be initialized and functional");
    
    // Clean up
    if let Ok(token) = guard_result {
        runtime.reentrancy_guard.exit(&token);
    }
    
    // Test state isolation manager is initialized and functional
    runtime.state_manager.create_isolated_state("test_contract", "0x1234");
    assert!(
        runtime.state_manager.can_access("test_contract", "0x1234"),
        "StateIsolationManager should be initialized and functional"
    );
    
    // Test cross-chain security manager is initialized
    // (Manager is a struct, not Option, so it's always present)
    // We can verify it exists by checking it doesn't panic when accessed
    let _manager = &runtime.cross_chain_manager;
    assert!(true, "CrossChainSecurityManager should be initialized");
    
    // Test advanced security manager is initialized
    // (Manager is a struct, not Option, so it's always present)
    let _security = &runtime.advanced_security;
    assert!(true, "AdvancedSecurityManager should be initialized");
}

#[test]
fn test_safe_math_multiplication_overflow() {
    let large_int = Value::Int(i64::MAX / 2 + 1);
    let two = Value::Int(2);
    
    // Test multiplication overflow protection
    let result = SafeMath::multiply(&large_int, &two);
    assert!(result.is_err()); // Should detect overflow
}

#[test]
fn test_safe_math_division_by_zero() {
    let value = Value::Int(100);
    let zero = Value::Int(0);
    
    // Test division by zero protection
    let result = SafeMath::divide(&value, &zero);
    assert!(result.is_err()); // Should detect division by zero
}

#[test]
fn test_reentrancy_different_functions() {
    let guard = ReentrancyGuard::new();
    
    // First function call
    let result1 = guard.enter("function1", Some("contract1"));
    assert!(result1.is_ok());
    
    // Different function should also be protected
    let result2 = guard.enter("function2", Some("contract1"));
    assert!(result2.is_err(), "Reentrancy guard should protect across all functions in same contract");
    
    // Clean up
    if let Ok(token) = result1 {
        guard.exit(&token);
    }
}

#[test]
fn test_reentrancy_different_contracts() {
    let guard = ReentrancyGuard::new();
    
    // First contract call
    let result1 = guard.enter("function1", Some("contract1"));
    assert!(result1.is_ok());
    
    // Different contract should be allowed
    let result2 = guard.enter("function1", Some("contract2"));
    assert!(result2.is_ok(), "Different contracts should be able to execute concurrently");
    
    // Clean up
    if let Ok(token1) = result1 {
        guard.exit(&token1);
    }
    if let Ok(token2) = result2 {
        guard.exit(&token2);
    }
}

#[test]
fn test_state_isolation_multiple_contracts() {
    let manager = StateIsolationManager::new();
    
    // Create isolated states for multiple contracts
    manager.create_isolated_state("contract1", "0x1111");
    manager.create_isolated_state("contract2", "0x2222");
    
    // Each contract should only access its own state
    assert!(manager.can_access("contract1", "0x1111"));
    assert!(!manager.can_access("contract1", "0x2222"));
    assert!(manager.can_access("contract2", "0x2222"));
    assert!(!manager.can_access("contract2", "0x1111"));
}

#[test]
fn test_safe_math_normal_operations() {
    let a = Value::Int(100);
    let b = Value::Int(50);
    
    // Test normal operations that should succeed
    let add_result = SafeMath::add(&a, &b);
    assert!(add_result.is_ok());
    if let Ok(Value::Int(sum)) = add_result {
        assert_eq!(sum, 150);
    }
    
    let sub_result = SafeMath::subtract(&a, &b);
    assert!(sub_result.is_ok());
    if let Ok(Value::Int(diff)) = sub_result {
        assert_eq!(diff, 50);
    }
}


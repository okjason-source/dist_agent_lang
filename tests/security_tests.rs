// Security-focused integration tests

use dist_agent_lang::runtime::values::Value;
use dist_agent_lang::runtime::{ReentrancyGuard, SafeMath, StateIsolationManager};
use dist_agent_lang::Runtime;

#[test]
fn test_runtime_initialization() {
    // Test that runtime initializes with security features
    let _runtime = Runtime::new();
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

    // Drop the token (which will exit automatically)
    drop(result1.unwrap());

    // Now new calls should succeed
    let result3 = guard.enter("function1", Some("contract1"));
    assert!(result3.is_ok());
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
    let mut manager = StateIsolationManager::new();

    // Create isolated contract state
    let result = manager.create_contract(
        "0x1234".to_string(),
        "Contract1".to_string(),
        "0xOwner".to_string(),
        "hybrid".to_string(),
    );

    assert!(result.is_ok());

    // Verify contract exists
    assert!(manager.get_contract("0x1234").is_some());

    // Verify different contract doesn't exist
    assert!(manager.get_contract("0x5678").is_none());
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

    // Different function in same contract should be allowed (function-level guard)
    let result2 = guard.enter("function2", Some("contract1"));
    assert!(result2.is_ok(), "Different functions in same contract can execute concurrently (function-level reentrancy protection)");

    // But calling the SAME function again should fail
    let result3 = guard.enter("function1", Some("contract1"));
    assert!(
        result3.is_err(),
        "Same function should be protected from reentrancy"
    );

    // Clean up
    drop(result1.unwrap());
    drop(result2.unwrap());
}

#[test]
fn test_reentrancy_different_contracts() {
    let guard = ReentrancyGuard::new();

    // First contract call
    let result1 = guard.enter("function1", Some("contract1"));
    assert!(result1.is_ok());

    // Different contract should be allowed to execute concurrently
    let result2 = guard.enter("function1", Some("contract2"));
    assert!(
        result2.is_ok(),
        "Different contracts should be able to execute concurrently"
    );

    // Clean up
    drop(result1.unwrap());
    drop(result2.unwrap());
}

#[test]
fn test_state_isolation_multiple_contracts() {
    let mut manager = StateIsolationManager::new();

    // Create multiple isolated contract states
    let result1 = manager.create_contract(
        "0x1111".to_string(),
        "Contract1".to_string(),
        "0xOwner1".to_string(),
        "hybrid".to_string(),
    );
    assert!(result1.is_ok());

    let result2 = manager.create_contract(
        "0x2222".to_string(),
        "Contract2".to_string(),
        "0xOwner2".to_string(),
        "hybrid".to_string(),
    );
    assert!(result2.is_ok());

    // Each contract should exist independently
    assert!(manager.get_contract("0x1111").is_some());
    assert!(manager.get_contract("0x2222").is_some());

    // Verify isolation - contract 1 shouldn't be accessible as contract 2
    assert!(manager.get_contract("0x3333").is_none());
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

    let mul_result = SafeMath::multiply(&a, &b);
    assert!(mul_result.is_ok());
    if let Ok(Value::Int(product)) = mul_result {
        assert_eq!(product, 5000);
    }

    let div_result = SafeMath::divide(&a, &b);
    assert!(div_result.is_ok());
    if let Ok(Value::Int(quotient)) = div_result {
        assert_eq!(quotient, 2);
    }
}

#[test]
fn test_runtime_security_features() {
    // Test that runtime has security features enabled
    let mut runtime = Runtime::new();

    // Runtime should be initialized with security features
    // Verify that security managers are accessible and initialized

    // Test reentrancy guard is initialized and functional
    let guard_result = runtime
        .reentrancy_guard
        .enter("test_function", Some("test_contract"));
    assert!(
        guard_result.is_ok(),
        "ReentrancyGuard should be initialized and functional"
    );

    // Clean up - drop the token (which will exit automatically via Drop trait)
    drop(guard_result.unwrap());

    // Test state isolation manager is initialized and functional
    let result = runtime.state_manager.create_contract(
        "0x1234".to_string(),
        "test_contract".to_string(),
        "0xOwner".to_string(),
        "hybrid".to_string(),
    );
    assert!(
        result.is_ok(),
        "StateIsolationManager should be initialized and functional"
    );
    assert!(
        runtime.state_manager.get_contract("0x1234").is_some(),
        "StateIsolationManager should create contracts"
    );

    // Test cross-chain security manager is initialized
    // (Manager is a struct, not Option, so it's always present)
    // We can verify it exists by checking it doesn't panic when accessed
    let _manager = &runtime.cross_chain_manager;
    let _security = &runtime.advanced_security;
}

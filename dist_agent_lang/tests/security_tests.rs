// Security-focused integration tests

use dist_agent_lang::Runtime;
use dist_agent_lang::runtime::{ReentrancyGuard, SafeMath, StateIsolationManager};
use dist_agent_lang::runtime::values::Value;

#[test]
fn test_runtime_initialization() {
    // Test that runtime initializes with security features
    let _runtime = Runtime::new();
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


// Integration tests for transaction module with Runtime/Engine
// Tests Phase 3: Engine and Runtime Integration

use dist_agent_lang::runtime::transaction::IsolationLevel;
use dist_agent_lang::runtime::values::Value;
use dist_agent_lang::runtime::Runtime;

#[test]
fn test_runtime_transaction_lifecycle() {
    let mut runtime = Runtime::new();

    // Begin transaction via Runtime method
    let tx_id = runtime
        .begin_transaction(IsolationLevel::Serializable, None)
        .expect("Failed to begin transaction");

    assert_eq!(
        runtime.current_transaction(),
        Some(tx_id.as_str()),
        "Transaction should be active"
    );

    // Write some transactional state
    runtime
        .transaction_write("test_key".to_string(), Value::Int(42))
        .expect("Failed to write to transaction");

    // Read it back
    let value = runtime
        .transaction_read("test_key")
        .expect("Failed to read from transaction");
    assert_eq!(value, Some(Value::Int(42)), "Should read written value");

    // Commit
    runtime
        .commit_transaction()
        .expect("Failed to commit transaction");

    assert_eq!(
        runtime.current_transaction(),
        None,
        "Transaction should be cleared after commit"
    );
}

#[test]
fn test_runtime_transaction_rollback() {
    let mut runtime = Runtime::new();

    // Setup: commit initial state
    runtime
        .begin_transaction(IsolationLevel::Serializable, None)
        .unwrap();
    runtime
        .transaction_write("balance".to_string(), Value::Int(1000))
        .unwrap();
    runtime.commit_transaction().unwrap();

    // Start new transaction and modify state
    runtime
        .begin_transaction(IsolationLevel::Serializable, None)
        .unwrap();
    runtime
        .transaction_write("balance".to_string(), Value::Int(500))
        .unwrap();

    // Rollback
    runtime
        .rollback_transaction()
        .expect("Failed to rollback transaction");

    // Verify state reverted
    runtime
        .begin_transaction(IsolationLevel::ReadCommitted, None)
        .unwrap();
    let balance = runtime.transaction_read("balance").unwrap();
    assert_eq!(
        balance,
        Some(Value::Int(1000)),
        "Balance should be reverted to 1000 after rollback"
    );
    runtime.commit_transaction().unwrap();
}

#[test]
fn test_runtime_savepoint_and_partial_rollback() {
    let mut runtime = Runtime::new();

    runtime
        .begin_transaction(IsolationLevel::Serializable, None)
        .unwrap();

    // First write
    runtime
        .transaction_write("key1".to_string(), Value::String("value1".to_string()))
        .unwrap();

    // Create savepoint
    runtime
        .create_savepoint("sp1".to_string())
        .expect("Failed to create savepoint");

    // Second write after savepoint
    runtime
        .transaction_write("key2".to_string(), Value::String("value2".to_string()))
        .unwrap();

    // Rollback to savepoint (key2 should be removed, key1 kept)
    runtime
        .rollback_to_savepoint("sp1")
        .expect("Failed to rollback to savepoint");

    // Check state
    let val1 = runtime.transaction_read("key1").unwrap();
    let val2 = runtime.transaction_read("key2").unwrap();

    assert_eq!(
        val1,
        Some(Value::String("value1".to_string())),
        "key1 should still exist"
    );
    assert_eq!(val2, None, "key2 should be rolled back");

    runtime.commit_transaction().unwrap();
}

#[test]
fn test_no_transaction_errors() {
    let mut runtime = Runtime::new();

    // Try to commit without active transaction
    let result = runtime.commit_transaction();
    assert!(result.is_err(), "Commit without transaction should error");

    // Try to rollback without active transaction
    let result = runtime.rollback_transaction();
    assert!(result.is_err(), "Rollback without transaction should error");

    // Try to read without active transaction
    let result = runtime.transaction_read("key");
    assert!(result.is_err(), "Read without transaction should error");

    // Try to write without active transaction
    let result = runtime.transaction_write("key".to_string(), Value::Int(1));
    assert!(result.is_err(), "Write without transaction should error");
}

#[test]
fn test_transaction_isolation_levels() {
    // Test different isolation levels are accepted
    let mut runtime = Runtime::new();

    for level in &[
        IsolationLevel::ReadUncommitted,
        IsolationLevel::ReadCommitted,
        IsolationLevel::RepeatableRead,
        IsolationLevel::Serializable,
    ] {
        let _tx_id = runtime
            .begin_transaction(*level, None)
            .expect("Failed to begin transaction");
        assert!(runtime.current_transaction().is_some());
        runtime.rollback_transaction().unwrap();
    }
}

#[test]
fn test_transaction_timeout_configuration() {
    let mut runtime = Runtime::new();

    // Begin transaction with 5-second timeout
    let _tx_id = runtime
        .begin_transaction(IsolationLevel::Serializable, Some(5000))
        .expect("Failed to begin transaction with timeout");

    assert!(runtime.current_transaction().is_some());

    // Transaction should work normally within timeout
    runtime
        .transaction_write("key".to_string(), Value::Int(123))
        .unwrap();
    runtime.commit_transaction().unwrap();
}

#[test]
fn test_multiple_transactions_sequential() {
    let mut runtime = Runtime::new();

    // Transaction 1: Set initial value
    runtime
        .begin_transaction(IsolationLevel::Serializable, None)
        .unwrap();
    runtime
        .transaction_write("counter".to_string(), Value::Int(0))
        .unwrap();
    runtime.commit_transaction().unwrap();

    // Transaction 2: Increment
    runtime
        .begin_transaction(IsolationLevel::Serializable, None)
        .unwrap();
    let val = runtime.transaction_read("counter").unwrap().unwrap();
    if let Value::Int(n) = val {
        runtime
            .transaction_write("counter".to_string(), Value::Int(n + 1))
            .unwrap();
    }
    runtime.commit_transaction().unwrap();

    // Transaction 3: Verify
    runtime
        .begin_transaction(IsolationLevel::ReadCommitted, None)
        .unwrap();
    let final_val = runtime.transaction_read("counter").unwrap();
    assert_eq!(
        final_val,
        Some(Value::Int(1)),
        "Counter should be incremented to 1"
    );
    runtime.commit_transaction().unwrap();
}

/// @txn attribute: function with @txn commits on success
#[test]
fn test_txn_attribute_commit_on_success() {
    let source = r#"
        @txn
        fn write_key() {
            database::tx_write("attr_key", 100);
        }
        write_key();
        let tx_id = database::begin_transaction("read_committed", 5000);
        let v = database::tx_read("attr_key");
        database::commit();
        v
    "#;
    let result = dist_agent_lang::execute_source(source).expect("Execution failed");
    assert_eq!(result, Value::Int(100), "@txn function should commit tx_write");
}

/// @txn attribute: function with @txn rolls back on error
#[test]
fn test_txn_attribute_rollback_on_error() {
    // Single script: commit 42, then @txn fn writes 999 and throws → rollback, then read → 42
    let source = r#"
        let tx_id = database::begin_transaction("read_committed", 5000);
        database::tx_write("rollback_key", 42);
        database::commit();

        @txn
        fn bad_write() {
            database::tx_write("rollback_key", 999);
            throw "abort";
        }
        try {
            bad_write();
        } catch (e) {
            null;
        }
        let tx_id2 = database::begin_transaction("read_committed", 5000);
        let v = database::tx_read("rollback_key");
        database::commit();
        v
    "#;
    let result = dist_agent_lang::execute_source(source).expect("Execution failed");
    assert_eq!(
        result, Value::Int(42),
        "After @txn function throws, tx_write should be rolled back"
    );
}

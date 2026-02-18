pub mod engine;
pub mod functions;
pub mod scope;
pub mod values;
pub mod types;
pub mod reentrancy;
pub mod safe_math;
pub mod state_isolation;
pub mod advanced_security;
pub mod transaction;
pub mod control_flow;

pub use engine::Runtime;

// Re-export security modules for testing and external use
pub use reentrancy::ReentrancyGuard;
pub use safe_math::SafeMath;
pub use state_isolation::StateIsolationManager;
pub use control_flow::{ControlFlow, StatementOutcome, StatementResult};
pub use transaction::{
    TransactionManager, IsolationLevel, TransactionError, TransactionState,
    StateStorage, InMemoryStorage, FileBackedStorage,
    TransactionEvent, TransactionEventCallback,
    TransactionLog, TransactionLogEntry,
};

#[cfg(feature = "sqlite-storage")]
pub use transaction::SqliteStorage;

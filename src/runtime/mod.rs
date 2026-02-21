pub mod advanced_security;
pub mod control_flow;
pub mod engine;
pub mod functions;
pub mod reentrancy;
pub mod safe_math;
pub mod scope;
pub mod state_isolation;
pub mod transaction;
pub mod types;
pub mod values;

pub use engine::Runtime;
pub use functions::{CallFrameInfo, RuntimeError, RuntimeErrorWithContext, SourceLocation};

// Re-export security modules for testing and external use
pub use control_flow::{ControlFlow, StatementOutcome, StatementResult};
pub use reentrancy::ReentrancyGuard;
pub use safe_math::SafeMath;
pub use state_isolation::StateIsolationManager;
pub use transaction::{
    FileBackedStorage, InMemoryStorage, IsolationLevel, StateStorage, TransactionError,
    TransactionEvent, TransactionEventCallback, TransactionLog, TransactionLogEntry,
    TransactionManager, TransactionState,
};

#[cfg(feature = "sqlite-storage")]
pub use transaction::SqliteStorage;

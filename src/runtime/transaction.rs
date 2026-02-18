// Transaction Atomicity Module
// Provides ACID guarantees for distributed operations
//
// # Features
//
// - **Transaction lifecycle**: begin/commit/rollback with ACID semantics
// - **Savepoints**: Partial rollback to named checkpoints within a transaction
// - **Isolation levels**: ReadUncommitted, ReadCommitted (default), RepeatableRead, Serializable
// - **Deadlock detection**: Timeout-based detection of circular wait conditions
// - **Observability**: Optional event callbacks for transaction lifecycle (begin, commit, rollback, conflicts, deadlocks)
// - **Pluggable storage**: StateStorage trait; in-memory, file-backed, and SQLite backends available
// - **Resource limits**: Configurable max active transactions and keys per transaction (Phase 4)
// - **Audit logging**: Optional append-only transaction log for compliance and debugging (Phase 4)
// - **Two-phase commit**: Basic support for distributed transactions
//
// # Isolation Levels
//
// - **ReadUncommitted**: Lowest isolation; dirty reads possible; highest performance; no read locks.
// - **ReadCommitted** (recommended default): Prevents dirty reads; transactions see only committed data; standard for most use cases.
// - **RepeatableRead**: Prevents non-repeatable reads; same query returns same results within a transaction.
// - **Serializable**: Highest isolation; transactions appear to execute serially; lowest concurrency.
//
// # Safe Production Defaults (Phase 4)
//
// - **Isolation level**: `ReadCommitted` (balance of correctness and performance)
// - **Transaction timeout**: `30000ms` (30 seconds; prevents hung transactions)
// - **Max active transactions**: `1000` (prevents resource exhaustion)
// - **Max keys per transaction**: `10000` (prevents unbounded memory use)
// - **Storage backend**: `memory` (use `file` or `sqlite` for persistence)
//
// Set via environment:
// ```bash
// export DAL_TX_STORAGE=sqlite
// export DAL_TX_TIMEOUT_MS=30000
// export DAL_TX_MAX_ACTIVE=1000
// export DAL_TX_MAX_KEYS=10000
// export DAL_TX_LOG_PATH=/var/log/dal/transactions.log  # Optional audit log
// ```
//
// # Usage
//
// ```rust
// use dist_agent_lang::runtime::{TransactionManager, IsolationLevel, Value};
//
// let mut manager = TransactionManager::new()
//     .with_default_timeout(Some(30000)) // 30 seconds
//     .with_event_callback(Box::new(|event| {
//         println!("Transaction event: {:?}", event);
//     }));
//
// // Begin a transaction
// let tx_id = manager.begin_transaction(IsolationLevel::ReadCommitted).unwrap();
//
// // Read and write within transaction
// manager.write(&tx_id, "balance".to_string(), Value::Int(100)).unwrap();
// let value = manager.read(&tx_id, "balance").unwrap();
//
// // Create savepoint (optional)
// manager.create_savepoint(&tx_id, "checkpoint1".to_string()).unwrap();
//
// // Commit or rollback
// manager.commit(&tx_id).unwrap();
// // or: manager.rollback(&tx_id).unwrap();
// // or: manager.rollback_to_savepoint(&tx_id, "checkpoint1").unwrap();
// ```
//
// # Deadlock Detection (Phase 1)
//
// Current implementation uses timeout-based deadlock detection: if a transaction
// exceeds its timeout while trying to acquire a lock, it's treated as a potential
// deadlock and returns `TransactionError::Deadlock`. Future phases may add
// cycle detection in the lock wait graph for more precise detection.
//
// # Durability Contracts (Phase 2+)
//
// ## InMemoryStorage
// - **Durability**: None. State is lost on process exit or crash.
// - **Performance**: Highest; no I/O overhead.
// - **Use case**: Development, testing, ephemeral state.
//
// ## FileBackedStorage
// - **Durability**: Strong. Each `set()` and `remove()` flushes to disk immediately by default.
// - **Performance**: Moderate; disk I/O on every write. Suitable for moderate write loads.
// - **Use case**: Single-node production deployments with state persistence requirements.
// - **Format**: JSON file; human-readable; atomic rename for crash safety.
//
// ## TransactionLog (Optional)
// - **Durability**: Strong. Append-only log with immediate flush after each event.
// - **Performance**: Moderate; one append per transaction event.
// - **Use case**: Audit trail, debugging, potential recovery (future).
// - **Format**: Line-delimited JSON; one event per line.
//
// # Audit Logging (Phase 4)
//
// Enable structured audit logging with `DAL_TX_LOG_PATH`:
// ```bash
// export DAL_TX_LOG_PATH=/var/log/dal/transactions.log
// ```
//
// Each transaction event is written as line-delimited JSON:
// ```json
// {"timestamp":1675889234567,"tx_id":"tx_1","event_type":"begin","keys":[],"isolation_level":"ReadCommitted"}
// {"timestamp":1675889234578,"tx_id":"tx_1","event_type":"write","keys":["account:123:balance"],"isolation_level":null}
// {"timestamp":1675889234589,"tx_id":"tx_1","event_type":"commit","keys":["account:123:balance"],"isolation_level":null}
// ```
//
// Use for:
// - Compliance and audit trails
// - Debugging production issues
// - Performance analysis
// - Future: Replay and recovery
//
// # Future Enhancements (Phase 5+)
//
// See `docs/development/implementation/TRANSACTION_ADVANCED_FEATURES_PLAN.md` for detailed roadmap:
//
// - **Phase 5**: Write-ahead log (WAL) for point-in-time recovery, read-only transaction optimization, key expiration (TTL)
// - **Phase 6**: Cycle detection for deadlocks (wait-for graph), distributed two-phase commit with real coordinator
// - **Phase 7**: Transaction metrics & monitoring (Prometheus), transaction debugger/profiler, adaptive isolation levels
// - **Phase 8**: Optimistic concurrency control (OCC), multi-version concurrency control (MVCC)

use crate::runtime::values::Value;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Transaction lifecycle event for observability and debugging
#[derive(Debug, Clone)]
pub enum TransactionEvent {
    Begin {
        tx_id: String,
        isolation_level: IsolationLevel,
    },
    Read {
        tx_id: String,
        key: String,
    },
    Write {
        tx_id: String,
        key: String,
    },
    SavepointCreated {
        tx_id: String,
        savepoint_name: String,
    },
    SavepointRolledBack {
        tx_id: String,
        savepoint_name: String,
    },
    Commit {
        tx_id: String,
        keys_modified: usize,
    },
    Rollback {
        tx_id: String,
    },
    Timeout {
        tx_id: String,
        elapsed_ms: u64,
    },
    Conflict {
        tx_id: String,
        key: String,
        reason: String,
    },
    Deadlock {
        tx_id: String,
    },
}

/// Optional callback for transaction events (for logging, metrics, debugging)
pub type TransactionEventCallback = Box<dyn Fn(&TransactionEvent) + Send + Sync>;

/// Transaction log entry for audit and recovery
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionLogEntry {
    pub timestamp: u64,
    pub tx_id: String,
    pub event_type: String,
    pub keys: Vec<String>,
    pub isolation_level: Option<String>,
}

/// Append-only transaction log for audit trail and potential recovery.
/// Each transaction lifecycle event is written to a line-delimited JSON file.
pub struct TransactionLog {
    file: Option<BufWriter<File>>,
    #[allow(dead_code)]
    #[allow(dead_code)]
    path: PathBuf, // Kept for future use (e.g., rotation, inspection)
}

impl TransactionLog {
    /// Create or open a transaction log at the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Open file in append mode
        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(Self {
            file: Some(BufWriter::new(file)),
            path,
        })
    }

    /// Append a transaction event to the log
    pub fn log_event(&mut self, event: &TransactionEvent) -> io::Result<()> {
        let entry = match event {
            TransactionEvent::Begin {
                tx_id,
                isolation_level,
            } => TransactionLogEntry {
                timestamp: get_current_timestamp(),
                tx_id: tx_id.clone(),
                event_type: "begin".to_string(),
                keys: vec![],
                isolation_level: Some(format!("{:?}", isolation_level)),
            },
            TransactionEvent::Read { tx_id, key } => TransactionLogEntry {
                timestamp: get_current_timestamp(),
                tx_id: tx_id.clone(),
                event_type: "read".to_string(),
                keys: vec![key.clone()],
                isolation_level: None,
            },
            TransactionEvent::Write { tx_id, key } => TransactionLogEntry {
                timestamp: get_current_timestamp(),
                tx_id: tx_id.clone(),
                event_type: "write".to_string(),
                keys: vec![key.clone()],
                isolation_level: None,
            },
            TransactionEvent::Commit {
                tx_id,
                keys_modified,
            } => TransactionLogEntry {
                timestamp: get_current_timestamp(),
                tx_id: tx_id.clone(),
                event_type: "commit".to_string(),
                keys: vec![format!("modified:{}", keys_modified)],
                isolation_level: None,
            },
            TransactionEvent::Rollback { tx_id } => TransactionLogEntry {
                timestamp: get_current_timestamp(),
                tx_id: tx_id.clone(),
                event_type: "rollback".to_string(),
                keys: vec![],
                isolation_level: None,
            },
            TransactionEvent::SavepointCreated {
                tx_id,
                savepoint_name,
            } => TransactionLogEntry {
                timestamp: get_current_timestamp(),
                tx_id: tx_id.clone(),
                event_type: "savepoint_created".to_string(),
                keys: vec![savepoint_name.clone()],
                isolation_level: None,
            },
            TransactionEvent::SavepointRolledBack {
                tx_id,
                savepoint_name,
            } => TransactionLogEntry {
                timestamp: get_current_timestamp(),
                tx_id: tx_id.clone(),
                event_type: "savepoint_rollback".to_string(),
                keys: vec![savepoint_name.clone()],
                isolation_level: None,
            },
            TransactionEvent::Timeout { tx_id, elapsed_ms } => TransactionLogEntry {
                timestamp: get_current_timestamp(),
                tx_id: tx_id.clone(),
                event_type: "timeout".to_string(),
                keys: vec![format!("elapsed_ms:{}", elapsed_ms)],
                isolation_level: None,
            },
            TransactionEvent::Conflict { tx_id, key, reason } => TransactionLogEntry {
                timestamp: get_current_timestamp(),
                tx_id: tx_id.clone(),
                event_type: "conflict".to_string(),
                keys: vec![key.clone(), reason.clone()],
                isolation_level: None,
            },
            TransactionEvent::Deadlock { tx_id } => TransactionLogEntry {
                timestamp: get_current_timestamp(),
                tx_id: tx_id.clone(),
                event_type: "deadlock".to_string(),
                keys: vec![],
                isolation_level: None,
            },
        };

        if let Some(ref mut file) = self.file {
            // Write as line-delimited JSON
            serde_json::to_writer(&mut *file, &entry)
                .map_err(io::Error::other)?;
            file.write_all(b"\n")?;
            file.flush()?;
        }

        Ok(())
    }

    /// Close the log file (called automatically on drop)
    pub fn close(&mut self) -> io::Result<()> {
        if let Some(mut file) = self.file.take() {
            file.flush()?;
        }
        Ok(())
    }
}

impl Drop for TransactionLog {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Pluggable backend for transaction state. In production, implement with a DB, chain state, or durable key-value store.
pub trait StateStorage {
    fn get(&self, key: &str) -> Option<Value>;
    fn set(&mut self, key: &str, value: Value);

    /// Check if a key exists in storage
    fn contains_key(&self, key: &str) -> bool;

    /// Remove a key from storage, returning the previous value if it existed
    fn remove(&mut self, key: &str) -> Option<Value>;
}

/// In-memory storage (default). For production, replace with a persistent implementation.
#[derive(Default)]
pub struct InMemoryStorage {
    state: HashMap<String, Value>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
    /// Build storage with initial state (e.g. for tests or bootstrap).
    pub fn from_map(map: HashMap<String, Value>) -> Self {
        Self { state: map }
    }
}

impl StateStorage for InMemoryStorage {
    fn get(&self, key: &str) -> Option<Value> {
        self.state.get(key).cloned()
    }
    fn set(&mut self, key: &str, value: Value) {
        self.state.insert(key.to_string(), value);
    }
    fn contains_key(&self, key: &str) -> bool {
        self.state.contains_key(key)
    }
    fn remove(&mut self, key: &str) -> Option<Value> {
        self.state.remove(key)
    }
}

/// File-backed storage for persistent state across process restarts.
///
/// **Durability contract**: Each `set()` and `remove()` operation flushes to disk immediately.
/// Data survives process crashes and restarts. Storage is a JSON file containing the full state.
///
/// **Performance**: Slower than in-memory due to disk I/O on every write. Suitable for
/// single-node deployments with moderate write loads. For high-throughput, use batched writes
/// or consider a database-backed implementation.
///
/// **Format**: JSON file with structure: `{"key1": value1, "key2": value2, ...}`
pub struct FileBackedStorage {
    state: HashMap<String, Value>,
    file_path: PathBuf,
    auto_flush: bool, // If true, flush on every write (durability); if false, manual flush required
}

impl FileBackedStorage {
    /// Create or open a file-backed storage at the given path.
    /// Loads existing data if the file exists; creates new file if not.
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::with_auto_flush(path, true)
    }

    /// Create file-backed storage with configurable auto-flush.
    /// If `auto_flush` is false, caller must manually call `flush()` to persist changes.
    pub fn with_auto_flush<P: AsRef<Path>>(path: P, auto_flush: bool) -> io::Result<Self> {
        let file_path = path.as_ref().to_path_buf();

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load existing state or create empty
        let state = if file_path.exists() {
            Self::load_from_file(&file_path)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            state,
            file_path,
            auto_flush,
        })
    }

    /// Load state from JSON file
    fn load_from_file(path: &Path) -> io::Result<HashMap<String, Value>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // Parse JSON
        match serde_json::from_reader(reader) {
            Ok(state) => Ok(state),
            Err(e) => {
                // If file is empty or corrupt, start with empty state
                eprintln!("Warning: Failed to load transaction state from {:?}: {}. Starting with empty state.", path, e);
                Ok(HashMap::new())
            }
        }
    }

    /// Flush current state to disk
    pub fn flush(&self) -> io::Result<()> {
        // Write to temp file first, then atomic rename (safer)
        let temp_path = self.file_path.with_extension("tmp");

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;

        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.state)
            .map_err(io::Error::other)?;

        // Atomic rename (on Unix; on Windows this may fail if file is open)
        fs::rename(&temp_path, &self.file_path)?;

        Ok(())
    }
}

impl StateStorage for FileBackedStorage {
    fn get(&self, key: &str) -> Option<Value> {
        self.state.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: Value) {
        self.state.insert(key.to_string(), value);

        if self.auto_flush {
            if let Err(e) = self.flush() {
                eprintln!("Warning: Failed to flush transaction state to disk: {}", e);
            }
        }
    }

    fn contains_key(&self, key: &str) -> bool {
        self.state.contains_key(key)
    }

    fn remove(&mut self, key: &str) -> Option<Value> {
        let result = self.state.remove(key);

        if self.auto_flush && result.is_some() {
            if let Err(e) = self.flush() {
                eprintln!("Warning: Failed to flush transaction state to disk: {}", e);
            }
        }

        result
    }
}

impl Drop for FileBackedStorage {
    fn drop(&mut self) {
        // Ensure final flush on drop
        if let Err(e) = self.flush() {
            eprintln!("Warning: Failed to flush transaction state on drop: {}", e);
        }
    }
}

/// SQLite-backed storage for persistent state with database reliability.
///
/// **Durability contract**: Each `set()` and `remove()` operation commits immediately.
/// Provides ACID guarantees from SQLite. Data survives crashes and supports concurrent access.
///
/// **Performance**: Good; SQLite is optimized for single-writer workloads. Better than file
/// for high write loads. Supports WAL mode for improved concurrency.
///
/// **Format**: SQLite database with single table `kv_store(key TEXT PRIMARY KEY, value TEXT)`.
/// Values are JSON-encoded.
#[cfg(feature = "sqlite-storage")]
pub struct SqliteStorage {
    conn: rusqlite::Connection,
}

#[cfg(feature = "sqlite-storage")]
impl SqliteStorage {
    /// Create or open a SQLite database at the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = rusqlite::Connection::open(path)?;

        // Create table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Enable WAL mode for better concurrency (PRAGMA returns a result, use pragma_update)
        conn.pragma_update(None, "journal_mode", "WAL")?;

        Ok(Self { conn })
    }

    /// Create in-memory SQLite database (for testing)
    pub fn new_in_memory() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = rusqlite::Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }
}

#[cfg(feature = "sqlite-storage")]
impl StateStorage for SqliteStorage {
    fn get(&self, key: &str) -> Option<Value> {
        let result: Result<String, rusqlite::Error> =
            self.conn
                .query_row("SELECT value FROM kv_store WHERE key = ?1", [key], |row| {
                    row.get(0)
                });

        match result {
            Ok(json_str) => {
                // Deserialize JSON value
                serde_json::from_str(&json_str).ok()
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => {
                eprintln!("Warning: SQLite get error for key '{}': {}", key, e);
                None
            }
        }
    }

    fn set(&mut self, key: &str, value: Value) {
        // Serialize value to JSON
        let json_str = match serde_json::to_string(&value) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to serialize value for key '{}': {}",
                    key, e
                );
                return;
            }
        };

        // Insert or replace
        if let Err(e) = self.conn.execute(
            "INSERT OR REPLACE INTO kv_store (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, json_str],
        ) {
            eprintln!("Warning: SQLite set error for key '{}': {}", key, e);
        }
    }

    fn contains_key(&self, key: &str) -> bool {
        let result: Result<i64, rusqlite::Error> = self.conn.query_row(
            "SELECT COUNT(*) FROM kv_store WHERE key = ?1",
            [key],
            |row| row.get(0),
        );

        matches!(result, Ok(count) if count > 0)
    }

    fn remove(&mut self, key: &str) -> Option<Value> {
        // Get current value first
        let current = self.get(key);

        if current.is_some() {
            if let Err(e) = self
                .conn
                .execute("DELETE FROM kv_store WHERE key = ?1", [key])
            {
                eprintln!("Warning: SQLite remove error for key '{}': {}", key, e);
                return None;
            }
        }

        current
    }
}

/// Transaction errors
#[derive(Error, Debug, Clone)]
pub enum TransactionError {
    #[error("Transaction not found: {0}")]
    NotFound(String),

    #[error("Transaction already active")]
    AlreadyActive,

    #[error("No active transaction")]
    NoActiveTransaction,

    #[error("Transaction conflict detected")]
    Conflict,

    #[error("Deadlock detected")]
    Deadlock,

    #[error("Transaction timeout")]
    Timeout,

    #[error("Resource limit exceeded: {0}")]
    LimitExceeded(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
}

/// Transaction isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted, // Lowest isolation, highest performance
    ReadCommitted,   // Default for most databases
    RepeatableRead,  // Prevents non-repeatable reads
    Serializable,    // Highest isolation, lowest performance
}

/// Transaction state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionState {
    Active,
    Preparing, // Two-phase commit
    Committed,
    RolledBack,
    Failed,
}

/// Savepoint for partial rollback
#[derive(Debug, Clone)]
pub struct Savepoint {
    pub name: String,
    pub state_snapshot: HashMap<String, Value>,
    pub timestamp: u64,
}

/// Transaction metadata
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub state: TransactionState,
    pub isolation_level: IsolationLevel,
    pub start_time: u64,
    pub timeout_ms: Option<u64>,

    // State management
    pub original_state: HashMap<String, Value>,
    pub modified_state: HashMap<String, Value>,
    pub savepoints: Vec<Savepoint>,

    // Distributed transaction support
    pub participants: Vec<String>, // Participant IDs for 2PC
    pub is_distributed: bool,
}

/// Transaction manager. Uses pluggable [`StateStorage`]; default is in-memory.
pub struct TransactionManager {
    active_transactions: HashMap<String, Transaction>,
    transaction_counter: u64,
    storage: Box<dyn StateStorage>,
    read_locks: HashMap<String, Vec<String>>, // key -> [transaction_ids]
    write_locks: HashMap<String, String>,     // key -> transaction_id
    default_timeout_ms: Option<u64>,          // Default timeout for new transactions
    event_callback: Option<TransactionEventCallback>, // Optional lifecycle event observer
    transaction_log: Option<TransactionLog>,  // Optional persistent audit log

    // Resource limits (Phase 4: Production Hardening)
    max_active_transactions: usize, // Max concurrent transactions (0 = unlimited)
    max_keys_per_transaction: usize, // Max keys modified per transaction (0 = unlimited)
}

impl Transaction {
    pub fn new(id: String, isolation_level: IsolationLevel) -> Self {
        Self {
            id,
            state: TransactionState::Active,
            isolation_level,
            start_time: get_current_timestamp(),
            timeout_ms: Some(30000), // Default 30 second timeout
            original_state: HashMap::new(),
            modified_state: HashMap::new(),
            savepoints: Vec::new(),
            participants: Vec::new(),
            is_distributed: false,
        }
    }

    /// Check if transaction has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout_ms {
            let elapsed = get_current_timestamp() - self.start_time;
            elapsed > timeout
        } else {
            false
        }
    }

    /// Create a savepoint
    pub fn create_savepoint(&mut self, name: String) {
        let savepoint = Savepoint {
            name,
            state_snapshot: self.modified_state.clone(),
            timestamp: get_current_timestamp(),
        };
        self.savepoints.push(savepoint);
    }

    /// Rollback to a savepoint
    pub fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), TransactionError> {
        if let Some(pos) = self.savepoints.iter().position(|sp| sp.name == name) {
            let savepoint = &self.savepoints[pos];
            self.modified_state = savepoint.state_snapshot.clone();

            // Remove savepoints created after this one
            self.savepoints.truncate(pos + 1);

            Ok(())
        } else {
            Err(TransactionError::NotFound(format!(
                "Savepoint '{}' not found",
                name
            )))
        }
    }
}

impl TransactionManager {
    pub fn new() -> Self {
        Self::with_storage(Box::new(InMemoryStorage::new()))
    }

    /// Create TransactionManager from environment configuration.
    ///
    /// **Environment variables**:
    /// - `DAL_TX_STORAGE`: Storage backend type (`memory`, `file`, `sqlite`) - default: `memory`
    /// - `DAL_TX_STORAGE_PATH`: File path for file/sqlite backend - default: `./dal_tx_state.json` or `./dal_tx_state.db`
    /// - `DAL_TX_LOG_PATH`: Optional transaction log path for audit trail
    /// - `DAL_TX_TIMEOUT_MS`: Default transaction timeout in milliseconds - default: `30000`
    /// - `DAL_TX_MAX_ACTIVE`: Maximum concurrent active transactions - default: `1000`
    /// - `DAL_TX_MAX_KEYS`: Maximum keys modified per transaction - default: `10000`
    ///
    /// **Example**:
    /// ```bash
    /// export DAL_TX_STORAGE=sqlite
    /// export DAL_TX_STORAGE_PATH=/var/lib/dal/transactions.db
    /// export DAL_TX_LOG_PATH=/var/log/dal/transactions.log
    /// export DAL_TX_TIMEOUT_MS=60000
    /// export DAL_TX_MAX_ACTIVE=500
    /// export DAL_TX_MAX_KEYS=5000
    /// ```
    pub fn from_env() -> io::Result<Self> {
        let storage_type = std::env::var("DAL_TX_STORAGE").unwrap_or_else(|_| "memory".to_string());
        let log_path = std::env::var("DAL_TX_LOG_PATH").ok();
        let timeout_ms = std::env::var("DAL_TX_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());

        // Select storage backend
        let storage: Box<dyn StateStorage> = match storage_type.as_str() {
            "file" => {
                let storage_path = std::env::var("DAL_TX_STORAGE_PATH")
                    .unwrap_or_else(|_| "./dal_tx_state.json".to_string());
                Box::new(FileBackedStorage::new(storage_path)?)
            }
            #[cfg(feature = "sqlite-storage")]
            "sqlite" => {
                let storage_path = std::env::var("DAL_TX_STORAGE_PATH")
                    .unwrap_or_else(|_| "./dal_tx_state.db".to_string());
                Box::new(SqliteStorage::new(storage_path).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, format!("SQLite error: {}", e))
                })?)
            }
            #[cfg(not(feature = "sqlite-storage"))]
            "sqlite" => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "SQLite backend requires 'sqlite-storage' feature. Compile with: cargo build --features sqlite-storage"
                ));
            }
            "memory" | _ => Box::new(InMemoryStorage::new()),
        };

        let mut manager = Self::with_storage(storage);

        // Set timeout if provided
        if let Some(timeout) = timeout_ms {
            manager = manager.with_default_timeout(Some(timeout));
        }

        // Enable transaction log if path provided
        if let Some(log) = log_path {
            manager = manager.with_transaction_log(log)?;
        }

        // Phase 4: Set resource limits from environment
        if let Ok(max_active) = std::env::var("DAL_TX_MAX_ACTIVE") {
            if let Ok(limit) = max_active.parse::<usize>() {
                manager = manager.with_max_active_transactions(limit);
            }
        }

        if let Ok(max_keys) = std::env::var("DAL_TX_MAX_KEYS") {
            if let Ok(limit) = max_keys.parse::<usize>() {
                manager = manager.with_max_keys_per_transaction(limit);
            }
        }

        Ok(manager)
    }

    /// Use a custom storage backend (DB, chain state, etc.).
    pub fn with_storage(storage: Box<dyn StateStorage>) -> Self {
        Self {
            active_transactions: HashMap::new(),
            transaction_counter: 0,
            storage,
            read_locks: HashMap::new(),
            write_locks: HashMap::new(),
            default_timeout_ms: Some(30000), // Default: 30 seconds
            event_callback: None,
            transaction_log: None,
            max_active_transactions: 1000, // Default: 1000 concurrent transactions
            max_keys_per_transaction: 10000, // Default: 10,000 keys per transaction
        }
    }

    /// Set default timeout for new transactions (in milliseconds). None = no timeout.
    pub fn with_default_timeout(mut self, timeout_ms: Option<u64>) -> Self {
        self.default_timeout_ms = timeout_ms;
        self
    }

    /// Set maximum number of active transactions (0 = unlimited, not recommended).
    pub fn with_max_active_transactions(mut self, max: usize) -> Self {
        self.max_active_transactions = max;
        self
    }

    /// Set maximum number of keys per transaction (0 = unlimited, not recommended).
    pub fn with_max_keys_per_transaction(mut self, max: usize) -> Self {
        self.max_keys_per_transaction = max;
        self
    }

    /// Set an event callback for transaction lifecycle observability.
    pub fn with_event_callback(mut self, callback: TransactionEventCallback) -> Self {
        self.event_callback = Some(callback);
        self
    }

    /// Enable persistent transaction logging to the given file path.
    /// The log is an append-only, line-delimited JSON file for audit and recovery.
    pub fn with_transaction_log<P: AsRef<Path>>(mut self, path: P) -> io::Result<Self> {
        self.transaction_log = Some(TransactionLog::new(path)?);
        Ok(self)
    }

    /// Emit a transaction event if a callback is registered and/or log to persistent log.
    fn emit_event(&mut self, event: TransactionEvent) {
        // Call event callback if registered
        if let Some(ref callback) = self.event_callback {
            callback(&event);
        }

        // Write to transaction log if enabled
        if let Some(ref mut log) = self.transaction_log {
            if let Err(e) = log.log_event(&event) {
                eprintln!("Warning: Failed to write to transaction log: {}", e);
            }
        }
    }

    /// Read committed value for a key (for tests or inspection).
    pub fn get_committed(&self, key: &str) -> Option<Value> {
        self.storage.get(key)
    }

    /// Access active transaction by id (for tests).
    pub fn get_transaction(&self, tx_id: &str) -> Option<&Transaction> {
        self.active_transactions.get(tx_id)
    }

    /// Set timeout for an active transaction (for tests or tuning).
    pub fn set_transaction_timeout(
        &mut self,
        tx_id: &str,
        timeout_ms: Option<u64>,
    ) -> Result<(), TransactionError> {
        let tx = self
            .active_transactions
            .get_mut(tx_id)
            .ok_or_else(|| TransactionError::NotFound(tx_id.to_string()))?;
        tx.timeout_ms = timeout_ms;
        Ok(())
    }

    /// Begin a new transaction
    pub fn begin_transaction(
        &mut self,
        isolation_level: IsolationLevel,
    ) -> Result<String, TransactionError> {
        // Phase 4: Enforce max active transactions limit
        if self.max_active_transactions > 0
            && self.active_transactions.len() >= self.max_active_transactions
        {
            return Err(TransactionError::LimitExceeded(format!(
                "Maximum active transactions limit reached ({}/{})",
                self.active_transactions.len(),
                self.max_active_transactions
            )));
        }

        self.transaction_counter += 1;
        let tx_id = format!("tx_{}", self.transaction_counter);

        let mut transaction = Transaction::new(tx_id.clone(), isolation_level);
        transaction.timeout_ms = self.default_timeout_ms; // Use manager's default

        self.emit_event(TransactionEvent::Begin {
            tx_id: tx_id.clone(),
            isolation_level,
        });

        self.active_transactions.insert(tx_id.clone(), transaction);

        Ok(tx_id)
    }

    /// Read a value within a transaction
    pub fn read(&mut self, tx_id: &str, key: &str) -> Result<Option<Value>, TransactionError> {
        // First, check transaction state and isolation level
        let (should_lock, is_timed_out, modified_value) = {
            let tx = self
                .active_transactions
                .get(tx_id)
                .ok_or_else(|| TransactionError::NotFound(tx_id.to_string()))?;

            if tx.state != TransactionState::Active {
                return Err(TransactionError::NoActiveTransaction);
            }

            let should_lock = tx.isolation_level != IsolationLevel::ReadUncommitted;
            let is_timed_out = tx.is_timed_out();
            let modified_value = tx.modified_state.get(key).cloned();

            (should_lock, is_timed_out, modified_value)
        };

        // Check timeout
        if is_timed_out {
            self.emit_event(TransactionEvent::Timeout {
                tx_id: tx_id.to_string(),
                elapsed_ms: get_current_timestamp() - self.active_transactions[tx_id].start_time,
            });
            return Err(TransactionError::Timeout);
        }

        // Acquire read lock based on isolation level
        if should_lock {
            self.acquire_read_lock(tx_id, key)?;
        }

        self.emit_event(TransactionEvent::Read {
            tx_id: tx_id.to_string(),
            key: key.to_string(),
        });

        // Check modified state first, then storage
        if let Some(value) = modified_value {
            return Ok(Some(value));
        }

        Ok(self.storage.get(key))
    }

    /// Write a value within a transaction
    pub fn write(
        &mut self,
        tx_id: &str,
        key: String,
        value: Value,
    ) -> Result<(), TransactionError> {
        // Acquire write lock
        self.acquire_write_lock(tx_id, &key)?;

        // Emit event before mutable borrow
        self.emit_event(TransactionEvent::Write {
            tx_id: tx_id.to_string(),
            key: key.clone(),
        });

        let tx = self
            .active_transactions
            .get_mut(tx_id)
            .ok_or_else(|| TransactionError::NotFound(tx_id.to_string()))?;

        if tx.state != TransactionState::Active {
            return Err(TransactionError::NoActiveTransaction);
        }

        // Phase 4: Enforce max keys per transaction limit
        if self.max_keys_per_transaction > 0
            && !tx.modified_state.contains_key(&key)
            && tx.modified_state.len() >= self.max_keys_per_transaction
        {
            return Err(TransactionError::LimitExceeded(format!(
                "Maximum keys per transaction limit reached ({}/{})",
                tx.modified_state.len(),
                self.max_keys_per_transaction
            )));
        }

        // Save original value if not already saved
        if !tx.original_state.contains_key(&key) {
            if let Some(original) = self.storage.get(&key) {
                tx.original_state.insert(key.clone(), original);
            }
        }

        // Write to transaction's modified state
        tx.modified_state.insert(key, value);

        Ok(())
    }

    /// Commit a transaction
    pub fn commit(&mut self, tx_id: &str) -> Result<(), TransactionError> {
        let keys_modified = {
            let tx = self
                .active_transactions
                .get(tx_id)
                .ok_or_else(|| TransactionError::NotFound(tx_id.to_string()))?;

            if tx.state != TransactionState::Active {
                return Err(TransactionError::NoActiveTransaction);
            }

            // Check timeout
            if tx.is_timed_out() {
                let elapsed_ms = get_current_timestamp() - tx.start_time;
                self.emit_event(TransactionEvent::Timeout {
                    tx_id: tx_id.to_string(),
                    elapsed_ms,
                });
                self.rollback(tx_id)?;
                return Err(TransactionError::Timeout);
            }

            // For distributed transactions, use two-phase commit
            if tx.is_distributed {
                return self.two_phase_commit(tx_id);
            }

            tx.modified_state.len()
        };

        let tx = self.active_transactions.get_mut(tx_id).unwrap();

        // Apply all modifications to storage
        for (key, value) in &tx.modified_state {
            self.storage.set(key, value.clone());
        }

        // Update state
        tx.state = TransactionState::Committed;

        self.emit_event(TransactionEvent::Commit {
            tx_id: tx_id.to_string(),
            keys_modified,
        });

        // Release locks
        self.release_locks(tx_id);

        // Remove transaction
        self.active_transactions.remove(tx_id);

        Ok(())
    }

    /// Rollback a transaction
    pub fn rollback(&mut self, tx_id: &str) -> Result<(), TransactionError> {
        let tx = self
            .active_transactions
            .get_mut(tx_id)
            .ok_or_else(|| TransactionError::NotFound(tx_id.to_string()))?;

        // Restore original state (if any changes were made to global state)
        // In this implementation, changes are buffered, so no restoration needed

        // Update state
        tx.state = TransactionState::RolledBack;

        self.emit_event(TransactionEvent::Rollback {
            tx_id: tx_id.to_string(),
        });

        // Release locks
        self.release_locks(tx_id);

        // Remove transaction
        self.active_transactions.remove(tx_id);

        Ok(())
    }

    /// Two-phase commit for distributed transactions
    fn two_phase_commit(&mut self, tx_id: &str) -> Result<(), TransactionError> {
        let tx = self
            .active_transactions
            .get_mut(tx_id)
            .ok_or_else(|| TransactionError::NotFound(tx_id.to_string()))?;

        // Phase 1: Prepare
        tx.state = TransactionState::Preparing;

        // In production, would send prepare messages to all participants
        // For now, simulate immediate success

        // Phase 2: Commit
        for (key, value) in &tx.modified_state {
            self.storage.set(key, value.clone());
        }

        tx.state = TransactionState::Committed;

        // Release locks
        self.release_locks(tx_id);

        // Remove transaction
        self.active_transactions.remove(tx_id);

        Ok(())
    }

    /// Acquire read lock
    fn acquire_read_lock(&mut self, tx_id: &str, key: &str) -> Result<(), TransactionError> {
        // Deadlock detection: check if transaction has timed out (timeout-based deadlock detection)
        if let Some(tx) = self.active_transactions.get(tx_id) {
            if tx.is_timed_out() {
                self.emit_event(TransactionEvent::Deadlock {
                    tx_id: tx_id.to_string(),
                });
                return Err(TransactionError::Deadlock);
            }
        }

        // Check for write lock by another transaction
        if let Some(write_owner) = self.write_locks.get(key) {
            if write_owner != tx_id {
                self.emit_event(TransactionEvent::Conflict {
                    tx_id: tx_id.to_string(),
                    key: key.to_string(),
                    reason: format!("Read blocked by write lock from {}", write_owner),
                });
                return Err(TransactionError::Conflict);
            }
        }

        // Add read lock
        self.read_locks
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(tx_id.to_string());

        Ok(())
    }

    /// Acquire write lock
    fn acquire_write_lock(&mut self, tx_id: &str, key: &str) -> Result<(), TransactionError> {
        // Deadlock detection: check if transaction has timed out (timeout-based deadlock detection)
        if let Some(tx) = self.active_transactions.get(tx_id) {
            if tx.is_timed_out() {
                self.emit_event(TransactionEvent::Deadlock {
                    tx_id: tx_id.to_string(),
                });
                return Err(TransactionError::Deadlock);
            }
        }

        // Check for existing write lock by another transaction
        if let Some(write_owner) = self.write_locks.get(key) {
            if write_owner != tx_id {
                self.emit_event(TransactionEvent::Conflict {
                    tx_id: tx_id.to_string(),
                    key: key.to_string(),
                    reason: format!("Write blocked by write lock from {}", write_owner),
                });
                return Err(TransactionError::Conflict);
            }
        }

        // Check for read locks by other transactions
        if let Some(readers) = self.read_locks.get(key) {
            if readers.iter().any(|r| r != tx_id) {
                self.emit_event(TransactionEvent::Conflict {
                    tx_id: tx_id.to_string(),
                    key: key.to_string(),
                    reason: format!("Write blocked by {} read lock(s)", readers.len()),
                });
                return Err(TransactionError::Conflict);
            }
        }

        // Acquire write lock
        self.write_locks.insert(key.to_string(), tx_id.to_string());

        Ok(())
    }

    /// Release all locks held by a transaction
    fn release_locks(&mut self, tx_id: &str) {
        // Release read locks
        self.read_locks.retain(|_, readers| {
            readers.retain(|r| r != tx_id);
            !readers.is_empty()
        });

        // Release write locks
        self.write_locks.retain(|_, owner| owner != tx_id);
    }

    /// Create a savepoint within a transaction
    pub fn create_savepoint(&mut self, tx_id: &str, name: String) -> Result<(), TransactionError> {
        let tx = self
            .active_transactions
            .get_mut(tx_id)
            .ok_or_else(|| TransactionError::NotFound(tx_id.to_string()))?;

        tx.create_savepoint(name.clone());

        self.emit_event(TransactionEvent::SavepointCreated {
            tx_id: tx_id.to_string(),
            savepoint_name: name,
        });

        Ok(())
    }

    /// Rollback to a savepoint
    pub fn rollback_to_savepoint(
        &mut self,
        tx_id: &str,
        name: &str,
    ) -> Result<(), TransactionError> {
        let tx = self
            .active_transactions
            .get_mut(tx_id)
            .ok_or_else(|| TransactionError::NotFound(tx_id.to_string()))?;

        let result = tx.rollback_to_savepoint(name);

        if result.is_ok() {
            self.emit_event(TransactionEvent::SavepointRolledBack {
                tx_id: tx_id.to_string(),
                savepoint_name: name.to_string(),
            });
        }

        result
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp in milliseconds
fn get_current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_begin_commit() {
        let mut manager = TransactionManager::new();

        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        assert!(manager.active_transactions.contains_key(&tx_id));

        manager
            .write(&tx_id, "key1".to_string(), Value::Int(42))
            .unwrap();
        manager.commit(&tx_id).unwrap();

        assert!(!manager.get_transaction(&tx_id).is_some());
        assert_eq!(manager.get_committed("key1"), Some(Value::Int(42)));
    }

    #[test]
    fn test_transaction_rollback() {
        let mut manager =
            TransactionManager::with_storage(Box::new(InMemoryStorage::from_map(HashMap::from([
                ("key1".to_string(), Value::Int(10)),
            ]))));

        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        manager
            .write(&tx_id, "key1".to_string(), Value::Int(42))
            .unwrap();
        manager.rollback(&tx_id).unwrap();

        assert_eq!(manager.get_committed("key1"), Some(Value::Int(10)));
    }

    #[test]
    fn test_savepoint_rollback() {
        let mut manager = TransactionManager::new();

        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();

        manager
            .write(&tx_id, "key1".to_string(), Value::Int(1))
            .unwrap();
        manager.create_savepoint(&tx_id, "sp1".to_string()).unwrap();

        manager
            .write(&tx_id, "key1".to_string(), Value::Int(2))
            .unwrap();
        manager.rollback_to_savepoint(&tx_id, "sp1").unwrap();

        let tx = manager.get_transaction(&tx_id).unwrap();
        assert_eq!(tx.modified_state.get("key1"), Some(&Value::Int(1)));
    }

    #[test]
    fn test_isolation_read_committed() {
        let mut manager =
            TransactionManager::with_storage(Box::new(InMemoryStorage::from_map(HashMap::from([
                ("counter".to_string(), Value::Int(0)),
            ]))));

        let tx1 = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();

        // tx1 writes
        manager
            .write(&tx1, "counter".to_string(), Value::Int(1))
            .unwrap();

        // Start tx2 after tx1 has acquired write lock
        let tx2 = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();

        // tx2 trying to read a key with active write lock from tx1 will conflict
        // This is correct behavior for isolation - we can't read while another tx has write lock
        let _read_result = manager.read(&tx2, "counter");
        // For now, this will error due to conflict - which is safe behavior

        // Commit tx1
        manager.commit(&tx1).unwrap();

        // Now tx2 should be able to read the committed value
        let value = manager.read(&tx2, "counter").unwrap();
        assert_eq!(value, Some(Value::Int(1)));

        manager.commit(&tx2).unwrap();
    }

    #[test]
    fn test_write_conflict() {
        let mut manager = TransactionManager::new();

        let tx1 = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        let tx2 = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();

        // tx1 acquires write lock
        manager
            .write(&tx1, "key1".to_string(), Value::Int(1))
            .unwrap();

        // tx2 should fail to acquire write lock on same key
        let result = manager.write(&tx2, "key1".to_string(), Value::Int(2));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransactionError::Conflict));
    }

    #[test]
    fn test_transaction_timeout() {
        let mut manager = TransactionManager::new();
        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        manager.set_transaction_timeout(&tx_id, Some(1)).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = manager.commit(&tx_id);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransactionError::Timeout));
    }

    // ===== Phase 1 Feature Tests =====

    #[test]
    fn test_state_storage_contains_key() {
        let mut storage = InMemoryStorage::new();
        assert!(!storage.contains_key("key1"));

        storage.set("key1", Value::Int(42));
        assert!(storage.contains_key("key1"));
        assert!(!storage.contains_key("key2"));
    }

    #[test]
    fn test_state_storage_remove() {
        let mut storage = InMemoryStorage::new();
        storage.set("key1", Value::Int(42));
        storage.set("key2", Value::String("hello".to_string()));

        let removed = storage.remove("key1");
        assert_eq!(removed, Some(Value::Int(42)));
        assert!(!storage.contains_key("key1"));
        assert!(storage.contains_key("key2"));

        let not_found = storage.remove("key3");
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_configurable_default_timeout() {
        let mut manager = TransactionManager::new().with_default_timeout(Some(5000)); // 5 seconds

        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        let tx = manager.get_transaction(&tx_id).unwrap();
        assert_eq!(tx.timeout_ms, Some(5000));
    }

    #[test]
    fn test_no_default_timeout() {
        let mut manager = TransactionManager::new().with_default_timeout(None); // No timeout

        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        let tx = manager.get_transaction(&tx_id).unwrap();
        assert_eq!(tx.timeout_ms, None);
        assert!(!tx.is_timed_out());
    }

    #[test]
    fn test_transaction_event_callback() {
        use std::sync::{Arc, Mutex};

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let mut manager = TransactionManager::new().with_event_callback(Box::new(move |event| {
            events_clone.lock().unwrap().push(format!("{:?}", event));
        }));

        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        manager
            .write(&tx_id, "key1".to_string(), Value::Int(42))
            .unwrap();
        manager.commit(&tx_id).unwrap();

        let captured_events = events.lock().unwrap();
        assert!(captured_events.len() >= 3); // At least: Begin, Write, Commit

        // Check that Begin event was captured
        assert!(captured_events.iter().any(|e| e.contains("Begin")));
        // Check that Write event was captured
        assert!(captured_events.iter().any(|e| e.contains("Write")));
        // Check that Commit event was captured
        assert!(captured_events.iter().any(|e| e.contains("Commit")));
    }

    #[test]
    fn test_savepoint_events() {
        use std::sync::{Arc, Mutex};

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let mut manager = TransactionManager::new().with_event_callback(Box::new(move |event| {
            events_clone.lock().unwrap().push(format!("{:?}", event));
        }));

        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        manager
            .write(&tx_id, "key1".to_string(), Value::Int(1))
            .unwrap();
        manager.create_savepoint(&tx_id, "sp1".to_string()).unwrap();
        manager
            .write(&tx_id, "key1".to_string(), Value::Int(2))
            .unwrap();
        manager.rollback_to_savepoint(&tx_id, "sp1").unwrap();
        manager.commit(&tx_id).unwrap();

        let captured_events = events.lock().unwrap();
        assert!(captured_events
            .iter()
            .any(|e| e.contains("SavepointCreated")));
        assert!(captured_events
            .iter()
            .any(|e| e.contains("SavepointRolledBack")));
    }

    #[test]
    fn test_conflict_event() {
        use std::sync::{Arc, Mutex};

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let mut manager = TransactionManager::new().with_event_callback(Box::new(move |event| {
            events_clone.lock().unwrap().push(format!("{:?}", event));
        }));

        let tx1 = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        let tx2 = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();

        // tx1 acquires write lock
        manager
            .write(&tx1, "key1".to_string(), Value::Int(1))
            .unwrap();

        // tx2 tries to acquire write lock on same key - should conflict
        let result = manager.write(&tx2, "key1".to_string(), Value::Int(2));
        assert!(result.is_err());

        let captured_events = events.lock().unwrap();
        assert!(captured_events.iter().any(|e| e.contains("Conflict")));
    }

    #[test]
    fn test_deadlock_detection_timeout() {
        use std::sync::{Arc, Mutex};

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let mut manager = TransactionManager::new()
            .with_default_timeout(Some(1)) // 1ms timeout
            .with_event_callback(Box::new(move |event| {
                events_clone.lock().unwrap().push(format!("{:?}", event));
            }));

        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();

        // Sleep to exceed timeout
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Try to acquire lock - should detect deadlock (timeout)
        let result = manager.write(&tx_id, "key1".to_string(), Value::Int(1));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransactionError::Deadlock));

        let captured_events = events.lock().unwrap();
        assert!(captured_events.iter().any(|e| e.contains("Deadlock")));
    }

    #[test]
    fn test_rollback_event() {
        use std::sync::{Arc, Mutex};

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let mut manager = TransactionManager::new().with_event_callback(Box::new(move |event| {
            events_clone.lock().unwrap().push(format!("{:?}", event));
        }));

        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        manager
            .write(&tx_id, "key1".to_string(), Value::Int(42))
            .unwrap();
        manager.rollback(&tx_id).unwrap();

        let captured_events = events.lock().unwrap();
        assert!(captured_events.iter().any(|e| e.contains("Rollback")));
    }

    // ===== Phase 2 Feature Tests =====

    #[test]
    fn test_file_backed_storage_persistence() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("test_state.json");

        // Create storage, write data, drop
        {
            let mut storage = FileBackedStorage::new(&storage_path).unwrap();
            storage.set("key1", Value::Int(42));
            storage.set("key2", Value::String("hello".to_string()));
            assert_eq!(storage.get("key1"), Some(Value::Int(42)));
        } // storage dropped, should flush

        // Reload storage from same file - data should persist
        {
            let storage = FileBackedStorage::new(&storage_path).unwrap();
            assert_eq!(storage.get("key1"), Some(Value::Int(42)));
            assert_eq!(
                storage.get("key2"),
                Some(Value::String("hello".to_string()))
            );
        }
    }

    #[test]
    fn test_file_backed_storage_remove_persists() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("test_remove.json");

        {
            let mut storage = FileBackedStorage::new(&storage_path).unwrap();
            storage.set("key1", Value::Int(1));
            storage.set("key2", Value::Int(2));
            storage.remove("key1");
        }

        {
            let storage = FileBackedStorage::new(&storage_path).unwrap();
            assert_eq!(storage.get("key1"), None);
            assert_eq!(storage.get("key2"), Some(Value::Int(2)));
        }
    }

    #[test]
    fn test_transaction_manager_with_file_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("tx_state.json");

        // Create manager with file storage, commit transaction, drop
        {
            let mut manager = TransactionManager::with_storage(Box::new(
                FileBackedStorage::new(&storage_path).unwrap(),
            ));

            let tx_id = manager
                .begin_transaction(IsolationLevel::ReadCommitted)
                .unwrap();
            manager
                .write(&tx_id, "balance".to_string(), Value::Int(1000))
                .unwrap();
            manager.commit(&tx_id).unwrap();
        }

        // Reload manager with same file - committed data should persist
        {
            let manager = TransactionManager::with_storage(Box::new(
                FileBackedStorage::new(&storage_path).unwrap(),
            ));

            assert_eq!(manager.get_committed("balance"), Some(Value::Int(1000)));
        }
    }

    #[test]
    fn test_transaction_log_writes_events() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("tx.log");

        {
            let mut manager = TransactionManager::new()
                .with_transaction_log(&log_path)
                .unwrap();

            let tx_id = manager
                .begin_transaction(IsolationLevel::ReadCommitted)
                .unwrap();
            manager
                .write(&tx_id, "key1".to_string(), Value::Int(42))
                .unwrap();
            manager.commit(&tx_id).unwrap();
        }

        // Read log file and verify events were written
        let log_contents = fs::read_to_string(&log_path).unwrap();
        assert!(log_contents.contains("begin"));
        assert!(log_contents.contains("write"));
        assert!(log_contents.contains("commit"));

        // Verify it's line-delimited JSON
        let lines: Vec<&str> = log_contents.lines().collect();
        assert!(lines.len() >= 3); // At least begin, write, commit

        // Verify each line is valid JSON
        for line in lines {
            if !line.trim().is_empty() {
                let parsed: Result<TransactionLogEntry, _> = serde_json::from_str(line);
                assert!(parsed.is_ok(), "Log line should be valid JSON: {}", line);
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_from_env_memory_backend() {
        // Test default (memory) backend when no env vars set
        std::env::remove_var("DAL_TX_STORAGE");

        let manager = TransactionManager::from_env().unwrap();
        // Should work - verify with a simple transaction
        let mut manager = manager;
        let tx_id = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        manager
            .write(&tx_id, "test".to_string(), Value::Int(1))
            .unwrap();
        manager.commit(&tx_id).unwrap();
        assert_eq!(manager.get_committed("test"), Some(Value::Int(1)));

        // Cleanup
        std::env::remove_var("DAL_TX_STORAGE");
        std::env::remove_var("DAL_TX_STORAGE_PATH");
        std::env::remove_var("DAL_TX_LOG_PATH");
        std::env::remove_var("DAL_TX_TIMEOUT_MS");
    }

    #[test]
    #[serial_test::serial]
    fn test_from_env_file_backend() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("env_test.json");

        std::env::set_var("DAL_TX_STORAGE", "file");
        std::env::set_var("DAL_TX_STORAGE_PATH", storage_path.to_str().unwrap());

        {
            let mut manager = TransactionManager::from_env().unwrap();
            let tx_id = manager
                .begin_transaction(IsolationLevel::ReadCommitted)
                .unwrap();
            manager
                .write(&tx_id, "persisted".to_string(), Value::Int(999))
                .unwrap();
            manager.commit(&tx_id).unwrap();
        }

        // Reload with same env - should persist
        {
            let manager = TransactionManager::from_env().unwrap();
            assert_eq!(manager.get_committed("persisted"), Some(Value::Int(999)));
        }

        // Cleanup
        std::env::remove_var("DAL_TX_STORAGE");
        std::env::remove_var("DAL_TX_STORAGE_PATH");
        std::env::remove_var("DAL_TX_LOG_PATH");
        std::env::remove_var("DAL_TX_TIMEOUT_MS");
    }

    #[test]
    fn test_file_backed_storage_concurrent_transactions() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("concurrent.json");

        let mut manager = TransactionManager::with_storage(Box::new(
            FileBackedStorage::new(&storage_path).unwrap(),
        ));

        // Multiple transactions should work with file backend
        let tx1 = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        manager
            .write(&tx1, "account1".to_string(), Value::Int(100))
            .unwrap();
        manager.commit(&tx1).unwrap();

        let tx2 = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        manager
            .write(&tx2, "account2".to_string(), Value::Int(200))
            .unwrap();
        manager.commit(&tx2).unwrap();

        assert_eq!(manager.get_committed("account1"), Some(Value::Int(100)));
        assert_eq!(manager.get_committed("account2"), Some(Value::Int(200)));
    }

    // ===== Phase 4: Resource Limits Tests =====

    #[test]
    fn test_max_active_transactions_limit() {
        let mut manager = TransactionManager::new().with_max_active_transactions(3); // Only allow 3 concurrent transactions

        // Create 3 transactions (should succeed)
        let tx1 = manager
            .begin_transaction(IsolationLevel::Serializable)
            .unwrap();
        let tx2 = manager
            .begin_transaction(IsolationLevel::Serializable)
            .unwrap();
        let tx3 = manager
            .begin_transaction(IsolationLevel::Serializable)
            .unwrap();

        // Attempt 4th transaction (should fail)
        let result = manager.begin_transaction(IsolationLevel::Serializable);
        assert!(result.is_err(), "Should reject 4th transaction");
        assert!(result.unwrap_err().to_string().contains("limit"));

        // Commit one transaction
        manager.commit(&tx1).unwrap();

        // Now we should be able to start another
        let tx4 = manager
            .begin_transaction(IsolationLevel::Serializable)
            .unwrap();
        assert!(tx4.starts_with("tx_"));

        // Clean up
        manager.rollback(&tx2).unwrap();
        manager.rollback(&tx3).unwrap();
        manager.rollback(&tx4).unwrap();
    }

    #[test]
    fn test_max_keys_per_transaction_limit() {
        let mut manager = TransactionManager::new().with_max_keys_per_transaction(5); // Only allow 5 keys per transaction

        let tx_id = manager
            .begin_transaction(IsolationLevel::Serializable)
            .unwrap();

        // Write 5 keys (should succeed)
        for i in 0..5 {
            manager
                .write(&tx_id, format!("key_{}", i), Value::Int(i))
                .unwrap();
        }

        // Attempt 6th key (should fail)
        let result = manager.write(&tx_id, "key_6".to_string(), Value::Int(6));
        assert!(result.is_err(), "Should reject 6th key");
        assert!(result.unwrap_err().to_string().contains("limit"));

        // Updating an existing key should still work (doesn't count toward limit)
        manager
            .write(&tx_id, "key_0".to_string(), Value::Int(100))
            .unwrap();

        manager.commit(&tx_id).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn test_resource_limits_from_env() {
        std::env::set_var("DAL_TX_STORAGE", "memory");
        std::env::set_var("DAL_TX_MAX_ACTIVE", "10");
        std::env::set_var("DAL_TX_MAX_KEYS", "100");

        let mut manager = TransactionManager::from_env().unwrap();

        // Create 10 transactions
        let mut txs = vec![];
        for _ in 0..10 {
            txs.push(
                manager
                    .begin_transaction(IsolationLevel::Serializable)
                    .unwrap(),
            );
        }

        // 11th should fail
        let result = manager.begin_transaction(IsolationLevel::Serializable);
        assert!(result.is_err(), "Should respect DAL_TX_MAX_ACTIVE limit");

        // Clean up
        for tx in txs {
            let _ = manager.rollback(&tx);
        }
        std::env::remove_var("DAL_TX_STORAGE");
        std::env::remove_var("DAL_TX_MAX_ACTIVE");
        std::env::remove_var("DAL_TX_MAX_KEYS");
    }

    #[test]
    fn test_unlimited_resources() {
        // Set limits to 0 (unlimited)
        let mut manager = TransactionManager::new()
            .with_max_active_transactions(0)
            .with_max_keys_per_transaction(0);

        // Should be able to create many transactions
        let mut txs = vec![];
        for _ in 0..100 {
            txs.push(
                manager
                    .begin_transaction(IsolationLevel::Serializable)
                    .unwrap(),
            );
        }

        // And write many keys in one transaction
        let tx_id = manager
            .begin_transaction(IsolationLevel::Serializable)
            .unwrap();
        for i in 0..100 {
            manager
                .write(&tx_id, format!("key_{}", i), Value::Int(i))
                .unwrap();
        }

        manager.commit(&tx_id).unwrap();

        // Clean up
        for tx in txs {
            let _ = manager.rollback(&tx);
        }
    }

    // ===== SQLite Backend Tests =====

    #[cfg(feature = "sqlite-storage")]
    #[test]
    fn test_sqlite_storage_basic_operations() {
        let mut storage = SqliteStorage::new_in_memory().unwrap();

        // Set and get
        storage.set("key1", Value::Int(42));
        assert_eq!(storage.get("key1"), Some(Value::Int(42)));

        // Contains
        assert!(storage.contains_key("key1"));
        assert!(!storage.contains_key("key2"));

        // Remove
        let removed = storage.remove("key1");
        assert_eq!(removed, Some(Value::Int(42)));
        assert!(!storage.contains_key("key1"));
    }

    #[cfg(feature = "sqlite-storage")]
    #[test]
    fn test_sqlite_storage_persistence() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create storage, write data, drop
        {
            let mut storage = SqliteStorage::new(&db_path).unwrap();
            storage.set("key1", Value::Int(42));
            storage.set("key2", Value::String("hello".to_string()));
        }

        // Reload storage from same DB - data should persist
        {
            let storage = SqliteStorage::new(&db_path).unwrap();
            assert_eq!(storage.get("key1"), Some(Value::Int(42)));
            assert_eq!(
                storage.get("key2"),
                Some(Value::String("hello".to_string()))
            );
        }
    }

    #[cfg(feature = "sqlite-storage")]
    #[test]
    fn test_transaction_manager_with_sqlite_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("tx.db");

        // Create manager with SQLite storage, commit transaction, drop
        {
            let mut manager =
                TransactionManager::with_storage(Box::new(SqliteStorage::new(&db_path).unwrap()));

            let tx_id = manager
                .begin_transaction(IsolationLevel::ReadCommitted)
                .unwrap();
            manager
                .write(&tx_id, "balance".to_string(), Value::Int(5000))
                .unwrap();
            manager
                .write(
                    &tx_id,
                    "user".to_string(),
                    Value::String("alice".to_string()),
                )
                .unwrap();
            manager.commit(&tx_id).unwrap();
        }

        // Reload manager with same DB - committed data should persist
        {
            let manager =
                TransactionManager::with_storage(Box::new(SqliteStorage::new(&db_path).unwrap()));

            assert_eq!(manager.get_committed("balance"), Some(Value::Int(5000)));
            assert_eq!(
                manager.get_committed("user"),
                Some(Value::String("alice".to_string()))
            );
        }
    }

    #[cfg(feature = "sqlite-storage")]
    #[test]
    fn test_sqlite_rollback_doesnt_persist() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("rollback.db");

        {
            let mut manager =
                TransactionManager::with_storage(Box::new(SqliteStorage::new(&db_path).unwrap()));

            // Commit first value
            let tx1 = manager
                .begin_transaction(IsolationLevel::ReadCommitted)
                .unwrap();
            manager
                .write(&tx1, "count".to_string(), Value::Int(10))
                .unwrap();
            manager.commit(&tx1).unwrap();

            // Rollback second transaction
            let tx2 = manager
                .begin_transaction(IsolationLevel::ReadCommitted)
                .unwrap();
            manager
                .write(&tx2, "count".to_string(), Value::Int(999))
                .unwrap();
            manager.rollback(&tx2).unwrap();
        }

        // Reload - only committed value should persist
        {
            let manager =
                TransactionManager::with_storage(Box::new(SqliteStorage::new(&db_path).unwrap()));
            assert_eq!(manager.get_committed("count"), Some(Value::Int(10)));
        }
    }

    #[cfg(feature = "sqlite-storage")]
    #[test]
    #[serial_test::serial]
    fn test_from_env_sqlite_backend() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("env_sqlite.db");

        std::env::set_var("DAL_TX_STORAGE", "sqlite");
        std::env::set_var("DAL_TX_STORAGE_PATH", db_path.to_str().unwrap());

        {
            let mut manager = TransactionManager::from_env().unwrap();
            let tx_id = manager
                .begin_transaction(IsolationLevel::ReadCommitted)
                .unwrap();
            manager
                .write(&tx_id, "sqlite_test".to_string(), Value::Int(777))
                .unwrap();
            manager.commit(&tx_id).unwrap();
        }

        // Reload with same env - should persist
        {
            let manager = TransactionManager::from_env().unwrap();
            assert_eq!(manager.get_committed("sqlite_test"), Some(Value::Int(777)));
        }

        // Cleanup
        std::env::remove_var("DAL_TX_STORAGE");
        std::env::remove_var("DAL_TX_STORAGE_PATH");
        std::env::remove_var("DAL_TX_LOG_PATH");
        std::env::remove_var("DAL_TX_TIMEOUT_MS");
    }
}

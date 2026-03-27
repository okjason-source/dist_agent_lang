# Transaction Module — Completed Features and Usage

This document describes what is **implemented** in the transaction module (`src/runtime/transaction.rs`): features, configuration, durability, recovery, and how to use it. For the `@txn` attribute see [TXN_ATTRIBUTE_GUIDE.md](./TXN_ATTRIBUTE_GUIDE.md). For the implementation plan and future work see [TRANSACTION_IMPLEMENTATION_PLAN.md](../development/implementation/TRANSACTION_IMPLEMENTATION_PLAN.md).

---

## Implemented Features

- **Transaction lifecycle**: begin/commit/rollback with ACID semantics
- **Savepoints**: Partial rollback to named checkpoints within a transaction
- **Isolation levels**: ReadUncommitted, ReadCommitted (default), RepeatableRead, Serializable
- **Deadlock detection**: Timeout-based returns `TransactionError::Deadlock`; cycle-based (wait-for graph) returns `TransactionError::DeadlockWithCycle(cycle)` with involved tx ids
- **Observability**: Optional event callbacks for lifecycle (begin, commit, rollback, conflicts, deadlocks)
- **Pluggable storage**: `StateStorage` trait with in-memory, file-backed, and SQLite backends
- **Resource limits**: Configurable max active transactions and max keys per transaction
- **Audit logging**: Optional append-only transaction log (line-delimited JSON)
- **Two-phase commit**: Basic single-node “prepare then commit” support for distributed-style transactions
- **WAL / recovery**: SqliteStorage uses SQLite WAL (automatic recovery on open); FileBackedStorage recovers from `.tmp` when main file is missing or corrupt
- **Read-only audit optimization**: Optional; when enabled, read-only commits (no writes) are not written to the transaction log (callback still runs). Env: `DAL_TX_READ_ONLY_AUDIT_OPTIMIZATION=1` or `with_read_only_audit_optimization(true)`.
- **Cycle-based deadlock detection**: In addition to timeout-based deadlock, the manager maintains a wait-for graph and detects cycles when a lock would block; returns `TransactionError::DeadlockWithCycle(cycle)` with involved tx ids.

---

## Isolation Levels

| Level | Behavior |
|-------|----------|
| **ReadUncommitted** | Lowest isolation; dirty reads possible; highest performance; no read locks. |
| **ReadCommitted** (default) | Prevents dirty reads; transactions see only committed data. |
| **RepeatableRead** | Prevents non-repeatable reads; same query returns same results within a transaction. |
| **Serializable** | Highest isolation; transactions appear to execute serially; lowest concurrency. |

---

## Safe Production Defaults

| Setting | Default | Purpose |
|--------|--------|---------|
| Isolation level | ReadCommitted | Balance of correctness and performance |
| Transaction timeout | 30000 ms (30 s) | Prevents hung transactions |
| Max active transactions | 1000 | Prevents resource exhaustion |
| Max keys per transaction | 10000 | Prevents unbounded memory use |
| Storage backend | memory | Use `file` or `sqlite` for persistence |

---

## Configuration (Environment)

```bash
export DAL_TX_STORAGE=sqlite          # or memory | file
export DAL_TX_STORAGE_PATH=/var/lib/dal/transactions.db
export DAL_TX_LOG_PATH=/var/log/dal/transactions.log   # optional audit log
export DAL_TX_TIMEOUT_MS=30000
export DAL_TX_MAX_ACTIVE=1000
export DAL_TX_MAX_KEYS=10000
```

Use `TransactionManager::from_env()` to build from these variables.

Optional: `DAL_TX_READ_ONLY_AUDIT_OPTIMIZATION=1` to skip writing read-only commits to the transaction log (callback still runs).

---

## Durability and Storage Backends

### InMemoryStorage

- **Durability**: None. State is lost on process exit or crash.
- **Performance**: Highest; no I/O.
- **Use case**: Development, testing, ephemeral state.

### FileBackedStorage

- **Durability**: Strong. Each `set()` and `remove()` flushes to disk by default (write to `.tmp` then atomic rename).
- **Performance**: Moderate; disk I/O on every write.
- **Use case**: Single-node production with state persistence.
- **Format**: JSON file; human-readable.
- **Recovery**: On startup, if the main file is missing or corrupt, state is loaded from the `.tmp` file (from an interrupted flush) if present and promoted to main.

### SqliteStorage (feature: `sqlite-storage`)

- **Durability**: Strong. Each `set()`/`remove()` commits immediately. ACID from SQLite.
- **Performance**: Good; supports WAL mode for concurrency.
- **Format**: SQLite database; table `kv_store(key, value)`; values JSON-encoded.
- **Recovery**: WAL mode is enabled. SQLite recovers automatically on next open; no application-level recovery steps.

### TransactionLog (optional)

- **Durability**: Strong. Append-only log with flush after each event.
- **Use case**: Audit trail, debugging. **Not** used for state recovery (audit-only).
- **Format**: Line-delimited JSON; one event per line.

---

## Recovery Semantics (WAL / Crash Recovery)

- **SqliteStorage**: WAL mode enabled. After a crash, SQLite recovers on next open (replay or rollback of WAL). No app steps required.
- **FileBackedStorage**: State is written to `.tmp` then atomically renamed to main. On startup, if main is missing or corrupt, state is loaded from `.tmp` if present and the file is promoted to main.
- **TransactionLog**: Audit-only; does not store key-value payloads. For state recovery use SqliteStorage or FileBackedStorage.

---

## Audit Logging

Set `DAL_TX_LOG_PATH` to a file path to enable. Each transaction event is written as line-delimited JSON, for example:

```json
{"timestamp":1675889234567,"tx_id":"tx_1","event_type":"begin","keys":[],"isolation_level":"ReadCommitted"}
{"timestamp":1675889234578,"tx_id":"tx_1","event_type":"write","keys":["account:123:balance"],"isolation_level":null}
{"timestamp":1675889234589,"tx_id":"tx_1","event_type":"commit","keys":["account:123:balance"],"isolation_level":null}
```

Use for compliance, debugging, and performance analysis. With **read-only audit optimization** enabled (`DAL_TX_READ_ONLY_AUDIT_OPTIMIZATION=1`), commits that modified zero keys are not written to the log file (the event callback still runs).

**Deadlock handling:** Timeout-based deadlock returns `TransactionError::Deadlock`. Cycle-based deadlock (when the wait-for graph has a cycle) returns `TransactionError::DeadlockWithCycle(cycle)` with the list of involved transaction ids; handle both by rolling back and optionally logging the cycle.

---

## Using transactions without `@txn`

You can use DAL transactions **without** the `@txn` attribute by calling the `database::` API explicitly. The attribute only automates begin/commit/rollback around a single function; the same operations are available manually.

**Manual transaction in DAL:**

```dal
fn transfer_manual(from: String, to: String, amount: Int) -> Result<Unit, Error> {
    let tx = database::begin_transaction("read_committed", 30000);
    let balance_from = database::tx_read("balance:" + from);
    let balance_to = database::tx_read("balance:" + to);
    if (balance_from < amount) {
        database::rollback();
        return Err(Error::new("InsufficientFunds", ""));
    }
    database::tx_write("balance:" + from, balance_from - amount);
    database::tx_write("balance:" + to, balance_to + amount);
    database::commit();
    return Ok(Unit);
}
```

**API (no attribute required):** `database::begin_transaction(isolation_level, timeout_ms?)` → tx_id, `database::commit()`, `database::rollback()`, `database::tx_read(key)`, `database::tx_write(key, value)`, `database::tx_savepoint(name)`, `database::tx_rollback_to(name)`.

Use manual control when you need custom boundaries (e.g. multiple operations in one transaction, or coordination with external systems) or when you are integrating with a multi-node or custom coordinator (see below).

---

## Multi-node and DAL

The **built-in runtime is single-node**: one process, one `TransactionManager`, one storage backend (memory, file, or SQLite). There is no built-in distributed transaction coordinator across processes or machines.

**If you need multi-node:**

- **From DAL (same process):** You can still use `database::*` (with or without `@txn`) for local ACID. To participate in a distributed transaction, you would need the runtime to be wired to your coordinator (e.g. a custom storage or engine integration that talks to your distributed system). The current `StateStorage` and transaction API do not implement cross-node 2PC.
- **From Rust:** You can use the public API (`TransactionManager`, `begin_transaction`, `commit`, `rollback`, etc.) and plug in a custom `StateStorage` or wrap the engine so that commit/rollback are forwarded to your own multi-node coordinator. DAL code that uses `database::*` then runs on that same engine; you do not need the `@txn` attribute for that.

So: **yes, you can use DAL without relying on the attribute**, and for multi-node you keep using DAL (with manual `database::*` or `@txn` as desired) and add multi-node behavior at the **integration layer** (custom storage, coordinator, or engine wrapper), not by the attribute itself.

---

## Usage (Rust)

```rust
use dist_agent_lang::runtime::{TransactionManager, IsolationLevel, Value};

let mut manager = TransactionManager::new()
    .with_default_timeout(Some(30000))
    .with_event_callback(Box::new(|event| {
        println!("Transaction event: {:?}", event);
    }));

let tx_id = manager.begin_transaction(IsolationLevel::ReadCommitted).unwrap();
manager.write(&tx_id, "balance".to_string(), Value::Int(100)).unwrap();
let value = manager.read(&tx_id, "balance").unwrap();
manager.create_savepoint(&tx_id, "checkpoint1".to_string()).unwrap();  // optional
manager.commit(&tx_id).unwrap();
// or: manager.rollback(&tx_id).unwrap();
// or: manager.rollback_to_savepoint(&tx_id, "checkpoint1").unwrap();
```

---

## Related Docs

- [TXN_ATTRIBUTE_GUIDE.md](./TXN_ATTRIBUTE_GUIDE.md) — `@txn` attribute for DAL functions
- [TRANSACTION_IMPLEMENTATION_PLAN.md](../development/implementation/TRANSACTION_IMPLEMENTATION_PLAN.md) — Implementation plan and future enhancements
- [TRANSACTIONAL_SCOPE_DESIGN.md](../development/implementation/TRANSACTIONAL_SCOPE_DESIGN.md) — Transactional state model and DAL API

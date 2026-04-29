# Persistent agent memory guide

This guide describes how agent memory, tasks, messages, and runtime state persist across restarts in dist_agent_lang. Persistence is **on by default** — no setup required. This guide covers configuration, backends, and troubleshooting.

---

## Table of contents

1. [Overview](#1-overview)
2. [What persists](#2-what-persists)
3. [Configuration](#3-configuration)
4. [Backends](#4-backends)
5. [How it works](#5-how-it-works)
6. [Disabling persistence](#6-disabling-persistence)
7. [Backup and recovery](#7-backup-and-recovery)
8. [Troubleshooting](#8-troubleshooting)
9. [References](#9-references)

---

## 1. Overview

When you run `dal agent serve` or execute agent DAL scripts, the agent accumulates state: key-value memory, pending tasks, queued messages, evolution data, and registered skills. By default, all of this is saved to disk after every change and restored automatically when the process restarts.

This means:

- **Deploys and restarts** don't wipe agent state.
- **Agent memory** (`store_memory`) carries across sessions.
- **Pending tasks and messages** survive crashes.
- **Runtime-registered skills** are restored without re-registering.

Conversation history (evolve) is already file-based and unaffected by this system.

---

## 2. What persists

| State | Persists? | Location |
|-------|-----------|----------|
| Conversation / context | Yes (already) | `evolve.md` |
| Agent memory (key-value) | **Yes** | Snapshot file or SQLite |
| Task queue | **Yes** | Snapshot file or SQLite |
| Message bus | **Yes** | Snapshot file or SQLite |
| Evolution store | **Yes** | Snapshot file or SQLite |
| Agent contexts (registry) | **Yes** | Snapshot file or SQLite |
| Runtime-registered skills | **Yes** | Snapshot file or SQLite |
| Serve agent ID | **Yes** | Snapshot file or SQLite |

---

## 3. Configuration

### Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DAL_AGENT_RUNTIME_PERSIST` | `1` (enabled) | Set to `0` or `false` to disable |
| `DAL_AGENT_RUNTIME_BACKEND` | `file` | `file` for JSON, `sqlite` for SQLite |
| `DAL_AGENT_RUNTIME_PATH` | auto-derived | Override the snapshot path |

### agent.toml / dal.toml

```toml
[agent]
runtime_persist = true                          # false to disable
runtime_path = ".dal/agent_runtime.json"        # override path
```

### Path resolution

The snapshot path is resolved in this order:

1. **`DAL_AGENT_RUNTIME_PATH`** env var (used directly if set)
2. **`[agent] runtime_path`** in `agent.toml` or `dal.toml` (resolved relative to cwd if not absolute)
3. **Default**: `.dal/agent_runtime.json` (file backend) or `.dal/agent_runtime.db` (SQLite backend) under the current working directory

---

## 4. Backends

### File backend (default)

- **Format**: Pretty-printed JSON
- **File**: `agent_runtime.json`
- **Writes**: Atomic (temp file + rename) — safe against partial writes and crashes
- **Concurrency**: Single writer per path
- **Best for**: Most use cases; human-readable, easy to inspect and back up

### SQLite backend

- **Format**: SQLite database with WAL mode
- **File**: `agent_runtime.db`
- **Writes**: ACID-guaranteed; immediate commit per mutation
- **Concurrency**: WAL mode allows concurrent readers; writes are serialized via Mutex
- **Best for**: High write throughput, multi-threaded access, ACID requirements
- **Requires**: Build with `--features sqlite-storage`

Select the backend:

```bash
# File backend (default)
export DAL_AGENT_RUNTIME_BACKEND=file

# SQLite backend
export DAL_AGENT_RUNTIME_BACKEND=sqlite
```

---

## 5. How it works

### Startup

When the agent runtime is first accessed, the persistence layer:

1. Reads the config (env vars, then agent.toml/dal.toml)
2. Opens the snapshot file or database
3. Deserializes the snapshot and populates the in-memory runtime
4. Restores runtime-registered skills into the global skill registry

If the file is missing, the runtime starts empty (no error). If the file is corrupt, a warning is logged and the runtime starts empty.

### During execution

Every mutation triggers an immediate save:

- Spawning an agent
- Setting the serve agent
- Coordinating tasks
- Receiving pending tasks
- Sending or receiving messages
- Evolving an agent
- Registering runtime skills

### Limits

- Task queue and message bus are capped at **10,000 items** in the snapshot to prevent unbounded disk growth
- Oldest items beyond the cap are dropped during save

### Schema versioning

The snapshot includes a `version` field (currently `1`). If you upgrade to a newer version of dist_agent_lang that changes the schema, the runtime automatically migrates old snapshots. Snapshots from a *newer* version than the runtime supports are rejected (runtime starts empty with a warning).

---

## 6. Disabling persistence

To opt out of persistence entirely:

```bash
export DAL_AGENT_RUNTIME_PERSIST=0
```

Or in `agent.toml`:

```toml
[agent]
runtime_persist = false
```

When disabled:

- No snapshot file is read or written
- All state is in-memory only (original behavior)
- Restarts clear everything except evolve (which is independent)

---

## 7. Backup and recovery

### Backup

Copy the snapshot file:

```bash
cp .dal/agent_runtime.json .dal/agent_runtime.json.bak
```

For SQLite:

```bash
cp .dal/agent_runtime.db .dal/agent_runtime.db.bak
```

### Recovery

If the snapshot is corrupt:

1. The runtime starts with empty state and logs a warning
2. Evolve history is unaffected (separate file)
3. To restore from backup, replace the snapshot file with your backup and restart

### Inspecting state

The JSON snapshot is human-readable:

```bash
cat .dal/agent_runtime.json | python3 -m json.tool
```

For SQLite:

```bash
sqlite3 .dal/agent_runtime.db "SELECT data FROM agent_runtime_snapshot WHERE id = 1" | python3 -m json.tool
```

---

## 8. Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| State lost after restart | Persistence disabled | Check `DAL_AGENT_RUNTIME_PERSIST` and `agent.toml` |
| "Corrupt snapshot" warning | Invalid JSON or schema mismatch | Delete or restore snapshot from backup |
| "Version > supported" warning | Snapshot from newer dist_agent_lang | Upgrade dist_agent_lang or delete snapshot |
| Permission errors | Cannot write to `.dal/` | Check directory permissions |
| SQLite errors | Missing `sqlite-storage` feature | Rebuild with `--features sqlite-storage` |

---

## 9. References

- [PERSISTENT_AGENT_MEMORY_PLAN.md](../PERSISTENT_AGENT_MEMORY_PLAN.md) — Implementation plan and architecture details
- [SKILLS_AND_REGISTRY.md](SKILLS_AND_REGISTRY.md) — User-defined skills (works alongside persistent memory)
- [AGENT_SETUP_AND_USAGE.md](AGENT_SETUP_AND_USAGE.md) — Agent setup, CLI, serve, molds
- `src/stdlib/agent_persist.rs` — Persistence implementation
- `src/stdlib/agent.rs` — Agent runtime with integrated persistence

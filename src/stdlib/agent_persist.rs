//! Persistent agent runtime: snapshot, backends (file, SQLite), config.
//! Persistence is ON by default; set DAL_AGENT_RUNTIME_PERSIST=0 or agent.toml [agent] runtime_persist = false to disable.
//! Backend: DAL_AGENT_RUNTIME_BACKEND=file|sqlite (default: file). Path: DAL_AGENT_RUNTIME_PATH or agent.toml [agent] runtime_path.

use crate::runtime::values::Value;
use crate::stdlib::agent::{
    AgentConfig, AgentContext, AgentMessage, AgentMetrics, AgentStatus, AgentTask, AgentType,
    LifecycleHooks, MessagePriority, TaskPriority, TaskStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ── Snapshot version ────────────────────────────────────────────────

pub const SNAPSHOT_VERSION: u32 = 1;

/// Max items in task_queue / message_bus to persist (prevents unbounded growth).
const MAX_QUEUE_LEN: usize = 10_000;

// ── Serializable DTOs ───────────────────────────────────────────────
// We convert to/from domain types so agent.rs doesn't need serde derives.

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentRuntimeSnapshot {
    pub version: u32,
    pub agent_contexts: HashMap<String, AgentContextDto>,
    pub task_queue: Vec<(String, AgentTaskDto)>,
    pub message_bus: Vec<(String, AgentMessageDto)>,
    pub evolution_store: HashMap<String, HashMap<String, Value>>,
    pub serve_agent_id: Option<String>,
    /// Skills registered at runtime (not from .skill.dal files or built-ins).
    #[serde(default)]
    pub registered_skills: Vec<SkillDefinitionDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentContextDto {
    pub agent_id: String,
    pub config: AgentConfigDto,
    pub status: String,
    pub memory: HashMap<String, Value>,
    pub tasks: Vec<AgentTaskDto>,
    pub message_queue: Vec<AgentMessageDto>,
    pub created_at: String,
    pub last_active: String,
    pub performance_metrics: AgentMetricsDto,
    pub trust_score: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentConfigDto {
    pub name: String,
    pub agent_type: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub skills: Vec<String>,
    pub trust_level: String,
    pub max_memory: usize,
    pub learning_enabled: bool,
    pub communication_enabled: bool,
    pub coordination_enabled: bool,
    pub metadata: HashMap<String, Value>,
    pub lifecycle: Option<LifecycleHooksDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LifecycleHooksDto {
    pub on_create: Option<String>,
    pub on_message: Option<String>,
    pub on_evolve: Option<String>,
    pub on_destroy: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentTaskDto {
    pub task_id: String,
    pub description: String,
    pub priority: String,
    pub status: String,
    pub assigned_at: String,
    pub completed_at: Option<String>,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentMessageDto {
    pub message_id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub message_type: String,
    pub content: Value,
    pub timestamp: String,
    pub priority: String,
    pub requires_response: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentMetricsDto {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub tasks_assigned: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub coordination_events: u64,
    pub learning_sessions: u64,
    pub status_changes: u64,
    pub average_response_time: f64,
    pub uptime_percentage: f64,
}

// ── Domain ↔ DTO conversions ────────────────────────────────────────

impl From<&AgentContext> for AgentContextDto {
    fn from(ctx: &AgentContext) -> Self {
        Self {
            agent_id: ctx.agent_id.clone(),
            config: AgentConfigDto::from(&ctx.config),
            status: ctx.status.to_string(),
            memory: ctx.memory.clone(),
            tasks: ctx.tasks.iter().map(AgentTaskDto::from).collect(),
            message_queue: ctx
                .message_queue
                .iter()
                .map(AgentMessageDto::from)
                .collect(),
            created_at: ctx.created_at.clone(),
            last_active: ctx.last_active.clone(),
            performance_metrics: AgentMetricsDto::from(&ctx.performance_metrics),
            trust_score: ctx.trust_score,
        }
    }
}

impl AgentContextDto {
    pub fn to_domain(&self) -> AgentContext {
        AgentContext {
            agent_id: self.agent_id.clone(),
            config: self.config.to_domain(),
            status: AgentStatus::from_string(&self.status).unwrap_or(AgentStatus::Idle),
            memory: self.memory.clone(),
            tasks: self.tasks.iter().map(|t| t.to_domain()).collect(),
            message_queue: self.message_queue.iter().map(|m| m.to_domain()).collect(),
            created_at: self.created_at.clone(),
            last_active: self.last_active.clone(),
            performance_metrics: self.performance_metrics.to_domain(),
            trust_score: self.trust_score,
        }
    }
}

impl From<&AgentConfig> for AgentConfigDto {
    fn from(cfg: &AgentConfig) -> Self {
        Self {
            name: cfg.name.clone(),
            agent_type: cfg.agent_type.to_string(),
            role: cfg.role.clone(),
            capabilities: cfg.capabilities.clone(),
            skills: cfg.skills.clone(),
            trust_level: cfg.trust_level.clone(),
            max_memory: cfg.max_memory,
            learning_enabled: cfg.learning_enabled,
            communication_enabled: cfg.communication_enabled,
            coordination_enabled: cfg.coordination_enabled,
            metadata: cfg.metadata.clone(),
            lifecycle: cfg.lifecycle.as_ref().map(LifecycleHooksDto::from),
        }
    }
}

impl AgentConfigDto {
    pub fn to_domain(&self) -> AgentConfig {
        AgentConfig {
            name: self.name.clone(),
            agent_type: AgentType::from_string(&self.agent_type).unwrap_or(AgentType::AI),
            role: self.role.clone(),
            capabilities: self.capabilities.clone(),
            skills: self.skills.clone(),
            trust_level: self.trust_level.clone(),
            max_memory: self.max_memory,
            learning_enabled: self.learning_enabled,
            communication_enabled: self.communication_enabled,
            coordination_enabled: self.coordination_enabled,
            metadata: self.metadata.clone(),
            lifecycle: self.lifecycle.as_ref().map(|l| l.to_domain()),
        }
    }
}

impl From<&LifecycleHooks> for LifecycleHooksDto {
    fn from(l: &LifecycleHooks) -> Self {
        Self {
            on_create: l.on_create.clone(),
            on_message: l.on_message.clone(),
            on_evolve: l.on_evolve.clone(),
            on_destroy: l.on_destroy.clone(),
        }
    }
}

impl LifecycleHooksDto {
    pub fn to_domain(&self) -> LifecycleHooks {
        LifecycleHooks {
            on_create: self.on_create.clone(),
            on_message: self.on_message.clone(),
            on_evolve: self.on_evolve.clone(),
            on_destroy: self.on_destroy.clone(),
        }
    }
}

impl From<&AgentTask> for AgentTaskDto {
    fn from(t: &AgentTask) -> Self {
        Self {
            task_id: t.task_id.clone(),
            description: t.description.clone(),
            priority: t.priority.to_string(),
            status: t.status.to_string(),
            assigned_at: t.assigned_at.clone(),
            completed_at: t.completed_at.clone(),
            dependencies: t.dependencies.clone(),
            metadata: t.metadata.clone(),
        }
    }
}

impl AgentTaskDto {
    pub fn to_domain(&self) -> AgentTask {
        AgentTask {
            task_id: self.task_id.clone(),
            description: self.description.clone(),
            priority: TaskPriority::from_string(&self.priority).unwrap_or(TaskPriority::Medium),
            status: TaskStatus::from_string(&self.status).unwrap_or(TaskStatus::Pending),
            assigned_at: self.assigned_at.clone(),
            completed_at: self.completed_at.clone(),
            dependencies: self.dependencies.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl From<&AgentMessage> for AgentMessageDto {
    fn from(m: &AgentMessage) -> Self {
        Self {
            message_id: m.message_id.clone(),
            sender_id: m.sender_id.clone(),
            receiver_id: m.receiver_id.clone(),
            message_type: m.message_type.clone(),
            content: m.content.clone(),
            timestamp: m.timestamp.clone(),
            priority: m.priority.to_string(),
            requires_response: m.requires_response,
        }
    }
}

impl AgentMessageDto {
    pub fn to_domain(&self) -> AgentMessage {
        AgentMessage {
            message_id: self.message_id.clone(),
            sender_id: self.sender_id.clone(),
            receiver_id: self.receiver_id.clone(),
            message_type: self.message_type.clone(),
            content: self.content.clone(),
            timestamp: self.timestamp.clone(),
            priority: MessagePriority::from_string(&self.priority)
                .unwrap_or(MessagePriority::Normal),
            requires_response: self.requires_response,
        }
    }
}

impl From<&AgentMetrics> for AgentMetricsDto {
    fn from(m: &AgentMetrics) -> Self {
        Self {
            tasks_completed: m.tasks_completed,
            tasks_failed: m.tasks_failed,
            tasks_assigned: m.tasks_assigned,
            messages_sent: m.messages_sent,
            messages_received: m.messages_received,
            coordination_events: m.coordination_events,
            learning_sessions: m.learning_sessions,
            status_changes: m.status_changes,
            average_response_time: m.average_response_time,
            uptime_percentage: m.uptime_percentage,
        }
    }
}

impl AgentMetricsDto {
    pub fn to_domain(&self) -> AgentMetrics {
        AgentMetrics {
            tasks_completed: self.tasks_completed,
            tasks_failed: self.tasks_failed,
            tasks_assigned: self.tasks_assigned,
            messages_sent: self.messages_sent,
            messages_received: self.messages_received,
            coordination_events: self.coordination_events,
            learning_sessions: self.learning_sessions,
            status_changes: self.status_changes,
            average_response_time: self.average_response_time,
            uptime_percentage: self.uptime_percentage,
        }
    }
}

// ── Skill DTO ───────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkillDefinitionDto {
    pub name: String,
    pub category: Option<String>,
    pub description: String,
    pub tools: Vec<String>,
}

impl SkillDefinitionDto {
    pub fn from_domain(skill: &crate::skills::SkillDefinition) -> Self {
        Self {
            name: skill.name.clone(),
            category: skill.category.as_ref().map(|c| c.as_str().to_string()),
            description: skill.description.clone(),
            tools: skill.tools.clone(),
        }
    }

    pub fn to_domain(&self) -> crate::skills::SkillDefinition {
        crate::skills::SkillDefinition {
            name: self.name.clone(),
            category: self
                .category
                .as_deref()
                .and_then(crate::skills::SkillCategory::from_str),
            description: self.description.clone(),
            tools: self.tools.clone(),
            builtin: false,
        }
    }
}

// ── Snapshot helpers ─────────────────────────────────────────────────

impl AgentRuntimeSnapshot {
    pub fn empty() -> Self {
        Self {
            version: SNAPSHOT_VERSION,
            agent_contexts: HashMap::new(),
            task_queue: Vec::new(),
            message_bus: Vec::new(),
            evolution_store: HashMap::new(),
            serve_agent_id: None,
            registered_skills: Vec::new(),
        }
    }

    /// Build snapshot from current runtime state. Caps queues to MAX_QUEUE_LEN.
    pub fn from_runtime(
        agent_contexts: &HashMap<String, AgentContext>,
        task_queue: &[(String, AgentTask)],
        message_bus: &[(String, AgentMessage)],
        evolution_store: &HashMap<String, HashMap<String, Value>>,
        serve_agent_id: &Option<String>,
        registered_skills: &[crate::skills::SkillDefinition],
    ) -> Self {
        let capped_tasks: Vec<(String, AgentTaskDto)> = task_queue
            .iter()
            .rev()
            .take(MAX_QUEUE_LEN)
            .rev()
            .map(|(id, t)| (id.clone(), AgentTaskDto::from(t)))
            .collect();
        let capped_messages: Vec<(String, AgentMessageDto)> = message_bus
            .iter()
            .rev()
            .take(MAX_QUEUE_LEN)
            .rev()
            .map(|(id, m)| (id.clone(), AgentMessageDto::from(m)))
            .collect();
        Self {
            version: SNAPSHOT_VERSION,
            agent_contexts: agent_contexts
                .iter()
                .map(|(k, v)| (k.clone(), AgentContextDto::from(v)))
                .collect(),
            task_queue: capped_tasks,
            message_bus: capped_messages,
            evolution_store: evolution_store.clone(),
            serve_agent_id: serve_agent_id.clone(),
            registered_skills: registered_skills
                .iter()
                .filter(|s| !s.builtin)
                .map(SkillDefinitionDto::from_domain)
                .collect(),
        }
    }

    /// Migrate snapshot from an older version to the current version.
    /// Returns the snapshot unmodified if already at SNAPSHOT_VERSION.
    /// Future schema changes add migration arms here (e.g. 1 → 2).
    pub fn migrate(mut self) -> Self {
        // Example future migration:
        // if self.version == 1 {
        //     // add new field defaults, restructure, etc.
        //     self.version = 2;
        // }
        if self.version < SNAPSHOT_VERSION {
            log::info!(
                "Migrating agent runtime snapshot v{} → v{}",
                self.version,
                SNAPSHOT_VERSION
            );
            self.version = SNAPSHOT_VERSION;
        }
        self
    }
}

// ── Persistence config ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PersistBackend {
    File,
    #[cfg(feature = "sqlite-storage")]
    Sqlite,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct PersistConfig {
    pub backend: PersistBackend,
    pub path: PathBuf,
}

impl PersistConfig {
    /// Resolve config from env and (optionally) agent.toml.
    /// Persistence is ON by default (file backend).
    /// Set DAL_AGENT_RUNTIME_PERSIST=0 to disable.
    /// Set DAL_AGENT_RUNTIME_BACKEND=sqlite to use SQLite.
    /// Set DAL_AGENT_RUNTIME_PATH to override path.
    pub fn from_env() -> Self {
        let disabled = std::env::var("DAL_AGENT_RUNTIME_PERSIST")
            .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
            .unwrap_or(false);
        if disabled {
            return Self {
                backend: PersistBackend::Disabled,
                path: PathBuf::new(),
            };
        }

        let backend_str =
            std::env::var("DAL_AGENT_RUNTIME_BACKEND").unwrap_or_else(|_| "file".to_string());

        let backend = match backend_str.to_lowercase().as_str() {
            #[cfg(feature = "sqlite-storage")]
            "sqlite" => PersistBackend::Sqlite,
            "disabled" | "off" | "none" => PersistBackend::Disabled,
            _ => PersistBackend::File,
        };

        if backend == PersistBackend::Disabled {
            return Self {
                backend,
                path: PathBuf::new(),
            };
        }

        let path = if let Ok(p) = std::env::var("DAL_AGENT_RUNTIME_PATH") {
            PathBuf::from(p)
        } else {
            Self::derive_path_from_context(&backend)
        };

        Self { backend, path }
    }

    fn derive_path_from_context(backend: &PersistBackend) -> PathBuf {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        // Try to read context_path from agent.toml / dal.toml to put snapshot near evolve
        for name in &["agent.toml", "dal.toml"] {
            let toml_path = cwd.join(name);
            if let Ok(content) = std::fs::read_to_string(&toml_path) {
                if let Ok(table) = content.parse::<toml::Table>() {
                    // Check [agent] runtime_persist = false
                    if let Some(persist_val) = table
                        .get("agent")
                        .and_then(|v| v.as_table())
                        .and_then(|t| t.get("runtime_persist"))
                    {
                        if persist_val.as_bool() == Some(false)
                            || persist_val.as_str().map(|s| s == "false") == Some(true)
                        {
                            // Will be caught by caller — but return empty
                        }
                    }
                    // Check [agent] runtime_path
                    if let Some(rp) = table
                        .get("agent")
                        .and_then(|v| v.as_table())
                        .and_then(|t| t.get("runtime_path"))
                        .and_then(|v| v.as_str())
                    {
                        let mut p = PathBuf::from(rp);
                        if !p.is_absolute() {
                            p = cwd.join(p);
                        }
                        return p;
                    }
                    // Check [agent] runtime_backend override from toml
                    // (already handled above via env; toml is a fallback)
                }
            }
        }
        // Default: .dal/agent_runtime.json or .dal/agent_runtime.db
        let dal_dir = cwd.join(".dal");
        match backend {
            #[cfg(feature = "sqlite-storage")]
            PersistBackend::Sqlite => dal_dir.join("agent_runtime.db"),
            PersistBackend::File => dal_dir.join("agent_runtime.json"),
            PersistBackend::Disabled => PathBuf::new(),
        }
    }

    /// Re-check agent.toml for runtime_persist = false (post env check).
    pub fn is_disabled_by_toml(&self) -> bool {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        for name in &["agent.toml", "dal.toml"] {
            let toml_path = cwd.join(name);
            if let Ok(content) = std::fs::read_to_string(&toml_path) {
                if let Ok(table) = content.parse::<toml::Table>() {
                    if let Some(persist_val) = table
                        .get("agent")
                        .and_then(|v| v.as_table())
                        .and_then(|t| t.get("runtime_persist"))
                    {
                        if persist_val.as_bool() == Some(false)
                            || persist_val.as_str().map(|s| s == "false") == Some(true)
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

// ── Persistence trait ───────────────────────────────────────────────

pub trait RuntimePersistence: Send + Sync {
    fn load(&self) -> Result<AgentRuntimeSnapshot, String>;
    fn save(&self, snapshot: &AgentRuntimeSnapshot) -> Result<(), String>;
}

// ── File-backed persistence ─────────────────────────────────────────

pub struct FileBackedPersistence {
    path: PathBuf,
}

impl FileBackedPersistence {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {:?}: {}", parent, e))?;
        }
        Ok(Self { path })
    }
}

impl RuntimePersistence for FileBackedPersistence {
    fn load(&self) -> Result<AgentRuntimeSnapshot, String> {
        if !self.path.exists() {
            return Ok(AgentRuntimeSnapshot::empty());
        }
        let data = std::fs::read_to_string(&self.path)
            .map_err(|e| format!("Failed to read {:?}: {}", self.path, e))?;
        let snapshot: AgentRuntimeSnapshot = serde_json::from_str(&data).map_err(|e| {
            log::warn!(
                "Corrupt agent runtime snapshot at {:?}: {}. Starting empty.",
                self.path,
                e
            );
            format!("Corrupt snapshot: {}", e)
        })?;
        if snapshot.version > SNAPSHOT_VERSION {
            log::warn!(
                "Agent runtime snapshot version {} > supported {}. Starting empty.",
                snapshot.version,
                SNAPSHOT_VERSION
            );
            return Ok(AgentRuntimeSnapshot::empty());
        }
        Ok(snapshot.migrate())
    }

    fn save(&self, snapshot: &AgentRuntimeSnapshot) -> Result<(), String> {
        let temp_path = self.path.with_extension("json.tmp");
        let data = serde_json::to_string_pretty(snapshot)
            .map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write(&temp_path, &data)
            .map_err(|e| format!("Write temp {:?}: {}", temp_path, e))?;
        std::fs::rename(&temp_path, &self.path)
            .map_err(|e| format!("Rename {:?} → {:?}: {}", temp_path, self.path, e))?;
        Ok(())
    }
}

// ── SQLite-backed persistence ───────────────────────────────────────

#[cfg(feature = "sqlite-storage")]
pub struct SqliteRuntimePersistence {
    conn: std::sync::Mutex<rusqlite::Connection>,
}

#[cfg(feature = "sqlite-storage")]
impl SqliteRuntimePersistence {
    pub fn new(path: &std::path::Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create dir {:?}: {}", parent, e))?;
        }
        let conn = rusqlite::Connection::open(path)
            .map_err(|e| format!("open sqlite {:?}: {}", path, e))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| format!("WAL mode: {}", e))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_runtime_snapshot (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                data TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )
        .map_err(|e| format!("create table: {}", e))?;
        Ok(Self {
            conn: std::sync::Mutex::new(conn),
        })
    }
}

#[cfg(feature = "sqlite-storage")]
impl RuntimePersistence for SqliteRuntimePersistence {
    fn load(&self) -> Result<AgentRuntimeSnapshot, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        let result: Result<String, rusqlite::Error> = conn.query_row(
            "SELECT data FROM agent_runtime_snapshot WHERE id = 1",
            [],
            |row| row.get(0),
        );
        match result {
            Ok(json) => {
                let snapshot: AgentRuntimeSnapshot = serde_json::from_str(&json).map_err(|e| {
                    log::warn!(
                        "Corrupt agent runtime snapshot in SQLite: {}. Starting empty.",
                        e
                    );
                    format!("corrupt: {}", e)
                })?;
                if snapshot.version > SNAPSHOT_VERSION {
                    log::warn!(
                        "Agent runtime snapshot version {} > supported {}. Starting empty.",
                        snapshot.version,
                        SNAPSHOT_VERSION
                    );
                    return Ok(AgentRuntimeSnapshot::empty());
                }
                Ok(snapshot.migrate())
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AgentRuntimeSnapshot::empty()),
            Err(e) => {
                log::warn!(
                    "Failed to load agent runtime from SQLite: {}. Starting empty.",
                    e
                );
                Ok(AgentRuntimeSnapshot::empty())
            }
        }
    }

    fn save(&self, snapshot: &AgentRuntimeSnapshot) -> Result<(), String> {
        let json = serde_json::to_string(snapshot).map_err(|e| format!("serialize: {}", e))?;
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO agent_runtime_snapshot (id, data, updated_at) VALUES (1, ?1, datetime('now'))",
            [&json],
        )
        .map_err(|e| format!("upsert: {}", e))?;
        Ok(())
    }
}

// ── In-memory (no-op) persistence ───────────────────────────────────

pub struct NullPersistence;

impl RuntimePersistence for NullPersistence {
    fn load(&self) -> Result<AgentRuntimeSnapshot, String> {
        Ok(AgentRuntimeSnapshot::empty())
    }
    fn save(&self, _snapshot: &AgentRuntimeSnapshot) -> Result<(), String> {
        Ok(())
    }
}

// ── Factory ─────────────────────────────────────────────────────────

pub fn create_persistence(config: &PersistConfig) -> Result<Box<dyn RuntimePersistence>, String> {
    match &config.backend {
        PersistBackend::Disabled => Ok(Box::new(NullPersistence)),
        PersistBackend::File => {
            let fb = FileBackedPersistence::new(config.path.clone())?;
            Ok(Box::new(fb))
        }
        #[cfg(feature = "sqlite-storage")]
        PersistBackend::Sqlite => {
            let sq = SqliteRuntimePersistence::new(&config.path)?;
            Ok(Box::new(sq))
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_context() -> AgentContext {
        let config = AgentConfig::new("test_agent".to_string(), AgentType::AI)
            .with_role("tester".to_string())
            .with_capabilities(vec!["read".to_string()])
            .with_skills(vec!["development".to_string()])
            .with_lifecycle(Some(LifecycleHooks {
                on_create: Some("log(\"created\")".to_string()),
                on_message: None,
                on_evolve: Some("log(\"evolved\")".to_string()),
                on_destroy: None,
            }));
        let mut ctx = AgentContext::new("agent_test_1".to_string(), config);
        ctx.store_memory("key1".to_string(), Value::String("val1".to_string()));
        ctx.store_memory("count".to_string(), Value::Int(42));
        ctx
    }

    fn sample_task() -> AgentTask {
        AgentTask {
            task_id: "task_1".to_string(),
            description: "test task".to_string(),
            priority: TaskPriority::High,
            status: TaskStatus::Pending,
            assigned_at: "2026-01-01T00:00:00Z".to_string(),
            completed_at: None,
            dependencies: vec!["dep_1".to_string()],
            metadata: HashMap::new(),
        }
    }

    fn sample_message() -> AgentMessage {
        AgentMessage {
            message_id: "msg_1".to_string(),
            sender_id: "agent_a".to_string(),
            receiver_id: "agent_b".to_string(),
            message_type: "user".to_string(),
            content: Value::String("hello".to_string()),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            priority: MessagePriority::High,
            requires_response: true,
        }
    }

    #[test]
    fn roundtrip_snapshot_json() {
        let mut contexts = HashMap::new();
        let ctx = sample_context();
        contexts.insert(ctx.agent_id.clone(), ctx);
        let tasks = vec![("agent_test_1".to_string(), sample_task())];
        let msgs = vec![("agent_b".to_string(), sample_message())];
        let mut evo = HashMap::new();
        evo.insert("agent_test_1".to_string(), {
            let mut m = HashMap::new();
            m.insert("learned".to_string(), Value::Bool(true));
            m
        });
        let snap = AgentRuntimeSnapshot::from_runtime(
            &contexts,
            &tasks,
            &msgs,
            &evo,
            &Some("agent_test_1".to_string()),
            &[],
        );

        let json = serde_json::to_string_pretty(&snap).expect("serialize");
        let restored: AgentRuntimeSnapshot = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.version, SNAPSHOT_VERSION);
        assert_eq!(restored.serve_agent_id, Some("agent_test_1".to_string()));
        assert_eq!(restored.agent_contexts.len(), 1);
        let rctx = &restored.agent_contexts["agent_test_1"];
        assert_eq!(rctx.config.name, "test_agent");
        assert_eq!(rctx.config.agent_type, "ai");
        assert_eq!(
            rctx.memory.get("key1"),
            Some(&Value::String("val1".to_string()))
        );
        assert_eq!(rctx.memory.get("count"), Some(&Value::Int(42)));
        assert_eq!(
            rctx.config.lifecycle.as_ref().unwrap().on_create,
            Some("log(\"created\")".to_string())
        );
        assert_eq!(restored.task_queue.len(), 1);
        assert_eq!(restored.task_queue[0].1.priority, "high");
        assert_eq!(restored.message_bus.len(), 1);
        assert_eq!(restored.message_bus[0].1.priority, "high");
        assert!(restored.evolution_store.contains_key("agent_test_1"));
    }

    #[test]
    fn roundtrip_domain_conversion() {
        let ctx = sample_context();
        let dto = AgentContextDto::from(&ctx);
        let restored = dto.to_domain();
        assert_eq!(restored.agent_id, ctx.agent_id);
        assert_eq!(restored.config.name, ctx.config.name);
        assert_eq!(
            restored.config.agent_type.to_string(),
            ctx.config.agent_type.to_string()
        );
        assert_eq!(restored.status.to_string(), ctx.status.to_string());
        assert_eq!(restored.memory.len(), ctx.memory.len());
        assert_eq!(restored.trust_score, ctx.trust_score);
    }

    #[test]
    fn file_backed_persistence_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_runtime.json");
        let fb = FileBackedPersistence::new(path).unwrap();

        let loaded = fb.load().unwrap();
        assert!(loaded.agent_contexts.is_empty());

        let mut contexts = HashMap::new();
        contexts.insert("a1".to_string(), sample_context());
        let snap = AgentRuntimeSnapshot::from_runtime(
            &contexts,
            &[],
            &[],
            &HashMap::new(),
            &Some("a1".to_string()),
            &[],
        );
        fb.save(&snap).unwrap();

        let reloaded = fb.load().unwrap();
        assert_eq!(reloaded.agent_contexts.len(), 1);
        assert_eq!(reloaded.serve_agent_id, Some("a1".to_string()));
    }

    #[test]
    fn file_backed_corrupt_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupt.json");
        std::fs::write(&path, "not json").unwrap();
        let fb = FileBackedPersistence::new(path).unwrap();
        assert!(fb.load().is_err());
    }

    #[test]
    fn null_persistence_noop() {
        let np = NullPersistence;
        let loaded = np.load().unwrap();
        assert!(loaded.agent_contexts.is_empty());
        np.save(&AgentRuntimeSnapshot::empty()).unwrap();
    }

    #[cfg(feature = "sqlite-storage")]
    #[test]
    fn sqlite_persistence_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_runtime.db");
        let sq = SqliteRuntimePersistence::new(&path).unwrap();

        let loaded = sq.load().unwrap();
        assert!(loaded.agent_contexts.is_empty());

        let mut contexts = HashMap::new();
        contexts.insert("a1".to_string(), sample_context());
        let snap = AgentRuntimeSnapshot::from_runtime(
            &contexts,
            &[("a1".to_string(), sample_task())],
            &[("agent_b".to_string(), sample_message())],
            &HashMap::new(),
            &Some("a1".to_string()),
            &[],
        );
        sq.save(&snap).unwrap();

        let reloaded = sq.load().unwrap();
        assert_eq!(reloaded.agent_contexts.len(), 1);
        assert_eq!(reloaded.task_queue.len(), 1);
        assert_eq!(reloaded.message_bus.len(), 1);
    }

    #[test]
    fn persist_config_default_is_enabled() {
        // Without env vars, persistence is ON by default
        let config = PersistConfig::from_env();
        assert_ne!(config.backend, PersistBackend::Disabled);
    }

    #[test]
    fn queue_capping() {
        let contexts = HashMap::new();
        let tasks: Vec<(String, AgentTask)> = (0..20_000)
            .map(|i| (format!("agent_{}", i), sample_task()))
            .collect();
        let snap =
            AgentRuntimeSnapshot::from_runtime(&contexts, &tasks, &[], &HashMap::new(), &None, &[]);
        assert_eq!(snap.task_queue.len(), MAX_QUEUE_LEN);
    }

    #[test]
    fn registered_skills_roundtrip() {
        let skill = crate::skills::SkillDefinition {
            name: "ms_office".to_string(),
            category: Some(crate::skills::SkillCategory::Office),
            description: "Use MS Office tools.".to_string(),
            tools: vec!["run".to_string(), "search".to_string()],
            builtin: false,
        };
        let snap = AgentRuntimeSnapshot::from_runtime(
            &HashMap::new(),
            &[],
            &[],
            &HashMap::new(),
            &None,
            &[skill.clone()],
        );
        assert_eq!(snap.registered_skills.len(), 1);
        assert_eq!(snap.registered_skills[0].name, "ms_office");

        let json = serde_json::to_string(&snap).expect("serialize");
        let restored: AgentRuntimeSnapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.registered_skills.len(), 1);
        let restored_skill = restored.registered_skills[0].to_domain();
        assert_eq!(restored_skill.name, "ms_office");
        assert_eq!(
            restored_skill.category,
            Some(crate::skills::SkillCategory::Office)
        );
        assert_eq!(
            restored_skill.tools,
            vec!["run".to_string(), "search".to_string()]
        );
        assert!(!restored_skill.builtin);
    }

    #[test]
    fn builtin_skills_excluded_from_snapshot() {
        let builtin = crate::skills::SkillDefinition {
            name: "development".to_string(),
            category: Some(crate::skills::SkillCategory::Development),
            description: "Built-in dev.".to_string(),
            tools: vec![],
            builtin: true,
        };
        let user = crate::skills::SkillDefinition {
            name: "custom".to_string(),
            category: None,
            description: "Custom skill.".to_string(),
            tools: vec!["run".to_string()],
            builtin: false,
        };
        let snap = AgentRuntimeSnapshot::from_runtime(
            &HashMap::new(),
            &[],
            &[],
            &HashMap::new(),
            &None,
            &[builtin, user],
        );
        assert_eq!(snap.registered_skills.len(), 1);
        assert_eq!(snap.registered_skills[0].name, "custom");
    }

    #[test]
    fn snapshot_without_skills_field_deserializes() {
        let json = r#"{"version":1,"agent_contexts":{},"task_queue":[],"message_bus":[],"evolution_store":{},"serve_agent_id":null}"#;
        let snap: AgentRuntimeSnapshot = serde_json::from_str(json).expect("deserialize");
        assert!(snap.registered_skills.is_empty());
    }
}

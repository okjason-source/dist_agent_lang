use crate::runtime::values::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Agent ABI - Interface for AI Agent operations
///
/// This provides a namespace-based approach to AI Agent operations:
/// - agent::spawn(agent_type, config, capabilities) - Spawn new agent
/// - agent::coordinate(agent_id, task, coordination_type) - Coordinate agent activities
/// - agent::communicate(sender_id, receiver_id, message) - Agent-to-agent communication
/// - agent::evolve(agent_id, evolution_data) - Agent learning and evolution
/// - agent::validate_capabilities(agent_type, required_capabilities) - Validate agent capabilities

#[derive(Debug, Clone, PartialEq)]
pub enum AgentType {
    AI,
    System,
    Worker,
    Custom(String),
}

impl AgentType {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "ai" => Some(AgentType::AI),
            "system" => Some(AgentType::System),
            "worker" => Some(AgentType::Worker),
            custom if custom.starts_with("custom:") => {
                Some(AgentType::Custom(custom[7..].to_string()))
            },
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            AgentType::AI => "ai".to_string(),
            AgentType::System => "system".to_string(),
            AgentType::Worker => "worker".to_string(),
            AgentType::Custom(name) => format!("custom:{}", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Idle,
    Active,
    Learning,
    Coordinating,
    Error,
    Terminated,
}

impl AgentStatus {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "idle" => Some(AgentStatus::Idle),
            "active" => Some(AgentStatus::Active),
            "learning" => Some(AgentStatus::Learning),
            "coordinating" => Some(AgentStatus::Coordinating),
            "error" => Some(AgentStatus::Error),
            "terminated" => Some(AgentStatus::Terminated),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            AgentStatus::Idle => "idle".to_string(),
            AgentStatus::Active => "active".to_string(),
            AgentStatus::Learning => "learning".to_string(),
            AgentStatus::Coordinating => "coordinating".to_string(),
            AgentStatus::Error => "error".to_string(),
            AgentStatus::Terminated => "terminated".to_string(),
        }
    }
}

/// Lifecycle hooks: DAL code executed at agent events. Set by molds.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LifecycleHooks {
    pub on_create: Option<String>,
    pub on_message: Option<String>,
    pub on_evolve: Option<String>,
    pub on_destroy: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: AgentType,
    pub role: String,
    pub capabilities: Vec<String>,
    pub trust_level: String,
    pub max_memory: usize,
    pub learning_enabled: bool,
    pub communication_enabled: bool,
    pub coordination_enabled: bool,
    pub metadata: HashMap<String, Value>,
    pub lifecycle: Option<LifecycleHooks>,
}

impl AgentConfig {
    pub fn new(name: String, agent_type: AgentType) -> Self {
        Self {
            name,
            agent_type,
            role: String::new(),
            capabilities: Vec::new(),
            trust_level: "standard".to_string(),
            max_memory: 1000,
            learning_enabled: true,
            communication_enabled: true,
            coordination_enabled: true,
            metadata: HashMap::new(),
            lifecycle: None,
        }
    }

    pub fn with_lifecycle(mut self, lifecycle: Option<LifecycleHooks>) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.role = role;
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_trust_level(mut self, trust_level: String) -> Self {
        self.trust_level = trust_level;
        self
    }

    pub fn with_max_memory(mut self, max_memory: usize) -> Self {
        self.max_memory = max_memory;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn enable_learning(mut self, enabled: bool) -> Self {
        self.learning_enabled = enabled;
        self
    }

    pub fn with_learning_enabled(mut self, enabled: bool) -> Self {
        self.learning_enabled = enabled;
        self
    }

    pub fn enable_communication(mut self, enabled: bool) -> Self {
        self.communication_enabled = enabled;
        self
    }

    pub fn with_communication_enabled(mut self, enabled: bool) -> Self {
        self.communication_enabled = enabled;
        self
    }

    pub fn enable_coordination(mut self, enabled: bool) -> Self {
        self.coordination_enabled = enabled;
        self
    }

    pub fn with_coordination_enabled(mut self, enabled: bool) -> Self {
        self.coordination_enabled = enabled;
        self
    }
}

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub agent_id: String,
    pub config: AgentConfig,
    pub status: AgentStatus,
    pub memory: HashMap<String, Value>,
    pub tasks: Vec<AgentTask>,
    pub message_queue: Vec<AgentMessage>,
    pub created_at: String,
    pub last_active: String,
    pub performance_metrics: AgentMetrics,
    pub trust_score: f64,
}

impl AgentContext {
    pub fn new(agent_id: String, config: AgentConfig) -> Self {
        Self {
            agent_id,
            config,
            status: AgentStatus::Idle,
            memory: HashMap::new(),
            tasks: Vec::new(),
            message_queue: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_active: chrono::Utc::now().to_rfc3339(),
            performance_metrics: AgentMetrics::new(),
            trust_score: 1.0,
        }
    }

    pub fn update_status(&mut self, status: AgentStatus) {
        self.status = status;
        self.last_active = chrono::Utc::now().to_rfc3339();
        self.performance_metrics.status_changes += 1;
    }

    pub fn add_task(&mut self, task: AgentTask) {
        self.tasks.push(task);
        self.performance_metrics.tasks_assigned += 1;
    }

    pub fn add_message(&mut self, message: AgentMessage) {
        self.message_queue.push(message);
        self.performance_metrics.messages_received += 1;
    }

    pub fn store_memory(&mut self, key: String, value: Value) {
        self.memory.insert(key, value);
        if self.memory.len() > self.config.max_memory {
            // Remove oldest entries if memory limit exceeded
            let keys_to_remove: Vec<String> = self.memory.keys()
                .take(self.memory.len() - self.config.max_memory)
                .cloned()
                .collect();
            for key in keys_to_remove {
                self.memory.remove(&key);
            }
        }
    }

    pub fn retrieve_memory(&self, key: &str) -> Option<&Value> {
        self.memory.get(key)
    }

    pub fn update_trust_score(&mut self, score_change: f64) {
        self.trust_score = (self.trust_score + score_change).max(0.0).min(1.0);
    }

    pub fn is_capable(&self, capability: &str) -> bool {
        self.config.capabilities.contains(&capability.to_string())
    }

    pub fn can_communicate(&self) -> bool {
        self.config.communication_enabled && self.status != AgentStatus::Error
    }

    pub fn can_coordinate(&self) -> bool {
        self.config.coordination_enabled && self.status != AgentStatus::Error
    }

    pub fn can_learn(&self) -> bool {
        self.config.learning_enabled && self.status != AgentStatus::Error
    }
}

#[derive(Debug, Clone)]
pub struct AgentTask {
    pub task_id: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub assigned_at: String,
    pub completed_at: Option<String>,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl TaskPriority {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "low" => Some(TaskPriority::Low),
            "medium" => Some(TaskPriority::Medium),
            "high" => Some(TaskPriority::High),
            "critical" => Some(TaskPriority::Critical),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TaskPriority::Low => "low".to_string(),
            TaskPriority::Medium => "medium".to_string(),
            TaskPriority::High => "high".to_string(),
            TaskPriority::Critical => "critical".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TaskStatus::Pending),
            "in_progress" => Some(TaskStatus::InProgress),
            "completed" => Some(TaskStatus::Completed),
            "failed" => Some(TaskStatus::Failed),
            "cancelled" => Some(TaskStatus::Cancelled),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TaskStatus::Pending => "pending".to_string(),
            TaskStatus::InProgress => "in_progress".to_string(),
            TaskStatus::Completed => "completed".to_string(),
            TaskStatus::Failed => "failed".to_string(),
            TaskStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub message_id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub message_type: String,
    pub content: Value,
    pub timestamp: String,
    pub priority: MessagePriority,
    pub requires_response: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl MessagePriority {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "low" => Some(MessagePriority::Low),
            "normal" => Some(MessagePriority::Normal),
            "high" => Some(MessagePriority::High),
            "urgent" => Some(MessagePriority::Urgent),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            MessagePriority::Low => "low".to_string(),
            MessagePriority::Normal => "normal".to_string(),
            MessagePriority::High => "high".to_string(),
            MessagePriority::Urgent => "urgent".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentMetrics {
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

impl AgentMetrics {
    pub fn new() -> Self {
        Self {
            tasks_completed: 0,
            tasks_failed: 0,
            tasks_assigned: 0,
            messages_sent: 0,
            messages_received: 0,
            coordination_events: 0,
            learning_sessions: 0,
            status_changes: 0,
            average_response_time: 0.0,
            uptime_percentage: 100.0,
        }
    }

    pub fn record_task_completion(&mut self, success: bool) {
        if success {
            self.tasks_completed += 1;
        } else {
            self.tasks_failed += 1;
        }
    }

    pub fn record_message(&mut self, sent: bool) {
        if sent {
            self.messages_sent += 1;
        } else {
            self.messages_received += 1;
        }
    }

    pub fn calculate_success_rate(&self) -> f64 {
        let total_tasks = self.tasks_completed + self.tasks_failed;
        if total_tasks == 0 {
            return 0.0;
        }
        self.tasks_completed as f64 / total_tasks as f64
    }

    pub fn get_performance_score(&self) -> f64 {
        let success_rate = self.calculate_success_rate();
        let activity_score = (self.messages_sent + self.messages_received) as f64 / 100.0;
        let coordination_score = self.coordination_events as f64 / 50.0;

        (success_rate * 0.5) + (activity_score * 0.3) + (coordination_score * 0.2)
    }
}

/// Spawn a new agent
pub fn spawn(config: AgentConfig) -> Result<AgentContext, String> {
    // Generate unique agent ID
    let agent_id = format!("agent_{}", generate_id());

    // Create agent context
    let mut agent_context = AgentContext::new(agent_id.clone(), config);

    // Initialize agent based on type
    match agent_context.config.agent_type {
        AgentType::AI => initialize_ai_agent(&mut agent_context),
        AgentType::System => initialize_system_agent(&mut agent_context),
        AgentType::Worker => initialize_worker_agent(&mut agent_context),
        AgentType::Custom(_) => initialize_custom_agent(&mut agent_context),
    }

    // Run on_create lifecycle hook if present (from mold)
    if let Some(ref lifecycle) = agent_context.config.lifecycle {
        if let Some(ref on_create) = lifecycle.on_create {
            let mut vars = HashMap::new();
            vars.insert("agent_id".to_string(), Value::String(agent_id.clone()));
            if let Err(e) = crate::execute_dal_with_scope(&vars, on_create) {
                log::warn!("Mold on_create lifecycle hook failed for {}: {}", agent_id, e);
                // Don't fail spawn; hook errors are non-fatal
            }
        }
    }

    // Log agent creation
    log::info!("agent: {:?}", {
        let mut data = HashMap::new();
        data.insert("agent_id".to_string(), Value::String(agent_id.clone()));
        data.insert("agent_type".to_string(), Value::String(agent_context.config.agent_type.to_string()));
        data.insert("agent_name".to_string(), Value::String(agent_context.config.name.clone()));
        data.insert("status".to_string(), Value::String(agent_context.status.to_string()));
        data.insert("message".to_string(), Value::String("Agent spawned successfully".to_string()));
        data
    });

    Ok(agent_context)
}

/// In-memory coordination runtime: task queue and message bus for coordinate/communicate.
struct AgentRuntime {
    task_queue: Vec<(String, AgentTask)>,
    message_bus: Vec<(String, AgentMessage)>,
    evolution_store: HashMap<String, HashMap<String, Value>>,
}

fn get_runtime() -> std::sync::MutexGuard<'static, AgentRuntime> {
    static RUNTIME: OnceLock<Mutex<AgentRuntime>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| Mutex::new(AgentRuntime {
            task_queue: Vec::new(),
            message_bus: Vec::new(),
            evolution_store: HashMap::new(),
        }))
        .lock()
        .unwrap()
}

/// Coordinate agent activities. Uses in-memory task queue; tasks are routed to agent_id.
pub fn coordinate(agent_id: &str, task: AgentTask, coordination_type: &str) -> Result<bool, String> {
    match coordination_type {
        "task_distribution" => {
            get_runtime().task_queue.push((agent_id.to_string(), task.clone()));
            log::info!("agent_coordination: {:?}", {
                let mut data = HashMap::new();
                data.insert("agent_id".to_string(), Value::String(agent_id.to_string()));
                data.insert("task_id".to_string(), Value::String(task.task_id.clone()));
                data.insert("coordination_type".to_string(), Value::String(coordination_type.to_string()));
                data.insert("message".to_string(), Value::String("Task distributed successfully".to_string()));
                data
            });
            Ok(true)
        },
        "resource_sharing" => {
            get_runtime().task_queue.push((agent_id.to_string(), task.clone()));
            log::info!("agent_coordination: {:?}", {
                let mut data = HashMap::new();
                data.insert("agent_id".to_string(), Value::String(agent_id.to_string()));
                data.insert("task_id".to_string(), Value::String(task.task_id.clone()));
                data.insert("coordination_type".to_string(), Value::String(coordination_type.to_string()));
                data.insert("message".to_string(), Value::String("Resources shared successfully".to_string()));
                data
            });
            Ok(true)
        },
        "conflict_resolution" => {
            get_runtime().task_queue.push((agent_id.to_string(), task.clone()));
            log::info!("agent_coordination: {:?}", {
                let mut data = HashMap::new();
                data.insert("agent_id".to_string(), Value::String(agent_id.to_string()));
                data.insert("task_id".to_string(), Value::String(task.task_id.clone()));
                data.insert("coordination_type".to_string(), Value::String(coordination_type.to_string()));
                data.insert("message".to_string(), Value::String("Conflict resolved successfully".to_string()));
                data
            });
            Ok(true)
        },
        _ => Err(format!("Unknown coordination type: {}", coordination_type))
    }
}

/// Returns pending tasks assigned to this agent (and removes them from the queue).
pub fn receive_pending_tasks(agent_id: &str) -> Vec<AgentTask> {
    let mut runtime = get_runtime();
    let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut runtime.task_queue)
        .into_iter()
        .partition(|(id, _)| id == agent_id);
    runtime.task_queue = rest;
    mine.into_iter().map(|(_, t)| t).collect()
}

/// Send message between agents. Routes via in-memory message bus; receiver can call receive_messages().
pub fn communicate(sender_id: &str, receiver_id: &str, message: AgentMessage) -> Result<bool, String> {
    get_runtime().message_bus.push((receiver_id.to_string(), message.clone()));
    log::info!("agent_communication: {:?}", {
        let mut data = HashMap::new();
        data.insert("sender_id".to_string(), Value::String(sender_id.to_string()));
        data.insert("receiver_id".to_string(), Value::String(receiver_id.to_string()));
        data.insert("message_id".to_string(), Value::String(message.message_id.clone()));
        data.insert("message_type".to_string(), Value::String(message.message_type.clone()));
        data.insert("message".to_string(), Value::String("Message sent successfully".to_string()));
        data
    });

    Ok(true)
}

/// Returns messages for this agent (and removes them from the bus).
pub fn receive_messages(receiver_id: &str) -> Vec<AgentMessage> {
    let mut runtime = get_runtime();
    let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut runtime.message_bus)
        .into_iter()
        .partition(|(id, _)| id == receiver_id);
    runtime.message_bus = rest;
    mine.into_iter().map(|(_, m)| m).collect()
}

/// Evolve agent through learning. Persists evolution_data in in-memory store; use get_evolution_data() to retrieve.
pub fn evolve(agent_id: &str, evolution_data: HashMap<String, Value>) -> Result<bool, String> {
    get_runtime()
        .evolution_store
        .insert(agent_id.to_string(), evolution_data.clone());
    log::info!("agent_evolution: {:?}", {
        let mut data = HashMap::new();
        data.insert("agent_id".to_string(), Value::String(agent_id.to_string()));
        data.insert("evolution_data_keys".to_string(), Value::Int(evolution_data.len() as i64));
        data.insert("message".to_string(), Value::String("Agent evolved successfully".to_string()));
        data
    });

    Ok(true)
}

/// Returns stored evolution data for an agent, if any.
pub fn get_evolution_data(agent_id: &str) -> Option<HashMap<String, Value>> {
    get_runtime().evolution_store.get(agent_id).cloned()
}

/// Capability registry: when set via register_capabilities(), validation uses it; otherwise built-in defaults.
static CAPABILITY_REGISTRY: Mutex<Option<HashMap<String, Vec<String>>>> = Mutex::new(None);

/// Register capabilities per agent type (e.g. from config/DB). Overrides built-in defaults for validation.
pub fn register_capabilities(agent_type: String, capabilities: Vec<String>) {
    let mut reg = CAPABILITY_REGISTRY.lock().unwrap();
    if reg.is_none() {
        *reg = Some(HashMap::new());
    }
    if let Some(ref mut m) = *reg {
        m.insert(agent_type, capabilities);
    }
}

/// Validate agent capabilities. Uses registry if set (e.g. from config); otherwise built-in per agent type.
pub fn validate_capabilities(agent_type: &str, required_capabilities: Vec<String>) -> Result<bool, String> {
    let agent_type_enum = AgentType::from_string(agent_type)
        .ok_or_else(|| format!("Invalid agent type: {}", agent_type))?;

    let capabilities: Vec<String> = {
        let reg = CAPABILITY_REGISTRY.lock().unwrap();
        if let Some(ref m) = *reg {
            if let Some(caps) = m.get(agent_type) {
                caps.clone()
            } else {
                builtin_capabilities(&agent_type_enum).into_iter().map(String::from).collect()
            }
        } else {
            builtin_capabilities(&agent_type_enum).into_iter().map(String::from).collect()
        }
    };

    let mut missing_capabilities = Vec::new();
    for required_cap in &required_capabilities {
        if !capabilities.contains(required_cap) {
            missing_capabilities.push(required_cap);
        }
    }

    if missing_capabilities.is_empty() {
        log::info!("capability_validation: {:?}", {
            let mut data = HashMap::new();
            data.insert("agent_type".to_string(), Value::String(agent_type.to_string()));
            data.insert("required_capabilities".to_string(), Value::Int(required_capabilities.len() as i64));
            data.insert("message".to_string(), Value::String("All capabilities validated".to_string()));
            data
        });
        Ok(true)
    } else {
        Err(format!("Missing capabilities: {:?}", missing_capabilities))
    }
}

/// Create agent configuration
pub fn create_agent_config(name: String, agent_type: &str, role: String) -> Option<AgentConfig> {
    let agent_type_enum = AgentType::from_string(agent_type)?;
    Some(AgentConfig::new(name, agent_type_enum).with_role(role))
}

/// Create agent task
pub fn create_agent_task(task_id: String, description: String, priority: &str) -> Option<AgentTask> {
    let priority_enum = TaskPriority::from_string(priority)?;
    Some(AgentTask {
        task_id,
        description,
        priority: priority_enum,
        status: TaskStatus::Pending,
        assigned_at: chrono::Utc::now().to_rfc3339(),
        completed_at: None,
        dependencies: Vec::new(),
        metadata: HashMap::new(),
    })
}

/// Create agent message
pub fn create_agent_message(message_id: String, sender_id: String, receiver_id: String, message_type: String, content: Value) -> AgentMessage {
    AgentMessage {
        message_id,
        sender_id,
        receiver_id,
        message_type,
        content,
        timestamp: chrono::Utc::now().to_rfc3339(),
        priority: MessagePriority::Normal,
        requires_response: false,
    }
}

// Helper functions

fn builtin_capabilities(agent_type: &AgentType) -> Vec<&'static str> {
    match agent_type {
        AgentType::AI => vec!["analysis", "learning", "communication", "task_execution"],
        AgentType::System => vec!["monitoring", "coordination", "resource_management"],
        AgentType::Worker => vec!["task_execution", "data_processing", "automation"],
        AgentType::Custom(_) => vec!["custom_processing"],
    }
}

fn generate_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

fn initialize_ai_agent(agent_context: &mut AgentContext) {
    // Initialize AI-specific capabilities
    agent_context.store_memory("ai_model".to_string(), Value::String("gpt-4".to_string()));
    agent_context.store_memory("learning_algorithm".to_string(), Value::String("reinforcement".to_string()));
    agent_context.store_memory("communication_protocol".to_string(), Value::String("natural_language".to_string()));

    // Set initial capabilities
    agent_context.config.capabilities = vec![
        "analysis".to_string(),
        "learning".to_string(),
        "communication".to_string(),
        "task_execution".to_string(),
        "problem_solving".to_string(),
    ];
}

fn initialize_system_agent(agent_context: &mut AgentContext) {
    // Initialize system-specific capabilities
    agent_context.store_memory("system_role".to_string(), Value::String("coordinator".to_string()));
    agent_context.store_memory("monitoring_enabled".to_string(), Value::Bool(true));
    agent_context.store_memory("resource_tracking".to_string(), Value::Bool(true));

    // Set initial capabilities
    agent_context.config.capabilities = vec![
        "monitoring".to_string(),
        "coordination".to_string(),
        "resource_management".to_string(),
        "system_optimization".to_string(),
    ];
}

fn initialize_worker_agent(agent_context: &mut AgentContext) {
    // Initialize worker-specific capabilities
    agent_context.store_memory("worker_type".to_string(), Value::String("general".to_string()));
    agent_context.store_memory("automation_level".to_string(), Value::String("high".to_string()));
    agent_context.store_memory("task_queue_size".to_string(), Value::Int(100));

    // Set initial capabilities
    agent_context.config.capabilities = vec![
        "task_execution".to_string(),
        "data_processing".to_string(),
        "automation".to_string(),
        "workflow_management".to_string(),
    ];
}

fn initialize_custom_agent(agent_context: &mut AgentContext) {
    // Initialize custom agent with basic capabilities
    agent_context.store_memory("custom_type".to_string(), Value::String(agent_context.config.agent_type.to_string()));
    agent_context.store_memory("flexibility".to_string(), Value::String("high".to_string()));

    // Set initial capabilities
    agent_context.config.capabilities = vec![
        "custom_processing".to_string(),
        "adaptation".to_string(),
        "flexibility".to_string(),
    ];
}

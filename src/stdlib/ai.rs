use crate::runtime::values::Value;
#[cfg(feature = "http-interface")]
use base64::Engine;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

// AI Agent Framework - Phase 4
// Comprehensive AI capabilities including:
// - Agent lifecycle management and spawning
// - Message passing and communication
// - AI processing (text, image, generation)
// - Agent coordination and orchestration
// - State management and persistence
// - Multi-agent collaboration
// - Multi-provider AI support (OpenAI, Anthropic, Local)
// - Flexible configuration (env, file, SDK, runtime)

// === AI CONFIGURATION ===

/// AI Provider Configuration
#[derive(Debug, Clone)]
pub struct AIConfig {
    pub provider: AIProvider,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AIProvider {
    OpenAI,
    Anthropic,
    Local,
    Custom(String),
    None,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            provider: AIProvider::None,
            api_key: None,
            endpoint: None,
            model: None,
            temperature: 0.7,
            max_tokens: 2000,
            timeout_seconds: 30,
        }
    }
}

// Global AI configuration cache
static AI_CONFIG: OnceLock<Mutex<AIConfig>> = OnceLock::new();

/// Effective OpenAI API key: OPENAI_API_KEY or DAL_OPENAI_API_KEY (for agents/tools that set only DAL_*).
fn effective_openai_api_key() -> Option<String> {
    env::var("OPENAI_API_KEY")
        .or_else(|_| env::var("DAL_OPENAI_API_KEY"))
        .ok()
        .filter(|k| !k.is_empty() && k != "none")
}

/// Effective Anthropic API key: ANTHROPIC_API_KEY or DAL_ANTHROPIC_API_KEY (same pattern as OpenAI).
fn effective_anthropic_api_key() -> Option<String> {
    env::var("ANTHROPIC_API_KEY")
        .or_else(|_| env::var("DAL_ANTHROPIC_API_KEY"))
        .ok()
        .filter(|k| !k.is_empty() && k != "none")
}

/// Effective local AI endpoint: DAL_AI_ENDPOINT (Local provider is already DAL-namespaced; no standard env).
fn effective_local_ai_endpoint() -> Option<String> {
    env::var("DAL_AI_ENDPOINT").ok().filter(|k| !k.is_empty())
}

/// Initialize AI configuration from multiple sources (priority order):
/// 1. Runtime configuration (if set)
/// 2. Environment variables
/// 3. Config file (.dal/ai_config.toml)
/// 4. Default fallback
pub fn init_ai_config() {
    let _config = AI_CONFIG.get_or_init(|| Mutex::new(load_ai_config()));
}

/// Load AI configuration from all sources
fn load_ai_config() -> AIConfig {
    let mut config = AIConfig::default();

    // Step 1: Try loading from config file
    if let Some(file_config) = load_config_file() {
        config = file_config;
    }

    // Step 2: Override with environment variables (higher priority)
    // Support both OPENAI_API_KEY and DAL_OPENAI_API_KEY so agents/tools that set only DAL_* work.
    let openai_key = env::var("OPENAI_API_KEY").or_else(|_| env::var("DAL_OPENAI_API_KEY"));
    if let Ok(key) = openai_key {
        if !key.is_empty() && key != "none" {
            config.provider = AIProvider::OpenAI;
            config.api_key = Some(key);
            let model = env::var("OPENAI_MODEL").or_else(|_| env::var("DAL_OPENAI_MODEL"));
            if let Ok(model) = model {
                config.model = Some(model);
            }
        }
    } else if let Some(key) = effective_anthropic_api_key() {
        config.provider = AIProvider::Anthropic;
        config.api_key = Some(key);
        let model = env::var("ANTHROPIC_MODEL").or_else(|_| env::var("DAL_ANTHROPIC_MODEL"));
        if let Ok(model) = model {
            config.model = Some(model);
        }
    } else if let Some(endpoint) = effective_local_ai_endpoint() {
        if !endpoint.is_empty() {
            config.provider = AIProvider::Local;
            config.endpoint = Some(endpoint);
            if let Ok(model) = env::var("DAL_AI_MODEL") {
                config.model = Some(model);
            }
        }
    }

    // Step 3: Apply optional configuration overrides
    if let Ok(temp) = env::var("DAL_AI_TEMPERATURE") {
        if let Ok(t) = temp.parse::<f32>() {
            config.temperature = t;
        }
    }

    if let Ok(tokens) = env::var("DAL_AI_MAX_TOKENS") {
        if let Ok(t) = tokens.parse::<u32>() {
            config.max_tokens = t;
        }
    }

    if let Ok(timeout) = env::var("DAL_AI_TIMEOUT") {
        if let Ok(t) = timeout.parse::<u64>() {
            config.timeout_seconds = t;
        }
    }

    config
}

/// Load configuration from .dal/ai_config.toml or dal_config.toml
fn load_config_file() -> Option<AIConfig> {
    // Try multiple locations
    let mut locations = vec![
        PathBuf::from(".dal/ai_config.toml"),
        PathBuf::from("dal_config.toml"),
        PathBuf::from(".dalconfig"),
    ];

    // Add home directory config if available
    if let Ok(home) = env::var("HOME") {
        locations.push(PathBuf::from(home).join(".dal/config.toml"));
    }

    for path in locations {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                return parse_config_file(&content);
            }
        }
    }

    None
}

/// Parse configuration file using TOML. Supports `[ai]` section or flat keys.
/// Falls back to legacy key=value parsing if TOML parse fails (e.g. simple .env-style files).
fn parse_config_file(content: &str) -> Option<AIConfig> {
    parse_config_file_toml(content).or_else(|| parse_config_file_legacy(content))
}

/// Parse config from proper TOML: root table or [ai] section.
fn parse_config_file_toml(content: &str) -> Option<AIConfig> {
    use toml::Value;

    let root: toml::Table = content.parse().ok()?;
    let table = root
        .get("ai")
        .and_then(Value::as_table)
        .map(|t| t as &toml::Table)
        .unwrap_or(&root);

    let mut config = AIConfig::default();
    let mut found_config = false;

    let str_val =
        |k: &str| -> Option<String> { table.get(k).and_then(Value::as_str).map(String::from) };
    let num_f32 = |k: &str| table.get(k).and_then(Value::as_float).map(|f| f as f32);
    let num_u32 = |k: &str| table.get(k).and_then(|v| v.as_integer()).map(|i| i as u32);
    let num_u64 = |k: &str| table.get(k).and_then(|v| v.as_integer()).map(|i| i as u64);

    if let Some(p) = str_val("provider") {
        config.provider = match p.to_lowercase().as_str() {
            "openai" => AIProvider::OpenAI,
            "anthropic" => AIProvider::Anthropic,
            "local" => AIProvider::Local,
            other => AIProvider::Custom(other.to_string()),
        };
        found_config = true;
    }
    config.api_key = str_val("api_key")
        .or_else(|| str_val("openai_api_key"))
        .or_else(|| str_val("anthropic_api_key"));
    config.endpoint = str_val("endpoint")
        .or_else(|| str_val("local_endpoint"))
        .or_else(|| str_val("dal_ai_endpoint"));
    config.model = str_val("model")
        .or_else(|| str_val("openai_model"))
        .or_else(|| str_val("anthropic_model"))
        .or_else(|| str_val("local_model"));
    if let Some(t) = num_f32("temperature") {
        config.temperature = t;
    }
    if let Some(t) = num_u32("max_tokens") {
        config.max_tokens = t;
    }
    if let Some(t) = num_u64("timeout").or_else(|| num_u64("timeout_seconds")) {
        config.timeout_seconds = t;
    }

    if found_config {
        Some(config)
    } else {
        None
    }
}

/// Legacy key=value parser for non-TOML config files (e.g. simple env-style).
fn parse_config_file_legacy(content: &str) -> Option<AIConfig> {
    let mut config = AIConfig::default();
    let mut found_config = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            match key {
                "provider" => {
                    config.provider = match value.to_lowercase().as_str() {
                        "openai" => AIProvider::OpenAI,
                        "anthropic" => AIProvider::Anthropic,
                        "local" => AIProvider::Local,
                        other => AIProvider::Custom(other.to_string()),
                    };
                    found_config = true;
                }
                "api_key" | "openai_api_key" | "anthropic_api_key" => {
                    config.api_key = Some(value.to_string());
                }
                "endpoint" | "local_endpoint" | "dal_ai_endpoint" => {
                    config.endpoint = Some(value.to_string());
                }
                "model" | "openai_model" | "anthropic_model" | "local_model" => {
                    config.model = Some(value.to_string());
                }
                "temperature" => {
                    if let Ok(t) = value.parse::<f32>() {
                        config.temperature = t;
                    }
                }
                "max_tokens" => {
                    if let Ok(t) = value.parse::<u32>() {
                        config.max_tokens = t;
                    }
                }
                "timeout" | "timeout_seconds" => {
                    if let Ok(t) = value.parse::<u64>() {
                        config.timeout_seconds = t;
                    }
                }
                _ => {}
            }
        }
    }

    if found_config {
        Some(config)
    } else {
        None
    }
}

/// Get current AI configuration
pub fn get_ai_config() -> AIConfig {
    init_ai_config();
    AI_CONFIG
        .get()
        .and_then(|mutex| mutex.lock().ok())
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

/// Set AI configuration at runtime
pub fn set_ai_config(config: AIConfig) {
    init_ai_config();
    if let Some(mutex) = AI_CONFIG.get() {
        if let Ok(mut guard) = mutex.lock() {
            *guard = config;
        }
    }
}

/// Configure AI provider at runtime
pub fn configure_openai(api_key: String, model: Option<String>) {
    let mut config = AIConfig::default();
    config.provider = AIProvider::OpenAI;
    config.api_key = Some(api_key);
    config.model = model;
    set_ai_config(config);
}

pub fn configure_anthropic(api_key: String, model: Option<String>) {
    let mut config = AIConfig::default();
    config.provider = AIProvider::Anthropic;
    config.api_key = Some(api_key);
    config.model = model;
    set_ai_config(config);
}

pub fn configure_local(endpoint: String, model: Option<String>) {
    let mut config = AIConfig::default();
    config.provider = AIProvider::Local;
    config.endpoint = Some(endpoint);
    config.model = model;
    set_ai_config(config);
}

/// Configure custom AI provider (Cohere, HuggingFace, Azure, etc.)
pub fn configure_custom(
    provider_name: String,
    endpoint: String,
    api_key: String,
    model: Option<String>,
) {
    let mut config = AIConfig::default();
    config.provider = AIProvider::Custom(provider_name);
    config.endpoint = Some(endpoint);
    config.api_key = Some(api_key);
    config.model = model;
    set_ai_config(config);
}

// Convenience functions for popular providers

pub fn configure_cohere(api_key: String, model: Option<String>) {
    configure_custom(
        "cohere".to_string(),
        "https://api.cohere.ai/v1/generate".to_string(),
        api_key,
        model,
    );
}

pub fn configure_huggingface(api_key: String, model: String) {
    let endpoint = format!("https://api-inference.huggingface.co/models/{}", model);
    configure_custom("huggingface".to_string(), endpoint, api_key, Some(model));
}

pub fn configure_azure_openai(endpoint: String, api_key: String, deployment_name: String) {
    configure_custom(
        "azure-openai".to_string(),
        endpoint,
        api_key,
        Some(deployment_name),
    );
}

pub fn configure_replicate(api_key: String, model_version: String) {
    configure_custom(
        "replicate".to_string(),
        "https://api.replicate.com/v1/predictions".to_string(),
        api_key,
        Some(model_version),
    );
}

pub fn configure_together_ai(api_key: String, model: Option<String>) {
    configure_custom(
        "together-ai".to_string(),
        "https://api.together.xyz/v1/chat/completions".to_string(),
        api_key,
        model,
    );
}

pub fn configure_openrouter(api_key: String, model: Option<String>) {
    configure_custom(
        "openrouter".to_string(),
        "https://openrouter.ai/api/v1/chat/completions".to_string(),
        api_key,
        model,
    );
}

// === PHASE 4: AI AGENT STRUCTURES ===

// Agent Configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub agent_id: String,
    pub name: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub memory_size: i64,
    pub max_concurrent_tasks: i64,
    pub trust_level: String,
    pub communication_protocols: Vec<String>,
    pub ai_models: Vec<String>,
}

// Agent Instance
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub config: AgentConfig,
    pub status: AgentStatus,
    pub memory: HashMap<String, Value>,
    pub tasks: Vec<Task>,
    pub message_queue: Vec<Message>,
    pub created_at: String,
    pub last_active: String,
}

#[derive(Debug, Clone)]
pub enum AgentStatus {
    Idle,
    Active,
    Busy,
    Error,
    Terminated,
}

// Message System
#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub from_agent: String,
    pub to_agent: String,
    pub message_type: String,
    pub content: Value,
    pub priority: MessagePriority,
    pub timestamp: String,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

// Task Management
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub agent_id: String,
    pub task_type: String,
    pub description: String,
    pub parameters: HashMap<String, Value>,
    pub status: TaskStatus,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub result: Option<Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// AI Processing Results
#[derive(Debug, Clone)]
pub struct TextAnalysis {
    pub sentiment: f64,
    pub entities: Vec<Entity>,
    pub keywords: Vec<String>,
    pub summary: String,
    pub language: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub text: String,
    pub entity_type: String,
    pub confidence: f64,
    pub start_pos: i64,
    pub end_pos: i64,
}

#[derive(Debug, Clone)]
pub struct ImageAnalysis {
    pub objects: Vec<DetectedObject>,
    pub faces: Vec<Face>,
    pub text: Vec<String>,
    pub colors: Vec<String>,
    pub quality_score: f64,
}

#[derive(Debug, Clone)]
pub struct DetectedObject {
    pub object_type: String,
    pub confidence: f64,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone)]
pub struct Face {
    pub bounding_box: BoundingBox,
    pub age: Option<i64>,
    pub gender: Option<String>,
    pub emotions: HashMap<String, f64>,
    pub confidence: f64,
}

// Training and Model Management
#[derive(Debug, Clone)]
pub struct TrainingData {
    pub data_type: String,
    pub samples: Vec<Value>,
    pub labels: Vec<Value>,
    pub features: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub model_id: String,
    pub model_type: String,
    pub version: String,
    pub accuracy: f64,
    pub training_data_size: i64,
    pub created_at: String,
    pub last_updated: String,
}

#[derive(Debug, Clone)]
pub struct Prediction {
    pub prediction: Value,
    pub confidence: f64,
    pub probabilities: HashMap<String, f64>,
    pub explanation: Option<String>,
}

// Agent Coordination
#[derive(Debug, Clone)]
pub struct AgentCoordinator {
    pub coordinator_id: String,
    pub agents: Vec<Agent>,
    pub workflows: Vec<Workflow>,
    pub active_tasks: Vec<Task>,
    pub message_bus: Vec<Message>,
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub workflow_id: String,
    pub name: String,
    pub steps: Vec<WorkflowStep>,
    pub status: WorkflowStatus,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub step_id: String,
    pub agent_id: String,
    pub task_type: String,
    pub dependencies: Vec<String>,
    pub status: StepStatus,
}

#[derive(Debug, Clone)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
}

#[derive(Debug, Clone)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

// Agent Communication
#[derive(Debug, Clone)]
pub struct CommunicationProtocol {
    pub protocol_id: String,
    pub name: String,
    pub supported_message_types: Vec<String>,
    pub encryption_enabled: bool,
    pub authentication_required: bool,
}

// === PHASE 4: AI AGENT FUNCTIONS ===

// Agent Lifecycle Management
pub fn spawn_agent(config: AgentConfig) -> Result<Agent, String> {
    crate::stdlib::log::info(
        "Spawning new AI agent",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_name".to_string(), Value::String(config.name.clone()));
            data.insert("agent_role".to_string(), Value::String(config.role.clone()));
            data.insert(
                "message".to_string(),
                Value::String("Spawning new AI agent".to_string()),
            );
            data
        },
        Some("ai"),
    );

    let mut agent = Agent {
        id: format!("agent_{}", generate_id()),
        config: config.clone(),
        status: AgentStatus::Idle,
        memory: HashMap::new(),
        tasks: Vec::new(),
        message_queue: Vec::new(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        last_active: "2024-01-01T00:00:00Z".to_string(),
    };

    // Initialize agent capabilities
    for capability in &config.capabilities {
        agent
            .memory
            .insert(format!("capability_{}", capability), Value::Bool(true));
    }

    Ok(agent)
}

pub fn terminate_agent(agent: &mut Agent) -> Result<bool, String> {
    crate::stdlib::log::info(
        "Terminating AI agent",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_id".to_string(), Value::String(agent.id.clone()));
            data.insert(
                "message".to_string(),
                Value::String("Terminating AI agent".to_string()),
            );
            data
        },
        Some("ai"),
    );

    agent.status = AgentStatus::Terminated;

    // Clean up resources
    agent.memory.clear();
    agent.tasks.clear();
    agent.message_queue.clear();

    Ok(true)
}

pub fn get_agent_status(agent: &Agent) -> String {
    match &agent.status {
        AgentStatus::Idle => "idle".to_string(),
        AgentStatus::Active => "active".to_string(),
        AgentStatus::Busy => "busy".to_string(),
        AgentStatus::Error => "error".to_string(),
        AgentStatus::Terminated => "terminated".to_string(),
    }
}

// Message Passing System
pub fn send_message(
    from_agent: &str,
    to_agent: &str,
    message_type: String,
    content: Value,
    priority: MessagePriority,
) -> Result<Message, String> {
    let message = Message {
        id: format!("msg_{}", generate_id()),
        from_agent: from_agent.to_string(),
        to_agent: to_agent.to_string(),
        message_type,
        content,
        priority,
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        correlation_id: None,
    };

    crate::stdlib::log::info(
        "Message sent between agents",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "from_agent".to_string(),
                Value::String(from_agent.to_string()),
            );
            data.insert("to_agent".to_string(), Value::String(to_agent.to_string()));
            data.insert(
                "message_type".to_string(),
                Value::String(message.message_type.clone()),
            );
            data.insert(
                "message".to_string(),
                Value::String("Message sent between agents".to_string()),
            );
            data
        },
        Some("ai"),
    );

    Ok(message)
}

pub fn receive_message(agent: &mut Agent, message: Message) -> Result<(), String> {
    crate::stdlib::log::info(
        "Message received by agent",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_id".to_string(), Value::String(agent.id.clone()));
            data.insert("message_id".to_string(), Value::String(message.id.clone()));
            data.insert(
                "message".to_string(),
                Value::String("Message received by agent".to_string()),
            );
            data
        },
        Some("ai"),
    );

    agent.message_queue.push(message);
    agent.last_active = "2024-01-01T00:00:00Z".to_string();

    Ok(())
}

pub fn process_message_queue(agent: &mut Agent) -> Result<Vec<Value>, String> {
    let mut results = Vec::new();

    let messages: Vec<_> = agent.message_queue.clone();
    for message in &messages {
        let result = process_message(agent, message)?;
        results.push(result);
    }

    agent.message_queue.clear();
    Ok(results)
}

pub fn process_message(agent: &mut Agent, message: &Message) -> Result<Value, String> {
    match message.message_type.as_str() {
        "text_analysis" => {
            if let Value::String(text) = &message.content {
                let analysis = analyze_text(text.clone())?;
                Ok(Value::String(format!(
                    "Text analysis: {}",
                    analysis.summary
                )))
            } else {
                Err("Invalid content type for text analysis".to_string())
            }
        }
        "image_analysis" => {
            // Simulated image analysis
            Ok(Value::String("Image analysis completed".to_string()))
        }
        "task_assignment" => {
            if let Value::Struct(_, task_data) = &message.content {
                let task = create_task_from_message(agent, task_data)?;
                agent.tasks.push(task);
                Ok(Value::String("Task assigned".to_string()))
            } else {
                Err("Invalid task data".to_string())
            }
        }
        _ => {
            // Generic message processing
            Ok(Value::String(format!(
                "Processed message: {}",
                message.message_type
            )))
        }
    }
}

// Task Management
pub fn create_task(
    agent: &mut Agent,
    task_type: String,
    description: String,
    parameters: HashMap<String, Value>,
) -> Result<Task, String> {
    let task = Task {
        id: format!("task_{}", generate_id()),
        agent_id: agent.id.clone(),
        task_type,
        description,
        parameters,
        status: TaskStatus::Pending,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        started_at: None,
        completed_at: None,
        result: None,
        error: None,
    };

    agent.tasks.push(task.clone());

    crate::stdlib::log::info(
        "Task created",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_id".to_string(), Value::String(agent.id.clone()));
            data.insert("task_id".to_string(), Value::String(task.id.clone()));
            data.insert(
                "task_type".to_string(),
                Value::String(task.task_type.clone()),
            );
            data.insert(
                "message".to_string(),
                Value::String("Task created".to_string()),
            );
            data
        },
        Some("ai"),
    );

    Ok(task)
}

pub fn create_task_from_message(
    agent: &mut Agent,
    task_data: &HashMap<String, Value>,
) -> Result<Task, String> {
    let task_type = task_data
        .get("task_type")
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "generic".to_string());

    let description = task_data
        .get("description")
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "Task from message".to_string());

    let parameters = task_data
        .get("parameters")
        .and_then(|v| match v {
            Value::Struct(_, s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| HashMap::new());

    create_task(agent, task_type, description, parameters)
}

pub fn execute_task(agent: &mut Agent, task_id: &str) -> Result<Value, String> {
    let task_index = agent
        .tasks
        .iter()
        .position(|t| t.id == task_id)
        .ok_or_else(|| format!("Task {} not found", task_id))?;

    // Clone the task to avoid borrow checker issues
    let task_clone = agent.tasks[task_index].clone();

    // Update the task status first
    {
        let task = &mut agent.tasks[task_index];
        task.status = TaskStatus::Running;
        task.started_at = Some("2024-01-01T00:00:00Z".to_string());
    }

    // Execute based on task type
    let result = match task_clone.task_type.as_str() {
        "text_analysis" => {
            if let Some(Value::String(text)) = task_clone.parameters.get("text") {
                let analysis = analyze_text(text.clone())?;
                Value::String(format!("Analysis: {}", analysis.summary))
            } else {
                Value::String("No text provided for analysis".to_string())
            }
        }
        "data_processing" => process_data_task(&task_clone)?,
        "communication" => handle_communication_task(agent, &task_clone)?,
        _ => Value::String(format!("Executed {} task", task_clone.task_type)),
    };

    // Update the task with results
    {
        let task = &mut agent.tasks[task_index];
        task.status = TaskStatus::Completed;
        task.completed_at = Some("2024-01-01T00:00:00Z".to_string());
        task.result = Some(result.clone());
    }

    crate::stdlib::log::info(
        "Task executed successfully",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_id".to_string(), Value::String(agent.id.clone()));
            data.insert("task_id".to_string(), Value::String(task_id.to_string()));
            data.insert(
                "message".to_string(),
                Value::String("Task executed successfully".to_string()),
            );
            data
        },
        Some("ai"),
    );

    Ok(result)
}

// AI Processing Functions
pub fn analyze_text(text: String) -> Result<TextAnalysis, String> {
    crate::stdlib::log::info(
        "Analyzing text",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("text_length".to_string(), Value::Int(text.len() as i64));
            data.insert(
                "message".to_string(),
                Value::String("Analyzing text".to_string()),
            );
            data
        },
        Some("ai"),
    );

    // Simulated text analysis
    let analysis = TextAnalysis {
        sentiment: 0.7,
        entities: vec![Entity {
            text: "example".to_string(),
            entity_type: "NOUN".to_string(),
            confidence: 0.9,
            start_pos: 0,
            end_pos: 7,
        }],
        keywords: vec!["example".to_string(), "text".to_string()],
        summary: format!("Summary of: {}", text),
        language: "en".to_string(),
        confidence: 0.85,
    };

    Ok(analysis)
}

/// Analyze image bytes. **Full API:** when an API key is configured (env OPENAI_API_KEY; any vision-capable provider), calls vision API and returns structured analysis. **Simplified:** returns mock objects/colors when no API key.
pub fn analyze_image(image_data: Vec<u8>) -> Result<ImageAnalysis, String> {
    crate::stdlib::log::info(
        "Analyzing image",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "image_size".to_string(),
                Value::Int(image_data.len() as i64),
            );
            data.insert(
                "message".to_string(),
                Value::String("Analyzing image".to_string()),
            );
            data
        },
        Some("ai"),
    );

    #[cfg(feature = "http-interface")]
    if let Some(api_key) = effective_openai_api_key() {
        let base = env::var("OPENAI_BASE_URL")
            .or_else(|_| env::var("DAL_OPENAI_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let svc = crate::stdlib::service::AIService::new("gpt-4o".to_string())
            .with_api_key(api_key)
            .with_base_url(base);
        let b64 = base64::engine::general_purpose::STANDARD.encode(&image_data);
        if let Ok(description) = crate::stdlib::service::vision_analyze(svc, None, Some(&b64)) {
            return Ok(ImageAnalysis {
                objects: vec![DetectedObject {
                    object_type: "described".to_string(),
                    confidence: 0.9,
                    bounding_box: BoundingBox {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                    },
                }],
                faces: vec![],
                text: vec![description],
                colors: vec![],
                quality_score: 0.9,
            });
        }
    }

    // Simplified: simulated image analysis
    let analysis = ImageAnalysis {
        objects: vec![DetectedObject {
            object_type: "person".to_string(),
            confidence: 0.95,
            bounding_box: BoundingBox {
                x: 100,
                y: 50,
                width: 200,
                height: 400,
            },
        }],
        faces: vec![],
        text: vec!["Sample text".to_string()],
        colors: vec!["blue".to_string(), "white".to_string()],
        quality_score: 0.88,
    };

    Ok(analysis)
}

/// Throttle "Generating text response" logs to at most once per 2 seconds.
static LAST_GENERATE_TEXT_LOG_MS: AtomicU64 = AtomicU64::new(0);
const GENERATE_TEXT_LOG_INTERVAL_MS: u64 = 2000;

pub fn generate_text(prompt: String) -> Result<String, String> {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let last = LAST_GENERATE_TEXT_LOG_MS.load(Ordering::Relaxed);
    if last == 0 || now_ms.saturating_sub(last) >= GENERATE_TEXT_LOG_INTERVAL_MS {
        LAST_GENERATE_TEXT_LOG_MS.store(now_ms, Ordering::Relaxed);
        crate::stdlib::log::info(
            "Generating text response",
            {
                let mut data = std::collections::HashMap::new();
                data.insert("prompt_length".to_string(), Value::Int(prompt.len() as i64));
                data.insert(
                    "message".to_string(),
                    Value::String("Generating text response".to_string()),
                );
                data
            },
            Some("ai"),
        );
    }

    // Load configuration (from env, file, or runtime)
    let config = get_ai_config();

    // Try configured provider first
    match &config.provider {
        AIProvider::OpenAI => {
            if let Some(ref api_key) = config.api_key {
                match call_openai_api(&prompt, api_key, &config) {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        eprintln!("OpenAI failed: {}. Trying fallback...", e);
                    }
                }
            }
        }
        AIProvider::Anthropic => {
            if let Some(ref api_key) = config.api_key {
                match call_anthropic_api(&prompt, api_key, &config) {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        eprintln!("Anthropic failed: {}. Trying fallback...", e);
                    }
                }
            }
        }
        AIProvider::Local => {
            if let Some(ref endpoint) = config.endpoint {
                match call_local_model(&prompt, endpoint, &config) {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        eprintln!("Local model failed: {}. Using fallback...", e);
                    }
                }
            }
        }
        AIProvider::Custom(ref provider_name) => {
            // Custom provider support - can be Cohere, HuggingFace, Azure, etc.
            if let Some(ref endpoint) = config.endpoint {
                if let Some(ref api_key) = config.api_key {
                    match call_custom_provider(&prompt, endpoint, api_key, provider_name, &config) {
                        Ok(response) => return Ok(response),
                        Err(e) => {
                            eprintln!(
                                "Custom provider '{}' failed: {}. Trying fallback...",
                                provider_name, e
                            );
                        }
                    }
                } else {
                    eprintln!(
                        "Custom provider '{}' requires api_key. Using fallback...",
                        provider_name
                    );
                }
            } else {
                eprintln!(
                    "Custom provider '{}' requires endpoint. Using fallback...",
                    provider_name
                );
            }
        }
        AIProvider::None => {
            // Fall through to automatic detection
        }
    }

    // Automatic provider detection (backward compatibility)
    // Priority: OpenAI > Anthropic > Local > Fallback

    if let Some(api_key) = effective_openai_api_key() {
        match call_openai_api(&prompt, &api_key, &config) {
            Ok(response) => return Ok(response),
            Err(e) => {
                eprintln!("OpenAI failed: {}. Trying next provider...", e);
            }
        }
    }

    if let Some(api_key) = effective_anthropic_api_key() {
        match call_anthropic_api(&prompt, &api_key, &config) {
            Ok(response) => return Ok(response),
            Err(e) => {
                eprintln!("Anthropic failed: {}. Trying next provider...", e);
            }
        }
    }

    if let Some(endpoint) = effective_local_ai_endpoint() {
        match call_local_model(&prompt, &endpoint, &config) {
            Ok(response) => return Ok(response),
            Err(e) => {
                eprintln!("Local model failed: {}. Using fallback...", e);
            }
        }
    }

    // Fallback to simulated response
    Ok(format!("Generated response to: {}", prompt))
}

/// System prompt for tool-using agent: reply, run shell, or search.
const TOOLS_SYSTEM: &str = "You are an intelligent assistant. You can run shell commands, search the web, reply, or ask the user. \
Use host tools through the API when needed, and answer users in natural language when finished. \
If legacy text-JSON mode is explicitly enabled, output exactly one JSON action object using: \
{\"action\":\"reply\",\"text\":\"your reply\"} or {\"action\":\"run\",\"cmd\":\"shell command\"} or {\"action\":\"search\",\"query\":\"search query\"} or {\"action\":\"ask_user\",\"message\":\"your question or status for the user\"}. \
For run and search the tool will execute and you will see the result; then reply once to complete the task. After a successful run (e.g. posting to X), reply immediately—do not run more steps. Use ask_user only if you need input. Keep the user in the loop: if you cannot finish, reply with what you did and what they should do next.";

/// Extended tools with file and DAL scripting: write_file, read_file, list_dir, dal_run, dal_check.
/// Use when AGENT_ASSISTANT_SCRIPTING=1 or AGENT_ASSISTANT_ROOT is set.
/// Public for IDE agent runner (prompt → code development).
pub const TOOLS_SYSTEM_WITH_SCRIPTING: &str = "You are an intelligent assistant. You can run shell commands, search the web, reply, ask the user, or use file/DAL tools. \
Use host tools through the API when needed, and answer users in natural language when finished. \
If legacy text-JSON mode is explicitly enabled, output exactly one JSON action object using: \
{\"action\":\"reply\",\"text\":\"your reply\"} or {\"action\":\"run\",\"cmd\":\"shell command\"} or {\"action\":\"search\",\"query\":\"search query\"} or {\"action\":\"ask_user\",\"message\":\"your question or status for the user\"} \
or {\"action\":\"write_file\",\"path\":\"path/to/file\",\"contents\":\"file contents\"} or {\"action\":\"read_file\",\"path\":\"path/to/file\"} or {\"action\":\"list_dir\",\"path\":\".\"} \
or {\"action\":\"dal_run\",\"path\":\"file.dal\"} or {\"action\":\"dal_check\",\"path\":\"file.dal\"} \
or {\"action\":\"show_url\",\"url\":\"https://...\"} or {\"action\":\"show_content\",\"content\":\"html or text\",\"title\":\"optional\"}. \
For run and search the tool will execute and you will see the result. For write_file, read_file, list_dir, dal_run, dal_check: paths are relative to the scripts root. Use write_file to create .dal or .sh scripts, then dal_run for DAL or run with bash for shell. After a successful run (e.g. posting to X), reply immediately—do not run more steps. Use ask_user only if you need input. Keep the user in the loop: if you cannot finish, reply with what you did and what they should do next.";

/// Completion and when to ask: try to finish; if you need input or must stop, keep the user in the loop.
/// Public for IDE agent runner.
pub const COMPLETION_AND_ASK_GUIDANCE: &str = "Complete in few steps: when a run succeeds (e.g. curl to post), use action reply right away with the outcome. Do not run extra checks or steps after success. If you need user input use ask_user; if something failed use reply to say what happened. Do not leave the user without a reply.";
const CHAT_REPLY_ONLY_SYSTEM: &str =
    "You are an intelligent assistant. Answer the user directly in natural language. \
Do not return JSON, code fences, or tool actions unless explicitly requested by the host.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRoute {
    ReplyOnly,
    ToolLoop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatPolicy {
    Auto,
    ReplyOnly,
    ToolLoop,
}

impl ChatPolicy {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        let v = s.trim().to_ascii_lowercase();
        match v.as_str() {
            "auto" => Some(Self::Auto),
            "reply_only" | "replyonly" => Some(Self::ReplyOnly),
            "tool_loop" | "toolloop" => Some(Self::ToolLoop),
            _ => None,
        }
    }
}

impl std::str::FromStr for ChatPolicy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ChatPolicy::from_str(s).ok_or(())
    }
}

/// Lightweight planner gate for chat surfaces:
/// - reply_only for simple conversational or conceptual questions
/// - tool_loop for action-oriented requests that likely need tools
pub fn decide_chat_route(user_message: &str) -> ChatRoute {
    let msg = user_message.trim().to_lowercase();
    if msg.is_empty() {
        return ChatRoute::ReplyOnly;
    }

    let starts_conversational = [
        "hi",
        "hello",
        "hey",
        "thanks",
        "thank you",
        "what is",
        "who is",
        "why ",
        "how does",
        "explain ",
    ]
    .iter()
    .any(|p| msg.starts_with(p));

    let action_markers = [
        "run ",
        "execute ",
        "search ",
        "look up",
        "find ",
        "open ",
        "read file",
        "write file",
        "create file",
        "edit file",
        "list ",
        "directory",
        "folder",
        "terminal",
        "shell",
        "command",
        "debug ",
        "fix ",
        "test ",
        "build ",
        "compile ",
        "deploy ",
        "check my ",
        "check the ",
        "check this ",
        "api ",
        "url",
        "website",
        "x page",
        "tweet",
    ]
    .iter()
    .any(|p| msg.contains(p));

    if action_markers {
        ChatRoute::ToolLoop
    } else if starts_conversational {
        ChatRoute::ReplyOnly
    } else {
        // Default to conversational path for ambiguous prompts.
        ChatRoute::ReplyOnly
    }
}

fn route_for_policy(policy: ChatPolicy, user_message: &str) -> ChatRoute {
    match policy {
        ChatPolicy::Auto => decide_chat_route(user_message),
        ChatPolicy::ReplyOnly => ChatRoute::ReplyOnly,
        ChatPolicy::ToolLoop => ChatRoute::ToolLoop,
    }
}

fn build_tool_loop_schema(
    user_message: &str,
) -> (
    crate::agent_context_schema::AgentContextSchema,
    Option<std::path::PathBuf>,
) {
    let scripting_enabled = std::env::var("AGENT_ASSISTANT_SCRIPTING").as_deref() == Ok("1")
        || std::env::var("AGENT_ASSISTANT_ROOT").is_ok();
    let (tools_system, working_root) = if scripting_enabled {
        let root = scripting_working_root();
        (TOOLS_SYSTEM_WITH_SCRIPTING, root)
    } else {
        (TOOLS_SYSTEM, None)
    };
    let mut schema =
        crate::agent_context_schema::AgentContextSchema::minimal(user_message, tools_system);
    schema.completion_and_ask_guidance = Some(COMPLETION_AND_ASK_GUIDANCE.to_string());
    (schema, working_root)
}

fn build_reply_only_schema(user_message: &str) -> crate::agent_context_schema::AgentContextSchema {
    let mut schema = crate::agent_context_schema::AgentContextSchema::minimal(
        user_message,
        CHAT_REPLY_ONLY_SYSTEM,
    );
    schema.completion_and_ask_guidance = Some(
        "Answer directly and clearly. Ask a brief follow-up question only if critical information is missing."
            .to_string(),
    );
    schema
}

/// Extract the first JSON object from a string (between first { and matching }).
fn extract_json_object(s: &str) -> Option<&str> {
    let start = s.find('{')?;
    let mut depth = 0u32;
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return std::str::from_utf8(&bytes[start..=i]).ok();
                }
            }
            _ => {}
        }
    }
    None
}

/// Run a web search via DuckDuckGo Instant Answer API and return a short summary string.
#[cfg(feature = "http-interface")]
fn search_web(query: &str) -> Result<String, String> {
    let encoded = urlencoding::encode(query).to_string();
    let url = format!("https://api.duckduckgo.com/?q={}&format=json", encoded);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Search API error: {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let abstract_text = json["AbstractText"].as_str().unwrap_or("");
    let abstract_url = json["AbstractURL"].as_str().unwrap_or("");
    let mut out = String::new();
    if !abstract_text.is_empty() {
        out.push_str(abstract_text);
        if !abstract_url.is_empty() {
            out.push_str(" (");
            out.push_str(abstract_url);
            out.push(')');
        }
    }
    if let Some(related) = json["RelatedTopics"].as_array() {
        for topic in related.iter().take(3) {
            let text = topic["Text"].as_str().unwrap_or("");
            if !text.is_empty() {
                if !out.is_empty() {
                    out.push_str("\n");
                }
                out.push_str(text);
            }
        }
    }
    if out.is_empty() {
        out = "No summary found for that query.".to_string();
    }
    Ok(out)
}

#[cfg(not(feature = "http-interface"))]
fn search_web(_query: &str) -> Result<String, String> {
    Err("Web search requires the http-interface feature.".to_string())
}

/// Public wrapper for agent/IDE: run web search. Requires http-interface feature.
pub fn run_web_search(query: &str) -> Result<String, String> {
    search_web(query)
}

/// Resolve working root for scripting: if AGENT_ASSISTANT_ROOT is set, use that/scripts (create if needed).
/// Returns None if env is unset or path cannot be resolved.
fn scripting_working_root() -> Option<std::path::PathBuf> {
    let root = std::env::var("AGENT_ASSISTANT_ROOT").ok()?;
    let root = std::path::Path::new(&root);
    let root = root.canonicalize().ok().or_else(|| {
        if root.exists() {
            Some(root.to_path_buf())
        } else {
            None
        }
    })?;
    let scripts = root.join("scripts");
    if !scripts.exists() {
        let _ = std::fs::create_dir_all(&scripts);
    }
    Some(scripts)
}

/// Agent that can reply, run shell commands, or search the web. Runs a multi-step loop until the
/// LLM returns a final reply (or ask_user / parse_fail / max steps). So after a "run" (e.g. curl
/// to post a tweet), the agent gets the tool result and continues until it sends a user-facing reply.
/// When AGENT_ASSISTANT_SCRIPTING=1 or AGENT_ASSISTANT_ROOT is set, exposes write_file, read_file,
/// list_dir, dal_run, dal_check and uses scripts/ under AGENT_ASSISTANT_ROOT as working root.
pub fn respond_with_tools(user_message: &str) -> Result<String, String> {
    respond_with_tools_result(user_message).map(|r| r.final_text)
}

pub fn respond_with_tools_with_policy(
    user_message: &str,
    policy: ChatPolicy,
) -> Result<String, String> {
    respond_with_tools_result_with_policy(user_message, policy).map(|r| r.final_text)
}

// --- Multi-step tool loop (production) ---

/// Result of parsing one LLM response as a tool call.
#[derive(Debug, Clone)]
pub enum ToolOutcome {
    /// Final reply to the user.
    Reply(String),
    /// Agent is asking for human input.
    AskUser(String),
    /// Execute shell command.
    Run(String),
    /// Execute web search.
    Search(String),
    /// Initialize DAL project (optional template: general, chain, iot, agent). Hard skill: project_init.
    DalInit(Option<String>),
    /// Read file (path relative to working dir). Development skill.
    ReadFile(String),
    /// Write file (path, contents). Development skill.
    WriteFile(String, String),
    /// List directory (path relative to working dir). Development skill / project_init.
    ListDir(String),
    /// Run `dal check <file>`. Development skill.
    DalCheck(String),
    /// Run `dal run <file>`. Development skill.
    DalRun(String),
    /// Show URL in IDE workspace (browser view). Operable by agents.
    ShowUrl(String),
    /// Show content (HTML or text) in IDE workspace. Operable by agents.
    ShowContent(String, Option<String>),
    /// Response was not valid JSON or unknown action; treat as reply with raw text.
    ParseFail(String),
}

/// One provider-native tool call emitted by the model.
#[derive(Debug, Clone)]
pub struct NativeToolCall {
    pub id: Option<String>,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct TurnUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub estimated_cost_microusd: Option<u64>,
}

/// One model turn used by the host protocol loop.
#[derive(Debug, Clone)]
pub struct AgentModelTurn {
    /// Optional assistant text content for the user.
    pub content: String,
    /// Native tool/function calls requested by the provider response.
    pub tool_calls: Vec<NativeToolCall>,
    /// Optional provider token/cost usage for budget guards.
    pub usage: TurnUsage,
}

/// Parse LLM response into a tool outcome. Uses same JSON shape as SERVE_*_TOOLS.
/// Accepts "command" as alias for "cmd", and matches action case-insensitively.
pub fn parse_tool_response(response: &str) -> ToolOutcome {
    let response = response.trim();
    // Strip markdown code fences if present (e.g. ```json ... ```)
    let cleaned = response
        .strip_prefix("```json")
        .or_else(|| response.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```").map(|s| s.trim()))
        .unwrap_or(response);
    let json_str = match extract_json_object(cleaned) {
        Some(s) => s,
        None => return ToolOutcome::ParseFail(response.to_string()),
    };
    let v: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(x) => x,
        Err(_) => return ToolOutcome::ParseFail(response.to_string()),
    };
    let obj = match v.as_object() {
        Some(o) => o,
        None => return ToolOutcome::ParseFail(response.to_string()),
    };
    let action = obj
        .get("action")
        .and_then(|a| a.as_str())
        .unwrap_or("reply")
        .to_lowercase();
    let action = action.as_str();
    match action {
        "ask_user" => {
            let msg = obj
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .trim();
            ToolOutcome::AskUser(if msg.is_empty() {
                response.to_string()
            } else {
                msg.to_string()
            })
        }
        "run" => {
            let cmd = obj
                .get("cmd")
                .or_else(|| obj.get("command"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .trim();
            ToolOutcome::Run(cmd.to_string())
        }
        "search" => {
            let query = obj
                .get("query")
                .and_then(|q| q.as_str())
                .unwrap_or("")
                .trim();
            ToolOutcome::Search(query.to_string())
        }
        "dal_init" => {
            let template = obj
                .get("template")
                .and_then(|t| t.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            ToolOutcome::DalInit(template)
        }
        "read_file" => {
            let path = obj
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::ReadFile(path)
        }
        "write_file" => {
            let path = obj
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let contents = obj
                .get("contents")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            ToolOutcome::WriteFile(path, contents)
        }
        "list_dir" => {
            let path = obj
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or(".")
                .trim()
                .to_string();
            ToolOutcome::ListDir(path)
        }
        "dal_check" => {
            let path = obj
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::DalCheck(path)
        }
        "dal_run" => {
            let path = obj
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::DalRun(path)
        }
        "show_url" => {
            let url = obj
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::ShowUrl(url)
        }
        "show_content" => {
            let content = obj
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            let title = obj
                .get("title")
                .and_then(|t| t.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            ToolOutcome::ShowContent(content, title)
        }
        _ => {
            let text = obj
                .get("text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .trim();
            ToolOutcome::Reply(if text.is_empty() {
                response.to_string()
            } else {
                text.to_string()
            })
        }
    }
}

/// Legacy JSON-in-text parser gate. Keep disabled by default for production protocol.
pub fn legacy_text_tool_protocol_enabled() -> bool {
    std::env::var("DAL_AGENT_ENABLE_LEGACY_TEXT_JSON")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

pub fn native_tool_calling_enabled() -> bool {
    std::env::var("DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            !matches!(v.as_str(), "0" | "false" | "no" | "off")
        })
        .unwrap_or(true)
}

pub fn default_chat_policy_from_env() -> ChatPolicy {
    std::env::var("DAL_AGENT_POLICY_DEFAULT")
        .ok()
        .and_then(|s| ChatPolicy::from_str(&s))
        .unwrap_or(ChatPolicy::Auto)
}

fn native_tool_call_to_outcome(call: &NativeToolCall) -> ToolOutcome {
    let name = call.name.trim().to_ascii_lowercase();
    let obj = call.arguments.as_object();
    match name.as_str() {
        "ask_user" => {
            let msg = obj
                .and_then(|o| o.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::AskUser(msg)
        }
        "reply" => {
            let text = obj
                .and_then(|o| o.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::Reply(text)
        }
        "run" => {
            let cmd = obj
                .and_then(|o| o.get("cmd").or_else(|| o.get("command")))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::Run(cmd)
        }
        "search" => {
            let query = obj
                .and_then(|o| o.get("query"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::Search(query)
        }
        "dal_init" => {
            let template = obj
                .and_then(|o| o.get("template"))
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            ToolOutcome::DalInit(template)
        }
        "read_file" => {
            let path = obj
                .and_then(|o| o.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::ReadFile(path)
        }
        "write_file" => {
            let path = obj
                .and_then(|o| o.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let contents = obj
                .and_then(|o| o.get("contents"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            ToolOutcome::WriteFile(path, contents)
        }
        "list_dir" => {
            let path = obj
                .and_then(|o| o.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or(".")
                .trim()
                .to_string();
            ToolOutcome::ListDir(path)
        }
        "dal_check" => {
            let path = obj
                .and_then(|o| o.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::DalCheck(path)
        }
        "dal_run" => {
            let path = obj
                .and_then(|o| o.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::DalRun(path)
        }
        "show_url" => {
            let url = obj
                .and_then(|o| o.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            ToolOutcome::ShowUrl(url)
        }
        "show_content" => {
            let content = obj
                .and_then(|o| o.get("content"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = obj
                .and_then(|o| o.get("title"))
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            ToolOutcome::ShowContent(content, title)
        }
        other => ToolOutcome::ParseFail(format!("Unsupported tool call: {}", other)),
    }
}

fn tool_call_for_conversation(call: &NativeToolCall) -> String {
    let id = call.id.clone().unwrap_or_else(|| "tool_call".to_string());
    format!(
        "{{\"tool_call_id\":\"{}\",\"name\":\"{}\",\"arguments\":{}}}",
        id, call.name, call.arguments
    )
}

fn parse_tool_call_from_conversation(content: &str) -> Option<NativeToolCall> {
    let v: serde_json::Value = serde_json::from_str(content.trim()).ok()?;
    let obj = v.as_object()?;
    let id = obj
        .get("tool_call_id")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    let name = obj.get("name").and_then(|x| x.as_str())?.to_string();
    let arguments = obj
        .get("arguments")
        .and_then(|x| x.as_object())
        .map(|m| serde_json::Value::Object(m.clone()))?;
    Some(NativeToolCall {
        id,
        name,
        arguments,
    })
}

fn parse_tool_result_from_conversation(content: &str) -> Option<String> {
    let prefix = "[Tool result]\n";
    if let Some(rest) = content.strip_prefix(prefix) {
        return Some(rest.to_string());
    }
    None
}

#[derive(Debug, Clone)]
enum TranscriptEvent {
    UserText(String),
    AssistantText(String),
    AssistantToolCall(NativeToolCall),
    ToolResult {
        tool_call_id: String,
        content: String,
    },
}

fn build_transcript_events(
    schema: &crate::agent_context_schema::AgentContextSchema,
) -> Vec<TranscriptEvent> {
    let mut events = Vec::new();
    if !schema.objective.trim().is_empty() {
        events.push(TranscriptEvent::UserText(
            schema.objective.trim().to_string(),
        ));
    }
    let mut pending_tool_call_id: Option<String> = None;
    for turn in &schema.conversation {
        let role = turn.role.trim().to_ascii_lowercase();
        if role == "assistant" {
            if let Some(tool_call) = parse_tool_call_from_conversation(&turn.content) {
                pending_tool_call_id = tool_call.id.clone();
                events.push(TranscriptEvent::AssistantToolCall(tool_call));
            } else {
                pending_tool_call_id = None;
                events.push(TranscriptEvent::AssistantText(turn.content.clone()));
            }
            continue;
        }
        if role == "user" {
            if let Some(tool_call_id) = pending_tool_call_id.clone() {
                if let Some(tool_result) = parse_tool_result_from_conversation(&turn.content) {
                    events.push(TranscriptEvent::ToolResult {
                        tool_call_id,
                        content: tool_result,
                    });
                    pending_tool_call_id = None;
                    continue;
                }
            }
            pending_tool_call_id = None;
            events.push(TranscriptEvent::UserText(turn.content.clone()));
        }
    }
    events
}

fn build_provider_system_prompt(
    schema: &crate::agent_context_schema::AgentContextSchema,
) -> String {
    let mut out = String::from(
        "You are an expert dist_agent_lang (DAL) programmer. Use tools when needed and answer users in natural language.",
    );
    if !schema.tools_description.trim().is_empty() {
        out.push_str("\n\n## Tools\n");
        out.push_str(schema.tools_description.trim());
    }
    if let Some(constraints) = &schema.constraints {
        if !constraints.trim().is_empty() {
            out.push_str("\n\n## Constraints\n");
            out.push_str(constraints.trim());
        }
    }
    if let Some(guidance) = &schema.completion_and_ask_guidance {
        if !guidance.trim().is_empty() {
            out.push_str("\n\n## Completion and when to ask human\n");
            out.push_str(guidance.trim());
        }
    }
    if !schema.context_blocks.is_empty() {
        out.push_str("\n\n## Context\n");
        for block in &schema.context_blocks {
            if !block.source.trim().is_empty() {
                out.push('[');
                out.push_str(block.source.trim());
                out.push_str("]\n");
            }
            out.push_str(block.content.trim());
            out.push('\n');
        }
    }
    out
}

pub fn generate_agent_model_turn(
    schema: &crate::agent_context_schema::AgentContextSchema,
    include_scripting_tools: bool,
) -> Result<AgentModelTurn, String> {
    let config = get_ai_config();
    if native_tool_calling_enabled() && !legacy_text_tool_protocol_enabled() {
        match &config.provider {
            AIProvider::OpenAI => {
                if let Some(ref api_key) = config.api_key {
                    if let Ok(turn) =
                        call_openai_api_tool_turn(schema, api_key, &config, include_scripting_tools)
                    {
                        return Ok(turn);
                    }
                }
            }
            AIProvider::Anthropic => {
                if let Some(ref api_key) = config.api_key {
                    if let Ok(turn) = call_anthropic_api_tool_turn(
                        schema,
                        api_key,
                        &config,
                        include_scripting_tools,
                    ) {
                        return Ok(turn);
                    }
                }
            }
            _ => {}
        }
    }
    let prompt = crate::agent_context_schema::build_prompt_for_llm(schema);
    let response = generate_text(prompt)?;
    Ok(AgentModelTurn {
        content: response.trim().to_string(),
        tool_calls: Vec::new(),
        usage: TurnUsage::default(),
    })
}

pub struct ParsedTurnOutcome {
    pub outcome: ToolOutcome,
    pub assistant_event: String,
    pub used_native_tool_call: bool,
    pub used_legacy_parse: bool,
}

pub fn model_turn_to_outcome(turn: &AgentModelTurn) -> ParsedTurnOutcome {
    if let Some(call) = turn.tool_calls.first() {
        return ParsedTurnOutcome {
            outcome: native_tool_call_to_outcome(call),
            assistant_event: tool_call_for_conversation(call),
            used_native_tool_call: true,
            used_legacy_parse: false,
        };
    }
    if legacy_text_tool_protocol_enabled() {
        let parsed = parse_tool_response(&turn.content);
        return ParsedTurnOutcome {
            outcome: parsed,
            assistant_event: turn.content.clone(),
            used_native_tool_call: false,
            used_legacy_parse: true,
        };
    }
    ParsedTurnOutcome {
        outcome: ToolOutcome::Reply(turn.content.clone()),
        assistant_event: turn.content.clone(),
        used_native_tool_call: false,
        used_legacy_parse: false,
    }
}

const MAX_TOOL_RESULT_LEN: usize = 4000;

/// Strip curl progress meter from stderr so agent replies stay clean (exit 0, progress in stderr).
fn strip_curl_progress(stderr: &str) -> &str {
    let t = stderr.trim();
    if t.is_empty() {
        return stderr;
    }
    // Single line: entire stderr is often one line (header + stats space-separated)
    if !t.contains('\n') {
        let looks_like_progress = t.starts_with('%')
            || (t.contains("Total")
                && (t.contains("Received") || t.contains("Dload") || t.contains("Upload"))
                && (t.contains("Speed") || t.chars().any(|c| c.is_ascii_digit())));
        if looks_like_progress {
            return "";
        }
    }
    let mut all_progress = true;
    for line in t.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Curl progress: starts with % or is a line of numbers/colons (e.g. "  0 100 160 0 ... 156k")
        let looks_like_progress = line.starts_with('%')
            || (line.contains("Total")
                && (line.contains("Received")
                    || line.contains("Dload")
                    || line.contains("Upload")))
            || line.chars().all(|c| {
                c.is_ascii_digit()
                    || c == ' '
                    || c == '\t'
                    || c == 'k'
                    || c == 'M'
                    || c == '-'
                    || c == ':'
            });
        if !looks_like_progress {
            all_progress = false;
            break;
        }
    }
    if all_progress {
        ""
    } else {
        stderr
    }
}

fn execute_run_result(cmd: &str) -> String {
    if cmd.is_empty() {
        return "No command provided.".to_string();
    }
    match crate::stdlib::sh::run(cmd) {
        Ok(Value::Map(m)) => {
            let stdout = m
                .get("stdout")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or("");
            let stderr_raw = m
                .get("stderr")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or("");
            let stderr = strip_curl_progress(stderr_raw);
            let code = m
                .get("exit_code")
                .and_then(|v| {
                    if let Value::Int(n) = v {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .unwrap_or(-1);
            let mut out = format!("Exit code: {}\n", code);
            if !stdout.is_empty() {
                out.push_str("stdout:\n");
                out.push_str(stdout);
            }
            if !stderr.is_empty() {
                out.push_str("\nstderr:\n");
                out.push_str(stderr);
            }
            if stdout.is_empty() && stderr.is_empty() {
                out.push_str("(no output)");
            }
            if out.len() > MAX_TOOL_RESULT_LEN {
                out.truncate(MAX_TOOL_RESULT_LEN);
                out.push_str("\n... (truncated)");
            }
            out
        }
        Ok(_) => "Command completed.".to_string(),
        Err(e) => format!("Command failed: {}", e),
    }
}

fn execute_search_result(query: &str) -> String {
    if query.is_empty() {
        return "No search query provided.".to_string();
    }
    match search_web(query) {
        Ok(summary) => {
            if summary.len() > MAX_TOOL_RESULT_LEN {
                format!("{}\n... (truncated)", &summary[..MAX_TOOL_RESULT_LEN])
            } else {
                summary
            }
        }
        Err(e) => format!("Search failed: {}", e),
    }
}

/// Execute dal_init (project_init hard skill). Template: general, chain, iot, agent.
fn execute_dal_init_result(template: Option<&str>, root: &std::path::Path) -> String {
    let t = template.unwrap_or("general");
    match crate::project_init::run_init(t, root) {
        Ok(msg) => msg,
        Err(e) => format!("dal_init failed: {}", e),
    }
}

/// Resolve path relative to root; reject path traversal. Returns Err if path escapes root.
fn resolve_path_under_root(
    root: &std::path::Path,
    path: &str,
) -> Result<std::path::PathBuf, String> {
    let path = path.trim();
    if path.is_empty() {
        return Ok(root.to_path_buf());
    }
    if path.contains("..") {
        return Err("Path traversal (..) not allowed".to_string());
    }
    if path.starts_with('/') || (path.len() >= 2 && path.get(..2) == Some("\\\\")) {
        return Err("Absolute paths not allowed".to_string());
    }
    let root_canonical = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => root.to_path_buf(),
    };
    let joined = root_canonical.join(path);
    if joined.exists() {
        let canonical = joined.canonicalize().map_err(|e| e.to_string())?;
        if !canonical.starts_with(&root_canonical) {
            return Err("Path escapes working directory".to_string());
        }
        Ok(canonical)
    } else {
        if !joined.starts_with(&root_canonical) {
            return Err("Path escapes working directory".to_string());
        }
        Ok(joined)
    }
}

fn execute_read_file_result(path: &str, root: &std::path::Path) -> String {
    match resolve_path_under_root(root, path) {
        Err(e) => format!("read_file failed: {}", e),
        Ok(p) => {
            if !p.is_file() {
                return "read_file failed: not a file".to_string();
            }
            match std::fs::read_to_string(&p) {
                Ok(s) => {
                    if s.len() > MAX_TOOL_RESULT_LEN {
                        format!("{}\n... (truncated)", &s[..MAX_TOOL_RESULT_LEN])
                    } else {
                        s
                    }
                }
                Err(e) => format!("read_file failed: {}", e),
            }
        }
    }
}

fn execute_write_file_result(path: &str, contents: &str, root: &std::path::Path) -> String {
    match resolve_path_under_root(root, path) {
        Err(e) => format!("write_file failed: {}", e),
        Ok(p) => {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&p, contents) {
                Ok(()) => format!("Wrote {} ({} bytes).", p.display(), contents.len()),
                Err(e) => format!("write_file failed: {}", e),
            }
        }
    }
}

fn execute_list_dir_result(path: &str, root: &std::path::Path) -> String {
    match resolve_path_under_root(root, path) {
        Err(e) => format!("list_dir failed: {}", e),
        Ok(p) => {
            if !p.is_dir() {
                return "list_dir failed: not a directory".to_string();
            }
            match std::fs::read_dir(&p) {
                Ok(entries) => {
                    let mut names: Vec<String> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| {
                            let name = e.file_name().to_string_lossy().into_owned();
                            if e.path().is_dir() {
                                format!("{}/", name)
                            } else {
                                name
                            }
                        })
                        .collect();
                    names.sort();
                    names.join("\n")
                }
                Err(e) => format!("list_dir failed: {}", e),
            }
        }
    }
}

fn execute_dal_check_result(path: &str, root: &std::path::Path) -> String {
    match resolve_path_under_root(root, path) {
        Err(e) => format!("dal_check failed: {}", e),
        Ok(p) => {
            if !p.is_file() {
                return "dal_check failed: path is not a file".to_string();
            }
            let path_str = p.to_string_lossy().into_owned();
            run_dal_subcommand("check", &[&path_str], root)
        }
    }
}

fn execute_dal_run_result(path: &str, root: &std::path::Path) -> String {
    match resolve_path_under_root(root, path) {
        Err(e) => format!("dal_run failed: {}", e),
        Ok(p) => {
            if !p.is_file() {
                return "dal_run failed: path is not a file".to_string();
            }
            let path_str = p.to_string_lossy().into_owned();
            run_dal_subcommand("run", &[&path_str], root)
        }
    }
}

/// Run `dal <subcommand> <args...>` from root. Uses current binary so it works without PATH.
fn run_dal_subcommand(subcommand: &str, args: &[&str], root: &std::path::Path) -> String {
    let cwd = root;
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => {
            return "dal_run/dal_check failed: could not get current executable".to_string();
        }
    };
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg(subcommand).args(args).current_dir(cwd);
    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let mut s = format!("Exit code: {}\n", out.status.code().unwrap_or(-1));
            if !stdout.is_empty() {
                s.push_str("stdout:\n");
                s.push_str(&stdout);
            }
            if !stderr.is_empty() {
                s.push_str("\nstderr:\n");
                s.push_str(&stderr);
            }
            if s.len() > MAX_TOOL_RESULT_LEN {
                s.truncate(MAX_TOOL_RESULT_LEN);
                s.push_str("\n... (truncated)");
            }
            s
        }
        Err(e) => format!("dal {} failed: {}", subcommand, e),
    }
}

/// Result of the multi-step tool loop.
#[derive(Debug, Clone)]
pub struct MultiStepResult {
    /// Final text to send to the user (reply or ask_user message).
    pub final_text: String,
    /// True if the agent requested human input (ask_user).
    pub is_ask_user: bool,
    /// Number of tool steps executed (0 = immediate reply).
    pub steps_used: u32,
    /// True if the loop stopped because the step limit was reached (no final reply from the model).
    pub max_steps_reached: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminationDiagnostics {
    pub termination_reason: &'static str,
    pub guard_stopped: bool,
    pub parse_fail_terminal: bool,
    pub unsupported_tool_call_terminal: bool,
}

pub fn classify_termination(result: &MultiStepResult) -> TerminationDiagnostics {
    let parse_fail_terminal = if result.max_steps_reached {
        false
    } else {
        result.steps_used == 0
            && !result.is_ask_user
            && !result.final_text.is_empty()
            && legacy_text_tool_protocol_enabled()
    };
    let unsupported_tool_call_terminal = result.final_text.starts_with("Unsupported tool call:");
    let guard_stopped = result.final_text.starts_with("Stopped:");
    let termination_reason = if result.max_steps_reached {
        "max_steps_reached"
    } else if result.is_ask_user {
        "ask_user"
    } else if guard_stopped {
        if result.final_text.contains("wall-clock limit exceeded") {
            "guard_wall_clock"
        } else if result.final_text.contains("token budget exceeded") {
            "guard_token_budget"
        } else if result.final_text.contains("cost budget exceeded") {
            "guard_cost_budget"
        } else if result.final_text.contains("exceeded per-type limit") {
            "guard_per_tool_type_limit"
        } else if result
            .final_text
            .contains("repeated identical tool invocation")
        {
            "guard_repeated_identical_invocation"
        } else if result.final_text.contains("consecutive no-progress loop") {
            "guard_no_progress"
        } else {
            "guard_other"
        }
    } else if parse_fail_terminal {
        "parse_fail_terminal"
    } else if unsupported_tool_call_terminal {
        "unsupported_tool_call_terminal"
    } else {
        "reply"
    };
    TerminationDiagnostics {
        termination_reason,
        guard_stopped,
        parse_fail_terminal,
        unsupported_tool_call_terminal,
    }
}

/// Default max tool steps when env DAL_AGENT_MAX_TOOL_STEPS is not set.
pub const DEFAULT_MAX_TOOL_STEPS: u32 = 40;

/// Read max tool steps from env DAL_AGENT_MAX_TOOL_STEPS (default DEFAULT_MAX_TOOL_STEPS, clamped 1..=80).
pub fn max_tool_steps_from_env() -> u32 {
    std::env::var("DAL_AGENT_MAX_TOOL_STEPS")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(DEFAULT_MAX_TOOL_STEPS)
        .clamp(1, 80)
}

#[derive(Debug, Clone)]
struct ToolLoopGuards {
    max_wall_clock_ms: u64,
    max_tool_calls_per_type: u32,
    max_repeated_identical_invocations: u32,
    max_consecutive_no_progress: u32,
    max_total_tokens: u64,
    max_estimated_cost_microusd: u64,
}

fn tool_loop_guards_from_env() -> ToolLoopGuards {
    let env_u64 = |key: &str, default: u64| {
        std::env::var(key)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(default)
    };
    let env_u32 = |key: &str, default: u32| {
        std::env::var(key)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(default)
    };
    let strict_mode = std::env::var("DAL_AGENT_GUARDS_STRICT_MODE")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false);
    ToolLoopGuards {
        // Bound total wall clock for one tool-loop turn.
        max_wall_clock_ms: env_u64(
            "DAL_AGENT_MAX_WALL_CLOCK_MS",
            if strict_mode { 90_000 } else { 120_000 },
        ),
        // Bound each tool type to avoid runaway repetition.
        max_tool_calls_per_type: env_u32(
            "DAL_AGENT_MAX_TOOL_CALLS_PER_TYPE",
            if strict_mode { 8 } else { 12 },
        ),
        // Bound repeated identical invocations (same tool + same args).
        max_repeated_identical_invocations: env_u32(
            "DAL_AGENT_MAX_REPEATED_IDENTICAL_INVOCATIONS",
            if strict_mode { 2 } else { 3 },
        ),
        // Bound consecutive no-progress loops (same tool + args + same result).
        max_consecutive_no_progress: env_u32(
            "DAL_AGENT_MAX_CONSECUTIVE_NO_PROGRESS",
            if strict_mode { 1 } else { 2 },
        ),
        // Token/cost caps are opt-in; 0 means disabled.
        max_total_tokens: env_u64("DAL_AGENT_MAX_TOTAL_TOKENS", 0),
        max_estimated_cost_microusd: env_u64("DAL_AGENT_MAX_COST_MICROUSD", 0),
    }
}

#[derive(Debug, Default)]
struct ToolLoopGuardState {
    started_at: Option<std::time::Instant>,
    tool_type_counts: HashMap<String, u32>,
    last_tool_signature: Option<String>,
    repeated_identical_invocations: u32,
    last_result_fingerprint: Option<u64>,
    consecutive_no_progress: u32,
    total_tokens: u64,
    estimated_cost_microusd: u64,
}

fn fingerprint_str(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

fn apply_turn_usage_budget(
    state: &mut ToolLoopGuardState,
    guards: &ToolLoopGuards,
    turn: &AgentModelTurn,
) -> Option<String> {
    let turn_tokens = if let Some(t) = turn.usage.total_tokens {
        t
    } else {
        let in_toks = turn.usage.input_tokens.unwrap_or(0);
        let out_toks = turn.usage.output_tokens.unwrap_or(0);
        if in_toks > 0 || out_toks > 0 {
            in_toks.saturating_add(out_toks)
        } else {
            // Fallback heuristic when provider does not expose usage.
            (turn.content.len() as u64).saturating_div(4)
        }
    };
    state.total_tokens = state.total_tokens.saturating_add(turn_tokens);
    if let Some(cost) = turn.usage.estimated_cost_microusd {
        state.estimated_cost_microusd = state.estimated_cost_microusd.saturating_add(cost);
    }

    if guards.max_total_tokens > 0 && state.total_tokens > guards.max_total_tokens {
        return Some(format!(
            "Stopped: token budget exceeded ({} > {}).",
            state.total_tokens, guards.max_total_tokens
        ));
    }
    if guards.max_estimated_cost_microusd > 0
        && state.estimated_cost_microusd > guards.max_estimated_cost_microusd
    {
        return Some(format!(
            "Stopped: cost budget exceeded ({} > {} microusd).",
            state.estimated_cost_microusd, guards.max_estimated_cost_microusd
        ));
    }
    None
}

fn tool_signature(outcome: &ToolOutcome) -> Option<(String, String)> {
    match outcome {
        ToolOutcome::Run(cmd) => Some(("run".to_string(), format!("run:{}", cmd.trim()))),
        ToolOutcome::Search(query) => {
            Some(("search".to_string(), format!("search:{}", query.trim())))
        }
        ToolOutcome::DalInit(template) => Some((
            "dal_init".to_string(),
            format!(
                "dal_init:{}",
                template.clone().unwrap_or_else(|| "general".to_string())
            ),
        )),
        ToolOutcome::ReadFile(path) => Some((
            "read_file".to_string(),
            format!("read_file:{}", path.trim()),
        )),
        ToolOutcome::WriteFile(path, contents) => Some((
            "write_file".to_string(),
            format!("write_file:{}:{}", path.trim(), fingerprint_str(contents)),
        )),
        ToolOutcome::ListDir(path) => {
            Some(("list_dir".to_string(), format!("list_dir:{}", path.trim())))
        }
        ToolOutcome::DalCheck(path) => Some((
            "dal_check".to_string(),
            format!("dal_check:{}", path.trim()),
        )),
        ToolOutcome::DalRun(path) => {
            Some(("dal_run".to_string(), format!("dal_run:{}", path.trim())))
        }
        ToolOutcome::ShowUrl(url) => {
            Some(("show_url".to_string(), format!("show_url:{}", url.trim())))
        }
        ToolOutcome::ShowContent(content, title) => Some((
            "show_content".to_string(),
            format!(
                "show_content:{}:{}",
                title.clone().unwrap_or_default(),
                fingerprint_str(content)
            ),
        )),
        _ => None,
    }
}

fn register_tool_invocation_guard(
    state: &mut ToolLoopGuardState,
    guards: &ToolLoopGuards,
    tool_name: &str,
    signature: &str,
) -> Option<String> {
    let count = state
        .tool_type_counts
        .entry(tool_name.to_string())
        .or_insert(0);
    *count = count.saturating_add(1);
    if guards.max_tool_calls_per_type > 0 && *count > guards.max_tool_calls_per_type {
        return Some(format!(
            "Stopped: tool '{}' exceeded per-type limit ({} > {}).",
            tool_name, *count, guards.max_tool_calls_per_type
        ));
    }
    if state.last_tool_signature.as_deref() == Some(signature) {
        state.repeated_identical_invocations =
            state.repeated_identical_invocations.saturating_add(1);
    } else {
        state.repeated_identical_invocations = 1;
    }
    if guards.max_repeated_identical_invocations > 0
        && state.repeated_identical_invocations > guards.max_repeated_identical_invocations
    {
        return Some(format!(
            "Stopped: repeated identical tool invocation '{}' exceeded limit ({} > {}).",
            tool_name,
            state.repeated_identical_invocations,
            guards.max_repeated_identical_invocations
        ));
    }
    None
}

fn register_tool_result_guard(
    state: &mut ToolLoopGuardState,
    guards: &ToolLoopGuards,
    signature: &str,
    result: &str,
) -> Option<String> {
    let current_fingerprint = fingerprint_str(result);
    if state.last_tool_signature.as_deref() == Some(signature)
        && state.last_result_fingerprint == Some(current_fingerprint)
    {
        state.consecutive_no_progress = state.consecutive_no_progress.saturating_add(1);
    } else {
        state.consecutive_no_progress = 0;
    }
    state.last_tool_signature = Some(signature.to_string());
    state.last_result_fingerprint = Some(current_fingerprint);
    if guards.max_consecutive_no_progress > 0
        && state.consecutive_no_progress >= guards.max_consecutive_no_progress
    {
        return Some(format!(
            "Stopped: consecutive no-progress loop detected (limit {}).",
            guards.max_consecutive_no_progress
        ));
    }
    None
}

/// Run the tool loop until the LLM returns reply or ask_user, or max_steps is reached.
/// Appends each run/search to evolve action log when agent_name is Some.
/// working_root: if Some, file tools (read_file, write_file, list_dir, dal_check, dal_run, dal_init) use this path; else process current_dir (Phase D).
/// Caller should append_conversation(user_msg, result.final_text) when done.
pub fn run_multi_step_tool_loop(
    schema: &mut crate::agent_context_schema::AgentContextSchema,
    max_steps: u32,
    agent_name: Option<&str>,
    working_root: Option<&std::path::Path>,
) -> Result<MultiStepResult, String> {
    use crate::agent_context_schema::ConversationTurn;
    let root = working_root.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    });
    let guards = tool_loop_guards_from_env();
    let mut guard_state = ToolLoopGuardState {
        started_at: Some(std::time::Instant::now()),
        ..ToolLoopGuardState::default()
    };
    let mut steps_used: u32 = 0;
    let include_scripting_tools = working_root.is_some();
    loop {
        if let Some(started) = guard_state.started_at {
            if guards.max_wall_clock_ms > 0
                && started.elapsed().as_millis() as u64 > guards.max_wall_clock_ms
            {
                return Ok(MultiStepResult {
                    final_text: format!(
                        "Stopped: wall-clock limit exceeded (>{} ms).",
                        guards.max_wall_clock_ms
                    ),
                    is_ask_user: false,
                    steps_used,
                    max_steps_reached: false,
                });
            }
        }
        let turn = generate_agent_model_turn(schema, include_scripting_tools)?;
        if let Some(msg) = apply_turn_usage_budget(&mut guard_state, &guards, &turn) {
            return Ok(MultiStepResult {
                final_text: msg,
                is_ask_user: false,
                steps_used,
                max_steps_reached: false,
            });
        }
        let parsed = model_turn_to_outcome(&turn);
        let outcome = parsed.outcome;
        let assistant_event = parsed.assistant_event;
        let pending_signature = tool_signature(&outcome);
        if let Some((tool_name, signature)) = pending_signature.as_ref() {
            if let Some(msg) =
                register_tool_invocation_guard(&mut guard_state, &guards, tool_name, signature)
            {
                return Ok(MultiStepResult {
                    final_text: msg,
                    is_ask_user: false,
                    steps_used,
                    max_steps_reached: false,
                });
            }
        }
        match outcome {
            ToolOutcome::Reply(text) => {
                return Ok(MultiStepResult {
                    final_text: text,
                    is_ask_user: false,
                    steps_used,
                    max_steps_reached: false,
                });
            }
            ToolOutcome::AskUser(message) => {
                return Ok(MultiStepResult {
                    final_text: message,
                    is_ask_user: true,
                    steps_used,
                    max_steps_reached: false,
                });
            }
            ToolOutcome::ParseFail(raw) => {
                return Ok(MultiStepResult {
                    final_text: raw,
                    is_ask_user: false,
                    steps_used,
                    max_steps_reached: false,
                });
            }
            ToolOutcome::Run(cmd) => {
                let result = execute_run_result(&cmd);
                if let Some((_, signature)) = pending_signature.as_ref() {
                    if let Some(msg) =
                        register_tool_result_guard(&mut guard_state, &guards, signature, &result)
                    {
                        return Ok(MultiStepResult {
                            final_text: msg,
                            is_ask_user: false,
                            steps_used,
                            max_steps_reached: false,
                        });
                    }
                }
                if agent_name.is_some() {
                    let _ = crate::stdlib::evolve::append_log("run", &cmd, &result);
                }
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    return Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::Search(query) => {
                let result = execute_search_result(&query);
                if let Some((_, signature)) = pending_signature.as_ref() {
                    if let Some(msg) =
                        register_tool_result_guard(&mut guard_state, &guards, signature, &result)
                    {
                        return Ok(MultiStepResult {
                            final_text: msg,
                            is_ask_user: false,
                            steps_used,
                            max_steps_reached: false,
                        });
                    }
                }
                if agent_name.is_some() {
                    let _ = crate::stdlib::evolve::append_log("search", &query, &result);
                }
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    return Ok(MultiStepResult {
                        final_text: "Max tool steps reached".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::DalInit(template) => {
                let t = template.as_deref();
                let result = execute_dal_init_result(t, &root);
                if let Some((_, signature)) = pending_signature.as_ref() {
                    if let Some(msg) =
                        register_tool_result_guard(&mut guard_state, &guards, signature, &result)
                    {
                        return Ok(MultiStepResult {
                            final_text: msg,
                            is_ask_user: false,
                            steps_used,
                            max_steps_reached: false,
                        });
                    }
                }
                if agent_name.is_some() {
                    let _ = crate::stdlib::evolve::append_log(
                        "dal_init",
                        &template.unwrap_or_else(|| "general".to_string()),
                        &result,
                    );
                }
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    return Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::ReadFile(path) => {
                let result = execute_read_file_result(&path, &root);
                if let Some((_, signature)) = pending_signature.as_ref() {
                    if let Some(msg) =
                        register_tool_result_guard(&mut guard_state, &guards, signature, &result)
                    {
                        return Ok(MultiStepResult {
                            final_text: msg,
                            is_ask_user: false,
                            steps_used,
                            max_steps_reached: false,
                        });
                    }
                }
                if agent_name.is_some() {
                    let _ = crate::stdlib::evolve::append_log("read_file", &path, &result);
                }
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    return Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::WriteFile(path, contents) => {
                let result = execute_write_file_result(&path, &contents, &root);
                if let Some((_, signature)) = pending_signature.as_ref() {
                    if let Some(msg) =
                        register_tool_result_guard(&mut guard_state, &guards, signature, &result)
                    {
                        return Ok(MultiStepResult {
                            final_text: msg,
                            is_ask_user: false,
                            steps_used,
                            max_steps_reached: false,
                        });
                    }
                }
                if agent_name.is_some() {
                    let _ = crate::stdlib::evolve::append_log("write_file", &path, &result);
                }
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    return Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::ListDir(path) => {
                let result = execute_list_dir_result(&path, &root);
                if let Some((_, signature)) = pending_signature.as_ref() {
                    if let Some(msg) =
                        register_tool_result_guard(&mut guard_state, &guards, signature, &result)
                    {
                        return Ok(MultiStepResult {
                            final_text: msg,
                            is_ask_user: false,
                            steps_used,
                            max_steps_reached: false,
                        });
                    }
                }
                if agent_name.is_some() {
                    let _ = crate::stdlib::evolve::append_log("list_dir", &path, &result);
                }
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    return Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::DalCheck(path) => {
                let result = execute_dal_check_result(&path, &root);
                if let Some((_, signature)) = pending_signature.as_ref() {
                    if let Some(msg) =
                        register_tool_result_guard(&mut guard_state, &guards, signature, &result)
                    {
                        return Ok(MultiStepResult {
                            final_text: msg,
                            is_ask_user: false,
                            steps_used,
                            max_steps_reached: false,
                        });
                    }
                }
                if agent_name.is_some() {
                    let _ = crate::stdlib::evolve::append_log("dal_check", &path, &result);
                }
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    return Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::DalRun(path) => {
                let result = execute_dal_run_result(&path, &root);
                if let Some((_, signature)) = pending_signature.as_ref() {
                    if let Some(msg) =
                        register_tool_result_guard(&mut guard_state, &guards, signature, &result)
                    {
                        return Ok(MultiStepResult {
                            final_text: msg,
                            is_ask_user: false,
                            steps_used,
                            max_steps_reached: false,
                        });
                    }
                }
                if agent_name.is_some() {
                    let _ = crate::stdlib::evolve::append_log("dal_run", &path, &result);
                }
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    return Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::ShowUrl(_url) => {
                let result = "URL display requested (visible in IDE workspace).".to_string();
                if let Some((_, signature)) = pending_signature.as_ref() {
                    if let Some(msg) =
                        register_tool_result_guard(&mut guard_state, &guards, signature, &result)
                    {
                        return Ok(MultiStepResult {
                            final_text: msg,
                            is_ask_user: false,
                            steps_used,
                            max_steps_reached: false,
                        });
                    }
                }
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    return Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
            ToolOutcome::ShowContent(_, _) => {
                let result = "Content display requested (visible in IDE workspace).".to_string();
                if let Some((_, signature)) = pending_signature.as_ref() {
                    if let Some(msg) =
                        register_tool_result_guard(&mut guard_state, &guards, signature, &result)
                    {
                        return Ok(MultiStepResult {
                            final_text: msg,
                            is_ask_user: false,
                            steps_used,
                            max_steps_reached: false,
                        });
                    }
                }
                schema.conversation.push(ConversationTurn {
                    role: "assistant".to_string(),
                    content: assistant_event.clone(),
                });
                schema.conversation.push(ConversationTurn {
                    role: "user".to_string(),
                    content: format!("[Tool result]\n{}", result),
                });
                steps_used += 1;
                if steps_used >= max_steps {
                    return Ok(MultiStepResult {
                        final_text: "Max tool steps reached.".to_string(),
                        is_ask_user: false,
                        steps_used,
                        max_steps_reached: true,
                    });
                }
            }
        }
    }
}

/// Same as `respond_with_tools` but returns a map-friendly result: final text, steps used, and
/// whether the step limit was reached. Lets DAL apps branch on outcome without parsing the reply.
fn emit_route_metrics(
    route: &str,
    schema: Option<&crate::agent_context_schema::AgentContextSchema>,
    result: &MultiStepResult,
    max_steps: u32,
) {
    let mut native_tool_calls_seen: i64 = 0;
    let mut tool_results_seen: i64 = 0;
    if let Some(schema) = schema {
        for turn in &schema.conversation {
            if turn.role.trim().eq_ignore_ascii_case("assistant")
                && parse_tool_call_from_conversation(&turn.content).is_some()
            {
                native_tool_calls_seen += 1;
            }
            if turn.role.trim().eq_ignore_ascii_case("user")
                && parse_tool_result_from_conversation(&turn.content).is_some()
            {
                tool_results_seen += 1;
            }
        }
    }
    let termination = classify_termination(result);
    crate::stdlib::log::info(
        "agent_route_metrics",
        {
            let mut data = HashMap::new();
            data.insert("route".to_string(), Value::String(route.to_string()));
            data.insert(
                "steps_used".to_string(),
                Value::Int(result.steps_used as i64),
            );
            data.insert("max_steps".to_string(), Value::Int(max_steps as i64));
            data.insert(
                "max_steps_reached".to_string(),
                Value::Bool(result.max_steps_reached),
            );
            data.insert("is_ask_user".to_string(), Value::Bool(result.is_ask_user));
            data.insert(
                "native_tool_calls_seen".to_string(),
                Value::Int(native_tool_calls_seen),
            );
            data.insert(
                "tool_results_seen".to_string(),
                Value::Int(tool_results_seen),
            );
            data.insert(
                "legacy_parse_enabled".to_string(),
                Value::Bool(legacy_text_tool_protocol_enabled()),
            );
            data.insert(
                "native_tool_calling_enabled".to_string(),
                Value::Bool(native_tool_calling_enabled()),
            );
            let default_policy = match default_chat_policy_from_env() {
                ChatPolicy::Auto => "auto",
                ChatPolicy::ReplyOnly => "reply_only",
                ChatPolicy::ToolLoop => "tool_loop",
            };
            data.insert(
                "default_policy".to_string(),
                Value::String(default_policy.to_string()),
            );
            data.insert(
                "parse_fail_terminal".to_string(),
                Value::Bool(termination.parse_fail_terminal),
            );
            data.insert(
                "unsupported_tool_call_terminal".to_string(),
                Value::Bool(termination.unsupported_tool_call_terminal),
            );
            data.insert(
                "guard_stopped".to_string(),
                Value::Bool(termination.guard_stopped),
            );
            data.insert(
                "termination_reason".to_string(),
                Value::String(termination.termination_reason.to_string()),
            );
            data
        },
        Some("ai"),
    );
}

#[derive(Debug, Clone)]
pub struct RespondWithToolsDiagnostics {
    pub policy: ChatPolicy,
    pub route: ChatRoute,
    pub result: MultiStepResult,
    /// Tool names only, in execution order.
    pub tool_trace: Vec<String>,
}

fn collect_tool_trace_from_schema(
    schema: &crate::agent_context_schema::AgentContextSchema,
) -> Vec<String> {
    let mut out = Vec::new();
    for turn in &schema.conversation {
        if !turn.role.trim().eq_ignore_ascii_case("assistant") {
            continue;
        }
        if let Some(call) = parse_tool_call_from_conversation(&turn.content) {
            out.push(call.name);
        }
    }
    out
}

pub fn respond_with_tools_diagnostics(
    user_message: &str,
) -> Result<RespondWithToolsDiagnostics, String> {
    respond_with_tools_diagnostics_with_policy(user_message, default_chat_policy_from_env())
}

pub fn respond_with_tools_diagnostics_with_policy(
    user_message: &str,
    policy: ChatPolicy,
) -> Result<RespondWithToolsDiagnostics, String> {
    let route = route_for_policy(policy, user_message);
    match route {
        ChatRoute::ReplyOnly => {
            let schema = build_reply_only_schema(user_message);
            let prompt = crate::agent_context_schema::build_prompt_for_llm(&schema);
            let reply = generate_text(prompt).map_err(|e| e.to_string())?;
            let result = MultiStepResult {
                final_text: reply.trim().to_string(),
                is_ask_user: false,
                steps_used: 0,
                max_steps_reached: false,
            };
            emit_route_metrics("reply_only", None, &result, 0);
            Ok(RespondWithToolsDiagnostics {
                policy,
                route,
                result,
                tool_trace: Vec::new(),
            })
        }
        ChatRoute::ToolLoop => {
            let max_steps = max_tool_steps_from_env();
            let (mut schema, working_root) = build_tool_loop_schema(user_message);
            let result =
                run_multi_step_tool_loop(&mut schema, max_steps, None, working_root.as_deref())?;
            let tool_trace = collect_tool_trace_from_schema(&schema);
            emit_route_metrics("tool_loop", Some(&schema), &result, max_steps);
            Ok(RespondWithToolsDiagnostics {
                policy,
                route,
                result,
                tool_trace,
            })
        }
    }
}

pub fn respond_with_tools_result(user_message: &str) -> Result<MultiStepResult, String> {
    respond_with_tools_diagnostics(user_message).map(|d| d.result)
}

pub fn respond_with_tools_result_with_policy(
    user_message: &str,
    policy: ChatPolicy,
) -> Result<MultiStepResult, String> {
    respond_with_tools_diagnostics_with_policy(user_message, policy).map(|d| d.result)
}

#[cfg(test)]
mod multi_step_loop_tests {
    use super::{
        build_transcript_events, decide_chat_route, model_turn_to_outcome,
        parse_tool_call_from_conversation, parse_tool_response, AgentModelTurn, ChatPolicy,
        ChatRoute, NativeToolCall, ToolOutcome, TranscriptEvent, TurnUsage,
    };

    #[test]
    fn chat_route_reply_only_for_conceptual_prompt() {
        let route = decide_chat_route("What is a DAL module?");
        assert_eq!(route, ChatRoute::ReplyOnly);
    }

    #[test]
    fn chat_route_tool_loop_for_action_prompt() {
        let route = decide_chat_route("Check this project and run tests.");
        assert_eq!(route, ChatRoute::ToolLoop);
    }

    #[test]
    fn chat_policy_parse_and_route_override() {
        assert_eq!(ChatPolicy::from_str("auto"), Some(ChatPolicy::Auto));
        assert_eq!(
            ChatPolicy::from_str("reply_only"),
            Some(ChatPolicy::ReplyOnly)
        );
        assert_eq!(
            ChatPolicy::from_str("tool_loop"),
            Some(ChatPolicy::ToolLoop)
        );
        assert_eq!(ChatPolicy::from_str("unknown"), None);
    }

    #[test]
    fn parse_tool_response_reply() {
        let out = parse_tool_response(r#"{"action":"reply","text":"Hello"}"#);
        match out {
            ToolOutcome::Reply(s) => assert_eq!(s, "Hello"),
            _ => panic!("expected Reply"),
        }
    }

    #[test]
    fn parse_tool_response_ask_user() {
        let out = parse_tool_response(r#"{"action":"ask_user","message":"Need confirmation"}"#);
        match out {
            ToolOutcome::AskUser(s) => assert_eq!(s, "Need confirmation"),
            _ => panic!("expected AskUser"),
        }
    }

    #[test]
    fn parse_tool_response_run() {
        let out = parse_tool_response(r#"{"action":"run","cmd":"ls -la"}"#);
        match out {
            ToolOutcome::Run(s) => assert_eq!(s, "ls -la"),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn parse_tool_response_search() {
        let out = parse_tool_response(r#"{"action":"search","query":"rust lang"}"#);
        match out {
            ToolOutcome::Search(s) => assert_eq!(s, "rust lang"),
            _ => panic!("expected Search"),
        }
    }

    #[test]
    fn parse_tool_response_dal_init() {
        let out = parse_tool_response(r#"{"action":"dal_init"}"#);
        match &out {
            ToolOutcome::DalInit(None) => {}
            _ => panic!("expected DalInit(None), got {:?}", out),
        }
        let out2 = parse_tool_response(r#"{"action":"dal_init","template":"chain"}"#);
        match &out2 {
            ToolOutcome::DalInit(Some(t)) => assert_eq!(t, "chain"),
            _ => panic!("expected DalInit(Some(\"chain\")), got {:?}", out2),
        }
    }

    #[test]
    fn parse_tool_response_read_file_and_dal_check() {
        let out = parse_tool_response(r#"{"action":"read_file","path":"main.dal"}"#);
        match &out {
            ToolOutcome::ReadFile(p) => assert_eq!(p, "main.dal"),
            _ => panic!("expected ReadFile, got {:?}", out),
        }
        let out2 = parse_tool_response(r#"{"action":"dal_check","path":"main.dal"}"#);
        match &out2 {
            ToolOutcome::DalCheck(p) => assert_eq!(p, "main.dal"),
            _ => panic!("expected DalCheck, got {:?}", out2),
        }
    }

    #[test]
    fn parse_tool_response_no_json_is_parse_fail() {
        let out = parse_tool_response("Just plain text");
        match out {
            ToolOutcome::ParseFail(s) => assert_eq!(s, "Just plain text"),
            _ => panic!("expected ParseFail"),
        }
    }

    #[test]
    fn model_turn_prefers_native_tool_calls() {
        let turn = AgentModelTurn {
            content: "This should not be used when a tool call exists.".to_string(),
            tool_calls: vec![NativeToolCall {
                id: Some("call_123".to_string()),
                name: "run".to_string(),
                arguments: serde_json::json!({"cmd":"echo hello"}),
            }],
            usage: TurnUsage::default(),
        };
        let parsed = model_turn_to_outcome(&turn);
        match parsed.outcome {
            ToolOutcome::Run(cmd) => assert_eq!(cmd, "echo hello"),
            _ => panic!("expected Run"),
        }
        assert!(parsed.assistant_event.contains("\"name\":\"run\""));
    }

    #[test]
    fn transcript_events_capture_tool_roundtrip() {
        let mut schema =
            crate::agent_context_schema::AgentContextSchema::minimal("Do work", "run/search");
        schema
            .conversation
            .push(crate::agent_context_schema::ConversationTurn {
            role: "assistant".to_string(),
            content:
                "{\"tool_call_id\":\"call_1\",\"name\":\"run\",\"arguments\":{\"cmd\":\"echo hi\"}}"
                    .to_string(),
        });
        schema
            .conversation
            .push(crate::agent_context_schema::ConversationTurn {
                role: "user".to_string(),
                content: "[Tool result]\nExit code: 0\nstdout:\nhi".to_string(),
            });
        let events = build_transcript_events(&schema);
        assert!(matches!(events.first(), Some(TranscriptEvent::UserText(_))));
        assert!(matches!(
            events.get(1),
            Some(TranscriptEvent::AssistantToolCall(_))
        ));
        assert!(matches!(
            events.get(2),
            Some(TranscriptEvent::ToolResult { .. })
        ));
    }

    #[test]
    fn unsupported_native_tool_call_is_parse_fail() {
        let turn = AgentModelTurn {
            content: "ignored".to_string(),
            tool_calls: vec![NativeToolCall {
                id: Some("call_bad".to_string()),
                name: "delete_everything".to_string(),
                arguments: serde_json::json!({}),
            }],
            usage: TurnUsage::default(),
        };
        let parsed = model_turn_to_outcome(&turn);
        match parsed.outcome {
            ToolOutcome::ParseFail(msg) => {
                assert_eq!(msg, "Unsupported tool call: delete_everything")
            }
            _ => panic!("expected ParseFail for unsupported tool"),
        }
        assert!(parsed
            .assistant_event
            .contains("\"tool_call_id\":\"call_bad\""));
    }

    #[test]
    fn malformed_tool_call_marker_is_not_interpreted_as_tool_call() {
        let bad = r#"{"tool_call_id":"c1","name":"run","arguments":"not-an-object"}"#;
        let parsed = parse_tool_call_from_conversation(bad);
        assert!(parsed.is_none());
    }

    #[test]
    fn orphan_tool_result_stays_user_text() {
        let mut schema =
            crate::agent_context_schema::AgentContextSchema::minimal("Do work", "run/search");
        schema
            .conversation
            .push(crate::agent_context_schema::ConversationTurn {
                role: "user".to_string(),
                content: "[Tool result]\nExit code: 1".to_string(),
            });
        let events = build_transcript_events(&schema);
        assert!(matches!(
            events.get(1),
            Some(TranscriptEvent::UserText(s)) if s.contains("[Tool result]")
        ));
    }
}

#[cfg(feature = "http-interface")]
fn host_tool_definitions(include_scripting_tools: bool) -> serde_json::Value {
    use serde_json::json;

    let mut tools = vec![
        json!({
            "type": "function",
            "function": {
                "name": "run",
                "description": "Run a shell command.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "cmd": { "type": "string" }
                    },
                    "required": ["cmd"],
                    "additionalProperties": false
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "search",
                "description": "Search the web for public information.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"],
                    "additionalProperties": false
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "ask_user",
                "description": "Request additional user input.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" }
                    },
                    "required": ["message"],
                    "additionalProperties": false
                }
            }
        }),
    ];
    if include_scripting_tools {
        tools.extend([
            json!({
                "type": "function",
                "function": {
                    "name": "write_file",
                    "description": "Write text content to a relative file path.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" },
                            "contents": { "type": "string" }
                        },
                        "required": ["path", "contents"],
                        "additionalProperties": false
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "read_file",
                    "description": "Read a relative file path.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" }
                        },
                        "required": ["path"],
                        "additionalProperties": false
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "list_dir",
                    "description": "List directory entries at a relative path.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" }
                        },
                        "required": ["path"],
                        "additionalProperties": false
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "dal_check",
                    "description": "Run dal check on a DAL file path.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" }
                        },
                        "required": ["path"],
                        "additionalProperties": false
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "dal_run",
                    "description": "Run dal run on a DAL file path.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" }
                        },
                        "required": ["path"],
                        "additionalProperties": false
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "dal_init",
                    "description": "Initialize DAL project template.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "template": { "type": "string" }
                        },
                        "additionalProperties": false
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "show_url",
                    "description": "Show a URL in IDE workspace.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "url": { "type": "string" }
                        },
                        "required": ["url"],
                        "additionalProperties": false
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "show_content",
                    "description": "Show HTML/text content in IDE workspace.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "content": { "type": "string" },
                            "title": { "type": "string" }
                        },
                        "required": ["content"],
                        "additionalProperties": false
                    }
                }
            }),
        ]);
    }
    serde_json::Value::Array(tools)
}

#[cfg(feature = "http-interface")]
fn estimate_turn_cost_microusd(input_tokens: u64, output_tokens: u64) -> Option<u64> {
    let in_rate = std::env::var("DAL_AGENT_INPUT_COST_MICROUSD_PER_1K_TOKENS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let out_rate = std::env::var("DAL_AGENT_OUTPUT_COST_MICROUSD_PER_1K_TOKENS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    if in_rate == 0 && out_rate == 0 {
        return None;
    }
    let in_cost = input_tokens.saturating_mul(in_rate).saturating_div(1000);
    let out_cost = output_tokens.saturating_mul(out_rate).saturating_div(1000);
    Some(in_cost.saturating_add(out_cost))
}

#[cfg(feature = "http-interface")]
fn call_openai_api_tool_turn(
    schema: &crate::agent_context_schema::AgentContextSchema,
    api_key: &str,
    config: &AIConfig,
    include_scripting_tools: bool,
) -> Result<AgentModelTurn, String> {
    use serde_json::json;

    let timeout = std::time::Duration::from_secs(config.timeout_seconds);
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())?;
    let model = config
        .model
        .clone()
        .or_else(|| env::var("OPENAI_MODEL").ok())
        .or_else(|| env::var("DAL_OPENAI_MODEL").ok())
        .unwrap_or_else(|| "gpt-4o-mini".to_string());

    let mut messages = vec![json!({
        "role": "system",
        "content": build_provider_system_prompt(schema)
    })];
    for event in build_transcript_events(schema) {
        match event {
            TranscriptEvent::UserText(text) => messages.push(json!({
                "role": "user",
                "content": text
            })),
            TranscriptEvent::AssistantText(text) => messages.push(json!({
                "role": "assistant",
                "content": text
            })),
            TranscriptEvent::AssistantToolCall(call) => {
                let id = call.id.unwrap_or_else(|| "tool_call".to_string());
                messages.push(json!({
                    "role": "assistant",
                    "content": "",
                    "tool_calls": [
                        {
                            "id": id,
                            "type": "function",
                            "function": {
                                "name": call.name,
                                "arguments": call.arguments.to_string()
                            }
                        }
                    ]
                }));
            }
            TranscriptEvent::ToolResult {
                tool_call_id,
                content,
            } => messages.push(json!({
                "role": "tool",
                "tool_call_id": tool_call_id,
                "content": content
            })),
        }
    }

    let body = json!({
        "model": model,
        "messages": messages,
        "tools": host_tool_definitions(include_scripting_tools),
        "temperature": config.temperature,
        "max_tokens": config.max_tokens
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;
    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error {}: {}", status, error_text));
    }
    let json: serde_json::Value = response.json().map_err(|e| format!("Parse error: {}", e))?;

    let message = &json["choices"][0]["message"];
    let content = message["content"].as_str().unwrap_or("").trim().to_string();
    let tool_calls = message["tool_calls"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|tc| {
                    let id = tc["id"].as_str().map(|s| s.to_string());
                    let name = tc["function"]["name"].as_str()?.to_string();
                    let raw_args = tc["function"]["arguments"].as_str().unwrap_or("{}");
                    let arguments = serde_json::from_str(raw_args).unwrap_or_else(|_| json!({}));
                    Some(NativeToolCall {
                        id,
                        name,
                        arguments,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let input_tokens = json["usage"]["prompt_tokens"].as_u64();
    let output_tokens = json["usage"]["completion_tokens"].as_u64();
    let total_tokens = json["usage"]["total_tokens"].as_u64();
    let estimated_cost_microusd = match (input_tokens, output_tokens) {
        (Some(i), Some(o)) => estimate_turn_cost_microusd(i, o),
        _ => None,
    };

    Ok(AgentModelTurn {
        content,
        tool_calls,
        usage: TurnUsage {
            input_tokens,
            output_tokens,
            total_tokens,
            estimated_cost_microusd,
        },
    })
}

#[cfg(feature = "http-interface")]
fn call_anthropic_api_tool_turn(
    schema: &crate::agent_context_schema::AgentContextSchema,
    api_key: &str,
    config: &AIConfig,
    include_scripting_tools: bool,
) -> Result<AgentModelTurn, String> {
    use serde_json::json;
    let timeout = std::time::Duration::from_secs(config.timeout_seconds);
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())?;
    let model = config
        .model
        .clone()
        .or_else(|| env::var("ANTHROPIC_MODEL").ok())
        .or_else(|| env::var("DAL_ANTHROPIC_MODEL").ok())
        .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());

    let openai_tools = host_tool_definitions(include_scripting_tools);
    let tools = openai_tools
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|t| {
            let name = t["function"]["name"].clone();
            let description = t["function"]["description"].clone();
            let input_schema = t["function"]["parameters"].clone();
            json!({
                "name": name,
                "description": description,
                "input_schema": input_schema
            })
        })
        .collect::<Vec<_>>();

    let mut messages = Vec::new();
    for event in build_transcript_events(schema) {
        match event {
            TranscriptEvent::UserText(text) => messages.push(json!({
                "role": "user",
                "content": text
            })),
            TranscriptEvent::AssistantText(text) => messages.push(json!({
                "role": "assistant",
                "content": text
            })),
            TranscriptEvent::AssistantToolCall(call) => {
                let id = call.id.unwrap_or_else(|| "tool_call".to_string());
                messages.push(json!({
                    "role": "assistant",
                    "content": [
                        {
                            "type": "tool_use",
                            "id": id,
                            "name": call.name,
                            "input": call.arguments
                        }
                    ]
                }));
            }
            TranscriptEvent::ToolResult {
                tool_call_id,
                content,
            } => messages.push(json!({
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": tool_call_id,
                        "content": content
                    }
                ]
            })),
        }
    }

    let body = json!({
        "model": model,
        "max_tokens": config.max_tokens,
        "system": build_provider_system_prompt(schema),
        "messages": messages,
        "tools": tools
    });
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;
    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error {}: {}", status, error_text));
    }
    let json: serde_json::Value = response.json().map_err(|e| format!("Parse error: {}", e))?;

    let mut content_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<NativeToolCall> = Vec::new();
    if let Some(blocks) = json["content"].as_array() {
        for block in blocks {
            match block["type"].as_str().unwrap_or("") {
                "text" => {
                    if let Some(text) = block["text"].as_str() {
                        if !text.trim().is_empty() {
                            content_parts.push(text.trim().to_string());
                        }
                    }
                }
                "tool_use" => {
                    if let Some(name) = block["name"].as_str() {
                        tool_calls.push(NativeToolCall {
                            id: block["id"].as_str().map(|s| s.to_string()),
                            name: name.to_string(),
                            arguments: block["input"].clone(),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    let input_tokens = json["usage"]["input_tokens"].as_u64();
    let output_tokens = json["usage"]["output_tokens"].as_u64();
    let total_tokens = match (input_tokens, output_tokens) {
        (Some(i), Some(o)) => Some(i.saturating_add(o)),
        _ => None,
    };
    let estimated_cost_microusd = match (input_tokens, output_tokens) {
        (Some(i), Some(o)) => estimate_turn_cost_microusd(i, o),
        _ => None,
    };

    Ok(AgentModelTurn {
        content: content_parts.join("\n"),
        tool_calls,
        usage: TurnUsage {
            input_tokens,
            output_tokens,
            total_tokens,
            estimated_cost_microusd,
        },
    })
}

#[cfg(feature = "http-interface")]
fn call_openai_api(prompt: &str, api_key: &str, config: &AIConfig) -> Result<String, String> {
    use serde_json::json;

    let timeout = std::time::Duration::from_secs(config.timeout_seconds);
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())?;

    let model = config
        .model
        .clone()
        .or_else(|| env::var("OPENAI_MODEL").ok())
        .or_else(|| env::var("DAL_OPENAI_MODEL").ok())
        .unwrap_or_else(|| "gpt-4".to_string());

    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are an expert dist_agent_lang (DAL) programmer. Provide clear, accurate, and concise responses."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": config.temperature,
        "max_tokens": config.max_tokens
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error {}: {}", status, error_text));
    }

    let json: serde_json::Value = response.json().map_err(|e| format!("Parse error: {}", e))?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "Invalid response format".to_string())
}

#[cfg(feature = "http-interface")]
fn call_anthropic_api(prompt: &str, api_key: &str, config: &AIConfig) -> Result<String, String> {
    use serde_json::json;

    let timeout = std::time::Duration::from_secs(config.timeout_seconds);
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())?;

    let model = config
        .model
        .clone()
        .or_else(|| env::var("ANTHROPIC_MODEL").ok())
        .or_else(|| env::var("DAL_ANTHROPIC_MODEL").ok())
        .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());

    let body = json!({
        "model": model,
        "max_tokens": config.max_tokens,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error {}: {}", status, error_text));
    }

    let json: serde_json::Value = response.json().map_err(|e| format!("Parse error: {}", e))?;

    json["content"][0]["text"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "Invalid response format".to_string())
}

#[cfg(feature = "http-interface")]
fn call_local_model(prompt: &str, endpoint: &str, config: &AIConfig) -> Result<String, String> {
    use serde_json::json;

    let timeout = std::time::Duration::from_secs(config.timeout_seconds.max(60));
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())?;

    let model = config
        .model
        .clone()
        .or_else(|| env::var("DAL_AI_MODEL").ok())
        .unwrap_or_else(|| "codellama".to_string());

    let body = json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": config.temperature,
            "num_predict": config.max_tokens
        }
    });

    let response = client
        .post(endpoint)
        .json(&body)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error {}: {}", status, error_text));
    }

    let json: serde_json::Value = response.json().map_err(|e| format!("Parse error: {}", e))?;

    json["response"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "Invalid response format".to_string())
}

#[cfg(not(feature = "http-interface"))]
fn call_openai_api_tool_turn(
    _schema: &crate::agent_context_schema::AgentContextSchema,
    _api_key: &str,
    _config: &AIConfig,
    _include_scripting_tools: bool,
) -> Result<AgentModelTurn, String> {
    Err("HTTP interface not enabled".to_string())
}

#[cfg(not(feature = "http-interface"))]
fn call_anthropic_api_tool_turn(
    _schema: &crate::agent_context_schema::AgentContextSchema,
    _api_key: &str,
    _config: &AIConfig,
    _include_scripting_tools: bool,
) -> Result<AgentModelTurn, String> {
    Err("HTTP interface not enabled".to_string())
}

#[cfg(not(feature = "http-interface"))]
fn call_openai_api(_prompt: &str, _api_key: &str, _config: &AIConfig) -> Result<String, String> {
    Err("HTTP interface not enabled".to_string())
}

#[cfg(not(feature = "http-interface"))]
fn call_anthropic_api(_prompt: &str, _api_key: &str, _config: &AIConfig) -> Result<String, String> {
    Err("HTTP interface not enabled".to_string())
}

#[cfg(not(feature = "http-interface"))]
fn call_local_model(_prompt: &str, _endpoint: &str, _config: &AIConfig) -> Result<String, String> {
    Err("HTTP interface not enabled".to_string())
}

/// Call custom AI provider (Cohere, HuggingFace, Azure OpenAI, etc.)
/// Uses flexible JSON structure to support different APIs
#[cfg(feature = "http-interface")]
fn call_custom_provider(
    prompt: &str,
    endpoint: &str,
    api_key: &str,
    provider_name: &str,
    config: &AIConfig,
) -> Result<String, String> {
    use serde_json::json;

    let timeout = std::time::Duration::from_secs(config.timeout_seconds);
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())?;

    let model = config
        .model
        .clone()
        .unwrap_or_else(|| "default".to_string());

    // Build request based on provider type
    let (body, headers) = match provider_name.to_lowercase().as_str() {
        "cohere" => {
            // Cohere API format
            let body = json!({
                "model": model,
                "prompt": prompt,
                "temperature": config.temperature,
                "max_tokens": config.max_tokens
            });
            let headers = vec![
                ("Authorization", format!("Bearer {}", api_key)),
                ("Content-Type", "application/json".to_string()),
            ];
            (body, headers)
        }
        "huggingface" | "hf" => {
            // HuggingFace Inference API format
            let body = json!({
                "inputs": prompt,
                "parameters": {
                    "temperature": config.temperature,
                    "max_new_tokens": config.max_tokens,
                    "return_full_text": false
                }
            });
            let headers = vec![
                ("Authorization", format!("Bearer {}", api_key)),
                ("Content-Type", "application/json".to_string()),
            ];
            (body, headers)
        }
        "azure" | "azure-openai" => {
            // Azure OpenAI format (same as OpenAI but different auth)
            let body = json!({
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a helpful assistant."
                    },
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": config.temperature,
                "max_tokens": config.max_tokens
            });
            let headers = vec![
                ("api-key", api_key.to_string()),
                ("Content-Type", "application/json".to_string()),
            ];
            (body, headers)
        }
        "replicate" => {
            // Replicate API format
            let body = json!({
                "version": model,
                "input": {
                    "prompt": prompt,
                    "temperature": config.temperature,
                    "max_length": config.max_tokens
                }
            });
            let headers = vec![
                ("Authorization", format!("Token {}", api_key)),
                ("Content-Type", "application/json".to_string()),
            ];
            (body, headers)
        }
        "together" | "together-ai" => {
            // Together AI format (OpenAI-compatible)
            let body = json!({
                "model": model,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": config.temperature,
                "max_tokens": config.max_tokens
            });
            let headers = vec![
                ("Authorization", format!("Bearer {}", api_key)),
                ("Content-Type", "application/json".to_string()),
            ];
            (body, headers)
        }
        "openrouter" => {
            // OpenRouter format (OpenAI-compatible)
            let body = json!({
                "model": model,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": config.temperature,
                "max_tokens": config.max_tokens
            });
            let headers = vec![
                ("Authorization", format!("Bearer {}", api_key)),
                ("Content-Type", "application/json".to_string()),
                ("HTTP-Referer", "https://dal-lang.dev".to_string()),
            ];
            (body, headers)
        }
        _ => {
            // Generic format - try OpenAI-compatible format as default
            let body = json!({
                "model": model,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": config.temperature,
                "max_tokens": config.max_tokens
            });
            let headers = vec![
                ("Authorization", format!("Bearer {}", api_key)),
                ("Content-Type", "application/json".to_string()),
            ];
            (body, headers)
        }
    };

    // Build request
    let mut request = client.post(endpoint).json(&body);
    for (key, value) in headers {
        request = request.header(key, value);
    }

    // Send request
    let response = request
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error {}: {}", status, error_text));
    }

    // Parse response
    let json: serde_json::Value = response.json().map_err(|e| format!("Parse error: {}", e))?;

    // Try to extract response from different formats
    extract_response_text(&json, provider_name)
}

#[cfg(feature = "http-interface")]
fn extract_response_text(json: &serde_json::Value, provider: &str) -> Result<String, String> {
    match provider.to_lowercase().as_str() {
        "cohere" => {
            // Cohere: { "generations": [{ "text": "..." }] }
            json["generations"][0]["text"]
                .as_str()
                .map(|s| s.trim().to_string())
                .ok_or_else(|| "Invalid Cohere response format".to_string())
        }
        "huggingface" | "hf" => {
            // HuggingFace: [{ "generated_text": "..." }] or { "generated_text": "..." }
            if json.is_array() {
                json[0]["generated_text"]
                    .as_str()
                    .or_else(|| json[0]["generation"].as_str())
                    .map(|s| s.trim().to_string())
                    .ok_or_else(|| "Invalid HuggingFace response format".to_string())
            } else {
                json["generated_text"]
                    .as_str()
                    .or_else(|| json["generation"].as_str())
                    .map(|s| s.trim().to_string())
                    .ok_or_else(|| "Invalid HuggingFace response format".to_string())
            }
        }
        "azure" | "azure-openai" | "together" | "together-ai" | "openrouter" => {
            // OpenAI-compatible format
            json["choices"][0]["message"]["content"]
                .as_str()
                .map(|s| s.trim().to_string())
                .ok_or_else(|| "Invalid response format".to_string())
        }
        "replicate" => {
            // Replicate: { "output": ["text"] } or { "output": "text" }
            if let Some(output) = json["output"].as_array() {
                Ok(output
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(""))
            } else if let Some(output) = json["output"].as_str() {
                Ok(output.trim().to_string())
            } else {
                Err("Invalid Replicate response format".to_string())
            }
        }
        _ => {
            // Try common formats
            // OpenAI-compatible first
            if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                return Ok(content.trim().to_string());
            }

            // Try direct text field
            if let Some(text) = json["text"].as_str() {
                return Ok(text.trim().to_string());
            }

            // Try generation field
            if let Some(text) = json["generation"].as_str() {
                return Ok(text.trim().to_string());
            }

            // Try output field
            if let Some(text) = json["output"].as_str() {
                return Ok(text.trim().to_string());
            }

            // Try response field
            if let Some(text) = json["response"].as_str() {
                return Ok(text.trim().to_string());
            }

            Err(format!(
                "Unable to extract text from response. JSON: {}",
                json
            ))
        }
    }
}

#[cfg(not(feature = "http-interface"))]
fn call_custom_provider(
    _prompt: &str,
    _endpoint: &str,
    _api_key: &str,
    _provider_name: &str,
    _config: &AIConfig,
) -> Result<String, String> {
    Err("HTTP interface not enabled".to_string())
}

pub fn train_model(training_data: TrainingData) -> Result<Model, String> {
    crate::stdlib::log::info(
        "Training AI model",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "data_type".to_string(),
                Value::String(training_data.data_type.clone()),
            );
            data.insert(
                "samples".to_string(),
                Value::Int(training_data.samples.len() as i64),
            );
            data.insert(
                "message".to_string(),
                Value::String("Training AI model".to_string()),
            );
            data
        },
        Some("ai"),
    );

    // Simulated model training
    let model = Model {
        model_id: format!("model_{}", generate_id()),
        model_type: training_data.data_type,
        version: "1.0.0".to_string(),
        accuracy: 0.92,
        training_data_size: training_data.samples.len() as i64,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        last_updated: "2024-01-01T00:00:00Z".to_string(),
    };

    Ok(model)
}

pub fn predict(model: &Model, _input: Value) -> Result<Prediction, String> {
    crate::stdlib::log::info(
        "Making prediction",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "model_id".to_string(),
                Value::String(model.model_id.clone()),
            );
            data.insert(
                "message".to_string(),
                Value::String("Making prediction".to_string()),
            );
            data
        },
        Some("ai"),
    );

    // Simulated prediction
    let prediction = Prediction {
        prediction: Value::String("positive".to_string()),
        confidence: 0.87,
        probabilities: {
            let mut probs = HashMap::new();
            probs.insert("positive".to_string(), 0.87);
            probs.insert("negative".to_string(), 0.13);
            probs
        },
        explanation: Some("Based on sentiment analysis".to_string()),
    };

    Ok(prediction)
}

// Agent Coordination
pub fn create_coordinator(coordinator_id: String) -> AgentCoordinator {
    crate::stdlib::log::info(
        "Creating agent coordinator",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "coordinator_id".to_string(),
                Value::String(coordinator_id.clone()),
            );
            data.insert(
                "message".to_string(),
                Value::String("Creating agent coordinator".to_string()),
            );
            data
        },
        Some("ai"),
    );

    AgentCoordinator {
        coordinator_id,
        agents: Vec::new(),
        workflows: Vec::new(),
        active_tasks: Vec::new(),
        message_bus: Vec::new(),
    }
}

pub fn add_agent_to_coordinator(coordinator: &mut AgentCoordinator, agent: Agent) {
    let agent_id = agent.id.clone();
    coordinator.agents.push(agent);

    crate::stdlib::log::info(
        "Agent added to coordinator",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "coordinator_id".to_string(),
                Value::String(coordinator.coordinator_id.clone()),
            );
            data.insert("agent_id".to_string(), Value::String(agent_id));
            data.insert(
                "message".to_string(),
                Value::String("Agent added to coordinator".to_string()),
            );
            data
        },
        Some("ai"),
    );
}

pub fn create_workflow(
    coordinator: &mut AgentCoordinator,
    name: String,
    steps: Vec<WorkflowStep>,
) -> Workflow {
    let workflow = Workflow {
        workflow_id: format!("workflow_{}", generate_id()),
        name,
        steps,
        status: WorkflowStatus::Pending,
        created_at: "2024-01-01T00:00:00Z".to_string(),
    };

    coordinator.workflows.push(workflow.clone());

    crate::stdlib::log::info(
        "Workflow created",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "workflow_id".to_string(),
                Value::String(workflow.workflow_id.clone()),
            );
            data.insert(
                "workflow_name".to_string(),
                Value::String(workflow.name.clone()),
            );
            data.insert("steps".to_string(), Value::Int(workflow.steps.len() as i64));
            data.insert(
                "message".to_string(),
                Value::String("Workflow created".to_string()),
            );
            data
        },
        Some("ai"),
    );

    workflow
}

pub fn execute_workflow(
    coordinator: &mut AgentCoordinator,
    workflow_id: &str,
) -> Result<bool, String> {
    let workflow_index = coordinator
        .workflows
        .iter()
        .position(|w| w.workflow_id == workflow_id)
        .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

    let workflow = &mut coordinator.workflows[workflow_index];
    workflow.status = WorkflowStatus::Running;

    // Collect step IDs and completed step IDs before mutable iteration
    let step_ids: Vec<_> = workflow.steps.iter().map(|s| s.step_id.clone()).collect();
    let completed_step_ids: Vec<_> = workflow
        .steps
        .iter()
        .filter(|s| matches!(s.status, StepStatus::Completed))
        .map(|s| s.step_id.clone())
        .collect();

    for step in &mut workflow.steps {
        // Check dependencies using the pre-collected data
        let dependencies_met = step.dependencies.iter().all(|dep_id| {
            step_ids.iter().any(|s_id| s_id == dep_id)
                && completed_step_ids.iter().any(|s_id| s_id == dep_id)
        });

        if dependencies_met {
            step.status = StepStatus::Running;

            // Find the agent for this step
            if let Some(agent) = coordinator
                .agents
                .iter_mut()
                .find(|a| a.id == step.agent_id)
            {
                // Create and execute task
                let _task = create_task(
                    agent,
                    step.task_type.clone(),
                    format!("Workflow step: {}", step.step_id),
                    HashMap::new(),
                )?;
                let _result = execute_task(agent, &_task.id)?;
                step.status = StepStatus::Completed;
            }
        }
    }

    workflow.status = WorkflowStatus::Completed;

    crate::stdlib::log::info(
        "Workflow executed successfully",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "workflow_id".to_string(),
                Value::String(workflow_id.to_string()),
            );
            data.insert(
                "message".to_string(),
                Value::String("Workflow executed successfully".to_string()),
            );
            data
        },
        Some("ai"),
    );

    Ok(true)
}

// Helper Functions
pub fn process_data_task(_task: &Task) -> Result<Value, String> {
    // Simulated data processing
    Ok(Value::String("Data processed successfully".to_string()))
}

pub fn handle_communication_task(_agent: &mut Agent, _task: &Task) -> Result<Value, String> {
    // Simulated communication task
    Ok(Value::String("Communication handled".to_string()))
}

/// Unique ID for agents, messages, tasks. Uses UUID v4 when available for request/session IDs.
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// Agent State Management
pub fn save_agent_state(agent: &Agent) -> Result<bool, String> {
    crate::stdlib::log::info(
        "Saving agent state",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_id".to_string(), Value::String(agent.id.clone()));
            data.insert(
                "message".to_string(),
                Value::String("Saving agent state".to_string()),
            );
            data
        },
        Some("ai"),
    );

    // Simulated state saving
    Ok(true)
}

pub fn load_agent_state(agent_id: &str) -> Result<Agent, String> {
    crate::stdlib::log::info(
        "Loading agent state",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_id".to_string(), Value::String(agent_id.to_string()));
            data.insert(
                "message".to_string(),
                Value::String("Loading agent state".to_string()),
            );
            data
        },
        Some("ai"),
    );

    // Simulated state loading
    Err("Agent state not found".to_string())
}

// Agent Communication Protocols
pub fn create_communication_protocol(
    name: String,
    supported_types: Vec<String>,
    encryption: bool,
    auth: bool,
) -> CommunicationProtocol {
    CommunicationProtocol {
        protocol_id: format!("protocol_{}", generate_id()),
        name,
        supported_message_types: supported_types,
        encryption_enabled: encryption,
        authentication_required: auth,
    }
}

pub fn validate_message_protocol(
    message: &Message,
    protocol: &CommunicationProtocol,
) -> Result<bool, String> {
    if !protocol
        .supported_message_types
        .contains(&message.message_type)
    {
        return Err(format!(
            "Message type {} not supported by protocol {}",
            message.message_type, protocol.name
        ));
    }

    Ok(true)
}

// Performance Monitoring
pub fn get_agent_metrics(agent: &Agent) -> HashMap<String, Value> {
    let mut metrics = HashMap::new();
    metrics.insert("agent_id".to_string(), Value::String(agent.id.clone()));
    metrics.insert("status".to_string(), Value::String(get_agent_status(agent)));
    metrics.insert(
        "tasks_count".to_string(),
        Value::Int(agent.tasks.len() as i64),
    );
    metrics.insert(
        "messages_count".to_string(),
        Value::Int(agent.message_queue.len() as i64),
    );
    metrics.insert(
        "memory_entries".to_string(),
        Value::Int(agent.memory.len() as i64),
    );
    metrics.insert(
        "created_at".to_string(),
        Value::String(agent.created_at.clone()),
    );
    metrics.insert(
        "last_active".to_string(),
        Value::String(agent.last_active.clone()),
    );

    metrics
}

pub fn get_coordinator_metrics(coordinator: &AgentCoordinator) -> HashMap<String, Value> {
    let mut metrics = HashMap::new();
    metrics.insert(
        "coordinator_id".to_string(),
        Value::String(coordinator.coordinator_id.clone()),
    );
    metrics.insert(
        "agents_count".to_string(),
        Value::Int(coordinator.agents.len() as i64),
    );
    metrics.insert(
        "workflows_count".to_string(),
        Value::Int(coordinator.workflows.len() as i64),
    );
    metrics.insert(
        "active_tasks".to_string(),
        Value::Int(coordinator.active_tasks.len() as i64),
    );
    metrics.insert(
        "messages_in_bus".to_string(),
        Value::Int(coordinator.message_bus.len() as i64),
    );

    metrics
}

// ============================================================================
// SIMPLIFIED WRAPPER API (Phase 4.1)
// ============================================================================
// **Simplified vs full API:** Most functions have two behaviors:
// - **Simplified:** When OPENAI_API_KEY is not set (or http-interface is off),
//   local mocks are used: rule-based classify, hash-based embeddings, keyword
//   recommend, placeholder image analysis/generation.
// - **Full:** When OPENAI_API_KEY (and optionally OPENAI_BASE_URL) are set and
//   http-interface is enabled, real APIs are called: vision for image analysis,
//   /images/generations for image gen, embeddings for recommend. For chat/classify
//   use service::ai() with AIService or the env-based path in classify/generate.
// **Audio:** No dedicated speech-to-text or text-to-speech functions yet; a full
// API would call an audio model or service when configured.
// ============================================================================

/// Model Registry for named models (thread-safe, avoids mutable static)
static MODEL_REGISTRY: OnceLock<Mutex<HashMap<String, Model>>> = OnceLock::new();

fn get_model_registry() -> std::sync::MutexGuard<'static, HashMap<String, Model>> {
    MODEL_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
}

/// Register a trained model with a name for easy access
pub fn register_model(name: String, model: Model) {
    let mut registry = get_model_registry();

    crate::stdlib::log::info(
        "Model registered",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("model_name".to_string(), Value::String(name.clone()));
            data.insert(
                "message".to_string(),
                Value::String("Model registered".to_string()),
            );
            data
        },
        Some("ai"),
    );

    registry.insert(name, model);
}

/// Get a registered model by name
pub fn get_model(name: &str) -> Option<Model> {
    let registry = get_model_registry();
    registry.get(name).cloned()
}

// ============================================================================
// SIMPLIFIED AI FUNCTIONS
// ============================================================================

/// Classify text using a named model (simplified API)
///
/// This is a convenience wrapper that:
/// 1. Creates a temporary agent
/// 2. Performs text analysis
/// 3. Returns a simplified classification result
/// 4. Cleans up automatically
pub fn classify(model: &str, input: &str) -> Result<String, String> {
    crate::stdlib::log::info(
        "Classifying text (simplified API)",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("model".to_string(), Value::String(model.to_string()));
            data.insert("input_length".to_string(), Value::Int(input.len() as i64));
            data.insert(
                "message".to_string(),
                Value::String("Classifying text (simplified API)".to_string()),
            );
            data
        },
        Some("ai"),
    );

    // Optional real API path when API key is set (OPENAI_API_KEY or DAL_OPENAI_API_KEY)
    #[cfg(feature = "http-interface")]
    if let Some(api_key) = effective_openai_api_key() {
        let base = env::var("OPENAI_BASE_URL")
            .or_else(|_| env::var("DAL_OPENAI_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let svc = crate::stdlib::service::AIService::new(model.to_string())
            .with_api_key(api_key)
            .with_base_url(base);
        let prompt = format!(
            "Classify the following text. Reply with only one word or short phrase (the category). Do not explain.\n\nText: {}",
            input
        );
        if let Ok(resp) = crate::stdlib::service::ai(&prompt, svc) {
            let label = resp.lines().next().map(str::trim).unwrap_or("").to_string();
            if !label.is_empty() {
                return Ok(label);
            }
        }
    }

    // Fallback: built-in text analysis
    let analysis = analyze_text(input.to_string())?;

    // Map model type to classification
    match model {
        "sentiment_model" | "sentiment" => {
            // Sentiment: > 0.7 = positive, < 0.3 = negative, else neutral
            if analysis.sentiment > 0.7 {
                Ok("positive".to_string())
            } else if analysis.sentiment < 0.3 {
                Ok("negative".to_string())
            } else {
                Ok("neutral".to_string())
            }
        }
        "spam_detector" | "spam" => {
            // Spam detection based on sentiment and keywords
            let has_spam_keywords = analysis.keywords.iter().any(|k| {
                k.to_lowercase().contains("free")
                    || k.to_lowercase().contains("win")
                    || k.to_lowercase().contains("click")
            });

            if has_spam_keywords {
                Ok("spam".to_string())
            } else {
                Ok("legitimate".to_string())
            }
        }
        "topic_classifier" | "topic" => {
            // Simple topic classification based on keywords
            if !analysis.keywords.is_empty() {
                Ok(analysis.keywords[0].clone())
            } else {
                Ok("general".to_string())
            }
        }
        "intent_classifier" | "intent" => {
            // Intent detection
            let text_lower = input.to_lowercase();
            if text_lower.contains("buy") || text_lower.contains("purchase") {
                Ok("buy_intent".to_string())
            } else if text_lower.contains("sell") {
                Ok("sell_intent".to_string())
            } else if text_lower.contains("help") || text_lower.contains("?") {
                Ok("help_intent".to_string())
            } else {
                Ok("general_intent".to_string())
            }
        }
        "risk_classifier" | "risk" => {
            // Risk classification based on sentiment
            if analysis.sentiment < 0.3 {
                Ok("high_risk".to_string())
            } else if analysis.sentiment > 0.7 {
                Ok("low_risk".to_string())
            } else {
                Ok("medium_risk".to_string())
            }
        }
        _ => {
            // Default: return sentiment-based classification
            if analysis.sentiment > 0.5 {
                Ok("positive".to_string())
            } else {
                Ok("negative".to_string())
            }
        }
    }
}

/// Classify with confidence score
pub fn classify_with_confidence(model: &str, input: &str) -> Result<(String, f64), String> {
    let analysis = analyze_text(input.to_string())?;
    let classification = classify(model, input)?;
    Ok((classification, analysis.confidence))
}

/// Generate text using a named model (simplified API). When an API key is configured (env OPENAI_API_KEY; any compatible provider), calls real LLM via service::ai().
pub fn generate(model: &str, prompt: &str) -> Result<String, String> {
    crate::stdlib::log::info(
        "Generating text (simplified API)",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("model".to_string(), Value::String(model.to_string()));
            data.insert("prompt_length".to_string(), Value::Int(prompt.len() as i64));
            data.insert(
                "message".to_string(),
                Value::String("Generating text (simplified API)".to_string()),
            );
            data
        },
        Some("ai"),
    );

    // Optional real API path when API key is set (OPENAI_API_KEY or DAL_OPENAI_API_KEY)
    #[cfg(feature = "http-interface")]
    if let Some(api_key) = effective_openai_api_key() {
        let base = env::var("OPENAI_BASE_URL")
            .or_else(|_| env::var("DAL_OPENAI_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let svc = crate::stdlib::service::AIService::new(model.to_string())
            .with_api_key(api_key)
            .with_base_url(base);
        if let Ok(resp) = crate::stdlib::service::ai(prompt, svc) {
            return Ok(resp);
        }
    }

    // Fallback: built-in text generation
    let mut response = generate_text(prompt.to_string())?;

    // Add model-specific formatting
    match model {
        "gpt-4" | "gpt-3.5" => {
            response = format!("[GPT] {}", response);
        }
        "claude-3" | "claude" => {
            response = format!("[Claude] {}", response);
        }
        "llama-3" | "llama" => {
            response = format!("[Llama] {}", response);
        }
        "mistral" => {
            response = format!("[Mistral] {}", response);
        }
        _ => {
            // Default: no prefix
        }
    }

    Ok(response)
}

/// Generate embeddings for text (simplified API). When an API key is configured (env OPENAI_API_KEY; any provider with /embeddings), calls service::embeddings().
pub fn embed(text: &str) -> Result<Vec<f64>, String> {
    crate::stdlib::log::info(
        "Generating embeddings",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("text_length".to_string(), Value::Int(text.len() as i64));
            data.insert(
                "message".to_string(),
                Value::String("Generating embeddings".to_string()),
            );
            data
        },
        Some("ai"),
    );

    // Optional real API path when API key is set (OPENAI_API_KEY or DAL_OPENAI_API_KEY)
    #[cfg(feature = "http-interface")]
    if let Some(api_key) = effective_openai_api_key() {
        let base = env::var("OPENAI_BASE_URL")
            .or_else(|_| env::var("DAL_OPENAI_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let svc = crate::stdlib::service::AIService::new("text-embedding-3-small".to_string())
            .with_api_key(api_key)
            .with_base_url(base);
        if let Ok(vec) = crate::stdlib::service::embeddings(text, svc) {
            return Ok(vec);
        }
    }

    // Fallback: hash-based embedding
    let mut embeddings = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();

    // Generate 384-dimensional embeddings (common size)
    for i in 0..384 {
        let mut value = 0.0;
        for (j, word) in words.iter().enumerate() {
            let hash = simple_hash(word, i);
            value += (hash as f64 / 1000000.0) * (1.0 / (j + 1) as f64);
        }
        value = value.tanh(); // Normalize to [-1, 1]
        embeddings.push(value);
    }

    Ok(embeddings)
}

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(vec1: &[f64], vec2: &[f64]) -> Result<f64, String> {
    if vec1.len() != vec2.len() {
        return Err("Vectors must have the same length".to_string());
    }

    let mut dot_product = 0.0;
    let mut norm1 = 0.0;
    let mut norm2 = 0.0;

    for i in 0..vec1.len() {
        dot_product += vec1[i] * vec2[i];
        norm1 += vec1[i] * vec1[i];
        norm2 += vec2[i] * vec2[i];
    }

    norm1 = norm1.sqrt();
    norm2 = norm2.sqrt();

    if norm1 == 0.0 || norm2 == 0.0 {
        return Ok(0.0);
    }

    Ok(dot_product / (norm1 * norm2))
}

/// Detect anomalies in data (simplified API)
pub fn detect_anomaly(data: &[f64], new_value: f64) -> Result<bool, String> {
    if data.is_empty() {
        return Ok(false);
    }

    crate::stdlib::log::info(
        "Detecting anomaly",
        {
            let mut log_data = std::collections::HashMap::new();
            log_data.insert("data_points".to_string(), Value::Int(data.len() as i64));
            log_data.insert("new_value".to_string(), Value::Int(new_value as i64));
            log_data.insert(
                "message".to_string(),
                Value::String("Detecting anomaly in data".to_string()),
            );
            log_data
        },
        Some("ai"),
    );

    // Calculate mean and standard deviation
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
    let std_dev = variance.sqrt();

    // Z-score threshold for anomaly detection
    let z_score = (new_value - mean).abs() / std_dev;
    let threshold = 3.0; // 3 standard deviations

    Ok(z_score > threshold)
}

/// Predict using a named model (simplified API)
pub fn predict_with_model(model_name: &str, input: Value) -> Result<Value, String> {
    crate::stdlib::log::info(
        "Making prediction (simplified API)",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "model_name".to_string(),
                Value::String(model_name.to_string()),
            );
            data.insert(
                "message".to_string(),
                Value::String("Making prediction (simplified API)".to_string()),
            );
            data
        },
        Some("ai"),
    );

    // Try to get registered model
    if let Some(model) = get_model(model_name) {
        let prediction = predict(&model, input)?;
        return Ok(prediction.prediction);
    }

    // Fall back to built-in prediction logic based on model name
    match model_name {
        "price_model" | "price_predictor" => {
            // Simple price prediction
            if let Value::Array(prices) = input {
                let sum: i64 = prices
                    .iter()
                    .filter_map(|v| match v {
                        Value::Int(i) => Some(i),
                        _ => None,
                    })
                    .sum();
                let avg = if !prices.is_empty() {
                    sum / prices.len() as i64
                } else {
                    0
                };
                // Predict slight increase
                Ok(Value::Int(avg + (avg / 20))) // +5%
            } else {
                Err("Invalid input for price prediction".to_string())
            }
        }
        "risk_model" | "risk_predictor" => {
            // Risk score (0-100)
            Ok(Value::Int(50)) // Default medium risk
        }
        _ => Err(format!("Model '{}' not found", model_name)),
    }
}

/// Analyze image from URL. **Full API:** when OPENAI_API_KEY is set and http-interface enabled, sends image URL to vision API. **Simplified:** returns mock analysis when no API key.
pub fn analyze_image_url(url: &str) -> Result<ImageAnalysis, String> {
    crate::stdlib::log::info(
        "Analyzing image from URL",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("url".to_string(), Value::String(url.to_string()));
            data.insert(
                "message".to_string(),
                Value::String("Analyzing image from URL".to_string()),
            );
            data
        },
        Some("ai"),
    );

    #[cfg(feature = "http-interface")]
    if let Some(api_key) = effective_openai_api_key() {
        let base = env::var("OPENAI_BASE_URL")
            .or_else(|_| env::var("DAL_OPENAI_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let svc = crate::stdlib::service::AIService::new("gpt-4o".to_string())
            .with_api_key(api_key)
            .with_base_url(base);
        if let Ok(description) = crate::stdlib::service::vision_analyze(svc, Some(url), None) {
            return Ok(ImageAnalysis {
                objects: vec![DetectedObject {
                    object_type: "described".to_string(),
                    confidence: 0.9,
                    bounding_box: BoundingBox {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                    },
                }],
                faces: vec![],
                text: vec![description],
                colors: vec![],
                quality_score: 0.9,
            });
        }
    }

    // Simplified: no image data to analyze locally
    analyze_image(vec![])
}

/// Generate image from prompt. **Full API:** when an API key is configured (env OPENAI_API_KEY; any provider with /images/generations), returns image URL or base64. **Simplified:** returns a placeholder URL when no API key.
pub fn generate_image(model: &str, prompt: &str) -> Result<String, String> {
    let msg = "Generating image from prompt";
    crate::stdlib::log::info(
        msg,
        {
            let mut data = std::collections::HashMap::new();
            data.insert("model".to_string(), Value::String(model.to_string()));
            data.insert("prompt".to_string(), Value::String(prompt.to_string()));
            data.insert("message".to_string(), Value::String(msg.to_string()));
            data
        },
        Some("ai"),
    );

    #[cfg(feature = "http-interface")]
    if let Some(api_key) = effective_openai_api_key() {
        let base = env::var("OPENAI_BASE_URL")
            .or_else(|_| env::var("DAL_OPENAI_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let image_model = if model.is_empty() || model == "default" {
            "dall-e-2"
        } else {
            model
        };
        let svc = crate::stdlib::service::AIService::new(image_model.to_string())
            .with_api_key(api_key)
            .with_base_url(base);
        if let Ok(url_or_b64) = crate::stdlib::service::image_generate(svc, prompt) {
            return Ok(url_or_b64);
        }
    }

    // Simplified: placeholder URL
    Ok(format!(
        "https://ai-generated-images.example.com/{}/{}",
        model,
        simple_hash_str(prompt, 0)
    ))
}

/// Recommend items based on preferences. **Full API:** when an API key is configured (env OPENAI_API_KEY; any provider with embeddings), embeds preferences and items and ranks by cosine similarity. **Simplified:** keyword matching when no API key.
pub fn recommend(
    user_preferences: Vec<String>,
    available_items: Vec<String>,
    count: usize,
) -> Result<Vec<String>, String> {
    crate::stdlib::log::info(
        "Generating recommendations",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "preferences_count".to_string(),
                Value::Int(user_preferences.len() as i64),
            );
            data.insert(
                "items_count".to_string(),
                Value::Int(available_items.len() as i64),
            );
            data.insert(
                "message".to_string(),
                Value::String("Generating recommendations".to_string()),
            );
            data
        },
        Some("ai"),
    );

    #[cfg(feature = "http-interface")]
    if let Ok(pref_emb) = embed(&user_preferences.join(" ")) {
        let mut scored: Vec<(String, f64)> = Vec::new();
        for item in &available_items {
            if let Ok(item_emb) = embed(item) {
                if let Ok(sim) = cosine_similarity(&pref_emb, &item_emb) {
                    scored.push((item.clone(), sim));
                }
            }
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        return Ok(scored.into_iter().take(count).map(|(s, _)| s).collect());
    }

    // Simplified: keyword matching
    let mut recommendations = Vec::new();
    for item in available_items.iter() {
        let mut score = 0;
        for pref in user_preferences.iter() {
            if item.to_lowercase().contains(&pref.to_lowercase()) {
                score += 1;
            }
        }
        if score > 0 {
            recommendations.push((item.clone(), score));
        }
    }
    recommendations.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(recommendations
        .iter()
        .take(count)
        .map(|(item, _)| item.clone())
        .collect())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Simple hash function for embedding generation
fn simple_hash(text: &str, seed: usize) -> u64 {
    let mut hash: u64 = seed as u64;
    for byte in text.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

fn simple_hash_str(text: &str, seed: usize) -> String {
    format!("{:x}", simple_hash(text, seed))
}

// ============================================================================
// TESTS FOR WRAPPER API
// ============================================================================

#[cfg(test)]
mod wrapper_tests {
    use super::*;

    #[test]
    fn test_classify_sentiment() {
        let result = classify("sentiment", "This is amazing! I love it!");
        assert!(result.is_ok());
        let classification = result.unwrap();
        // Accept any valid sentiment
        assert!(
            classification == "positive"
                || classification == "neutral"
                || classification == "negative"
        );

        let result = classify("sentiment", "This is terrible and awful.");
        assert!(result.is_ok());
        let classification = result.unwrap();
        assert!(
            classification == "positive"
                || classification == "neutral"
                || classification == "negative"
        );
    }

    #[test]
    fn test_classify_with_confidence() {
        let result = classify_with_confidence("sentiment", "Great product!");
        assert!(result.is_ok());
        let (classification, confidence) = result.unwrap();
        // Accept any valid sentiment
        assert!(
            classification == "positive"
                || classification == "neutral"
                || classification == "negative"
        );
        assert!(confidence > 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_generate() {
        let result = generate("gpt-4", "Explain blockchain");
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("GPT"));
    }

    #[test]
    fn test_embed() {
        let result = embed("Hello world");
        assert!(result.is_ok());
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 384);

        // Check values are in reasonable range
        for val in embeddings {
            assert!((-1.0..=1.0).contains(&val));
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![1.0, 0.0, 0.0];
        let result = cosine_similarity(&vec1, &vec2);
        assert!(result.is_ok());
        assert!((result.unwrap() - 1.0).abs() < 0.001);

        let vec3 = vec![0.0, 1.0, 0.0];
        let result = cosine_similarity(&vec1, &vec3);
        assert!(result.is_ok());
        assert!((result.unwrap() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_detect_anomaly() {
        let data = vec![10.0, 12.0, 11.0, 13.0, 10.5];

        // Normal value
        let result = detect_anomaly(&data, 11.5);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Anomalous value
        let result = detect_anomaly(&data, 50.0);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_recommend() {
        let preferences = vec!["blockchain".to_string(), "defi".to_string()];
        let items = vec![
            "Blockchain Tutorial".to_string(),
            "DeFi Protocol".to_string(),
            "Web Development".to_string(),
            "Blockchain DeFi Guide".to_string(),
        ];

        let result = recommend(preferences, items, 2);
        assert!(result.is_ok());
        let recommendations = result.unwrap();
        assert_eq!(recommendations.len(), 2);
        assert!(recommendations[0].contains("Blockchain") || recommendations[0].contains("DeFi"));
    }

    #[test]
    fn test_model_registry() {
        let model = Model {
            model_id: "test_model".to_string(),
            model_type: "classifier".to_string(),
            version: "1.0.0".to_string(),
            accuracy: 0.95,
            training_data_size: 1000,
            created_at: "2024-01-01".to_string(),
            last_updated: "2024-01-01".to_string(),
        };

        register_model("test".to_string(), model.clone());

        let retrieved = get_model("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().model_id, "test_model");
    }
}

use crate::runtime::values::Value;
#[cfg(feature = "http-interface")]
use base64::Engine;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

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
    if let Ok(key) = env::var("OPENAI_API_KEY") {
        if !key.is_empty() && key != "none" {
            config.provider = AIProvider::OpenAI;
            config.api_key = Some(key);
            if let Ok(model) = env::var("OPENAI_MODEL") {
                config.model = Some(model);
            }
        }
    } else if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
        if !key.is_empty() && key != "none" {
            config.provider = AIProvider::Anthropic;
            config.api_key = Some(key);
            if let Ok(model) = env::var("ANTHROPIC_MODEL") {
                config.model = Some(model);
            }
        }
    } else if let Ok(endpoint) = env::var("DAL_AI_ENDPOINT") {
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

/// Parse configuration file (simple key=value format)
/// TODO: Use proper TOML parser for production
fn parse_config_file(content: &str) -> Option<AIConfig> {
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
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        let base =
            env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
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

pub fn generate_text(prompt: String) -> Result<String, String> {
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

    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        if !api_key.is_empty() && api_key != "none" {
            match call_openai_api(&prompt, &api_key, &config) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    eprintln!("OpenAI failed: {}. Trying next provider...", e);
                }
            }
        }
    }

    if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
        if !api_key.is_empty() && api_key != "none" {
            match call_anthropic_api(&prompt, &api_key, &config) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    eprintln!("Anthropic failed: {}. Trying next provider...", e);
                }
            }
        }
    }

    if let Ok(endpoint) = env::var("DAL_AI_ENDPOINT") {
        if !endpoint.is_empty() {
            match call_local_model(&prompt, &endpoint, &config) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    eprintln!("Local model failed: {}. Using fallback...", e);
                }
            }
        }
    }

    // Fallback to simulated response
    Ok(format!("Generated response to: {}", prompt))
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

    // Optional real API path when OPENAI_API_KEY (and optionally OPENAI_BASE_URL) are set
    #[cfg(feature = "http-interface")]
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        let base =
            env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
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

    // Optional real API path when OPENAI_API_KEY (and optionally OPENAI_BASE_URL) are set
    #[cfg(feature = "http-interface")]
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        let base =
            env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
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

    // Optional real API path when OPENAI_API_KEY (and optionally OPENAI_BASE_URL) are set
    #[cfg(feature = "http-interface")]
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        let base =
            env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
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
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        let base =
            env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
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
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        let base =
            env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
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
            assert!(val >= -1.0 && val <= 1.0);
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
        assert_eq!(result.unwrap(), false);

        // Anomalous value
        let result = detect_anomaly(&data, 50.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
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

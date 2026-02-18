use crate::runtime::values::Value;
use crate::stdlib::crypto;
use std::collections::HashMap;
use std::env;

/// Configuration namespace for secure environment variable and secrets management
/// Provides type-safe configuration access with encryption and validation

#[derive(Debug, Clone)]
pub struct ConfigManager {
    pub environment: String,
    pub encrypted_secrets: HashMap<String, String>,
    pub config_cache: HashMap<String, Value>,
    pub validation_rules: HashMap<String, ConfigValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ConfigValidationRule {
    pub required: bool,
    pub default_value: Option<Value>,
    pub validation_type: ValidationType,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub allowed_values: Option<Vec<String>>,
    pub regex_pattern: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ValidationType {
    String,
    Integer,
    Float,
    Boolean,
    URL,
    Email,
    PrivateKey,
    PublicKey,
    JWT,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct SecretConfig {
    pub key: String,
    pub encrypted_value: String,
    pub encryption_algorithm: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub access_count: i64,
}

impl ConfigManager {
    pub fn new() -> Self {
        let environment = env::var("DIST_AGENT_ENV").unwrap_or_else(|_| "development".to_string());

        Self {
            environment,
            encrypted_secrets: HashMap::new(),
            config_cache: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }

    /// Get environment variable with type safety and validation
    pub fn get_env(
        key: &str,
        validation_rule: Option<ConfigValidationRule>,
    ) -> Result<Value, String> {
        let value =
            env::var(key).map_err(|_| format!("Environment variable '{}' not found", key))?;

        if let Some(rule) = validation_rule {
            Self::validate_value(&value, &rule)?;
        }

        Ok(Value::String(value))
    }

    /// Get environment variable with default value
    pub fn get_env_or_default(key: &str, default: Value) -> Value {
        match env::var(key) {
            Ok(value) => Value::String(value),
            Err(_) => default,
        }
    }

    /// Get required environment variable (fails if not found)
    pub fn get_required_env(key: &str) -> Result<Value, String> {
        let rule = ConfigValidationRule {
            required: true,
            default_value: None,
            validation_type: ValidationType::String,
            min_length: Some(1),
            max_length: None,
            allowed_values: None,
            regex_pattern: None,
        };

        Self::get_env(key, Some(rule))
    }

    /// Store encrypted secret
    pub fn store_secret(
        &mut self,
        key: &str,
        value: &str,
        encryption_key: &str,
    ) -> Result<(), String> {
        let encrypted_value = crypto::encrypt_aes256(value, encryption_key)?;

        let secret = SecretConfig {
            key: key.to_string(),
            encrypted_value,
            encryption_algorithm: "AES-256-GCM".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            expires_at: None,
            access_count: 0,
        };

        self.encrypted_secrets
            .insert(key.to_string(), secret.encrypted_value);
        Ok(())
    }

    /// Retrieve decrypted secret
    pub fn get_secret(&mut self, key: &str, encryption_key: &str) -> Result<String, String> {
        let encrypted_value = self
            .encrypted_secrets
            .get(key)
            .ok_or_else(|| format!("Secret '{}' not found", key))?;

        crypto::decrypt_aes256(encrypted_value, encryption_key)
    }

    /// Database configuration with environment variables
    pub fn get_database_config() -> Result<HashMap<String, Value>, String> {
        let mut config = HashMap::new();

        config.insert("host".to_string(), Self::get_required_env("DB_HOST")?);
        config.insert(
            "port".to_string(),
            Self::get_env_or_default("DB_PORT", Value::Int(5432)),
        );
        config.insert("database".to_string(), Self::get_required_env("DB_NAME")?);
        config.insert("username".to_string(), Self::get_required_env("DB_USER")?);
        config.insert(
            "password".to_string(),
            Self::get_required_env("DB_PASSWORD")?,
        );
        config.insert(
            "ssl_mode".to_string(),
            Self::get_env_or_default("DB_SSL_MODE", Value::String("require".to_string())),
        );

        Ok(config)
    }

    /// API configuration with environment variables
    pub fn get_api_config() -> Result<HashMap<String, Value>, String> {
        let mut config = HashMap::new();

        config.insert(
            "base_url".to_string(),
            Self::get_required_env("API_BASE_URL")?,
        );
        config.insert(
            "timeout".to_string(),
            Self::get_env_or_default("API_TIMEOUT", Value::Int(30000)),
        );
        config.insert(
            "retry_count".to_string(),
            Self::get_env_or_default("API_RETRY_COUNT", Value::Int(3)),
        );

        // API keys
        if let Ok(api_key) = Self::get_env("API_KEY", None) {
            config.insert("api_key".to_string(), api_key);
        }

        Ok(config)
    }

    /// Blockchain configuration with environment variables
    pub fn get_blockchain_config() -> Result<HashMap<String, Value>, String> {
        let mut config = HashMap::new();

        config.insert(
            "rpc_url".to_string(),
            Self::get_required_env("BLOCKCHAIN_RPC_URL")?,
        );
        config.insert(
            "chain_id".to_string(),
            Self::get_required_env("BLOCKCHAIN_CHAIN_ID")?,
        );
        config.insert(
            "private_key".to_string(),
            Self::get_required_env("BLOCKCHAIN_PRIVATE_KEY")?,
        );
        config.insert(
            "gas_limit".to_string(),
            Self::get_env_or_default("BLOCKCHAIN_GAS_LIMIT", Value::Int(21000)),
        );
        config.insert(
            "gas_price".to_string(),
            Self::get_env_or_default("BLOCKCHAIN_GAS_PRICE", Value::Float(20.0)),
        );

        Ok(config)
    }

    /// AI service configuration
    pub fn get_ai_config() -> Result<HashMap<String, Value>, String> {
        let mut config = HashMap::new();

        config.insert(
            "model".to_string(),
            Self::get_env_or_default("AI_MODEL", Value::String("gpt-4".to_string())),
        );
        config.insert(
            "temperature".to_string(),
            Self::get_env_or_default("AI_TEMPERATURE", Value::Float(0.7)),
        );
        config.insert(
            "max_tokens".to_string(),
            Self::get_env_or_default("AI_MAX_TOKENS", Value::Int(1000)),
        );

        if let Ok(api_key) = Self::get_env("AI_API_KEY", None) {
            config.insert("api_key".to_string(), api_key);
        }

        Ok(config)
    }

    /// Validate configuration value against rules
    fn validate_value(value: &str, rule: &ConfigValidationRule) -> Result<(), String> {
        // Check required
        if rule.required && value.is_empty() {
            return Err("Value is required but empty".to_string());
        }

        // Check length constraints
        if let Some(min_len) = rule.min_length {
            if value.len() < min_len {
                return Err(format!("Value too short, minimum length: {}", min_len));
            }
        }

        if let Some(max_len) = rule.max_length {
            if value.len() > max_len {
                return Err(format!("Value too long, maximum length: {}", max_len));
            }
        }

        // Check allowed values
        if let Some(allowed) = &rule.allowed_values {
            if !allowed.contains(&value.to_string()) {
                return Err(format!(
                    "Value '{}' not in allowed values: {:?}",
                    value, allowed
                ));
            }
        }

        // Type-specific validation
        match rule.validation_type {
            ValidationType::Integer => {
                value
                    .parse::<i64>()
                    .map_err(|_| "Value is not a valid integer".to_string())?;
            }
            ValidationType::Float => {
                value
                    .parse::<f64>()
                    .map_err(|_| "Value is not a valid float".to_string())?;
            }
            ValidationType::Boolean => {
                if !["true", "false", "1", "0"].contains(&value.to_lowercase().as_str()) {
                    return Err("Value is not a valid boolean".to_string());
                }
            }
            ValidationType::URL => {
                if !value.starts_with("http://") && !value.starts_with("https://") {
                    return Err("Value is not a valid URL".to_string());
                }
            }
            ValidationType::Email => {
                if !value.contains('@') || !value.contains('.') {
                    return Err("Value is not a valid email address".to_string());
                }
            }
            ValidationType::PrivateKey => {
                if !value.starts_with("0x") || value.len() != 66 {
                    return Err("Value is not a valid private key".to_string());
                }
            }
            ValidationType::PublicKey => {
                if !value.starts_with("0x") || value.len() != 130 {
                    return Err("Value is not a valid public key".to_string());
                }
            }
            ValidationType::JWT => {
                let parts: Vec<&str> = value.split('.').collect();
                if parts.len() != 3 {
                    return Err("Value is not a valid JWT token".to_string());
                }
            }
            _ => {} // Custom validation types handled separately
        }

        Ok(())
    }

    /// Load configuration from file
    pub fn load_from_file(_path: &str) -> Result<HashMap<String, Value>, String> {
        // Implementation would read from JSON, YAML, or TOML files
        // For now, return empty config
        Ok(HashMap::new())
    }

    /// Export configuration to environment variables (for testing)
    pub fn export_to_env(config: &HashMap<String, Value>) -> Result<(), String> {
        for (key, value) in config {
            let string_value = match value {
                Value::String(s) => s.clone(),
                Value::Int(i) => i.to_string(),
                Value::Float(f) => f.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => return Err(format!("Cannot export value type for key: {}", key)),
            };

            env::set_var(key, string_value);
        }

        Ok(())
    }
}

/// Get environment variable with validation
pub fn get_env(key: &str) -> Result<Value, String> {
    ConfigManager::get_env(key, None)
}

/// Get environment variable with default
pub fn get_env_or_default(key: &str, default: Value) -> Value {
    ConfigManager::get_env_or_default(key, default)
}

/// Get required environment variable
pub fn get_required_env(key: &str) -> Result<Value, String> {
    ConfigManager::get_required_env(key)
}

/// Get database configuration
pub fn get_database_config() -> Result<HashMap<String, Value>, String> {
    ConfigManager::get_database_config()
}

/// Get API configuration
pub fn get_api_config() -> Result<HashMap<String, Value>, String> {
    ConfigManager::get_api_config()
}

/// Get blockchain configuration
pub fn get_blockchain_config() -> Result<HashMap<String, Value>, String> {
    ConfigManager::get_blockchain_config()
}

/// Get AI configuration
pub fn get_ai_config() -> Result<HashMap<String, Value>, String> {
    ConfigManager::get_ai_config()
}

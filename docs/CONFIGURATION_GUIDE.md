# 🔐 Configuration Management Guide for dist_agent_lang

## Overview

This guide covers the comprehensive configuration management system in `dist_agent_lang`, including secure environment variable handling, encrypted secrets management, and configuration validation. The system is designed to provide type-safe, secure, and flexible configuration management for both development and production environments.

---

## 🏗️ **Core Components**

### **1. Configuration Manager (`config::ConfigManager`)**
- **Environment variable access** with type safety and validation
- **Encrypted secrets storage** using AES-256-GCM encryption
- **Configuration caching** for performance optimization
- **Validation rules** for data integrity

### **2. Validation System**
- **Type validation** (String, Integer, Float, Boolean, URL, Email, PrivateKey, JWT)
- **Length constraints** (min/max length)
- **Value constraints** (allowed values, regex patterns)
- **Required field validation**

### **3. Secrets Management**
- **Encrypted storage** of sensitive data
- **Access logging** for audit trails
- **Secret rotation** capabilities
- **Metadata tracking** (creation time, access count, etc.)

---

## 🚀 **Quick Start**

### **Basic Environment Variable Usage**

```rust
// Get required environment variable
let db_host = config::get_required_env("DB_HOST")?;

// Get environment variable with default
let db_port = config::get_env_or_default("DB_PORT", Value::Int(5432));

// Get optional environment variable
let api_key = config::get_env("API_KEY");
```

### **Database Configuration**

```rust
// Get complete database configuration
let db_config = config::get_database_config()?;

// Use in database connection
let connection = database::connect("postgresql", db_config);
```

### **API Configuration**

```rust
// Get API configuration with defaults
let api_config = config::get_api_config()?;

// Create HTTP client
let client = web::create_http_client(api_config);
```

---

## 🔧 **Environment Variables**

### **Required Environment Variables**

| Variable | Description | Example |
|----------|-------------|---------|
| `DB_HOST` | Database host address | `localhost` |
| `DB_NAME` | Database name | `myapp` |
| `DB_USER` | Database username | `app_user` |
| `DB_PASSWORD` | Database password | `secure_password` |
| `API_BASE_URL` | API base URL | `https://api.example.com` |
| `BLOCKCHAIN_RPC_URL` | Blockchain RPC endpoint | `https://mainnet.infura.io/v3/...` |
| `BLOCKCHAIN_CHAIN_ID` | Blockchain chain ID | `1` (Ethereum) |
| `BLOCKCHAIN_PRIVATE_KEY` | Wallet private key | `0x1234...` |
| `MASTER_ENCRYPTION_KEY` | Master encryption key | `32+ character key` |

### **Optional Environment Variables**

| Variable | Description | Default |
|----------|-------------|---------|
| `DIST_AGENT_ENV` | Environment name | `development` |
| `DB_PORT` | Database port | `5432` |
| `DB_SSL_MODE` | Database SSL mode | `require` |
| `API_TIMEOUT` | API timeout (ms) | `30000` |
| `API_RETRY_COUNT` | API retry count | `3` |
| `AI_MODEL` | AI model name | `gpt-4` |
| `AI_TEMPERATURE` | AI temperature | `0.7` |
| `AI_MAX_TOKENS` | AI max tokens | `1000` |

---

## 🔐 **Secrets Management**

### **Storing Encrypted Secrets**

```rust
// Initialize secrets management
let mut config_manager = config::ConfigManager::new();

// Store sensitive data encrypted
config_manager.store_secret(
    "stripe_secret_key",
    "sk_test_123456789",
    master_encryption_key
)?;
```

### **Retrieving Encrypted Secrets**

```rust
// Retrieve decrypted secret
let stripe_key = config_manager.get_secret(
    "stripe_secret_key",
    master_encryption_key
)?;

// Use in API client
let client = web::create_http_client({
    "base_url": "https://api.stripe.com/v1",
    "headers": {
        "Authorization": format!("Bearer {}", stripe_key)
    }
});
```

### **Secret Rotation**

```rust
// Rotate secret with new value
secrets_service.rotate_secret(
    "api_key",
    "new_secret_value",
    master_encryption_key
)?;
```

---

## ✅ **Configuration Validation**

### **Validation Rules**

```rust
// Define validation rules
let db_host_rule = ConfigValidationRule {
    required: true,
    validation_type: ValidationType::String,
    min_length: Some(1),
    max_length: Some(255),
    allowed_values: None,
    regex_pattern: None,
};

let db_port_rule = ConfigValidationRule {
    required: false,
    validation_type: ValidationType::Integer,
    default_value: Some(Value::Int(5432)),
    min_length: None,
    max_length: None,
    allowed_values: None,
    regex_pattern: None,
};

let private_key_rule = ConfigValidationRule {
    required: true,
    validation_type: ValidationType::PrivateKey,
    min_length: Some(66),
    max_length: Some(66),
    allowed_values: None,
    regex_pattern: None,
};
```

### **Validation Types**

| Type | Description | Validation |
|------|-------------|------------|
| `String` | Basic string validation | Length constraints |
| `Integer` | Integer validation | Range constraints |
| `Float` | Float validation | Range constraints |
| `Boolean` | Boolean validation | true/false/1/0 |
| `URL` | URL validation | Protocol, format |
| `Email` | Email validation | @ symbol, domain |
| `PrivateKey` | Private key validation | 0x prefix, 66 chars |
| `PublicKey` | Public key validation | 0x prefix, 130 chars |
| `JWT` | JWT token validation | 3 parts separated by dots |

---

## 🌍 **Environment-Specific Configuration**

### **Development Environment**

```rust
// Development configuration
let dev_config = {
    "database": {
        "host": "localhost",
        "port": 5432,
        "ssl_mode": "disable"
    },
    "api": {
        "timeout": 5000,
        "retry_count": 1
    },
    "logging": {
        "level": "debug",
        "output": "console"
    },
    "security": {
        "encryption_enabled": false,
        "rate_limiting": false
    }
};
```

### **Production Environment**

```rust
// Production configuration
let prod_config = {
    "database": {
        "host": "prod-db.example.com",
        "port": 5432,
        "ssl_mode": "require"
    },
    "api": {
        "timeout": 30000,
        "retry_count": 3
    },
    "logging": {
        "level": "warn",
        "output": "syslog"
    },
    "security": {
        "encryption_enabled": true,
        "rate_limiting": true,
        "audit_logging": true
    }
};
```

---

## 🔒 **Security Best Practices**

### **1. Encryption Key Management**

```rust
// Use strong master encryption key
let master_key = config::get_required_env("MASTER_ENCRYPTION_KEY")?;

// Validate key strength
if master_key.as_string()?.length() < 32 {
    return Err(Error::new("WeakEncryptionKey", "Master key must be at least 32 characters"));
}
```

### **2. Environment Variable Security**

```rust
// Never log sensitive values
log::info("config", {
    "database_configured": true,
    "host": db_config["host"],
    // Don't log: "password": db_config["password"]
});

// Use environment-specific validation
if config::get_env_or_default("DIST_AGENT_ENV", Value::String("development".to_string())).as_string()? == "production" {
    // Stricter validation for production
    validate_production_config()?;
}
```

### **3. Secret Access Logging**

```rust
// Log secret access for audit trails
self.access_log.push({
    "secret_name": secret_name,
    "accessed_at": chain::get_block_timestamp(),
    "access_count": secret_record.access_count,
    "user_id": current_user_id // If available
});
```

### **4. Configuration Validation**

```rust
// Validate all configuration on startup
let validation_service = ConfigurationValidationService::new();
validation_service.initialize()?;

let validation_results = validation_service.validate_configuration()?;

if validation_results.size() == 0 {
    log::error("config", {
        "message": "Configuration validation failed",
        "errors": validation_results
    });
    return Err(Error::new("ConfigurationError", "Invalid configuration"));
}
```

---

## 📋 **Configuration Examples**

### **Complete Service Configuration**

```rust
@trust("hybrid")
@secure
service SecureService {
    config_manager: any,
    
    fn initialize() -> Result<Unit, Error> {
        // Initialize configuration manager
        self.config_manager = config::ConfigManager::new();
        
        // Load and validate configuration
        self.load_configuration()?;
        self.validate_configuration()?;
        
        // Setup encrypted secrets
        self.setup_secrets()?;
        
        return Ok(());
    }
    
    fn load_configuration() -> Result<Unit, Error> {
        // Database configuration
        let db_config = config::get_database_config()?;
        self.database = database::connect("postgresql", db_config);
        
        // API configuration
        let api_config = config::get_api_config()?;
        self.api_client = web::create_http_client(api_config);
        
        // Blockchain configuration
        let blockchain_config = config::get_blockchain_config()?;
        self.blockchain = chain::connect(blockchain_config);
        
        return Ok(());
    }
    
    fn setup_secrets() -> Result<Unit, Error> {
        let master_key = config::get_required_env("MASTER_ENCRYPTION_KEY")?;
        
        // Store API keys encrypted
        if let Ok(stripe_key) = config::get_env("STRIPE_SECRET_KEY") {
            self.config_manager.store_secret("stripe", stripe_key.as_string()?, master_key.as_string()?)?;
        }
        
        if let Ok(openai_key) = config::get_env("OPENAI_API_KEY") {
            self.config_manager.store_secret("openai", openai_key.as_string()?, master_key.as_string()?)?;
        }
        
        return Ok(());
    }
}
```

### **Environment-Specific Setup**

```rust
// .env.development
DIST_AGENT_ENV=development
DB_HOST=localhost
DB_NAME=myapp_dev
DB_USER=dev_user
DB_PASSWORD=dev_password
API_BASE_URL=http://localhost:3000
BLOCKCHAIN_RPC_URL=https://goerli.infura.io/v3/YOUR_PROJECT_ID
BLOCKCHAIN_CHAIN_ID=5
BLOCKCHAIN_PRIVATE_KEY=0x1234...
MASTER_ENCRYPTION_KEY=dev_master_key_32_chars_minimum

// .env.production
DIST_AGENT_ENV=production
DB_HOST=prod-db.example.com
DB_NAME=myapp_prod
DB_USER=prod_user
DB_PASSWORD=prod_secure_password
API_BASE_URL=https://api.example.com
BLOCKCHAIN_RPC_URL=https://mainnet.infura.io/v3/YOUR_PROJECT_ID
BLOCKCHAIN_CHAIN_ID=1
BLOCKCHAIN_PRIVATE_KEY=0x5678...
MASTER_ENCRYPTION_KEY=prod_master_key_very_secure_64_chars
```

---

## 🛠️ **Advanced Features**

### **1. Configuration Caching**

```rust
// Cache frequently accessed configuration
let cached_config = config_manager.config_cache.get("database");
if cached_config {
    return Ok(cached_config);
}

// Load and cache configuration
let db_config = config::get_database_config()?;
config_manager.config_cache.insert("database".to_string(), Value::Map(db_config));
```

### **2. Configuration Hot Reloading**

```rust
// Monitor configuration changes
fn watch_configuration_changes() -> Result<Unit, Error> {
    let config_watcher = file_system::watch_file(".env");
    
    config_watcher.on_change(|event| {
        if event.file_path.ends_with(".env") {
            reload_configuration()?;
        }
    });
    
    return Ok(());
}
```

### **3. Configuration Export/Import**

```rust
// Export configuration for backup
fn export_configuration() -> Result<String, Error> {
    let config_data = {
        "environment": config::get_env_or_default("DIST_AGENT_ENV", Value::String("development".to_string())),
        "database": config::get_database_config()?,
        "api": config::get_api_config()?,
        "blockchain": config::get_blockchain_config()?,
        "ai": config::get_ai_config()?
    };
    
    return Ok(json::stringify(config_data));
}

// Import configuration from backup
fn import_configuration(config_json: String) -> Result<Unit, Error> {
    let config_data = json::parse(config_json)?;
    
    // Apply imported configuration
    config::export_to_env(&config_data)?;
    
    return Ok(());
}
```

---

## 🔍 **Troubleshooting**

### **Common Issues**

1. **Environment Variable Not Found**
   ```rust
   // Error: Environment variable 'DB_HOST' not found
   // Solution: Set the required environment variable
   export DB_HOST=localhost
   ```

2. **Invalid Configuration Type**
   ```rust
   // Error: Value 'abc' is not a valid integer
   // Solution: Ensure environment variable contains valid integer
   export DB_PORT=5432  // Not: export DB_PORT=abc
   ```

3. **Weak Encryption Key**
   ```rust
   // Error: Master key must be at least 32 characters
   // Solution: Use a strong encryption key
   export MASTER_ENCRYPTION_KEY=very_long_secure_key_at_least_32_characters
   ```

4. **Invalid Private Key Format**
   ```rust
   // Error: Value '123' is not a valid private key
   // Solution: Use proper private key format
   export BLOCKCHAIN_PRIVATE_KEY=0x1234567890abcdef...
   ```

### **Debug Configuration**

```rust
// Enable debug logging for configuration
log::set_level("debug");

// Log configuration loading process
log::debug("config", {
    "loading_database_config": true,
    "environment": config::get_env_or_default("DIST_AGENT_ENV", Value::String("development".to_string()))
});
```

---

## 📚 **API Reference**

### **Core Functions**

| Function | Description | Returns |
|----------|-------------|---------|
| `config::get_env(key)` | Get environment variable | `Result<Value, String>` |
| `config::get_env_or_default(key, default)` | Get env var with default | `Value` |
| `config::get_required_env(key)` | Get required env var | `Result<Value, String>` |
| `config::get_database_config()` | Get database config | `Result<HashMap<String, Value>, String>` |
| `config::get_api_config()` | Get API config | `Result<HashMap<String, Value>, String>` |
| `config::get_blockchain_config()` | Get blockchain config | `Result<HashMap<String, Value>, String>` |
| `config::get_ai_config()` | Get AI config | `Result<HashMap<String, Value>, String>` |

### **ConfigManager Methods**

| Method | Description | Returns |
|--------|-------------|---------|
| `store_secret(key, value, encryption_key)` | Store encrypted secret | `Result<(), String>` |
| `get_secret(key, encryption_key)` | Retrieve decrypted secret | `Result<String, String>` |
| `validate_value(value, rule)` | Validate configuration value | `Result<(), String>` |
| `load_from_file(path)` | Load config from file | `Result<HashMap<String, Value>, String>` |
| `export_to_env(config)` | Export config to env vars | `Result<(), String>` |

---

**The configuration management system in `dist_agent_lang` provides enterprise-grade security and flexibility for managing application configuration across different environments.** 🔐✨

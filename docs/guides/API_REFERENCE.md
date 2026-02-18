# 📚 API Reference - Standard Library (v1.0.5)

> **📢 Beta Release v1.0.5:** Actively maintained with consistent updates. Test thoroughly before production use. **Beta testing contributions appreciated!** 🙏

Complete reference for all **standard library modules** in dist_agent_lang.

---

## 📋 Table of Contents

### Core Modules
1. [chain:: - Blockchain Operations](#1-chain-blockchain-operations)
2. [oracle:: - External Data Integration](#2-oracle-external-data-integration)
3. [service:: - Service Integration](#3-service-service-integration)
4. [auth:: - Authentication](#4-auth-authentication)
5. [crypto:: - Cryptography](#5-crypto-cryptography)

### Advanced Features
6. [ai:: - AI/ML Integration](#6-ai-aiml-integration)
7. [database:: - Database Operations](#7-database-database-operations)
8. [web:: - HTTP Server](#8-web-http-server)
9. [agent:: - Agent Orchestration](#9-agent-agent-orchestration)
10. [mold:: - Agent Molds & Templates](#10-mold-agent-molds--templates)

### Security & Trust
11. [cloudadmin:: - CloudAdmin Security](#11-cloudadmin-cloudadmin-security)
12. [admin:: - Process Management](#12-admin-process-management)
13. [trust:: - Trust & Permissions](#13-trust-trust--permissions)
14. [key:: - Capability-Based Access Control](#14-key-capability-based-access-control)
15. [kyc:: - Know Your Customer](#15-kyc-know-your-customer)
16. [aml:: - Anti-Money Laundering](#16-aml-anti-money-laundering)

### Infrastructure
17. [iot:: - IoT Device Integration](#17-iot-iot-device-integration)
18. [mobile:: - Mobile Integration](#18-mobile-mobile-integration)
19. [sync:: - Data Synchronization](#19-sync-data-synchronization)
20. [config:: - Configuration](#20-config-configuration)

### Utility Modules
21. [log:: - Logging](#21-log-logging)
22. [test:: - Testing Framework](#22-test-testing-framework)

---

## 1. chain:: - Blockchain Operations

Core blockchain interaction functions for deploying contracts, making calls, and managing assets.

### Functions

#### `chain::deploy(chain_id: int, contract_name: string, constructor_args: map<string, string>) -> string`
Deploy a smart contract to the blockchain.

**Parameters:**
- `chain_id`: Blockchain chain ID (1=Ethereum, 137=Polygon, 42161=Arbitrum, etc.)
- `contract_name`: Name of the contract to deploy
- `constructor_args`: Map of constructor argument names to values

**Returns:** Contract address as string

```dal
let args = {
    "name": "MyToken",
    "symbol": "MTK"
};
let address = chain::deploy(1, "TokenContract", args);
log::info("chain", "Contract deployed at: " + address);
```

---

#### `chain::call(chain_id: int, contract_address: string, function_name: string, args: string) -> any`
Call a function on a deployed contract.

**Parameters:**
- `chain_id`: Blockchain chain ID
- `contract_address`: Address of the deployed contract
- `function_name`: Name of the function to call
- `args`: JSON string of function arguments

**Returns:** Function call result

```dal
let args_json = json::stringify({"to": "0xabc...", "amount": 100});
let result = chain::call(1, "0x742d35...", "transfer", args_json);
```

---

#### `chain::get_balance(chain_id: int, address: string) -> int`
Get the native token balance of an address.

**Parameters:**
- `chain_id`: Blockchain chain ID
- `address`: Wallet address to query

**Returns:** Balance in wei (smallest denomination)

```dal
let balance = chain::get_balance(1, "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0");
log::info("chain", "Balance: " + balance.to_string());
```

---

#### `chain::get_gas_price(chain_id: int) -> float`
Get current gas price for a chain.

**Returns:** Gas price in gwei

```dal
let gas_price = chain::get_gas_price(1);
log::info("chain", "Gas price: " + gas_price.to_string() + " gwei");
```

---

#### `chain::estimate_gas(chain_id: int, operation: string) -> int`
Estimate gas cost for an operation.

**Returns:** Estimated gas units

```dal
let estimated_gas = chain::estimate_gas(1, "transfer");
log::info("chain", "Estimated gas: " + estimated_gas.to_string());
```

---

#### `chain::get_transaction_status(chain_id: int, tx_hash: string) -> string`
Get status of a transaction.

**Returns:** Status string ("pending", "confirmed", "failed")

```dal
let status = chain::get_transaction_status(1, "0x123...");
if status == "confirmed" {
    log::info("chain", "Transaction confirmed");
}
```

---

#### `chain::get_block_timestamp(chain_id: int) -> int`
Get latest block timestamp.

**Returns:** Unix timestamp

```dal
let timestamp = chain::get_block_timestamp(1);
log::info("chain", "Block timestamp: " + timestamp.to_string());
```

---

#### `chain::get_chain_config(chain_id: int) -> map<string, any>`
Get blockchain configuration.

**Returns:** Configuration map with chain details

```dal
let config = chain::get_chain_config(1);
log::info("chain", "Chain name: " + config.get("name", ""));
```

---

#### `chain::get_supported_chains() -> list<map<string, any>>`
Get list of supported blockchains.

**Returns:** List of chain configurations

```dal
let chains = chain::get_supported_chains();
for chain in chains {
    log::info("chain", "Supported chain: " + chain.get("name", ""));
}
```

---

## 2. oracle:: - External Data Integration

Secure oracle integration with multi-source validation.

### Types

```dal
// OracleSource structure
{
    name: string,
    url: string,
    api_key: string?,
    rate_limit: int?,
    trusted: bool,
    public_key: string?
}

// OracleQuery structure
{
    query_type: string,
    parameters: map<string, any>,
    timeout: int?,
    require_signature: bool,
    min_confirmations: int?
}

// OracleResponse structure
{
    data: any,
    timestamp: int,
    source: string,
    signature: string?,
    verified: bool,
    confidence_score: float
}
```

### Functions

#### `oracle::create_source(name: string, url: string) -> OracleSource`
Create a new oracle source.

```dal
let chainlink = oracle::create_source("chainlink", "https://api.chainlink.org");
```

---

#### `oracle::create_query(query_type: string) -> OracleQuery`
Create a new oracle query.

```dal
let query = oracle::create_query("BTC/USD");
```

---

#### `oracle::fetch(source: string, query: OracleQuery) -> Result<OracleResponse, string>`
Fetch data from an oracle source.

```dal
let query = oracle::create_query("ETH/USD");
let result = oracle::fetch("chainlink", query);

if result.is_ok() {
    let response = result.unwrap();
    if response.verified {
        let price = response.data;
        log::info("oracle", "ETH price: " + price.to_string());
    }
}
```

---

#### `oracle::fetch_with_consensus(sources: list<string>, query: OracleQuery, threshold: float) -> Result<OracleResponse, string>`
Fetch data from multiple sources and require consensus.

**Parameters:**
- `sources`: List of oracle source names
- `query`: Oracle query
- `threshold`: Minimum agreement percentage (0.0 to 1.0)

**Returns:** OracleResponse with consensus data

```dal
let query = oracle::create_query("ETH/USD");
let result = oracle::fetch_with_consensus(
    ["chainlink", "uniswap", "band"],
    query,
    0.66  // 66% must agree
);

if result.is_ok() {
    let response = result.unwrap();
    log::info("oracle", "Consensus price: " + response.data.to_string());
    log::info("oracle", "Confidence: " + response.confidence_score.to_string());
}
```

---

#### `oracle::verify(data: any, signature: string) -> bool`
Verify oracle data signature.

```dal
let is_valid = oracle::verify(price_data, signature);
if is_valid {
    log::info("oracle", "Data signature verified");
}
```

---

#### `oracle::stream(source: string, callback: string) -> Result<string, string>`
Stream real-time data from an oracle.

```dal
let stream_result = oracle::stream("chainlink", "onPriceUpdate");
if stream_result.is_ok() {
    let stream_id = stream_result.unwrap();
    log::info("oracle", "Stream started: " + stream_id);
}
```

---

## 3. service:: - Service Integration

HTTP service integration and external API calls.

### Functions

#### `service::http_get(url: string) -> Result<string, string>`
Make an HTTP GET request.

```dal
let result = service::http_get("https://api.example.com/data");
if result.is_ok() {
    let response = result.unwrap();
    log::info("service", "Response: " + response);
}
```

---

#### `service::http_post(url: string, body: string) -> Result<string, string>`
Make an HTTP POST request.

```dal
let body = json::stringify({"key": "value"});
let result = service::http_post("https://api.example.com/submit", body);
if result.is_ok() {
    let response = result.unwrap();
    log::info("service", "Response: " + response);
}
```

---

#### `service::call(service_name: string, method: string, args: list<any>) -> any`
Call an external service.

```dal
let result = service::call(
    "payment_processor",
    "processPayment",
    [1000, "USD"]
);
```

---

## 4. auth:: - Authentication

Secure authentication and access control.

### Functions

#### `auth::create_user(username: string, password: string, email: string, roles: list<string>) -> Result<string, string>`
Create a new user account.

**Returns:** User ID on success

```dal
let result = auth::create_user(
    "alice",
    "secure_password",
    "alice@example.com",
    ["user", "trader"]
);

if result.is_ok() {
    let user_id = result.unwrap();
    log::info("auth", "User created: " + user_id);
}
```

---

#### `auth::authenticate(username: string, password: string) -> Result<Session, string>`
Authenticate user and create secure session.

**Returns:** Session object on success

```dal
let result = auth::authenticate("alice", "secure_password");
if result.is_ok() {
    let session = result.unwrap();
    log::info("auth", "Session created: " + session.id);
    log::info("auth", "User ID: " + session.user_id);
    log::info("auth", "Roles: " + json::stringify(session.roles));
}
```

---

#### `auth::validate_token(token: string) -> Option<Session>`
Validate a session token.

**Returns:** Session object if valid, null otherwise

```dal
let session = auth::validate_token(session_token);
if session != null {
    log::info("auth", "Session valid for user: " + session.user_id);
} else {
    log::warn("auth", "Invalid session token");
}
```

---

#### `auth::has_permission(session: Session, permission: string) -> bool`
Check if session has a specific permission.

```dal
let session = auth::validate_token(token);
if session != null {
    if auth::has_permission(session, "write") {
        // User has write permission
    }
}
```

---

#### `auth::has_role(session: Session, role: string) -> bool`
Check if session has a specific role.

```dal
let session = auth::validate_token(token);
if session != null {
    if auth::has_role(session, "admin") {
        // User has admin role
    }
}
```

---

## 5. crypto:: - Cryptography

Cryptographic operations including hashing, signing, and encryption.

### Functions

#### `crypto::hash(data: string, algorithm: string) -> string`
Hash data using specified algorithm.

**Supported algorithms:** "sha256", "keccak256", "md5"

```dal
let sha256_hash = crypto::hash(data, "sha256");
let keccak256_hash = crypto::hash(data, "keccak256");
```

---

#### `crypto::sign(data: string, private_key: string, algorithm: string) -> string`
Sign data with a private key.

**Supported algorithms:** "ecdsa", "eddsa"

```dal
let signature = crypto::sign(message, private_key, "ecdsa");
```

---

#### `crypto::verify(data: string, signature: string, public_key: string, algorithm: string) -> bool`
Verify a signature.

```dal
let is_valid = crypto::verify(message, signature, public_key, "ecdsa");
if is_valid {
    log::info("crypto", "Signature verified");
}
```

---

#### `crypto::generate_keypair(algorithm: string) -> map<string, string>`
Generate a cryptographic keypair.

**Returns:** Map with "private_key" and "public_key"

```dal
let keypair = crypto::generate_keypair("ecdsa");
let private_key = keypair.get("private_key", "");
let public_key = keypair.get("public_key", "");
```

---

#### `crypto::encrypt(data: string, public_key: string) -> string`
Encrypt data with a public key.

```dal
let encrypted = crypto::encrypt("sensitive data", public_key);
```

---

#### `crypto::decrypt(encrypted_data: string, private_key: string) -> Option<string>`
Decrypt data with a private key.

```dal
let decrypted = crypto::decrypt(encrypted, private_key);
if decrypted != null {
    log::info("crypto", "Decrypted: " + decrypted);
}
```

---

#### `crypto::encrypt_aes256(data: string, key: string) -> Result<string, string>`
Encrypt data using AES-256.

```dal
let result = crypto::encrypt_aes256("data", "encryption_key");
if result.is_ok() {
    let encrypted = result.unwrap();
    log::info("crypto", "Encrypted: " + encrypted);
}
```

---

#### `crypto::decrypt_aes256(encrypted_data: string, key: string) -> Result<string, string>`
Decrypt data using AES-256.

```dal
let result = crypto::decrypt_aes256(encrypted, "encryption_key");
if result.is_ok() {
    let decrypted = result.unwrap();
    log::info("crypto", "Decrypted: " + decrypted);
}
```

---

## 6. ai:: - AI/ML Integration

AI agent framework for building intelligent, autonomous agents.

### Agent Lifecycle

#### `ai::spawn_agent(config: map<string, any>) -> Result<Agent, string>`
Create and spawn a new AI agent.

**Config fields:**
- `agent_id`: Unique agent identifier
- `name`: Agent name
- `role`: Agent role (e.g., "market_analyzer", "trading_agent")
- `capabilities`: List of capabilities
- `memory_size`: Memory size in bytes
- `max_concurrent_tasks`: Maximum concurrent tasks
- `trust_level`: Trust level ("high", "medium", "low")
- `communication_protocols`: List of protocols
- `ai_models`: List of AI models to use

```dal
let config = {
    "agent_id": "trader_001",
    "name": "Trading Agent",
    "role": "market_analyzer",
    "capabilities": ["text_analysis", "trading"],
    "memory_size": 1024,
    "max_concurrent_tasks": 5,
    "trust_level": "high",
    "communication_protocols": ["secure"],
    "ai_models": ["sentiment", "predictor"]
};

let result = ai::spawn_agent(config);
if result.is_ok() {
    let agent = result.unwrap();
    log::info("ai", "Agent created: " + agent.id);
}
```

---

#### `ai::get_agent_status(agent: Agent) -> string`
Get the current status of an agent.

**Returns:** "idle", "active", "busy", "error", or "terminated"

```dal
let status = ai::get_agent_status(agent);
log::info("ai", "Agent status: " + status);
```

---

#### `ai::terminate_agent(agent: Agent) -> Result<bool, string>`
Terminate an agent and clean up resources.

```dal
let result = ai::terminate_agent(agent);
if result.is_ok() {
    log::info("ai", "Agent terminated");
}
```

---

### AI Processing Functions

#### `ai::analyze_text(text: string) -> Result<TextAnalysis, string>`
Analyze text and extract insights.

**TextAnalysis includes:**
- `sentiment`: float (0.0 to 1.0)
- `entities`: list of Entity objects
- `keywords`: list of strings
- `summary`: string
- `language`: string
- `confidence`: float

```dal
let result = ai::analyze_text("This is a great product!");
if result.is_ok() {
    let analysis = result.unwrap();
    log::info("ai", "Sentiment: " + analysis.sentiment.to_string());
    log::info("ai", "Confidence: " + analysis.confidence.to_string());
}
```

---

#### `ai::generate_text(prompt: string) -> Result<string, string>`
Generate text based on a prompt.

```dal
let result = ai::generate_text("Analyze this market data: " + data);
if result.is_ok() {
    let response = result.unwrap();
    log::info("ai", "Generated: " + response);
}
```

---

#### `ai::classify(model: string, input: string) -> Result<string, string>`
Classify text using a named model (simplified API).

```dal
let result = ai::classify("sentiment_model", "This is great!");
if result.is_ok() {
    let classification = result.unwrap();
    log::info("ai", "Classification: " + classification);
}
```

---

#### `ai::generate(model: string, prompt: string) -> Result<string, string>`
Generate text using a specific model (simplified API).

```dal
let result = ai::generate("gpt-4", "Write a summary");
if result.is_ok() {
    let text = result.unwrap();
    log::info("ai", "Generated: " + text);
}
```

---

#### `ai::embed(text: string) -> Result<list<float>, string>`
Generate embeddings for text.

```dal
let result = ai::embed("sample text");
if result.is_ok() {
    let embeddings = result.unwrap();
    log::info("ai", "Embeddings length: " + embeddings.len().to_string());
}
```

---

#### `ai::analyze_image(image_data: list<int>) -> Result<ImageAnalysis, string>`
Analyze image and detect objects, faces, text.

**ImageAnalysis includes:**
- `objects`: list of DetectedObject
- `faces`: list of Face
- `text`: list of strings
- `colors`: list of strings
- `quality_score`: float

```dal
let result = ai::analyze_image(image_bytes);
if result.is_ok() {
    let analysis = result.unwrap();
    log::info("ai", "Quality score: " + analysis.quality_score.to_string());
}
```

---

#### `ai::detect_anomaly(data: list<float>, new_value: float) -> Result<bool, string>`
Detect anomalies in data.

```dal
let historical_data = [1.0, 2.0, 3.0, 4.0, 5.0];
let result = ai::detect_anomaly(historical_data, 100.0);
if result.is_ok() && result.unwrap() {
    log::warn("ai", "Anomaly detected!");
}
```

---

### Task Management

#### `ai::create_task(agent: Agent, task_type: string, description: string, params: map<string, any>) -> Result<Task, string>`
Create a task for an agent.

```dal
let params = {
    "text": "Analyze this data"
};
let result = ai::create_task(agent, "text_analysis", "Analyze data", params);
if result.is_ok() {
    let task = result.unwrap();
    log::info("ai", "Task created: " + task.id);
}
```

---

#### `ai::execute_task(agent: Agent, task_id: string) -> Result<any, string>`
Execute a task and return results.

```dal
let result = ai::execute_task(agent, task_id);
if result.is_ok() {
    let task_result = result.unwrap();
    log::info("ai", "Task completed: " + json::stringify(task_result));
}
```

---

### Message Passing

#### `ai::send_message(from_agent: string, to_agent: string, message_type: string, content: any, priority: string) -> Result<Message, string>`
Send a message between agents.

**Priority:** "low", "normal", "high", "urgent"

```dal
let result = ai::send_message(
    "agent_1",
    "agent_2",
    "task_assignment",
    "Process this data",
    "high"
);
if result.is_ok() {
    let message = result.unwrap();
    log::info("ai", "Message sent: " + message.id);
}
```

---

#### `ai::process_message_queue(agent: Agent) -> Result<list<any>, string>`
Process all queued messages for an agent.

```dal
let result = ai::process_message_queue(agent);
if result.is_ok() {
    let results = result.unwrap();
    log::info("ai", "Processed " + results.len().to_string() + " messages");
}
```

---

### Multi-Agent Coordination

#### `ai::create_coordinator(coordinator_id: string) -> AgentCoordinator`
Create a coordinator for managing multiple agents.

```dal
let coordinator = ai::create_coordinator("main_coordinator");
log::info("ai", "Coordinator created: " + coordinator.coordinator_id);
```

---

#### `ai::add_agent_to_coordinator(coordinator: AgentCoordinator, agent: Agent)`
Add an agent to a coordinator.

```dal
ai::add_agent_to_coordinator(coordinator, agent);
log::info("ai", "Agent added to coordinator");
```

---

#### `ai::create_workflow(coordinator: AgentCoordinator, name: string, steps: list<WorkflowStep>) -> Result<Workflow, string>`
Create a multi-agent workflow.

```dal
let steps = [
    {
        "step_id": "step_1",
        "agent_id": "analyst_agent",
        "task_type": "analysis",
        "dependencies": []
    }
];
let result = ai::create_workflow(coordinator, "Analysis Workflow", steps);
if result.is_ok() {
    let workflow = result.unwrap();
    log::info("ai", "Workflow created: " + workflow.workflow_id);
}
```

---

#### `ai::execute_workflow(coordinator: AgentCoordinator, workflow_id: string) -> Result<bool, string>`
Execute a workflow.

```dal
let result = ai::execute_workflow(coordinator, workflow_id);
if result.is_ok() && result.unwrap() {
    log::info("ai", "Workflow executed successfully");
}
```

---

## 7. database:: - Database Operations

Database integration for off-chain storage.

### Functions

#### `database::connect(connection_string: string) -> Result<DbConnection, string>`
Connect to a database.

```dal
let result = database::connect("postgresql://user:pass@localhost/db");
if result.is_ok() {
    let db = result.unwrap();
    log::info("database", "Connected to database");
}
```

---

#### `database::query(conn: DbConnection, sql: string, params: list<any>) -> Result<list<map<string, any>>, string>`
Execute a SQL query.

```dal
let result = database::query(
    db,
    "SELECT * FROM users WHERE age > $1",
    [18]
);
if result.is_ok() {
    let rows = result.unwrap();
    log::info("database", "Found " + rows.len().to_string() + " rows");
}
```

---

#### `database::execute(conn: DbConnection, sql: string, params: list<any>) -> Result<int, string>`
Execute a SQL statement (INSERT/UPDATE/DELETE).

**Returns:** Number of rows affected

```dal
let result = database::execute(
    db,
    "INSERT INTO users (name, email) VALUES ($1, $2)",
    ["Alice", "alice@example.com"]
);
if result.is_ok() {
    let rows_affected = result.unwrap();
    log::info("database", "Inserted " + rows_affected.to_string() + " rows");
}
```

---

## 8. web:: - HTTP Server

Built-in HTTP server for APIs.

### Functions

#### `web::create_server(port: int) -> HttpServer`
Create a new HTTP server.

```dal
let server = web::create_server(8080);
log::info("web", "Server created on port 8080");
```

---

#### `web::route(server: HttpServer, path: string, handler: function)`
Add a route to the server.

```dal
web::route(server, "/api/health", fn(req, res) {
    res.send(json::stringify({"status": "healthy"}));
});
```

---

#### `web::start(server: HttpServer)`
Start the HTTP server.

```dal
web::start(server);
log::info("web", "Server listening on port 8080");
```

---

## 9. agent:: - Agent Orchestration

Agent lifecycle and coordination functions.

### Functions

#### `agent::create(config: map<string, any>) -> Agent`
Create a new agent.

```dal
let config = {
    "name": "MyAgent",
    "type": "worker",
    "capabilities": ["process", "analyze"]
};
let agent = agent::create(config);
log::info("agent", "Agent created: " + agent.id);
```

---

#### `agent::coordinate(agents: list<Agent>, task: string) -> Result<any, string>`
Coordinate multiple agents to complete a task.

```dal
let result = agent::coordinate([agent1, agent2], "process_data");
if result.is_ok() {
    let result_data = result.unwrap();
    log::info("agent", "Task completed");
}
```

---

## 10. mold:: - Agent Molds & Templates

Load and spawn agents from mold configurations.

### Functions

#### `mold::list() -> list<string>`
List local mold file paths.

```dal
let paths = mold::list();
for path in paths {
    log::info("mold", "Found mold: " + path);
}
```

---

#### `mold::load(path_or_name: string) -> Result<map<string, any>, string>`
Load a mold configuration by path, name, or IPFS CID.

```dal
let result = mold::load("verify_mold");
if result.is_ok() {
    let config = result.unwrap();
    log::info("mold", "Mold loaded: " + config.get("name", ""));
}
```

---

#### `mold::spawn_from(path_or_name: string, agent_name: string) -> Result<string, string>`
Spawn an agent from a mold; returns the new agent ID.

```dal
let result = mold::spawn_from("verify_mold", "MyAgent");
if result.is_ok() {
    let agent_id = result.unwrap();
    log::info("mold", "Agent spawned: " + agent_id);
}
```

---

## 11. cloudadmin:: - CloudAdmin Security

Hybrid trust and administrative control. See [CloudAdmin Guide](CLOUDADMIN_GUIDE.md) for complete documentation.

### Functions

#### `cloudadmin::authorize(admin_id: string, operation: string, resource: string) -> bool`
Check if admin is authorized for operation on resource.

**Operations:** "read" (all), "write" (admin+), "delete" (superadmin only)

```dal
let can_write = cloudadmin::authorize("admin_001", "write", "/data/config");
if can_write {
    // Execute write operation
}
```

---

#### `cloudadmin::enforce_policy(policy_name: string, context: AdminContext) -> Result<bool, string>`
Enforce admin policy based on context.

**Policies:** "strict" (superadmin), "moderate" (admin+), "permissive" (all)

```dal
let context = cloudadmin::create_admin_context("admin_001", "admin");
if context != null {
    let result = cloudadmin::enforce_policy("moderate", context);
    if result.is_ok() && result.unwrap() {
        // Execute operation
    }
}
```

---

#### `cloudadmin::validate_hybrid_trust(admin_trust: string, user_trust: string) -> bool`
Validate hybrid trust between admin and user. Both must be "valid".

```dal
let is_trusted = cloudadmin::validate_hybrid_trust("valid", "valid");
if is_trusted {
    // Both trusts are valid
}
```

---

#### `cloudadmin::bridge_trusts(centralized_trust: string, decentralized_trust: string) -> bool`
Bridge centralized admin trust with decentralized user trust.

**Requirements:** centralized = "admin" AND decentralized = "user"

```dal
let can_bridge = cloudadmin::bridge_trusts("admin", "user");
if can_bridge {
    // Trusts are bridged
}
```

---

#### `cloudadmin::create_admin_context(admin_id: string, level: string) -> Option<AdminContext>`
Create admin context with specified level.

**Levels:** "superadmin", "admin", "moderator", "user"

```dal
let context = cloudadmin::create_admin_context("admin_001", "admin");
if context != null {
    // Use context
}
```

---

## 12. admin:: - Process Management

Administrative control over system processes. See [CloudAdmin Guide](CLOUDADMIN_GUIDE.md) for complete documentation.

### Functions

#### `admin::kill(process_id: string, reason: string) -> Result<bool, string>`
Terminate process or agent. Reason is required.

```dal
let result = admin::kill("agent_123", "resource_violation");
if result.is_ok() {
    log::info("admin", "Process terminated");
}
```

---

#### `admin::get_process_info(process_id: string) -> Result<ProcessInfo, string>`
Get detailed process information.

```dal
let result = admin::get_process_info("agent_123");
if result.is_ok() {
    let info = result.unwrap();
    log::info("admin", "Process: " + info.name);
    log::info("admin", "Status: " + info.status);
}
```

---

#### `admin::list_processes() -> list<ProcessInfo>`
List all running processes.

```dal
let processes = admin::list_processes();
for process in processes {
    log::info("admin", "Process: " + process.process_id);
}
```

---

## 13. trust:: - Trust & Permissions

Trust and permission management functions.

### Functions

#### `trust::authorize(admin_id: string, operation: string, resource: string) -> bool`
Authorize admin operation (delegates to key registry and admin registry).

```dal
let authorized = trust::authorize("admin_001", "read", "/data");
if authorized {
    // Access granted
}
```

---

#### `trust::enforce_policy(policy_name: string, context: AdminContext) -> Result<bool, string>`
Enforce trust policy.

```dal
let context = trust::create_admin_context("admin_001", "admin");
if context != null {
    let result = trust::enforce_policy("moderate", context);
    if result.is_ok() && result.unwrap() {
        // Policy enforced
    }
}
```

---

#### `trust::register_admin(admin_id: string, level: AdminLevel, permissions: list<string>)`
Register an admin in the registry.

```dal
trust::register_admin("admin_001", "admin", ["read", "write"]);
```

---

## 14. key:: - Capability-Based Access Control

Capability-based access control using keys to resources.

### Functions

#### `key::create_capability_request(resource: string, operation: string, principal: string) -> CapabilityRequest`
Create a capability request.

```dal
let request = key::create_capability_request("/data", "read", "user_001");
```

---

#### `key::check(request: CapabilityRequest) -> Result<bool, string>`
Check if a capability is granted.

```dal
let request = key::create_capability_request("/data", "read", "user_001");
let result = key::check(request);
if result.is_ok() && result.unwrap() {
    // Access granted
}
```

---

## 15. kyc:: - Know Your Customer

KYC verification functions.

### Functions

#### `kyc::verify(user_id: string, documents: list<any>) -> Result<bool, string>`
Verify KYC for a user.

```dal
let result = kyc::verify("user_001", documents);
if result.is_ok() && result.unwrap() {
    log::info("kyc", "KYC verified");
}
```

---

#### `kyc::get_verification(user_id: string) -> Option<map<string, any>>`
Get KYC verification status.

```dal
let verification = kyc::get_verification("user_001");
if verification != null {
    log::info("kyc", "Verification status: " + verification.get("status", ""));
}
```

---

## 16. aml:: - Anti-Money Laundering

AML compliance functions.

### Functions

#### `aml::perform_check(address: string) -> Result<bool, string>`
Perform AML check on an address.

```dal
let result = aml::perform_check("0x742d35...");
if result.is_ok() && result.unwrap() {
    log::info("aml", "AML check passed");
}
```

---

#### `aml::get_status(address: string) -> string`
Get AML status for an address.

```dal
let status = aml::get_status("0x742d35...");
log::info("aml", "AML status: " + status);
```

---

## 17. iot:: - IoT Device Integration

IoT device management functions.

### Functions

#### `iot::send_command(device_id: string, command: string, params: map<string, any>) -> Result<bool, string>`
Send command to IoT device.

```dal
let params = {"duration": 30};
let result = iot::send_command("device_12345", "unlock", params);
if result.is_ok() && result.unwrap() {
    log::info("iot", "Command sent");
}
```

---

## 18. mobile:: - Mobile Integration

Mobile app integration functions.

### Functions

#### `mobile::push_notification(token: string, title: string, body: string) -> Result<bool, string>`
Send push notification.

```dal
let result = mobile::push_notification(
    device_token,
    "Transaction Confirmed",
    "Your transfer was successful"
);
if result.is_ok() && result.unwrap() {
    log::info("mobile", "Notification sent");
}
```

---

## 19. sync:: - Data Synchronization

Data synchronization functions.

### Functions

#### `sync::sync_data(source: string, destination: string, options: map<string, any>) -> Result<bool, string>`
Synchronize data between sources.

```dal
let options = {"mode": "bidirectional"};
let result = sync::sync_data("source_db", "dest_db", options);
if result.is_ok() && result.unwrap() {
    log::info("sync", "Data synchronized");
}
```

---

## 20. config:: - Configuration

Configuration management functions.

### Functions

#### `config::get_env(key: string) -> Option<string>`
Get environment variable.

```dal
let api_key = config::get_env("API_KEY");
if api_key != null {
    log::info("config", "API key found");
}
```

---

#### `config::get_database_config() -> map<string, any>`
Get database configuration.

```dal
let db_config = config::get_database_config();
let host = db_config.get("host", "");
log::info("config", "Database host: " + host);
```

---

## 21. log:: - Logging

Logging functions for debugging and audit trails.

### Functions

#### `log::info(message: string, data: map<string, any>, source: string?)`
Log an info message.

```dal
log::info("Operation completed", {"operation": "transfer", "amount": 100}, "service");
```

---

#### `log::warning(message: string, data: map<string, any>, source: string?)`
Log a warning.

```dal
log::warning("Low balance detected", {"balance": 50}, "wallet");
```

---

#### `log::error(message: string, data: map<string, any>, source: string?)`
Log an error.

```dal
log::error("Transaction failed", {"error": "insufficient_funds"}, "chain");
```

---

#### `log::audit(event: string, data: map<string, any>, source: string?)`
Log an audit event.

```dal
log::audit("user_login", {"user_id": "user_001"}, "auth");
```

---

#### `log::debug(message: string, data: map<string, any>, source: string?)`
Log a debug message.

```dal
log::debug("Processing request", {"request_id": "req_001"}, "api");
```

---

#### `log::get_entries() -> list<LogEntry>`
Get all log entries.

```dal
let entries = log::get_entries();
log::info("log", "Total entries: " + entries.len().to_string());
```

---

#### `log::get_entries_by_level(level: string) -> list<LogEntry>`
Get log entries by level.

```dal
let errors = log::get_entries_by_level("error");
log::info("log", "Error count: " + errors.len().to_string());
```

---

#### `log::clear()`
Clear all log entries.

```dal
log::clear();
log::info("log", "Logs cleared");
```

---

## 22. test:: - Testing Framework

Testing framework for DAL code.

### Functions

#### `test::run(test_suite: TestSuite) -> TestResult`
Run a test suite.

```dal
let suite = test::create_suite("MyTests");
let result = test::run(suite);
log::info("test", "Tests passed: " + result.passed.to_string());
```

---

## 📖 Usage Examples

See complete examples in:
- [Quick Start](QUICK_START.md) - Basic usage
- [AI Features Guide](AI_FEATURES_GUIDE.md) - AI capabilities
- [CloudAdmin Guide](CLOUDADMIN_GUIDE.md) - Security and admin
- [Best Practices](BEST_PRACTICES.md) - Production patterns

---

## 📚 Related Documentation

- [Standard Library Reference](../STDLIB_REFERENCE.md) - Machine-readable API reference
- [Syntax Reference](../syntax.md) - DAL language syntax
- [Attributes Reference](../attributes.md) - Service and function attributes

---

**Version**: v1.0.5 (Beta Release)  
**Last Updated**: 2026-02-17

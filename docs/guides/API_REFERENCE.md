# 📚 API Reference - Standard Library (v1.0.1)

> **📢 Beta Release v1.0.1:** Actively maintained with consistent updates. Test thoroughly before production use. **Beta testing contributions appreciated!** 🙏

Complete reference for all **25 standard library modules** in dist_agent_lang.

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
9. [ipfs:: - IPFS Storage](#9-ipfs-ipfs-storage)
10. [social:: - Social Platform Integration](#10-social-social-platform-integration)

### Specialized Modules
11. [rwa:: - Real World Assets](#11-rwa-real-world-assets)
12. [defi:: - DeFi Protocols](#12-defi-defi-protocols)
13. [nft:: - NFT Standards](#13-nft-nft-standards)
14. [governance:: - DAO Governance](#14-governance-dao-governance)
15. [compliance:: - KYC/AML](#15-compliance-kycaml)

### Utility Modules
16. [log:: - Logging](#16-log-logging)
17. [time:: - Time Operations](#17-time-time-operations)
18. [math:: - Mathematics](#18-math-mathematics)
19. [string:: - String Operations](#19-string-string-operations)
20. [json:: - JSON Processing](#20-json-json-processing)

### Administrative & Security
21. [cloudadmin:: - CloudAdmin Security](#21-cloudadmin-cloudadmin-security)
22. [admin:: - Process Management](#22-admin-process-management)

### Infrastructure
23. [mobile:: - Mobile Integration](#23-mobile-mobile-integration)
24. [iot:: - IoT Device Integration](#24-iot-iot-device-integration)
25. [mold:: - Agent Molds & Templates](#25-mold-agent-molds--templates)

---

## 1. chain:: - Blockchain Operations

Core blockchain interaction functions.

### Functions

#### `chain::id() -> int`
Get the current blockchain ID.

```dal
let chainId = chain::id();
// 1 = Ethereum Mainnet
// 137 = Polygon
// 42161 = Arbitrum
```

#### `chain::block_number() -> int`
Get the current block number.

```dal
let currentBlock = chain::block_number();
```

#### `chain::timestamp() -> int`
Get the current block timestamp.

```dal
let now = chain::timestamp();
```

#### `chain::balance(address: string) -> int`
Get the native token balance of an address.

```dal
let balance = chain::balance("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0");
```

#### `chain::call(contract: string, method: string, args: array) -> any`
Make a cross-contract call.

```dal
let result = chain::call(
    "0xTokenAddress",
    "balanceOf",
    ["0xUserAddress"]
);
```

#### `chain::deploy(bytecode: string, args: array) -> string`
Deploy a contract and return its address.

```dal
let contractAddress = chain::deploy(
    compiledBytecode,
    ["Constructor", "Args"]
);
```

---

## 2. oracle:: - External Data Integration

Secure oracle integration with multi-source validation.

### Types

```dal
struct OracleSource {
    name: string,
    url: string,
    api_key: string?,
    rate_limit: int?,
    trusted: bool,
    public_key: string?
}

struct OracleQuery {
    query_type: string,
    parameters: map<string, any>,
    timeout: int?,
    require_signature: bool,
    min_confirmations: int?
}

struct OracleResponse {
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
let chainlink = oracle::create_source(
    "chainlink",
    "https://api.chainlink.org"
)
.with_api_key(apiKey)
.with_rate_limit(100)  // 100 requests/second
.with_trust(publicKey);  // Mark as trusted
```

#### `oracle::create_query(query_type: string) -> OracleQuery`
Create a new oracle query.

```dal
let query = oracle::create_query("BTC/USD")
    .with_parameter("pair", "BTC/USD")
    .with_timeout(5000)  // 5 seconds
    .require_signature(true)
    .with_confirmations(2);  // Require 2 sources
```

#### `oracle::fetch(source: string, query: OracleQuery) -> OracleResponse`
Fetch data from an oracle source.

```dal
let price = oracle::fetch("chainlink", query);
if (price.verified) {
    btcPrice = price.data;
}
```

#### `oracle::fetch_with_consensus(sources: array, query: OracleQuery, threshold: float) -> OracleResponse`
Fetch data from multiple sources and require consensus.

```dal
let price = oracle::fetch_with_consensus(
    ["chainlink", "uniswap", "band"],
    oracle::create_query("ETH/USD"),
    0.66  // 66% must agree
);

// price.confidence_score = actual agreement percentage
```

#### `oracle::verify(data: any, signature: string) -> bool`
Verify oracle data signature.

```dal
let isValid = oracle::verify(priceData, signature);
```

#### `oracle::stream(source: string, callback: string) -> string`
Stream real-time data from an oracle.

```dal
let streamId = oracle::stream("chainlink", "onPriceUpdate");
```

---

## 3. service:: - Service Integration

HTTP service integration and external API calls.

### Functions

#### `service::http_get(url: string) -> string`
Make an HTTP GET request.

```dal
let response = service::http_get("https://api.example.com/data");
```

#### `service::http_post(url: string, body: string) -> string`
Make an HTTP POST request.

```dal
let response = service::http_post(
    "https://api.example.com/submit",
    json::stringify({"key": "value"})
);
```

#### `service::call(service_name: string, method: string, args: array) -> any`
Call an external service.

```dal
let result = service::call(
    "payment_processor",
    "processPayment",
    [amount, currency]
);
```

---

## 4. auth:: - Authentication

Secure authentication and access control.

### Functions

#### `auth::hash_password(password: string) -> string`
Hash a password securely (bcrypt).

```dal
let hashedPassword = auth::hash_password(userPassword);
// Store hashedPassword in your system
```

#### `auth::verify_password(password: string, hash: string) -> bool`
Verify a password against a hash.

```dal
let isValid = auth::verify_password(inputPassword, storedHash);
```

#### `auth::create_session(user_id: string, ttl: int) -> string`
Create a new user session.

```dal
let sessionToken = auth::create_session(userId, 3600);  // 1 hour TTL
```

#### `auth::verify_session(token: string) -> bool`
Verify a session token.

```dal
if (auth::verify_session(sessionToken)) {
    // Session is valid
}
```

#### `auth::revoke_session(token: string)`
Revoke a session.

```dal
auth::revoke_session(sessionToken);
```

#### `auth::sign_message(message: string, private_key: string) -> string`
Sign a message with a private key.

```dal
let signature = auth::sign_message(message, privateKey);
```

#### `auth::verify_signature(message: string, signature: string, address: string) -> bool`
Verify a message signature.

```dal
let isValid = auth::verify_signature(message, signature, signerAddress);
```

---

## 5. crypto:: - Cryptography

Cryptographic operations.

### Functions

#### `crypto::hash(data: string, algorithm: string) -> string`
Hash data using specified algorithm.

```dal
let sha256Hash = crypto::hash(data, "sha256");
let keccak256Hash = crypto::hash(data, "keccak256");
let md5Hash = crypto::hash(data, "md5");
```

#### `crypto::verify_ecdsa(message: string, signature: string, public_key: string) -> bool`
Verify ECDSA signature.

```dal
let isValid = crypto::verify_ecdsa(message, signature, publicKey);
```

#### `crypto::verify_eddsa(message: string, signature: string, public_key: string) -> bool`
Verify EdDSA signature.

```dal
let isValid = crypto::verify_eddsa(message, signature, publicKey);
```

#### `crypto::random_bytes(length: int) -> string`
Generate cryptographically secure random bytes.

```dal
let randomBytes = crypto::random_bytes(32);  // 32 bytes
```

---

## 6. ai:: - AI/ML Integration

AI agent framework for building intelligent, autonomous agents.

### Agent Lifecycle

#### `ai::spawn_agent(config: AgentConfig) -> Agent`
Create and spawn a new AI agent.

```dal
let config = AgentConfig {
    agent_id: "trader_001",
    name: "Trading Agent",
    role: "market_analyzer",
    capabilities: vec!["text_analysis", "trading"],
    memory_size: 1024,
    max_concurrent_tasks: 5,
    trust_level: "high",
    communication_protocols: vec!["secure"],
    ai_models: vec!["sentiment", "predictor"]
};

let agent = ai::spawn_agent(config);
```

#### `ai::terminate_agent(agent: &mut Agent) -> bool`
Terminate an agent and clean up resources.

```dal
ai::terminate_agent(&mut agent);
```

#### `ai::get_agent_status(agent: &Agent) -> string`
Get the current status of an agent.

```dal
let status = ai::get_agent_status(&agent);
// Returns: "idle", "active", "busy", "error", or "terminated"
```

### AI Processing Functions

#### `ai::analyze_text(text: string) -> TextAnalysis`
Analyze text and extract insights.

```dal
let analysis = ai::analyze_text(userComment);
// TextAnalysis includes: sentiment (f64), entities, keywords, summary, language, confidence
```

#### `ai::analyze_image(imageData: Vec<u8>) -> ImageAnalysis`
Analyze image and detect objects, faces, text.

```dal
let analysis = ai::analyze_image(imageBytes);
// ImageAnalysis includes: objects, faces, text, colors, quality_score
```

#### `ai::generate_text(prompt: string) -> string`
Generate text based on a prompt.

```dal
let response = ai::generate_text("Analyze this market data: " + data);
```

#### `ai::train_model(data: TrainingData) -> Model`
Train a custom AI model.

```dal
let model = ai::train_model(trainingData);
```

#### `ai::predict(model: &Model, input: Value) -> Prediction`
Make predictions using a trained model.

```dal
let prediction = ai::predict(&model, inputData);
// Prediction includes: prediction, confidence, probabilities, explanation
```

### Task Management

#### `ai::create_task(agent: &mut Agent, taskType: string, description: string, params: HashMap<String, Value>) -> Task`
Create a task for an agent.

```dal
let mut params = HashMap::new();
params.insert("text", Value::String(data));

let task = ai::create_task(&mut agent, "text_analysis", "Analyze data", params);
```

#### `ai::execute_task(agent: &mut Agent, taskId: string) -> Value`
Execute a task and return results.

```dal
let result = ai::execute_task(&mut agent, &task.id);
```

### Message Passing

#### `ai::send_message(from: string, to: string, messageType: string, content: Value, priority: MessagePriority) -> Message`
Send a message between agents.

```dal
let message = ai::send_message(
    "agent_1",
    "agent_2",
    "task_assignment",
    Value::String(taskData),
    MessagePriority::High
);
```

#### `ai::receive_message(agent: &mut Agent, message: Message) -> Result<(), String>`
Agent receives a message.

```dal
ai::receive_message(&mut agent, message);
```

#### `ai::process_message_queue(agent: &mut Agent) -> Vec<Value>`
Process all queued messages.

```dal
let results = ai::process_message_queue(&mut agent);
```

### Multi-Agent Coordination

#### `ai::create_coordinator(id: string) -> AgentCoordinator`
Create a coordinator for managing multiple agents.

```dal
let coordinator = ai::create_coordinator("main_coordinator");
```

#### `ai::add_agent_to_coordinator(coordinator: &mut AgentCoordinator, agent: Agent)`
Add an agent to a coordinator.

```dal
ai::add_agent_to_coordinator(&mut coordinator, agent);
```

#### `ai::create_workflow(coordinator: &mut AgentCoordinator, name: string, steps: Vec<WorkflowStep>) -> Workflow`
Create a multi-agent workflow.

```dal
let workflow = ai::create_workflow(&mut coordinator, "Analysis Workflow", steps);
```

#### `ai::execute_workflow(coordinator: &mut AgentCoordinator, workflowId: string) -> bool`
Execute a workflow.

```dal
ai::execute_workflow(&mut coordinator, &workflow.workflow_id);
```

---

## 7. database:: - Database Operations

Database integration for off-chain storage.

### Functions

#### `database::connect(connection_string: string) -> DbConnection`
Connect to a database.

```dal
let db = database::connect("postgresql://user:pass@localhost/db");
```

#### `database::query(conn: DbConnection, sql: string, params: array) -> array`
Execute a SQL query.

```dal
let results = database::query(
    db,
    "SELECT * FROM users WHERE age > $1",
    [18]
);
```

#### `database::execute(conn: DbConnection, sql: string, params: array) -> int`
Execute a SQL statement (INSERT/UPDATE/DELETE).

```dal
let rowsAffected = database::execute(
    db,
    "INSERT INTO users (name, email) VALUES ($1, $2)",
    ["Alice", "alice@example.com"]
);
```

---

## 8. web:: - HTTP Server

Built-in HTTP server for APIs.

### Types

```dal
struct HttpServer {
    port: int,
    routes: map<string, HttpRoute>,
    middleware: array<Middleware>,
    static_files: map<string, string>,
    config: ServerConfig
}

struct ServerConfig {
    max_connections: int,
    timeout_seconds: int,
    cors_enabled: bool,
    ssl_enabled: bool,
    static_path: string
}
```

### Functions

#### `web::create_server(port: int) -> HttpServer`
Create a new HTTP server.

```dal
let server = web::create_server(8080);
```

#### `web::route(server: HttpServer, path: string, handler: function)`
Add a route to the server.

```dal
web::route(server, "/api/health", fn(req, res) {
    res.send(json::stringify({"status": "healthy"}));
});
```

#### `web::middleware(server: HttpServer, middleware: function)`
Add middleware to the server.

```dal
web::middleware(server, fn(req, res, next) {
    log::info("Request: " + req.path);
    next();
});
```

#### `web::start(server: HttpServer)`
Start the HTTP server.

```dal
web::start(server);
// Server listening on port 8080
```

---

## 9. ipfs:: - IPFS Storage

Decentralized file storage via IPFS.

### Functions

#### `ipfs::upload(data: string) -> string`
Upload data to IPFS and return CID.

```dal
let cid = ipfs::upload(fileData);
// Returns: "QmX7gZk..."
```

#### `ipfs::download(cid: string) -> string`
Download data from IPFS.

```dal
let fileData = ipfs::download(cid);
```

#### `ipfs::pin(cid: string)`
Pin a CID to ensure persistence.

```dal
ipfs::pin(cid);
```

---

## 10. social:: - Social Platform Integration

Integration with social platforms.

### Functions

#### `social::post_twitter(message: string) -> string`
Post to Twitter.

```dal
let tweetId = social::post_twitter("Hello from DAL!");
```

#### `social::post_discord(webhook: string, message: string)`
Post to Discord.

```dal
social::post_discord(webhookUrl, "Deployment successful!");
```

---

## 11. rwa:: - Real World Assets

Tokenization of real-world assets.

### Functions

#### `rwa::tokenize(asset_id: string, value: int, metadata: map) -> string`
Tokenize a real-world asset.

```dal
let tokenId = rwa::tokenize(
    "PROPERTY-123",
    1000000,  // $1M value
    {
        "type": "real_estate",
        "location": "NYC",
        "size": "2500 sqft"
    }
);
```

#### `rwa::verify_asset(token_id: string) -> bool`
Verify an RWA token's authenticity.

```dal
let isVerified = rwa::verify_asset(tokenId);
```

---

## 12. defi:: - DeFi Protocols

DeFi protocol integration.

### Functions

#### `defi::swap(from_token: string, to_token: string, amount: int, dex: string) -> int`
Swap tokens on a DEX.

```dal
let receivedAmount = defi::swap(
    usdcAddress,
    wethAddress,
    1000,  // 1000 USDC
    "uniswap_v3"
);
```

#### `defi::add_liquidity(token_a: string, token_b: string, amount_a: int, amount_b: int) -> int`
Add liquidity to a pool.

```dal
let lpTokens = defi::add_liquidity(
    daiAddress,
    usdcAddress,
    1000,  // 1000 DAI
    1000   // 1000 USDC
);
```

---

## 13. nft:: - NFT Standards

NFT operations.

### Functions

#### `nft::mint(to: string, token_id: int, metadata_uri: string)`
Mint a new NFT.

```dal
nft::mint(
    recipientAddress,
    tokenId,
    "ipfs://QmX7gZk..."
);
```

#### `nft::transfer(from: string, to: string, token_id: int)`
Transfer an NFT.

```dal
nft::transfer(ownerAddress, recipientAddress, tokenId);
```

#### `nft::burn(token_id: int)`
Burn an NFT.

```dal
nft::burn(tokenId);
```

---

## 14. governance:: - DAO Governance

DAO governance operations.

### Functions

#### `governance::create_proposal(title: string, description: string, actions: array) -> int`
Create a governance proposal.

```dal
let proposalId = governance::create_proposal(
    "Increase Treasury Allocation",
    "Proposal to increase treasury allocation by 10%",
    [
        {"target": treasuryAddress, "function": "increaseAllocation", "args": [10]}
    ]
);
```

#### `governance::vote(proposal_id: int, support: bool, voting_power: int)`
Vote on a proposal.

```dal
governance::vote(proposalId, true, votingPower);
```

#### `governance::execute(proposal_id: int)`
Execute a passed proposal.

```dal
governance::execute(proposalId);
```

---

## 15. compliance:: - KYC/AML

Compliance and regulatory functions.

### Functions

#### `compliance::verify_kyc(user_id: string, documents: array) -> bool`
Verify KYC for a user.

```dal
let isVerified = compliance::verify_kyc(userId, kycDocuments);
```

#### `compliance::check_sanctions(address: string) -> bool`
Check if an address is sanctioned.

```dal
let isSanctioned = compliance::check_sanctions(userAddress);
if (isSanctioned) {
    revert("Address is sanctioned");
}
```

---

## 16. log:: - Logging

Logging functions.

### Functions

#### `log::info(message: string)`
Log an info message.

```dal
log::info("Contract deployed successfully");
```

#### `log::warn(message: string)`
Log a warning.

```dal
log::warn("Low balance detected");
```

#### `log::error(message: string)`
Log an error.

```dal
log::error("Transaction failed: " + errorMessage);
```

---

## 17. time:: - Time Operations

Time-related functions.

### Functions

#### `time::now() -> int`
Get current timestamp.

```dal
let timestamp = time::now();
```

#### `time::parse(date_string: string, format: string) -> int`
Parse a date string to timestamp.

```dal
let timestamp = time::parse("2024-12-30", "YYYY-MM-DD");
```

#### `time::format(timestamp: int, format: string) -> string`
Format a timestamp to string.

```dal
let dateString = time::format(timestamp, "YYYY-MM-DD HH:mm:ss");
```

---

## 18. math:: - Mathematics

Mathematical operations.

### Functions

#### `math::sqrt(x: int) -> int`
Square root.

```dal
let result = math::sqrt(16);  // 4
```

#### `math::pow(base: int, exponent: int) -> int`
Power function.

```dal
let result = math::pow(2, 10);  // 1024
```

#### `math::min(a: int, b: int) -> int` / `math::max(a: int, b: int) -> int`
Minimum and maximum.

```dal
let minimum = math::min(10, 20);  // 10
let maximum = math::max(10, 20);  // 20
```

---

## 19. string:: - String Operations

String manipulation functions.

### Functions

#### `string::concat(strings: array<string>) -> string`
Concatenate strings.

```dal
let result = string::concat(["Hello", " ", "World"]);  // "Hello World"
```

#### `string::split(text: string, delimiter: string) -> array<string>`
Split a string.

```dal
let parts = string::split("a,b,c", ",");  // ["a", "b", "c"]
```

#### `string::to_upper(text: string) -> string` / `string::to_lower(text: string) -> string`
Change case.

```dal
let upper = string::to_upper("hello");  // "HELLO"
let lower = string::to_lower("WORLD");  // "world"
```

---

## 20. json:: - JSON Processing

JSON serialization and parsing.

### Functions

#### `json::stringify(data: any) -> string`
Convert data to JSON string.

```dal
let jsonString = json::stringify({"key": "value", "number": 42});
```

#### `json::parse(json_string: string) -> any`
Parse JSON string to data.

```dal
let data = json::parse('{"key":"value"}');
```

---

## 21. cloudadmin:: - CloudAdmin Security

Hybrid trust and administrative control for applications combining centralized admin oversight with decentralized user operations.

### Core Concepts

- **Hybrid Trust**: Bridge centralized admin control with decentralized user trust
- **Multi-Level Administration**: 4-tier admin hierarchy (SuperAdmin, Admin, Moderator, User)
- **Policy Enforcement**: Flexible security policies (strict, moderate, permissive)
- **Trust Bridging**: Connect centralized and decentralized trust models

### Functions

#### `cloudadmin::authorize(admin_id: string, operation: string, resource: string) -> bool`
Check if admin is authorized for operation on resource.

**Operations:**
- `"read"` - All admin levels
- `"write"` - Admin, SuperAdmin only
- `"delete"` - SuperAdmin only

```dal
let can_write = cloudadmin::authorize("admin_001", "write", "/data/config");
let can_delete = cloudadmin::authorize("superadmin_001", "delete", "/data/users");
```

---

#### `cloudadmin::enforce_policy(policy_name: string, context: AdminContext) -> Result<bool, string>`
Enforce admin policy based on context.

**Policies:**
- `"strict"` - SuperAdmin only
- `"moderate"` - Admin or SuperAdmin
- `"permissive"` - All users

```dal
let context = cloudadmin::create_admin_context("admin_001", "admin");
let allowed = cloudadmin::enforce_policy("moderate", context);

if allowed.is_ok() && allowed.unwrap() {
    // Execute operation
}
```

---

#### `cloudadmin::validate_hybrid_trust(admin_trust: string, user_trust: string) -> bool`
Validate hybrid trust between admin and user. Both must be "valid" for success.

```dal
let admin_verified = cloudadmin::authorize(admin_id, "write", resource);
let admin_trust = if admin_verified { "valid" } else { "invalid" };

let user_verified = auth::verify_signature();
let user_trust = if user_verified { "valid" } else { "invalid" };

let is_trusted = cloudadmin::validate_hybrid_trust(admin_trust, user_trust);
```

---

#### `cloudadmin::bridge_trusts(centralized_trust: string, decentralized_trust: string) -> bool`
Bridge centralized admin trust with decentralized user trust.

**Requirements:** centralized = "admin" AND decentralized = "user"

```dal
let admin_approved = cloudadmin::authorize(admin_id, "write", "/bridge");
let centralized = if admin_approved { "admin" } else { "none" };

let user_verified = auth::verify_address(user_address);
let decentralized = if user_verified { "user" } else { "none" };

let can_bridge = cloudadmin::bridge_trusts(centralized, decentralized);
```

---

#### `cloudadmin::create_admin_context(admin_id: string, level: string) -> Option<AdminContext>`
Create admin context with specified level.

**Levels:** "superadmin", "admin", "moderator", "user"

```dal
let context = cloudadmin::create_admin_context("admin_001", "admin");

// Add permissions
context = context.with_permissions(["read_users", "write_config"]);

// Add metadata
context = context.with_metadata({
    "department": "engineering",
    "region": "us-west"
});
```

### Use Cases

- **Hybrid Marketplaces**: Admin moderation + decentralized trading
- **Regulated DeFi**: Compliance oversight + user autonomy
- **Multi-Chain Apps**: Centralized coordination + blockchain operations
- **AI Agent Systems**: Admin control + autonomous agents

### Example: Hybrid Approval System

```dal
@trust("hybrid")
@secure
service HybridApproval {
    fn approve_transaction(tx_id: string, admin_id: string, user_sig: string) -> bool {
        // Check admin authorization
        let admin_ok = cloudadmin::authorize(admin_id, "write", "/transactions");
        let admin_trust = if admin_ok { "valid" } else { "invalid" };
        
        // Verify user signature
        let user_ok = auth::verify_signature(user_sig);
        let user_trust = if user_ok { "valid" } else { "invalid" };
        
        // Validate hybrid trust
        let is_trusted = cloudadmin::validate_hybrid_trust(admin_trust, user_trust);
        
        if is_trusted {
            chain::execute_transaction(tx_id);
            return true;
        }
        
        return false;
    }
}
```

**See Also:** [CloudAdmin Guide](CLOUDADMIN_GUIDE.md) | [Hybrid Marketplace Tutorial](tutorials/03_hybrid_marketplace_cloudadmin.md)

---

## 22. admin:: - Process Management

Administrative control over system processes, agents, and resources.

### Functions

#### `admin::kill(process_id: string, reason: string) -> Result<bool, string>`
Terminate process or agent. Reason is required.

**Valid Reasons:**
- `"resource_violation"` - Excessive resource usage
- `"security_breach"` - Security threat detected
- `"policy_violation"` - Policy compliance issue
- `"maintenance"` - Scheduled maintenance

```dal
// Kill agent exceeding resources
let result = admin::kill("agent_123", "resource_violation: CPU > 90%");

if result.is_ok() {
    log::audit("admin", "Agent terminated successfully");
} else {
    log::error("admin", "Failed: " + result.error());
}
```

---

#### `admin::get_process_info(process_id: string) -> Result<ProcessInfo, string>`
Get detailed process information including resource usage.

**Returns:**
- `process_id` - Process identifier
- `name` - Process name
- `status` - Current status (running, stopped, error)
- `start_time` - Unix timestamp
- `resource_usage` - Map of resource metrics (cpu, memory, etc.)

```dal
let info = admin::get_process_info("agent_123");

if info.is_ok() {
    let process = info.unwrap();
    log::info("admin", "Process: " + process.name);
    log::info("admin", "Status: " + process.status);
    log::info("admin", "CPU: " + process.resource_usage["cpu"]);
    log::info("admin", "Memory: " + process.resource_usage["memory"]);
    
    // Check if exceeding limits
    if process.resource_usage["cpu"] > 80 {
        admin::kill(process_id, "excessive_cpu_usage");
    }
}
```

---

#### `admin::list_processes() -> Vec<ProcessInfo>`
List all running processes in the system.

```dal
let processes = admin::list_processes();

for process in processes {
    log::info("admin", "ID: " + process.process_id);
    log::info("admin", "Name: " + process.name);
    log::info("admin", "Status: " + process.status);
    
    // Monitor AI agents
    if process.name.starts_with("ai_agent_") {
        let cpu = process.resource_usage["cpu"].as_int();
        if cpu > 75 {
            log::warn("admin", "AI agent high CPU: " + process.process_id);
        }
    }
}
```

### Use Cases

- **Agent Monitoring**: Track AI agent resource usage
- **Process Lifecycle**: Manage process termination
- **Resource Control**: Enforce resource limits
- **Health Checks**: Monitor system health

### Example: Agent Health Monitor

```dal
@ai
service AgentHealthMonitor {
    fn monitor_agents() {
        let processes = admin::list_processes();
        
        for process in processes {
            // Check if it's an AI agent
            if process.name.starts_with("ai_agent_") {
                let cpu = process.resource_usage["cpu"].as_int();
                let memory = process.resource_usage["memory"].as_int();
                
                // Kill if exceeding limits
                if cpu > 80 || memory > 4096 {
                    let result = admin::kill(
                        process.process_id,
                        "resource_violation: CPU=" + cpu + "%, Memory=" + memory + "MB"
                    );
                    
                    if result.is_ok() {
                        log::audit("monitor", "Terminated agent: " + process.process_id);
                    }
                }
            }
        }
    }
}
```

**See Also:** [CloudAdmin Guide](CLOUDADMIN_GUIDE.md) | [Process Management Best Practices](BEST_PRACTICES.md)

---

## 23. mobile:: - Mobile Integration

Mobile app integration.

### Functions

#### `mobile::push_notification(token: string, title: string, body: string)`
Send push notification.

```dal
mobile::push_notification(
    deviceToken,
    "Transaction Confirmed",
    "Your transfer of 100 tokens was successful"
);
```

---

## 24. iot:: - IoT Device Integration

IoT device integration.

### Functions

#### `iot::send_command(device_id: string, command: string, params: map)`
Send command to IoT device.

```dal
iot::send_command(
    "device_12345",
    "unlock",
    {"duration": 30}
);
```

---

## 25. mold:: - Agent Molds & Templates

Load and spawn agents from mold configurations (local files, IPFS, or on-chain registry).

### Functions

#### `mold::list() -> list<string>`
List local mold file paths in the current directory and standard locations.

```dal
let paths = mold::list();
// Returns e.g. ["verify_mold.mold.json", "traffic_analyst.mold.json"]
```

#### `mold::load(path_or_name: string) -> map`
Load a mold configuration by path, name, or IPFS CID.

```dal
let config = mold::load("verify_mold");
// or mold::load("ipfs://Qm...")
```

#### `mold::spawn_from(path_or_name: string, agent_name: string) -> string`
Spawn an agent from a mold; returns the new agent ID.

```dal
let agent_id = mold::spawn_from("verify_mold", "MyAgent");
```

#### `mold::get_info(mold_id: string) -> map` (requires web3)
Fetch mold metadata from on-chain MoldRegistry.

#### `mold::use_mold(mold_id: string, agent_name: string) -> string` (requires web3)
Pay mint fee, load from IPFS, and spawn agent from on-chain mold.

See [STDLIB_REFERENCE.md](../STDLIB_REFERENCE.md#mold-module) for full details.

---

## 📖 Usage Examples

See complete examples in:
- [Quick Start](QUICK_START.md) - Basic usage
- [Tutorials](tutorials/) - Step-by-step guides
- [Best Practices](BEST_PRACTICES.md) - Production patterns

---

**Next:** [Tutorial Series →](tutorials/)


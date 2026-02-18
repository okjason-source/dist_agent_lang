# DAL Standard Library Reference

**Version:** 1.0.5  
**Last Updated:** 2026-02-06  
**Optimized for:** AI/LLM code generation and human reference

---

## Overview

This document provides a comprehensive, machine-readable reference for all DAL standard library modules.

**Format Convention:**
```
function_name(param1: Type, param2: Type) -> ReturnType
```

---

## Index

- [chain](#chain-module) - Blockchain operations
- [crypto](#crypto-module) - Cryptographic functions
- [auth](#auth-module) - Authentication
- [db](#db-module) - Database operations
- [ai](#ai-module) - AI/ML operations
- [agent](#agent-module) - Agent orchestration
- [mold](#mold-module) - Mold load/spawn (agent templates)
- [iot](#iot-module) - IoT device management
- [oracle](#oracle-module) - Oracle data feeds
- [sync](#sync-module) - Data synchronization
- [web](#web-module) - HTTP operations
- [log](#log-module) - Logging
- [config](#config-module) - Configuration
- [cloudadmin](#cloudadmin-module) - Cloud administration
- [trust](#trust-module) - Trust and permissions
- [key](#key-module) - Capability-based access control (keys to resources)
- [aml](#aml-module) - Anti-money laundering
- [kyc](#kyc-module) - Know Your Customer
- [test](#test-module) - Testing framework

---

## chain Module

Blockchain operations for deploying contracts, making calls, and managing assets.

### Functions

#### deploy
```dal
chain::deploy(contract_name: String, constructor_args: String) -> Result<DeployResult, Error>
```
Deploy a smart contract to the blockchain.

**Parameters:**
- `contract_name`: Name of the contract to deploy
- `constructor_args`: JSON string of constructor arguments

**Returns:** Deployment result with contract address

**Example:**
```dal
let result = chain::deploy("MyToken", "{}");
log::info("Contract deployed at: " + result.address);
```

---

#### call
```dal
chain::call(contract_address: String, function_name: String, args: String) -> Result<CallResult, Error>
```
Call a function on a deployed contract.

**Parameters:**
- `contract_address`: Address of the deployed contract
- `function_name`: Name of the function to call
- `args`: JSON string of function arguments

**Returns:** Function call result

**Example:**
```dal
let result = chain::call("0x742d35...", "transfer", "{\"to\": \"0xabc\", \"amount\": 100}");
```

---

#### get_balance
```dal
chain::get_balance(chain_id: Int, address: String) -> Int
```
Get the native token balance of an address.

**Parameters:**
- `chain_id`: Blockchain chain ID (1=Ethereum, 137=Polygon, etc.)
- `address`: Wallet address to query

**Returns:** Balance in wei (smallest denomination)

**Example:**
```dal
let balance = chain::get_balance(1, "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb");
log::info("Balance: " + balance);
```

---

#### get_gas_price
```dal
chain::get_gas_price(chain_id: Int) -> Int
```
Get current gas price for a chain.

**Returns:** Gas price in gwei

---

#### estimate_gas
```dal
chain::estimate_gas(chain_id: Int, operation: String) -> Int
```
Estimate gas cost for an operation.

**Returns:** Estimated gas units

---

#### get_transaction_status
```dal
chain::get_transaction_status(chain_id: Int, tx_hash: String) -> String
```
Get status of a transaction.

**Returns:** Status string ("pending", "confirmed", "failed")

---

#### get_block_timestamp
```dal
chain::get_block_timestamp(chain_id: Int) -> Int
```
Get latest block timestamp.

**Returns:** Unix timestamp

---

#### mint
```dal
chain::mint(name: String) -> String
```
Mint an on-chain asset.

**Returns:** Asset ID

---

#### get
```dal
chain::get(asset_id: String) -> Map<String, Value>
```
Get asset information.

**Returns:** Asset metadata

---

#### get_chain_config
```dal
chain::get_chain_config(chain_id: Int) -> Map<String, Value>
```
Get blockchain configuration.

**Returns:** Configuration map

---

#### get_supported_chains
```dal
chain::get_supported_chains() -> List<Map<String, Value>>
```
Get list of supported blockchains.

**Returns:** List of chain configs

---

## crypto Module

Cryptographic operations including hashing, signing, and encryption.

### Functions

#### hash
```dal
crypto::hash(data: String, algorithm: String) -> String
```
Hash data using specified algorithm.

**Parameters:**
- `data`: Data to hash
- `algorithm`: "sha256" or "sha512"

**Returns:** Hex-encoded hash

**Example:**
```dal
let hash = crypto::hash("hello world", "sha256");
```

---

#### sign
```dal
crypto::sign(data: String, private_key: String) -> String
```
Sign data with private key.

**Returns:** Signature (hex-encoded)

---

#### verify
```dal
crypto::verify(data: String, signature: String, public_key: String) -> Bool
```
Verify signature.

**Returns:** True if valid

---

#### generate_keypair
```dal
crypto::generate_keypair(algorithm: String) -> Map<String, String>
```
Generate public/private keypair.

**Parameters:**
- `algorithm`: "rsa" or "ed25519"

**Returns:** Map with "public" and "private" keys

---

#### encrypt
```dal
crypto::encrypt(data: String, public_key: String) -> String
```
Encrypt data with public key.

**Returns:** Encrypted data (base64)

---

#### decrypt
```dal
crypto::decrypt(encrypted_data: String, private_key: String) -> String
```
Decrypt data with private key.

**Returns:** Original data

---

#### encrypt_aes256
```dal
crypto::encrypt_aes256(data: String, key: String) -> String
```
Encrypt with AES-256.

**Returns:** Encrypted data

---

#### decrypt_aes256
```dal
crypto::decrypt_aes256(encrypted_data: String, key: String) -> String
```
Decrypt with AES-256.

**Returns:** Decrypted data

---

#### random_hash
```dal
crypto::random_hash(algorithm: String) -> String
```
Generate random hash.

**Returns:** Random hash string

---

## auth Module

Authentication and user management.

### Functions

#### create_user
```dal
auth::create_user(username: String, password: String, email: String) -> String
```
Create a new user.

**Returns:** User ID

**Example:**
```dal
let user_id = auth::create_user("alice", "password123", "alice@example.com");
```

---

#### login
```dal
auth::login(username: String, password: String) -> String
```
Authenticate user.

**Returns:** Session token

---

#### validate_token
```dal
auth::validate_token(token: String) -> Bool
```
Validate session token.

**Returns:** True if valid

---

#### session
```dal
auth::session() -> Map<String, Value>
auth::session(user_id: String) -> Map<String, Value>
```
Get current session information.

**Returns:** Session map with user_id, timestamp, etc.

---

#### init_auth_system
```dal
auth::init_auth_system() -> Bool
```
Initialize authentication system.

**Returns:** Success status

---

## db Module

Database operations for querying, migrations, and management.

### Functions

#### connect
```dal
db::connect(connection_string: String) -> String
```
Connect to database.

**Returns:** Connection ID

**Example:**
```dal
let conn = db::connect("postgresql://localhost:5432/mydb");
```

---

#### query
```dal
db::query(conn: String, sql: String) -> List<Map<String, Value>>
```
Execute SQL query.

**Returns:** Result rows

**Example:**
```dal
let users = db::query(conn, "SELECT * FROM users");
```

---

#### execute
```dal
db::execute(conn: String, sql: String) -> Int
```
Execute SQL statement (INSERT, UPDATE, DELETE).

**Returns:** Number of affected rows

---

#### list_tables
```dal
db::list_tables(conn: String) -> List<String>
```
List all tables.

**Returns:** Table names

---

#### get_table_schema
```dal
db::get_table_schema(conn: String, table_name: String) -> Map<String, Value>
```
Get table schema.

**Returns:** Schema information

---

#### get_query_plan
```dal
db::get_query_plan(conn: String, sql: String) -> String
```
Get query execution plan.

**Returns:** Query plan

---

#### backup_database
```dal
db::backup_database(conn: String, backup_path: String) -> Bool
```
Backup database.

**Returns:** Success status

---

#### restore_database
```dal
db::restore_database(conn: String, backup_path: String) -> Bool
```
Restore from backup.

**Returns:** Success status

---

#### apply_migration
```dal
db::apply_migration(conn: String, migration_sql: String) -> Bool
```
Apply database migration.

**Returns:** Success status

---

#### rollback_migration
```dal
db::rollback_migration(conn: String, version: String) -> Bool
```
Rollback migration.

**Returns:** Success status

---

#### ping_database
```dal
db::ping_database(conn: String) -> Bool
```
Check database connectivity.

**Returns:** True if connected

---

#### get_database_metrics
```dal
db::get_database_metrics(conn: String) -> Map<String, Value>
```
Get database metrics.

**Returns:** Metrics map (connections, queries/sec, etc.)

---

## agent Module

AI agent orchestration, coordination, and communication.

### Functions

#### spawn
```dal
agent::spawn(config: Map<String, Value>) -> AgentContext
```
Create a new agent.

**Parameters:**
- `config`: Agent configuration map with:
  - `name`: Agent name
  - `type`: "ai", "system", "worker", or "custom:<name>"
  - `role`: Agent role description
  - `capabilities`: List of capabilities
  - `trust_level`: "standard", "high", or "critical"

**Returns:** Agent context with agent_id

**Example:**
```dal
let agent = agent::spawn({
    "name": "FraudDetector",
    "type": "ai",
    "role": "Detect fraudulent transactions",
    "capabilities": ["fraud_detection", "pattern_analysis"],
    "trust_level": "high"
});
log::info("Agent created: " + agent.agent_id);
```

---

#### coordinate
```dal
agent::coordinate(agent_id: String, task: AgentTask, coordination_type: String) -> Bool
```
Coordinate agent activities.

**Parameters:**
- `agent_id`: Target agent ID
- `task`: Task object
- `coordination_type`: "task_distribution", "resource_sharing", or "conflict_resolution"

**Returns:** Success status

---

#### communicate
```dal
agent::communicate(sender_id: String, receiver_id: String, message: AgentMessage) -> Bool
```
Send message between agents.

**Returns:** Success status

**Example:**
```dal
let msg = agent::create_agent_message(
    "msg_001",
    sender_id,
    receiver_id,
    "task_assignment",
    "Process batch 42"
);
agent::communicate(sender_id, receiver_id, msg);
```

---

#### evolve
```dal
agent::evolve(agent_id: String, evolution_data: Map<String, Value>) -> Bool
```
Update agent through learning.

**Returns:** Success status

---

#### validate_capabilities
```dal
agent::validate_capabilities(agent_type: String, required_capabilities: List<String>) -> Bool
```
Validate agent has required capabilities.

**Returns:** True if capable

---

#### create_agent_config
```dal
agent::create_agent_config(name: String, agent_type: String, role: String) -> AgentConfig
```
Create agent configuration.

**Returns:** Config object

---

#### create_agent_task
```dal
agent::create_agent_task(task_id: String, description: String, priority: String) -> AgentTask
```
Create agent task.

**Parameters:**
- `priority`: "low", "medium", "high", or "critical"

**Returns:** Task object

---

#### create_agent_message
```dal
agent::create_agent_message(
    message_id: String,
    sender_id: String,
    receiver_id: String,
    message_type: String,
    content: Value
) -> AgentMessage
```
Create agent message.

**Returns:** Message object

---

#### receive_pending_tasks
```dal
agent::receive_pending_tasks(agent_id: String) -> List<AgentTask>
```
Get pending tasks for agent.

**Returns:** List of tasks

---

#### receive_messages
```dal
agent::receive_messages(agent_id: String) -> List<AgentMessage>
```
Get messages for agent.

**Returns:** List of messages

---

## mold Module

Load and spawn agents from mold files (reusable agent templates). Supports local paths, IPFS, and on-chain molds.

### Functions

#### load
```dal
mold::load(source: String) -> Map<String, Value>
```
Load mold config from path, name, or `ipfs://cid`. Returns config map with name, version, agent {...}.

**Parameters:**
- `source`: Path, mold name (e.g. `"verify_mold"`), or `ipfs://Qm...`

**Returns:** Config map (name, version, agent {...})

**Example:**
```dal
let config = mold::load("verify_mold");
log::info("Mold: " + config.name + " v" + config.version);
```

---

#### spawn_from
```dal
mold::spawn_from(source: String, name_override?: String) -> String
```
Load mold and spawn agent. Returns agent_id.

**Parameters:**
- `source`: Path, mold name, or `ipfs://cid`
- `name_override`: Optional agent name (else uses mold name)

**Returns:** agent_id

**Example:**
```dal
let agent_id = mold::spawn_from("verify_mold", "MyAgent");
```

---

#### list
```dal
mold::list() -> List<String>
```
List local mold file paths (base, mold/, mold/samples).

**Returns:** List of path strings

**Example:**
```dal
let paths = mold::list();
```

---

#### get_info
```dal
mold::get_info(mold_id: Int) -> Map<String, Value>
```
Get on-chain mold info (creator, ipfs_hash, mint_fee, mint_count, etc.). Requires `web3` feature.

**Parameters:**
- `mold_id`: On-chain mold NFT ID

**Returns:** Map with creator, ipfs_hash, mint_fee, mint_count, max_use_count, active, created_at, updated_at

---

#### use_mold
```dal
mold::use_mold(mold_id: Int, name_override?: String) -> String
```
Use on-chain mold: pay mint fee, enforce cap, load from IPFS, spawn agent. Requires `web3` feature and MoldRegistry deployed.

**Parameters:**
- `mold_id`: On-chain mold NFT ID
- `name_override`: Optional agent name

**Returns:** agent_id

**Example:**
```dal
let agent_id = mold::use_mold(123, "OnChainAgent");
```

---

## ai Module

AI and machine learning operations.

### Functions

#### generate_text
```dal
ai::generate_text(prompt: String) -> String
```
Generate text from prompt.

**Returns:** Generated text

---

#### classify
```dal
ai::classify(model: String, input: String) -> String
```
Classify input with model.

**Returns:** Classification result

---

#### embed
```dal
ai::embed(text: String) -> List<Float>
```
Get embedding vector for text.

**Returns:** Embedding vector

---

#### cosine_similarity
```dal
ai::cosine_similarity(vec1: List<Float>, vec2: List<Float>) -> Float
```
Calculate cosine similarity.

**Returns:** Similarity score (-1 to 1)

---

#### analyze_text
```dal
ai::analyze_text(text: String) -> Map<String, Value>
```
Analyze text (sentiment, entities, etc.).

**Returns:** Analysis results

---

#### analyze_image_url
```dal
ai::analyze_image_url(url: String) -> Map<String, Value>
```
Analyze image from URL.

**Returns:** Analysis results

---

## iot Module

IoT device management and sensor operations.

### Functions

#### register_device
```dal
iot::register_device(device_config: Map<String, Value>) -> String
```
Register IoT device.

**Returns:** Device ID

---

#### connect_device
```dal
iot::connect_device(device_id: String) -> Bool
```
Connect to device.

**Returns:** Success status

---

#### get_device_status
```dal
iot::get_device_status(device_id: String) -> String
```
Get device status.

**Returns:** Status ("online", "offline", "error")

---

#### read_sensor_data
```dal
iot::read_sensor_data(sensor_id: String) -> Map<String, Value>
```
Read sensor data.

**Returns:** Sensor readings

---

#### send_actuator_command
```dal
iot::send_actuator_command(actuator_id: String, command: String) -> Bool
```
Send command to actuator.

**Returns:** Success status

---

#### monitor_power_consumption
```dal
iot::monitor_power_consumption(device_id: String) -> Float
```
Monitor device power usage.

**Returns:** Power consumption (watts)

---

#### optimize_power_usage
```dal
iot::optimize_power_usage(device_id: String, target_hours: Int) -> String
```
Optimize power usage.

**Returns:** Optimization result

---

#### detect_sensor_anomalies
```dal
iot::detect_sensor_anomalies(sensor_data: Map<String, Value>) -> String
```
Detect anomalies in sensor data.

**Returns:** Anomaly report

---

#### predict_device_failure
```dal
iot::predict_device_failure(device_id: String, historical_data: Map<String, Value>) -> Float
```
Predict device failure probability.

**Returns:** Failure probability (0-1)

---

## oracle Module

Oracle data feed operations.

### Functions

#### fetch
```dal
oracle::fetch(source: String, query_type: String) -> Map<String, Value>
```
Fetch data from oracle.

**Returns:** Oracle data

---

#### stream
```dal
oracle::stream(source: String, callback: String) -> String
```
Create data stream from oracle.

**Returns:** Stream ID

---

#### get_stream
```dal
oracle::get_stream(stream_id: String) -> Map<String, Value>
```
Get stream status.

**Returns:** Stream info

---

#### close_stream
```dal
oracle::close_stream(stream_id: String) -> Bool
```
Close data stream.

**Returns:** Success status

---

#### create_source
```dal
oracle::create_source(name: String, url: String) -> String
```
Create oracle source.

**Returns:** Source ID

---

#### verify
```dal
oracle::verify(data: Map<String, Value>, signature: String) -> Bool
```
Verify oracle data signature.

**Returns:** True if valid

---

## web Module

HTTP operations for web requests and responses.

### Functions

#### get_request
```dal
web::get_request(url: String) -> String
```
HTTP GET request.

**Returns:** Response body

---

#### post_request
```dal
web::post_request(url: String, data: String) -> String
```
HTTP POST request.

**Returns:** Response body

---

#### parse_url
```dal
web::parse_url(url: String) -> Map<String, String>
```
Parse URL into components.

**Returns:** Map with scheme, host, path, query, etc.

---

#### render_template
```dal
web::render_template(template: String, variables: Map<String, Value>) -> String
```
Render template with variables.

**Returns:** Rendered HTML

---

## log Module

Logging operations.

### Functions

#### info
```dal
log::info(message: String)
log::info(source: String, message: String)
```
Log info message.

---

#### warn
```dal
log::warn(message: String)
log::warn(source: String, message: String)
```
Log warning message.

---

#### error
```dal
log::error(message: String)
log::error(source: String, message: String)
```
Log error message.

---

#### debug
```dal
log::debug(message: String)
log::debug(source: String, message: String)
```
Log debug message.

---

#### audit
```dal
log::audit(event: String, data: Map<String, Value>)
log::audit(source: String, event: String, data: Map<String, Value>)
```
Log audit event.

---

#### get_stats
```dal
log::get_stats() -> Map<String, Int>
```
Get logging statistics.

**Returns:** Stats (total logs, by level, etc.)

---

#### get_entries
```dal
log::get_entries() -> List<Map<String, Value>>
```
Get recent log entries.

**Returns:** Log entries

---

#### get_entries_by_level
```dal
log::get_entries_by_level(level: String) -> List<Map<String, Value>>
```
Get logs by level.

**Returns:** Filtered log entries

---

#### clear
```dal
log::clear()
```
Clear all logs.

---

## config Module

Configuration management.

### Functions

#### get_env
```dal
config::get_env(key: String) -> String
```
Get environment variable.

**Returns:** Value or empty string

---

#### get_database_config
```dal
config::get_database_config() -> Map<String, Value>
```
Get database configuration.

**Returns:** DB config

---

#### get_api_config
```dal
config::get_api_config() -> Map<String, Value>
```
Get API configuration.

**Returns:** API config

---

#### get_blockchain_config
```dal
config::get_blockchain_config() -> Map<String, Value>
```
Get blockchain configuration.

**Returns:** Blockchain config

---

#### get_ai_config
```dal
config::get_ai_config() -> Map<String, Value>
```
Get AI configuration.

**Returns:** AI config

---

## cloudadmin Module

Cloud administration and governance.

### Functions

#### authorize
```dal
cloudadmin::authorize(admin_id: String, operation: String, resource: String) -> Bool
```
Check authorization.

**Returns:** True if authorized

---

#### enforce_policy
```dal
cloudadmin::enforce_policy(policy_name: String, context: AdminContext) -> Bool
```
Enforce admin policy.

**Returns:** True if allowed

---

#### validate_hybrid_trust
```dal
cloudadmin::validate_hybrid_trust(admin_trust: String, user_trust: String) -> Bool
```
Validate hybrid trust.

**Returns:** True if valid

---

#### bridge_trusts
```dal
cloudadmin::bridge_trusts(centralized_trust: String, decentralized_trust: String) -> Bool
```
Bridge trust models.

**Returns:** True if bridged

---

## trust Module

Trust and permission management.

### Functions

#### authorize
```dal
trust::authorize(admin_id: String, operation: String, resource: String) -> Bool
```
Authorize operation.

**Returns:** True if authorized

---

#### enforce_policy
```dal
trust::enforce_policy(policy_name: String, context: AdminContext) -> Bool
```
Enforce policy.

**Returns:** True if allowed

---

#### validate_hybrid_trust
```dal
trust::validate_hybrid_trust(admin_trust: String, user_trust: String) -> Bool
```
Validate hybrid trust.

**Returns:** True if valid

---

#### bridge_trusts
```dal
trust::bridge_trusts(centralized_trust: String, decentralized_trust: String) -> Bool
```
Bridge trust models.

**Returns:** True if bridged

---

#### register_admin
```dal
trust::register_admin(admin_id: String, level: String, permissions: List<String>)
```
Register admin user.

**Parameters:**
- `level`: "user", "moderator", "admin", or "superadmin"

---

## key Module

Capability-based access control (keys to resources).

### Functions

#### create
```dal
key::create(resource: String, permissions: List<String>) -> Capability
```
Create key (capability).

**Returns:** Capability

---

#### check
```dal
key::check(request: CapabilityRequest) -> Bool
```
Check if operation is allowed.

**Returns:** True if allowed

---

#### create_principal
```dal
key::create_principal(principal_id: String, name: String) -> Principal
```
Create security principal.

**Returns:** Principal

---

## aml Module

Anti-money laundering checks.

### Functions

#### perform_check
```dal
aml::perform_check(transaction: Map<String, Value>, address: String) -> String
```
Perform AML check.

**Returns:** Check ID

---

#### get_check_status
```dal
aml::get_check_status(check_id: String) -> String
```
Get check status.

**Returns:** Status ("pending", "passed", "failed")

---

#### list_providers
```dal
aml::list_providers() -> List<String>
```
List AML providers.

**Returns:** Provider names

---

## kyc Module

Know Your Customer verification.

### Functions

#### verify
```dal
kyc::verify(user_id: String, verification_data: Map<String, Value>) -> String
```
Verify user identity.

**Returns:** Verification ID

---

#### get_verification_status
```dal
kyc::get_verification_status(verification_id: String) -> String
```
Get verification status.

**Returns:** Status ("pending", "verified", "rejected")

---

## test Module

Testing framework.

### Functions

#### run_registered_tests
```dal
test::run_registered_tests() -> Map<String, Value>
```
Run all registered tests.

**Returns:** Test results

---

#### get_test_suites
```dal
test::get_test_suites() -> List<String>
```
Get test suite names.

**Returns:** Suite names

---

#### get_results
```dal
test::get_results() -> Map<String, Value>
```
Get last test results.

**Returns:** Results

---

## Type Definitions

### Common Types

```dal
// Agent Types
type AgentContext = Map<String, Value>
type AgentConfig = Map<String, Value>
type AgentTask = Map<String, Value>
type AgentMessage = Map<String, Value>

// Admin Types
type AdminContext = Map<String, Value>

// Capability Types
type CapabilityRequest = Map<String, Value>

// Result Types
type DeployResult = Map<String, Value>  // { address: String, tx_hash: String }
type CallResult = Map<String, Value>    // { success: Bool, result: Value }
```

---

## Error Handling

All functions that can fail return `Result<T, Error>` or throw runtime errors. Use try-catch for error handling:

```dal
try {
    let result = chain::deploy("MyContract", "{}");
    log::info("Success: " + result.address);
} catch error {
    log::error("Failed: " + error);
}
```

---

## Best Practices for AI/LLM Code Generation

1. **Always import modules:**
   ```dal
   import stdlib::chain;
   import stdlib::agent;
   ```

2. **Use try-catch for error handling:**
   ```dal
   try {
       // risky operation
   } catch error {
       log::error("Error: " + error);
   }
   ```

3. **Leverage built-in functions:**
   - Don't implement crypto: use `crypto::hash()`
   - Don't write HTTP clients: use `web::get_request()`
   - Don't implement auth: use `auth::create_user()`

4. **Use services for stateful components:**
   ```dal
   @chain("ethereum")
   service MyContract {
       var state: String = "initial";
       
       function update() {
           state = "updated";
       }
   }
   ```

5. **Follow attribute patterns:**
   ```dal
   @trust("hybrid")
   @chain("ethereum")
   @ai
   service SmartAgent { }
   ```

---

**Document Version:** 1.0  
**Last Updated for AI Training:** 2026-02-06  
**Machine-Readable Format:** Yes  
**Status:** Production Ready

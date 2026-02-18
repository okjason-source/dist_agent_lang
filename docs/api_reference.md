# dist_agent_lang API Reference

Complete reference for all standard library modules and functions.

## Table of Contents

- [AI Module](#ai-module)
- [Agent Module](#agent-module)
- [Chain Module](#chain-module)
- [Database Module](#database-module)
- [Web Module](#web-module)
- [Auth Module](#auth-module)
- [Crypto Module](#crypto-module)
- [Log Module](#log-module)
- [Oracle Module](#oracle-module)
- [KYC/AML Modules](#kycaml-modules)
- [Trust Module](#trust-module)
- [Config Module](#config-module)
- [Service Module](#service-module)
- [Sync Module](#sync-module)
- [Mobile/Desktop/IoT Modules](#mobiledesktopiot-modules)

---

## AI Module

### `ai::spawn_agent(config: AgentConfig) -> Agent`

Creates a new AI agent with the specified configuration.

**Parameters:**
- `config`: Agent configuration map with `name`, `role`, `capabilities`

**Returns:** Agent object

**Example:**
```rust
let agent = ai::spawn_agent({
    "name": "assistant",
    "role": "helper",
    "capabilities": ["text_analysis"]
});
```

### `ai::send_message(from: string, to: string, message_type: string, content: Value, priority: string) -> Message`

Sends a message between agents.

**Parameters:**
- `from`: Sender agent ID
- `to`: Recipient agent ID
- `message_type`: Type of message
- `content`: Message content
- `priority`: Message priority ("low", "normal", "high", "urgent")

**Returns:** Message object

### `ai::create_coordinator(coordinator_id: string) -> AgentCoordinator`

Creates a new agent coordinator for managing multiple agents.

### `ai::create_workflow(coordinator: AgentCoordinator, name: string, steps: vector<WorkflowStep>) -> Workflow`

Creates a workflow with defined steps.

### `ai::execute_workflow(coordinator: AgentCoordinator, workflow_id: string) -> bool`

Executes a workflow.

### `ai::analyze_text(text: string) -> TextAnalysis`

Analyzes text using AI.

### `ai::generate_text(prompt: string) -> string`

Generates text using AI models.

---

## Chain Module

### `chain::get_supported_chains() -> vector<ChainConfig>`

Returns list of all supported blockchain networks.

**Returns:** Vector of chain configurations

### `chain::deploy(chain_id: int, contract_name: string, constructor_args: map<string, string>) -> string`

Deploys a smart contract to the specified chain.

**Parameters:**
- `chain_id`: Chain ID (1=Ethereum, 137=Polygon, etc.)
- `contract_name`: Name of the contract
- `constructor_args`: Constructor arguments

**Returns:** Contract address

**Example:**
```rust
let address = chain::deploy(1, "MyToken", {
    "name": "MyToken",
    "symbol": "MTK"
});
```

### `chain::call(chain_id: int, contract_address: string, function_name: string, args: map<string, string>) -> string`

Calls a function on a deployed contract.

### `chain::get_balance(chain_id: int, address: string) -> int`

Gets the balance of an address.

### `chain::estimate_gas(chain_id: int, operation: string) -> int`

Estimates gas cost for an operation.

### `chain::get_gas_price(chain_id: int) -> float`

Gets current gas price.

### `chain::mint(name: string, metadata: map<string, string>) -> int`

Mints an NFT or token.

### `chain::get(asset_id: int) -> map<string, string>`

Gets asset information.

---

## Database Module

### `database::connect(connection_string: string) -> Database`

Connects to a database.

**Parameters:**
- `connection_string`: Database connection string (e.g., "postgresql://user:pass@localhost/db")

**Returns:** Database connection object

### `database::query(db: Database, sql: string, params: vector<Value>) -> QueryResult`

Executes a SELECT query.

### `database::execute(db: Database, sql: string, params: vector<Value>) -> bool`

Executes an INSERT, UPDATE, or DELETE query.

### `database::create_table(db: Database, table_name: string, schema: TableSchema) -> bool`

Creates a database table.

### `database::create_cache(cache_id: string) -> Cache`

Creates a cache instance.

### `database::cache_set(cache: Cache, key: string, value: Value, ttl_seconds: int) -> bool`

Sets a value in cache.

### `database::cache_get(cache: Cache, key: string) -> Value`

Gets a value from cache.

---

## Web Module

### `web::create_server(config: map<string, Value>) -> HttpServer`

Creates an HTTP server.

**Parameters:**
- `config`: Server configuration with `port`, `host`

**Example:**
```rust
let server = web::create_server({
    "port": 8080,
    "host": "0.0.0.0"
});
```

### `web::add_route(server: HttpServer, method: string, path: string, handler: Function) -> bool`

Adds a route to the server.

### `web::start(server: HttpServer) -> bool`

Starts the HTTP server.

### `web::create_websocket_server(port: int) -> WebSocketServer`

Creates a WebSocket server.

---

## Auth Module

### `auth::login(username: string, password: string) -> Session`

Authenticates a user and creates a session.

### `auth::logout(session: Session) -> bool`

Logs out a user.

### `auth::session() -> Session`

Gets the current session.

### `auth::has_permission(session: Session, permission: string) -> bool`

Checks if session has a permission.

---

## Crypto Module

### `crypto::hash(data: string, algorithm: string) -> string`

Hashes data using specified algorithm.

**Algorithms:** "sha256", "sha512", "md5"

### `crypto::encrypt(data: string, key: string) -> string`

Encrypts data using AES-256.

### `crypto::decrypt(encrypted_data: string, key: string) -> string`

Decrypts data.

### `crypto::sign(data: string, private_key: string) -> string`

Signs data with a private key.

### `crypto::verify(data: string, signature: string, public_key: string) -> bool`

Verifies a signature.

---

## Log Module

### `log::info(module: string, message: string) -> void`

Logs an info message.

### `log::error(module: string, message: string) -> void`

Logs an error message.

### `log::warn(module: string, message: string) -> void`

Logs a warning message.

### `log::debug(module: string, message: string) -> void`

Logs a debug message.

**Example:**
```rust
log::info("main", "Application started");
log::error("api", "Failed to connect");
```

---

## Oracle Module

### `oracle::fetch(source: string, query: map<string, Value>) -> Value`

Fetches data from an oracle source.

### `oracle::verify(data: Value, signature: string) -> bool`

Verifies oracle data signature.

### `oracle::stream(source: string, callback: string) -> string`

Streams data from an oracle.

---

## KYC/AML Modules

### `kyc::verify(user_data: map<string, string>) -> map<string, Value>`

Verifies user identity (KYC).

**Returns:** Verification result with `status`, `score`, `details`

### `kyc::validate_document(document_type: string, document_data: map<string, string>) -> map<string, Value>`

Validates identity documents.

### `aml::check_transaction(transaction_data: map<string, string>) -> map<string, Value>`

Checks transaction for AML compliance.

**Returns:** Result with `risk_level`, `risk_score`, `flags`

### `aml::check_user(user_data: map<string, string>) -> map<string, Value>`

Checks user against AML databases.

---

## Trust Module

### `trust::get_trust_level(entity_id: string) -> string`

Gets trust level of an entity.

**Returns:** "high", "medium", "low"

### `trust::set_trust_level(entity_id: string, level: string) -> bool`

Sets trust level.

### `trust::authorize(admin_id: string, operation: string, resource: string) -> bool`

Authorizes an operation.

---

## Config Module

### `config::get(key: string) -> Value`

Gets a configuration value.

### `config::set(key: string, value: Value) -> bool`

Sets a configuration value.

### `config::load_from_file(path: string) -> map<string, Value>`

Loads configuration from a file.

---

## Service Module

### `service::register(name: string, endpoint: string) -> bool`

Registers a service.

### `service::discover(service_name: string) -> ServiceInfo`

Discovers a service.

### `service::call(service_name: string, method: string, args: map<string, Value>) -> Value`

Calls a service method.

---

## Sync Module

### `sync::push(data: map<string, Value>, target: SyncTarget) -> bool`

Pushes data to a sync target.

### `sync::pull(source: SyncTarget) -> map<string, Value>`

Pulls data from a sync source.

---

## Mobile/Desktop/IoT Modules

### Mobile Module

- `mobile::create_app(config: AppConfig) -> MobileApp`
- `mobile::create_screen(app: MobileApp, screen_config: ScreenConfig) -> Screen`
- `mobile::request_permission(permission: string) -> bool`

### Desktop Module

- `desktop::create_window(config: WindowConfig) -> Window`
- `desktop::show_dialog(type: string, message: string) -> string`

### IoT Module

- `iot::connect_device(device_id: string, protocol: string) -> Device`
- `iot::send_command(device: Device, command: string, params: map<string, Value>) -> bool`
- `iot::get_device_status(device_id: string) -> DeviceStatus`

---

**For more examples, see the [Examples](../examples/) directory.**


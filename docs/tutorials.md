# dist_agent_lang Tutorial Series

Welcome to the comprehensive tutorial series for `dist_agent_lang`! This guide will take you from complete beginner to advanced practitioner, covering all aspects of the language.

## 📚 Tutorial Overview

### Beginner Level (Tutorials 1-4)
- Basic syntax and concepts
- Variables, functions, and control flow
- Standard library basics (22 modules)
- Multi-chain operations

### Intermediate Level (Tutorials 5-8)
- Attributes and security
- Error handling
- Service architecture
- AI integration and agents

### Advanced Level (Tutorials 9-12)
- Database operations and web APIs
- Compliance features (KYC/AML)
- Performance optimization
- Testing and debugging
- Desktop/mobile/IoT support

## 🚀 Tutorial 1: Getting Started

### Prerequisites
- Rust installed (1.70+)
- Basic programming knowledge
- Terminal/command line experience

### Installation
```bash
# Clone the repository
git clone <repository-url>
cd dist_agent_lang

# Build the project
cargo build

# Run the first example
cargo run
```

### Your First Program
Create a file called `hello.dal`:

```rust
// Hello World - Your first dist_agent_lang program
fn main() {
    // Basic variable assignment
    let message = "Hello from dist_agent_lang!";
    let version = "1.0.0";
    
    // Print to console
    print(message);
    
    // Log with structured data
    log::info("Application started", { 
        "version": version, 
        "timestamp": 1234567890 
    });
    
    // Basic arithmetic
    let x = 42;
    let y = 8;
    let sum = add(x, y);
    
    log::info("Calculation completed", { 
        "x": x, 
        "y": y, 
        "sum": sum 
    });
    
    // Return null to indicate successful completion
    null
}
```

### Running Your Program
```bash
cargo run -- hello.dal
```

### What You Learned
- ✅ Basic syntax structure
- ✅ Variable declaration with `let`
- ✅ Function definition with `fn`
- ✅ Built-in functions (`print`, `add`)
- ✅ Standard library usage (`log::info`)
- ✅ Return statements

### Exercises
1. Modify the program to calculate the product of two numbers
2. Add more variables and print them
3. Try different data types (strings, numbers)

---

## 🚀 Tutorial 2: Variables, Functions, and Control Flow

### Variables and Data Types
```rust
fn data_types_demo() {
    // Integer
    let age = 25;
    
    // String
    let name = "Alice";
    
    // Boolean
    let is_active = true;
    
    // Null value
    let empty = null;
    
    // Print all variables
    print("Age: " + age);
    print("Name: " + name);
    print("Active: " + is_active);
}
```

### Functions with Parameters and Return Values
```rust
fn calculate_total(price: int, quantity: int, tax_rate: float) -> int {
    let subtotal = price * quantity;
    let tax = subtotal * tax_rate;
    subtotal + tax
}

fn greet_user(name: string, greeting: string) -> string {
    greeting + ", " + name + "!"
}

fn main() {
    let total = calculate_total(100, 3, 0.08);
    let message = greet_user("Alice", "Hello");
    
    print("Total: " + total);
    print(message);
}
```

### Control Flow
```rust
fn control_flow_demo(score: int) {
    if score >= 90 {
        print("Excellent!");
    } else if score >= 80 {
        print("Good!");
    } else if score >= 70 {
        print("Fair!");
    } else {
        print("Needs improvement!");
    }
}

fn loop_demo() {
    let count = 0;
    while count < 5 {
        print("Count: " + count);
        count = count + 1;
    }
}
```

### What You Learned
- ✅ Different data types (int, string, bool, null)
- ✅ Function parameters and return types
- ✅ If/else conditional statements
- ✅ While loops
- ✅ String concatenation

### Exercises
1. Create a function that calculates the factorial of a number
2. Write a function that checks if a number is even or odd
3. Create a simple calculator with add, subtract, multiply, divide

---

## 🚀 Tutorial 3: Standard Library Basics (22 Modules)

### Logging with `log::` Namespace
```rust
fn logging_demo() {
    // Different log levels
    log::info("User logged in", { 
        "user_id": "123", 
        "ip": "192.168.1.1" 
    });
    
    log::warning("High memory usage", { 
        "usage": "85%", 
        "threshold": "80%" 
    });
    
    log::error("Database connection failed", { 
        "error_code": "CONN_001" 
    });
    
    log::audit("user_login", { 
        "user_id": "123", 
        "success": true 
    });
    
    log::debug("Function execution time", { 
        "duration_ms": 150 
    });
}
```

### Blockchain Operations with `chain::` Namespace
```rust
fn blockchain_demo() {
    // Create an NFT
    let metadata = {
        "name": "My First NFT",
        "description": "A digital artwork",
        "creator": "alice",
        "created_at": timestamp()
    };
    
    let asset_id = chain::mint("MyNFT", metadata);
    print("Created NFT with ID: " + asset_id);
    
    // Get asset information
    let asset_info = chain::get(asset_id);
    print("Asset info: " + asset_info);
    
    // Check if asset exists
    let exists = chain::exists(asset_id);
    print("Asset exists: " + exists);
    
    // Multi-chain operations
    let chains = [1, 137, 56, 42161]; // Ethereum, Polygon, BSC, Arbitrum
    
    for chain_id in chains {
        let gas_price = chain::get_gas_price(chain_id);
        let balance = chain::get_balance(chain_id, "0x123...");
        
        log::info("Chain info", { 
            "chain_id": chain_id, 
            "gas_price": gas_price,
            "balance": balance 
        });
    }
}
```

### Authentication with `auth::` Namespace
```rust
fn auth_demo() {
    // Create a user session
    let session = auth::session("user123", ["admin", "moderator"]);
    print("Session created: " + session);
    
    // Check session validity
    let is_valid = auth::is_valid_session(&session);
    print("Session valid: " + is_valid);
    
    // Check user roles
    let has_admin = auth::has_role(&session, "admin");
    let can_write = auth::has_permission(&session, "write");
    
    print("Has admin role: " + has_admin);
    print("Can write: " + can_write);
}
```

### Cryptography with `crypto::` Namespace
```rust
fn crypto_demo() {
    // Hash a message
    let message = "Hello, World!";
    let hash = crypto::hash(message, HashAlgorithm::SHA256);
    print("SHA256 hash: " + hash);
    
    // Generate a keypair
    let keypair = crypto::generate_keypair(SignatureAlgorithm::RSA);
    print("Generated keypair: " + keypair);
    
    // Sign a message
    let signature = crypto::sign(
        message, 
        &keypair["private_key"], 
        SignatureAlgorithm::RSA
    );
    print("Signature: " + signature);
    
    // Verify signature
    let is_valid = crypto::verify(
        message, 
        &signature, 
        &keypair["public_key"], 
        SignatureAlgorithm::RSA
    );
    print("Signature valid: " + is_valid);
}
```

### What You Learned
- ✅ Logging with different levels and structured data
- ✅ Blockchain operations (mint, get, exists)
- ✅ Multi-chain support (Ethereum, Polygon, BSC, Arbitrum)
- ✅ Authentication and authorization
- ✅ Cryptographic operations (hash, sign, verify)
- ✅ Working with complex data structures

### Exercises
1. Create a simple user registration system with logging
2. Build a basic NFT creation workflow
3. Implement a simple authentication check
4. Deploy a contract to multiple chains

---

## 🚀 Tutorial 4: Multi-Chain Operations

### Understanding Multi-Chain Support
`dist_agent_lang` provides native support for multiple blockchain networks, allowing you to deploy and interact with contracts across different chains.

### Chain Selection
```rust
fn chain_selection_demo() {
    // Define supported chains
    let chains = {
        "ethereum": 1,      // Mainnet
        "polygon": 137,     // Mainnet
        "bsc": 56,          // Mainnet
        "arbitrum": 42161,  // Mainnet
        "goerli": 5,        // Testnet
        "mumbai": 80001     // Testnet
    };
    
    // Select chain based on use case
    fn select_chain(use_case: string) -> int {
        match use_case {
            "high_value" => chains["ethereum"],
            "gaming" => chains["polygon"],
            "micro_transaction" => chains["bsc"],
            "defi" => chains["arbitrum"],
            "testing" => chains["goerli"],
            _ => chains["ethereum"]
        }
    }
    
    // Use cases
    let high_value_chain = select_chain("high_value");
    let gaming_chain = select_chain("gaming");
    let defi_chain = select_chain("defi");
    
    log::info("Chain selection", {
        "high_value": high_value_chain,
        "gaming": gaming_chain,
        "defi": defi_chain
    });
}
```

### Multi-Chain Deployment
```rust
fn multi_chain_deployment() {
    // Deploy to all supported chains
    let contract_name = "MyToken";
    let constructor_args = {
        "name": "Multi-Chain Token",
        "symbol": "MCT",
        "total_supply": 1000000
    };
    
    let deployed_addresses = {};
    
    for chain_id in [1, 137, 56, 42161] {
        try {
            let address = chain::deploy(chain_id, contract_name, constructor_args);
            deployed_addresses[chain_id] = address;
            
            log::info("Deployed successfully", {
                "chain_id": chain_id,
                "address": address
            });
        } catch error {
            log::error("Deployment failed", {
                "chain_id": chain_id,
                "error": error
            });
        }
    }
    
    return deployed_addresses;
}
```

### Cross-Chain Operations
```rust
fn cross_chain_operations() {
    // Find cheapest chain for operation
    fn find_cheapest_chain(operation: string) -> int {
        let mut cheapest_chain = 1;
        let mut lowest_cost = 999999;
        
        for chain_id in [1, 137, 56, 42161] {
            let gas_estimate = chain::estimate_gas(chain_id, operation);
            let gas_price = chain::get_gas_price(chain_id);
            let total_cost = gas_estimate * gas_price;
            
            if total_cost < lowest_cost {
                lowest_cost = total_cost;
                cheapest_chain = chain_id;
            }
        }
        
        return cheapest_chain;
    }
    
    // Execute on cheapest chain
    let operation = "transfer";
    let cheapest_chain = find_cheapest_chain(operation);
    
    let result = chain::call(cheapest_chain, contract_address, operation, {
        "to": recipient,
        "amount": amount
    });
    
    log::info("Cross-chain operation", {
        "chain_id": cheapest_chain,
        "operation": operation,
        "result": result
    });
}
```

### What You Learned
- ✅ Multi-chain support (Ethereum, Polygon, BSC, Arbitrum)
- ✅ Chain selection based on use case
- ✅ Multi-chain deployment
- ✅ Cross-chain operations
- ✅ Gas optimization across chains

### Exercises
1. Deploy a simple token to all supported chains
2. Create a function that finds the cheapest chain for transfers
3. Implement cross-chain balance checking
4. Build a multi-chain NFT marketplace

---

## 🚀 Tutorial 5: Attributes and Security

### Understanding Attributes
Attributes are annotations that configure behavior without expanding keywords. They provide declarative security and resource management.

### Basic Attributes
```rust
// Security attribute - enforces security checks
@secure
fn sensitive_operation() {
    // This function will have security checks applied
    log::audit("sensitive_operation_called", { "timestamp": timestamp() });
}

// Transaction attribute - wraps in transaction
@txn
fn transfer_money(from: string, to: string, amount: int) {
    // This function will be wrapped in a transaction
    // If any part fails, the entire operation is rolled back
}

// Resource limit attribute - limits execution resources
@limit(1000)
fn expensive_operation() {
    // This function is limited to 1000 operations
    // Useful for preventing infinite loops or resource exhaustion
}

// Trust attribute - specifies trust model
@trust(hybrid)
fn hybrid_function() {
    // This function uses hybrid trust (decentralized + centralized)
}

// Compilation target attribute
@compile_target(blockchain)
fn blockchain_function() {
    // This function is compiled for blockchain execution
}

// Chain specification attribute
@chain(ethereum)
fn ethereum_function() {
    // This function is specific to Ethereum
}

// Interface generation attribute
@interface(typescript)
fn typescript_interface() {
    // This function generates TypeScript interface
}
```

### Combining Attributes
```rust
@secure
@txn
@limit(100)
@trust(hybrid)
@compile_target(blockchain)
@chain(ethereum)
@interface(typescript)
fn secure_transfer(from: string, to: string, amount: int) -> bool {
    // This function has:
    // - Security checks (@secure)
    // - Transaction wrapping (@txn)
    // - Resource limits (@limit)
    // - Hybrid trust (@trust)
    // - Blockchain compilation (@compile_target)
    // - Ethereum specificity (@chain)
    // - TypeScript interface generation (@interface)
    
    if amount <= 0 {
        return false;
    }
    
    // Perform transfer logic
    let success = perform_transfer(from, to, amount);
    
    if success {
        log::audit("transfer_successful", { 
            "from": from, 
            "to": to, 
            "amount": amount 
        });
    }
    
    success
}
```

### What You Learned
- ✅ Purpose and usage of attributes
- ✅ Security attribute (`@secure`)
- ✅ Transaction attribute (`@txn`)
- ✅ Resource limit attribute (`@limit`)
- ✅ Trust attribute (`@trust`)
- ✅ Compilation target attribute (`@compile_target`)
- ✅ Chain specification attribute (`@chain`)
- ✅ Interface generation attribute (`@interface`)
- ✅ Combining multiple attributes

### Exercises
1. Create a secure user management service with multiple attributes
2. Implement a transaction-based payment system
3. Build a resource-limited data processing function
4. Create a multi-chain contract with interface generation

---

## 🚀 Tutorial 6: AI Integration and Agents

### Understanding AI Integration
`dist_agent_lang` provides native AI agent and workflow management capabilities, allowing you to create intelligent, autonomous systems.

### Creating AI Agents
```rust
fn ai_agent_demo() {
    // Create an AI agent configuration
    let agent_config = {
        "agent_id": "trading_agent_001",
        "name": "Crypto Trading Agent",
        "role": "market_analyzer",
        "capabilities": ["price_analysis", "risk_assessment", "trade_execution"],
        "memory_size": 1000,
        "max_concurrent_tasks": 5,
        "trust_level": "hybrid",
        "communication_protocols": ["http", "websocket"],
        "ai_models": ["gpt-4", "claude-3"]
    };
    
    // Create the agent
    let agent = ai::create_agent(agent_config);
    
    // Add agent to coordinator
    let coordinator = ai::create_agent_coordinator();
    let agent_id = ai::add_agent_to_coordinator(coordinator, agent);
    
    log::info("AI agent created", {
        "agent_id": agent_id,
        "name": agent_config["name"]
    });
}
```

### Agent Tasks and Workflows
```rust
fn agent_tasks_demo() {
    // Create a task for the agent
    let task = ai::create_task(agent, "market_analysis", "Analyze BTC market conditions", {
        "asset": "BTC",
        "timeframe": "1h",
        "indicators": ["rsi", "macd", "volume"]
    });
    
    // Execute the task
    let result = ai::execute_task(agent, task.id);
    
    // Create a workflow
    let workflow_config = {
        "workflow_id": "trading_workflow_001",
        "name": "Automated Trading Workflow",
        "steps": [
            {
                "step_id": "market_analysis",
                "task_type": "market_analysis",
                "agent_id": "trading_agent_001",
                "dependencies": []
            },
            {
                "step_id": "risk_assessment",
                "task_type": "risk_assessment",
                "agent_id": "trading_agent_001",
                "dependencies": ["market_analysis"]
            },
            {
                "step_id": "trade_execution",
                "task_type": "trade_execution",
                "agent_id": "trading_agent_001",
                "dependencies": ["risk_assessment"]
            }
        ]
    };
    
    let workflow = ai::create_workflow(coordinator, workflow_config);
    let success = ai::execute_workflow(coordinator, workflow.workflow_id);
}
```

### AI Service Integration
```rust
fn ai_service_demo() {
    // Create AI service
    let ai_service = service::create_ai_service("gpt-4");
    
    // AI-powered analysis
    let analysis = await service::ai("Analyze the current market conditions for Bitcoin", ai_service);
    
    // AI-powered prediction
    let prediction = await service::ai("Predict Bitcoin price movement for the next 24 hours", ai_service);
    
    // AI-powered decision making
    let decision = await service::ai("Should I buy or sell Bitcoin based on current market data?", ai_service);
    
    log::info("AI analysis completed", {
        "analysis": analysis,
        "prediction": prediction,
        "decision": decision
    });
}
```

### Agent Communication
```rust
fn agent_communication_demo() {
    // Create multiple agents
    let agent1 = ai::create_agent({
        "name": "Data Collector",
        "role": "data_gathering"
    });
    
    let agent2 = ai::create_agent({
        "name": "Data Analyzer",
        "role": "data_analysis"
    });
    
    let agent3 = ai::create_agent({
        "name": "Decision Maker",
        "role": "decision_making"
    });
    
    // Agent communication workflow
    let message1 = agent::create_agent_message(
        agent1.id, 
        agent2.id, 
        "data_collected", 
        { "data": collected_data }
    );
    
    agent::send_message(agent1.id, message1);
    
    let message2 = agent::create_agent_message(
        agent2.id, 
        agent3.id, 
        "analysis_complete", 
        { "analysis": analysis_result }
    );
    
    agent::send_message(agent2.id, message2);
}
```

### What You Learned
- ✅ AI agent creation and configuration
- ✅ Agent task management
- ✅ Workflow creation and execution
- ✅ AI service integration
- ✅ Agent communication patterns
- ✅ Multi-agent coordination

### Exercises
1. Create an AI agent for price monitoring
2. Build a workflow for automated trading
3. Implement agent communication for data processing
4. Create a multi-agent system for portfolio management

---

## 🚀 Tutorial 7: Database Operations

### Understanding Database Integration
`dist_agent_lang` provides comprehensive database operations for both traditional and blockchain data storage.

### Database Connection
```rust
fn database_connection_demo() {
    // Connect to database
    let connection_string = "postgresql://user:password@localhost:5432/mydb";
    let db = database::connect(connection_string);
    
    // Check connection
    let connected = database::is_connected(db);
    print("Database connected: " + connected);
    
    // Get database info
    let db_info = database::get_database_info(db);
    log::info("Database info", db_info);
}
```

### Basic CRUD Operations
```rust
fn crud_operations_demo() {
    let db = database::connect("postgresql://user:password@localhost:5432/mydb");
    
    // Create (Insert)
    let user_data = {
        "name": "John Doe",
        "email": "john@example.com",
        "age": 30
    };
    
    let inserted = database::insert(db, "users", user_data);
    log::info("User inserted", { "id": inserted.id });
    
    // Read (Query)
    let users = database::query(db, "SELECT * FROM users WHERE age > 25", []);
    log::info("Users found", { "count": users.length });
    
    // Update
    let update_data = { "age": 31 };
    let updated = database::update(db, "users", update_data, "id = " + inserted.id);
    log::info("User updated", { "rows_affected": updated });
    
    // Delete
    let deleted = database::delete(db, "users", "id = " + inserted.id);
    log::info("User deleted", { "rows_affected": deleted });
}
```

### Transaction Management
```rust
fn transaction_demo() {
    let db = database::connect("postgresql://user:password@localhost:5432/mydb");
    
    // Begin transaction
    let transaction = database::begin_transaction(db);
    
    try {
        // Multiple operations in transaction
        let user1 = database::insert(db, "users", { "name": "Alice", "email": "alice@example.com" });
        let user2 = database::insert(db, "users", { "name": "Bob", "email": "bob@example.com" });
        
        // Create relationship
        database::insert(db, "friendships", {
            "user1_id": user1.id,
            "user2_id": user2.id,
            "created_at": timestamp()
        });
        
        // Commit transaction
        let committed = database::commit_transaction(transaction);
        log::info("Transaction committed", { "success": committed });
        
    } catch error {
        // Rollback transaction on error
        let rolled_back = database::rollback_transaction(transaction);
        log::error("Transaction rolled back", { "error": error });
    }
}
```

### Advanced Database Features
```rust
fn advanced_database_demo() {
    let db = database::connect("postgresql://user:password@localhost:5432/mydb");
    
    // Batch operations
    let users = [
        { "name": "Alice", "email": "alice@example.com" },
        { "name": "Bob", "email": "bob@example.com" },
        { "name": "Charlie", "email": "charlie@example.com" }
    ];
    
    let batch_result = database::batch_insert(db, "users", users);
    log::info("Batch insert completed", { "inserted": batch_result.length });
    
    // Complex queries with parameters
    let query = "SELECT * FROM users WHERE age > ? AND status = ?";
    let params = [25, "active"];
    let results = database::query(db, query, params);
    
    // Database backup
    let backup_info = database::create_backup(db, {
        "format": "sql",
        "compress": true,
        "include_data": true
    });
    
    log::info("Backup created", {
        "backup_id": backup_info.id,
        "size": backup_info.size,
        "path": backup_info.path
    });
}
```

### What You Learned
- ✅ Database connection management
- ✅ CRUD operations (Create, Read, Update, Delete)
- ✅ Transaction management
- ✅ Batch operations
- ✅ Complex queries with parameters
- ✅ Database backup and restore

### Exercises
1. Create a user management system with database operations
2. Implement a transaction-based payment system
3. Build a data migration script
4. Create a database backup and restore system

---

## 🚀 Tutorial 8: Web API Operations

### Understanding Web Integration
`dist_agent_lang` provides comprehensive web API support for HTTP and WebSocket operations.

### HTTP Operations
```rust
fn http_operations_demo() {
    // GET request
    let response = web::get("https://api.example.com/users");
    log::info("GET response", { "status": response.status, "data": response.data });
    
    // POST request
    let post_data = { "name": "John Doe", "email": "john@example.com" };
    let post_response = web::post("https://api.example.com/users", post_data);
    
    // PUT request
    let put_data = { "name": "John Smith" };
    let put_response = web::put("https://api.example.com/users/123", put_data);
    
    // DELETE request
    let delete_response = web::delete("https://api.example.com/users/123");
    
    // Request with headers
    let headers = {
        "Authorization": "Bearer token123",
        "Content-Type": "application/json"
    };
    
    let auth_response = web::get_with_headers("https://api.example.com/protected", headers);
}
```

### WebSocket Operations
```rust
fn websocket_demo() {
    // Connect to WebSocket
    let ws = web::websocket_connect("wss://echo.websocket.org");
    
    // Send message
    let message = { "type": "ping", "data": "Hello WebSocket!" };
    let sent = web::websocket_send(ws, message);
    
    // Receive message
    let received = web::websocket_receive(ws);
    log::info("WebSocket message received", received);
    
    // Close connection
    web::websocket_close(ws);
}
```

### API Integration Examples
```rust
fn api_integration_demo() {
    // External API integration
    let weather_api = "https://api.openweathermap.org/data/2.5/weather";
    let city = "London";
    let api_key = config::get_env("WEATHER_API_KEY");
    
    let weather_url = weather_api + "?q=" + city + "&appid=" + api_key;
    let weather_response = web::get(weather_url);
    
    log::info("Weather data", {
        "city": city,
        "temperature": weather_response.data.main.temp,
        "description": weather_response.data.weather[0].description
    });
    
    // Cryptocurrency price API
    let crypto_api = "https://api.coingecko.com/api/v3/simple/price";
    let crypto_url = crypto_api + "?ids=bitcoin,ethereum&vs_currencies=usd";
    let crypto_response = web::get(crypto_url);
    
    log::info("Crypto prices", {
        "bitcoin": crypto_response.data.bitcoin.usd,
        "ethereum": crypto_response.data.ethereum.usd
    });
}
```

### What You Learned
- ✅ HTTP operations (GET, POST, PUT, DELETE)
- ✅ WebSocket connections and messaging
- ✅ API integration with external services
- ✅ Request headers and authentication
- ✅ Response handling and error management

### Exercises
1. Create a weather monitoring system using external APIs
2. Build a cryptocurrency price tracker
3. Implement a real-time chat system with WebSockets
4. Create a REST API client for your application

---

## 🚀 Tutorial 9: Compliance Features (KYC/AML)

### Understanding Compliance
`dist_agent_lang` provides built-in KYC (Know Your Customer) and AML (Anti-Money Laundering) features for regulatory compliance.

### KYC Operations
```rust
fn kyc_demo() {
    // User verification
    let user_data = {
        "name": "John Doe",
        "email": "john@example.com",
        "phone": "+1234567890",
        "date_of_birth": "1990-01-01",
        "nationality": "US"
    };
    
    let verification = kyc::verify_user(user_data);
    log::info("KYC verification", {
        "user_id": verification.user_id,
        "status": verification.status,
        "score": verification.verification_score
    });
    
    // Document verification
    let document_data = {
        "document_type": "passport",
        "document_number": "123456789",
        "issuing_country": "US",
        "expiry_date": "2025-01-01"
    };
    
    let document_verified = kyc::verify_document(document_data);
    
    // Identity verification
    let identity_data = {
        "full_name": "John Doe",
        "date_of_birth": "1990-01-01",
        "address": "123 Main St, City, State, ZIP"
    };
    
    let identity_verified = kyc::verify_identity(identity_data);
}
```

### AML Operations
```rust
fn aml_demo() {
    // Transaction monitoring
    let transaction_data = {
        "from_address": "0x123...",
        "to_address": "0x456...",
        "amount": 10000,
        "currency": "USD",
        "transaction_type": "transfer",
        "timestamp": timestamp()
    };
    
    let risk_score = aml::calculate_risk_score(transaction_data);
    log::info("AML risk assessment", {
        "transaction_id": transaction_data.id,
        "risk_score": risk_score,
        "risk_level": risk_score > 0.7 ? "high" : "low"
    });
    
    // Flag suspicious transactions
    let flagged = aml::flag_suspicious_transaction(transaction_data);
    if flagged {
        log::warning("Suspicious transaction flagged", {
            "transaction_id": transaction_data.id,
            "reason": flagged.reason
        });
    }
    
    // Compliance checks
    let user_data = {
        "user_id": "user123",
        "kyc_status": "verified",
        "risk_level": "medium"
    };
    
    let compliant = aml::check_compliance(user_data, transaction_data);
    
    // Generate compliance report
    let report = aml::generate_compliance_report("user123");
    log::info("Compliance report generated", {
        "report_id": report.id,
        "total_transactions": report.total_transactions,
        "flagged_transactions": report.flagged_transactions
    });
}
```

### Compliance Workflow
```rust
fn compliance_workflow_demo() {
    // Complete compliance workflow
    fn process_user_registration(user_data: map<string, any>) -> bool {
        // Step 1: KYC verification
        let kyc_result = kyc::verify_user(user_data);
        
        if kyc_result.status != "verified" {
            log::error("KYC verification failed", { "user_id": user_data.user_id });
            return false;
        }
        
        // Step 2: Document verification
        let document_result = kyc::verify_document(user_data.documents);
        
        if !document_result.verified {
            log::error("Document verification failed", { "user_id": user_data.user_id });
            return false;
        }
        
        // Step 3: Risk assessment
        let risk_assessment = aml::calculate_risk_score(user_data);
        
        if risk_assessment > 0.8 {
            log::warning("High-risk user detected", { "user_id": user_data.user_id });
            // Additional verification required
        }
        
        // Step 4: Compliance check
        let compliant = aml::check_compliance(user_data, {});
        
        if compliant {
            log::audit("User registration completed", { "user_id": user_data.user_id });
            return true;
        } else {
            log::error("Compliance check failed", { "user_id": user_data.user_id });
            return false;
        }
    }
}
```

### What You Learned
- ✅ KYC user verification
- ✅ Document verification
- ✅ Identity verification
- ✅ AML risk assessment
- ✅ Suspicious transaction flagging
- ✅ Compliance reporting
- ✅ Complete compliance workflow

### Exercises
1. Create a complete user onboarding system with KYC/AML
2. Implement transaction monitoring for suspicious activity
3. Build a compliance reporting dashboard
4. Create a risk assessment system

---

## 🚀 Tutorial 10: Error Handling

### Understanding Error Handling
Error handling in `dist_agent_lang` uses try-catch blocks and Result types for robust error management.

### Try-Catch Blocks
```rust
fn basic_error_handling() {
    try {
        // Risky operation that might fail
        let result = risky_operation();
        print("Operation succeeded: " + result);
        
    } catch (error) {
        // Handle the error
        log::error("Operation failed", { "error": error });
        print("Operation failed: " + error);
        
    } finally {
        // Always executed, for cleanup
        cleanup_resources();
    }
}
```

### Result Types
```rust
fn safe_division(numerator: int, denominator: int) -> Result<float, string> {
    try {
        if denominator == 0 {
            throw "Division by zero";
        }
        
        let result = numerator / denominator;
        Ok(result)
        
    } catch (error) {
        log::error("Division error", { 
            "numerator": numerator,
            "denominator": denominator,
            "error": error 
        });
        
        Err(error)
    }
}

fn use_result_types() {
    let result1 = safe_division(10, 2);
    let result2 = safe_division(10, 0);
    
    if result1.is_ok() {
        print("Result 1: " + result1.unwrap());
    } else {
        print("Error 1: " + result1.unwrap_err());
    }
    
    if result2.is_ok() {
        print("Result 2: " + result2.unwrap());
    } else {
        print("Error 2: " + result2.unwrap_err());
    }
}
```

### Complex Error Handling
```rust
@secure
fn process_user_data(user_id: string, data: map<string, any>) -> Result<bool, string> {
    let transaction_id = generate_transaction_id();
    
    try {
        // Validate input
        if user_id == "" {
            throw "User ID cannot be empty";
        }
        
        if !data.contains("name") {
            throw "Name is required";
        }
        
        // Check if user exists
        if !users.contains(user_id) {
            throw "User not found";
        }
        
        // Process the data
        let user = users[user_id];
        user.name = data["name"];
        user.last_updated = timestamp();
        
        users[user_id] = user;
        
        // Log success
        log::audit("user_updated", { 
            "user_id": user_id, 
            "transaction_id": transaction_id 
        });
        
        Ok(true)
        
    } catch (error) {
        // Log error
        log::error("User update failed", { 
            "user_id": user_id,
            "transaction_id": transaction_id,
            "error": error 
        });
        
        // Rollback transaction if needed
        rollback_transaction(transaction_id);
        
        Err(error)
        
    } finally {
        // Cleanup
        cleanup_transaction_resources(transaction_id);
    }
}
```

### What You Learned
- ✅ Try-catch-finally blocks
- ✅ Result types (Ok/Err)
- ✅ Error logging and reporting
- ✅ Transaction rollback
- ✅ Resource cleanup
- ✅ Complex error handling patterns

### Exercises
1. Create a file processing function with error handling
2. Implement a database operation with rollback
3. Build a batch data validation system
4. Create a comprehensive error handling framework

---

## 🚀 Tutorial 11: Performance Optimization

### Understanding Performance Optimization
Performance optimization involves improving execution speed, memory usage, and resource efficiency.

### Benchmarking Your Code
```rust
fn benchmark_demo() {
    // Benchmark lexer performance
    benchmark "lexer_performance" {
        let source_code = "let x = 42; let y = 10; let z = x + y;";
        let tokens = lexer::tokenize(source_code);
        // Performance measurement happens automatically
    }
    
    // Benchmark parser performance
    benchmark "parser_performance" {
        let tokens = lexer::tokenize(source_code);
        let ast = parser::parse(tokens);
        // Performance measurement happens automatically
    }
    
    // Benchmark runtime performance
    benchmark "runtime_performance" {
        let ast = parser::parse(tokens);
        let result = runtime::execute(ast);
        // Performance measurement happens automatically
    }
}
```

### Memory Management
```rust
fn memory_optimization_demo() {
    // Use object pooling for frequently created objects
    let string_pool = memory_manager.create_object_pool::<String>("strings", 100);
    
    for i in 0..1000 {
        let pooled_string = string_pool.acquire();
        *pooled_string.value_mut() = "String " + i;
        
        // Use the string
        process_string(pooled_string.value());
        
        // String is automatically returned to pool when dropped
    }
    
    // Get memory statistics
    let memory_stats = memory_manager.get_stats();
    log::info("Memory usage", {
        "total_allocated": memory_stats.total_allocated,
        "current_usage": memory_stats.current_usage,
        "peak_usage": memory_stats.peak_usage
    });
}
```

### What You Learned
- ✅ Benchmarking code performance
- ✅ Memory management with object pooling
- ✅ Performance monitoring
- ✅ Optimization techniques

### Exercises
1. Optimize a data processing pipeline
2. Implement memory-efficient caching
3. Build a high-performance web service
4. Create performance monitoring tools

---

## 🚀 Tutorial 12: Testing and Debugging

### Understanding Testing
Testing ensures your code works correctly and helps catch bugs early.

### Unit Testing
```rust
test "basic_arithmetic" {
    let result = add(5, 3);
    assert_eq(result, 8);
    
    let result2 = multiply(4, 6);
    assert_eq(result2, 24);
}

test "user_creation" {
    let user = create_user("test_user", "Test User", "test@example.com");
    assert_eq(user.id, "test_user");
    assert_eq(user.name, "Test User");
    assert_eq(user.status, "active");
}
```

### Integration Testing
```rust
test_suite "user_management_integration" {
    test "complete_user_lifecycle" {
        // Create user
        let user = create_user("alice", "Alice Johnson", "alice@example.com");
        assert_eq(user.status, "active");
        
        // Login user
        let session = login_user("alice", "password123");
        assert_eq(session.user_id, "alice");
        assert_gt(session.expires_at, timestamp());
        
        // Update user
        let updated = update_user("alice", { "name": "Alice Smith" });
        assert_eq(updated.name, "Alice Smith");
        
        // Logout user
        let logout_success = logout_user(session.id);
        assert_eq(logout_success, true);
    }
}
```

### What You Learned
- ✅ Unit testing with assertions
- ✅ Integration testing
- ✅ Test suites and organization
- ✅ Debugging techniques

### Exercises
1. Create comprehensive tests for a user management system
2. Implement mocking for blockchain operations
3. Build a debugging framework for your application
4. Create automated testing workflows

---

## 🎯 Next Steps

Congratulations! You've completed the comprehensive tutorial series for `dist_agent_lang`. Here's what you can do next:

### Practice Projects
1. **Build a DeFi Protocol**: Create a decentralized finance application
2. **Create a Trading Bot**: Implement an AI-powered trading system
3. **Develop a Social Platform**: Build a decentralized social network
4. **Design a Supply Chain System**: Create a blockchain-based supply chain

### Advanced Topics
1. **Smart Contract Integration**: Deep dive into blockchain operations
2. **AI/ML Integration**: Explore advanced AI service usage
3. **Scalability**: Learn about horizontal scaling with agents
4. **Security**: Master advanced security patterns

### Community
1. **Join the Community**: Connect with other developers
2. **Contribute**: Help improve the language and documentation
3. **Share Projects**: Showcase your applications
4. **Get Support**: Ask questions and get help

### Resources
- [Language Reference](../Documentation.md)
- [API Documentation](../docs/api.md)
- [Examples](../examples/)
- [Best Practices](../docs/best-practices.md)

Happy coding with `dist_agent_lang`! 🚀

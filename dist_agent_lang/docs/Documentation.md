# dist_agent_lang - Comprehensive Documentation

## Table of Contents
1. [Language Overview](#language-overview)
2. [Core Architecture](#core-architecture)
3. [Language Syntax](#language-syntax)
4. [Multi-Chain Support](#multi-chain-support)
5. [Standard Library](#standard-library)
6. [Development Tools](#development-tools)
7. [Examples and Use Cases](#examples-and-use-cases)
8. [Security Model](#security-model)
9. [Performance and Optimization](#performance-and-optimization)
10. [Integration Guide](#integration-guide)

## Language Overview

### Vision
dist_agent_lang is designed to bridge the gap between decentralized and centralized networks, providing a unified programming language for:
- **Smart Contract Development**: Simplified blockchain programming
- **Distributed AI**: Agent-based systems and coordination
- **Multi-Chain Operations**: Seamless cross-chain interactions
- **Hybrid Systems**: Combining centralized and decentralized trust models
- **Compliance**: Built-in KYC/AML capabilities
- **AI Integration**: Native AI agent and workflow management

### Key Principles
1. **Agent-First Design**: Everything is an agent with messaging capabilities
2. **Hybrid Trust**: Support for both centralized and decentralized trust models
3. **Multi-Chain Native**: Built-in support for multiple blockchain networks
4. **Type Safety**: Static typing with runtime type checking
5. **Security by Default**: Capability-based security and audit trails
6. **AI Native**: First-class AI agent support
7. **Compliance Ready**: Built-in KYC/AML features

## Core Architecture

### Language Components

#### 1. Lexer (`src/lexer/`)
- **Purpose**: Tokenizes source code into tokens
- **Key Features**:
  - Support for all language keywords (`@trust`, `@secure`, `service`, `fn`, etc.)
  - Token types: Keywords, Identifiers, Literals, Operators, Punctuation
  - Error handling with detailed error messages
  - Line and column tracking for debugging
  - 197+ different token types supported

#### 2. Parser (`src/parser/`)
- **Purpose**: Builds Abstract Syntax Tree (AST) from tokens
- **Key Features**:
  - Recursive descent parsing
  - Support for all language constructs
  - Error recovery mechanisms
  - Detailed parse error messages with context

#### 3. Runtime (`src/runtime/`)
- **Purpose**: Executes AST with scope management
- **Key Features**:
  - Stack-based execution engine
  - Dynamic scope management
  - Built-in function registration
  - Namespace function calls (`chain::`, `log::`, etc.)
  - Error handling and recovery

#### 4. Standard Library (`src/stdlib/`) - 22 Modules
- **Purpose**: Built-in functions and namespaces
- **Key Features**:
  - Multi-chain operations (`chain::`)
  - Logging and audit trails (`log::`)
  - Authentication (`auth::`)
  - Cryptographic operations (`crypto::`)
  - Service management (`service::`)
  - Oracle integration (`oracle::`)
  - Configuration management (`config::`)
  - KYC features (`kyc::`)
  - AML features (`aml::`)
  - AI agent management (`ai::`)
  - Agent system (`agent::`)
  - Database operations (`database::`)
  - Web API operations (`web::`)
  - Desktop support (`desktop::`)
  - Mobile support (`mobile::`)
  - IoT support (`iot::`)
  - Trust management (`trust::`)
  - CloudAdmin security (`cloudadmin::`)

### Value System

#### Supported Types
```rust
enum Value {
    Int(i64),           // Integer values
    Float(f64),         // Floating-point values
    String(String),     // String values
    Bool(bool),         // Boolean values
    Null,               // Null value
}
```

#### Type Operations
- **Type Checking**: `type(value)` function
- **Type Conversion**: `to_string()`, `to_int()`, `to_bool()`
- **Type Safety**: Compile-time and runtime type checking

## Language Syntax

### Service Definition
```rust
@trust("hybrid")
@secure
@limit(15000)
service MyService {
    // State variables
    total_supply: int,
    balances: map<string, int>,
    
    // Events
    event Transfer { from: string, to: string, amount: int },
    
    // Functions
    fn mint(to: string, amount: int) -> bool {
        self.balances[to] = self.balances[to] + amount;
        self.total_supply = self.total_supply + amount;
        return true;
    }
}
```

### Function Definition
```rust
fn function_name(param1: type1, param2: type2) -> return_type {
    // Function body
    let result = param1 + param2;
    return result;
}
```

### Control Flow
```rust
// If statements
if condition {
    // code
} else {
    // code
}

// While loops
while condition {
    // code
}

// For loops
for item in collection {
    // code
}

// Match statements
match value {
    "option1" => { /* code */ },
    "option2" => { /* code */ },
    _ => { /* default code */ }
}
```

### Agent Operations
```rust
// Spawn agent
let agent_id = spawn {
    // agent code
    while true {
        // continuous operation
        await 60; // wait 60 seconds
    }
};

// Send message
msg(agent_id, "Hello from main!");

// Await operation
let result = await some_async_operation();
```

### Error Handling
```rust
try {
    // risky operation
    let result = chain::call(1, "0x123...", "transfer", {});
} catch error {
    // handle error
    log::error("transfer", format!("Transfer failed: {}", error));
}
```

## Multi-Chain Support

### Supported Chains
The language includes built-in support for major blockchain networks:

| Chain | Chain ID | Type | Use Case |
|-------|----------|------|----------|
| Ethereum | 1 | Mainnet | High-value transactions, DeFi |
| Polygon | 137 | Mainnet | Gaming, NFTs, low-cost transactions |
| BSC | 56 | Mainnet | Micro-transactions, DeFi |
| Arbitrum | 42161 | Mainnet | L2 scaling, DeFi |
| Goerli | 5 | Testnet | Ethereum testing |
| Mumbai | 80001 | Testnet | Polygon testing |

### Chain Operations

#### Deployment
```rust
let address = chain::deploy(
    chain_id,           // Target chain ID
    contract_name,     // Contract name
    constructor_args   // Constructor arguments
);
```

#### Contract Calls
```rust
let result = chain::call(
    chain_id,           // Target chain ID
    contract_address,   // Contract address
    function_name,      // Function to call
    arguments          // Function arguments
);
```

#### Gas Management
```rust
// Estimate gas for operation
let gas_estimate = chain::estimate_gas(chain_id, "transfer");

// Get current gas price
let gas_price = chain::get_gas_price(chain_id);

// Calculate total cost
let total_cost = gas_estimate * gas_price;
```

#### Balance and Status
```rust
// Get account balance
let balance = chain::get_balance(chain_id, address);

// Get transaction status
let status = chain::get_transaction_status(chain_id, tx_hash);

// Get block timestamp
let timestamp = chain::get_block_timestamp(chain_id);
```

### Smart Chain Selection
```rust
fn select_chain_by_use_case(use_case: string) -> int {
    match use_case {
        "high_value" => 1,      // Ethereum for security
        "gaming" => 137,        // Polygon for speed
        "micro_transaction" => 56, // BSC for lowest cost
        "defi" => 42161,        // Arbitrum for L2
        _ => 1                  // Default to Ethereum
    }
}
```

### Custom Chains
```rust
// Custom chain configuration
service CustomChainService {
    custom_chain_id: int = 999,
    
    fn deploy_to_custom_chain() -> string {
        return chain::deploy(
            self.custom_chain_id,
            "MyContract",
            { "name": "Custom Token" }
        );
    }
}
```

## Standard Library

### Chain Namespace (`chain::`)
```rust
// Core operations
chain::deploy(chain_id, contract_name, args)
chain::call(chain_id, address, function, args)
chain::estimate_gas(chain_id, operation)
chain::get_gas_price(chain_id)
chain::get_balance(chain_id, address)
chain::get_transaction_status(chain_id, tx_hash)
chain::get_block_timestamp(chain_id)

// Asset operations
chain::mint(name, metadata)
chain::update(id, metadata)
chain::get(id)
chain::exists(id)

// Chain information
chain::get_chain_config(chain_id)
chain::get_supported_chains()
```

### Log Namespace (`log::`)
```rust
// Logging levels
log::info(component, message)
log::error(component, data)
log::audit(operation, data)

// Example
log::info("transfer", format!("Transferring {} tokens", amount));
log::audit("mint", {
    "amount": amount,
    "recipient": recipient,
    "timestamp": timestamp
});
```

### Auth Namespace (`auth::`)
```rust
// Authentication operations
auth::verify_signature(message, signature, public_key)
auth::hash_password(password)
auth::verify_password(password, hash)
auth::generate_keypair()
```

### Crypto Namespace (`crypto::`)
```rust
// Cryptographic operations
crypto::sha256(data)
crypto::sha512(data)
crypto::md5(data)
crypto::random_bytes(length)
crypto::encrypt(data, key)
crypto::decrypt(data, key)
```

### Service Namespace (`service::`)
```rust
// Service management
service::register(name, endpoint)
service::discover(name)
service::call(name, method, args)
service::health_check(name)
```

### Oracle Namespace (`oracle::`)
```rust
// External data integration
oracle::fetch(url)
oracle::query_database(query)
oracle::get_price(asset)
oracle::get_weather(location)
```

### AI Namespace (`ai::`)
```rust
// AI agent management
ai::create_agent(agent_config)
ai::add_agent_to_coordinator(coordinator, agent)
ai::create_task(agent, task_type, description, parameters)
ai::execute_task(agent, task_id)
ai::create_workflow(coordinator, workflow_config)
ai::execute_workflow(coordinator, workflow_id)
```

### Database Namespace (`database::`)
```rust
// Database operations
database::connect(connection_string)
database::query(db, sql, params)
database::insert(db, table_name, data)
database::update(db, table_name, data, condition)
database::delete(db, table_name, condition)
database::begin_transaction(db)
database::commit_transaction(transaction)
database::rollback_transaction(transaction)
```

### Web Namespace (`web::`)
```rust
// Web API operations
web::get(url)
web::post(url, data)
web::put(url, data)
web::delete(url)
web::websocket_connect(url)
web::websocket_send(ws, message)
web::websocket_receive(ws)
```

### KYC Namespace (`kyc::`)
```rust
// KYC operations
kyc::verify_user(user_data)
kyc::get_verification_status(user_id)
kyc::verify_document(document_data)
kyc::verify_identity(identity_data)
```

### AML Namespace (`aml::`)
```rust
// AML operations
aml::calculate_risk_score(transaction_data)
aml::flag_suspicious_transaction(transaction_data)
aml::check_compliance(user_data, transaction_data)
aml::generate_compliance_report(user_id)
```

## Development Tools

### Testing Framework
```rust
// Test suite definition
@test
service MyTestSuite {
    fn test_basic_operations() -> bool {
        // Create service instance - both syntaxes work:
        let service = MyService::new();
        // or: let service = service::new("MyService");
        
        let result = service.mint("0x123...", 1000);
        return result == true;
    }
    
    fn test_error_handling() -> bool {
        let service = MyService::new();
        try {
            service.mint("invalid", -100);
            return false; // Should not reach here
        } catch error {
            return true; // Expected error
        }
    }
}
```

### Performance Benchmarks
```rust
// Benchmark definition
@benchmark
fn benchmark_transfer() {
    let service = TokenService::new();
    b.iter(|| {
        service.transfer("0x123...", "0x456...", 100);
    });
}
```

### Error Handling
```rust
// Parse errors
ParseError::UnexpectedToken { token, expected, line, column }
ParseError::UnexpectedEOF { expected }
ParseError::InvalidAttribute { attribute, line }

// Runtime errors
RuntimeError::StackUnderflow
RuntimeError::FunctionNotFound(name)
RuntimeError::TypeMismatch(message)
RuntimeError::DivisionByZero
RuntimeError::UnsupportedOperation(operation)
```

## Examples and Use Cases

### Smart Contract Examples

#### Basic ERC20 Token
```rust
@trust("hybrid")
@secure
service BasicToken {
    total_supply: int,
    balances: map<string, int>,
    
    fn mint(to: string, amount: int) -> bool {
        self.balances[to] = self.balances[to] + amount;
        self.total_supply = self.total_supply + amount;
        return true;
    }
    
    fn transfer(from: string, to: string, amount: int) -> bool {
        if self.balances[from] >= amount {
            self.balances[from] = self.balances[from] - amount;
            self.balances[to] = self.balances[to] + amount;
            return true;
        }
        return false;
    }
}
```

#### Multi-Chain DeFi Protocol
```rust
@trust("hybrid")
@secure
service MultiChainDeFi {
    contract_addresses: map<int, string>,
    oracle_feeds: map<string, OracleFeed>,
    
    fn deploy_to_all_chains() -> bool {
        let chains = [1, 137, 56, 42161];
        
        for chain_id in chains {
            let address = chain::deploy(chain_id, "DeFiProtocol", {
                "name": "Multi-Chain DeFi",
                "version": "1.0"
            });
            self.contract_addresses[chain_id] = address;
        }
        
        return true;
    }
    
    fn execute_on_cheapest_chain(operation: string, args: map<string, string>) -> string {
        let mut cheapest_chain = 1;
        let mut lowest_cost = chain::estimate_gas(1, operation) * chain::get_gas_price(1);
        
        for chain_id in [137, 56, 42161] {
            let cost = chain::estimate_gas(chain_id, operation) * chain::get_gas_price(chain_id);
            if cost < lowest_cost {
                lowest_cost = cost;
                cheapest_chain = chain_id;
            }
        }
        
        return chain::call(cheapest_chain, self.contract_addresses[cheapest_chain], operation, args);
    }
}
```

### Agent-Based Systems

#### Monitoring Agent
```rust
@trust("hybrid")
@secure
service MonitoringSystem {
    agents: map<string, agent>,
    
    fn spawn_gas_monitor() -> string {
        let agent_id = spawn {
            while true {
                let eth_gas = chain::get_gas_price(1);
                let poly_gas = chain::get_gas_price(137);
                
                if eth_gas > 50 {
                    log::info("monitor", "Ethereum gas price is high!");
                }
                
                if poly_gas < 1 {
                    log::info("monitor", "Polygon gas price is very low!");
                }
                
                await 300; // Check every 5 minutes
            }
        };
        
        self.agents["gas_monitor"] = agent_id;
        return agent_id;
    }
    
    fn spawn_price_monitor() -> string {
        let agent_id = spawn {
            while true {
                let eth_price = oracle::get_price("ETH");
                let poly_price = oracle::get_price("MATIC");
                
                log::info("monitor", format!("ETH: ${}, MATIC: ${}", eth_price, poly_price));
                await 60; // Check every minute
            }
        };
        
        self.agents["price_monitor"] = agent_id;
        return agent_id;
    }
}
```

### Web Integration Examples

#### User Interface Integration
```html
<!DOCTYPE html>
<html>
<head>
    <title>dist_agent_lang Token Interface</title>
</head>
<body>
    <div id="wallet-connect">
        <button onclick="connectWallet()">Connect Wallet</button>
    </div>
    
    <div id="token-info">
        <h2>Token Balance</h2>
        <p id="balance">0 KEYS</p>
        <button onclick="transfer()">Transfer</button>
    </div>
    
    <script>
        async function connectWallet() {
            // Wallet connection logic
            console.log("Connecting to dist_agent_lang service...");
        }
        
        async function transfer() {
            // Transfer logic using dist_agent_lang
            console.log("Executing transfer on optimal chain...");
        }
    </script>
</body>
</html>
```

## Security Model

### Trust Models
```rust
// Hybrid trust (default)
@trust("hybrid")
service HybridService {
    // Combines centralized and decentralized trust
}

// Pure decentralized trust
@trust("decentralized")
service DecentralizedService {
    // Only blockchain-based trust
}

// Centralized trust
@trust("centralized")
service CentralizedService {
    // Traditional centralized trust
}
```

### Capability-Based Security
```rust
@cap("admin")
fn admin_only_function() {
    // Only callable by admin
}

@cap("user")
fn user_function() {
    // Callable by users
}

@auth("signature")
fn authenticated_function() {
    // Requires signature verification
}
```

### Audit Trails
```rust
fn sensitive_operation() {
    log::audit("sensitive_op", {
        "user": current_user,
        "operation": "mint",
        "amount": amount,
        "timestamp": timestamp,
        "chain_id": chain_id
    });
    
    // Perform operation
    let result = chain::call(chain_id, address, "mint", { "amount": amount });
    
    log::audit("sensitive_op_result", {
        "result": result,
        "tx_hash": tx_hash
    });
}
```

## Performance and Optimization

### Gas Optimization
```rust
fn optimized_transfer() {
    // Choose cheapest chain
    let cheapest_chain = find_cheapest_chain();
    
    // Batch operations
    let batch_transfers = [transfer1, transfer2, transfer3];
    let batch_result = chain::call(cheapest_chain, address, "batchTransfer", {
        "transfers": batch_transfers
    });
}
```

### Memory Management
```rust
fn memory_efficient_operation() {
    // Use references where possible
    let data = get_large_dataset();
    
    // Process in chunks
    for chunk in data.chunks(1000) {
        process_chunk(chunk);
    }
    
    // Explicit cleanup
    drop(data);
}
```

### Caching Strategies
```rust
service CachedService {
    cache: map<string, value>,
    
    fn get_cached_data(key: string) -> value {
        if self.cache.contains(key) {
            return self.cache[key];
        }
        
        let data = fetch_expensive_data(key);
        self.cache[key] = data;
        return data;
    }
}
```

## Integration Guide

### Building from Source
```bash
# Clone repository
git clone https://github.com/yourusername/dist_agent_lang.git
cd dist_agent_lang

# Build project
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Adding New Features
1. **Lexer**: Add tokens in `src/lexer/tokens.rs`
2. **Parser**: Add AST nodes in `src/parser/ast.rs`
3. **Runtime**: Add execution logic in `src/runtime/engine.rs`
4. **Standard Library**: Add functions in `src/stdlib/`

### Error Handling Best Practices
```rust
// Use try-catch for risky operations
try {
    let result = chain::call(chain_id, address, function, args);
    return result;
} catch error {
    log::error("chain_call", format!("Failed: {}", error));
    return "error";
}

// Validate inputs
fn safe_function(input: string) -> bool {
    if input.length() == 0 {
        return false;
    }
    
    // Perform operation
    return true;
}
```

### Testing Best Practices
```rust
@test
service ComprehensiveTests {
    fn test_happy_path() -> bool {
        // Test normal operation
        let service = MyService::new();
        return service.basic_operation() == expected_result;
    }
    
    fn test_error_conditions() -> bool {
        // Test error handling
        try {
            service.invalid_operation();
            return false;
        } catch error {
            return true;
        }
    }
    
    fn test_edge_cases() -> bool {
        // Test boundary conditions
        let result = service.operation_with_edge_case();
        return result.is_valid();
    }
}
```

---

This documentation provides a comprehensive overview of dist_agent_lang's capabilities, architecture, and usage patterns. For more specific examples and advanced usage, refer to the individual example files in the `examples/` directory.

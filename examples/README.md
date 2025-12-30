# dist_agent_lang Examples

This directory contains comprehensive examples demonstrating the features and capabilities of the `dist_agent_lang` programming language.

## 📁 Example Files

### 01_hello_world.dal
**Difficulty**: Beginner  
**Features**: Basic syntax, variables, functions, logging

A simple "Hello World" program that demonstrates:
- Basic variable assignment
- Function calls
- Logging with structured data
- Built-in functions (`print`, `add`)

**Run**: `cargo run --example hello_world`

### 02_nft_marketplace.dal
**Difficulty**: Intermediate  
**Features**: Blockchain operations, attributes, complex data structures

A complete NFT marketplace implementation showcasing:
- `@trust(hybrid)` service definition
- `@secure` and `@txn` attributes
- `@limit(n)` resource constraints
- `chain::` namespace operations (mint, update, get)
- Complex data structures (structs, maps, vectors)
- Comprehensive audit logging

**Key Concepts**:
- Service-oriented architecture
- Blockchain integration
- Security attributes
- Data validation
- Transaction management

**Run**: `cargo run --example nft_marketplace`

### 03_trading_bot.dal
**Difficulty**: Advanced  
**Features**: Async/await, agent system, oracle integration, AI services

An AI-powered trading bot demonstrating:
- `spawn agent` for concurrent execution
- `async`/`await` for asynchronous operations
- `oracle::` namespace for external data
- `service::` namespace for AI integration
- Complex agent lifecycle management
- Risk management and decision making

**Key Concepts**:
- Agent-based programming
- Asynchronous execution
- External data integration
- AI service integration
- Risk management
- Event-driven architecture

**Run**: `cargo run --example trading_bot`

### 04_error_handling.dal
**Difficulty**: Intermediate  
**Features**: Try-catch blocks, error recovery, comprehensive error management

A comprehensive error handling demonstration featuring:
- `try`/`catch`/`finally` blocks
- `throw` expressions
- `Result<T, E>` types
- Error logging and analysis
- Transaction rollback mechanisms
- Batch processing with error handling

**Key Concepts**:
- Structured error handling
- Error recovery strategies
- Transaction safety
- Error analysis and reporting
- Resource cleanup

**Run**: `cargo run --example error_handling`

## 🚀 Running Examples

### Prerequisites
- Rust installed (1.70+)
- Cargo package manager
- Clone the repository

### Basic Setup
```bash
# Clone the repository
git clone <repository-url>
cd dist_agent_lang

# Build the project
cargo build

# Run examples
cargo run --example hello_world
cargo run --example nft_marketplace
cargo run --example trading_bot
cargo run --example error_handling
```

### Running with Custom Input
```bash
# Run with specific parameters
cargo run --example nft_marketplace -- --user-id alice --nft-name "My NFT"
```

## 📚 Learning Path

### For Beginners
1. Start with `01_hello_world.dal` to understand basic syntax
2. Study the variable declarations and function calls
3. Experiment with different data types

### For Intermediate Developers
1. Examine `02_nft_marketplace.dal` for service architecture
2. Understand attribute usage (`@secure`, `@txn`, `@limit`)
3. Study blockchain integration patterns
4. Learn about structured logging

### For Advanced Developers
1. Dive into `03_trading_bot.dal` for async programming
2. Understand agent system and concurrency
3. Study external service integration
4. Learn about complex state management

### For All Developers
1. Review `04_error_handling.dal` for robust error management
2. Understand transaction safety
3. Learn about error recovery strategies

## 🔧 Customizing Examples

### Adding New Features
Each example can be extended with additional features:

```rust
// Add new attributes
@secure
@limit(1000)
fn new_function() {
    // Your code here
}

// Add new standard library calls
let result = oracle::fetch("new_feed", query);
let ai_response = service::ai("New prompt", ai_service);
```

### Modifying Parameters
Adjust example parameters to test different scenarios:

```rust
// In trading bot example
let agent = await create_trading_agent("user1", "aggressive", 50000);

// In NFT marketplace example
let nft = create_nft("Rare NFT", "Very rare digital art", "artist1");
```

## 🧪 Testing Examples

### Unit Testing
Each example includes testable components:

```rust
test "nft_creation" {
    let nft_id = create_nft("Test NFT", "Description", "creator");
    assert_gt(nft_id, 0);
}

test "transfer_validation" {
    let result = transfer_money("alice", "bob", 100);
    assert_eq(result.is_ok(), true);
}
```

### Integration Testing
Test complete workflows:

```rust
test_suite "marketplace_integration" {
    test "complete_nft_lifecycle" {
        // Create user
        // Create NFT
        // List NFT
        // Buy NFT
        // Verify ownership
    }
}
```

## 📖 Example Patterns

### Service Definition Pattern
```rust
@trust(hybrid)
service MyService {
    users: map<string, User>,
    transactions: vector<Transaction>
}
```

### Attribute Usage Pattern
```rust
@secure
@txn
@limit(100)
fn secure_function() -> Result<T, E> {
    // Implementation
}
```

### Agent Pattern
```rust
let worker = spawn agent {
    agent_loop(agent_id);
};

async fn agent_loop(id: int) {
    while active {
        // Agent logic
        await sleep(interval);
    }
}
```

### Error Handling Pattern
```rust
fn safe_operation() -> Result<T, E> {
    try {
        // Risky operation
        Ok(result)
    } catch (error) {
        log::error("Operation failed", { "error": error });
        Err(error)
    } finally {
        cleanup();
    }
}
```

## 🎯 Best Practices

### Code Organization
- Use clear, descriptive function names
- Group related functionality together
- Add comprehensive comments
- Use structured logging

### Error Handling
- Always use try-catch for risky operations
- Log errors with context
- Provide meaningful error messages
- Implement rollback mechanisms

### Performance
- Use appropriate resource limits
- Implement efficient data structures
- Monitor performance with profiling
- Use object pooling for frequent operations

### Security
- Validate all inputs
- Use appropriate security attributes
- Implement proper access controls
- Audit all critical operations

## 🔗 Related Documentation

- [Language Reference](../Documentation.md)
- [API Documentation](../docs/api.md)
- [Tutorial Series](../docs/tutorials.md)
- [Best Practices](../docs/best-practices.md)

## 🤝 Contributing

To add new examples:

1. Create a new `.dal` file with descriptive name
2. Include comprehensive comments
3. Add corresponding tests
4. Update this README
5. Follow the established patterns

## 📄 License

These examples are part of the `dist_agent_lang` project and follow the same license terms.

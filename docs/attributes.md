# dist_agent_lang Attributes Reference

Complete reference for all attributes in dist_agent_lang.

## Overview

Attributes in dist_agent_lang are annotations that modify the behavior of services, functions, and other language constructs. They use the `@` prefix.

## Service-Level Attributes

### `@trust(model)`

Specifies the trust model for a service.

**Values:**
- `"decentralized"` - Fully decentralized, no central authority
- `"hybrid"` - Combines centralized and decentralized trust
- `"centralized"` - Traditional centralized trust model

**Example:**
```rust
@trust("hybrid")
service MyService {
    // Service code
}
```

### `@chain(chain1, chain2, ...)`

Specifies which blockchain networks the service supports.

**Supported Chains:**
- `"ethereum"` - Ethereum Mainnet (Chain ID: 1)
- `"polygon"` - Polygon (Chain ID: 137)
- `"binance"` - Binance Smart Chain (Chain ID: 56)
- `"solana"` - Solana (Chain ID: 101)
- `"avalanche"` - Avalanche (Chain ID: 43114)
- `"arbitrum"` - Arbitrum (Chain ID: 42161)
- `"optimism"` - Optimism (Chain ID: 10)

**Example:**
```rust
@chain("ethereum", "polygon")
service MultiChainService {
    // Works on both Ethereum and Polygon
}
```

### `@compile_target(target)`

Specifies the compilation target for the service.

**Values:**
- `"blockchain"` - Compile for blockchain deployment
- `"webassembly"` - Compile to WebAssembly (also accepted: `"wasm"`)
- `"native"` - Native binary compilation
- `"mobile"` - Mobile app compilation
- `"edge"` - Edge computing devices

When `@compile_target` is present, the parser enforces **required attributes** and **forbidden operations** for that target. See [Compile targets (constraints)](#compile-targets-constraints) below.

**Example:**
```rust
@compile_target("blockchain")
@secure
@trust("hybrid")
service SmartContract {
    // Compiled as smart contract; @secure and @trust required for blockchain
}
```

#### Compile targets (constraints)

Each target defines required attributes (must appear on the service) and forbidden operations (stdlib namespaces that must not be used in service methods). The parser validates these at parse time; the runtime may re-check required attributes when the service is instantiated.

| Target | Required attributes | Forbidden operations (namespaces / ops) |
|--------|---------------------|-----------------------------------------|
| **blockchain** | `@secure`, `@trust` | `web::http_request`, `web::websocket`, `desktop::window`, `mobile::notification`, `iot::sensor_read` |
| **webassembly** | `@web` | `chain::transaction`, `chain::deploy`, `desktop::file_system`, `mobile::camera`, `iot::device_control` |
| **native** | `@native` | `chain::transaction`, `mobile::touch_event`, `iot::sensor_read` |
| **mobile** | `@mobile` | `chain::transaction`, `desktop::window`, `iot::device_control` |
| **edge** | `@edge` | `chain::transaction`, `web::dom_manipulation`, `desktop::window`, `mobile::camera` |

Allowed operations per target are defined in the implementation (`get_target_constraints()`); only the forbidden set and required attributes are listed here. If you use `@compile_target`, you must include the required attributes and avoid calling forbidden namespaces in any service method.

### `@interface(language)`

Generates client interface in the specified language.

**Supported Languages:**
- `"typescript"` - TypeScript interface
- `"javascript"` - JavaScript interface
- `"python"` - Python interface
- `"rust"` - Rust interface
- `"java"` - Java interface
- `"go"` - Go interface

**Example:**
```rust
@interface("typescript")
service APIService {
    // Generates TypeScript client interface
}
```

### `@secure`

Enforces security requirements on the service.

**Effects:**
- Requires authentication for all operations
- Enables audit logging
- Enforces capability checks

**Example:**
```rust
@secure
@trust("hybrid")
service SecureService {
    // All operations are secured
}
```

### `@limit(n)`

Sets resource limits for the service.

**Parameters:**
- `n` - Maximum number of operations/resources

**Example:**
```rust
@limit(1000)
service LimitedService {
    // Maximum 1000 operations
}
```

### `@txn`

Wraps operations in a transaction.

**Effects:**
- Operations are atomic
- Rollback on error
- Transaction logging

**Example:**
```rust
@txn
fn transfer_funds() {
    // Transaction-wrapped operation
}
```

## Function-Level Attributes

### `@secure`

Marks a function as requiring security checks.

```rust
@secure
fn admin_operation() {
    // Requires authentication and authorization
}
```

### `@txn`

Wraps function execution in a transaction.

```rust
@txn
fn update_balance() {
    // Executed as transaction
}
```

### `@limit(n)`

Sets resource limit for a function.

```rust
@limit(100)
fn process_batch() {
    // Limited to 100 operations
}
```

## Platform-Specific Attributes

### `@mobile`

Indicates mobile platform support.

```rust
@mobile
service MobileApp {
    // Mobile-specific features
}
```

### `@desktop`

Indicates desktop platform support.

```rust
@desktop
service DesktopApp {
    // Desktop-specific features
}
```

### `@iot`

Indicates IoT/edge device support.

```rust
@iot
service IoTDevice {
    // IoT-specific features
}
```

## AI-Specific Attributes

### `@ai`

Enables AI agent capabilities.

```rust
@ai
service AIService {
    // AI agent features enabled
}
```

## Caching Attributes

### `@cached`

Enables caching for a function.

```rust
@cached
fn expensive_operation() -> string {
    // Results are cached
    return compute_expensive_result();
}
```

### `@persistent`

Marks data as persistent.

```rust
@persistent
service PersistentService {
    // Data persists across restarts
}
```

## Versioning Attributes

### `@versioned`

Enables versioning for a service.

```rust
@versioned
service VersionedService {
    // Service versioning enabled
}
```

### `@deprecated`

Marks a function or service as deprecated.

```rust
@deprecated
fn old_function() {
    // This function is deprecated
}
```

## Attribute Combinations

Attributes can be combined:

```rust
@trust("hybrid")
@chain("ethereum", "polygon")
@secure
@compile_target("blockchain")
@interface("typescript")
service CompleteService {
    @secure
    @txn
    @limit(1000)
    fn secure_transaction() {
        // Multiple attributes applied
    }
}
```

## Attribute Inheritance

Service-level attributes apply to all functions unless overridden:

```rust
@secure
service SecureService {
    // All functions inherit @secure
    
    fn public_function() {
        // Still requires security
    }
    
    // Can override at function level if needed
}
```

## Best Practices

1. **Use `@trust` appropriately**: Choose the right trust model for your use case
2. **Specify `@chain` explicitly**: Always specify which chains your service supports
3. **Use `@secure` for sensitive operations**: Mark all security-critical functions
4. **Combine attributes wisely**: Use multiple attributes when needed
5. **Document attribute usage**: Explain why specific attributes are used

## Examples

### DeFi Service

```rust
@trust("decentralized")
@chain("ethereum", "polygon")
@secure
@compile_target("blockchain")
service DeFiService {
    @txn
    @limit(10000)
    fn swap_tokens() {
        // Secure, transactional token swap
    }
}
```

### AI Agent Service

```rust
@trust("hybrid")
@ai
@compile_target("native")
service AIAgentService {
    @cached
    fn analyze_data(data: string) -> map<string, any> {
        // Cached AI analysis
    }
}
```

### Web API Service

```rust
@trust("centralized")
@compile_target("webassembly")
@interface("typescript", "python")
service WebAPIService {
    @secure
    fn get_user_data() -> map<string, any> {
        // Secure API endpoint
    }
}
```

---

**See also:** [Syntax Reference](syntax.md) | [API Reference](api_reference.md)


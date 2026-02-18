# Auto-Detection Guide: When @api is Not Specified

## Overview

When you don't specify the `@api` attribute, dist_agent_lang **automatically detects** which interface (HTTP or FFI) to use based on intelligent heuristics. This provides the best of both worlds - performance when needed, flexibility when required.

## How Auto-Detection Works

### 1. Function Name Pattern Analysis

The system analyzes function names to detect operation types:

#### FFI-Preferred Patterns (Compute-Intensive)
- `hash*`, `sign*`, `verify*`, `encrypt*`, `decrypt*`
- `compute*`, `calculate*`, `process*`, `transform*`
- `batch_*`, `parallel_*`, `fast_*`
- `crypto::*`

**Example:**
```rust
// Auto-detects FFI (contains "hash")
fn hash_data(data: string) -> string {
    return crypto::hash(data, "SHA256");
}
```

#### HTTP-Preferred Patterns (Network Operations)
- `chain::*`, `database::*`, `network_*`, `remote_*`
- `fetch*`, `request*`, `api_*`, `http_*`, `web_*`

**Example:**
```rust
// Auto-detects HTTP (contains "chain::")
fn get_balance(chain_id: int, address: string) -> int {
    return chain::get_balance(chain_id, address);
}
```

### 2. Argument Size Analysis

- **Small arguments (< 1KB)**: Prefers FFI (low serialization overhead)
- **Large arguments (> 1KB)**: Prefers HTTP (better for large data transfer)

### 3. Operation Type Detection

- **Compute-bound operations**: Auto-selects FFI
- **Network-bound operations**: Auto-selects HTTP
- **Mixed operations**: Uses "both" and selects per call

### 4. Call Frequency Estimation

- **High frequency (> 1000 calls/sec)**: Prefers FFI
- **Low frequency (< 10 calls/sec)**: Prefers HTTP
- **Medium frequency**: Uses "both" with smart routing

## Examples

### Example 1: Auto-Detection Based on Function Name

```rust
@trust("hybrid")
// No @api attribute - auto-detects
service SmartService {
    // Auto-detects FFI (contains "hash")
    fn hash_data(data: string) -> string {
        return crypto::hash(data, "SHA256");
    }
    
    // Auto-detects HTTP (contains "chain::")
    fn get_balance(chain_id: int, address: string) -> int {
        return chain::get_balance(chain_id, address);
    }
    
    // Auto-detects FFI (contains "batch_")
    fn batch_process(data_list: vector<string>) -> vector<string> {
        let results = [];
        for data in data_list {
            results.push(self.hash_data(data));
        }
        return results;
    }
}
```

### Example 2: Mixed Operations

```rust
@trust("hybrid")
service MixedService {
    // Mixed operation - uses "both" and auto-selects per call
    fn process_and_store(data: string) -> map<string, any> {
        // Hash (compute) - auto-selects FFI
        let hash = crypto::hash(data, "SHA256");
        
        // Store (network) - auto-selects HTTP
        let stored = database::store("hashes", {"hash": hash});
        
        return stored;
    }
}
```

### Example 3: Override Auto-Detection

```rust
@trust("hybrid")
service OverrideService {
    // Auto-detects FFI, but you can override
    @api("http")  // Force HTTP for this function
    fn hash_data(data: string) -> string {
        return crypto::hash(data, "SHA256");
    }
    
    // Auto-detects HTTP, but you can override
    @api("ffi")  // Force FFI for this function
    fn get_balance(chain_id: int, address: string) -> int {
        return chain::get_balance(chain_id, address);
    }
}
```

## Auto-Detection Rules

### Rule 1: High-Frequency Operations
If a function is called frequently (> 1000 times/second), auto-selects FFI.

### Rule 2: Network Operations
If a function uses network operations (`chain::`, `database::`, `web::`), auto-selects HTTP.

### Rule 3: Compute Operations
If a function uses compute-intensive operations (`crypto::`, `hash`, `sign`), auto-selects FFI.

### Rule 4: Mixed Operations
If a function has both network and compute operations, uses "both" and auto-selects per call.

### Rule 5: Default Behavior
If no clear pattern is detected, defaults to "both" with smart per-call selection.

## Runtime Behavior

### When Both Interfaces Are Available

```python
# Python example
import dist_agent_lang

runtime = dist_agent_lang.DistAgentLangRuntime()

# Auto-selects FFI (compute operation)
result1 = runtime.call_function("SmartService", "hash_data", ["data"])

# Auto-selects HTTP (network operation)
result2 = runtime.call_function("SmartService", "get_balance", [1, "0x123..."])

# System automatically chooses the best interface
```

### Fallback Mechanism

If the auto-selected interface fails, the system automatically falls back:

1. **FFI fails** → Falls back to HTTP
2. **HTTP fails** → Falls back to FFI (if available)
3. **Both fail** → Returns error

## Configuration

### Global Default

You can set a global default interface:

```rust
use dist_agent_lang::ffi::{FFIConfig, InterfaceType};

// Set default to auto-detect (default behavior)
let config = FFIConfig::auto_detect();

// Or set explicit default
let config = FFIConfig::both();  // Use both, auto-select
let config = FFIConfig::http_only();  // Always HTTP
let config = FFIConfig::ffi_only();  // Always FFI
```

### Service-Level Metadata

You can register service metadata for better detection:

```rust
use dist_agent_lang::ffi::{InterfaceSelector, ServiceMetadata, CallFrequency};

let mut selector = InterfaceSelector::new();

selector.register_service(ServiceMetadata {
    name: "CryptoService".to_string(),
    function_names: vec!["hash".to_string(), "sign".to_string()],
    has_network_operations: false,
    has_compute_operations: true,
    estimated_call_frequency: CallFrequency::High,
});
```

## Best Practices

### 1. Trust Auto-Detection
The auto-detection is quite smart. Only override when you have a specific reason.

### 2. Use Explicit @api for Critical Paths
For performance-critical functions, explicitly specify `@api("ffi")` to ensure optimal performance.

### 3. Use @api("http") for Distributed Operations
For functions that must work across networks, explicitly specify `@api("http")`.

### 4. Monitor Performance
Profile your application to see if auto-detection is making good choices. Adjust if needed.

### 5. Use "both" for Flexibility
If you're unsure, use `@api("both")` to let the system choose per call.

## Troubleshooting

### Auto-Detection Choosing Wrong Interface

If auto-detection is not choosing the right interface:

1. **Explicitly specify**: Use `@api("http")` or `@api("ffi")`
2. **Check function name**: Rename to match expected patterns
3. **Register metadata**: Provide service metadata for better detection

### Performance Issues

If you're experiencing performance issues:

1. **Check interface selection**: Verify auto-detection is choosing correctly
2. **Profile calls**: Measure actual performance
3. **Override if needed**: Explicitly specify the better interface

## Summary

- **No @api attribute**: System auto-detects based on heuristics
- **Function patterns**: Analyzes function names for operation type
- **Smart routing**: Chooses best interface per call
- **Automatic fallback**: Falls back if selected interface fails
- **Override available**: Can explicitly specify if needed

Auto-detection provides the best balance of performance and flexibility without requiring explicit configuration.

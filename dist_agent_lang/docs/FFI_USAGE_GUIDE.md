# FFI (Foreign Function Interface) Usage Guide

## Overview

dist_agent_lang supports **both HTTP/REST and FFI interfaces**, allowing users to choose the best option for their use case:

- **HTTP/REST**: Best for distributed systems, microservices, cross-language compatibility
- **FFI**: Best for high-performance, low-latency, local function calls

## Quick Start

### Choosing the Interface

Use the `@api` attribute to specify which interface to use:

```rust
// HTTP only - for microservices
@api("http")
service MyService {
    fn process(data: string) -> string;
}

// FFI only - for high performance
@api("ffi")
service MyService {
    fn process(data: string) -> string;
}

// Both - flexible choice
@api("both")
service MyService {
    fn process(data: string) -> string;
}
```

## Python FFI Integration

### Installation

```bash
# Build with Python FFI support
cargo build --features python-ffi

# Install Python module
pip install dist_agent_lang
```

### Usage

```python
import dist_agent_lang

# Create runtime
runtime = dist_agent_lang.DistAgentLangRuntime()

# Direct function calls (FFI)
result = runtime.call_function(
    "CryptoService",
    "hash_data",
    ["Hello, World!"]
)

# High-performance crypto operations
hash_result = dist_agent_lang.hash_data(b"data", "SHA256")
signature = dist_agent_lang.sign_data(b"data", private_key)
is_valid = dist_agent_lang.verify_signature(b"data", signature, public_key)
```

### Performance

- **FFI**: ~1-5µs per call
- **HTTP**: ~500-2000µs per call
- **Speedup**: 100-1000x faster with FFI

## Rust FFI Integration

### Usage

```rust
use dist_agent_lang::ffi::rust::RustFFIRuntime;
use dist_agent_lang::runtime::values::Value;

let mut runtime = RustFFIRuntime::new();

// Direct function calls (zero overhead)
let args = vec![Value::String("Hello".to_string())];
let result = runtime.call_function("MyService", "process", &args)?;
```

### C-Compatible FFI

```rust
use std::ffi::CString;

let data = b"Hello, World!";
let mut output = vec![0u8; 64];

unsafe {
    dist_agent_lang::ffi::rust::dist_agent_lang_hash(
        data.as_ptr(),
        data.len(),
        CString::new("SHA256").unwrap().as_ptr(),
        output.as_mut_ptr() as *mut i8,
        output.len(),
    );
}
```

## HTTP/REST Integration

### Usage

```python
import requests

# HTTP API call
response = requests.post(
    "http://localhost:3000/api/MyService/process",
    json=["Hello, World!"]
)
result = response.json()
```

```javascript
// JavaScript/Node.js
const response = await fetch('http://localhost:3000/api/MyService/process', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(["Hello, World!"])
});
const result = await response.json();
```

## When to Use Each Interface

### Use HTTP/REST When:
- ✅ Building microservices
- ✅ Cross-network communication
- ✅ Language-agnostic access needed
- ✅ Easy deployment and scaling
- ✅ Network-bound operations (chain calls, database)

### Use FFI When:
- ✅ High-frequency operations (>1000 calls/sec)
- ✅ Low-latency requirements (<1ms)
- ✅ Local function calls
- ✅ Maximum performance needed
- ✅ Compute-bound operations (crypto, math)

### Use Both When:
- ✅ Need flexibility
- ✅ Different functions have different requirements
- ✅ Want automatic fallback

## Performance Comparison

| Operation | HTTP/REST | FFI | Speedup |
|-----------|-----------|-----|---------|
| Single hash | ~500µs | ~5µs | 100x |
| 1M hashes | ~500s | ~5s | 100x |
| Crypto sign | ~1ms | ~100µs | 10x |
| Chain call | ~10ms | ~10ms | 1x (network bound) |

## Hybrid Approach

You can use both interfaces in the same application:

```python
# High-frequency operations use FFI
for i in range(1_000_000):
    hash_result = dist_agent_lang.hash_data(data, "SHA256")  # FFI

# Distributed operations use HTTP
response = requests.post(
    "http://api.example.com/chain/call",
    json=tx_data
)  # HTTP
```

## Configuration

### Service-Level Configuration

```rust
@api("both")  // Both HTTP and FFI available
@trust("hybrid")
service FlexibleService {
    // Available via both interfaces
    fn process(data: string) -> string;
    
    @api("ffi")  // Function-level override - prefer FFI
    fn high_frequency(data: string) -> string;
    
    @api("http")  // Function-level override - HTTP only
    fn distributed(data: string) -> string;
}
```

### Runtime Configuration

```rust
use dist_agent_lang::ffi::{FFIConfig, FFIInterface, InterfaceType};

// HTTP only
let config = FFIConfig::http_only();
let interface = FFIInterface::new(config);

// FFI only
let config = FFIConfig::ffi_only();
let interface = FFIInterface::new(config);

// Both (default)
let config = FFIConfig::both();
let interface = FFIInterface::new(config);
```

## Best Practices

1. **Use FFI for compute-bound operations**: Crypto, hashing, math
2. **Use HTTP for network-bound operations**: Chain calls, database queries
3. **Profile your application**: Measure actual performance to decide
4. **Consider deployment**: FFI requires native libraries, HTTP is universal
5. **Hybrid approach**: Use both - FFI for hot paths, HTTP for everything else

## Troubleshooting

### FFI Not Available

If FFI is not available, the system automatically falls back to HTTP:

```python
try:
    result = runtime.call_function(...)  # Try FFI
except ImportError:
    result = call_via_http(...)  # Fallback to HTTP
```

### Build Issues

```bash
# Python FFI requires PyO3
cargo build --features python-ffi

# C FFI is always available
cargo build --features c-ffi
```

## Examples

See the examples directory:
- `examples/http_vs_ffi_example.dal` - Service definitions
- `examples/python_ffi_example.py` - Python integration
- `examples/rust_ffi_example.rs` - Rust integration

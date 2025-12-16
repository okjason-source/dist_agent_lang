#!/usr/bin/env python3
"""
Python FFI Example - High-Performance Integration
Demonstrates using dist_agent_lang via FFI for maximum performance
"""

# Example 1: Direct FFI calls (fastest)
try:
    import dist_agent_lang  # PyO3 module
    
    # Create runtime
    runtime = dist_agent_lang.DistAgentLangRuntime()
    
    # Direct function calls - zero HTTP overhead
    # ~100-1000x faster than HTTP for high-frequency operations
    
    # Hash data (FFI call)
    data = b"Hello, World!"
    hash_result = dist_agent_lang.hash_data(data, "SHA256")
    print(f"Hash: {hash_result}")
    
    # Sign data (FFI call)
    private_key = b"your_private_key_here"
    signature = dist_agent_lang.sign_data(data, private_key)
    print(f"Signature: {signature.hex()}")
    
    # Verify signature (FFI call)
    public_key = b"your_public_key_here"
    is_valid = dist_agent_lang.verify_signature(data, signature, public_key)
    print(f"Valid: {is_valid}")
    
    # Call service function (FFI call)
    result = runtime.call_function(
        "CryptoService",
        "batch_hash",
        [["data1", "data2", "data3"]]
    )
    print(f"Batch hash results: {result}")
    
except ImportError:
    print("FFI module not available, falling back to HTTP")

# Example 2: HTTP fallback (when FFI not available)
import requests
import json

def call_via_http(service_name, function_name, args):
    """Fallback to HTTP when FFI not available"""
    url = f"http://localhost:3000/api/{service_name}/{function_name}"
    response = requests.post(url, json=args)
    return response.json()

# Example 3: Hybrid approach - try FFI first, fallback to HTTP
def call_service(service_name, function_name, args, prefer_ffi=True):
    """Smart routing: FFI if available, HTTP otherwise"""
    if prefer_ffi:
        try:
            runtime = dist_agent_lang.DistAgentLangRuntime()
            return runtime.call_function(service_name, function_name, args)
        except (ImportError, AttributeError):
            pass
    
    # Fallback to HTTP
    return call_via_http(service_name, function_name, args)

# Example 4: Performance comparison
import time

def benchmark_ffi_vs_http():
    """Compare FFI vs HTTP performance"""
    data = b"test_data" * 1000
    iterations = 10000
    
    # FFI benchmark
    try:
        start = time.time()
        for _ in range(iterations):
            dist_agent_lang.hash_data(data, "SHA256")
        ffi_time = time.time() - start
        print(f"FFI: {ffi_time:.4f}s ({iterations/ffi_time:.0f} ops/sec)")
    except ImportError:
        print("FFI not available")
        ffi_time = None
    
    # HTTP benchmark
    start = time.time()
    for _ in range(iterations):
        call_via_http("CryptoService", "hash_data", [data.decode()])
    http_time = time.time() - start
    print(f"HTTP: {http_time:.4f}s ({iterations/http_time:.0f} ops/sec)")
    
    if ffi_time:
        speedup = http_time / ffi_time
        print(f"Speedup: {speedup:.1f}x faster with FFI")

if __name__ == "__main__":
    print("=== Python FFI Example ===")
    benchmark_ffi_vs_http()

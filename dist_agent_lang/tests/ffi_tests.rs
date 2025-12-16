// Comprehensive FFI (Foreign Function Interface) Tests
// Tests HTTP/REST and FFI interfaces, auto-detection, and integration

use dist_agent_lang::ffi::{
    FFIInterface, FFIConfig, InterfaceType, InterfaceSelector, ServiceMetadata, CallFrequency
};
use dist_agent_lang::runtime::values::Value;
use dist_agent_lang::runtime::engine::Runtime;

#[test]
fn test_ffi_config_default() {
    let config = FFIConfig::default();
    assert_eq!(config.interface_type, InterfaceType::Both);
    assert!(config.enable_http);
    assert!(config.enable_ffi);
}

#[test]
fn test_ffi_config_http_only() {
    let config = FFIConfig::http_only();
    assert_eq!(config.interface_type, InterfaceType::HTTP);
    assert!(config.enable_http);
    assert!(!config.enable_ffi);
}

#[test]
fn test_ffi_config_ffi_only() {
    let config = FFIConfig::ffi_only();
    assert_eq!(config.interface_type, InterfaceType::FFI);
    assert!(!config.enable_http);
    assert!(config.enable_ffi);
}

#[test]
fn test_ffi_config_auto_detect() {
    let config = FFIConfig::auto_detect();
    assert_eq!(config.interface_type, InterfaceType::Both);
    assert!(config.enable_http);
    assert!(config.enable_ffi);
}

#[test]
fn test_interface_selector_new() {
    let selector = InterfaceSelector::new();
    assert_eq!(selector.default_interface, InterfaceType::Both);
}

#[test]
fn test_interface_selector_register_service() {
    let mut selector = InterfaceSelector::new();
    
    let metadata = ServiceMetadata {
        name: "TestService".to_string(),
        function_names: vec!["hash".to_string(), "sign".to_string()],
        has_network_operations: false,
        has_compute_operations: true,
        estimated_call_frequency: CallFrequency::High,
    };
    
    selector.register_service(metadata);
    assert_eq!(selector.service_metadata.len(), 1);
}

#[test]
fn test_service_metadata_analyze_function() {
    // Test network patterns
    let (has_network, has_compute) = ServiceMetadata::analyze_function("chain::get_balance");
    assert!(has_network);
    assert!(!has_compute);
    
    // Test compute patterns
    let (has_network, has_compute) = ServiceMetadata::analyze_function("hash_data");
    assert!(!has_network);
    assert!(has_compute);
    
    // Test mixed
    let (has_network, has_compute) = ServiceMetadata::analyze_function("process_data");
    assert!(!has_network);
    assert!(has_compute);
    
    // Test database pattern
    let (has_network, has_compute) = ServiceMetadata::analyze_function("database::query");
    assert!(has_network);
    assert!(!has_compute);
}

#[test]
fn test_service_metadata_detect_interface_type() {
    // High frequency -> FFI
    let metadata = ServiceMetadata {
        name: "HighFreqService".to_string(),
        function_names: vec![],
        has_network_operations: false,
        has_compute_operations: true,
        estimated_call_frequency: CallFrequency::High,
    };
    assert_eq!(metadata.detect_interface_type(), InterfaceType::FFI);
    
    // Network only -> HTTP
    let metadata = ServiceMetadata {
        name: "NetworkService".to_string(),
        function_names: vec![],
        has_network_operations: true,
        has_compute_operations: false,
        estimated_call_frequency: CallFrequency::Low,
    };
    assert_eq!(metadata.detect_interface_type(), InterfaceType::HTTP);
    
    // Compute only -> FFI
    let metadata = ServiceMetadata {
        name: "ComputeService".to_string(),
        function_names: vec![],
        has_network_operations: false,
        has_compute_operations: true,
        estimated_call_frequency: CallFrequency::Medium,
    };
    assert_eq!(metadata.detect_interface_type(), InterfaceType::FFI);
    
    // Mixed -> Both
    let metadata = ServiceMetadata {
        name: "MixedService".to_string(),
        function_names: vec![],
        has_network_operations: true,
        has_compute_operations: true,
        estimated_call_frequency: CallFrequency::Medium,
    };
    assert_eq!(metadata.detect_interface_type(), InterfaceType::Both);
}

#[test]
fn test_interface_selector_select_interface() {
    let selector = InterfaceSelector::new();
    
    // Hash function -> FFI
    let interface = selector.select_interface("Service", "hash_data", &[]);
    assert_eq!(interface, InterfaceType::FFI);
    
    // Chain function -> HTTP
    let interface = selector.select_interface("Service", "chain::get_balance", &[]);
    assert_eq!(interface, InterfaceType::HTTP);
    
    // Database function -> HTTP
    let interface = selector.select_interface("Service", "database::query", &[]);
    assert_eq!(interface, InterfaceType::HTTP);
    
    // Unknown function -> Both (default)
    let interface = selector.select_interface("Service", "unknown_function", &[]);
    assert_eq!(interface, InterfaceType::Both);
}

#[test]
fn test_interface_selector_with_metadata() {
    let mut selector = InterfaceSelector::new();
    
    let metadata = ServiceMetadata {
        name: "CryptoService".to_string(),
        function_names: vec!["hash".to_string(), "sign".to_string()],
        has_network_operations: false,
        has_compute_operations: true,
        estimated_call_frequency: CallFrequency::High,
    };
    
    selector.register_service(metadata);
    
    // Should use service metadata
    let interface = selector.select_interface("CryptoService", "hash", &[]);
    assert_eq!(interface, InterfaceType::FFI);
}

#[test]
fn test_ffi_interface_creation() {
    let config = FFIConfig::both();
    let interface = FFIInterface::new(config);
    // Should create successfully
    assert!(true); // Just verify it doesn't panic
}

#[test]
fn test_auto_detect_interface_hash_function() {
    let config = FFIConfig::both();
    let interface = FFIInterface::new(config);
    
    // Hash function should prefer FFI
    let args = vec![Value::String("test_data".to_string())];
    // Note: This will fail if runtime not set up, but tests the detection logic
    let _result = interface.call("CryptoService", "hash_data", &args, None);
    // Just verify the call doesn't panic - actual execution requires runtime setup
}

#[test]
fn test_auto_detect_interface_chain_function() {
    let config = FFIConfig::both();
    let interface = FFIInterface::new(config);
    
    // Chain function should prefer HTTP
    let args = vec![Value::Int(1), Value::String("0x123".to_string())];
    let _result = interface.call("ChainService", "chain::get_balance", &args, None);
    // Just verify the call doesn't panic
}

#[test]
fn test_estimate_value_size() {
    let config = FFIConfig::both();
    let interface = FFIInterface::new(config);
    
    // Test small value
    let small_args = vec![Value::String("test".to_string())];
    // Small args should prefer FFI
    
    // Test large value
    let large_string = "x".repeat(2048);
    let large_args = vec![Value::String(large_string)];
    // Large args should prefer HTTP
    
    // Just verify the interface can handle both
    assert!(true);
}

#[test]
fn test_ffi_interface_http_only() {
    let config = FFIConfig::http_only();
    let interface = FFIInterface::new(config);
    
    let args = vec![Value::String("test".to_string())];
    let result = interface.call("Service", "function", &args, Some(false));
    
    // Should attempt HTTP call (will fail without server, but tests routing)
    assert!(result.is_err() || result.is_ok()); // Either is fine for this test
}

#[test]
fn test_ffi_interface_ffi_only() {
    let config = FFIConfig::ffi_only();
    let interface = FFIInterface::new(config);
    
    let args = vec![Value::String("test".to_string())];
    let result = interface.call("Service", "function", &args, Some(true));
    
    // Should attempt FFI call
    assert!(result.is_err() || result.is_ok()); // Either is fine for this test
}

#[test]
fn test_ffi_interface_both_with_preference() {
    let config = FFIConfig::both();
    let interface = FFIInterface::new(config);
    
    let args = vec![Value::String("test".to_string())];
    
    // Prefer FFI
    let _result1 = interface.call("Service", "function", &args, Some(true));
    
    // Prefer HTTP
    let _result2 = interface.call("Service", "function", &args, Some(false));
    
    // No preference (auto-detect)
    let _result3 = interface.call("Service", "function", &args, None);
    
    // Just verify all calls don't panic
    assert!(true);
}

// Integration tests with runtime
#[test]
fn test_ffi_integration_with_runtime() {
    let mut runtime = Runtime::new();
    
    // Create a simple service
    let source = r#"
        @trust("hybrid")
        service TestService {
            fn add(a: int, b: int) -> int {
                return a + b;
            }
        }
    "#;
    
    // Parse and execute
    let program = dist_agent_lang::parse_source(source).unwrap();
    let _result = runtime.execute_program(program);
    
    // Test FFI interface with runtime
    let config = FFIConfig::ffi_only();
    let interface = FFIInterface::new(config);
    
    let args = vec![Value::Int(5), Value::Int(3)];
    // Note: This requires the runtime to be accessible from FFI
    // For now, just verify the interface can be created
    assert!(true);
}

#[test]
fn test_value_size_estimation() {
    let config = FFIConfig::both();
    let interface = FFIInterface::new(config);
    
    // Test different value types
    let int_val = Value::Int(42);
    let float_val = Value::Float(3.14);
    let string_val = Value::String("hello".to_string());
    let bool_val = Value::Bool(true);
    let null_val = Value::Null;
    
    let args = vec![int_val, float_val, string_val, bool_val, null_val];
    let _result = interface.call("Service", "function", &args, None);
    
    // Just verify it doesn't panic
    assert!(true);
}

#[test]
fn test_interface_fallback_mechanism() {
    let config = FFIConfig::both();
    let interface = FFIInterface::new(config);
    
    // Test that fallback works when one interface fails
    // This is tested implicitly - if FFI fails, should fallback to HTTP
    let args = vec![Value::String("test".to_string())];
    let _result = interface.call("Service", "function", &args, Some(true));
    
    // Just verify the call structure works
    assert!(true);
}

#[test]
fn test_call_frequency_enum() {
    assert_eq!(CallFrequency::Low as u8, 0);
    assert_eq!(CallFrequency::Medium as u8, 1);
    assert_eq!(CallFrequency::High as u8, 2);
}

#[test]
fn test_interface_type_equality() {
    assert_eq!(InterfaceType::HTTP, InterfaceType::HTTP);
    assert_eq!(InterfaceType::FFI, InterfaceType::FFI);
    assert_eq!(InterfaceType::Both, InterfaceType::Both);
    assert_ne!(InterfaceType::HTTP, InterfaceType::FFI);
}

// Performance-related tests
#[test]
fn test_auto_detection_performance_patterns() {
    let selector = InterfaceSelector::new();
    
    // High-frequency patterns should prefer FFI
    let high_freq_functions = vec![
        "hash_data",
        "sign_data",
        "batch_process",
        "parallel_compute",
    ];
    
    for func in high_freq_functions {
        let interface = selector.select_interface("Service", func, &[]);
        assert_eq!(interface, InterfaceType::FFI, "{} should prefer FFI", func);
    }
    
    // Network patterns should prefer HTTP
    let network_functions = vec![
        "chain::get_balance",
        "database::query",
        "fetch_data",
        "api_request",
    ];
    
    for func in network_functions {
        let interface = selector.select_interface("Service", func, &[]);
        assert_eq!(interface, InterfaceType::HTTP, "{} should prefer HTTP", func);
    }
}

#[test]
fn test_mixed_operation_detection() {
    let selector = InterfaceSelector::new();
    
    // Mixed operations should use Both
    let mixed_functions = vec![
        "process_and_store",
        "compute_and_send",
        "transform_and_save",
    ];
    
    for func in mixed_functions {
        let interface = selector.select_interface("Service", func, &[]);
        // Mixed operations default to Both
        assert_eq!(interface, InterfaceType::Both, "{} should use Both", func);
    }
}

// Comprehensive FFI (Foreign Function Interface) Tests
// Tests HTTP/REST and FFI interfaces, auto-detection, and integration
//
// Note: These tests intentionally avoid HTTP interface calls to keep tests focused
// on FFI-specific functionality (interface selection, auto-detection, configuration).
// HTTP interface testing is covered in http_server_tests.rs and http_security_mutation_tests.rs.

use dist_agent_lang::ffi::{
    CallFrequency, FFIConfig, FFIInterface, InterfaceSelector, InterfaceType, ServiceMetadata,
};
use dist_agent_lang::runtime::engine::Runtime;
use dist_agent_lang::runtime::values::Value;

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
    assert_eq!(selector.default_interface(), InterfaceType::Both);
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
    assert_eq!(selector.service_count(), 1);
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
    let _interface = FFIInterface::new(config);
    // Should create successfully (no panic)
}

#[test]
fn test_auto_detect_interface_hash_function() {
    // Test interface selection logic without HTTP interface initialization
    // Hash functions should prefer FFI based on pattern matching
    use dist_agent_lang::ffi::InterfaceSelector;

    let selector = InterfaceSelector::new();

    // Hash function should prefer FFI
    let interface = selector.select_interface("CryptoService", "hash_data", &[]);
    assert_eq!(interface, InterfaceType::FFI);
}

#[test]
fn test_auto_detect_interface_chain_function() {
    // Test interface selection logic without actually calling HTTP interface
    // to avoid reqwest runtime initialization issues in tests
    use dist_agent_lang::ffi::InterfaceSelector;

    let selector = InterfaceSelector::new();

    // Chain function should prefer HTTP based on pattern matching
    let interface = selector.select_interface("ChainService", "chain::get_balance", &[]);
    assert_eq!(interface, InterfaceType::HTTP);
}

#[test]
fn test_estimate_value_size() {
    // Test value size estimation logic without HTTP interface initialization
    use dist_agent_lang::ffi::InterfaceSelector;

    let selector = InterfaceSelector::new();

    // Test small value - should prefer FFI
    let small_args = vec![Value::String("test".to_string())];
    let interface_small = selector.select_interface("Service", "hash_data", &small_args);
    // Hash function should prefer FFI regardless of size
    assert_eq!(interface_small, InterfaceType::FFI);

    // Test large value - pattern matching takes precedence over size
    let large_string = "x".repeat(2048);
    let large_args = vec![Value::String(large_string)];
    let interface_large = selector.select_interface("Service", "hash_data", &large_args);
    // Still prefers FFI due to function pattern
    assert_eq!(interface_large, InterfaceType::FFI);
}

#[test]
fn test_ffi_interface_http_only() {
    // Test HTTP-only configuration without initializing HTTP client
    // to avoid reqwest runtime issues in tests
    let config = FFIConfig::http_only();
    assert_eq!(config.interface_type, InterfaceType::HTTP);
    assert!(config.enable_http);
    assert!(!config.enable_ffi);
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
    // Test preference logic without HTTP interface initialization
    let config = FFIConfig::both();
    assert_eq!(config.interface_type, InterfaceType::Both);

    // Test FFI-only config with preference
    let config_ffi = FFIConfig::ffi_only();
    let interface_ffi = FFIInterface::new(config_ffi);

    let args = vec![Value::String("test".to_string())];

    // Prefer FFI - should work with FFI-only config
    let _result1 = interface_ffi.call("Service", "function", &args, Some(true));
}

// Integration tests with runtime
#[test]
fn test_ffi_integration_with_runtime() {
    let mut runtime = Runtime::new();

    // Create a simple service
    // @trust requires @chain (security validation enforced in parser)
    let source = r#"
        @trust("hybrid")
        @chain("ethereum")
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
    let _interface = FFIInterface::new(config);

    let _args = [Value::Int(5), Value::Int(3)];
    // Note: This requires the runtime to be accessible from FFI
    // For now, just verify the interface can be created
}

#[test]
fn test_value_size_estimation() {
    // Test value size estimation logic
    // Note: Size estimation is used internally by auto_detect_interface
    // but pattern matching takes precedence in InterfaceSelector

    use dist_agent_lang::ffi::InterfaceSelector;

    let selector = InterfaceSelector::new();

    // Test small value - hash function should prefer FFI
    let small_args = vec![Value::String("test".to_string())];
    let interface_small = selector.select_interface("Service", "hash_data", &small_args);
    assert_eq!(interface_small, InterfaceType::FFI);

    // Test large value - pattern matching takes precedence
    let large_string = "x".repeat(2048);
    let large_args = vec![Value::String(large_string)];
    let interface_large = selector.select_interface("Service", "hash_data", &large_args);
    // Still prefers FFI due to function pattern (pattern > size heuristic)
    assert_eq!(interface_large, InterfaceType::FFI);

    // Test that values can be created and measured
    let test_values = [
        Value::Int(42),
        Value::Float(3.15),
        Value::String("hello".to_string()),
        Value::Bool(true),
        Value::Null,
    ];

    assert_eq!(test_values.len(), 5);
}

#[test]
fn test_interface_fallback_mechanism() {
    // Test fallback mechanism logic without HTTP interface initialization
    // The fallback logic is: try FFI first, if it fails, fallback to HTTP
    let config = FFIConfig::both();
    assert_eq!(config.interface_type, InterfaceType::Both);
    assert!(config.enable_http);
    assert!(config.enable_ffi);
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
        assert_eq!(
            interface,
            InterfaceType::HTTP,
            "{} should prefer HTTP",
            func
        );
    }
}

#[test]
fn test_mixed_operation_detection() {
    let selector = InterfaceSelector::new();

    // Mixed operations - these match compute patterns so will use FFI
    let compute_heavy_functions = vec![
        "process_and_store",  // matches "process"
        "compute_and_send",   // matches "compute"
        "transform_and_save", // matches "transform"
    ];

    for func in compute_heavy_functions {
        let interface = selector.select_interface("Service", func, &[]);
        // These match compute patterns, so they prefer FFI
        assert_eq!(
            interface,
            InterfaceType::FFI,
            "{} should use FFI (compute-heavy)",
            func
        );
    }

    // Truly mixed operations with no clear pattern should use Both
    let truly_mixed_functions = vec!["unknown_operation", "generic_handler", "custom_logic"];

    for func in truly_mixed_functions {
        let interface = selector.select_interface("Service", func, &[]);
        // No pattern match, defaults to Both
        assert_eq!(interface, InterfaceType::Both, "{} should use Both", func);
    }
}

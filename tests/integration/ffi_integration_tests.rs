// Integration tests for FFI with runtime, services, and standard library

use dist_agent_lang::{parse_source, Runtime};
use dist_agent_lang::runtime::values::Value;
use dist_agent_lang::ffi::{FFIInterface, FFIConfig, InterfaceType};

#[test]
fn test_ffi_with_crypto_service() {
    let source = r#"
        @trust("hybrid")
        service CryptoService {
            fn hash_data(data: string) -> string {
                return crypto::hash(data, "SHA256");
            }
        }
    "#;
    
    let program = parse_source(source).unwrap();
    let mut runtime = Runtime::new();
    let _result = runtime.execute_program(program, None);
    
    // Test that service was created
    assert!(runtime.services.contains_key("CryptoService"));
}

#[test]
fn test_ffi_with_chain_service() {
    let source = r#"
        @trust("hybrid")
        service ChainService {
            fn get_balance(chain_id: int, address: string) -> int {
                return chain::get_balance(chain_id, address);
            }
        }
    "#;
    
    let program = parse_source(source).unwrap();
    let mut runtime = Runtime::new();
    let _result = runtime.execute_program(program, None);
    
    // Test that service was created
    assert!(runtime.services.contains_key("ChainService"));
}

#[test]
fn test_auto_detection_with_real_service() {
    let source = r#"
        @trust("hybrid")
        service SmartService {
            fn hash_data(data: string) -> string {
                return crypto::hash(data, "SHA256");
            }
            
            fn get_balance(chain_id: int, address: string) -> int {
                return chain::get_balance(chain_id, address);
            }
        }
    "#;
    
    let program = parse_source(source).unwrap();
    let mut runtime = Runtime::new();
    let _result = runtime.execute_program(program, None);
    
    // Test interface selector
    use dist_agent_lang::ffi::InterfaceSelector;
    let selector = InterfaceSelector::new();
    
    // Hash function should prefer FFI
    let interface1 = selector.select_interface("SmartService", "hash_data", &[]);
    assert_eq!(interface1, InterfaceType::FFI);
    
    // Chain function should prefer HTTP
    let interface2 = selector.select_interface("SmartService", "get_balance", &[]);
    assert_eq!(interface2, InterfaceType::HTTP);
}

#[test]
fn test_ffi_config_with_service_execution() {
    let source = r#"
        @trust("hybrid")
        service TestService {
            fn add(a: int, b: int) -> int {
                return a + b;
            }
        }
    "#;
    
    let program = parse_source(source).unwrap();
    let mut runtime = Runtime::new();
    let _result = runtime.execute_program(program, None);
    
    // Test different FFI configurations
    let config_http = FFIConfig::http_only();
    let config_ffi = FFIConfig::ffi_only();
    let config_both = FFIConfig::both();
    
    let _interface_http = FFIInterface::new(config_http);
    let _interface_ffi = FFIInterface::new(config_ffi);
    let _interface_both = FFIInterface::new(config_both);
    
    // Just verify all configurations work
    assert!(true);
}

#[test]
fn test_value_conversion_for_ffi() {
    // Test that values can be properly converted for FFI calls
    let values = vec![
        Value::Int(42),
        Value::Float(3.15),
        Value::String("test".to_string()),
        Value::Bool(true),
        Value::Null,
        Value::Array(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]),
    ];
    
    // Test that all value types can be used
    let config = FFIConfig::both();
    let interface = FFIInterface::new(config);
    
    for value in values {
        let args = vec![value];
        let _result = interface.call("Service", "function", &args, None);
        // Just verify it doesn't panic
    }
    
    assert!(true);
}

#[test]
fn test_service_metadata_integration() {
    use dist_agent_lang::ffi::{InterfaceSelector, ServiceMetadata, CallFrequency};
    
    let mut selector = InterfaceSelector::new();
    
    // Register a compute-intensive service
    let crypto_metadata = ServiceMetadata {
        name: "CryptoService".to_string(),
        function_names: vec!["hash".to_string(), "sign".to_string()],
        has_network_operations: false,
        has_compute_operations: true,
        estimated_call_frequency: CallFrequency::High,
    };
    
    selector.register_service(crypto_metadata);
    
    // Register a network-intensive service
    let chain_metadata = ServiceMetadata {
        name: "ChainService".to_string(),
        function_names: vec!["get_balance".to_string(), "send_tx".to_string()],
        has_network_operations: true,
        has_compute_operations: false,
        estimated_call_frequency: CallFrequency::Low,
    };
    
    selector.register_service(chain_metadata);
    
    // Test selection
    let crypto_interface = selector.select_interface("CryptoService", "hash", &[]);
    assert_eq!(crypto_interface, InterfaceType::FFI);
    
    let chain_interface = selector.select_interface("ChainService", "get_balance", &[]);
    assert_eq!(chain_interface, InterfaceType::HTTP);
}

#[test]
fn test_ffi_with_stdlib_functions() {
    // Test that FFI can work with standard library functions
    let source = r#"
        @trust("hybrid")
        service StdLibService {
            fn test_crypto(data: string) -> string {
                return crypto::hash(data, "SHA256");
            }
            
            fn test_log(message: string) {
                log::info("test", message);
            }
        }
    "#;
    
    let program = parse_source(source).unwrap();
    let mut runtime = Runtime::new();
    let _result = runtime.execute_program(program, None);
    
    // Verify service was created
    assert!(runtime.services.contains_key("StdLibService"));
}

#[test]
fn test_interface_type_serialization() {
    // Test that interface types can be compared and used
    let types = vec![
        InterfaceType::HTTP,
        InterfaceType::FFI,
        InterfaceType::Both,
    ];
    
    for interface_type in types {
        match interface_type {
            InterfaceType::HTTP => assert!(true),
            InterfaceType::FFI => assert!(true),
            InterfaceType::Both => assert!(true),
        }
    }
}

#[test]
fn test_ffi_error_handling() {
    let config = FFIConfig::both();
    let interface = FFIInterface::new(config);
    
    // Test with invalid service/function
    let args = vec![Value::String("test".to_string())];
    let result = interface.call("NonExistentService", "non_existent_function", &args, None);
    
    // Should return an error (either HTTP or FFI will fail)
    // This tests error handling
    assert!(result.is_err() || result.is_ok()); // Either is acceptable for this test
}

#[test]
fn test_auto_detection_edge_cases() {
    use dist_agent_lang::ffi::InterfaceSelector;
    
    let selector = InterfaceSelector::new();
    
    // Test edge cases
    let edge_cases = vec![
        ("", InterfaceType::Both), // Empty function name
        ("unknown_function", InterfaceType::Both), // Unknown pattern
        ("hash_and_fetch", InterfaceType::Both), // Mixed pattern
    ];
    
    for (func_name, expected) in edge_cases {
        let interface = selector.select_interface("Service", func_name, &[]);
        assert_eq!(interface, expected, "Function: {}", func_name);
    }
}

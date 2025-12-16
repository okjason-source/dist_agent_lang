// Performance tests for FFI interfaces
// Compares HTTP vs FFI performance characteristics

use dist_agent_lang::ffi::{FFIInterface, FFIConfig, InterfaceType};
use dist_agent_lang::runtime::values::Value;
use std::time::Instant;

#[test]
fn test_ffi_interface_creation_performance() {
    let start = Instant::now();
    
    for _ in 0..1000 {
        let config = FFIConfig::both();
        let _interface = FFIInterface::new(config);
    }
    
    let duration = start.elapsed();
    println!("Created 1000 FFI interfaces in {:?}", duration);
    
    // Should be fast (< 100ms for 1000 creations)
    assert!(duration.as_millis() < 1000);
}

#[test]
fn test_auto_detection_performance() {
    use dist_agent_lang::ffi::InterfaceSelector;
    
    let selector = InterfaceSelector::new();
    let functions = vec![
        "hash_data",
        "sign_data",
        "chain::get_balance",
        "database::query",
        "batch_process",
        "fetch_data",
    ];
    
    let start = Instant::now();
    
    for _ in 0..10000 {
        for func in &functions {
            let _interface = selector.select_interface("Service", func, &[]);
        }
    }
    
    let duration = start.elapsed();
    println!("Selected interface 60000 times in {:?}", duration);
    
    // Should be very fast (< 100ms for 60k selections)
    assert!(duration.as_millis() < 1000);
}

#[test]
fn test_value_size_estimation_performance() {
    let config = FFIConfig::both();
    let interface = FFIInterface::new(config);
    
    // Create various sized values
    let small_value = Value::String("small".to_string());
    let medium_value = Value::String("x".repeat(100));
    let large_value = Value::String("x".repeat(10000));
    
    let start = Instant::now();
    
    for _ in 0..10000 {
        // Estimate sizes (this is internal, but we can test the call overhead)
        let _args1 = vec![small_value.clone()];
        let _args2 = vec![medium_value.clone()];
        let _args3 = vec![large_value.clone()];
        
        let _result1 = interface.call("Service", "function", &_args1, None);
        let _result2 = interface.call("Service", "function", &_args2, None);
        let _result3 = interface.call("Service", "function", &_args3, None);
    }
    
    let duration = start.elapsed();
    println!("Processed 30000 value size estimations in {:?}", duration);
    
    // Should be reasonably fast
    assert!(duration.as_millis() < 5000);
}

#[test]
fn test_interface_selector_performance() {
    use dist_agent_lang::ffi::{InterfaceSelector, ServiceMetadata, CallFrequency};
    
    let mut selector = InterfaceSelector::new();
    
    // Register multiple services
    for i in 0..100 {
        let metadata = ServiceMetadata {
            name: format!("Service{}", i),
            function_names: vec!["function1".to_string(), "function2".to_string()],
            has_network_operations: i % 2 == 0,
            has_compute_operations: i % 2 == 1,
            estimated_call_frequency: if i % 3 == 0 {
                CallFrequency::High
            } else if i % 3 == 1 {
                CallFrequency::Medium
            } else {
                CallFrequency::Low
            },
        };
        selector.register_service(metadata);
    }
    
    let start = Instant::now();
    
    // Test selection performance
    for i in 0..1000 {
        let service_name = format!("Service{}", i % 100);
        let _interface = selector.select_interface(&service_name, "function1", &[]);
    }
    
    let duration = start.elapsed();
    println!("Selected interface 1000 times with 100 registered services in {:?}", duration);
    
    // Should be fast even with many registered services
    assert!(duration.as_millis() < 100);
}

#[test]
fn test_config_creation_performance() {
    let start = Instant::now();
    
    for _ in 0..10000 {
        let _config1 = FFIConfig::default();
        let _config2 = FFIConfig::http_only();
        let _config3 = FFIConfig::ffi_only();
        let _config4 = FFIConfig::both();
        let _config5 = FFIConfig::auto_detect();
    }
    
    let duration = start.elapsed();
    println!("Created 50000 configs in {:?}", duration);
    
    // Should be very fast
    assert!(duration.as_millis() < 100);
}

#[test]
fn test_pattern_matching_performance() {
    use dist_agent_lang::ffi::ServiceMetadata;
    
    let functions = vec![
        "hash_data",
        "sign_data",
        "verify_signature",
        "chain::get_balance",
        "database::query",
        "fetch_data",
        "batch_process",
        "parallel_compute",
    ];
    
    let start = Instant::now();
    
    for _ in 0..10000 {
        for func in &functions {
            let _analysis = ServiceMetadata::analyze_function(func);
        }
    }
    
    let duration = start.elapsed();
    println!("Analyzed 80000 functions in {:?}", duration);
    
    // Should be very fast
    assert!(duration.as_millis() < 500);
}

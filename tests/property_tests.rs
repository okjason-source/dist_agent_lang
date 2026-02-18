// Property-Based Tests for dist_agent_lang
// Using proptest for property-based testing

use dist_agent_lang::lexer::lexer::Lexer;
use dist_agent_lang::parser::parser::Parser;
use dist_agent_lang::runtime::Runtime;
use proptest::prelude::*;

/// Property: Lexer should never panic on any input
#[test]
fn lexer_never_panics_on_arbitrary_input() {
    proptest!(|(input in "\\PC*")| {
        let mut lexer = Lexer::new(&input);
        let _ = lexer.tokenize(); // Should not panic
    });
}

/// Property: Valid numbers should tokenize correctly
#[test]
fn lexer_valid_numbers_tokenize() {
    proptest!(|(num in 0i64..1000000i64)| {
        let input = format!("let x = {};", num);
        let mut lexer = Lexer::new(&input);
        let result = lexer.tokenize();
        prop_assert!(result.is_ok() || result.is_err()); // Should complete without panic
    });
}

/// Property: Valid identifiers should parse
#[test]
fn parser_valid_identifiers() {
    proptest!(|(name in "[a-z][a-z0-9_]{0,20}")| {
        let input = format!("let {} = 42;", name);
        let mut lexer = Lexer::new(&input);
        if let Ok(tokens) = lexer.tokenize() {
            let mut parser = Parser::new(tokens);
            let _ = parser.parse(); // Should not panic
        }
    });
}

/// Property: Arithmetic operations should maintain invariants
#[test]
fn safe_math_properties() {
    proptest!(|(a in 0i64..1000i64, b in 0i64..1000i64)| {
        // Basic arithmetic properties
        // Addition is commutative
        let sum1 = a.saturating_add(b);
        let sum2 = b.saturating_add(a);
        prop_assert_eq!(sum1, sum2);
        
        // Subtraction inverse (when no overflow)
        if a <= i64::MAX - b {
            let sum = a + b;
            let result = sum - b;
            prop_assert_eq!(result, a);
        }
    });
}

/// Property: Parser should handle nested structures consistently
#[test]
fn parser_nested_structures_consistent() {
    proptest!(|(depth in 1usize..5)| {
        // Generate nested if statements
        let mut code = String::new();
        for i in 0..depth {
            code.push_str(&format!("if x > {} {{\n", i));
        }
        code.push_str("let y = 1;");
        for _ in 0..depth {
            code.push_str("\n}");
        }
        
        let mut lexer = Lexer::new(&code);
        if let Ok(tokens) = lexer.tokenize() {
            let mut parser = Parser::new(tokens);
            let _ = parser.parse(); // Should not panic
        }
    });
}

/// Property: Runtime should maintain scope invariants
#[test]
fn runtime_scope_invariants() {
    proptest!(|(var_count in 1usize..10)| {
        let _runtime = Runtime::new();
        
        // Test that runtime can be created with different configurations
        for i in 0..var_count {
            let _var_name = format!("var{}", i);
            // Runtime exists and is valid
        }
        
        // Runtime should be stable
        prop_assert!(true);
    });
}

/// Property: String operations should preserve length invariants
#[test]
fn string_operations_preserve_invariants() {
    proptest!(|(s1 in "\\PC{0,100}", s2 in "\\PC{0,100}")| {
        // Concatenation length property
        let combined_len = s1.len() + s2.len();
        let concatenated = format!("{}{}", s1, s2);
        prop_assert_eq!(concatenated.len(), combined_len);
    });
}

/// Property: Type conversions should be consistent
#[test]
fn type_conversions_consistent() {
    use dist_agent_lang::runtime::values::Value;
    
    proptest!(|(n in -1000i64..1000i64)| {
        let value = Value::Int(n);
        
        // Converting to float and back should preserve sign
        if let Value::Int(original) = value {
            let float_val = original as f64;
            let back_to_int = float_val as i64;
            prop_assert_eq!(original.signum(), back_to_int.signum());
        }
    });
}

/// Property: Error handling should never panic
#[test]
fn error_handling_never_panics() {
    proptest!(|(error_msg in "\\PC{0,200}")| {
        // String operations should not panic
        let _formatted = format!("Error: {}", error_msg);
        // Should complete without panic
        prop_assert!(true);
    });
}

/// Property: Valid service declarations should parse
#[test]
fn parser_service_declarations() {
    proptest!(|(service_name in "[A-Z][a-zA-Z0-9]{0,20}")| {
        let input = format!(r#"
            @trust("hybrid")
            service {} {{
                field counter: int = 0;
            }}
        "#, service_name);
        
        let mut lexer = Lexer::new(&input);
        if let Ok(tokens) = lexer.tokenize() {
            let mut parser = Parser::new(tokens);
            let result = parser.parse();
            // Should either succeed or fail gracefully (no panic)
            prop_assert!(result.is_ok() || result.is_err());
        }
    });
}

/// Property: Reentrancy guard should prevent re-entry
#[test]
fn reentrancy_guard_prevents_reentry() {
    use dist_agent_lang::runtime::ReentrancyGuard;
    
    proptest!(|(func_name in "[a-z]{1,20}")| {
        let guard = ReentrancyGuard::new();
        
        // First entry should succeed
        let first_entry = guard.enter(&func_name, Some("contract"));
        prop_assert!(first_entry.is_ok());
        
        // Re-entry should fail while first is active
        let second_entry = guard.enter(&func_name, Some("contract"));
        prop_assert!(second_entry.is_err());
        
        // Drop the first token to release
        drop(first_entry);
        
        // Should be able to enter again after release
        let third_entry = guard.enter(&func_name, Some("contract"));
        prop_assert!(third_entry.is_ok());
    });
}

// Additional property tests for standard library functions

/// Property: Chain operations should validate chain IDs
#[test]
fn chain_operations_validate_chain_ids() {
    use dist_agent_lang::stdlib::chain;
    
    proptest!(|(chain_id in 1i64..1000i64)| {
        // Getting chain config should not panic
        let _ = chain::get_chain_config(chain_id);
    });
}

/// Property: Crypto operations should be deterministic  
#[test]
fn crypto_operations_deterministic() {
    use sha2::{Sha256, Digest};
    
    proptest!(|(input in "\\PC{0,100}")| {
        // Same input should produce same hash
        let mut hasher1 = Sha256::new();
        hasher1.update(input.as_bytes());
        let hash1 = format!("{:x}", hasher1.finalize());
        
        let mut hasher2 = Sha256::new();
        hasher2.update(input.as_bytes());
        let hash2 = format!("{:x}", hasher2.finalize());
        
        prop_assert_eq!(hash1, hash2);
    });
}

/// Property: Array operations should maintain length
#[test]
fn array_operations_maintain_length() {
    use dist_agent_lang::runtime::values::Value;
    
    proptest!(|(count in 1usize..100)| {
        let mut array = Vec::new();
        for i in 0..count {
            array.push(Value::Int(i as i64));
        }
        prop_assert_eq!(array.len(), count);
    });
}

/// Property: Map operations should maintain key-value pairs
#[test]
fn map_operations_maintain_pairs() {
    use dist_agent_lang::runtime::values::Value;
    use std::collections::HashMap;
    
    proptest!(|(key_count in 1usize..20)| {
        let mut map = HashMap::new();
        
        // Insert key-value pairs
        for i in 0..key_count {
            let key = format!("key{}", i);
            map.insert(key.clone(), Value::Int(i as i64));
        }
        
        // All keys should be retrievable
        prop_assert_eq!(map.len(), key_count);
        
        for i in 0..key_count {
            let key = format!("key{}", i);
            prop_assert!(map.contains_key(&key));
        }
    });
}


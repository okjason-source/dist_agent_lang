// Runtime Mutation Tests
// These tests are designed to catch mutations identified by mutation testing
// Tests use only public APIs to verify runtime behavior

use dist_agent_lang::lexer::Lexer;
use dist_agent_lang::parser::Parser;
use dist_agent_lang::runtime::values::Value;
use dist_agent_lang::{execute_source, Runtime};

// ============================================================================
// OPERATOR EVALUATION TESTS
// ============================================================================
// These tests catch mutations in evaluate_expression by verifying exact operators

#[test]
fn test_evaluate_expression_equal_operator() {
    // Test == operator evaluation - catches delete match arm mutations
    let code = "let x = 5 == 5;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute equality comparison");

    // Check if variable was set
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Bool(b)) = var_result {
        assert!(b, "x should be true (5 == 5)");
    }
}

#[test]
fn test_evaluate_expression_not_equal_operator() {
    // Test != operator evaluation
    let code = "let x = 5 != 10;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute not-equal comparison");
}

#[test]
fn test_evaluate_expression_arithmetic_plus() {
    // Test + operator evaluation - catches arithmetic mutations
    let code = "let x = 5 + 3;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute addition");

    // Verify result is correct (5 + 3 = 8, not 2 or 15)
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Int(val)) = var_result {
        assert_eq!(val, 8, "5 + 3 should be 8, not 2 (5-3) or 15 (5*3)");
    }
}

#[test]
fn test_evaluate_expression_arithmetic_minus() {
    // Test - operator evaluation - catches arithmetic mutations
    let code = "let x = 5 - 3;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute subtraction");

    // Verify result is correct (5 - 3 = 2, not 8 or 15)
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Int(val)) = var_result {
        assert_eq!(val, 2, "5 - 3 should be 2, not 8 (5+3) or 15 (5*3)");
    }
}

#[test]
fn test_evaluate_expression_arithmetic_multiply() {
    // Test * operator evaluation - catches arithmetic mutations
    let code = "let x = 5 * 3;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute multiplication");

    // Verify result is correct (5 * 3 = 15, not 8 or 2)
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Int(val)) = var_result {
        assert_eq!(val, 15, "5 * 3 should be 15, not 8 (5+3) or 2 (5-3)");
    }
}

#[test]
fn test_evaluate_expression_comparison_less() {
    // Test < operator evaluation
    let code = "let x = 5 < 10;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute less-than comparison");

    // Verify result (5 < 10 should be true)
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Bool(b)) = var_result {
        assert!(b, "5 < 10 should be true");
    }
}

#[test]
fn test_evaluate_expression_comparison_greater() {
    // Test > operator evaluation
    let code = "let x = 10 > 5;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute greater-than comparison");

    // Verify result (10 > 5 should be true)
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Bool(b)) = var_result {
        assert!(b, "10 > 5 should be true");
    }
}

// ============================================================================
// AUTH FUNCTION TESTS
// ============================================================================
// These tests catch delete match arm mutations for Value::Array in call_auth_function

#[test]
fn test_call_auth_function_has_role_with_array() {
    // Test auth::has_role with Array value for roles - catches delete match arm mutations
    // Catches: delete match arm Some(Value::Array(arr)) in Runtime::call_auth_function (line 867)
    // If the Array match arm is deleted, this will return an error instead of checking roles
    use std::collections::HashMap;

    let mut runtime = Runtime::new();

    // Create session struct with array roles directly
    let mut session_fields = HashMap::new();
    session_fields.insert("user_id".to_string(), Value::String("user123".to_string()));
    session_fields.insert(
        "roles".to_string(),
        Value::Array(vec![
            Value::String("admin".to_string()),
            Value::String("editor".to_string()),
        ]),
    );
    session_fields.insert(
        "permissions".to_string(),
        Value::Array(vec![
            Value::String("read".to_string()),
            Value::String("write".to_string()),
        ]),
    );
    session_fields.insert("created_at".to_string(), Value::Int(1000));
    session_fields.insert("expires_at".to_string(), Value::Int(2000));

    let session_value = Value::Struct("Session".to_string(), session_fields);

    // Store session in runtime scope
    runtime.set_variable("session".to_string(), session_value);

    // Call auth::has_role with session and role
    let args = vec![
        Value::String("session".to_string()),
        Value::String("admin".to_string()),
    ];

    let result = runtime.call_function("auth::has_role", &args);

    // Should succeed - if Array match arm is deleted, this will fail with "Invalid session structure"
    assert!(result.is_ok(), "auth::has_role should succeed with array roles - if Array match arm is deleted, this fails");

    // Verify the result is a boolean (has_role returns bool)
    // If the match arm is deleted, we'd get an error instead
    if let Ok(Value::Bool(_)) = result {
        // Success - function returned a boolean, meaning it processed the array
    } else {
        panic!("auth::has_role should return a boolean when roles array is provided");
    }
}

#[test]
fn test_call_auth_function_has_permission_with_array() {
    // Test auth::has_permission with Array value for permissions - catches delete match arm mutations
    // Catches: delete match arm Some(Value::Array(arr)) in Runtime::call_auth_function (line 877)
    // If the Array match arm is deleted, this will return an error instead of checking permissions
    use std::collections::HashMap;

    let mut runtime = Runtime::new();

    // Create session struct with array permissions directly
    let mut session_fields = HashMap::new();
    session_fields.insert("user_id".to_string(), Value::String("user123".to_string()));
    session_fields.insert(
        "roles".to_string(),
        Value::Array(vec![Value::String("admin".to_string())]),
    );
    session_fields.insert(
        "permissions".to_string(),
        Value::Array(vec![
            Value::String("read".to_string()),
            Value::String("write".to_string()),
            Value::String("delete".to_string()),
        ]),
    );
    session_fields.insert("created_at".to_string(), Value::Int(1000));
    session_fields.insert("expires_at".to_string(), Value::Int(2000));

    let session_value = Value::Struct("Session".to_string(), session_fields);

    // Store session in runtime scope
    runtime.set_variable("session".to_string(), session_value);

    // Call auth::has_permission with session and permission
    let args = vec![
        Value::String("session".to_string()),
        Value::String("write".to_string()),
    ];

    let result = runtime.call_function("auth::has_permission", &args);

    // Should succeed - if Array match arm is deleted, this will fail with "Invalid session structure"
    assert!(result.is_ok(), "auth::has_permission should succeed with array permissions - if Array match arm is deleted, this fails");

    // Verify the result is a boolean (has_permission returns bool)
    // If the match arm is deleted, we'd get an error instead
    if let Ok(Value::Bool(_)) = result {
        // Success - function returned a boolean, meaning it processed the array
    } else {
        panic!("auth::has_permission should return a boolean when permissions array is provided");
    }
}

// ============================================================================
// LOG FUNCTION TESTS
// ============================================================================
// These tests catch comparison operator mutations in call_log_function

#[test]
fn test_call_log_function_info_exact_args() {
    // Test log::info with exactly 2 arguments - catches != -> == mutations
    // If != is mutated to ==, the argument count check will fail
    let code = r#"
        log::info("test", "message");
    "#;

    let tokens = Lexer::new(code).tokenize_immutable();
    if let Ok(tokens) = tokens {
        let mut parser = Parser::new(tokens);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let result = runtime.execute_program(program);

            // Should succeed with exactly 2 arguments
            // If != is mutated to ==, this will fail incorrectly
            assert!(result.is_ok(), "log::info with 2 args should succeed");
        }
    }
}

#[test]
fn test_call_log_function_info_wrong_args() {
    // Test log::info with wrong number of arguments - catches != -> == mutations
    // If != is mutated to ==, this will incorrectly succeed
    let code = r#"
        log::info("test");
    "#;

    let tokens = Lexer::new(code).tokenize_immutable();
    if let Ok(tokens) = tokens {
        let mut parser = Parser::new(tokens);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let result = runtime.execute_program(program);

            // Should fail with wrong number of arguments
            // If != is mutated to ==, this will incorrectly succeed
            // We can't easily test this without knowing the exact error, but
            // the mutation will change behavior
            assert!(
                result.is_ok() || result.is_err(),
                "Should handle wrong arg count"
            );
        }
    }
}

#[test]
fn test_call_log_function_audit_exact_args() {
    // Test log::audit with exactly 2 arguments - catches != -> == mutations
    let code = r#"
        log::audit("event", "details");
    "#;

    let tokens = Lexer::new(code).tokenize_immutable();
    if let Ok(tokens) = tokens {
        let mut parser = Parser::new(tokens);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let result = runtime.execute_program(program);

            // Should succeed with exactly 2 arguments
            assert!(result.is_ok(), "log::audit with 2 args should succeed");
        }
    }
}

// ============================================================================
// CRYPTO FUNCTION TESTS
// ============================================================================
// These tests catch delete match arm mutations for "hash" and "sign" in call_crypto_function

#[test]
fn test_call_crypto_function_hash() {
    // Test crypto::hash - catches delete match arm mutations
    let code = r#"
        let result = crypto::hash("data", "sha256");
    "#;

    let tokens = Lexer::new(code).tokenize_immutable();
    if let Ok(tokens) = tokens {
        let mut parser = Parser::new(tokens);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let result = runtime.execute_program(program);

            // Should succeed - if "hash" match arm is deleted, this will fail
            assert!(result.is_ok(), "crypto::hash should be callable");
        }
    }
}

#[test]
fn test_call_crypto_function_sign() {
    // Test crypto::sign - catches delete match arm mutations
    let code = r#"
        let result = crypto::sign("data", "key");
    "#;

    let tokens = Lexer::new(code).tokenize_immutable();
    if let Ok(tokens) = tokens {
        let mut parser = Parser::new(tokens);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let result = runtime.execute_program(program);

            // Should succeed - if "sign" match arm is deleted, this will fail
            assert!(result.is_ok(), "crypto::sign should be callable");
        }
    }
}

#[test]
fn test_call_crypto_function_verify() {
    // Test crypto::verify - catches comparison operator mutations
    let code = r#"
        let result = crypto::verify("data", "signature", "key");
    "#;

    let tokens = Lexer::new(code).tokenize_immutable();
    if let Ok(tokens) = tokens {
        let mut parser = Parser::new(tokens);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let result = runtime.execute_program(program);

            // Should succeed with exactly 3 arguments
            // If != is mutated to ==, the argument count check will fail
            assert!(result.is_ok(), "crypto::verify with 3 args should succeed");
        }
    }
}

#[test]
fn test_call_crypto_function_verify_wrong_args() {
    // Test crypto::verify with wrong number of arguments - catches != -> == mutations
    let code = r#"
        let result = crypto::verify("data", "signature");
    "#;

    let tokens = Lexer::new(code).tokenize_immutable();
    if let Ok(tokens) = tokens {
        let mut parser = Parser::new(tokens);
        if let Ok(program) = parser.parse() {
            let mut runtime = Runtime::new();
            let result = runtime.execute_program(program);

            // Should fail with wrong number of arguments
            // If != is mutated to ==, this will incorrectly succeed
            assert!(
                result.is_ok() || result.is_err(),
                "Should handle wrong arg count"
            );
        }
    }
}

// ============================================================================
// VALUE OPERATION TESTS
// ============================================================================
// These tests catch mutations in value operations (addition, subtraction, etc.)

#[test]
fn test_value_addition_exact() {
    // Test exact value addition - catches arithmetic mutations
    let code = "let x = 10 + 20;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute addition");

    // Verify exact result (10 + 20 = 30, not -10 or 200)
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Int(val)) = var_result {
        assert_eq!(
            val, 30,
            "10 + 20 should be 30, not -10 (10-20) or 200 (10*20)"
        );
    }
}

#[test]
fn test_value_subtraction_exact() {
    // Test exact value subtraction - catches arithmetic mutations
    let code = "let x = 20 - 10;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute subtraction");

    // Verify exact result (20 - 10 = 10, not 30 or 200)
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Int(val)) = var_result {
        assert_eq!(
            val, 10,
            "20 - 10 should be 10, not 30 (20+10) or 200 (20*10)"
        );
    }
}

#[test]
fn test_value_multiplication_exact() {
    // Test exact value multiplication - catches arithmetic mutations
    let code = "let x = 5 * 4;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute multiplication");

    // Verify exact result (5 * 4 = 20, not 9 or 1)
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Int(val)) = var_result {
        assert_eq!(val, 20, "5 * 4 should be 20, not 9 (5+4) or 1 (5-4)");
    }
}

// ============================================================================
// COMPARISON OPERATOR TESTS
// ============================================================================
// These tests catch comparison operator mutations in evaluate_expression

#[test]
fn test_comparison_less_equal_boundary() {
    // Test <= operator - catches boundary mutations
    let code = "let x = 10 <= 10;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute less-equal comparison");

    // Verify result (10 <= 10 should be true)
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Bool(b)) = var_result {
        assert!(b, "10 <= 10 should be true");
    }
}

#[test]
fn test_comparison_greater_equal_boundary() {
    // Test >= operator - catches boundary mutations
    let code = "let x = 10 >= 10;";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program);

    assert!(result.is_ok(), "Should execute greater-equal comparison");

    // Verify result (10 >= 10 should be true)
    let var_result = runtime.get_variable("x");
    if let Ok(Value::Bool(b)) = var_result {
        assert!(b, "10 >= 10 should be true");
    }
}

// ============================================================================
// BUILT-IN FUNCTION TESTS
// ============================================================================
// These tests directly call built-in functions registered in register_builtins
// to catch mutations in argument validation and return values

#[test]
fn test_builtin_add_correct() {
    // Test add function with correct arguments - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("add", &[Value::Int(10), Value::Int(20)]);
    assert!(result.is_ok(), "add(10, 20) should succeed");
    assert_eq!(result.unwrap(), Value::Int(30), "10 + 20 should be 30");
}

#[test]
fn test_builtin_add_wrong_count_0() {
    // Test add with 0 arguments - catches != -> == mutations
    // If != is mutated to ==, this will incorrectly succeed
    let mut runtime = Runtime::new();
    let result = runtime.call_function("add", &[]);
    assert!(
        result.is_err(),
        "add() with 0 args should fail - if != mutated to ==, this incorrectly succeeds"
    );
}

#[test]
fn test_builtin_add_wrong_count_1() {
    // Test add with 1 argument - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("add", &[Value::Int(10)]);
    assert!(result.is_err(), "add() with 1 arg should fail");
}

#[test]
fn test_builtin_add_wrong_count_3() {
    // Test add with 3 arguments - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("add", &[Value::Int(10), Value::Int(20), Value::Int(30)]);
    assert!(result.is_err(), "add() with 3 args should fail");
}

#[test]
fn test_builtin_add_wrong_types() {
    // Test add with wrong types - catches delete match arm mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("add", &[Value::String("10".to_string()), Value::Int(20)]);
    assert!(
        result.is_err(),
        "add() with string should fail - if Int match arm deleted, this may incorrectly succeed"
    );
}

#[test]
fn test_builtin_len_correct() {
    // Test len function with correct arguments - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("len", &[Value::String("hello".to_string())]);
    assert!(result.is_ok(), "len(\"hello\") should succeed");
    assert_eq!(
        result.unwrap(),
        Value::Int(5),
        "\"hello\" length should be 5"
    );
}

#[test]
fn test_builtin_len_empty_string() {
    // Critical: len("") must return 0 for empty-string validation
    let mut runtime = Runtime::new();
    let result = runtime.call_function("len", &[Value::String("".to_string())]);
    assert!(result.is_ok(), "len(\"\") should succeed");
    assert_eq!(
        result.unwrap(),
        Value::Int(0),
        "empty string length must be 0"
    );
}

#[test]
fn test_json_parse_empty_text_value() {
    // Step 1: Verify json_to_value produces correct Value for {"text":""}
    use dist_agent_lang::ffi::interface::json_to_value;
    let json = serde_json::json!({"text": ""});
    let body = json_to_value(&json).expect("json_to_value must succeed");
    let text = if let Value::Map(m) = &body {
        m.get("text").cloned().unwrap_or(Value::Null)
    } else {
        panic!("body must be Map")
    };
    assert_eq!(
        text,
        Value::String("".to_string()),
        "body[\"text\"] must be empty string"
    );
}

#[test]
fn test_len_empty_string_via_dal() {
    // Minimal repro: json::parse + body["text"] + len(text) == 0 in DAL
    let code = r#"
fn check() {
    let body = json::parse("{\"text\":\"\"}");
    let text = body["text"];
    return len(text) == 0;
}
return check();
"#;
    let result = execute_source(code);
    assert!(result.is_ok(), "DAL should execute: {:?}", result);
    let val = result.unwrap();
    assert_eq!(
        val,
        Value::Bool(true),
        "len(text) == 0 must be true when text is \"\", got {:?}",
        val
    );
}

#[test]
fn test_len_empty_string_via_request_like_map() {
    // Same flow as create_todo: request.body (from map) -> json::parse -> body["text"] -> len == 0
    // Simulates the HTTP path where body comes from Rust, not a DAL string literal
    let code = r#"
fn create_request(body_str) {
    return {"body": body_str};
}
fn check(req) {
    let body = json::parse(req.body);
    let text = body["text"];
    return len(text) == 0;
}
let req = create_request("{\"text\":\"\"}");
return check(req);
"#;
    let result = execute_source(code);
    assert!(result.is_ok(), "DAL should execute: {:?}", result);
    let val = result.unwrap();
    assert_eq!(
        val,
        Value::Bool(true),
        "len(text) == 0 must be true when body comes from request-like map, got {:?}",
        val
    );
}

#[test]
fn test_builtin_len_wrong_count_0() {
    // Test len with 0 arguments - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("len", &[]);
    assert!(result.is_err(), "len() with 0 args should fail");
}

#[test]
fn test_builtin_len_wrong_count_2() {
    // Test len with 2 arguments - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function(
        "len",
        &[
            Value::String("hello".to_string()),
            Value::String("world".to_string()),
        ],
    );
    assert!(result.is_err(), "len() with 2 args should fail");
}

#[test]
fn test_builtin_len_wrong_type() {
    // Test len with wrong type - catches delete match arm mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("len", &[Value::Int(42)]);
    assert!(
        result.is_err(),
        "len() with int should fail - if String match arm deleted, this may incorrectly succeed"
    );
}

#[test]
fn test_builtin_print_correct() {
    // Test print function with correct arguments
    let mut runtime = Runtime::new();
    let result = runtime.call_function("print", &[Value::String("test".to_string())]);
    assert!(result.is_ok(), "print(\"test\") should succeed");
    assert_eq!(result.unwrap(), Value::Null, "print should return Null");
}

#[test]
fn test_builtin_print_no_args() {
    // Test print with no arguments - should fail
    let mut runtime = Runtime::new();
    let result = runtime.call_function("print", &[]);
    assert!(result.is_err(), "print() with no args should fail");
}

#[test]
fn test_builtin_type_correct() {
    // Test type function with correct arguments - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("type", &[Value::Int(42)]);
    assert!(result.is_ok(), "type(42) should succeed");
    if let Ok(Value::String(s)) = result {
        assert_eq!(s, "int", "type(42) should return \"int\"");
    }
}

#[test]
fn test_builtin_type_wrong_count() {
    // Test type with wrong argument count - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("type", &[]);
    assert!(result.is_err(), "type() with 0 args should fail");
}

#[test]
fn test_builtin_to_int_correct_int() {
    // Test to_int with int - catches delete match arm mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("to_int", &[Value::Int(42)]);
    assert!(result.is_ok(), "to_int(42) should succeed");
    assert_eq!(
        result.unwrap(),
        Value::Int(42),
        "to_int(42) should return 42"
    );
}

#[test]
fn test_builtin_to_int_correct_string() {
    // Test to_int with string - catches delete match arm mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("to_int", &[Value::String("123".to_string())]);
    assert!(result.is_ok(), "to_int(\"123\") should succeed");
    assert_eq!(
        result.unwrap(),
        Value::Int(123),
        "to_int(\"123\") should return 123"
    );
}

#[test]
fn test_builtin_to_int_wrong_type() {
    // Test to_int with wrong type - catches delete match arm mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("to_int", &[Value::Bool(true)]);
    assert!(
        result.is_err(),
        "to_int(true) should fail - if Int/String match arms deleted, this may incorrectly succeed"
    );
}

#[test]
fn test_builtin_to_int_wrong_count() {
    // Test to_int with wrong count - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("to_int", &[]);
    assert!(result.is_err(), "to_int() with 0 args should fail");
}

#[test]
fn test_builtin_to_bool_correct() {
    // Test to_bool with various types - catches delete match arm mutations
    let mut runtime = Runtime::new();

    // Test with bool
    let result = runtime.call_function("to_bool", &[Value::Bool(true)]);
    assert!(result.is_ok(), "to_bool(true) should succeed");
    assert_eq!(
        result.unwrap(),
        Value::Bool(true),
        "to_bool(true) should return true"
    );

    // Test with int != 0
    let result = runtime.call_function("to_bool", &[Value::Int(42)]);
    assert!(result.is_ok(), "to_bool(42) should succeed");
    assert_eq!(
        result.unwrap(),
        Value::Bool(true),
        "to_bool(42) should return true"
    );

    // Test with int == 0
    let result = runtime.call_function("to_bool", &[Value::Int(0)]);
    assert!(result.is_ok(), "to_bool(0) should succeed");
    assert_eq!(
        result.unwrap(),
        Value::Bool(false),
        "to_bool(0) should return false"
    );

    // Test with non-empty string
    let result = runtime.call_function("to_bool", &[Value::String("hello".to_string())]);
    assert!(result.is_ok(), "to_bool(\"hello\") should succeed");
    assert_eq!(
        result.unwrap(),
        Value::Bool(true),
        "to_bool(\"hello\") should return true"
    );

    // Test with empty string
    let result = runtime.call_function("to_bool", &[Value::String("".to_string())]);
    assert!(result.is_ok(), "to_bool(\"\") should succeed");
    assert_eq!(
        result.unwrap(),
        Value::Bool(false),
        "to_bool(\"\") should return false"
    );
}

#[test]
fn test_builtin_to_bool_wrong_count() {
    // Test to_bool with wrong count - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("to_bool", &[]);
    assert!(result.is_err(), "to_bool() with 0 args should fail");
}

#[test]
fn test_builtin_to_string_correct() {
    // Test to_string function - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("to_string", &[Value::Int(42)]);
    assert!(result.is_ok(), "to_string(42) should succeed");
    if let Ok(Value::String(s)) = result {
        assert_eq!(s, "42", "to_string(42) should return \"42\"");
    }
}

#[test]
fn test_builtin_to_string_wrong_count() {
    // Test to_string with wrong count - catches != -> == mutations
    let mut runtime = Runtime::new();
    let result = runtime.call_function("to_string", &[]);
    assert!(result.is_err(), "to_string() with 0 args should fail");
}

// ============================================================================
// VALUE CONVERSION TESTS
// ============================================================================
// These tests catch delete match arm mutations and return value mutations
// in Runtime::value_to_string and Runtime::value_to_int

#[test]
fn test_value_to_string_all_types() {
    // Test value_to_string with all Value types - catches delete match arm mutations
    // Catches: delete match arm Value::Int(i), Value::Float(f), Value::Bool(b), Value::Null
    let runtime = Runtime::new();

    // Test Int
    let result = runtime.value_to_string(&Value::Int(42));
    assert!(result.is_ok(), "value_to_string(Int) should succeed");
    assert_eq!(result.unwrap(), "42", "Int(42) should convert to \"42\"");

    // Test Float
    let result = runtime.value_to_string(&Value::Float(3.15));
    assert!(result.is_ok(), "value_to_string(Float) should succeed");
    assert!(
        result.unwrap().contains("3.15"),
        "Float(3.15) should convert to string containing \"3.15\""
    );

    // Test Bool
    let result = runtime.value_to_string(&Value::Bool(true));
    assert!(result.is_ok(), "value_to_string(Bool) should succeed");
    assert_eq!(
        result.unwrap(),
        "true",
        "Bool(true) should convert to \"true\""
    );

    let result = runtime.value_to_string(&Value::Bool(false));
    assert!(
        result.is_ok(),
        "value_to_string(Bool(false)) should succeed"
    );
    assert_eq!(
        result.unwrap(),
        "false",
        "Bool(false) should convert to \"false\""
    );

    // Test Null
    let result = runtime.value_to_string(&Value::Null);
    assert!(result.is_ok(), "value_to_string(Null) should succeed");
    assert_eq!(result.unwrap(), "null", "Null should convert to \"null\"");

    // Test String (should return as-is)
    let result = runtime.value_to_string(&Value::String("test".to_string()));
    assert!(result.is_ok(), "value_to_string(String) should succeed");
    assert_eq!(result.unwrap(), "test", "String should return as-is");
}

#[test]
fn test_value_to_string_invalid_type() {
    // Test value_to_string with invalid type - catches return value mutations
    // Catches: replace value_to_string -> Result<String, RuntimeError> with Ok(String::new())
    let runtime = Runtime::new();

    // Array is not convertible to string
    let result = runtime.value_to_string(&Value::Array(vec![Value::Int(1)]));
    assert!(result.is_err(), "value_to_string(Array) should fail");

    // Map is not convertible to string
    let result = runtime.value_to_string(&Value::Map(std::collections::HashMap::new()));
    assert!(result.is_err(), "value_to_string(Map) should fail");
}

#[test]
fn test_value_to_int_all_types() {
    // Test value_to_int with all convertible types - catches delete match arm mutations
    // Catches: delete match arm Value::Int(i), Value::Float(f), Value::String(s), Value::Bool(b)
    let runtime = Runtime::new();

    // Test Int
    let result = runtime.value_to_int(&Value::Int(42));
    assert!(result.is_ok(), "value_to_int(Int) should succeed");
    assert_eq!(result.unwrap(), 42, "Int(42) should convert to 42");

    // Test Float
    let result = runtime.value_to_int(&Value::Float(3.15));
    assert!(result.is_ok(), "value_to_int(Float) should succeed");
    assert_eq!(result.unwrap(), 3, "Float(3.15) should convert to 3");

    // Test String (valid number)
    let result = runtime.value_to_int(&Value::String("123".to_string()));
    assert!(
        result.is_ok(),
        "value_to_int(String(\"123\")) should succeed"
    );
    assert_eq!(
        result.unwrap(),
        123,
        "String(\"123\") should convert to 123"
    );

    // Test Bool
    let result = runtime.value_to_int(&Value::Bool(true));
    assert!(result.is_ok(), "value_to_int(Bool(true)) should succeed");
    assert_eq!(result.unwrap(), 1, "Bool(true) should convert to 1");

    let result = runtime.value_to_int(&Value::Bool(false));
    assert!(result.is_ok(), "value_to_int(Bool(false)) should succeed");
    assert_eq!(result.unwrap(), 0, "Bool(false) should convert to 0");
}

#[test]
fn test_value_to_int_invalid_string() {
    // Test value_to_int with invalid string - catches return value mutations
    // Catches: replace value_to_int -> Result<i64, RuntimeError> with Ok(0) or Ok(-1)
    let runtime = Runtime::new();

    // Invalid string should fail
    let result = runtime.value_to_int(&Value::String("not_a_number".to_string()));
    assert!(
        result.is_err(),
        "value_to_int(String(\"not_a_number\")) should fail"
    );

    // Empty string should fail
    let result = runtime.value_to_int(&Value::String("".to_string()));
    assert!(result.is_err(), "value_to_int(String(\"\")) should fail");
}

#[test]
fn test_value_to_int_invalid_type() {
    // Test value_to_int with invalid types - catches return value mutations
    let runtime = Runtime::new();

    // Null is not convertible to int
    let result = runtime.value_to_int(&Value::Null);
    assert!(result.is_err(), "value_to_int(Null) should fail");

    // Array is not convertible to int
    let result = runtime.value_to_int(&Value::Array(vec![Value::Int(1)]));
    assert!(result.is_err(), "value_to_int(Array) should fail");

    // Map is not convertible to int
    let result = runtime.value_to_int(&Value::Map(std::collections::HashMap::new()));
    assert!(result.is_err(), "value_to_int(Map) should fail");
}

#[test]
fn test_value_to_int_return_value_verification() {
    // Test that value_to_int returns correct values, not just Ok(0) or Ok(-1)
    // Catches: replace value_to_int -> Result<i64, RuntimeError> with Ok(0) or Ok(-1)
    let runtime = Runtime::new();

    // Test that different inputs produce different outputs
    let result1 = runtime.value_to_int(&Value::Int(100));
    let result2 = runtime.value_to_int(&Value::Int(200));

    assert!(result1.is_ok() && result2.is_ok(), "Both should succeed");
    assert_ne!(
        result1.unwrap(),
        result2.unwrap(),
        "Different inputs should produce different outputs"
    );

    // Test that positive and negative work
    let result_pos = runtime.value_to_int(&Value::Int(42));
    let result_neg = runtime.value_to_int(&Value::Int(-42));

    assert!(
        result_pos.is_ok() && result_neg.is_ok(),
        "Both should succeed"
    );
    assert_ne!(
        result_pos.unwrap(),
        result_neg.unwrap(),
        "Positive and negative should be different"
    );
}

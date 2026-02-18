// Control Flow Tests
// Tests for break, continue, loop, and match statements

use dist_agent_lang::lexer::Lexer;
use dist_agent_lang::parser::Parser;
use dist_agent_lang::runtime::values::Value;
use dist_agent_lang::runtime::Runtime;

fn parse_and_execute(source: &str) -> Result<Value, String> {
    let lexer = Lexer::new(source);
    let tokens_with_pos = lexer
        .tokenize_with_positions_immutable()
        .map_err(|e| format!("Lexer error: {:?}", e))?;
    let mut parser = Parser::new_with_positions(tokens_with_pos);
    let program = parser
        .parse()
        .map_err(|e| format!("Parser error: {:?}", e))?;
    let mut runtime = Runtime::new();
    runtime
        .execute_program(program)
        .map_err(|e| format!("Runtime error: {:?}", e))?
        .ok_or_else(|| "No return value".to_string())
}

#[test]
fn test_break_in_while_loop() {
    let source = r#"
        let count = 0;
        while (count < 10) {
            count = count + 1;
            if (count == 5) {
                break;
            }
        }
        count
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_continue_in_while_loop() {
    let source = r#"
        let sum = 0;
        let count = 0;
        while (count < 10) {
            count = count + 1;
            if (count % 2 == 0) {
                continue;
            }
            sum = sum + count;
        }
        sum
    "#;

    let result = parse_and_execute(source).unwrap();
    // Sum of odd numbers 1+3+5+7+9 = 25
    assert_eq!(result, Value::Int(25));
}

#[test]
fn test_break_with_value() {
    let source = r#"
        let result = 0;
        while (true) {
            result = result + 1;
            if (result == 42) {
                break result;
            }
        }
        result
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_loop_statement() {
    let source = r#"
        let count = 0;
        loop {
            count = count + 1;
            if (count == 10) {
                break;
            }
        }
        count
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn test_continue_in_for_loop() {
    let source = r#"
        let sum = 0;
        for item in [1, 2, 3, 4, 5] {
            if (item % 2 == 0) {
                continue;
            }
            sum = sum + item;
        }
        sum
    "#;

    let result = parse_and_execute(source).unwrap();
    // Sum of odd numbers: 1 + 3 + 5 = 9
    assert_eq!(result, Value::Int(9));
}

#[test]
fn test_break_in_for_loop() {
    let source = r#"
        let count = 0;
        for item in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
            count = count + 1;
            if (item == 5) {
                break;
            }
        }
        count
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_match_literal_pattern() {
    let source = r#"
        let x = 42;
        match x {
            42 => 100,
            10 => 200,
            default => 0
        }
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Int(100));
}

#[test]
fn test_match_string_pattern() {
    let source = r#"
        let status = "success";
        match status {
            "success" => 1,
            "error" => 0,
            default => -1
        }
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Int(1));
}

#[test]
fn test_match_identifier_pattern() {
    let source = r#"
        let value = 42;
        match value {
            x => x * 2
        }
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Int(84));
}

#[test]
fn test_match_wildcard_pattern() {
    let source = r#"
        let value = 42;
        match value {
            _ => 100
        }
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Int(100));
}

#[test]
fn test_match_range_pattern() {
    let source = r#"
        let score = 85;
        match score {
            90..100 => "A",
            80..89 => "B",
            70..79 => "C",
            default => "F"
        }
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::String("B".to_string()));
}

#[test]
fn test_match_default_case() {
    let source = r#"
        let value = 999;
        match value {
            1 => "one",
            2 => "two",
            default => "other"
        }
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::String("other".to_string()));
}

#[test]
fn test_match_no_default() {
    let source = r#"
        let value = 999;
        match value {
            1 => "one",
            2 => "two"
        }
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_nested_loops_with_break() {
    let source = r#"
        let outer = 0;
        let inner = 0;
        while (outer < 5) {
            outer = outer + 1;
            while (inner < 10) {
                inner = inner + 1;
                if (inner == 3) {
                    break;
                }
            }
            if (outer == 2) {
                break;
            }
        }
        outer
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Int(2));
}

#[test]
fn test_break_in_match_case() {
    let source = r#"
        let result = 0;
        let value = 5;
        while (result < 10) {
            result = result + 1;
            match value {
                5 => break result,
                default => continue
            }
        }
        result
    "#;

    let result = parse_and_execute(source).unwrap();
    assert_eq!(result, Value::Int(1));
}

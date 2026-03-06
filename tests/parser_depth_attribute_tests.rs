//! Parser depth and attribute regression tests.
//!
//! Tests for:
//! - Deep nesting in statement expressions (if, while, etc.) respects MAX_RECURSION_DEPTH
//! - Non-@route attributes on nested functions are not dropped
//! - @txn with params parses correctly
//!
//! Run with: cargo test --test parser_depth_attribute_tests

use dist_agent_lang::parser::ast::{BlockStatement, Statement};
use dist_agent_lang::{parse_source, Lexer, Parser};

/// Deeply nested parens in if condition should hit MAX_RECURSION_DEPTH, not stack overflow.
/// Use 8MB stack so recursion limit triggers before stack overflow (like fuzz_regression_tests).
#[test]
fn test_deep_nesting_in_if_condition_fails_gracefully() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let nested = "(".repeat(110) + "1" + &")".repeat(110);
            let source = format!("fn main() {{ if {} {{ }} }}", nested);
            let lexer = Lexer::new(&source);
            let tokens = lexer.tokenize_with_positions_immutable().unwrap();
            let mut parser = Parser::new_with_positions(tokens);
            let (program, errors) = parser.parse_with_recovery();
            // Should get recursion error or parse without panic
            assert!(
                !errors.is_empty() || program.statements.is_empty(),
                "Deep nesting should fail with error or empty program"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Simple @txn(1, 2) parses and attaches to function.
#[test]
fn test_txn_attribute_with_params() {
    let source = r#"@txn("read_committed", 5000) fn foo() { return 1; }"#;
    let program = parse_source(source).unwrap();
    let func = program
        .statements
        .iter()
        .find_map(|s| {
            if let Statement::Function(f) = s {
                Some(f)
            } else {
                None
            }
        })
        .expect("should have function");
    assert!(
        func.attributes.iter().any(|a| a.name == "@txn"),
        "function should have @txn attribute"
    );
    assert_eq!(
        func.attributes[0].parameters.len(),
        2,
        "@txn should have 2 params"
    );
}

fn find_nested_function_with_name<'a>(
    program: &'a dist_agent_lang::parser::ast::Program,
    name: &str,
) -> Option<&'a dist_agent_lang::parser::ast::FunctionStatement> {
    fn find_in_stmt<'a>(
        stmt: &'a Statement,
        name: &str,
    ) -> Option<&'a dist_agent_lang::parser::ast::FunctionStatement> {
        match stmt {
            Statement::Function(f) if f.name == name => Some(f),
            Statement::Function(f) => find_in_block(&f.body, name),
            Statement::Try(t) => find_in_block(&t.try_block, name)
                .or_else(|| {
                    t.catch_blocks
                        .iter()
                        .find_map(|c| find_in_block(&c.body, name))
                })
                .or_else(|| {
                    t.finally_block
                        .as_ref()
                        .and_then(|b| find_in_block(b, name))
                }),
            Statement::Block(b) => find_in_block(b, name),
            Statement::If(i) => find_in_block(&i.consequence, name)
                .or_else(|| i.alternative.as_ref().and_then(|a| find_in_block(a, name))),
            Statement::While(w) => find_in_block(&w.body, name),
            Statement::ForIn(f) => find_in_block(&f.body, name),
            Statement::Match(m) => m
                .cases
                .iter()
                .find_map(|c| find_in_block(&c.body, name))
                .or_else(|| m.default_case.as_ref().and_then(|d| find_in_block(d, name))),
            _ => None,
        }
    }
    fn find_in_block<'a>(
        block: &'a BlockStatement,
        name: &str,
    ) -> Option<&'a dist_agent_lang::parser::ast::FunctionStatement> {
        for stmt in &block.statements {
            if let Some(f) = find_in_stmt(stmt, name) {
                return Some(f);
            }
        }
        None
    }
    for stmt in &program.statements {
        if let Some(f) = find_in_stmt(stmt, name) {
            return Some(f);
        }
    }
    None
}

/// @txn fn inside try block should retain @txn attribute (generalized workaround).
#[test]
fn test_txn_attribute_on_nested_function() {
    let source = r#"
        fn outer() {
            try {
                @txn
                fn inner() { return 1; }
            } catch { return 0; }
        }
    "#;
    let program = parse_source(source).unwrap();
    let inner = find_nested_function_with_name(&program, "inner");
    assert!(inner.is_some(), "inner function should be parsed");
    let attrs: Vec<&str> = inner
        .unwrap()
        .attributes
        .iter()
        .map(|a| a.name.as_str())
        .collect();
    assert!(
        attrs.contains(&"@txn"),
        "inner should have @txn attribute, got {:?}",
        attrs
    );
}

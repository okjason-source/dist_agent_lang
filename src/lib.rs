// dist_agent_lang library
// This file makes the project available as a library

pub mod cli;
pub mod cli_design;
pub mod lexer;
pub mod mold;
pub mod parser;
pub mod runtime;
pub mod stdlib;
pub mod testing;
pub mod performance;
pub mod http_server;
pub mod http_server_security;
pub mod http_server_security_middleware;
pub mod http_server_middleware;
pub mod http_server_converters;
pub mod http_server_handlers;
pub mod http_server_integration;
pub mod ffi;
pub mod solidity_converter;

// Re-export security modules for easier access
pub use http_server_security::{RateLimiter, RequestSizeLimiter, InputValidator, SecurityLogger};
pub use ffi::security::{FFIInputValidator, FFIResourceLimits};

// Re-export main components for easy access
pub use lexer::{Lexer, tokens::Token};
pub use parser::{Parser, ast, error::ParserError};
pub use runtime::{Runtime, values::Value};
pub use ffi::{FFIInterface, FFIConfig, InterfaceType};

// Re-export testing framework for app developers: use dist_agent_lang::{TestCase, TestSuite, ...}
pub use testing::{
    TestCase, TestSuite, TestResult, TestStatus, TestConfig, TestRunner,
    MockFunction, MockRegistry, MockBuilder,
};

// For external integrations
pub fn parse_source(source: &str) -> Result<ast::Program, Box<dyn std::error::Error>> {
    // Phase 2: Input size limit - prevent DoS via oversized source code
    const MAX_SOURCE_SIZE: usize = 10 * 1024 * 1024; // 10MB
    if source.len() > MAX_SOURCE_SIZE {
        return Err(format!(
            "Source code too large: {} bytes (max: {} bytes)",
            source.len(),
            MAX_SOURCE_SIZE
        ).into());
    }
    
    let lexer = Lexer::new(source);
    let tokens_with_pos = lexer
        .tokenize_with_positions_immutable()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    
    // Phase 2: Token count limit - prevent DoS via excessive tokens
    const MAX_TOKENS: usize = 1_000_000; // 1M tokens
    if tokens_with_pos.len() > MAX_TOKENS {
        return Err(format!(
            "Too many tokens: {} (max: {})",
            tokens_with_pos.len(),
            MAX_TOKENS
        ).into());
    }
    
    let mut parser = Parser::new_with_positions(tokens_with_pos);
    parser.parse().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

pub fn execute_source(source: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let program = parse_source(source)?;
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(result.unwrap_or(Value::Null))
}

/// Execute DAL code with pre-set scope variables (e.g. agent_id for lifecycle hooks).
pub fn execute_dal_with_scope(
    vars: &std::collections::HashMap<String, Value>,
    source: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let program = parse_source(source)?;
    let mut runtime = Runtime::new();
    for (k, v) in vars {
        runtime.set_variable(k.clone(), v.clone());
    }
    let result = runtime.execute_program(program).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(result.unwrap_or(Value::Null))
}

/// Execute DAL source and return (user_functions, scope) for HTTP server runtime factory.
/// Use when you need to serve HTTP routes whose handlers are defined in DAL.
/// The returned runtime state can be used with create_router_with_runtime_factory.
pub fn execute_dal_and_extract_handlers(
    source: &str,
) -> Result<
    (
        std::collections::HashMap<String, runtime::engine::UserFunction>,
        runtime::scope::Scope,
    ),
    Box<dyn std::error::Error>,
> {
    let program = parse_source(source)?;
    let mut runtime = Runtime::new();
    runtime.execute_program(program).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok((runtime.user_functions.clone(), runtime.scope.clone()))
}

// dist_agent_lang library
// This file makes the project available as a library

// Allow lints that need larger refactors or many mechanical fixes; address incrementally.
#![allow(clippy::result_large_err)]
#![allow(clippy::type_complexity)]
#![allow(clippy::module_inception)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::new_without_default)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::manual_strip)]
#![allow(clippy::inconsistent_digit_grouping)]
#![allow(clippy::redundant_pattern_matching)]
#![allow(clippy::for_kv_map)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::get_first)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::unnecessary_to_owned)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::unwrap_or_default)]
#![allow(clippy::len_zero)]
#![allow(clippy::manual_inspect)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::map_identity)]
#![allow(clippy::needless_return)]
#![allow(clippy::question_mark)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::useless_format)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::explicit_auto_deref)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::implicit_saturating_sub)]
#![allow(clippy::io_other_error)]
#![allow(clippy::let_and_return)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::manual_flatten)]
#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::search_is_some)]
#![allow(clippy::single_char_add_str)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::wildcard_in_or_patterns)]

pub mod cli;
pub mod cli_design;
pub mod ffi;
pub mod http_server;
pub mod http_server_converters;
pub mod http_server_handlers;
pub mod http_server_integration;
pub mod http_server_middleware;
pub mod http_server_security;
pub mod http_server_security_middleware;
pub mod lexer;
pub mod mold;
pub mod parser;
pub mod performance;
pub mod runtime;
pub mod solidity_converter;
pub mod stdlib;
pub mod testing;

// Re-export security modules for easier access
pub use ffi::security::{FFIInputValidator, FFIResourceLimits};
pub use http_server_security::{InputValidator, RateLimiter, RequestSizeLimiter, SecurityLogger};

// Re-export main components for easy access
pub use ffi::{FFIConfig, FFIInterface, InterfaceType};
pub use lexer::{tokens::Token, Lexer};
pub use parser::{ast, error::ParserError, Parser};
pub use runtime::{values::Value, Runtime};

// Re-export testing framework for app developers: use dist_agent_lang::{TestCase, TestSuite, ...}
pub use testing::{
    MockBuilder, MockFunction, MockRegistry, TestCase, TestConfig, TestResult, TestRunner,
    TestStatus, TestSuite,
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
        )
        .into());
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
        )
        .into());
    }

    let mut parser = Parser::new_with_positions(tokens_with_pos);
    parser
        .parse()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

pub fn execute_source(source: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let program = parse_source(source)?;
    let mut runtime = Runtime::new();
    let result = runtime
        .execute_program(program)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
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
    let result = runtime
        .execute_program(program)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
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
    runtime
        .execute_program(program)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok((runtime.user_functions.clone(), runtime.scope.clone()))
}

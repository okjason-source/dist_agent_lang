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

pub mod agent_context_schema;
pub mod cli;
pub mod cli_design;
pub mod compile;
pub mod dal_summary;
pub mod ffi;
pub mod fleet;
pub mod http_server;
pub mod http_server_converters;
pub mod http_server_handlers;
pub mod http_server_integration;
pub mod http_server_middleware;
pub mod http_server_security;
pub mod http_server_security_middleware;
pub mod lexer;
pub mod manifest;
pub mod module_resolver;
pub mod mold;
pub mod parser;
pub mod performance;
pub mod project_init;
pub mod registry;
pub mod reporting;
pub mod runtime;
pub mod skills;
pub mod solidity_converter;
pub mod stdlib;
pub mod testing;
pub mod venv;

// Re-export security modules for easier access
pub use ffi::security::{FFIInputValidator, FFIResourceLimits};
pub use http_server_security::{InputValidator, RateLimiter, RequestSizeLimiter, SecurityLogger};

// Re-export main components for easy access
pub use ffi::{FFIConfig, FFIInterface, InterfaceType};
pub use lexer::{tokens::Token, Lexer};
pub use parser::{ast, collect_warnings, error::ParserError, ParseWarning, Parser};
pub use runtime::{values::Value, Runtime};

// Module resolution (M2)
pub use module_resolver::{
    resolve_imports, ModuleResolver, ResolveError, ResolvedImport, ResolvedImportEntry,
};

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
        .execute_program(program, None)
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
        .execute_program(program, None)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(result.unwrap_or(Value::Null))
}

/// Execute a DAL file (e.g. agent.dal for serve behavior). Resolves imports when present.
/// Used by `dal agent serve --behavior path` so the script can spawn an agent and call agent::set_serve_agent(agent_id).
pub fn execute_dal_file(path: &str) -> Result<(), String> {
    use parser::ast::Statement;
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let program = parse_source(&source).map_err(|e| format!("Parse error: {}", e))?;
    let has_imports = program
        .statements
        .iter()
        .any(|s| matches!(s, Statement::Import(_)));
    let mut runtime = Runtime::new();
    if has_imports {
        let entry_path = std::path::Path::new(path);
        let entry_dir = entry_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let manifest_path = entry_dir.join("dal.toml");
        let mut resolver =
            module_resolver::ModuleResolver::new().with_root_dir(entry_dir.to_path_buf());
        if manifest_path.exists() {
            if let Ok(deps) = manifest::load_resolved_deps(&manifest_path) {
                resolver = resolver.with_dependencies(deps);
            }
        }
        let parse_fn = |s: &str| parse_source(s).map_err(|e| e.to_string());
        let resolved = resolver
            .resolve_program_with_cycles(&program, Some(entry_path), parse_fn)
            .map_err(|e| e.to_string())?;
        runtime
            .execute_program(program, Some(&resolved))
            .map_err(|e| format!("Runtime error: {}", e))?;
    } else {
        runtime
            .execute_program(program, None)
            .map_err(|e| format!("Runtime error: {}", e))?;
    }
    Ok(())
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
        .execute_program(program, None)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok((runtime.user_functions.clone(), runtime.scope.clone()))
}

/// Like execute_dal_and_extract_handlers but resolves imports using entry_path.
/// Use this for `dal serve <file>` so that handlers and their imports (e.g. workflows.dal) load correctly and all @route handlers are registered.
pub fn execute_dal_and_extract_handlers_with_path(
    source: &str,
    entry_path: &std::path::Path,
) -> Result<
    (
        std::collections::HashMap<String, runtime::engine::UserFunction>,
        runtime::scope::Scope,
    ),
    Box<dyn std::error::Error>,
> {
    use parser::ast::Statement;
    let program = parse_source(source)?;
    let has_imports = program
        .statements
        .iter()
        .any(|s| matches!(s, Statement::Import(_)));
    let mut runtime = Runtime::new();
    if has_imports {
        let entry_dir = entry_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let manifest_path = entry_dir.join("dal.toml");
        let mut resolver =
            module_resolver::ModuleResolver::new().with_root_dir(entry_dir.to_path_buf());
        if manifest_path.exists() {
            if let Ok(deps) = manifest::load_resolved_deps(&manifest_path) {
                resolver = resolver.with_dependencies(deps);
            }
        }
        let parse_fn = |s: &str| parse_source(s).map_err(|e| e.to_string());
        let resolved = resolver
            .resolve_program_with_cycles(&program, Some(entry_path), parse_fn)
            .map_err(|e| {
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                    as Box<dyn std::error::Error>
            })?;
        runtime
            .execute_program(program, Some(&resolved))
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    } else {
        runtime
            .execute_program(program, None)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    }
    Ok((runtime.user_functions.clone(), runtime.scope.clone()))
}

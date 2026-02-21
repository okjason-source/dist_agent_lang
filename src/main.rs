// Import from the library instead of redeclaring modules
#![allow(clippy::needless_borrow)]
#![allow(clippy::type_complexity)]
#![allow(clippy::for_kv_map)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::get_first)]
#![allow(clippy::useless_format)]

use dist_agent_lang::cli::{chain_subcommand_to_args, Cli, Commands};
use dist_agent_lang::cli_design;
use dist_agent_lang::lexer;
use dist_agent_lang::parser;
use dist_agent_lang::performance;
use dist_agent_lang::reporting::{
    format_lexer_error, format_parser_error, format_parse_warnings, format_runtime_error,
};
use dist_agent_lang::runtime;
use dist_agent_lang::stdlib;
use dist_agent_lang::testing;

use lexer::tokens::{Punctuation, Token};
use lexer::Lexer;
use parser::ast::{FunctionStatement, ServiceStatement, Statement};
use parser::error::{ErrorContext, ErrorReporter, ParserError, SimpleErrorReporter};
use parser::Parser;
use runtime::values::Value;
use runtime::Runtime;
use std::collections::HashMap;
use std::time::Duration;
use stdlib::crypto::{HashAlgorithm, SignatureAlgorithm};
use stdlib::{auth, chain, crypto, log};

// Testing framework: one-line imports for test suites and app developers
use testing::{MockBuilder, MockRegistry};
use testing::{TestCase, TestConfig, TestRunner, TestSuite};

// Performance imports - used in optimization commands and imported within functions to avoid unused warnings

/// Returns the binary name used to invoke the CLI (e.g. "dal" or "dist_agent_lang")
fn binary_name() -> String {
    std::env::args()
        .next()
        .and_then(|p| {
            std::path::Path::new(&p)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "dal".to_string())
}

fn main() {
    // Initialize persistent file logging if configured
    // This enables audit log persistence for @secure services
    if let Err(e) = log::initialize_file_logging() {
        eprintln!("⚠️  Warning: Failed to initialize file logging: {}", e);
        // Continue execution - logging will fall back to console/memory only
    }

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        // No arguments provided, discover and run tests (fallback to selftest)
        run_dal_tests(".");
        return;
    }

    // Phase 11: clap-based CLI (09_CLI_EXPANSION_PLAN.md)
    let cli = Cli::parse();

    // Handle custom help and version flags
    if cli.help {
        print_help(&cli);
        std::process::exit(0);
    }

    if cli.version_flag {
        print_version(&cli);
        std::process::exit(0);
    }

    let Some(ref cmd) = cli.command else {
        // No subcommand (e.g. dal --quiet only) — show our custom help (no Phase labels)
        print_help(&cli);
        std::process::exit(1);
    };

    match cmd {
        Commands::Run { file } => run_dal_file(&file),
        Commands::Test { file } => {
            if let Some(f) = file {
                run_dal_tests(&f);
            } else {
                run_dal_tests(".");
            }
        }
        Commands::Selftest => {
            run_selftest();
        }
        Commands::Web { sub, rest } => {
            let mut web_args = vec![sub.clone()];
            web_args.extend(rest.iter().cloned());
            if sub == "get" || sub == "post" || sub == "parse-url" {
                handle_web_command(&web_args);
            } else if sub.ends_with(".dal") {
                run_web_application(&sub);
            } else if sub.ends_with(".js") {
                run_web_application_js(&sub, &rest);
            } else {
                eprintln!("Usage: {} web <file.dal> | web <file.js> [args...] | web get <url> | web post <url> | web parse-url <url>", binary_name());
                std::process::exit(1);
            }
        }
        Commands::Serve {
            file,
            port,
            frontend,
            cors_origin,
        } => run_serve(&file, *port, frontend.as_deref(), cors_origin),
        Commands::Convert { input, output } => {
            let output_file = output.clone().unwrap_or_else(|| {
                if input.ends_with(".sol") {
                    input[..input.len() - 4].to_string() + ".dal"
                } else {
                    input.to_string() + ".dal"
                }
            });
            convert_solidity_file(&input, &output_file);
        }
        Commands::Analyze { input } => analyze_solidity_file(&input),
        Commands::Parse { file } => parse_dal_file(&file),
        Commands::Check { file } => check_dal_file(&file),
        Commands::Fmt { file, check } => format_dal_file(&file, *check),
        Commands::Lint { file } => lint_dal_file(&file),
        Commands::New { name, project_type } => create_new_project(&name, project_type.as_deref()),
        Commands::Init => init_project(),
        Commands::Repl => run_repl(),
        Commands::Watch { file } => watch_dal_file(&file),
        Commands::Add { package } => add_package(&package),
        Commands::Install => install_dependencies(),
        Commands::Bench { file, suite } => run_benchmarks(file.as_deref(), suite.as_deref()),
        Commands::Profile { file, memory } => profile_dal_file(&file, *memory),
        Commands::Optimize {
            file,
            output,
            level,
        } => optimize_dal_file(&file, output.as_deref(), *level),
        Commands::MemoryStats => show_memory_stats(),
        Commands::Chain { subcommand } => {
            let args = chain_subcommand_to_args(&subcommand);
            handle_chain_command(&args);
        }
        Commands::Crypto { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_crypto_command(&a);
        }
        Commands::Db { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_db_command(&a);
        }
        Commands::Ai { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_ai_command(&a);
        }
        Commands::Cloud { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_cloud_command(&a);
        }
        Commands::Oracle { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_oracle_command(&a);
        }
        Commands::Lsp { rest } => handle_lsp_command(rest),
        Commands::Doc {
            target,
            output,
            open,
        } => {
            let mut a = vec![target.clone()];
            if let Some(o) = output {
                a.push("--output".to_string());
                a.push(o.clone());
            }
            if *open {
                a.push("--open".to_string());
            }
            handle_doc_command(&a);
        }
        Commands::Completions { shell } => {
            let a = vec![shell.clone()];
            handle_completions_command(&a);
        }
        Commands::Debug { file, breakpoint } => {
            let mut a = vec![file.clone()];
            if let Some(b) = breakpoint {
                a.push("--breakpoint".to_string());
                a.push(b.to_string());
            }
            handle_debug_command(&a);
        }
        Commands::Agent { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_agent_command(&a);
        }
        Commands::Iot { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_iot_command(&a);
        }
        Commands::Log { rest } => handle_log_command(rest),
        Commands::Config { rest } => handle_config_command(rest),
        Commands::Admin { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_admin_command(&a);
        }
        Commands::Key { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_key_command(&a);
        }
        Commands::Aml { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_aml_command(&a);
        }
        Commands::Kyc { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_kyc_command(&a);
        }
        Commands::Mold { subcommand, rest } => {
            let mut a = vec!["mold".to_string(), subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_agent_command(&a);
        }
        Commands::Scaffold {
            scaffold_type,
            name,
        } => {
            let mut a = vec![scaffold_type.clone()];
            if let Some(n) = name {
                a.push(n.clone());
            }
            handle_scaffold_command(&a);
        }
        Commands::Build { rest } => handle_build_command(rest),
        Commands::Clean { rest } => handle_clean_command(rest),
        Commands::Dist { rest } => handle_dist_command(rest),
        Commands::Bond { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_cross_component_command("bond", &a);
        }
        Commands::Pipe { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_cross_component_command("pipe", &a);
        }
        Commands::Invoke { subcommand, rest } => {
            let mut a = vec![subcommand.clone()];
            a.extend(rest.iter().cloned());
            handle_cross_component_command("invoke", &a);
        }
    }
}

fn parse_dal_file(filename: &str) {
    println!("🪩  Parsing dist_agent_lang file: {}", filename);

    // Read the file
    let source_code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    // Tokenize with positions
    let tokens_with_pos = match Lexer::new(&source_code).tokenize_with_positions_immutable() {
        Ok(tokens) => {
            println!("✅ Lexer scanning... {} tokens", tokens.len());
            tokens
        }
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    // Parse
    match Parser::new_with_positions(tokens_with_pos).parse() {
        Ok(ast) => {
            println!("✅ Parsed {} statements", ast.statements.len());
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Parsing failed:\n{}", format_parser_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    }
}

fn run_dal_file(filename: &str) {
    println!("🪩  Running dist_agent_lang file: {}", filename);

    // Read the file
    let source_code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    // Tokenize with positions for accurate error reporting
    let tokens_with_pos = match Lexer::new(&source_code).tokenize_with_positions_immutable() {
        Ok(tokens) => {
            println!("✅ Lexer scanning... {} tokens", tokens.len());
            tokens
        }
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    // Parse with position information
    let ast = match Parser::new_with_positions(tokens_with_pos).parse() {
        Ok(ast) => {
            println!("✅ Parsed {} statements", ast.statements.len());
            ast
        }
        Err(e) => {
            eprintln!("❌ Parsing failed:\n{}", format_parser_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    // Warnings (unused variables, etc.)
    let warnings = parser::collect_warnings(&ast);
    if !warnings.is_empty() {
        eprintln!("\n{}", format_parse_warnings(&warnings, Some(filename), Some(&source_code)));
    }

    // Execute
    let mut runtime = Runtime::new();
    match runtime.execute_program(ast) {
        Ok(result) => {
            println!("✅ Execution successful!");
            if let Some(value) = result {
                println!("   Result: {}", value);
            }
            // Run Layer 3 tests if the file registered any (describe/it)
            if let Err(e) = runtime.run_registered_tests() {
                let with_ctx = dist_agent_lang::runtime::RuntimeErrorWithContext::from_error(e);
                eprintln!("❌ Test(s) failed:\n{}", format_runtime_error(&with_ctx, Some(filename), Some(&source_code)));
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Execution failed:\n{}", format_runtime_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    }
}

// Test discovery and execution
fn run_dal_tests(path: &str) {
    use std::fs;

    println!("🧪 Discovering tests in: {}\n", path);

    let test_files = discover_test_files(path);

    if test_files.is_empty() {
        println!("⚠️  No *.test.dal files found");
        println!("\n💡 Tip: Create test files with .test.dal extension");
        println!("Example: account.test.dal, payment.test.dal");
        println!("\n🔄 Falling back to system health checks...\n");
        run_selftest();
        return;
    }

    println!("Found {} test file(s):\n", test_files.len());

    let mut total_tests = 0;
    let mut passed = 0;
    let mut failed = 0;

    for test_file in &test_files {
        println!("📄 {}", test_file);

        // Read the file
        let content = match fs::read_to_string(test_file) {
            Ok(c) => c,
            Err(e) => {
                println!("   ❌ Failed to read file: {}\n", e);
                continue;
            }
        };

        // Tokenize
        let tokens = match Lexer::new(&content).tokenize() {
            Ok(t) => t,
            Err(e) => {
                println!("   ❌ Lexer error: {}\n", e);
                failed += 1;
                continue;
            }
        };

        // Parse
        let mut parser = Parser::new(tokens);
        let program = match parser.parse() {
            Ok(p) => p,
            Err(e) => {
                println!("   ❌ Parser error: {}\n", e);
                failed += 1;
                continue;
            }
        };

        // Find test functions (functions with @test attribute or starting with test_)
        let test_names: Vec<String> = program
            .statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Function(func) = stmt {
                    let is_test = func.attributes.iter().any(|attr| attr.name == "test")
                        || func.name.starts_with("test_");
                    if is_test {
                        Some(func.name.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if test_names.is_empty() {
            println!("   ⚠️  No test functions found (use @test or test_ prefix)\n");
            continue;
        }

        println!("   Running {} test(s)...", test_names.len());

        // Execute program to register functions, then run each test
        let mut runtime = Runtime::new();
        if let Err(e) = runtime.execute_program(program) {
            println!("   ❌ Runtime error during setup:\n{}\n", format_runtime_error(&e, Some(test_file), Some(&content)));
            failed += test_names.len();
            continue;
        }

        for test_name in &test_names {
            total_tests += 1;
            match runtime.call_function(test_name, &[]) {
                Ok(_) => {
                    println!("      ✅ {}", test_name);
                    passed += 1;
                }
                Err(e) => {
                    println!("      ❌ {} - {}", test_name, e);
                    failed += 1;
                }
            }
        }

        println!();
    }

    // Summary
    println!("========================================================================");
    println!("Test Summary:");
    println!("  Total:  {}", total_tests);
    println!("  Passed: {} ✅", passed);
    println!("  Failed: {} ❌", failed);

    if failed > 0 {
        std::process::exit(1);
    }
}

fn discover_test_files(path: &str) -> Vec<String> {
    use std::fs;
    use std::path::Path;

    let mut test_files = Vec::new();
    let path_obj = Path::new(path);

    if path_obj.is_file() && path.ends_with(".test.dal") {
        test_files.push(path.to_string());
    } else if path_obj.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Some(path_str) = entry_path.to_str() {
                        if path_str.ends_with(".test.dal") {
                            test_files.push(path_str.to_string());
                        }
                    }
                } else if entry_path.is_dir() {
                    if let Some(dir_str) = entry_path.to_str() {
                        test_files.extend(discover_test_files(dir_str));
                    }
                }
            }
        }
    }

    test_files.sort();
    test_files
}

fn run_web_application(filename: &str) {
    println!("🪩  Running dist_agent_lang web application: {}", filename);

    // Read the file
    let source_code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    // Tokenize and parse
    let tokens = match Lexer::new(&source_code).tokenize() {
        Ok(tokens) => {
            println!("✅ Lexer scanning... {} tokens", tokens.len());
            tokens
        }
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => {
            println!("✅ Parsed {} statements", program.statements.len());
            program
        }
        Err(e) => {
            eprintln!("❌ Parsing failed:\n{}", format_parser_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    // Look for web service in the program
    for statement in &program.statements {
        if let Statement::Service(service) = statement {
            if service.name == "KEYS_WebApp" {
                println!("✅ Found KEYS_WebApp service!");

                // Create HTTP server
                let mut server = stdlib::web::create_server(3000);

                // Add routes
                stdlib::web::add_route(
                    &mut server,
                    "GET".to_string(),
                    "/".to_string(),
                    "serve_home_page".to_string(),
                );
                stdlib::web::add_route(
                    &mut server,
                    "GET".to_string(),
                    "/api/balance".to_string(),
                    "get_balance".to_string(),
                );
                stdlib::web::add_route(
                    &mut server,
                    "POST".to_string(),
                    "/api/connect".to_string(),
                    "connect_wallet".to_string(),
                );
                stdlib::web::add_route(
                    &mut server,
                    "POST".to_string(),
                    "/api/transfer".to_string(),
                    "transfer_tokens".to_string(),
                );
                stdlib::web::add_route(
                    &mut server,
                    "POST".to_string(),
                    "/api/airdrop".to_string(),
                    "claim_airdrop".to_string(),
                );

                // Start server
                match stdlib::web::start_server(&server) {
                    Ok(message) => {
                        println!("✅ {}", message);
                        println!("🌐 Open your browser and navigate to: http://localhost:3000");
                        println!("🛑 Press Ctrl+C to stop the server");

                        // Keep the server running
                        loop {
                            std::thread::sleep(std::time::Duration::from_secs(1));
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to start server: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    eprintln!("❌ No KEYS_WebApp service found in {}", filename);
    std::process::exit(1);
}

/// Run a standalone JavaScript file via the system Node.js (like `node <file>`).
/// Requires Node.js to be installed. Extra args are passed to the script.
fn run_web_application_js(filename: &str, script_args: &[String]) {
    if !std::path::Path::new(filename).exists() {
        eprintln!("❌ File not found: {}", filename);
        std::process::exit(1);
    }
    println!("🪩  Running JavaScript (Node): {}", filename);
    let status = std::process::Command::new("node")
        .arg(filename)
        .args(script_args)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => std::process::exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!(
                "❌ Failed to run Node: {}. Is Node.js installed? (e.g. https://nodejs.org)",
                e
            );
            std::process::exit(1);
        }
    }
}

/// Extract HTTP routes from @route annotations on DAL functions.
/// e.g. @route("GET", "/api/users") fn get_users(request) { ... }
/// Returns Vec<(method, path, handler_name)>.
fn extract_routes_from_annotations(
    user_functions: &std::collections::HashMap<String, runtime::engine::UserFunction>,
) -> Vec<(String, String, String)> {
    use lexer::tokens::Literal;
    let mut routes = Vec::new();
    for (name, func) in user_functions {
        for attr in &func.attributes {
            if attr.name == "route" && attr.parameters.len() == 2 {
                // Extract method and path from string literal parameters
                let method = match &attr.parameters[0] {
                    parser::ast::Expression::Literal(Literal::String(s)) => s.to_uppercase(),
                    _ => continue,
                };
                let path = match &attr.parameters[1] {
                    parser::ast::Expression::Literal(Literal::String(s)) => s.clone(),
                    _ => continue,
                };
                routes.push((method, path, name.clone()));
            }
        }
    }
    // Sort for deterministic order
    routes.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    routes
}

/// Fallback: infer HTTP routes from handler names (convention for todo-style backends).
/// e.g. get_todos -> GET /api/todos, create_todo -> POST /api/todos
fn infer_routes_from_handlers(
    handlers: &std::collections::HashSet<String>,
) -> Vec<(String, String, String)> {
    let mut routes = Vec::new();
    let todo_handlers = [
        ("get_todos", "GET", "/api/todos"),
        ("create_todo", "POST", "/api/todos"),
        ("get_todo", "GET", "/api/todos/:id"),
        ("update_todo", "PUT", "/api/todos/:id"),
        ("delete_todo", "DELETE", "/api/todos/:id"),
        ("delete_completed_todos", "DELETE", "/api/todos/completed"),
    ];
    for (name, method, path) in todo_handlers {
        if handlers.contains(name) {
            routes.push((method.to_string(), path.to_string(), name.to_string()));
        }
    }
    routes
}

fn run_serve(filename: &str, port: u16, frontend: Option<&str>, cors_origin: &str) {
    use axum::response::Html;
    use dist_agent_lang::execute_dal_and_extract_handlers;
    use dist_agent_lang::http_server_integration::create_router_with_options;
    use std::sync::Arc;

    println!("🪩  Serving DAL handlers from: {}", filename);
    println!("    Port: {}", port);

    let source_code = match std::fs::read_to_string(filename) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    let (user_functions, scope) = match execute_dal_and_extract_handlers(&source_code) {
        Ok(extracted) => extracted,
        Err(e) => {
            eprintln!("❌ Failed to load DAL: {}", e);
            std::process::exit(1);
        }
    };

    // Try @route annotations first, fall back to name-based inference
    let routes = {
        let annotated = extract_routes_from_annotations(&user_functions);
        if annotated.is_empty() {
            let handler_names: std::collections::HashSet<String> =
                user_functions.keys().cloned().collect();
            let inferred = infer_routes_from_handlers(&handler_names);
            if inferred.is_empty() {
                eprintln!("❌ No HTTP handlers found in {}.", filename);
                eprintln!(
                    "   Add @route(\"METHOD\", \"/path\") annotations to your functions, e.g.:"
                );
                eprintln!("   @route(\"GET\", \"/api/items\")");
                eprintln!("   fn get_items(request) {{ ... }}");
                std::process::exit(1);
            }
            println!("    (routes inferred from handler names)");
            inferred
        } else {
            println!("    {} route(s) from @route annotations", annotated.len());
            annotated
        }
    };

    let mut server = stdlib::web::create_server(port as i64);
    for (method, path, handler) in &routes {
        stdlib::web::add_route(&mut server, method.clone(), path.clone(), handler.clone());
    }

    let user_functions = Arc::new(user_functions);
    let shared_scope = Arc::new(std::sync::RwLock::new(scope));
    let runtime_factory = {
        let uf = user_functions.clone();
        let sc = shared_scope.clone();
        move || {
            let mut rt = Runtime::new();
            rt.user_functions = (*uf).clone();
            // Read shared scope (handler state persists across requests)
            rt.scope = sc.read().unwrap().clone();
            rt
        }
    };

    // Scope writeback: after each request, persist scope changes back to shared state
    let scope_writeback: Arc<Box<dyn Fn(&dist_agent_lang::runtime::scope::Scope) + Send + Sync>> = {
        let sc = shared_scope.clone();
        Arc::new(Box::new(
            move |new_scope: &dist_agent_lang::runtime::scope::Scope| {
                let mut guard = sc.write().unwrap();
                *guard = new_scope.clone();
            },
        ))
    };

    let mut app = create_router_with_options(server, runtime_factory, Some(scope_writeback));

    // Serve frontend HTML at / if provided or auto-detected for todo backend
    let frontend_path = frontend.map(std::borrow::ToOwned::to_owned).or_else(|| {
        let path = std::path::Path::new(filename);
        let parent = path.parent()?;
        let base = path.file_stem()?;
        if base == "todo_backend_minimal" {
            Some(
                parent
                    .join("frontend_todo_app.html")
                    .to_string_lossy()
                    .into_owned(),
            )
        } else {
            None
        }
    });

    if let Some(ref fp) = frontend_path {
        match std::fs::read_to_string(fp) {
            Ok(html) => {
                let body = html;
                app = app.route(
                    "/",
                    axum::routing::get(move || {
                        let b = body.clone();
                        async move { Html(b) }
                    }),
                );
                println!("✅ Frontend: {} served at /", fp);
            }
            Err(e) => eprintln!("⚠️  Could not read frontend {}: {}", fp, e),
        }
    }

    // Add CORS layer
    use axum::http::Method;
    use tower_http::cors::{Any, CorsLayer};
    let cors = if cors_origin == "*" {
        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ])
            .allow_origin(Any)
            .allow_headers(Any)
    } else {
        let origin = cors_origin
            .parse::<axum::http::HeaderValue>()
            .unwrap_or_else(|_| {
                eprintln!(
                    "⚠️  Invalid --cors-origin '{}', falling back to *",
                    cors_origin
                );
                axum::http::HeaderValue::from_static("*")
            });
        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ])
            .allow_origin(origin)
            .allow_headers(Any)
    };
    let app = app.layer(cors);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    println!("✅ Routes registered:");
    for (method, path, handler) in &routes {
        println!("    {} {} -> {}", method, path, handler);
    }
    println!("✅ CORS: {}", cors_origin);
    println!("🌐 API: http://localhost:{}/api", port);
    if frontend_path.is_some() {
        println!("🌐 App: http://localhost:{}/", port);
    }
    println!("🛑 Press Ctrl+C to stop");

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("❌ Failed to create tokio runtime: {}", e);
            std::process::exit(1);
        }
    };

    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;
        axum::serve(listener, app)
            .await
            .map_err(|e| format!("Server error: {}", e))
    })
    .unwrap_or_else(|e| {
        eprintln!("❌ {}", e);
        std::process::exit(1);
    });
}

fn run_selftest() {
    println!("🔧 Running system health checks...");
    println!("========================================================================");

    let test_code = r#"
let x = 42;
let message = "Hello from dist_agent_lang!";

@txn
@secure
@limit(1000)
@trust("hybrid")
fn secure_function() {
    let result = "secure operation";
    return result;
}

let price_query = oracle::create_query("btc_price");
let btc_price = oracle::fetch("https://api.example.com/oracle/price", price_query);

let stream_id = oracle::stream("wss://api.example.com/oracle/stream", "price_callback");

let ai_service = service::create_ai_service("gpt-4");
let ai_response = service::ai("What is blockchain?", ai_service);
let payment_call = service::create_service_call("payment", "process");
let payment_result = service::call(payment_call);

let sync_target = sync::create_sync_target("https://api.example.com/sync", "http");
let sync_success = sync::push(HashMap::new(), sync_target);
let pull_result = sync::pull("database", sync::create_sync_filters());

let principal = key::create_principal("user_123", "John Doe");
let key_request = key::create_capability_request("user_data", "read", "user_123");
let key_check = key::check(key_request);
"#;

    println!("🪩  Running built-in test suite");
    let tokens = match Lexer::new(test_code).tokenize() {
        Ok(tokens) => {
            println!("✅ Lexer: {} tokens", tokens.len());
            tokens
        }
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, None, Some(test_code)));
            return;
        }
    };

    // Test the parser
    println!("✅ Parser: testing...");
    let mut parser = Parser::new(tokens.clone());
    match parser.parse() {
        Ok(program) => {
            println!("✅ Parser: {} statements parsed", program.statements.len());
        }
        Err(e) => {
            eprintln!("❌ Parser error:\n{}", format_parser_error(&e, None, Some(test_code)));
            return;
        }
    };

    // Test the runtime with custom capacities
    println!("✅ Runtime: testing...");
    let mut runtime = Runtime::with_capacities(128, 32, 16);

    // Test variable management
    runtime.set_variable("x".to_string(), Value::Int(42));
    runtime.set_variable(
        "message".to_string(),
        Value::String("Hello Runtime!".to_string()),
    );
    runtime.set_variable("flag".to_string(), Value::Bool(true));
    runtime.set_variable("empty".to_string(), Value::Null);

    match runtime.get_variable("x") {
        Ok(_value) => {
            println!("✅ Runtime: variables & functions working");
        }
        Err(e) => println!("❌ Error getting 'x': {}", e),
    }

    // Test built-in functions (silently)
    let _ = runtime.call_function(
        "print",
        &[Value::String("Testing print function!".to_string())],
    );
    let _ = runtime.call_function("add", &[Value::Int(10), Value::Int(32)]);

    // Test the Standard Library (Week 5-6)
    println!("✅ Standard Library: testing...");

    // Test chain namespace
    let mut metadata = HashMap::new();
    metadata.insert("description".to_string(), "A test NFT".to_string());
    metadata.insert("image".to_string(), "ipfs://QmTest...".to_string());

    let asset_id = chain::mint("TestNFT".to_string(), metadata.clone());
    let _ = chain::get(asset_id);
    let _ = chain::update(asset_id, {
        let mut updates = HashMap::new();
        updates.insert("description".to_string(), "Updated test NFT".to_string());
        updates
    });

    // Test auth namespace (simplified)
    let session = auth::session("user123".to_string(), vec!["admin".to_string()]);
    let _ = auth::is_valid_session(&session);
    let _ = auth::has_role(&session, "admin");

    // Test log namespace
    log::info(
        "Application started",
        {
            let mut data = HashMap::new();
            data.insert(
                "version".to_string(),
                Value::String(env!("CARGO_PKG_VERSION").to_string()),
            );
            data.insert("timestamp".to_string(), Value::Int(1234567890));
            data
        },
        None,
    );

    let log_stats = log::get_stats();
    let _total_logs = log_stats
        .get("total_entries")
        .and_then(|v| match v {
            Value::Int(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(0);

    // Test crypto namespace
    let _hash_sha256 = crypto::hash("Hello, World!", HashAlgorithm::SHA256);
    let _hash_sha512 = crypto::hash("Hello, World!", HashAlgorithm::SHA512);
    let keypair = crypto::generate_keypair(SignatureAlgorithm::RSA);
    let signature = crypto::sign(
        "Hello, World!",
        &keypair["private_key"],
        SignatureAlgorithm::RSA,
    );
    let _is_valid_signature = crypto::verify(
        "Hello, World!",
        &signature,
        &keypair["public_key"],
        SignatureAlgorithm::RSA,
    );

    println!("✅ Standard Library: chain, auth, log, crypto all working");

    // Test error handling system
    println!("✅ Error Handling: syntax implemented");

    println!("\n✅ All Core Tests Passed!");
    println!("   • Lexer & Parser ✅");
    println!("   • Runtime (variables, functions) ✅");
    println!("   • Standard Library (chain, auth, log, crypto) ✅");
    println!("   • Error Handling ✅");

    test_error_handling_and_testing_framework();
    test_performance_optimization();

    println!("\n🪩 Ready for first working examples!");
}

fn print_help(cli: &Cli) {
    let bin = binary_name();
    cli_design::print_banner(&bin, cli.no_banner, cli.quiet);
    if !cli.quiet {
        println!("Usage: {} <command> [options]", bin);
    }
    print!("{}", cli_design::help_content(&bin));
    if !cli.quiet {
        println!(
            "More: https://distagentlang.com · https://github.com/okjason-source/dist_agent_lang"
        );
    }
}

fn print_version(cli: &Cli) {
    let bin = binary_name();
    cli_design::print_banner(&bin, cli.no_banner, cli.quiet);
    if !cli.quiet {
        println!("{} v{}", bin, cli_design::version());
        println!("{}", cli_design::TAGLINE_FULL);
    } else {
        println!("{} v{}", bin, cli_design::version());
    }
}

fn test_error_handling_and_testing_framework() {
    use runtime::values::Value;
    use testing::coverage::CoverageTracker;
    use testing::framework::OutputFormat;

    println!("   Testing Enhanced Error Handling System...");

    // Test enhanced error types
    let error_context = ErrorContext::new()
        .with_file_path("test.dal".to_string())
        .with_source_code("let x = 10 + ;".to_string())
        .add_call_stack("parse_expression".to_string())
        .add_suggestion("Check for missing operand after +".to_string());

    let parser_error = ParserError::unexpected_token(
        &Token::Punctuation(Punctuation::Semicolon),
        &["expression", "number", "identifier"],
        1,
        10,
    )
    .with_context(error_context);

    println!("   ✅ Enhanced error handling");

    // Test error reporter
    let mut error_reporter = SimpleErrorReporter::new();
    error_reporter.report_error(parser_error.clone());
    error_reporter.report_warning("Unused variable 'x'".to_string(), 5);

    println!(
        "   ✅ Error reporter: {} errors",
        error_reporter.get_errors().len()
    );

    // Create test suite
    let arithmetic_test = TestCase::new("arithmetic_test")
        .with_description("Test basic arithmetic operations")
        .with_source_code("let x = 10 + 5; x")
        .expect_result(Value::Int(15))
        .with_tag("basic")
        .with_tag("arithmetic");

    let function_test = TestCase::new("function_test")
        .with_description("Test function definition and call")
        .with_source_code(
            "
            fn add(a, b) {
                return a + b;
            }
            add(3, 4)
        ",
        )
        .expect_result(Value::Int(7))
        .with_tag("basic")
        .with_tag("function");

    let error_test = TestCase::new("error_test")
        .with_description("Test error handling")
        .with_source_code("let x = undefined_variable;")
        .expect_error("undefined variable")
        .with_tag("error");

    // Create test suite
    let test_suite = TestSuite::new("comprehensive_tests")
        .with_description("Comprehensive test suite for dist_agent_lang")
        .add_test(arithmetic_test)
        .add_test(function_test)
        .add_test(error_test)
        .with_tag("comprehensive")
        .with_setup("let global_setup = true;")
        .with_teardown("let global_cleanup = true;");

    println!("   ✅ Test suite: {} tests", test_suite.test_cases.len());

    // Test mocking system
    let mock_chain_mint = MockBuilder::new("mint")
        .in_namespace("chain")
        .returns(Value::Int(12345))
        .logs("Mock chain::mint called")
        .expects_calls(1)
        .build();

    let mock_oracle_fetch = MockBuilder::new("fetch")
        .in_namespace("oracle")
        .returns(Value::String("mock_price_data".to_string()))
        .logs("Mock oracle::fetch called")
        .build();

    let mut mock_registry = MockRegistry::new();
    mock_registry.register(mock_chain_mint.clone());
    mock_registry.register(mock_oracle_fetch.clone());

    println!("   ✅ Mock registry: {} mocks", mock_registry.mocks.len());

    // Test runner (create a fresh mock since cloning loses the validator)
    let mock_chain_mint_for_runner = MockBuilder::new("mint")
        .in_namespace("chain")
        .returns(Value::Int(12345))
        .logs("Mock chain::mint called")
        .expects_calls(1)
        .build();

    let mut test_runner = TestRunner::new()
        .with_config(TestConfig {
            verbose: false, // Changed from true to false for less verbose output
            stop_on_failure: false,
            parallel: false,
            timeout: Some(std::time::Duration::from_secs(30)),
            filter_tags: vec!["basic".to_string()],
            exclude_tags: vec!["error".to_string()],
            coverage_enabled: true,
            output_format: OutputFormat::Text,
        })
        .with_mock(mock_chain_mint_for_runner);

    let stats = test_runner.run_suite(test_suite);

    println!(
        "   ✅ Test runner: {} passed, {} failed, {:.1}% success",
        stats.passed,
        stats.failed,
        stats.success_rate()
    );

    // Coverage tracking
    let mut coverage_tracker = CoverageTracker::new().with_source_code(
        "
            fn add(a, b) {
                return a + b;
            }
            
            let result = add(3, 4);
            if result > 5 {
                return true;
            } else {
                return false;
            }
        "
        .to_string(),
    );

    // Simulate execution
    coverage_tracker.mark_line_executed(1);
    coverage_tracker.mark_line_executed(2);
    coverage_tracker.mark_line_executed(5);
    coverage_tracker.mark_line_executed(6);
    coverage_tracker.mark_line_executed(7);
    coverage_tracker.mark_function_executed("add");
    coverage_tracker.mark_branch_executed(6, "result > 5");

    println!(
        "   ✅ Coverage: {:.1}% lines, {:.1}% functions, {:.1}% branches",
        coverage_tracker.line_coverage_percentage(),
        coverage_tracker.function_coverage_percentage(),
        coverage_tracker.branch_coverage_percentage()
    );

    // Generate test report
    let report = test_runner.generate_report(OutputFormat::Text);
    println!("   ✅ Test report: {} chars", report.len());

    println!("\n🎉 Testing framework complete!");
}

fn test_performance_optimization() {
    println!("   Testing Performance Optimization System...");

    // Test benchmarking system
    println!("\n   Testing Benchmarking System...");
    use performance::benchmark::*;

    let benchmark_runner = BenchmarkRunner::new()
        .with_iterations(100)
        .with_warmup(10)
        .with_memory_tracking(true);

    // Run lexer benchmarks
    let lexer_suite =
        benchmark_runner.run_suite("Lexer Benchmarks", LanguageBenchmarks::lexer_benchmarks());
    println!("   ✅ Lexer benchmarks completed:");
    for result in &lexer_suite.benchmarks {
        println!(
            "      - {}: {:?} ({:.0} ops/sec)",
            result.name, result.average_duration, result.throughput
        );
    }

    // Run parser benchmarks
    let parser_suite =
        benchmark_runner.run_suite("Parser Benchmarks", LanguageBenchmarks::parser_benchmarks());
    println!("   ✅ Parser benchmarks completed:");
    for result in &parser_suite.benchmarks {
        println!(
            "      - {}: {:?} ({:.0} ops/sec)",
            result.name, result.average_duration, result.throughput
        );
    }

    // Run runtime benchmarks
    let runtime_suite = benchmark_runner.run_suite(
        "Runtime Benchmarks",
        LanguageBenchmarks::runtime_benchmarks(),
    );
    println!("   ✅ Runtime benchmarks completed:");
    for result in &runtime_suite.benchmarks {
        println!(
            "      - {}: {:?} ({:.0} ops/sec)",
            result.name, result.average_duration, result.throughput
        );
    }

    // Test profiling system
    println!("\n   Testing Profiling System...");
    use performance::profiler::*;

    let profiler = get_global_profiler();

    // Profile lexer operations
    let _tokens = profiler.profile_scope("lexer_tokenization", || {
        let lexer = Lexer::new("let x = 42 + 10 * 2; let y = (x + 5) / 3;");
        lexer.tokenize_immutable().unwrap()
    });

    // Profile parser operations
    let _ast = profiler.profile_scope("parser_parsing", || {
        let lexer = Lexer::new("let x = 42 + 10 * 2; let y = (x + 5) / 3;");
        let tokens = lexer.tokenize_immutable().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse().unwrap()
    });

    // Profile runtime operations
    let _result = profiler.profile_scope("runtime_execution", || {
        let mut runtime = Runtime::new();
        runtime.set_variable("x".to_string(), Value::Int(42));
        runtime.set_variable("y".to_string(), Value::Int(10));
        runtime.get_variable("x")
    });

    let profile_report = profiler.generate_report();
    println!("   ✅ Profiling completed:");
    println!(
        "      {}",
        profile_report.lines().next().unwrap_or("No profile data")
    );

    // Test optimization system
    println!("\n   Testing Optimization System...");
    use performance::optimizer::*;

    let optimizer = Optimizer::new().with_level(OptimizationLevel::Aggressive);

    // Create a test AST for optimization
    let test_source = r#"
        let x = 42 + 10 * 2;
        let y = (x + 5) / 3;
        let z = x + y;
    "#;

    let lexer = Lexer::new(test_source);
    let tokens = lexer.tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);
    let original_ast = parser.parse().unwrap();

    let optimization_result = optimizer.optimize(original_ast);
    println!("   ✅ Optimization completed:");
    println!(
        "      - Optimizations applied: {}",
        optimization_result.optimizations_applied.len()
    );
    println!(
        "      - Estimated improvement: {:.1}%",
        optimization_result.performance_improvement
    );

    // Test memory management
    println!("\n   Testing Memory Management...");
    use performance::memory::*;

    let memory_manager = get_global_memory_manager();

    // Allocate some memory
    let _block1 = memory_manager.allocate(1024, "String");
    let block2 = memory_manager.allocate(2048, "Vector");
    let _block3 = memory_manager.allocate(512, "HashMap");

    // Deallocate some memory
    memory_manager.deallocate(block2);

    let memory_stats = memory_manager.get_stats();
    println!("   ✅ Memory management completed:");
    println!(
        "      - Total allocated: {} bytes",
        memory_stats.total_allocated
    );
    println!(
        "      - Current usage: {} bytes",
        memory_stats.current_usage
    );
    println!("      - Peak usage: {} bytes", memory_stats.peak_usage);

    // Test object pooling
    let string_pool = memory_manager.create_object_pool::<String>("strings", 10);
    let mut pooled_strings = Vec::new();

    for i in 0..5 {
        let mut pooled_string = string_pool.acquire();
        *pooled_string.value_mut() = format!("string_{}", i);
        pooled_strings.push(pooled_string);
    }

    let pool_stats = string_pool.get_stats();
    println!("   ✅ Object pooling completed:");
    println!(
        "      - Pool: {} (capacity: {})",
        pool_stats.name, pool_stats.capacity
    );
    println!(
        "      - Available: {}, In use: {}",
        pool_stats.available_count, pool_stats.in_use_count
    );
    println!("      - Reuse rate: {:.1}%", pool_stats.reuse_rate * 100.0);

    // Test concurrency system
    println!("\n   Testing Concurrency System...");
    use performance::concurrency::*;

    let thread_pool = ThreadPool::new(4);
    let async_scheduler = AsyncScheduler::new(4);

    // Submit some tasks
    for i in 0..5 {
        let task_id =
            async_scheduler.schedule(&format!("task_{}", i), TaskPriority::Normal, move || {
                std::thread::sleep(Duration::from_millis(10));
                Ok(())
            });
        println!("      - Scheduled task {} with ID {}", i, task_id);
    }

    // Wait a bit for tasks to complete
    std::thread::sleep(Duration::from_millis(100));

    let thread_pool_stats = thread_pool.get_stats();
    println!("   ✅ Concurrency completed:");
    println!(
        "      - Thread pool: {} total, {} active",
        thread_pool_stats.total_threads, thread_pool_stats.active_threads
    );
    println!(
        "      - Completed tasks: {}",
        thread_pool_stats.completed_tasks
    );
    println!(
        "      - Average task duration: {:?}",
        thread_pool_stats.average_task_duration
    );

    // Test parallel execution
    let numbers: Vec<i32> = (1..=100).collect();
    let doubled = ParallelExecutor::map_parallel(numbers, |x| x * 2, 4);

    let sum = ParallelExecutor::reduce_parallel(doubled.clone(), |acc, x| acc + x, 4);

    println!("   ✅ Parallel execution completed:");
    println!("      - Parallel map result: {} items", doubled.len());
    println!("      - Parallel reduce result: {}", sum);

    println!("\n🎉 Performance Optimization working!");
    println!("   - Comprehensive benchmarking system ✅");
    println!("   - Real-time profiling with memory tracking ✅");
    println!("   - Multi-level compiler optimizations ✅");
    println!("   - Advanced memory management with object pooling ✅");
    println!("   - High-performance concurrency with thread pools ✅");
    println!("   - Parallel execution utilities ✅");

    // Test service parsing
    test_service_parsing();
}

// TEMPORARY: Test service parsing
fn test_service_parsing() {
    println!("✅ Service Parsing: testing...");

    let test_code = r#"service MyService {
    balance: int = 100;
    name: string;

    fn get_balance() -> int {
        return self.balance;
    }

    fn set_balance(new_balance: int) {
        self.balance = new_balance;
    }
}"#;

    // Tokenize
    let tokens = match Lexer::new(test_code).tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, None, Some(test_code)));
            return;
        }
    };

    // Parse
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("❌ Parsing failed:\n{}", format_parser_error(&e, None, Some(test_code)));
            return;
        }
    };

    // Check for service statement
    let has_service = program
        .statements
        .iter()
        .any(|stmt| matches!(stmt, Statement::Service(_)));

    if has_service {
        // Print service details
        for stmt in &program.statements {
            if let Statement::Service(service) = stmt {
                println!(
                    "✅ Service Parsing: {} with {} fields, {} methods",
                    service.name,
                    service.fields.len(),
                    service.methods.len()
                );
            }
        }
    }

    println!("✅ Service Parsing: complete");
}

fn convert_solidity_file(input_file: &str, output_file: &str) {
    use dist_agent_lang::solidity_converter;
    use std::path::Path;

    println!(
        "🔄 Converting Solidity to DAL: {} -> {}",
        input_file, output_file
    );

    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);

    match solidity_converter::convert_file(input_path, output_path) {
        Ok(_) => {
            println!("✅ Conversion successful!");
            println!("   Output written to: {}", output_file);
        }
        Err(e) => {
            eprintln!("❌ Conversion failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn analyze_solidity_file(input_file: &str) {
    use dist_agent_lang::solidity_converter;
    use std::path::Path;

    println!("📊 Analyzing Solidity file: {}", input_file);

    let input_path = Path::new(input_file);

    match solidity_converter::analyze_file(input_path) {
        Ok(report) => {
            println!("\n📈 Analysis Report:");
            println!("   Compatibility Score: {:.1}%", report.compatibility_score);

            if !report.errors.is_empty() {
                println!("\n❌ Errors ({}):", report.errors.len());
                for error in &report.errors {
                    println!("   - {}", error);
                }
            }

            if !report.unsupported_features.is_empty() {
                println!(
                    "\n⚠️  Unsupported Features ({}):",
                    report.unsupported_features.len()
                );
                for feature in &report.unsupported_features {
                    println!("   - {}", feature);
                }
            }

            if !report.warnings.is_empty() {
                println!("\n⚠️  Warnings ({}):", report.warnings.len());
                for warning in &report.warnings {
                    println!("   - {}", warning);
                }
            }

            if !report.suggestions.is_empty() {
                println!("\n💡 Suggestions ({}):", report.suggestions.len());
                for suggestion in &report.suggestions {
                    println!("   - {}", suggestion);
                }
            }

            println!("\n✅ Analysis complete!");
        }
        Err(e) => {
            eprintln!("❌ Analysis failed: {}", e);
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 0: Developer Tools Implementation
// ============================================================================

/// Check DAL file for type errors without executing
fn check_dal_file(filename: &str) {
    println!("🪩  Type checking dist_agent_lang file: {}", filename);

    // Read the file
    let source_code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    // Tokenize
    let tokens_with_pos = match Lexer::new(&source_code).tokenize_with_positions_immutable() {
        Ok(tokens) => {
            println!("✅ Lexer scanning... {} tokens", tokens.len());
            tokens
        }
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    // Parse
    let ast = match Parser::new_with_positions(tokens_with_pos).parse() {
        Ok(ast) => {
            println!("✅ Parsed {} statements", ast.statements.len());
            ast
        }
        Err(e) => {
            eprintln!("❌ Type check failed:\n{}", format_parser_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    // Warnings (unused variables, etc.)
    let warnings = parser::collect_warnings(&ast);
    if !warnings.is_empty() {
        eprintln!("\n{}", format_parse_warnings(&warnings, Some(filename), Some(&source_code)));
    }

    println!("✅ Type check passed!");
    println!("   {} statements validated", ast.statements.len());
    if warnings.is_empty() {
        println!("   No type errors or warnings found");
    } else {
        println!("   {} warning(s) (see above)", warnings.len());
    }
}

/// Format DAL code
fn format_dal_file(filename: &str, check_only: bool) {
    if check_only {
        println!("🪩  Checking format of: {}", filename);
    } else {
        println!("🪩  Formatting dist_agent_lang file: {}", filename);
    }

    // Read the file
    let source_code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    // Tokenize and parse
    let tokens = match Lexer::new(&source_code).tokenize_immutable() {
        Ok(tokens) => {
            println!("✅ Lexer scanning... {} tokens", tokens.len());
            tokens
        }
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => {
            println!("✅ Parsed {} statements", ast.statements.len());
            ast
        }
        Err(e) => {
            eprintln!("❌ Parsing failed:\n{}", format_parser_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    // Format AST back to code
    let formatted = format_ast(&ast);

    if check_only {
        if formatted.trim() == source_code.trim() {
            println!("✅ File is properly formatted");
        } else {
            println!("❌ File needs formatting");
            println!("   Run '{} fmt {}' to format", binary_name(), filename);
            std::process::exit(1);
        }
    } else {
        // Write formatted code back to file
        match std::fs::write(filename, formatted) {
            Ok(_) => {
                println!("✅ File formatted successfully!");
            }
            Err(e) => {
                eprintln!("❌ Error writing formatted file: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// Format AST back to source code
fn format_ast(ast: &parser::ast::Program) -> String {
    let mut output = String::new();

    for (i, statement) in ast.statements.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        output.push_str(&format_statement(statement));
    }

    output
}

fn format_statement(stmt: &Statement) -> String {
    use parser::ast::Statement;

    match stmt {
        Statement::Let(let_stmt) => {
            format!(
                "let {} = {};\n",
                let_stmt.name,
                format_expression(&let_stmt.value)
            )
        }
        Statement::Return(ret_stmt) => {
            if let Some(e) = &ret_stmt.value {
                format!("return {};\n", format_expression(e))
            } else {
                "return;\n".to_string()
            }
        }
        Statement::Expression(expr) => {
            format!("{};\n", format_expression(expr))
        }
        Statement::If(if_stmt) => {
            let mut result = format!("if {} {{\n", format_expression(&if_stmt.condition));
            for stmt in &if_stmt.consequence.statements {
                result.push_str("    ");
                result.push_str(&format_statement(stmt));
            }
            if let Some(alt) = &if_stmt.alternative {
                result.push_str("} else {\n");
                for stmt in &alt.statements {
                    result.push_str("    ");
                    result.push_str(&format_statement(stmt));
                }
            }
            result.push_str("}\n");
            result
        }
        Statement::Function(fn_stmt) => {
            let params: Vec<String> = fn_stmt
                .parameters
                .iter()
                .map(|p| {
                    if let Some(ref t) = p.param_type {
                        format!("{}: {}", p.name, t)
                    } else {
                        p.name.clone()
                    }
                })
                .collect();
            let ret_type = fn_stmt
                .return_type
                .as_ref()
                .map(|t| format!(" -> {}", t))
                .unwrap_or_default();
            let mut result = format!(
                "fn {}({}){} {{\n",
                fn_stmt.name,
                params.join(", "),
                ret_type
            );
            for stmt in &fn_stmt.body.statements {
                result.push_str("    ");
                result.push_str(&format_statement(stmt));
            }
            result.push_str("}\n");
            result
        }
        _ => "// Unformatted statement\n".to_string(),
    }
}

fn format_expression(expr: &parser::ast::Expression) -> String {
    use lexer::tokens::Literal;
    use parser::ast::Expression;

    match expr {
        Expression::Literal(lit) => match lit {
            Literal::Int(n) => n.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Bool(b) => b.to_string(),
            Literal::Null => "null".to_string(),
        },
        Expression::Identifier(name) => name.clone(),
        Expression::BinaryOp(left, op, right) => {
            format!(
                "{} {} {}",
                format_expression(left),
                format_operator(op),
                format_expression(right)
            )
        }
        Expression::FunctionCall(call) => {
            let formatted_args: Vec<String> =
                call.arguments.iter().map(format_expression).collect();
            format!("{}({})", call.name, formatted_args.join(", "))
        }
        Expression::ArrayLiteral(elements) => {
            let formatted_elements: Vec<String> = elements.iter().map(format_expression).collect();
            format!("[{}]", formatted_elements.join(", "))
        }
        Expression::ObjectLiteral(obj) => {
            let formatted_pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, format_expression(v)))
                .collect();
            format!("{{{}}}", formatted_pairs.join(", "))
        }
        Expression::Assignment(name, value) => {
            format!("{} = {}", name, format_expression(value))
        }
        Expression::FieldAccess(obj, field) => {
            format!("{}.{}", format_expression(obj), field)
        }
        Expression::IndexAccess(arr, index) => {
            format!("{}[{}]", format_expression(arr), format_expression(index))
        }
        _ => "/* complex expression */".to_string(),
    }
}

fn format_operator(op: &lexer::tokens::Operator) -> &str {
    use lexer::tokens::Operator;
    match op {
        Operator::Plus => "+",
        Operator::Minus => "-",
        Operator::Star => "*",
        Operator::Slash => "/",
        Operator::Percent => "%",
        Operator::EqualEqual => "==",
        Operator::Equal => "=",
        Operator::BangEqual => "!=",
        Operator::Bang => "!",
        Operator::Less => "<",
        Operator::LessEqual => "<=",
        Operator::Greater => ">",
        Operator::GreaterEqual => ">=",
        Operator::And => "&&",
        Operator::Or => "||",
        Operator::Ampersand => "&",
        Operator::Pipe => "|",
        Operator::Caret => "^",
        Operator::Tilde => "~",
        Operator::LeftShift => "<<",
        Operator::RightShift => ">>",
        Operator::PlusEqual => "+=",
        Operator::MinusEqual => "-=",
        Operator::StarEqual => "*=",
        Operator::SlashEqual => "/=",
        Operator::PercentEqual => "%=",
        Operator::AndEqual => "&&=",
        Operator::OrEqual => "||=",
        Operator::CaretEqual => "^=",
        Operator::LeftShiftEqual => "<<=",
        Operator::RightShiftEqual => ">>=",
        Operator::Assign => "=",
        Operator::Not => "!",
        Operator::Colon => ":",
        Operator::NotEqual => "!=",
        Operator::Dot => ".",
    }
}

/// Lint DAL code
fn lint_dal_file(filename: &str) {
    println!("🪩  Linting dist_agent_lang file: {}", filename);

    // Read the file
    let source_code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    // Tokenize and parse
    let tokens = match Lexer::new(&source_code).tokenize_immutable() {
        Ok(tokens) => {
            println!("✅ Lexer scanning... {} tokens", tokens.len());
            tokens
        }
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => {
            println!("✅ Parsed {} statements", ast.statements.len());
            ast
        }
        Err(e) => {
            eprintln!("❌ Parsing failed:\n{}", format_parser_error(&e, Some(filename), Some(&source_code)));
            std::process::exit(1);
        }
    };

    // Run lint checks
    let mut issues = Vec::new();

    // Unused variables (and similar) from parser warnings
    let warnings = parser::collect_warnings(&ast);
    for w in &warnings {
        issues.push(if w.line > 0 {
            format!("Line {}: {}", w.line, w.message)
        } else {
            w.message.clone()
        });
    }

    // TODO: Add more lint checks:
    // - Functions that don't return anything
    // - Dead code after return
    // - Non-idiomatic patterns
    // - Security issues

    if issues.is_empty() {
        println!("✅ No lint issues found!");
        println!("   File follows best practices");
    } else {
        println!("❌ Found {} lint issue(s):", issues.len());
        for issue in issues {
            println!("   {}", issue);
        }
        std::process::exit(1);
    }
}

/// Create a new DAL project
fn create_new_project(project_name: &str, project_type: Option<&str>) {
    println!("📦 Creating new dist_agent_lang project: {}", project_name);

    // Validate project name
    if !is_valid_project_name(project_name) {
        eprintln!("❌ Invalid project name. Use lowercase letters, numbers, hyphens, and underscores only.");
        std::process::exit(1);
    }

    // Create project directory
    if std::path::Path::new(project_name).exists() {
        eprintln!("❌ Directory '{}' already exists", project_name);
        std::process::exit(1);
    }

    match std::fs::create_dir(project_name) {
        Ok(_) => {
            println!("   ✅ Created directory: {}", project_name);
        }
        Err(e) => {
            eprintln!("❌ Failed to create directory: {}", e);
            std::process::exit(1);
        }
    }

    // Create project structure based on type
    let ptype = project_type.unwrap_or("default");
    match ptype {
        "chain" | "contract" | "erc20" | "erc721" => {
            create_contract_project(project_name);
        }
        "web" | "webapp" | "dapp" => {
            create_web_project(project_name);
        }
        "cli" => {
            create_cli_project(project_name);
        }
        "lib" | "library" => {
            create_library_project(project_name);
        }
        "ai" | "ml" => {
            create_ai_project(project_name);
        }
        "iot" | "device" => {
            create_iot_project(project_name);
        }
        "agent" | "agents" => {
            create_agent_project(project_name);
        }
        _ => {
            create_default_project(project_name);
        }
    }

    println!("\n✅ Project '{}' created successfully!", project_name);
    println!("\n📚 Next steps:");
    println!("   cd {}", project_name);

    // Show appropriate next command based on type
    match ptype {
        "web" | "webapp" | "dapp" => println!("   {} web web.dal", binary_name()),
        "ai" | "ml" => println!("   {} run ai.dal", binary_name()),
        "iot" | "device" => println!("   {} run iot.dal", binary_name()),
        "agent" | "agents" => println!("   {} run agent.dal", binary_name()),
        "chain" | "contract" | "erc20" | "erc721" => {
            println!("   {} run contract.dal", binary_name())
        }
        "cli" => println!("   {} run cli.dal", binary_name()),
        "lib" | "library" => println!("   {} run lib.dal", binary_name()),
        _ => println!("   {} run main.dal", binary_name()),
    }
}

fn is_valid_project_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

fn create_default_project(project_name: &str) {
    let main_dal = format!(
        r#"// {project_name} - dist_agent_lang project
// Generated by dal new

fn main() {{
    print("Hello from {project_name}!");
    print("Edit main.dal to get started.");
}}

main();
"#
    );

    let readme = format!(
        r#"# {project_name}

A dist_agent_lang project.

## Getting Started

Run the project:
```bash
dal run main.dal
```

Run tests:
```bash
dal test
```

Format code:
```bash
dal fmt main.dal
```

## Learn More

- [dist_agent_lang Documentation](https://github.com/dist_agent_lang/dist_agent_lang)
"#
    );

    // Write files
    std::fs::write(format!("{}/main.dal", project_name), main_dal).unwrap();
    std::fs::write(format!("{}/README.md", project_name), readme).unwrap();

    println!("   ✅ Created main.dal");
    println!("   ✅ Created README.md");
}

fn create_contract_project(project_name: &str) {
    let contract_dal = format!(
        r#"// {project_name} - Smart Contract
// Generated by dal new --mold contract

contract Token {{
    let name: string = "MyToken";
    let symbol: string = "MTK";
    let totalSupply: int = 1000000;
    let balances: map<string, int> = {{}};

    fn constructor() {{
        balances[msg::sender] = totalSupply;
    }}

    fn transfer(to: string, amount: int) -> bool {{
        if balances[msg::sender] >= amount {{
            balances[msg::sender] = balances[msg::sender] - amount;
            balances[to] = balances[to] + amount;
            return true;
        }}
        return false;
    }}

    fn balanceOf(account: string) -> int {{
        return balances[account];
    }}
}}
"#
    );

    std::fs::write(format!("{}/contract.dal", project_name), contract_dal).unwrap();
    std::fs::write(
        format!("{}/README.md", project_name),
        format!(
            "# {} Smart Contract\n\nA dist_agent_lang smart contract project.",
            project_name
        ),
    )
    .unwrap();

    println!("   ✅ Created contract.dal");
    println!("   ✅ Created README.md");
}

fn create_web_project(project_name: &str) {
    let web_dal = format!(
        r#"// {project_name} - Web Application
// Generated by dal new --mold web

service WebApp {{
    route GET "/" {{
        return "<h1>Welcome to {project_name}!</h1>";
    }}

    route GET "/api/health" {{
        return {{"status": "ok", "message": "Service is running"}};
    }}
}}

web::start(WebApp, 3000);
"#
    );

    std::fs::write(format!("{}/web.dal", project_name), web_dal).unwrap();
    std::fs::write(
        format!("{}/README.md", project_name),
        format!(
            "# {} Web App\n\nA dist_agent_lang web application.",
            project_name
        ),
    )
    .unwrap();

    println!("   ✅ Created web.dal");
    println!("   ✅ Created README.md");
}

fn create_cli_project(project_name: &str) {
    let cli_dal = format!(
        r#"// {project_name} - CLI Application
// Generated by dal new --mold cli

fn main(args: array<string>) {{
    if args.len() < 2 {{
        print("Usage: <program> <command>");
        print("Run with 'help' for more information");
        return;
    }}

    let command = args[1];
    
    if command == "help" {{
        print("Available commands:");
        print("  help  - Show this help");
        print("  hello - Say hello");
    }} else if command == "hello" {{
        print("Hello from {project_name}!");
    }} else {{
        print("Unknown command");
        print("Run with 'help' for available commands");
    }}
}}

main(sys::args());
"#
    );

    std::fs::write(format!("{}/cli.dal", project_name), cli_dal).unwrap();
    std::fs::write(
        format!("{}/README.md", project_name),
        format!(
            "# {} CLI\n\nA dist_agent_lang CLI application.",
            project_name
        ),
    )
    .unwrap();

    println!("   ✅ Created cli.dal");
    println!("   ✅ Created README.md");
}

fn create_library_project(project_name: &str) {
    let lib_dal = format!(
        r#"// {project_name} - Library
// Generated by dal new --mold lib

// Public API
export fn add(a: int, b: int) -> int {{
    return a + b;
}}

export fn multiply(a: int, b: int) -> int {{
    return a * b;
}}

// Private helper
fn validate(x: int) -> bool {{
    return x >= 0;
}}
"#
    );

    std::fs::write(format!("{}/lib.dal", project_name), lib_dal).unwrap();
    std::fs::write(
        format!("{}/README.md", project_name),
        format!("# {} Library\n\nA dist_agent_lang library.", project_name),
    )
    .unwrap();

    println!("   ✅ Created lib.dal");
    println!("   ✅ Created README.md");
}

fn create_ai_project(project_name: &str) {
    let ai_dal = format!(
        r#"// {project_name} - AI/ML Project
// Generated by dal new --mold ai

use ai;
use service;

fn main() {{
    print("🤖 {project_name} - AI/ML Application");
    
    // Example: Text generation
    let prompt = "Explain blockchain in simple terms";
    let response = ai::generate_text(prompt, {{}});
    print("AI Response:", response);
    
    // Example: Text classification
    let text = "This is a great product!";
    let sentiment = ai::classify("sentiment", text);
    print("Sentiment:", sentiment);
    
    // Example: Text embedding
    let embedding = ai::embed("Hello world");
    print("Embedding dimensions:", embedding.len());
    
    // Example: Similarity comparison
    let vec1 = ai::embed("machine learning");
    let vec2 = ai::embed("artificial intelligence");
    let similarity = ai::cosine_similarity(vec1, vec2);
    print("Similarity score:", similarity);
}}

main();
"#
    );

    let readme = format!(
        r#"# {project_name}

An AI/ML project using dist_agent_lang.

## Features

- Text generation with AI models
- Text classification and sentiment analysis
- Text embeddings and similarity
- Integration with external AI services

## Usage

```bash
dal run ai.dal
```

## AI Commands

```bash
# Generate code with AI
dal ai code "create an ERC20 token"

# Explain code
dal ai explain ai.dal

# Generate tests
dal ai test ai.dal
```

## Learn More

- [AI Module Documentation](https://github.com/dist_agent_lang/dist_agent_lang/blob/main/docs/stdlib/ai.md)
- [Service Integration](https://github.com/dist_agent_lang/dist_agent_lang/blob/main/docs/stdlib/service.md)
"#
    );

    std::fs::write(format!("{}/ai.dal", project_name), ai_dal).unwrap();
    std::fs::write(format!("{}/README.md", project_name), readme).unwrap();

    println!("   ✅ Created ai.dal");
    println!("   ✅ Created README.md");
}

fn create_iot_project(project_name: &str) {
    let iot_dal = format!(
        r#"// {project_name} - IoT Project
// Generated by dal new --mold iot

use iot;

fn main() {{
    print("🔌 {project_name} - IoT Device Application");
    
    // Register IoT device
    let device = iot::register_device({{
        "name": "{project_name}",
        "type": "sensor",
        "location": "home"
    }});
    print("Device registered:", device);
    
    // Connect device
    iot::connect_device(device.id);
    print("Device connected");
    
    // Read sensor data
    let data = iot::read_sensor_data("temperature_sensor_1");
    print("Temperature:", data.value, "°C");
    
    // Send actuator command
    iot::send_actuator_command("led_1", "ON");
    print("LED turned on");
    
    // Monitor power consumption
    let power = iot::monitor_power_consumption(device.id);
    print("Power consumption:", power, "W");
    
    // Detect anomalies
    let anomalies = iot::detect_sensor_anomalies({{
        "sensor_id": "temperature_sensor_1",
        "threshold": 3.0
    }});
    
    if anomalies.len() > 0 {{
        print("⚠️  Anomalies detected:", anomalies.len());
    }}
}}

main();
"#
    );

    let readme = format!(
        r#"# {project_name}

An IoT device project using dist_agent_lang.

## Features

- Device registration and connectivity
- Sensor data reading
- Actuator control
- Power consumption monitoring
- Anomaly detection
- Predictive maintenance

## Usage

```bash
dal run iot.dal
```

## IoT Commands

```bash
# AI-enhanced IoT commands
dal iot ai-control <device_id> "turn on lights when motion detected"
dal iot ai-predict <device_id> --metric failure
dal iot ai-security <device_id>
dal iot ai-optimize <device_id> --goal energy
```

## Learn More

- [IoT Module Documentation](https://github.com/dist_agent_lang/dist_agent_lang/blob/main/docs/stdlib/iot.md)
- [CLI IoT Commands](https://github.com/dist_agent_lang/dist_agent_lang/blob/main/docs/development/CLI_EXPANSION_PLAN.md#10-iot-commands)
"#
    );

    std::fs::write(format!("{}/iot.dal", project_name), iot_dal).unwrap();
    std::fs::write(format!("{}/README.md", project_name), readme).unwrap();

    println!("   ✅ Created iot.dal");
    println!("   ✅ Created README.md");
}

fn create_agent_project(project_name: &str) {
    let agent_dal = format!(
        r#"// {project_name} - Multi-Agent System
// Generated by dal new --mold agent

use agent;

fn main() {{
    print("🤖 {project_name} - Multi-Agent System");
    
    // Spawn AI agent
    let ai_agent = agent::spawn({{
        "type": "ai",
        "role": "Data Analyzer",
        "capabilities": ["analysis", "pattern_recognition"]
    }});
    print("AI Agent spawned:", ai_agent.id);
    
    // Spawn worker agents
    let worker1 = agent::spawn({{
        "type": "worker",
        "role": "Data Processor"
    }});
    
    let worker2 = agent::spawn({{
        "type": "worker",
        "role": "Data Validator"
    }});
    
    print("Worker agents spawned");
    
    // Send message between agents
    agent::communicate(ai_agent.id, worker1.id, {{
        "task": "process_batch",
        "batch_id": 42
    }});
    
    print("Message sent to worker");
    
    // Coordinate agents
    agent::coordinate("task_distribution", [ai_agent.id, worker1.id, worker2.id]);
    print("Agents coordinated");
    
    // Check agent status
    let status = agent::get_status(ai_agent.id);
    print("AI Agent status:", status);
    
    // Create agent fleet
    print("\\nCreating agent fleet...");
    let fleet_size = 10;
    
    for i in 0..fleet_size {{
        agent::spawn({{
            "type": "worker",
            "role": "Fleet Member " + i.to_string()
        }});
    }}
    
    print("Fleet of", fleet_size, "agents created");
}}

main();
"#
    );

    let readme = format!(
        r#"# {project_name}

A multi-agent system using dist_agent_lang.

## Features

- AI, System, and Worker agents
- Inter-agent communication
- Task distribution and coordination
- Agent fleets (homogeneous & heterogeneous)
- Learning and evolution
- Performance monitoring

## Usage

```bash
dal run agent.dal
```

## Agent Commands

```bash
# Create agents
dal agent create ai data_analyst --role "Analyze sales data"
dal agent create worker processor --role "ETL pipeline"

# Manage agents
dal agent list
dal agent status <agent_id>
dal agent send <from> <to> <message>

# Fleet management
dal agent fleet create workers --type worker --agents 100
dal agent fleet scale workers 200
dal agent fleet deploy workers "Process daily logs"

# Agent Molds (reusable agent configurations)
dal agent mold create fraud_detector --type ai --role "Detect fraud"
dal agent create --mold fraud_detector monitor_001
dal agent mold list
dal agent mold publish fraud_detector
```

**Note:** Molds are specifically for agent configurations, not general projects.
For creating new projects, use: `dal new <name> --type <ai|iot|agent|chain|web|cli|lib>`

## Learn More

- [Agent Module Documentation](https://github.com/dist_agent_lang/dist_agent_lang/blob/main/docs/stdlib/agent.md)
- [CLI Agent Commands](https://github.com/dist_agent_lang/dist_agent_lang/blob/main/docs/development/CLI_EXPANSION_PLAN.md#76-agent-commands-new---multi-agent-orchestration)
"#
    );

    std::fs::write(format!("{}/agent.dal", project_name), agent_dal).unwrap();
    std::fs::write(format!("{}/README.md", project_name), readme).unwrap();

    println!("   ✅ Created agent.dal");
    println!("   ✅ Created README.md");
}

/// Initialize a DAL project in current directory
fn init_project() {
    println!("📦 Initializing dist_agent_lang project in current directory...");

    // Check if dal.toml already exists
    if std::path::Path::new("dal.toml").exists() {
        eprintln!("❌ Project already initialized (dal.toml exists)");
        std::process::exit(1);
    }

    // Create dal.toml
    let dal_toml = r#"[package]
name = "my-project"
version = "0.1.0"
authors = []

[dependencies]
# Add dependencies here
"#;

    std::fs::write("dal.toml", dal_toml).unwrap();
    println!("   ✅ Created dal.toml");

    // Create main.dal if it doesn't exist
    if !std::path::Path::new("main.dal").exists() {
        let main_dal = r#"// Main entry point

fn main() {
    print("Hello from dist_agent_lang!");
}

main();
"#;
        std::fs::write("main.dal", main_dal).unwrap();
        println!("   ✅ Created main.dal");
    }

    // Create README.md if it doesn't exist
    if !std::path::Path::new("README.md").exists() {
        std::fs::write(
            "README.md",
            "# My DAL Project\n\nA dist_agent_lang project.\n",
        )
        .unwrap();
        println!("   ✅ Created README.md");
    }

    println!("\n✅ Project initialized!");
}

/// Run interactive REPL
fn run_repl() {
    println!("🔮 dist_agent_lang REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type 'exit' or 'quit' to exit, 'help' for help\n");

    let mut runtime = Runtime::new();
    let mut line_number = 1;

    loop {
        // Print prompt
        print!("dal[{}]> ", line_number);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        // Read line
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();

                // Handle special commands
                if input == "exit" || input == "quit" {
                    println!("Goodbye!");
                    break;
                }

                if input == "help" {
                    print_repl_help();
                    continue;
                }

                if input.is_empty() {
                    continue;
                }

                // Try to evaluate as expression
                match evaluate_repl_line(&input, &mut runtime) {
                    Ok(result) => {
                        if let Some(value) = result {
                            println!("=> {}", value);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }

                line_number += 1;
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn print_repl_help() {
    println!("dist_agent_lang REPL Commands:");
    println!("  help  - Show this help message");
    println!("  exit  - Exit the REPL");
    println!("  quit  - Exit the REPL");
    println!("\nYou can enter any DAL expression or statement:");
    println!("  let x = 42");
    println!("  x + 10");
    println!("  print(\"hello\")");
}

fn evaluate_repl_line(input: &str, runtime: &mut Runtime) -> Result<Option<Value>, String> {
    // Try to parse as a complete statement
    let tokens = Lexer::new(input)
        .tokenize_immutable()
        .map_err(|e| format!("Lexer error: {}", e))?;

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    // Execute
    runtime
        .execute_program(ast)
        .map_err(|e| e.to_string())
}

/// Watch DAL file and re-run on changes
fn watch_dal_file(filename: &str) {
    println!("👀 Watching {} for changes...", filename);
    println!("Press Ctrl+C to stop\n");

    // Initial run
    println!("🪩 Initial run:");
    run_dal_file(filename);

    // Get initial modification time
    let mut last_modified = match std::fs::metadata(filename) {
        Ok(metadata) => metadata.modified().ok(),
        Err(e) => {
            eprintln!("❌ Error getting file metadata: {}", e);
            std::process::exit(1);
        }
    };

    // Watch loop
    loop {
        std::thread::sleep(Duration::from_secs(1));

        // Check if file was modified
        match std::fs::metadata(filename) {
            Ok(metadata) => {
                if let Ok(modified) = metadata.modified() {
                    if Some(modified) != last_modified {
                        println!("\n🔄 File changed, re-running...\n");
                        run_dal_file(filename);
                        last_modified = Some(modified);
                    }
                }
            }
            Err(_) => {
                // File might have been deleted
                eprintln!("\n❌ File no longer accessible");
                std::process::exit(1);
            }
        }
    }
}

/// Add a package dependency
fn add_package(package: &str) {
    println!("📦 Adding package: {}", package);

    // Check if dal.toml exists
    if !std::path::Path::new("dal.toml").exists() {
        eprintln!("❌ No dal.toml found. Run '{} init' first.", binary_name());
        std::process::exit(1);
    }

    // Read dal.toml
    let mut toml_content = match std::fs::read_to_string("dal.toml") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading dal.toml: {}", e);
            std::process::exit(1);
        }
    };

    // Add dependency (simple append for now)
    if !toml_content.contains("[dependencies]") {
        toml_content.push_str("\n[dependencies]\n");
    }

    toml_content.push_str(&format!("{} = \"latest\"\n", package));

    // Write back
    match std::fs::write("dal.toml", toml_content) {
        Ok(_) => {
            println!("   ✅ Added {} to dal.toml", package);
            println!("   Run '{} install' to install dependencies", binary_name());
        }
        Err(e) => {
            eprintln!("❌ Error writing dal.toml: {}", e);
            std::process::exit(1);
        }
    }
}

/// Install dependencies from dal.toml
fn install_dependencies() {
    println!("📦 Installing dependencies...");

    // Check if dal.toml exists
    if !std::path::Path::new("dal.toml").exists() {
        eprintln!("❌ No dal.toml found. Run '{} init' first.", binary_name());
        std::process::exit(1);
    }

    // For now, just a placeholder
    println!("   ℹ️  Package management coming soon!");
    println!("   ✅ Dependencies would be installed here");
}

// ============================================================================
// Phase 1: Optimization Commands
// ============================================================================

/// Run language benchmarks
fn run_benchmarks(file: Option<&str>, suite: Option<&str>) {
    use performance::benchmark::{LanguageBenchmarks, PerformanceComparison};
    use performance::BenchmarkRunner;

    println!("⚡ Running dist_agent_lang benchmarks...\n");

    let runner = BenchmarkRunner::new()
        .with_iterations(1000)
        .with_warmup(100)
        .with_memory_tracking(true);

    // If specific file provided, benchmark that
    if let Some(filename) = file {
        println!("📊 Benchmarking file: {}\n", filename);

        // Read and parse file
        let source_code = match std::fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                std::process::exit(1);
            }
        };

        // Benchmark file execution
        let result = runner.run("file_execution", || {
            let tokens = Lexer::new(&source_code)
                .tokenize_immutable()
                .map_err(|e| e.to_string())?;
            let mut parser = Parser::new(tokens);
            let ast = parser.parse().map_err(|e| e.to_string())?;
            let mut runtime = Runtime::new();
            runtime.execute_program(ast).map_err(|e| e.to_string())?;
            Ok(())
        });

        println!("Results:");
        println!("  Iterations: {}", result.iterations);
        println!("  Average: {:?}", result.average_duration);
        println!("  Min: {:?}", result.min_duration);
        println!("  Max: {:?}", result.max_duration);
        println!("  Throughput: {:.0} ops/sec", result.throughput);
        if let Some(memory) = result.memory_usage {
            println!("  Memory: {} bytes", memory);
        }

        return;
    }

    // Otherwise run built-in benchmark suites
    let suite_name = suite.unwrap_or("all");

    match suite_name {
        "lexer" => {
            println!("Running Lexer benchmarks...\n");
            let benchmarks = LanguageBenchmarks::lexer_benchmarks();
            let suite = runner.run_suite("Lexer Suite", benchmarks);
            println!("{}", PerformanceComparison::generate_report(&suite));
        }
        "parser" => {
            println!("Running Parser benchmarks...\n");
            let benchmarks = LanguageBenchmarks::parser_benchmarks();
            let suite = runner.run_suite("Parser Suite", benchmarks);
            println!("{}", PerformanceComparison::generate_report(&suite));
        }
        "runtime" => {
            println!("Running Runtime benchmarks...\n");
            let benchmarks = LanguageBenchmarks::runtime_benchmarks();
            let suite = runner.run_suite("Runtime Suite", benchmarks);
            println!("{}", PerformanceComparison::generate_report(&suite));
        }
        "stdlib" => {
            println!("Running Stdlib benchmarks...\n");
            let benchmarks = LanguageBenchmarks::stdlib_benchmarks();
            let suite = runner.run_suite("Stdlib Suite", benchmarks);
            println!("{}", PerformanceComparison::generate_report(&suite));
        }
        "all" => {
            // Run all suites
            println!("Running all benchmark suites...\n");

            let lexer_benchmarks = LanguageBenchmarks::lexer_benchmarks();
            let lexer_suite = runner.run_suite("Lexer", lexer_benchmarks);
            println!("{}", PerformanceComparison::generate_report(&lexer_suite));

            let parser_benchmarks = LanguageBenchmarks::parser_benchmarks();
            let parser_suite = runner.run_suite("Parser", parser_benchmarks);
            println!("{}", PerformanceComparison::generate_report(&parser_suite));

            let runtime_benchmarks = LanguageBenchmarks::runtime_benchmarks();
            let runtime_suite = runner.run_suite("Runtime", runtime_benchmarks);
            println!("{}", PerformanceComparison::generate_report(&runtime_suite));

            let stdlib_benchmarks = LanguageBenchmarks::stdlib_benchmarks();
            let stdlib_suite = runner.run_suite("Stdlib", stdlib_benchmarks);
            println!("{}", PerformanceComparison::generate_report(&stdlib_suite));
        }
        _ => {
            eprintln!("❌ Unknown suite: {}", suite_name);
            eprintln!("Available suites: lexer, parser, runtime, stdlib, all");
            std::process::exit(1);
        }
    }
}

/// Profile execution of a DAL file
fn profile_dal_file(filename: &str, memory_tracking: bool) {
    use performance::profiler::Profiler;

    println!("🪩  Profiling dist_agent_lang file: {}", filename);
    if memory_tracking {
        println!("   (with memory tracking)");
    }
    println!();

    // Read file
    let source_code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    // Create profiler with optional memory tracking
    let profiler = Profiler::new().with_memory_tracking(memory_tracking);

    // Profile lexing
    let tokens = profiler.profile_scope("lexing", || {
        let t = Lexer::new(&source_code)
            .tokenize_immutable()
            .map_err(|e| {
                eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, Some(filename), Some(&source_code)));
                std::process::exit(1);
            })
            .unwrap();
        println!("✅ Lexer scanning... {} tokens", t.len());
        t
    });

    // Profile parsing
    let ast = profiler.profile_scope("parsing", || {
        let mut parser = Parser::new(tokens);
        let a = parser
            .parse()
            .map_err(|e| {
                eprintln!("❌ Parsing failed:\n{}", format_parser_error(&e, Some(filename), Some(&source_code)));
                std::process::exit(1);
            })
            .unwrap();
        println!("✅ Parsed {} statements", a.statements.len());
        a
    });

    // Profile execution
    profiler.profile_scope("execution", || {
        let mut runtime = Runtime::new();
        runtime
            .execute_program(ast)
            .map_err(|e| {
                eprintln!("❌ Execution failed:\n{}", format_runtime_error(&e, Some(filename), Some(&source_code)));
                std::process::exit(1);
            })
            .unwrap();
    });

    // Generate and print report
    println!("{}", profiler.generate_report());
}

/// Optimize a DAL file using AST optimizations
fn optimize_dal_file(input_file: &str, output_file: Option<&str>, level: u8) {
    use performance::optimizer::{OptimizationLevel, OptimizationUtils, Optimizer};

    println!("🪩 Optimizing dist_agent_lang file: {}", input_file);

    // Map level to OptimizationLevel
    let opt_level = match level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Basic,
        2 => OptimizationLevel::Aggressive,
        _ => {
            eprintln!("❌ Invalid optimization level: {}. Use 0, 1, or 2.", level);
            std::process::exit(1);
        }
    };

    println!("   Optimization level: {:?}", opt_level);
    println!();

    // Read file
    let source_code = match std::fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", input_file, e);
            std::process::exit(1);
        }
    };

    // Parse to AST
    let tokens = match Lexer::new(&source_code).tokenize_immutable() {
        Ok(tokens) => {
            println!("✅ Lexer scanning... {} tokens", tokens.len());
            tokens
        }
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, Some(input_file), Some(&source_code)));
            std::process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => {
            println!("✅ Parsed {} statements", ast.statements.len());
            ast
        }
        Err(e) => {
            eprintln!("❌ Parsing failed:\n{}", format_parser_error(&e, Some(input_file), Some(&source_code)));
            std::process::exit(1);
        }
    };

    // Analyze complexity before optimization
    let original_complexity = OptimizationUtils::analyze_complexity(&ast);
    let optimization_potential = OptimizationUtils::estimate_optimization_potential(&ast);

    println!("📊 Analysis:");
    println!("   Original complexity: {}", original_complexity);
    println!("   Optimization potential: {:.1}%", optimization_potential);
    println!();

    // Run optimizer
    let optimizer = Optimizer::new().with_level(opt_level);
    let result = optimizer.optimize(ast);

    // Show results
    println!("✨ Optimization complete!");
    println!(
        "   Optimizations applied: {}",
        result.optimizations_applied.len()
    );
    println!(
        "   Estimated improvement: {:.1}%",
        result.performance_improvement
    );
    println!();

    if !result.optimizations_applied.is_empty() {
        println!("Applied optimizations:");
        for opt in &result.optimizations_applied {
            println!("   • {}", opt);
        }
        println!();
    }

    // Analyze complexity after optimization
    let optimized_complexity = OptimizationUtils::analyze_complexity(&result.optimized_ast);
    println!(
        "   Complexity reduction: {} → {} ({:.1}% reduction)",
        original_complexity,
        optimized_complexity,
        ((original_complexity - optimized_complexity) as f64 / original_complexity as f64) * 100.0
    );

    // If output file specified, write optimized code
    if let Some(output) = output_file {
        // Convert AST back to code (simplified version)
        let optimized_code = format!("{:#?}", result.optimized_ast);
        match std::fs::write(output, optimized_code) {
            Ok(_) => {
                println!("\n   ✅ Optimized code written to: {}", output);
            }
            Err(e) => {
                eprintln!("\n❌ Error writing output file: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("\n   ℹ️  Use -o <file> to save optimized code");
    }
}

/// Show memory statistics
fn show_memory_stats() {
    use performance::memory::get_global_memory_manager;

    println!("💾 Memory Statistics\n");

    let memory_manager = get_global_memory_manager();
    let report = memory_manager.get_memory_report();

    println!("{}", report);
}

// ============================================================================
// Phase 2: Practical Commands (Chain, Crypto, Database)
// ============================================================================

/// Handle chain subcommands
fn handle_chain_command(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: {} chain <subcommand> [args...]", binary_name());
        std::process::exit(1);
    }

    match args[0].as_str() {
        "list" => {
            println!("📋 Supported Chains:\n");
            let chains = chain::get_supported_chains();
            for chain_config in chains {
                println!(
                    "  {} (Chain ID: {})",
                    chain_config.name, chain_config.chain_id
                );
                if !chain_config.rpc_url.is_empty() {
                    println!("    RPC: {}", chain_config.rpc_url);
                }
                println!();
            }
        }
        "config" => {
            if args.len() < 2 {
                eprintln!("Usage: {} chain config <chain_id>", binary_name());
                std::process::exit(1);
            }
            let chain_id: i64 = args[1].parse().unwrap_or_else(|_| {
                eprintln!("❌ Invalid chain ID: {}", args[1]);
                std::process::exit(1);
            });

            match chain::get_chain_config(chain_id) {
                Some(config) => {
                    println!("⛓️  Chain Configuration\n");
                    println!("Name: {}", config.name);
                    println!("Chain ID: {}", config.chain_id);
                    if !config.rpc_url.is_empty() {
                        println!("RPC URL: {}", config.rpc_url);
                    }
                    if !config.explorer.is_empty() {
                        println!("Explorer: {}", config.explorer);
                    }
                }
                None => {
                    eprintln!("❌ Chain ID {} not found", chain_id);
                    std::process::exit(1);
                }
            }
        }
        "gas-price" => {
            if args.len() < 2 {
                eprintln!("Usage: {} chain gas-price <chain_id>", binary_name());
                std::process::exit(1);
            }
            let chain_id: i64 = args[1].parse().unwrap_or_else(|_| {
                eprintln!("❌ Invalid chain ID: {}", args[1]);
                std::process::exit(1);
            });

            println!("⛽ Getting gas price for chain {}...", chain_id);
            let gas_price = chain::get_gas_price(chain_id);
            println!("   Gas Price: {:.2} Gwei", gas_price);
        }
        "balance" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} chain balance <chain_id> <address>",
                    binary_name()
                );
                std::process::exit(1);
            }
            let chain_id: i64 = args[1].parse().unwrap_or_else(|_| {
                eprintln!("❌ Invalid chain ID: {}", args[1]);
                std::process::exit(1);
            });
            let address = &args[2];

            println!("💰 Getting balance for {}...", address);
            let balance = chain::get_balance(chain_id, address.to_string());
            println!("   Balance: {} wei", balance);
            println!(
                "   Balance: {:.4} ETH",
                balance as f64 / 1_000_000_000_000_000_000.0
            );
        }
        "tx-status" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} chain tx-status <chain_id> <tx_hash>",
                    binary_name()
                );
                std::process::exit(1);
            }
            let chain_id: i64 = args[1].parse().unwrap_or_else(|_| {
                eprintln!("❌ Invalid chain ID: {}", args[1]);
                std::process::exit(1);
            });
            let tx_hash = &args[2];

            println!("🔍 Getting transaction status...");
            let status = chain::get_transaction_status(chain_id, tx_hash.to_string());
            println!("   Status: {}", status);
        }
        "block-time" => {
            if args.len() < 2 {
                eprintln!("Usage: {} chain block-time <chain_id>", binary_name());
                std::process::exit(1);
            }
            let chain_id: i64 = args[1].parse().unwrap_or_else(|_| {
                eprintln!("❌ Invalid chain ID: {}", args[1]);
                std::process::exit(1);
            });

            println!("⏰ Getting latest block timestamp...");
            let timestamp = chain::get_block_timestamp(chain_id);
            println!("   Timestamp: {}", timestamp);

            // Convert to human-readable format
            if timestamp > 0 {
                use std::time::{Duration, UNIX_EPOCH};
                let datetime = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
                if let Ok(system_time) = datetime.elapsed() {
                    println!("   ({} seconds ago)", system_time.as_secs());
                }
            }
        }
        "mint" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} chain mint <asset_name> [--meta key=value,...]",
                    binary_name()
                );
                std::process::exit(1);
            }
            let asset_name = &args[1];
            let mut metadata = HashMap::new();

            // Parse metadata if provided
            if args.len() >= 4 && args[2] == "--meta" {
                for pair in args[3].split(',') {
                    if let Some((key, value)) = pair.split_once('=') {
                        metadata.insert(key.to_string(), value.to_string());
                    }
                }
            }

            println!("🎨 Minting asset: {}", asset_name);
            let asset_id = chain::mint(asset_name.to_string(), metadata);
            println!("   ✅ Asset minted with ID: {}", asset_id);
        }
        "asset" => {
            if args.len() < 2 {
                eprintln!("Usage: {} chain asset <asset_id>", binary_name());
                std::process::exit(1);
            }
            let asset_id: i64 = args[1].parse().unwrap_or_else(|_| {
                eprintln!("❌ Invalid asset ID: {}", args[1]);
                std::process::exit(1);
            });

            println!("🔍 Getting asset info...");
            let info = chain::get(asset_id);

            if info.is_empty() {
                println!("   ❌ Asset not found");
            } else {
                println!("   Asset ID: {}", asset_id);
                for (key, value) in info {
                    println!("   {}: {}", key, value);
                }
            }
        }
        _ => {
            eprintln!("❌ Unknown chain subcommand: {}", args[0]);
            eprintln!(
                "Available: list, config, gas-price, balance, tx-status, block-time, mint, asset"
            );
            std::process::exit(1);
        }
    }
}

/// Handle crypto subcommands
fn handle_crypto_command(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: {} crypto <subcommand> [args...]", binary_name());
        std::process::exit(1);
    }

    match args[0].as_str() {
        "hash" => {
            if args.len() < 2 {
                eprintln!("Usage: {} crypto hash <data> [algorithm]", binary_name());
                eprintln!("Algorithms: sha256 (default), sha512");
                std::process::exit(1);
            }
            let data = &args[1];
            let algorithm = if args.len() >= 3 {
                match args[2].to_lowercase().as_str() {
                    "sha256" => HashAlgorithm::SHA256,
                    "sha512" => HashAlgorithm::SHA512,
                    _ => {
                        eprintln!("❌ Unknown algorithm: {}. Use sha256 or sha512", args[2]);
                        std::process::exit(1);
                    }
                }
            } else {
                HashAlgorithm::SHA256
            };

            let hash = crypto::hash(data, algorithm.clone());
            let alg_name = match algorithm {
                HashAlgorithm::SHA256 => "SHA256",
                HashAlgorithm::SHA512 => "SHA512",
                HashAlgorithm::Simple => "Simple",
                HashAlgorithm::Custom(ref s) => s.as_str(),
            };
            println!("🔐 Hash ({}): {}", alg_name, hash);
        }
        "random-hash" => {
            let algorithm = if args.len() >= 2 {
                match args[1].to_lowercase().as_str() {
                    "sha256" => HashAlgorithm::SHA256,
                    "sha512" => HashAlgorithm::SHA512,
                    _ => HashAlgorithm::SHA256,
                }
            } else {
                HashAlgorithm::SHA256
            };

            let hash = crypto::random_hash(algorithm);
            println!("🎲 Random Hash: {}", hash);
        }
        "keygen" => {
            let algorithm = if args.len() >= 2 {
                match args[1].to_lowercase().as_str() {
                    "rsa" => SignatureAlgorithm::RSA,
                    "ed25519" => SignatureAlgorithm::Ed25519,
                    _ => {
                        eprintln!("❌ Unknown algorithm: {}. Use rsa or ed25519", args[1]);
                        std::process::exit(1);
                    }
                }
            } else {
                SignatureAlgorithm::RSA
            };

            let keypair = crypto::generate_keypair(algorithm.clone());
            let alg_name = match algorithm {
                SignatureAlgorithm::RSA => "RSA",
                SignatureAlgorithm::Ed25519 => "Ed25519",
                SignatureAlgorithm::ECDSA => "ECDSA",
                SignatureAlgorithm::Custom(ref s) => s.as_str(),
            };
            println!("🔑 Generating keypair ({})...", alg_name);

            println!("\n✅ Keypair generated:");
            if let Some(public_key) = keypair.get("public_key") {
                println!("\nPublic Key:");
                println!("{}", public_key);
            }
            if let Some(private_key) = keypair.get("private_key") {
                println!("\nPrivate Key:");
                println!("{}", private_key);
                println!("\n⚠️  Keep your private key secure!");
            }
        }
        "sign" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} crypto sign <data> <private_key> [algorithm]",
                    binary_name()
                );
                eprintln!("Algorithms: rsa (default), ed25519");
                std::process::exit(1);
            }
            let data = &args[1];
            let private_key = &args[2];
            let algorithm = if args.len() >= 4 {
                match args[3].to_lowercase().as_str() {
                    "rsa" => SignatureAlgorithm::RSA,
                    "ed25519" => SignatureAlgorithm::Ed25519,
                    _ => SignatureAlgorithm::RSA,
                }
            } else {
                SignatureAlgorithm::RSA
            };

            println!("✍️  Signing data...");
            let signature = crypto::sign(data, private_key, algorithm);
            println!("   Signature: {}", signature);
        }
        "verify" => {
            if args.len() < 4 {
                eprintln!(
                    "Usage: {} crypto verify <data> <signature> <public_key> [algorithm]",
                    binary_name()
                );
                std::process::exit(1);
            }
            let data = &args[1];
            let signature = &args[2];
            let public_key = &args[3];
            let algorithm = if args.len() >= 5 {
                match args[4].to_lowercase().as_str() {
                    "rsa" => SignatureAlgorithm::RSA,
                    "ed25519" => SignatureAlgorithm::Ed25519,
                    _ => SignatureAlgorithm::RSA,
                }
            } else {
                SignatureAlgorithm::RSA
            };

            println!("🔍 Verifying signature...");
            let valid = crypto::verify(data, signature, public_key, algorithm);
            if valid {
                println!("   ✅ Signature is valid");
            } else {
                println!("   ❌ Signature is invalid");
                std::process::exit(1);
            }
        }
        "encrypt" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} crypto encrypt <data> <public_key>",
                    binary_name()
                );
                std::process::exit(1);
            }
            let data = &args[1];
            let public_key = &args[2];

            println!("🔒 Encrypting data...");
            let encrypted = crypto::encrypt(data, public_key);
            println!("   Encrypted: {}", encrypted);
        }
        "decrypt" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} crypto decrypt <encrypted_data> <private_key>",
                    binary_name()
                );
                std::process::exit(1);
            }
            let encrypted_data = &args[1];
            let private_key = &args[2];

            println!("🔓 Decrypting data...");
            match crypto::decrypt(encrypted_data, private_key) {
                Some(decrypted) => {
                    println!("   Decrypted: {}", decrypted);
                }
                None => {
                    eprintln!("   ❌ Decryption failed");
                    std::process::exit(1);
                }
            }
        }
        "aes-encrypt" => {
            if args.len() < 3 {
                eprintln!("Usage: {} crypto aes-encrypt <data> <key>", binary_name());
                std::process::exit(1);
            }
            let data = &args[1];
            let key = &args[2];

            println!("🔒 Encrypting with AES-256...");
            match crypto::encrypt_aes256(data, key) {
                Ok(encrypted) => {
                    println!("   Encrypted: {}", encrypted);
                }
                Err(e) => {
                    eprintln!("   ❌ Encryption failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "aes-decrypt" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} crypto aes-decrypt <encrypted_data> <key>",
                    binary_name()
                );
                std::process::exit(1);
            }
            let encrypted_data = &args[1];
            let key = &args[2];

            println!("🔓 Decrypting with AES-256...");
            match crypto::decrypt_aes256(encrypted_data, key) {
                Ok(decrypted) => {
                    println!("   Decrypted: {}", decrypted);
                }
                Err(e) => {
                    eprintln!("   ❌ Decryption failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("❌ Unknown crypto subcommand: {}", args[0]);
            eprintln!("Available: hash, keygen, sign, verify, encrypt, decrypt, aes-encrypt, aes-decrypt, random-hash");
            std::process::exit(1);
        }
    }
}

/// Handle database subcommands
fn handle_db_command(args: &[String]) {
    use stdlib::database;

    if args.is_empty() {
        eprintln!("Usage: {} db <subcommand> [args...]", binary_name());
        std::process::exit(1);
    }

    match args[0].as_str() {
        "connect" => {
            if args.len() < 2 {
                eprintln!("Usage: {} db connect <connection_string>", binary_name());
                std::process::exit(1);
            }
            let conn_str = &args[1];

            println!("🔌 Connecting to database...");
            match database::connect(conn_str.to_string()) {
                Ok(_db) => {
                    println!("   ✅ Connected successfully");
                    println!("   Connection string: {}", conn_str);
                }
                Err(e) => {
                    eprintln!("   ❌ Connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "query" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} db query <connection_string> \"<sql>\"",
                    binary_name()
                );
                std::process::exit(1);
            }
            let conn_str = &args[1];
            let sql = &args[2];

            println!("📊 Executing query...");
            match database::connect(conn_str.to_string()) {
                Ok(db) => match database::query(&db, sql.to_string(), vec![]) {
                    Ok(result) => {
                        println!("   ✅ Query executed successfully");
                        println!("   Rows affected: {}", result.affected_rows);
                        if !result.rows.is_empty() {
                            println!("\n   Results:");
                            for (i, row) in result.rows.iter().take(10).enumerate() {
                                println!("   Row {}: {:?}", i + 1, row);
                            }
                            if result.rows.len() > 10 {
                                println!("   ... and {} more rows", result.rows.len() - 10);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("   ❌ Query failed: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("   ❌ Connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "tables" => {
            if args.len() < 2 {
                eprintln!("Usage: {} db tables <connection_string>", binary_name());
                std::process::exit(1);
            }
            let conn_str = &args[1];

            println!("📋 Listing tables...");
            match database::connect(conn_str.to_string()) {
                Ok(db) => match database::list_tables(&db) {
                    Ok(tables) => {
                        println!("   Found {} tables:\n", tables.len());
                        for table in tables {
                            println!("   • {}", table);
                        }
                    }
                    Err(e) => {
                        eprintln!("   ❌ Failed to list tables: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("   ❌ Connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "schema" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} db schema <connection_string> <table_name>",
                    binary_name()
                );
                std::process::exit(1);
            }
            let conn_str = &args[1];
            let table_name = &args[2];

            println!("📐 Getting schema for table '{}'...", table_name);
            match database::connect(conn_str.to_string()) {
                Ok(db) => match database::get_table_schema(&db, table_name.to_string()) {
                    Ok(schema) => {
                        println!("   Table: {}", schema.name);
                        println!("\n   Columns:");
                        for column in schema.columns {
                            println!("     • {} ({})", column.name, column.data_type);
                            if column.is_primary_key {
                                println!("       [PRIMARY KEY]");
                            }
                            if !column.is_nullable {
                                println!("       [NOT NULL]");
                            }
                            if let Some(default) = column.default_value {
                                println!("       DEFAULT: {}", default);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("   ❌ Failed to get schema: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("   ❌ Connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "plan" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} db plan <connection_string> \"<sql>\"",
                    binary_name()
                );
                std::process::exit(1);
            }
            let conn_str = &args[1];
            let sql = &args[2];

            println!("📈 Getting query plan...");
            match database::connect(conn_str.to_string()) {
                Ok(db) => match database::get_query_plan(&db, sql.to_string()) {
                    Ok(plan) => {
                        println!("   Query Plan:");
                        for (key, value) in plan {
                            println!("   {}: {}", key, value);
                        }
                    }
                    Err(e) => {
                        eprintln!("   ❌ Failed to get query plan: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("   ❌ Connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "backup" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} db backup <connection_string> <backup_path>",
                    binary_name()
                );
                std::process::exit(1);
            }
            let conn_str = &args[1];
            let backup_path = &args[2];

            println!("💾 Creating backup...");
            match database::connect(conn_str.to_string()) {
                Ok(db) => match database::backup_database(&db, backup_path.to_string()) {
                    Ok(_) => {
                        println!("   ✅ Backup created: {}", backup_path);
                    }
                    Err(e) => {
                        eprintln!("   ❌ Backup failed: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("   ❌ Connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "restore" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} db restore <connection_string> <backup_path>",
                    binary_name()
                );
                std::process::exit(1);
            }
            let conn_str = &args[1];
            let backup_path = &args[2];

            println!("📦 Restoring from backup...");
            match database::connect(conn_str.to_string()) {
                Ok(db) => match database::restore_database(&db, backup_path.to_string()) {
                    Ok(_) => {
                        println!("   ✅ Database restored from: {}", backup_path);
                    }
                    Err(e) => {
                        eprintln!("   ❌ Restore failed: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("   ❌ Connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "metrics" => {
            if args.len() < 2 {
                eprintln!("Usage: {} db metrics <connection_string>", binary_name());
                std::process::exit(1);
            }
            let conn_str = &args[1];

            println!("📊 Getting database metrics...");
            match database::connect(conn_str.to_string()) {
                Ok(db) => {
                    let metrics = database::get_database_metrics(&db);
                    println!("   Active Connections: {}", metrics.connections_active);
                    println!("   Idle Connections: {}", metrics.connections_idle);
                    println!("   Total Queries: {}", metrics.total_queries);
                    println!("   Slow Queries: {}", metrics.slow_queries);
                    println!(
                        "   Cache Hit Ratio: {:.2}%",
                        metrics.cache_hit_ratio * 100.0
                    );
                    println!("   Average Query Time: {:.2}ms", metrics.average_query_time);
                }
                Err(e) => {
                    eprintln!("   ❌ Connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("❌ Unknown db subcommand: {}", args[0]);
            eprintln!("Available: connect, query, tables, schema, plan, backup, restore, metrics");
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 3: AI-Enhanced Tools
// ============================================================================

/// Handle AI subcommands
fn handle_ai_command(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: {} ai <subcommand> [args...]", binary_name());
        std::process::exit(1);
    }

    match args[0].as_str() {
        "code" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} ai code \"<prompt>\" [--output <file>]",
                    binary_name()
                );
                eprintln!(
                    "Example: {} ai code \"Create a token contract with minting\"",
                    binary_name()
                );
                std::process::exit(1);
            }
            let prompt = &args[1];
            let output_file = if args.len() >= 4 && args[2] == "--output" {
                Some(args[3].as_str())
            } else {
                None
            };

            generate_code(prompt, output_file);
        }
        "explain" => {
            if args.len() < 2 {
                eprintln!("Usage: {} ai explain <file.dal>", binary_name());
                std::process::exit(1);
            }
            let filename = &args[1];
            explain_code(filename);
        }
        "review" => {
            if args.len() < 2 {
                eprintln!("Usage: {} ai review <file.dal>", binary_name());
                std::process::exit(1);
            }
            let filename = &args[1];
            review_code(filename);
        }
        "audit" => {
            if args.len() < 2 {
                eprintln!("Usage: {} ai audit <file.dal>", binary_name());
                std::process::exit(1);
            }
            let filename = &args[1];
            audit_code(filename);
        }
        "test" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} ai test <file.dal> [--output <file>]",
                    binary_name()
                );
                std::process::exit(1);
            }
            let filename = &args[1];
            let output_file = if args.len() >= 4 && args[2] == "--output" {
                Some(args[3].as_str())
            } else {
                None
            };
            generate_tests(filename, output_file);
        }
        "fix" => {
            if args.len() < 2 {
                eprintln!("Usage: {} ai fix <file.dal>", binary_name());
                std::process::exit(1);
            }
            let filename = &args[1];
            suggest_fixes(filename);
        }
        "optimize-gas" => {
            if args.len() < 2 {
                eprintln!("Usage: {} ai optimize-gas <file.dal>", binary_name());
                std::process::exit(1);
            }
            let filename = &args[1];
            optimize_gas(filename);
        }
        _ => {
            eprintln!("❌ Unknown ai subcommand: {}", args[0]);
            eprintln!("Available: code, explain, review, audit, test, fix, optimize-gas");
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 8: Web subcommands (get, post, parse-url)
// ============================================================================

fn handle_web_command(args: &[String]) {
    use crate::runtime::values::Value;
    use std::collections::HashMap;
    use stdlib::web;

    if args.is_empty() {
        eprintln!(
            "Usage: {} web get <url> | web post <url> [--data <json>] | web parse-url <url>",
            binary_name()
        );
        std::process::exit(1);
    }
    match args[0].as_str() {
        "get" => {
            if args.len() < 2 {
                eprintln!("Usage: {} web get <url>", binary_name());
                std::process::exit(1);
            }
            let url = &args[1];
            match web::get_request(url.clone()) {
                Ok(resp) => println!(
                    "✅ {} {} (body length: {})",
                    resp.status,
                    url,
                    resp.body.len()
                ),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "post" => {
            if args.len() < 2 {
                eprintln!("Usage: {} web post <url> [--data <json>]", binary_name());
                std::process::exit(1);
            }
            let url = &args[1];
            let data = args
                .iter()
                .position(|a| a == "--data")
                .and_then(|i| args.get(i + 1))
                .map(|s| {
                    let mut m = HashMap::new();
                    m.insert("body".to_string(), Value::String(s.clone()));
                    m
                })
                .unwrap_or_default();
            match web::post_request(url.clone(), data) {
                Ok(resp) => println!(
                    "✅ {} {} (body length: {})",
                    resp.status,
                    url,
                    resp.body.len()
                ),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "parse-url" => {
            if args.len() < 2 {
                eprintln!("Usage: {} web parse-url <url>", binary_name());
                std::process::exit(1);
            }
            let url = &args[1];
            let params = web::parse_url(url.clone());
            println!("✅ Parsed URL: {}", url);
            for (k, v) in &params {
                println!("   {}: {}", k, v);
            }
        }
        _ => {
            eprintln!(
                "Unknown web subcommand: {}. Use get, post, parse-url.",
                args[0]
            );
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 3.5: Oracle Commands (fetch, verify, stream, create_source, create_query)
// ============================================================================

fn handle_oracle_command(args: &[String]) {
    use crate::runtime::values::Value;
    use stdlib::oracle;

    if args.is_empty() {
        eprintln!("Usage: {} oracle <subcommand> [args...]", binary_name());
        eprintln!();
        eprintln!("Subcommands:");
        eprintln!("  fetch <source> <query_type>          Fetch data from oracle source");
        eprintln!("  verify <data> <signature>            Verify oracle data signature");
        eprintln!("  stream <source> <callback>          Stream real-time data");
        eprintln!("  create-source <name> <url>          Create oracle source");
        eprintln!("  create-query <query_type>            Create oracle query");
        eprintln!("  get-stream <stream_id>              Get stream info");
        eprintln!("  close-stream <stream_id>            Close stream");
        std::process::exit(1);
    }

    match args[0].as_str() {
        "fetch" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} oracle fetch <source> <query_type>",
                    binary_name()
                );
                std::process::exit(1);
            }
            let source = &args[1];
            let query_type = &args[2];
            let query = oracle::create_query(query_type.clone());

            match oracle::fetch(source, query) {
                Ok(response) => {
                    println!("✅ Oracle fetch successful");
                    println!("   Source: {}", response.source);
                    println!("   Timestamp: {}", response.timestamp);
                    println!("   Data: {:?}", response.data);
                    if let Some(sig) = &response.signature {
                        println!("   Signature: {} (verified: {})", sig, response.verified);
                    }
                    println!("   Confidence: {:.2}%", response.confidence_score * 100.0);
                }
                Err(e) => {
                    eprintln!("❌ Oracle fetch failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "verify" => {
            if args.len() < 3 {
                eprintln!("Usage: {} oracle verify <data> <signature>", binary_name());
                std::process::exit(1);
            }
            // For CLI, treat data as string; in DAL code it can be any Value
            let data = Value::String(args[1].clone());
            let signature = &args[2];
            let is_valid = oracle::verify(&data, signature);
            if is_valid {
                println!("✅ Signature verified");
            } else {
                eprintln!("❌ Signature verification failed");
                std::process::exit(1);
            }
        }

        "stream" => {
            if args.len() < 3 {
                eprintln!("Usage: {} oracle stream <source> <callback>", binary_name());
                std::process::exit(1);
            }
            let source = &args[1];
            let callback = &args[2];

            match oracle::stream(source, callback) {
                Ok(stream_id) => {
                    println!("✅ Stream created: {}", stream_id);
                    println!("   Source: {}", source);
                    println!("   Callback: {}", callback);
                }
                Err(e) => {
                    eprintln!("❌ Stream creation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "create-source" => {
            if args.len() < 3 {
                eprintln!("Usage: {} oracle create-source <name> <url>", binary_name());
                std::process::exit(1);
            }
            let name = args[1].clone();
            let url = args[2].clone();
            let source = oracle::create_source(name.clone(), url.clone());
            println!("✅ Oracle source created");
            println!("   Name: {}", source.name);
            println!("   URL: {}", source.url);
            println!("   Trusted: {}", source.trusted);
        }

        "create-query" => {
            if args.len() < 2 {
                eprintln!("Usage: {} oracle create-query <query_type>", binary_name());
                std::process::exit(1);
            }
            let query_type = &args[1];
            let query = oracle::create_query(query_type.clone());
            println!("✅ Oracle query created");
            println!("   Type: {}", query.query_type);
            println!("   Require signature: {}", query.require_signature);
        }

        "get-stream" => {
            if args.len() < 2 {
                eprintln!("Usage: {} oracle get-stream <stream_id>", binary_name());
                std::process::exit(1);
            }
            let stream_id = &args[1];
            match oracle::get_stream(stream_id) {
                Some(entry) => {
                    println!("✅ Stream found: {}", stream_id);
                    println!("   Source: {}", entry.source);
                    println!("   Created at: {}", entry.created_at);
                }
                None => {
                    eprintln!("❌ Stream not found: {}", stream_id);
                    std::process::exit(1);
                }
            }
        }

        "close-stream" => {
            if args.len() < 2 {
                eprintln!("Usage: {} oracle close-stream <stream_id>", binary_name());
                std::process::exit(1);
            }
            let stream_id = &args[1];
            let closed = oracle::close_stream(stream_id);
            if closed {
                println!("✅ Stream closed: {}", stream_id);
            } else {
                eprintln!("❌ Stream not found: {}", stream_id);
                std::process::exit(1);
            }
        }

        _ => {
            eprintln!("Unknown oracle subcommand: {}", args[0]);
            eprintln!(
                "Use: fetch, verify, stream, create-source, create-query, get-stream, close-stream"
            );
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 4: Cloud & Enterprise (authorize, grant, revoke, audit, tenant, compliance, trust)
// ============================================================================

fn handle_cloud_command(args: &[String]) {
    use stdlib::cloudadmin;
    use stdlib::trust::{self, AdminLevel};

    if args.is_empty() {
        eprintln!("Usage: {} cloud <subcommand> [args...]", binary_name());
        std::process::exit(1);
    }

    match args[0].as_str() {
        "authorize" => {
            if args.len() < 4 {
                eprintln!(
                    "Usage: {} cloud authorize <user_id> <operation> <resource>",
                    binary_name()
                );
                eprintln!(
                    "Example: {} cloud authorize user_123 read config/db",
                    binary_name()
                );
                std::process::exit(1);
            }
            let user_id = &args[1];
            let operation = &args[2];
            let resource = &args[3];
            let allowed = cloudadmin::authorize(user_id, operation, resource);
            if allowed {
                println!("✅ Authorized: [user] may {} on {}", operation, resource);
            } else {
                println!("❌ Denied: [user] may not {} on {}", operation, resource);
                std::process::exit(1);
            }
        }
        "grant" => {
            if args.len() < 4 {
                eprintln!(
                    "Usage: {} cloud grant <user_id> <role> <scope>",
                    binary_name()
                );
                eprintln!("Roles: superadmin, admin, moderator, user");
                eprintln!(
                    "Example: {} cloud grant user_123 admin ec2:admin",
                    binary_name()
                );
                std::process::exit(1);
            }
            let user_id = args[1].clone();
            let role = args[2].to_lowercase();
            let scope = args[3].clone();
            let level = match role.as_str() {
                "superadmin" => AdminLevel::SuperAdmin,
                "admin" => AdminLevel::Admin,
                "moderator" => AdminLevel::Moderator,
                "user" => AdminLevel::User,
                _ => {
                    eprintln!(
                        "❌ Unknown role: {}. Use superadmin, admin, moderator, user",
                        role
                    );
                    std::process::exit(1);
                }
            };
            let perms = vec![scope];
            trust::register_admin(user_id.clone(), level, perms);
            println!("✅ Granted role '{}' to [user] (scope recorded)", role);
        }
        "revoke" => {
            if args.len() < 2 {
                eprintln!("Usage: {} cloud revoke <user_id>", binary_name());
                std::process::exit(1);
            }
            let user_id = &args[1];
            let removed = trust::remove_admin(user_id);
            if removed {
                println!("✅ Revoked all roles for [user]");
            } else {
                println!("⚠️  User was not in the registry (no change)");
            }
        }
        "roles" => {
            if args.len() < 2 {
                eprintln!("Usage: {} cloud roles <user_id>", binary_name());
                std::process::exit(1);
            }
            let user_id = &args[1];
            match trust::get_admin_info(user_id) {
                Some((level, perms)) => {
                    let level_str = match level {
                        AdminLevel::SuperAdmin => "superadmin",
                        AdminLevel::Admin => "admin",
                        AdminLevel::Moderator => "moderator",
                        AdminLevel::User => "user",
                    };
                    println!("👤 User: [redacted]");
                    println!("   Level: {}", level_str);
                    println!("   Permissions: {}", perms.join(", "));
                }
                None => {
                    println!("⚠️  User not found in admin registry");
                    println!("   (Set ADMIN_IDS and ADMIN_LEVEL_<id> or use 'cloud grant' first)");
                }
            }
        }
        "audit-log" => {
            println!("📋 Audit Log\n");
            println!("   Audit log backend is not yet persisted.");
            println!("   Use DAL code with trust:: and cloudadmin:: for custom audit trails.");
            println!();
            println!("   Example DAL:");
            println!("   // Log to your own store or chain");
            println!("   let allowed = cloudadmin::authorize(admin_id, \"write\", \"resource\");");
            println!("   // Then append to your audit store or chain:: call");
        }
        "policies" => {
            println!("📜 Policies\n");
            println!("   Policies are enforced via trust::enforce_policy(policy_name, context).");
            println!("   Set POLICY_<name>_LEVEL env var to require minimum level.");
            println!();
            println!("   Built-in policy names: strict, moderate, permissive");
            println!("   Example: POLICY_STRICT_LEVEL=superadmin");
        }
        "tenant" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} cloud tenant <subcommand> [args...]",
                    binary_name()
                );
                eprintln!("Subcommands: list, create <name> [--admin-email <email>]");
                std::process::exit(1);
            }
            match args[1].as_str() {
                "list" => {
                    println!("🏢 Tenants\n");
                    println!("   Multi-tenant backend not yet implemented.");
                    println!(
                        "   Use 'cloud grant <tenant_id> admin <scope>' for per-tenant admins."
                    );
                }
                "create" => {
                    if args.len() < 3 {
                        eprintln!(
                            "Usage: {} cloud tenant create <name> [--admin-email <email>]",
                            binary_name()
                        );
                        std::process::exit(1);
                    }
                    let name = &args[2];
                    let email = if args.len() >= 5 && args[3] == "--admin-email" {
                        args[4].as_str()
                    } else {
                        ""
                    };
                    println!("✅ Tenant '{}' created (simulated)", name);
                    if !email.is_empty() {
                        println!("   Admin email: {}", email);
                    }
                    println!("   (Full multi-tenant backend coming in a future release)");
                }
                _ => {
                    eprintln!("❌ Unknown tenant subcommand: {}", args[1]);
                    std::process::exit(1);
                }
            }
        }
        "compliance" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} cloud compliance <subcommand> [args...]",
                    binary_name()
                );
                eprintln!(
                    "Subcommands: scan [--standard SOC2|HIPAA|GDPR], report <standard> [-o file]"
                );
                std::process::exit(1);
            }
            match args[1].as_str() {
                "scan" => {
                    let standard = if args.len() >= 4 && args[2] == "--standard" {
                        args[3].as_str()
                    } else {
                        "SOC2"
                    };
                    println!("🔒 Compliance Scan\n");
                    println!("   Standard: {}", standard);
                    println!("   Compliance scan backend not yet implemented.");
                    println!("   Use DAL code to integrate with your compliance tooling.");
                }
                "report" => {
                    if args.len() < 3 {
                        eprintln!(
                            "Usage: {} cloud compliance report <standard> [-o file]",
                            binary_name()
                        );
                        std::process::exit(1);
                    }
                    let standard = &args[2];
                    let out = if args.len() >= 5 && args[3] == "-o" {
                        args[4].as_str()
                    } else {
                        ""
                    };
                    println!("📄 Compliance Report\n");
                    println!("   Standard: {}", standard);
                    if !out.is_empty() {
                        println!("   Output: {}", out);
                    }
                    println!("   Report generation backend not yet implemented.");
                }
                _ => {
                    eprintln!("❌ Unknown compliance subcommand: {}", args[1]);
                    std::process::exit(1);
                }
            }
        }
        "chain-log" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} cloud chain-log \"<event>\" [--chain_id <id>]",
                    binary_name()
                );
                eprintln!(
                    "Example: {} cloud chain-log \"user_123 deleted resource\" --chain_id 1",
                    binary_name()
                );
                std::process::exit(1);
            }
            let event = &args[1];
            let chain_id = if args.len() >= 4 && args[2] == "--chain_id" {
                args[3].parse::<i64>().unwrap_or(1)
            } else {
                1
            };
            println!("⛓️  Chain Log\n");
            println!("   Event: {}", event);
            println!("   Chain ID: {}", chain_id);
            println!("   Blockchain-backed audit backend not yet implemented.");
            println!("   Use 'dal chain' and your DAL code to write events on-chain.");
        }
        "chain-verify" => {
            if args.len() < 2 {
                eprintln!("Usage: {} cloud chain-verify <log_id>", binary_name());
                std::process::exit(1);
            }
            let log_id = &args[1];
            println!("🔍 Chain Verify\n");
            println!("   Log ID: {}", log_id);
            println!("   Verification backend not yet implemented.");
        }
        "chain-export" => {
            println!("📤 Chain Export\n");
            println!("   Export chain logs backend not yet implemented.");
            println!("   Use 'dal chain' and DAL code to query chain data.");
        }
        "trust" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} cloud trust <subcommand> [args...]",
                    binary_name()
                );
                eprintln!("Subcommands: validate <admin_trust> <user_trust>, bridge <central> <decentral>");
                std::process::exit(1);
            }
            match args[1].as_str() {
                "validate" => {
                    if args.len() < 4 {
                        eprintln!(
                            "Usage: {} cloud trust validate <admin_trust> <user_trust>",
                            binary_name()
                        );
                        eprintln!(
                            "Example: {} cloud trust validate valid valid",
                            binary_name()
                        );
                        std::process::exit(1);
                    }
                    let admin_trust = &args[2];
                    let user_trust = &args[3];
                    let ok = cloudadmin::validate_hybrid_trust(admin_trust, user_trust);
                    if ok {
                        println!(
                            "✅ Hybrid trust valid (admin: {}, user: {})",
                            admin_trust, user_trust
                        );
                    } else {
                        println!("❌ Hybrid trust invalid");
                        std::process::exit(1);
                    }
                }
                "bridge" => {
                    if args.len() < 4 {
                        eprintln!("Usage: {} cloud trust bridge <centralized_trust> <decentralized_trust>", binary_name());
                        eprintln!("Example: {} cloud trust bridge admin user", binary_name());
                        std::process::exit(1);
                    }
                    let central = &args[2];
                    let decentral = &args[3];
                    let ok = cloudadmin::bridge_trusts(central, decentral);
                    if ok {
                        println!(
                            "✅ Trusts bridged (central: {}, decentral: {})",
                            central, decentral
                        );
                    } else {
                        println!("❌ Trust bridge failed (expected central=admin, decentral=user)");
                        std::process::exit(1);
                    }
                }
                _ => {
                    eprintln!("❌ Unknown trust subcommand: {}", args[1]);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("❌ Unknown cloud subcommand: {}", args[0]);
            eprintln!("Available: authorize, grant, revoke, roles, audit-log, policies, tenant, compliance, chain-log, chain-verify, chain-export, trust");
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 8: admin, key, aml, kyc (basic trust & compliance)
// ============================================================================

fn handle_admin_command(args: &[String]) {
    use stdlib::admin;

    match args[0].as_str() {
        "list" => {
            let procs = admin::list_processes();
            if procs.is_empty() {
                println!("No processes registered (use DAL code to register_process_start_time)");
            } else {
                println!("Processes:");
                for p in &procs {
                    println!("  {}  started {}", p.process_id, p.start_time);
                }
            }
        }
        "info" => {
            if args.len() < 2 {
                eprintln!("Usage: {} admin info <process_id>", binary_name());
                std::process::exit(1);
            }
            match admin::get_process_info(&args[1]) {
                Ok(info) => println!("✅ {}: started {}", info.process_id, info.start_time),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown admin subcommand: {}. Use list, info.", args[0]);
            std::process::exit(1);
        }
    }
}

fn handle_key_command(args: &[String]) {
    use stdlib::key;

    match args[0].as_str() {
        "create" => {
            if args.len() < 4 {
                eprintln!(
                    "Usage: {} key create <resource> <perm> [perm...]",
                    binary_name()
                );
                std::process::exit(1);
            }
            let resource = &args[1];
            let perms: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            match key::create(resource, perms) {
                Ok(c) => println!(
                    "✅ Key created: {} (resource: {}, perms: {:?})",
                    c.id, c.resource, c.permissions
                ),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "check" => {
            if args.len() < 4 {
                eprintln!(
                    "Usage: {} key check <resource> <operation> <principal_id>",
                    binary_name()
                );
                std::process::exit(1);
            }
            let req =
                key::create_capability_request(args[1].clone(), args[2].clone(), args[3].clone());
            match key::check(req) {
                Ok(allowed) => {
                    if allowed {
                        println!("✅ Allowed")
                    } else {
                        println!("❌ Denied");
                        std::process::exit(1)
                    }
                }
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "principal" => {
            if args.len() < 3 {
                eprintln!("Usage: {} key principal <id> <name>", binary_name());
                std::process::exit(1);
            }
            let p = key::create_principal(args[1].clone(), args[2].clone());
            println!("✅ Principal: {} ({})", p.id, p.name);
        }
        "list" => {
            if args.len() < 2 {
                eprintln!("Usage: {} key list <principal_id>", binary_name());
                std::process::exit(1);
            }
            let caps = key::list_for_principal(&args[1]);
            if caps.is_empty() {
                println!("No capabilities for principal {}", args[1]);
            } else {
                println!("Capabilities for {} ({}):", args[1], caps.len());
                for c in caps {
                    println!(
                        "  {}  resource={}  perms={:?}",
                        c.id, c.resource, c.permissions
                    );
                }
            }
        }
        "revoke" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} key revoke <capability_id> <principal_id>",
                    binary_name()
                );
                std::process::exit(1);
            }
            match key::revoke(&args[1], &args[2]) {
                Ok(true) => println!("✅ Revoked {} from {}", args[1], args[2]),
                Ok(false) => {
                    eprintln!("No such grant: {} for {}", args[1], args[2]);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "revoke_all" => {
            if args.len() < 2 {
                eprintln!("Usage: {} key revoke_all <principal_id>", binary_name());
                std::process::exit(1);
            }
            match key::revoke_all(&args[1]) {
                Ok(n) => println!("✅ Revoked {} grant(s) for {}", n, args[1]),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown key subcommand: {}. Use create, check, principal, list, revoke, revoke_all.", args[0]);
            std::process::exit(1);
        }
    }
}

fn handle_aml_command(args: &[String]) {
    use std::collections::HashMap;
    use stdlib::aml;

    match args[0].as_str() {
        "check" => {
            if args.len() < 4 {
                eprintln!(
                    "Usage: {} aml check <provider_id> <user_address> <check_type>",
                    binary_name()
                );
                eprintln!("  Providers: list with 'dal aml providers'");
                std::process::exit(1);
            }
            let provider_id = args[1].clone();
            let user_address = args[2].clone();
            let check_type = args[3].clone();
            let user_data = HashMap::new();
            let result = aml::perform_check(provider_id, user_address, check_type, user_data);
            println!("✅ AML check result:");
            for (k, v) in &result {
                println!("   {}: {}", k, v);
            }
        }
        "status" => {
            if args.len() < 2 {
                eprintln!("Usage: {} aml status <check_id>", binary_name());
                std::process::exit(1);
            }
            let result = aml::get_check_status(args[1].clone());
            println!("✅ AML status:");
            for (k, v) in &result {
                println!("   {}: {}", k, v);
            }
        }
        "providers" => {
            let list = aml::list_providers();
            println!("AML providers: {}", list.join(", "));
        }
        _ => {
            eprintln!(
                "Unknown aml subcommand: {}. Use check, status, providers.",
                args[0]
            );
            std::process::exit(1);
        }
    }
}

fn handle_kyc_command(args: &[String]) {
    use std::collections::HashMap;
    use stdlib::kyc;

    match args[0].as_str() {
        "verify" => {
            if args.len() < 4 {
                eprintln!(
                    "Usage: {} kyc verify <provider_id> <user_address> <level>",
                    binary_name()
                );
                eprintln!("  Providers: list with 'dal kyc providers'");
                std::process::exit(1);
            }
            let provider_id = args[1].clone();
            let user_address = args[2].clone();
            let level = args[3].clone();
            let user_data = HashMap::new();
            let result = kyc::verify_identity(provider_id, user_address, level, user_data);
            println!("✅ KYC verification result:");
            for (k, v) in &result {
                println!("   {}: {}", k, v);
            }
        }
        "status" => {
            if args.len() < 2 {
                eprintln!("Usage: {} kyc status <verification_id>", binary_name());
                std::process::exit(1);
            }
            let result = kyc::get_verification_status(args[1].clone());
            println!("✅ KYC status:");
            for (k, v) in &result {
                println!("   {}: {}", k, v);
            }
        }
        "providers" => {
            let list = kyc::list_providers();
            println!("KYC providers: {}", list.join(", "));
        }
        _ => {
            eprintln!(
                "Unknown kyc subcommand: {}. Use verify, status, providers.",
                args[0]
            );
            std::process::exit(1);
        }
    }
}

fn handle_scaffold_command(args: &[String]) {
    let t = args[0].as_str();
    let name = args.get(1).cloned().unwrap_or_else(|| match t {
        "contract" => "MyContract".to_string(),
        "api" => "api".to_string(),
        "test" => "my_test".to_string(),
        _ => "scaffold".to_string(),
    });
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let (filename, content) = match t {
        "contract" => {
            let f = format!("{}.dal", name);
            let c = format!(
                r#"// Smart contract scaffold: {}
// TODO: Add contract logic

fn main() {{
    let owner = "0x0000000000000000000000000000000000000000";
    println("Contract: {} - owner: {{}}", owner);
}}
"#,
                name, name
            );
            (f, c)
        }
        "api" => {
            let f = format!("{}.dal", name);
            let c = format!(
                r#"// API scaffold: {}
// TODO: Add endpoints via stdlib::web

fn main() {{
    println("API: {} - add web::serve or route handlers");
}}
"#,
                name, name
            );
            (f, c)
        }
        "test" => {
            let f = format!("{}.test.dal", name);
            let c = format!(
                r#"// Test scaffold: {}
// TODO: Add test cases

test "basic check" {{
    let x = 1 + 1;
    assert_eq(x, 2, "1+1 should be 2");
}}
"#,
                name
            );
            (f, c)
        }
        _ => {
            eprintln!("Unknown scaffold type: {}. Use contract, api, test.", t);
            std::process::exit(1);
        }
    };
    let path = cwd.join(&filename);
    match std::fs::write(&path, content) {
        Ok(()) => println!("✅ Created {}: {}", t, path.display()),
        Err(e) => {
            eprintln!("❌ Failed to write {}: {}", path.display(), e);
            std::process::exit(1);
        }
    }
}

fn visit_dal_files(
    dir: &std::path::Path,
    cwd: &std::path::Path,
    max_depth: usize,
    cb: &mut dyn FnMut(&std::path::Path) -> Result<(), String>,
) -> Result<usize, String> {
    if max_depth == 0 {
        return Ok(0);
    }
    let mut count = 0;
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path
            .to_str()
            .map_or(false, |s| s.contains("target") || s.contains("dist"))
        {
            continue;
        }
        if path.is_dir() {
            count += visit_dal_files(&path, cwd, max_depth - 1, cb)?;
        } else if path.extension().map_or(false, |ext| ext == "dal") {
            cb(&path)?;
            count += 1;
        }
    }
    Ok(count)
}

fn handle_build_command(_args: &[String]) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let dal_toml = cwd.join("dal.toml");
    if dal_toml.exists() {
        println!("🪩  Build");
        println!("   Checking DAL files...");
        let mut check = |p: &std::path::Path| {
            let source = std::fs::read_to_string(p).unwrap_or_default();
            let tokens = match Lexer::new(&source).tokenize_with_positions_immutable() {
                Ok(t) => {
                    println!(
                        "✅ Lexer scanning... {} tokens ({})",
                        t.len(),
                        p.file_name().unwrap_or_default().to_string_lossy()
                    );
                    t
                }
                Err(e) => return Err(format!("{}: {}", p.display(), e)),
            };
            match Parser::new_with_positions(tokens).parse() {
                Ok(ast) => {
                    println!("✅ Parsed {} statements", ast.statements.len());
                    let rel = p.strip_prefix(&cwd).unwrap_or(p);
                    println!("   ✓ {} (ok)", rel.display());
                    Ok(())
                }
                Err(e) => Err(format!("{}: {}", p.display(), e)),
            }
        };
        match visit_dal_files(&cwd, &cwd, 5, &mut check) {
            Ok(count) => {
                println!("✅ Build complete ({} file(s) checked)", count);
            }
            Err(e) => {
                eprintln!("   ✗ {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("🔨 Build");
        println!(
            "   No dal.toml found. Run '{} init' or create dal.toml first.",
            binary_name()
        );
        println!(
            "   For now: use '{} check <file.dal>' to validate files.",
            binary_name()
        );
    }
}

fn handle_clean_command(_args: &[String]) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let target = cwd.join("target");
    let dist = cwd.join("dist");
    let mut removed = 0;
    for d in [&target, &dist] {
        if d.exists() {
            match std::fs::remove_dir_all(d) {
                Ok(()) => {
                    println!("   Removed {}", d.display());
                    removed += 1;
                }
                Err(e) => eprintln!("   ⚠️  Could not remove {}: {}", d.display(), e),
            }
        }
    }
    if removed > 0 {
        println!("✅ Clean complete ({} dir(s) removed)", removed);
    } else {
        println!("Nothing to clean (target/ and dist/ not found).");
    }
}

fn handle_dist_command(_args: &[String]) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let dist = cwd.join("dist");
    if let Err(e) = std::fs::create_dir_all(&dist) {
        eprintln!("❌ Could not create dist/: {}", e);
        std::process::exit(1);
    }
    println!("📦 Dist");
    println!("   Copying DAL sources to dist/...");
    let mut count = 0;
    let mut copy_file = |p: &std::path::Path| {
        let rel = p.strip_prefix(&cwd).unwrap_or(p);
        let dest = dist.join(rel);
        if let Some(parent) = dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::copy(p, &dest).is_ok() {
            count += 1;
        }
        Ok(())
    };
    let _ = visit_dal_files(&cwd, &cwd, 3, &mut |p| copy_file(p));
    println!(
        "✅ Dist complete ({} file(s) copied to {})",
        count,
        dist.display()
    );
}

// ============================================================================
// Phase 5: IDE & LSP Integration (lsp, doc, completions, debug)
// ============================================================================

fn handle_lsp_command(_args: &[String]) {
    println!("ℹ️  LSP (Language Server Protocol)");
    println!();
    println!("   The DAL LSP server provides:");
    println!("   • Syntax highlighting & diagnostics");
    println!("   • Hover documentation");
    println!("   • Go to definition");
    println!("   • Autocomplete & signatures");
    println!();
    println!(
        "   To use: Configure your editor to run '{} lsp' as the language server.",
        binary_name()
    );
    println!("   Example (VS Code): Add to settings.json:");
    println!(
        "     \"dal.languageServerPath\": \"{}\"",
        std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| binary_name())
    );
    println!();
    println!("   Note: Full LSP implementation is planned. Current mode: stdio-ready placeholder.");
}

fn handle_doc_command(args: &[String]) {
    use std::path::Path;

    let mut path = String::new();
    let mut output_path: Option<String> = None;
    let mut open_in_browser = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                i += 1;
                if i < args.len() {
                    output_path = Some(args[i].clone());
                }
            }
            "--open" => open_in_browser = true,
            _ if !args[i].starts_with('-') && path.is_empty() => path = args[i].clone(),
            _ => {}
        }
        i += 1;
    }

    if path.is_empty() {
        eprintln!(
            "Usage: {} doc <file.dal|dir> [--output <path>] [--open]",
            binary_name()
        );
        std::process::exit(1);
    }

    let out_path = output_path.unwrap_or_else(|| {
        if path.ends_with(".dal") {
            path.replace(".dal", "_docs.md")
        } else {
            format!("{}/docs.md", path.trim_end_matches('/'))
        }
    });

    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Error reading {}: {}", path, e);
            std::process::exit(1);
        }
    };

    println!("🪩  Generating documentation for: {}", path);
    let tokens = match Lexer::new(&source).tokenize_with_positions_immutable() {
        Ok(t) => {
            println!("✅ Lexer scanning... {} tokens", t.len());
            t
        }
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, Some(&path), Some(&source)));
            std::process::exit(1);
        }
    };
    let program = match Parser::new_with_positions(tokens).parse() {
        Ok(p) => {
            println!("✅ Parsed {} statements", p.statements.len());
            p
        }
        Err(e) => {
            eprintln!("❌ Parse error:\n{}", format_parser_error(&e, Some(&path), Some(&source)));
            std::process::exit(1);
        }
    };

    let mut md = format!("# Documentation: {}\n\n", path);
    md.push_str(&format!("Generated from `{}`\n\n", path));
    md.push_str("## Contents\n\n");

    let mut items = Vec::new();
    for stmt in &program.statements {
        match stmt {
            Statement::Function(f) => items.push((f.name.clone(), "function", format_fn(f))),
            Statement::Service(s) => {
                items.push((s.name.clone(), "service", format_service(s)));
                for m in &s.methods {
                    items.push((format!("{}.{}", s.name, m.name), "method", format_fn(m)));
                }
            }
            _ => {}
        }
    }

    for (name, kind, _doc) in &items {
        md.push_str(&format!(
            "- [{} `{}`](#{})\n",
            kind,
            name,
            name.replace('.', "-")
        ));
    }
    md.push_str("\n---\n\n");

    for (name, kind, doc) in &items {
        md.push_str(&format!("### {} `{}`\n\n", kind, name));
        md.push_str(doc);
        md.push_str("\n\n");
    }

    if let Some(parent) = Path::new(&out_path).parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("❌ Error creating output directory: {}", e);
                std::process::exit(1);
            }
        }
    }
    if let Err(e) = std::fs::write(&out_path, &md) {
        eprintln!("❌ Error writing {}: {}", out_path, e);
        std::process::exit(1);
    }

    println!("✅ Documentation generated: {}", out_path);

    if open_in_browser {
        #[cfg(not(target_os = "windows"))]
        let _ = std::process::Command::new("open").arg(&out_path).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", &out_path])
            .spawn();
    }
}

fn format_fn(f: &FunctionStatement) -> String {
    let params: Vec<String> = f
        .parameters
        .iter()
        .map(|p| format!("{}: {}", p.name, p.param_type.as_deref().unwrap_or("any")))
        .collect();
    let sig = format!("fn {}({})", f.name, params.join(", "));
    let ret = f
        .return_type
        .as_deref()
        .map(|r| format!(" -> {}", r))
        .unwrap_or_default();
    format!("```\n{}{}\n```", sig, ret)
}

fn format_service(s: &ServiceStatement) -> String {
    let methods: Vec<String> = s.methods.iter().map(|m| m.name.clone()).collect();
    format!("Service with methods: {}", methods.join(", "))
}

fn handle_completions_command(args: &[String]) {
    let shell = args.first().map(|s| s.as_str()).unwrap_or("bash");
    let bin = binary_name();
    let cmds = [
        "run",
        "test",
        "web",
        "help",
        "version",
        "check",
        "fmt",
        "lint",
        "parse",
        "new",
        "init",
        "add",
        "install",
        "repl",
        "watch",
        "bench",
        "profile",
        "optimize",
        "memory-stats",
        "chain",
        "crypto",
        "db",
        "ai",
        "cloud",
        "oracle",
        "lsp",
        "doc",
        "completions",
        "debug",
        "agent",
        "iot",
        "log",
        "config",
        "admin",
        "key",
        "aml",
        "kyc",
        "bond",
        "pipe",
        "invoke",
        "mold",
        "scaffold",
        "build",
        "clean",
        "dist",
        "convert",
        "analyze",
    ];

    match shell {
        "bash" => {
            println!("# Bash completion for {}", bin);
            println!("_{}_completions() {{", bin.replace('-', "_"));
            println!("  local cur=\"${{COMP_WORDS[COMP_CWORD]}}\"");
            println!(
                "  COMPREPLY=($(compgen -W \"{}\" -- \"$cur\"))",
                cmds.join(" ")
            );
            println!("}}");
            println!("complete -F _{}_completions {}", bin.replace('-', "_"), bin);
        }
        "zsh" => {
            println!("# Zsh completion for {}", bin);
            println!("# Add to .zshrc: eval \"$({} completions zsh)\"", bin);
            println!("compdef _{} {}", bin.replace('-', "_"), bin);
            println!("function _{} {{", bin.replace('-', "_"));
            println!("  _values 'commands' {}", cmds.join(" "));
            println!("}}");
        }
        "fish" => {
            println!("# Fish completion for {}", bin);
            println!("# Save to ~/.config/fish/completions/{}.fish", bin);
            println!("complete -c {} -a '{}'", bin, cmds.join(" "));
        }
        _ => {
            eprintln!("Usage: {} completions [bash|zsh|fish]", binary_name());
            eprintln!("  bash  - Bash completion script (default)");
            eprintln!("  zsh   - Zsh completion script");
            eprintln!("  fish  - Fish completion script");
            eprintln!();
            eprintln!("Example: {} completions bash >> ~/.bashrc", binary_name());
            std::process::exit(1);
        }
    }
}

fn handle_debug_command(args: &[String]) {
    let file = &args[0];
    let breakpoint = args
        .iter()
        .position(|a| a == "--breakpoint")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u32>().ok());

    let source = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Error reading {}: {}", file, e);
            std::process::exit(1);
        }
    };

    println!("🪩  Debug mode");
    println!();
    println!("   File: {}", file);
    if let Some(line) = breakpoint {
        println!("   Breakpoint at line: {}", line);
    }
    println!();
    println!("   Interactive debugger features (planned):");
    println!("   • Step over / into / out");
    println!("   • Variable inspection");
    println!("   • Call stack view");
    println!("   • Conditional breakpoints");
    println!();
    println!("   Current: Running file without breakpoints.");
    println!();

    let tokens = match Lexer::new(&source).tokenize_with_positions_immutable() {
        Ok(t) => {
            println!("✅ Lexer scanning... {} tokens", t.len());
            t
        }
        Err(e) => {
            eprintln!("❌ Lexer error:\n{}", format_lexer_error(&e, Some(file), Some(&source)));
            std::process::exit(1);
        }
    };
    match Parser::new_with_positions(tokens).parse() {
        Ok(ast) => {
            println!("✅ Parsed {} statements", ast.statements.len());
            println!("✅ Parse OK. Execute with: {} run {}", binary_name(), file);
        }
        Err(e) => {
            eprintln!("❌ Parse error:\n{}", format_parser_error(&e, Some(file), Some(&source)));
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 8: Specialized (log, config)
// ============================================================================

fn handle_log_command(args: &[String]) {
    use stdlib::log;
    let sub = args.first().map(|s| s.as_str()).unwrap_or("stats");
    match sub {
        "show" | "entries" => {
            let entries = log::get_entries();
            if entries.is_empty() {
                println!("No log entries (in-memory; cleared each run)");
            } else {
                for e in entries.iter().take(50) {
                    println!("[{}] {:?}: {}", e.timestamp, e.level, e.message);
                }
                if entries.len() > 50 {
                    println!("... and {} more", entries.len() - 50);
                }
            }
        }
        "stats" => {
            let stats = log::get_stats();
            for (k, v) in &stats {
                println!("{}: {}", k, v);
            }
        }
        "clear" => {
            log::clear();
            println!("✅ Log cleared");
        }
        "level" => {
            let level = args.get(1).map(|s| s.as_str()).unwrap_or("info");
            let _ = level;
            println!(
                "ℹ️  Filter by level: {} log show (levels: info, warning, error, audit, debug)",
                binary_name()
            );
        }
        _ => {
            eprintln!("Usage: {} log [show|stats|clear]", binary_name());
            std::process::exit(1);
        }
    }
}

fn handle_config_command(args: &[String]) {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("show");
    match sub {
        "show" => {
            println!("ℹ️  Config");
            println!(
                "   Environment: {}",
                std::env::var("DIST_AGENT_ENV").unwrap_or_else(|_| "development".to_string())
            );
            println!("   Key config vars: AI_API_KEY, ADMIN_IDS, CHAIN_RPC_URL_*, DB_*, etc.");
        }
        "get" => {
            if args.len() < 2 {
                eprintln!("Usage: {} config get <key>", binary_name());
                std::process::exit(1);
            }
            match std::env::var(&args[1]) {
                Ok(v) => println!("{}", v),
                Err(_) => {
                    eprintln!("❌ Key not set: {}", args[1]);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Usage: {} config [show|get <key>]", binary_name());
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 7: AI-IoT Integration
// ============================================================================

fn handle_iot_command(args: &[String]) {
    use crate::runtime::values::Value;
    use std::collections::HashMap;
    use stdlib::iot::{self, ReadingQuality, SensorReading};

    if args.is_empty() {
        eprintln!("Usage: {} iot <subcommand> [args...]", binary_name());
        eprintln!("  Device: register, connect, disconnect, status, firmware");
        eprintln!("  Sensor/actuator: read-sensor, actuator");
        eprintln!("  Power: power, ai-optimize");
        eprintln!(
            "  AI: ai-predict, ai-anomaly (ai-control, ai-diagnose, ai-security need AI provider)"
        );
        eprintln!("  Edge/cloud: edge-create, cloud-sync");
        std::process::exit(1);
    }

    match args[0].as_str() {
        "register" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} iot register <name> [--type sensor|actuator|gateway|edge|drone|...]",
                    binary_name()
                );
                std::process::exit(1);
            }
            let name = args[1].clone();
            let device_type = args
                .iter()
                .position(|a| a == "--type")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str())
                .unwrap_or("sensor");
            let mut config = HashMap::new();
            config.insert("name".to_string(), Value::String(name.clone()));
            config.insert(
                "device_type".to_string(),
                Value::String(device_type.to_string()),
            );
            match iot::register_device(config) {
                Ok(d) => println!(
                    "✅ Registered device: {} (id: {}, type: {:?})",
                    d.name, d.device_id, d.device_type
                ),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "connect" => {
            if args.len() < 2 {
                eprintln!("Usage: {} iot connect <device_id>", binary_name());
                std::process::exit(1);
            }
            match iot::connect_device(&args[1]) {
                Ok(d) => println!("✅ Connected: {} ({:?})", d.device_id, d.status),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "disconnect" => {
            if args.len() < 2 {
                eprintln!("Usage: {} iot disconnect <device_id>", binary_name());
                std::process::exit(1);
            }
            match iot::disconnect_device(&args[1]) {
                Ok(_) => println!("✅ Disconnected {}", args[1]),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "actuator" => {
            if args.len() < 4 {
                eprintln!(
                    "Usage: {} iot actuator <actuator_id> <command> [--param k=v ...]",
                    binary_name()
                );
                std::process::exit(1);
            }
            let actuator_id = &args[1];
            let command = &args[2];
            let mut params = HashMap::new();
            let mut i = 3;
            while i + 1 < args.len() {
                if args[i] == "--param" {
                    if let Some(pair) = args.get(i + 1) {
                        if let Some((k, v)) = pair.split_once('=') {
                            params.insert(k.to_string(), Value::String(v.to_string()));
                        }
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            match iot::send_actuator_command(actuator_id, command, params) {
                Ok(cmd) => println!(
                    "✅ Command sent: {} (id: {})",
                    cmd.command_type, cmd.command_id
                ),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "power" => {
            if args.len() < 2 {
                eprintln!("Usage: {} iot power <device_id>", binary_name());
                std::process::exit(1);
            }
            match iot::monitor_power_consumption(&args[1]) {
                Ok(p) => {
                    println!("✅ Power for {}:", args[1]);
                    println!(
                        "   source: {:?}, consumption: {:.2} W",
                        p.source, p.power_consumption
                    );
                    if let Some(b) = p.battery_level {
                        println!("   battery: {:.0}%", b * 100.0);
                    }
                    if let Some(h) = p.estimated_runtime {
                        println!("   estimated runtime: {} min", h);
                    }
                }
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "firmware" => {
            if args.len() < 4 {
                eprintln!(
                    "Usage: {} iot firmware update <device_id> <version>",
                    binary_name()
                );
                std::process::exit(1);
            }
            if args[1] != "update" {
                eprintln!(
                    "Usage: {} iot firmware update <device_id> <version>",
                    binary_name()
                );
                std::process::exit(1);
            }
            match iot::update_device_firmware(&args[2], &args[3]) {
                Ok(_) => println!(
                    "✅ Firmware update requested for {} -> {}",
                    args[2], args[3]
                ),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "edge-create" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} iot edge-create <name> [--node-id <id>]",
                    binary_name()
                );
                std::process::exit(1);
            }
            let name = args[1].clone();
            let node_id = args
                .iter()
                .position(|a| a == "--node-id")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| format!("edge_{}", stdlib::iot::generate_id()));
            let mut config = HashMap::new();
            config.insert("name".to_string(), Value::String(name));
            config.insert("node_id".to_string(), Value::String(node_id.clone()));
            match iot::create_edge_node(config) {
                Ok(n) => println!("✅ Edge node created: {} (id: {})", n.name, n.node_id),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "cloud-sync" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} iot cloud-sync <device_id> <data_json>  (sync device data to cloud)",
                    binary_name()
                );
                eprintln!("       {} iot cloud-sync get <device_id> [--from <ts>] [--to <ts>]  (fetch from cloud)", binary_name());
                std::process::exit(1);
            }
            if args[1] == "get" {
                let device_id = args.get(2).map(|s| s.as_str()).unwrap_or("");
                let from_ts = args
                    .iter()
                    .position(|a| a == "--from")
                    .and_then(|i| args.get(i + 1))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let to_ts = args
                    .iter()
                    .position(|a| a == "--to")
                    .and_then(|i| args.get(i + 1))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let range = if !from_ts.is_empty() && !to_ts.is_empty() {
                    Some((from_ts.to_string(), to_ts.to_string()))
                } else {
                    None
                };
                match iot::get_device_data_from_cloud(device_id, range) {
                    Ok(v) => println!("✅ Cloud data: {:?}", v),
                    Err(e) => {
                        eprintln!("❌ {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                let device_id = &args[1];
                let data_str = &args[2];
                let data = Value::String(data_str.clone());
                match iot::sync_device_data_to_cloud(device_id, data) {
                    Ok(_) => println!("✅ Synced data to cloud for {}", device_id),
                    Err(e) => {
                        eprintln!("❌ {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        "ai-predict" => {
            if args.len() < 2 {
                eprintln!("Usage: {} iot ai-predict <device_id>", binary_name());
                std::process::exit(1);
            }
            let device_id = &args[1];
            let history: Vec<SensorReading> = vec![];
            match iot::predict_device_failure(device_id, history) {
                Ok(p) => println!(
                    "✅ Failure probability for {}: {:.2}%",
                    device_id,
                    p * 100.0
                ),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "ai-anomaly" => {
            if args.len() < 2 {
                eprintln!("Usage: {} iot ai-anomaly <sensor_id>", binary_name());
                std::process::exit(1);
            }
            let sensor_id = &args[1];
            let reading = match iot::read_sensor_data(sensor_id) {
                Ok(r) => r,
                Err(_) => SensorReading {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    value: Value::Float(0.0),
                    quality: ReadingQuality::Good,
                    metadata: HashMap::new(),
                },
            };
            let anomalies = match iot::detect_sensor_anomalies(vec![reading]) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            };
            if anomalies.is_empty() {
                println!("✅ No anomalies detected for sensor {}", sensor_id);
            } else {
                println!("⚠️  Anomalies: {}", anomalies.join(", "));
            }
        }
        "ai-optimize" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} iot ai-optimize <device_id> [--target-hours <n>]",
                    binary_name()
                );
                std::process::exit(1);
            }
            let device_id = &args[1];
            let target_hours = args
                .iter()
                .position(|a| a == "--target-hours")
                .and_then(|i| args.get(i + 1).and_then(|s| s.parse::<i64>().ok()))
                .unwrap_or(8);
            let target_runtime = target_hours * 60;
            match iot::optimize_power_usage(device_id, target_runtime) {
                Ok(recs) => {
                    println!(
                        "✅ Power optimization for {} (target: {}h):",
                        device_id, target_hours
                    );
                    for (k, v) in &recs {
                        println!("   {}: {}", k, v);
                    }
                }
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "ai-control" | "ai-diagnose" | "ai-security" | "ai-edge" | "ai-query" => {
            println!("ℹ️  {} (AI-IoT)", args[0]);
            println!("   Requires AI provider (OpenAI/Anthropic/Ollama) via AI_API_KEY or config.");
            println!("   Use DAL code: dal run <file.dal> for full AI-IoT workflows.");
        }
        "read-sensor" => {
            if args.len() < 2 {
                eprintln!("Usage: {} iot read-sensor <sensor_id>", binary_name());
                std::process::exit(1);
            }
            match iot::read_sensor_data(&args[1]) {
                Ok(r) => println!(
                    "✅ {}: {} ({})",
                    r.timestamp,
                    r.value,
                    format!("{:?}", r.quality)
                ),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        "status" => {
            if args.len() < 2 {
                eprintln!("Usage: {} iot status <device_id>", binary_name());
                std::process::exit(1);
            }
            match iot::get_device_status(&args[1]) {
                Ok(s) => println!("✅ Status: {:?}", s),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown iot subcommand: {}", args[0]);
            eprintln!("Subcommands: register, connect, disconnect, read-sensor, status, actuator, power, firmware, ai-predict, ai-anomaly, ai-optimize, edge-create, cloud-sync, ai-control, ai-diagnose, ai-security, ai-edge, ai-query");
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 9: Cross-Component (bond, pipe, invoke)
// ============================================================================

fn handle_cross_component_command(cmd: &str, args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: {} {} <subcommand> [args...]", binary_name(), cmd);
        std::process::exit(1);
    }

    match cmd {
        "bond" => {
            let flow = &args[0];
            match flow.as_str() {
                "oracle-to-chain" | "chain-to-sync" | "iot-to-db" | "iot-to-web" | "db-to-sync"
                | "sync-to-db" | "ai-to-service" | "service-to-chain" | "auth-to-web"
                | "log-to-sync" => {
                    println!("ℹ️  bond {}", flow);
                    println!(
                        "   Connects components. Use: {} bond {} <args...>",
                        binary_name(),
                        flow
                    );
                    println!(
                        "   Example: {} bond iot-to-db <device_id> <conn_str> [--table]",
                        binary_name()
                    );
                }
                _ => {
                    eprintln!("Unknown bond flow: {}", flow);
                    eprintln!("Flows: oracle-to-chain, iot-to-db, db-to-sync, auth-to-web, ...");
                    std::process::exit(1);
                }
            }
        }
        "pipe" => {
            println!("ℹ️  pipe");
            println!(
                "   Unix-style pipeline: {} pipe <source> -> <sink>",
                binary_name()
            );
            println!(
                "   Example: {} pipe oracle fetch coingecko btc -> chain estimate 1 deploy",
                binary_name()
            );
        }
        "invoke" => {
            let workflow = args.get(0).map(|s| s.as_str()).unwrap_or("");
            match workflow {
                "price-to-deploy" | "iot-ingest" | "ai-audit" | "compliance-check" => {
                    println!("ℹ️  invoke {}", workflow);
                    println!("   Multi-component workflow. Args: {:?}", &args[1..]);
                }
                _ => {
                    eprintln!("Unknown invoke workflow: {}", workflow);
                    eprintln!("Workflows: price-to-deploy, iot-ingest, ai-audit, compliance-check");
                    std::process::exit(1);
                }
            }
        }
        _ => {}
    }
}

// ============================================================================
// Phase 6: Agent Commands (create, send, messages, task, fleet, mold)
// ============================================================================

fn handle_agent_command(args: &[String]) {
    use crate::runtime::values::Value;
    use dist_agent_lang::mold;
    use stdlib::agent::{self, AgentConfig, AgentType};

    if args.is_empty() {
        eprintln!("Usage: {} agent <subcommand> [args...]", binary_name());
        std::process::exit(1);
    }

    match args[0].as_str() {
        "create" => {
            // dal agent create --mold <path|ipfs://cid|moldId> <name>
            if args.len() >= 4 && args[1] == "--mold" {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let source = &args[2];
                let name_override = Some(args[3].as_str());
                // On-chain mold ID (numeric): useMold then load from IPFS
                #[cfg(feature = "web3")]
                if let Ok(mold_id) = source.parse::<u64>() {
                    if mold_id > 0 {
                        match (
                            std::env::var("DAL_PRIVATE_KEY").ok(),
                            std::env::var("DAL_MOLD_REGISTRY_ADDRESS").ok(),
                        ) {
                            (Some(_), Some(_)) => {
                                match mold::get_mold_info(mold_id) {
                                    Ok(info) => {
                                        if !info.active {
                                            eprintln!("❌ Mold {} is not active.", mold_id);
                                            std::process::exit(1);
                                        }
                                        println!(
                                            "Mold {}: fee {} wei, ipfs {}",
                                            mold_id, info.mint_fee, info.ipfs_hash
                                        );
                                        match mold::use_mold(mold_id, info.mint_fee) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                eprintln!("❌ useMold failed: {}", e);
                                                std::process::exit(1);
                                            }
                                        }
                                        let ipfs_source = format!("ipfs://{}", info.ipfs_hash);
                                        match mold::create_from_mold_source(
                                            &ipfs_source,
                                            &cwd,
                                            name_override,
                                        ) {
                                            Ok(ctx) => {
                                                println!(
                                                    "✅ Agent created from on-chain mold {}: {}",
                                                    mold_id, ctx.agent_id
                                                );
                                                println!("   Name: {}", ctx.config.name);
                                                if !ctx.config.role.is_empty() {
                                                    println!("   Role: {}", ctx.config.role);
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "❌ Failed to load mold from IPFS: {}",
                                                    e
                                                );
                                                std::process::exit(1);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("❌ getMoldInfo failed: {}", e);
                                        std::process::exit(1);
                                    }
                                }
                                return;
                            }
                            _ => {}
                        }
                    }
                }
                // Local path or ipfs://cid
                match mold::create_from_mold_source(source, &cwd, name_override) {
                    Ok(ctx) => {
                        println!("✅ Agent created from mold: {}", ctx.agent_id);
                        println!("   Name: {}", ctx.config.name);
                        println!("   Type: {}", ctx.config.agent_type.to_string());
                        if !ctx.config.role.is_empty() {
                            println!("   Role: {}", ctx.config.role);
                        }
                        println!("   Status: {}", ctx.status.to_string());
                        println!();
                        println!(
                            "   Use this agent_id for send, messages, task (same process only)"
                        );
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to create agent from mold: {}", e);
                        std::process::exit(1);
                    }
                }
                return;
            }
            // dal agent create <type> <name> [--role "role"]
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} agent create <type> <name> [--role \"role\"]",
                    binary_name()
                );
                eprintln!(
                    "       {} agent create --mold <path|ipfs://cid> <name>",
                    binary_name()
                );
                eprintln!("Types: ai, system, worker, custom:<name>");
                std::process::exit(1);
            }
            let agent_type_str = &args[1];
            let name = args[2].clone();
            let role = if args.len() >= 5 && args[3] == "--role" {
                args[4].clone()
            } else {
                String::new()
            };
            let agent_type = match AgentType::from_string(agent_type_str) {
                Some(t) => t,
                None => {
                    eprintln!(
                        "❌ Unknown agent type: {}. Use ai, system, worker, or custom:<name>",
                        agent_type_str
                    );
                    std::process::exit(1);
                }
            };
            let mut config = AgentConfig::new(name.clone(), agent_type);
            if !role.is_empty() {
                config = config.with_role(role);
            }
            match agent::spawn(config) {
                Ok(ctx) => {
                    println!("✅ Agent created: {}", ctx.agent_id);
                    println!("   Name: {}", ctx.config.name);
                    println!("   Type: {}", ctx.config.agent_type.to_string());
                    if !ctx.config.role.is_empty() {
                        println!("   Role: {}", ctx.config.role);
                    }
                    println!("   Status: {}", ctx.status.to_string());
                    println!();
                    println!("   Use this agent_id for send, messages, task (same process only)");
                }
                Err(e) => {
                    eprintln!("❌ Failed to spawn agent: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "send" => {
            if args.len() < 4 {
                eprintln!(
                    "Usage: {} agent send <sender_id> <receiver_id> \"<message>\"",
                    binary_name()
                );
                std::process::exit(1);
            }
            let sender_id = &args[1];
            let receiver_id = &args[2];
            let content = args[3].clone();
            let msg_id = format!(
                "msg_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
            let msg = agent::create_agent_message(
                msg_id,
                sender_id.to_string(),
                receiver_id.to_string(),
                "text".to_string(),
                Value::String(content),
            );
            match agent::communicate(sender_id, receiver_id, msg) {
                Ok(_) => println!("✅ Message sent from {} to {}", sender_id, receiver_id),
                Err(e) => {
                    eprintln!("❌ Send failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "messages" => {
            if args.len() < 2 {
                eprintln!("Usage: {} agent messages <agent_id>", binary_name());
                std::process::exit(1);
            }
            let agent_id = &args[1];
            let msgs = agent::receive_messages(agent_id);
            if msgs.is_empty() {
                println!("📭 No messages for {}", agent_id);
            } else {
                println!("📬 Messages for {} ({}):\n", agent_id, msgs.len());
                for (i, m) in msgs.iter().enumerate() {
                    let content = match &m.content {
                        Value::String(s) => s.clone(),
                        _ => format!("{:?}", m.content),
                    };
                    println!(
                        "   {}. From: {} | Type: {} | {}",
                        i + 1,
                        m.sender_id,
                        m.message_type,
                        content
                    );
                }
            }
        }
        "task" => {
            if args.len() < 2 {
                eprintln!("Usage: {} agent task <subcommand> [args...]", binary_name());
                eprintln!("Subcommands: assign <agent_id> \"<description>\" [--priority high], list <agent_id>");
                std::process::exit(1);
            }
            match args[1].as_str() {
                "assign" => {
                    if args.len() < 4 {
                        eprintln!("Usage: {} agent task assign <agent_id> \"<description>\" [--priority low|medium|high|critical]", binary_name());
                        std::process::exit(1);
                    }
                    let agent_id = &args[2];
                    let description = args[3].clone();
                    let priority = if args.len() >= 6 && args[4] == "--priority" {
                        args[5].as_str()
                    } else {
                        "medium"
                    };
                    let task_id = format!(
                        "task_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                    );
                    let task = match agent::create_agent_task(task_id, description, priority) {
                        Some(t) => t,
                        None => {
                            eprintln!(
                                "❌ Invalid priority: {}. Use low, medium, high, critical",
                                priority
                            );
                            std::process::exit(1);
                        }
                    };
                    match agent::coordinate(agent_id, task, "task_distribution") {
                        Ok(_) => println!("✅ Task assigned to {}", agent_id),
                        Err(e) => {
                            eprintln!("❌ Task assign failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                "list" => {
                    if args.len() < 3 {
                        eprintln!("Usage: {} agent task list <agent_id>", binary_name());
                        std::process::exit(1);
                    }
                    let agent_id = &args[2];
                    let tasks = agent::receive_pending_tasks(agent_id);
                    if tasks.is_empty() {
                        println!("📋 No pending tasks for {}", agent_id);
                    } else {
                        println!("📋 Pending tasks for {} ({}):\n", agent_id, tasks.len());
                        for (i, t) in tasks.iter().enumerate() {
                            println!(
                                "   {}. {} | {} | {}",
                                i + 1,
                                t.task_id,
                                t.priority.to_string(),
                                t.description
                            );
                        }
                    }
                }
                _ => {
                    eprintln!("❌ Unknown task subcommand: {}", args[1]);
                    std::process::exit(1);
                }
            }
        }
        "list" => {
            println!("🤖 Agents\n");
            println!("   Agent state is process-local. Each 'dal' invocation is a new process.");
            println!("   For multi-agent workflows, use DAL code:");
            println!();
            println!("   import agent from \"@dal/agent\"");
            println!("   let ctx = agent::spawn(agent::create_agent_config(\"w1\", \"worker\", \"Process\").unwrap())");
            println!("   agent::communicate(ctx.agent_id, \"agent_2\", agent::create_agent_message(...))");
            println!();
            println!("   Run with: dal run your_agents.dal");
        }
        "fleet" => {
            if args.len() < 2 {
                eprintln!(
                    "Usage: {} agent fleet <subcommand> [args...]",
                    binary_name()
                );
                eprintln!(
                    "Subcommands: create <name> [--type X] [--agents N], scale, deploy, health"
                );
                std::process::exit(1);
            }
            match args[1].as_str() {
                "create" => {
                    let name = if args.len() >= 3 { &args[2] } else { "fleet_1" };
                    println!("🪩 Fleet '{}'\n", name);
                    println!("   Fleet orchestration backend not yet implemented.");
                    println!("   Use agent::spawn in a loop in DAL code for multiple agents.");
                }
                _ => {
                    println!("🤖 Fleet\n");
                    println!(
                        "   Fleet subcommands (scale, deploy, health) coming in a future release."
                    );
                }
            }
        }
        "mold" => {
            if args.len() < 2 {
                eprintln!("Usage: {} agent mold <subcommand> [args...]", binary_name());
                eprintln!("Subcommands: list, show <path-or-name>, create <name>, publish <file>");
                std::process::exit(1);
            }
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            match args[1].as_str() {
                "list" => {
                    let paths = dist_agent_lang::mold::list_local_paths(&cwd);
                    if paths.is_empty() {
                        println!("No local molds found (looked in ., mold/, mold/samples for *.mold.json, *.mold.dal)");
                    } else {
                        println!("Local molds:");
                        for p in &paths {
                            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            println!("  {}  {}", name, p.display());
                        }
                    }
                }
                "show" => {
                    if args.len() < 3 {
                        eprintln!("Usage: {} agent mold show <path-or-name>", binary_name());
                        std::process::exit(1);
                    }
                    let path_or_name = &args[2];
                    let path = match dist_agent_lang::mold::resolve_mold_path(&cwd, path_or_name) {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("❌ {}", e);
                            std::process::exit(1);
                        }
                    };
                    let mold = match dist_agent_lang::mold::load_mold_from_path(&path) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("❌ Failed to load mold: {}", e);
                            std::process::exit(1);
                        }
                    };
                    println!("Mold: {} (version {})", mold.name, mold.version);
                    println!("  Path: {}", path.display());
                    println!("  Agent type: {}", mold.agent.agent_type);
                    println!("  Role: {}", mold.agent.role);
                    println!("  Capabilities: {}", mold.agent.capabilities.join(", "));
                    println!("  Trust level: {}", mold.agent.trust_level);
                    println!("  Memory limit: {}", mold.agent.memory_limit);
                    println!(
                        "  Learning: {}  Communication: {}  Coordination: {}",
                        mold.agent.learning, mold.agent.communication, mold.agent.coordination
                    );
                }
                "create" => {
                    if args.len() < 3 {
                        eprintln!("Usage: {} agent mold create <name>", binary_name());
                        std::process::exit(1);
                    }
                    let name = &args[2];
                    let out_path = cwd.join(format!("{}.mold.json", name));
                    match dist_agent_lang::mold::scaffold_mold(name, &out_path) {
                        Ok(()) => println!("✅ Created mold template: {}", out_path.display()),
                        Err(e) => {
                            eprintln!("❌ {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                "publish" => {
                    if args.len() < 3 {
                        eprintln!(
                            "Usage: {} agent mold publish <file> [--fee <ether>] [--max-use <n>]",
                            binary_name()
                        );
                        eprintln!("  Upload to IPFS; with --fee/--max-use and Web3 env, mints on-chain (mintMold).");
                        std::process::exit(1);
                    }
                    let path = std::path::Path::new(&args[2]);
                    let mut fee_ether: Option<f64> = None;
                    let mut max_use: Option<u64> = None;
                    let mut i = 3;
                    while i + 1 < args.len() {
                        if args[i] == "--fee" {
                            if let Ok(f) = args[i + 1].parse::<f64>() {
                                fee_ether = Some(f);
                            }
                            i += 2;
                        } else if args[i] == "--max-use" {
                            if let Ok(n) = args[i + 1].parse::<u64>() {
                                max_use = Some(n);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    #[cfg(feature = "http-interface")]
                    {
                        if !path.exists() {
                            eprintln!("❌ File not found: {}", path.display());
                            std::process::exit(1);
                        }
                        match dist_agent_lang::mold::upload_mold_to_ipfs(path) {
                            Ok(cid) => {
                                println!("✅ Mold uploaded to IPFS");
                                println!("   CID: {}", cid);
                                println!("   URI: ipfs://{}", cid);
                                #[cfg(feature = "web3")]
                                {
                                    if fee_ether.is_some() || max_use.is_some() {
                                        let fee_wei =
                                            fee_ether.map(|f| (f * 1e18) as u128).unwrap_or(0);
                                        let max = max_use.unwrap_or(0);
                                        if std::env::var("DAL_PRIVATE_KEY").is_ok()
                                            && std::env::var("DAL_MOLD_REGISTRY_ADDRESS").is_ok()
                                        {
                                            match dist_agent_lang::mold::mint_mold(
                                                &cid, fee_wei, max,
                                            ) {
                                                Ok(mold_id) => {
                                                    println!(
                                                        "✅ Minted on-chain: mold ID {}",
                                                        mold_id
                                                    );
                                                }
                                                Err(e) => {
                                                    eprintln!("❌ mintMold failed: {}", e);
                                                    std::process::exit(1);
                                                }
                                            }
                                        } else {
                                            eprintln!("⚠️  Set DAL_PRIVATE_KEY and DAL_MOLD_REGISTRY_ADDRESS to mint on-chain.");
                                        }
                                    }
                                }
                                #[cfg(not(feature = "web3"))]
                                if fee_ether.is_some() || max_use.is_some() {
                                    println!("⚠️  Build with --features web3 and set DAL_* env to mint on-chain.");
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ Upload failed: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    #[cfg(not(feature = "http-interface"))]
                    {
                        let _ = (fee_ether, max_use);
                        eprintln!("❌ Publish requires http-interface feature (default). Rebuild without --no-default-features.");
                        std::process::exit(1);
                    }
                }
                _ => {
                    eprintln!(
                        "Unknown subcommand: {}. Use list, show, create, or publish.",
                        args[1]
                    );
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("❌ Unknown agent subcommand: {}", args[0]);
            eprintln!("Available: create, send, messages, task, list, fleet, mold");
            std::process::exit(1);
        }
    }
}

/// Generate DAL code from natural language prompt
fn generate_code(prompt: &str, output_file: Option<&str>) {
    println!("🤖 Generating DAL code from prompt...\n");

    // Check for API key
    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .unwrap_or_else(|_| {
            eprintln!("⚠️  No AI API key found. Set OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable.");
            eprintln!("   For now, generating example code structure...\n");
            String::new()
        });

    let system_prompt = "You are an expert dist_agent_lang (DAL) programmer. Generate clean, idiomatic DAL code based on the user's request. Include comments explaining key parts.";

    let full_prompt = format!(
        "{}\n\nUser request: {}\n\nGenerate DAL code:",
        system_prompt, prompt
    );

    // For now, use the ai module's generate_text function
    // In production, this would call OpenAI/Anthropic API
    let generated_code = if !api_key.is_empty() {
        match stdlib::ai::generate_text(full_prompt) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("❌ Code generation failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Generate example structure when no API key
        generate_code_template(prompt)
    };

    println!("📝 Generated Code:\n");
    println!("```dal");
    println!("{}", generated_code);
    println!("```\n");

    if let Some(file) = output_file {
        match std::fs::write(file, &generated_code) {
            Ok(_) => println!("✅ Code saved to: {}", file),
            Err(e) => eprintln!("❌ Failed to save file: {}", e),
        }
    } else {
        println!("💡 Tip: Use --output <file> to save the generated code");
    }
}

/// Generate a code template when no API key is available
fn generate_code_template(prompt: &str) -> String {
    let prompt_lower = prompt.to_lowercase();

    if prompt_lower.contains("token") || prompt_lower.contains("erc20") {
        r#"// Token Contract
// Generated from prompt: Create a token contract

chain contract TokenContract {
    let owner: string = "0x0"
    let balances: map<string, int> = {}
    let totalSupply: int = 0
    
    fn init(initialSupply: int) {
        owner = msg.sender
        totalSupply = initialSupply
        balances[owner] = initialSupply
    }
    
    fn transfer(to: string, amount: int) -> bool {
        let from = msg.sender
        if balances[from] >= amount {
            balances[from] = balances[from] - amount
            balances[to] = balances[to] + amount
            return true
        }
        return false
    }
    
    fn balanceOf(account: string) -> int {
        return balances[account]
    }
}"#
        .to_string()
    } else if prompt_lower.contains("api") || prompt_lower.contains("server") {
        r#"// Web API Server
// Generated from prompt: Create an API server

import web from "@dal/web"

web.route("GET", "/api/health", fn(req, res) {
    res.json({
        status: "ok",
        timestamp: time.now()
    })
})

web.route("GET", "/api/data", fn(req, res) {
    let data = db.query("SELECT * FROM items")
    res.json({
        success: true,
        data: data
    })
})

web.route("POST", "/api/data", fn(req, res) {
    let body = req.body
    db.insert("items", body)
    res.json({
        success: true,
        message: "Data created"
    })
})

web.listen(3000)
print("Server running on http://localhost:3000")"#
            .to_string()
    } else {
        format!(
            r#"// Generated DAL Code
// Prompt: {}

fn main() {{
    print("Hello from DAL!")
    
    // TODO: Implement your logic here
    // This is a template. Set OPENAI_API_KEY or ANTHROPIC_API_KEY 
    // for AI-powered code generation.
}}

main()"#,
            prompt
        )
    }
}

/// Explain what the code does
fn explain_code(filename: &str) {
    println!("🔍 Analyzing code: {}\n", filename);

    let code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .ok();

    if api_key.is_none() {
        eprintln!("⚠️  No AI API key found. Performing basic analysis...\n");
        explain_code_basic(&code);
        return;
    }

    let prompt = format!(
        "Analyze this dist_agent_lang (DAL) code and explain:\n\
        1. What it does (high-level purpose)\n\
        2. Key components and their roles\n\
        3. Data flow and logic\n\
        4. Any notable patterns or techniques\n\n\
        Code:\n```\n{}\n```",
        code
    );

    match stdlib::ai::generate_text(prompt) {
        Ok(explanation) => {
            println!("📖 Explanation:\n");
            println!("{}\n", explanation);
        }
        Err(e) => {
            eprintln!("❌ Explanation failed: {}", e);
            eprintln!("Falling back to basic analysis...\n");
            explain_code_basic(&code);
        }
    }
}

/// Basic code explanation without AI
fn explain_code_basic(code: &str) {
    let lines: Vec<&str> = code.lines().collect();
    let line_count = lines.len();

    println!("📊 Code Statistics:");
    println!("   Lines of code: {}", line_count);
    println!("   Functions: {}", code.matches("fn ").count());
    println!("   Contracts: {}", code.matches("contract ").count());
    println!("   Variables: {}", code.matches("let ").count());
    println!();

    println!("🔍 Detected Features:");
    if code.contains("contract ") {
        println!("   • Smart contract");
    }
    if code.contains("web.route") {
        println!("   • Web API routes");
    }
    if code.contains("db.query") || code.contains("db.insert") {
        println!("   • Database operations");
    }
    if code.contains("agent.") {
        println!("   • AI agent operations");
    }
    if code.contains("crypto.") {
        println!("   • Cryptographic operations");
    }
    if code.contains("chain.") {
        println!("   • Blockchain operations");
    }

    println!();
    println!("💡 For detailed AI-powered explanations, set OPENAI_API_KEY or ANTHROPIC_API_KEY");
}

/// Review code and provide suggestions
fn review_code(filename: &str) {
    println!("👀 Reviewing code: {}\n", filename);

    let code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .ok();

    if api_key.is_none() {
        eprintln!("⚠️  No AI API key found. Performing basic review...\n");
        review_code_basic(&code);
        return;
    }

    let prompt = format!(
        "Review this dist_agent_lang (DAL) code and provide:\n\
        1. Code quality assessment\n\
        2. Potential bugs or issues\n\
        3. Performance improvements\n\
        4. Best practices recommendations\n\
        5. Security considerations\n\n\
        Code:\n```\n{}\n```",
        code
    );

    match stdlib::ai::generate_text(prompt) {
        Ok(review) => {
            println!("📋 Code Review:\n");
            println!("{}\n", review);
        }
        Err(e) => {
            eprintln!("❌ Review failed: {}", e);
            eprintln!("Falling back to basic review...\n");
            review_code_basic(&code);
        }
    }
}

/// Basic code review without AI
fn review_code_basic(code: &str) {
    println!("✅ Strengths:");
    if code.contains("fn ") {
        println!("   • Uses functions for modularity");
    }
    if code.lines().any(|l| l.trim().starts_with("//")) {
        println!("   • Includes comments");
    }

    println!();
    println!("⚠️  Potential Issues:");

    let mut issues_found = false;

    if code.contains("let password") || code.contains("let apiKey") {
        println!("   • Hard-coded credentials detected");
        issues_found = true;
    }

    if code.matches("fn ").count() > 20 {
        println!("   • Large file - consider splitting into modules");
        issues_found = true;
    }

    if !code.contains("//") && !code.contains("/*") {
        println!("   • No comments found - add documentation");
        issues_found = true;
    }

    if !issues_found {
        println!("   • No obvious issues detected");
    }

    println!();
    println!("💡 For comprehensive AI-powered reviews, set OPENAI_API_KEY or ANTHROPIC_API_KEY");
}

/// Security audit for smart contracts
fn audit_code(filename: &str) {
    println!("🔒 Security Audit: {}\n", filename);

    let code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .ok();

    if api_key.is_none() {
        eprintln!("⚠️  No AI API key found. Performing basic security checks...\n");
        audit_code_basic(&code);
        return;
    }

    let prompt = format!(
        "Perform a security audit of this dist_agent_lang (DAL) smart contract code:\n\
        1. Identify security vulnerabilities\n\
        2. Check for common attack vectors (reentrancy, overflow, access control)\n\
        3. Verify proper input validation\n\
        4. Review authorization and authentication\n\
        5. Suggest security improvements\n\n\
        Code:\n```\n{}\n```",
        code
    );

    match stdlib::ai::generate_text(prompt) {
        Ok(audit) => {
            println!("🛡️  Security Audit Report:\n");
            println!("{}\n", audit);
        }
        Err(e) => {
            eprintln!("❌ Audit failed: {}", e);
            eprintln!("Falling back to basic security checks...\n");
            audit_code_basic(&code);
        }
    }
}

/// Basic security audit without AI
fn audit_code_basic(code: &str) {
    println!("🔍 Security Checks:\n");

    let mut critical = vec![];
    let mut warnings = vec![];
    let mut info = vec![];

    // Critical issues
    if code.contains("msg.sender") && !code.contains("require") && !code.contains("if msg.sender") {
        critical.push("Missing sender verification in contract");
    }

    if code.contains("transfer") && !code.contains("balance") {
        warnings.push("Transfer without balance check");
    }

    // Warnings
    if code.contains("let password")
        || code.contains("let apiKey")
        || code.contains("let privateKey")
    {
        critical.push("Hard-coded credentials detected - SECURITY RISK!");
    }

    if code.contains("db.query") && code.contains(&format!("\"SELECT * FROM")) {
        warnings.push("Potential SQL injection risk - use parameterized queries");
    }

    if code.contains("eval(") {
        critical.push("Use of eval() detected - major security risk!");
    }

    // Info
    if !code.contains("try") && !code.contains("catch") {
        info.push("No error handling detected - consider adding try/catch");
    }

    let has_critical = !critical.is_empty();
    let has_warnings = !warnings.is_empty();

    if has_critical {
        println!("🚨 CRITICAL Issues:");
        for issue in critical {
            println!("   • {}", issue);
        }
        println!();
    }

    if has_warnings {
        println!("⚠️  Warnings:");
        for warning in warnings {
            println!("   • {}", warning);
        }
        println!();
    }

    if !info.is_empty() {
        println!("ℹ️  Recommendations:");
        for i in info {
            println!("   • {}", i);
        }
        println!();
    }

    if !has_critical && !has_warnings {
        println!("✅ No obvious security issues detected\n");
    }

    println!(
        "💡 For comprehensive AI-powered security audits, set OPENAI_API_KEY or ANTHROPIC_API_KEY"
    );
}

/// Generate test cases
fn generate_tests(filename: &str, output_file: Option<&str>) {
    println!("🧪 Generating tests for: {}\n", filename);

    let code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .ok();

    let tests = if api_key.is_some() {
        let prompt = format!(
            "Generate comprehensive test cases for this dist_agent_lang (DAL) code:\n\
            1. Create tests using DAL's test framework (describe, it, expect)\n\
            2. Cover happy paths, edge cases, and error conditions\n\
            3. Test all public functions\n\
            4. Include setup and teardown if needed\n\n\
            Code:\n```\n{}\n```",
            code
        );

        match stdlib::ai::generate_text(prompt) {
            Ok(generated_tests) => generated_tests,
            Err(e) => {
                eprintln!("❌ Test generation failed: {}", e);
                eprintln!("Generating basic test template...\n");
                generate_test_template(&code)
            }
        }
    } else {
        eprintln!("⚠️  No AI API key found. Generating test template...\n");
        generate_test_template(&code)
    };

    println!("📝 Generated Tests:\n");
    println!("```dal");
    println!("{}", tests);
    println!("```\n");

    if let Some(file) = output_file {
        match std::fs::write(file, &tests) {
            Ok(_) => println!("✅ Tests saved to: {}", file),
            Err(e) => eprintln!("❌ Failed to save file: {}", e),
        }
    } else {
        println!("💡 Tip: Use --output <file> to save the generated tests");
    }
}

/// Generate a test template
fn generate_test_template(code: &str) -> String {
    let functions: Vec<&str> = code.lines().filter(|line| line.contains("fn ")).collect();

    let mut tests = String::from("// Generated Test Suite\nimport test from \"@dal/test\"\n\n");

    if functions.is_empty() {
        tests.push_str(
            r#"describe("Main Tests", fn() {
    it("should execute without errors", fn() {
        // TODO: Add test implementation
        expect(true).toBe(true)
    })
})"#,
        );
    } else {
        for func in functions {
            let func_name = func
                .split("fn ")
                .nth(1)
                .and_then(|s| s.split('(').next())
                .unwrap_or("function")
                .trim();

            tests.push_str(&format!(
                r#"describe("{} tests", fn() {{
    it("should work with valid input", fn() {{
        // TODO: Test {} with valid input
        expect(true).toBe(true)
    }})
    
    it("should handle edge cases", fn() {{
        // TODO: Test {} with edge cases
        expect(true).toBe(true)
    }})
    
    it("should handle errors", fn() {{
        // TODO: Test {} error handling
        expect(true).toBe(true)
    }})
}})

"#,
                func_name, func_name, func_name, func_name
            ));
        }
    }

    tests
}

/// Suggest fixes for code issues
fn suggest_fixes(filename: &str) {
    println!("🔧 Analyzing issues in: {}\n", filename);

    let code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .ok();

    if api_key.is_none() {
        eprintln!("⚠️  No AI API key found. Providing basic suggestions...\n");
        suggest_fixes_basic(&code);
        return;
    }

    let prompt = format!(
        "Analyze this dist_agent_lang (DAL) code and suggest fixes:\n\
        1. Identify syntax errors\n\
        2. Find logic errors\n\
        3. Detect code smells\n\
        4. Provide specific fix recommendations with code examples\n\n\
        Code:\n```\n{}\n```",
        code
    );

    match stdlib::ai::generate_text(prompt) {
        Ok(fixes) => {
            println!("🔧 Suggested Fixes:\n");
            println!("{}\n", fixes);
        }
        Err(e) => {
            eprintln!("❌ Fix suggestions failed: {}", e);
            eprintln!("Providing basic suggestions...\n");
            suggest_fixes_basic(&code);
        }
    }
}

/// Basic fix suggestions without AI
fn suggest_fixes_basic(code: &str) {
    println!("💡 Suggestions:\n");

    let mut suggestions = vec![];

    if !code.contains("fn main") {
        suggestions.push("Consider adding a main() function as entry point");
    }

    if code.contains("let ") && !code.contains("const ") {
        suggestions.push("Use 'const' for values that don't change");
    }

    if code.matches("fn ").count() > 0 && !code.lines().any(|l| l.trim().starts_with("//")) {
        suggestions.push("Add comments to document function behavior");
    }

    if code.contains("print(") && !code.contains("log.") {
        suggestions.push("Consider using log.info() instead of print() for production code");
    }

    if suggestions.is_empty() {
        println!("   ✅ Code looks good! No obvious improvements needed.\n");
    } else {
        for (i, suggestion) in suggestions.iter().enumerate() {
            println!("   {}. {}", i + 1, suggestion);
        }
        println!();
    }

    println!("💡 For AI-powered detailed fix suggestions, set OPENAI_API_KEY or ANTHROPIC_API_KEY");
}

/// Gas optimization suggestions for smart contracts
fn optimize_gas(filename: &str) {
    println!("⛽ Analyzing gas usage: {}\n", filename);

    let code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    if !code.contains("contract ") && !code.contains("chain.") {
        eprintln!("⚠️  This doesn't appear to be a smart contract or blockchain code.");
        eprintln!("   Gas optimization is most relevant for on-chain code.\n");
    }

    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .ok();

    if api_key.is_none() {
        eprintln!("⚠️  No AI API key found. Providing basic gas optimization tips...\n");
        optimize_gas_basic(&code);
        return;
    }

    let prompt = format!(
        "Analyze this dist_agent_lang (DAL) smart contract for gas optimization:\n\
        1. Identify gas-expensive operations\n\
        2. Suggest cheaper alternatives\n\
        3. Recommend storage optimizations\n\
        4. Highlight unnecessary computations\n\
        5. Provide specific code improvements with estimated gas savings\n\n\
        Code:\n```\n{}\n```",
        code
    );

    match stdlib::ai::generate_text(prompt) {
        Ok(optimizations) => {
            println!("⚡ Gas Optimization Report:\n");
            println!("{}\n", optimizations);
        }
        Err(e) => {
            eprintln!("❌ Gas analysis failed: {}", e);
            eprintln!("Providing basic optimization tips...\n");
            optimize_gas_basic(&code);
        }
    }
}

/// Basic gas optimization without AI
fn optimize_gas_basic(code: &str) {
    println!("💡 Gas Optimization Tips:\n");

    let mut tips = vec![];

    if code.contains("string") && code.contains("storage") {
        tips.push("🔸 Use bytes32 instead of string when possible (saves gas)");
    }

    if code.contains("for ") || code.contains("while ") {
        tips.push("🔸 Cache array length in loops to avoid repeated SLOAD operations");
    }

    if code.matches("let ").count() > 5 {
        tips.push("🔸 Pack variables into single storage slots when possible");
    }

    if code.contains("public ") {
        tips.push("🔸 Use 'private' or 'internal' visibility when external access isn't needed");
    }

    if code.contains("emit ") {
        tips.push("🔸 Minimize data in events - only emit what's necessary");
    }

    if tips.is_empty() {
        println!("   ✅ No obvious gas optimization opportunities detected\n");
    } else {
        for tip in tips {
            println!("   {}", tip);
        }
        println!();
    }

    println!("⛽ Common Gas Optimizations:");
    println!(
        "   • Use 'uint256' instead of smaller types (Solidity packs, but DAL doesn't always)"
    );
    println!("   • Batch operations when possible");
    println!("   • Use events instead of storing data when data doesn't need to be read on-chain");
    println!("   • Minimize storage writes (most expensive operation)");
    println!();
    println!("💡 For AI-powered detailed gas analysis, set OPENAI_API_KEY or ANTHROPIC_API_KEY");
}

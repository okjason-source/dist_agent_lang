use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dist_agent_lang::lexer::Lexer;
use dist_agent_lang::parser::Parser;
use dist_agent_lang::runtime::Runtime;
use std::time::Instant;

// Benchmark lexer performance
fn bench_lexer(c: &mut Criterion) {
    let test_code = r#"
@trust("hybrid")
@secure
@limit(1000)
fn benchmark_function() {
    let x = 42;
    let message = "Hello from dist_agent_lang!";
    let flag = true;
    
    if flag {
        let result = x * 2;
        return result;
    }
    
    return 0;
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

    c.bench_function("lexer_tokenization", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(test_code));
            lexer.tokenize().unwrap_or_else(|_| vec![])
        })
    });
}

// Benchmark parser performance
fn bench_parser(c: &mut Criterion) {
    let test_code = r#"
let x = 42;
let message = "Hello from dist_agent_lang!";
let flag = true;

@trust("hybrid")
@secure
fn test_function() {
    if flag {
        let result = x * 2;
        return result;
    }
    return 0;
}

let price = oracle::fetch("https://api.example.com/oracle/price", oracle::create_query("btc_price"));
let ai_response = service::ai("What is blockchain?", service::create_ai_service("gpt-4"));
"#;

    c.bench_function("parser_ast_generation", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(test_code));
            let tokens = lexer.tokenize().unwrap_or_else(|_| vec![]);
            if !tokens.is_empty() {
                let mut parser = Parser::new(tokens);
                parser
                    .parse()
                    .unwrap_or_else(|_| dist_agent_lang::parser::ast::Program {
                        statements: vec![],
                        statement_spans: vec![],
                    })
            } else {
                dist_agent_lang::parser::ast::Program {
                    statements: vec![],
                    statement_spans: vec![],
                }
            }
        })
    });
}

// Benchmark runtime performance
fn bench_runtime(c: &mut Criterion) {
    c.bench_function("runtime_variable_operations", |b| {
        b.iter(|| {
            let mut runtime = Runtime::new();

            // Set variables
            runtime.set_variable(
                "x".to_string(),
                dist_agent_lang::runtime::values::Value::Int(42),
            );
            runtime.set_variable(
                "message".to_string(),
                dist_agent_lang::runtime::values::Value::String("Hello".to_string()),
            );
            runtime.set_variable(
                "flag".to_string(),
                dist_agent_lang::runtime::values::Value::Bool(true),
            );

            // Get variables
            let _x = runtime
                .get_variable("x")
                .unwrap_or(dist_agent_lang::runtime::values::Value::Null);
            let _message = runtime
                .get_variable("message")
                .unwrap_or(dist_agent_lang::runtime::values::Value::Null);
            let _flag = runtime
                .get_variable("flag")
                .unwrap_or(dist_agent_lang::runtime::values::Value::Null);
        })
    });

    c.bench_function("runtime_function_calls", |b| {
        b.iter(|| {
            let mut runtime = Runtime::with_capacities(128, 32, 16);

            // Call built-in functions
            let _result1 = runtime.call_function(
                "add",
                &[
                    dist_agent_lang::runtime::values::Value::Int(10),
                    dist_agent_lang::runtime::values::Value::Int(32),
                ],
            );

            let _result2 = runtime.call_function(
                "print",
                &[dist_agent_lang::runtime::values::Value::String(
                    "test".to_string(),
                )],
            );
        })
    });
}

// Benchmark standard library performance
fn bench_stdlib(c: &mut Criterion) {
    c.bench_function("stdlib_chain_operations", |b| {
        b.iter(|| {
            use std::collections::HashMap;

            let metadata = {
                let mut map = HashMap::new();
                map.insert("description".to_string(), "A test NFT".to_string());
                map.insert("image".to_string(), "ipfs://QmTest...".to_string());
                map
            };

            let _asset_id =
                dist_agent_lang::stdlib::chain::mint("TestNFT".to_string(), metadata.clone());
            let _asset_info = dist_agent_lang::stdlib::chain::get(_asset_id);
            let _update_success = dist_agent_lang::stdlib::chain::update(_asset_id, {
                let mut updates = HashMap::new();
                updates.insert("description".to_string(), "Updated test NFT".to_string());
                updates
            });
        })
    });

    c.bench_function("stdlib_crypto_operations", |b| {
        b.iter(|| {
            let _hash_sha256 = dist_agent_lang::stdlib::crypto::hash(
                "Hello, World!",
                dist_agent_lang::stdlib::crypto::HashAlgorithm::SHA256,
            );

            let _hash_sha512 = dist_agent_lang::stdlib::crypto::hash(
                "Hello, World!",
                dist_agent_lang::stdlib::crypto::HashAlgorithm::SHA512,
            );

            let _keypair = dist_agent_lang::stdlib::crypto::generate_keypair(
                dist_agent_lang::stdlib::crypto::SignatureAlgorithm::RSA,
            );

            let _signature = dist_agent_lang::stdlib::crypto::sign(
                "Hello, World!",
                &_keypair["private_key"],
                dist_agent_lang::stdlib::crypto::SignatureAlgorithm::RSA,
            );
        })
    });

    c.bench_function("stdlib_auth_operations", |b| {
        b.iter(|| {
            let _session = dist_agent_lang::stdlib::auth::session(
                "user123".to_string(),
                vec!["admin".to_string()],
            );

            let _is_valid = dist_agent_lang::stdlib::auth::is_valid_session(&_session);
            let _has_admin = dist_agent_lang::stdlib::auth::has_role(&_session, "admin");
        })
    });
}

// Benchmark memory usage
fn bench_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let mut runtime = Runtime::with_capacities(1024, 128, 64);

            // Allocate many variables
            for i in 0..100 {
                runtime.set_variable(
                    format!("var_{}", i),
                    dist_agent_lang::runtime::values::Value::Int(i),
                );
            }

            // Access variables
            for i in 0..100 {
                let _value = runtime.get_variable(&format!("var_{}", i)).unwrap();
            }
        })
    });
}

// Benchmark concurrent operations (simulated)
fn bench_concurrent_operations(c: &mut Criterion) {
    c.bench_function("concurrent_variable_access", |b| {
        b.iter(|| {
            let mut runtime = Runtime::with_capacities(512, 64, 32);

            // Simulate concurrent-like operations
            let start = Instant::now();

            // Set variables rapidly
            for i in 0..50 {
                runtime.set_variable(
                    format!("concurrent_var_{}", i),
                    dist_agent_lang::runtime::values::Value::Int(i * 2),
                );
            }

            // Read variables rapidly
            for i in 0..50 {
                let _value = runtime
                    .get_variable(&format!("concurrent_var_{}", i))
                    .unwrap();
            }

            let duration = start.elapsed();
            black_box(duration);
        })
    });
}

// Custom benchmark group for language features
criterion_group!(
    name = language_benchmarks;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(10))
        .warm_up_time(std::time::Duration::from_secs(5));
    targets =
        bench_lexer,
        bench_parser,
        bench_runtime,
        bench_stdlib,
        bench_memory_usage,
        bench_concurrent_operations
);

criterion_main!(language_benchmarks);

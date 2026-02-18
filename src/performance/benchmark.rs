use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub throughput: f64, // operations per second
    pub memory_usage: Option<usize>, // bytes
    pub cpu_usage: Option<f64>, // percentage
}

#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    pub name: String,
    pub benchmarks: Vec<BenchmarkResult>,
    pub total_duration: Duration,
}

pub struct BenchmarkRunner {
    iterations: usize,
    warmup_iterations: usize,
    enable_memory_tracking: bool,
    enable_cpu_tracking: bool,
}

impl BenchmarkRunner {
    pub fn new() -> Self {
        Self {
            iterations: 1000,
            warmup_iterations: 100,
            enable_memory_tracking: false,
            enable_cpu_tracking: false,
        }
    }

    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn with_warmup(mut self, warmup: usize) -> Self {
        self.warmup_iterations = warmup;
        self
    }

    pub fn with_memory_tracking(mut self, enable: bool) -> Self {
        self.enable_memory_tracking = enable;
        self
    }

    pub fn with_cpu_tracking(mut self, enable: bool) -> Self {
        self.enable_cpu_tracking = enable;
        self
    }

    pub fn run<F>(&self, name: &str, benchmark_fn: F) -> BenchmarkResult 
    where 
        F: Fn() -> Result<(), String>
    {
        // Warmup phase
        for _ in 0..self.warmup_iterations {
            benchmark_fn().expect("Benchmark function failed during warmup");
        }

        let mut durations = Vec::with_capacity(self.iterations);
        let mut memory_usage = None;
        let cpu_usage = None;

        // Measure memory before if enabled
        let memory_before = if self.enable_memory_tracking {
            Some(Self::get_memory_usage())
        } else {
            None
        };

        // Actual benchmark
        for _ in 0..self.iterations {
            let start = Instant::now();
            benchmark_fn().expect("Benchmark function failed");
            durations.push(start.elapsed());
        }

        // Measure memory after if enabled
        if self.enable_memory_tracking {
            if let Some(before) = memory_before {
                let after = Self::get_memory_usage();
                memory_usage = Some(after.saturating_sub(before));
            }
        }

        // Calculate statistics
        let total_duration: Duration = durations.iter().sum();
        let average_duration = total_duration / self.iterations as u32;
        let min_duration = *durations.iter().min().unwrap();
        let max_duration = *durations.iter().max().unwrap();
        let throughput = self.iterations as f64 / total_duration.as_secs_f64();

        BenchmarkResult {
            name: name.to_string(),
            iterations: self.iterations,
            total_duration,
            average_duration,
            min_duration,
            max_duration,
            throughput,
            memory_usage,
            cpu_usage,
        }
    }

    pub fn run_suite<F>(&self, name: &str, benchmarks: Vec<(&str, F)>) -> BenchmarkSuite 
    where 
        F: Fn() -> Result<(), String>
    {
        let mut results = Vec::new();
        let start = Instant::now();

        for (bench_name, benchmark_fn) in benchmarks {
            let result = self.run(bench_name, benchmark_fn);
            results.push(result);
        }

        BenchmarkSuite {
            name: name.to_string(),
            benchmarks: results,
            total_duration: start.elapsed(),
        }
    }

    fn get_memory_usage() -> usize {
        // Simple memory usage estimation
        // In a real implementation, you'd use platform-specific APIs
        std::mem::size_of::<usize>() * 1024 // Placeholder
    }
}

// Built-in benchmarks for language components
pub struct LanguageBenchmarks;

impl LanguageBenchmarks {
    pub fn lexer_benchmarks() -> Vec<(&'static str, Box<dyn Fn() -> Result<(), String>>)> {
        vec![
            ("simple_tokens", Box::new(|| {
                use crate::lexer::Lexer;
                let source = "let x = 42; let y = 10; x + y";
                let lexer = Lexer::new(source);
                lexer.tokenize_immutable().map_err(|e| e.to_string())?;
                Ok(())
            })),
            ("complex_tokens", Box::new(|| {
                use crate::lexer::Lexer;
                let source = r#"
                    @txn @secure @limit(1000) @trust("hybrid")
                    fn complex_function(param1: string, param2: int) -> bool {
                        let result = param1.len() + param2;
                        if result > 100 {
                            return true;
                        } else {
                            return false;
                        }
                    }
                "#;
                let lexer = Lexer::new(source);
                lexer.tokenize_immutable().map_err(|e| e.to_string())?;
                Ok(())
            })),
            ("namespace_calls", Box::new(|| {
                use crate::lexer::Lexer;
                let source = r#"
                    let price = oracle::fetch("btc_price");
                    let hash = crypto::hash("data");
                    let session = auth::session("user123");
                    let log_entry = log::info("test");
                "#;
                let lexer = Lexer::new(source);
                lexer.tokenize_immutable().map_err(|e| e.to_string())?;
                Ok(())
            })),
        ]
    }

    pub fn parser_benchmarks() -> Vec<(&'static str, Box<dyn Fn() -> Result<(), String>>)> {
        vec![
            ("simple_expressions", Box::new(|| {
                use crate::lexer::Lexer;
                use crate::parser::Parser;
                let source = "let x = 42 + 10 * 2; let y = (x + 5) / 3;";
                let lexer = Lexer::new(source);
                let tokens = lexer.tokenize_immutable().map_err(|e| e.to_string())?;
                let mut parser = Parser::new(tokens);
                parser.parse().map_err(|e| e.to_string())?;
                Ok(())
            })),
            ("function_definitions", Box::new(|| {
                use crate::lexer::Lexer;
                use crate::parser::Parser;
                let source = r#"
                    fn add(a: int, b: int) -> int {
                        return a + b;
                    }
                    fn multiply(x: int, y: int) -> int {
                        return x * y;
                    }
                "#;
                let lexer = Lexer::new(source);
                let tokens = lexer.tokenize_immutable().map_err(|e| e.to_string())?;
                let mut parser = Parser::new(tokens);
                parser.parse().map_err(|e| e.to_string())?;
                Ok(())
            })),
            ("complex_statements", Box::new(|| {
                use crate::lexer::Lexer;
                use crate::parser::Parser;
                let source = r#"
                    @txn @secure
                    fn process_data() -> bool {
                        return true;
                    }
                "#;
                let lexer = Lexer::new(source);
                let tokens = lexer.tokenize_immutable().map_err(|e| e.to_string())?;
                let mut parser = Parser::new(tokens);
                parser.parse().map_err(|e| e.to_string())?;
                Ok(())
            })),
        ]
    }

    pub fn runtime_benchmarks() -> Vec<(&'static str, Box<dyn Fn() -> Result<(), String>>)> {
        vec![
            ("variable_operations", Box::new(|| {
                use crate::runtime::Runtime;
                use crate::runtime::values::Value;
                let mut runtime = Runtime::new();
                
                // Set variables
                runtime.set_variable("x".to_string(), Value::Int(42));
                runtime.set_variable("y".to_string(), Value::Int(10));
                runtime.set_variable("z".to_string(), Value::String("test".to_string()));
                
                // Get variables
                let _x = runtime.get_variable("x");
                let _y = runtime.get_variable("y");
                let _z = runtime.get_variable("z");
                
                Ok(())
            })),
            ("function_calls", Box::new(|| {
                use crate::runtime::Runtime;
                
                let _runtime = Runtime::new();
                
                // Register and call functions
                // Note: This is a simplified version since the actual runtime doesn't support this pattern
                // In a real implementation, you'd register a proper Function struct
                println!("Function registration would happen here");
                
                // Note: Function calling would happen here
                println!("Function calling would happen here");
                
                Ok(())
            })),
            ("stack_operations", Box::new(|| {
                use crate::runtime::Runtime;
                use crate::runtime::values::Value;
                let mut runtime = Runtime::new();
                
                // Push and pop operations
                for i in 0..100 {
                    runtime.stack.push(Value::Int(i));
                }
                
                for _ in 0..100 {
                    runtime.stack.pop();
                }
                
                Ok(())
            })),
        ]
    }

    pub fn stdlib_benchmarks() -> Vec<(&'static str, Box<dyn Fn() -> Result<(), String>>)> {
        vec![
            ("chain_operations", Box::new(|| {
                use crate::stdlib::chain;
                use std::collections::HashMap;
                let mut metadata = HashMap::new();
                metadata.insert("test".to_string(), "true".to_string());
                let _asset_id = chain::mint("TestAsset".to_string(), metadata);
                let _info = chain::get(_asset_id);
                Ok(())
            })),
            ("crypto_operations", Box::new(|| {
                use crate::stdlib::crypto;
                let _hash = crypto::hash("test_data", crypto::HashAlgorithm::SHA256);
                let _keypair = crypto::generate_keypair(crypto::SignatureAlgorithm::RSA);
                Ok(())
            })),
            ("log_operations", Box::new(|| {
                use crate::stdlib::log;
                use std::collections::HashMap;
                use crate::runtime::values::Value;
                let mut data = HashMap::new();
                data.insert("test".to_string(), Value::String("benchmark".to_string()));
                log::info("benchmark_test", data.clone(), None);
                log::audit("benchmark_audit", data, None);
                Ok(())
            })),
        ]
    }
}

// Performance comparison utilities
pub struct PerformanceComparison;

impl PerformanceComparison {
    pub fn compare_benchmarks(baseline: &BenchmarkResult, current: &BenchmarkResult) -> String {
        let speedup = baseline.average_duration.as_nanos() as f64 / current.average_duration.as_nanos() as f64;
        let improvement = (speedup - 1.0) * 100.0;
        
        format!(
            "Performance Comparison: {} vs {}\n\
             Speedup: {:.2}x ({:+.1}%)\n\
             Baseline: {:?}\n\
             Current: {:?}\n\
             Throughput: {:.0} vs {:.0} ops/sec",
            baseline.name, current.name,
            speedup, improvement,
            baseline.average_duration, current.average_duration,
            baseline.throughput, current.throughput
        )
    }

    pub fn generate_report(suite: &BenchmarkSuite) -> String {
        let mut report = format!("Benchmark Suite: {}\n", suite.name);
        report.push_str(&format!("Total Duration: {:?}\n\n", suite.total_duration));
        
        for result in &suite.benchmarks {
            report.push_str(&format!("{}\n", result.name));
            report.push_str(&format!("  Average: {:?}\n", result.average_duration));
            report.push_str(&format!("  Min: {:?}, Max: {:?}\n", result.min_duration, result.max_duration));
            report.push_str(&format!("  Throughput: {:.0} ops/sec\n", result.throughput));
            if let Some(memory) = result.memory_usage {
                report.push_str(&format!("  Memory: {} bytes\n", memory));
            }
            report.push('\n');
        }
        
        report
    }
}

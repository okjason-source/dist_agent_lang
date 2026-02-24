//! Mocking system for dist_agent_lang testing framework
//!
//! This module provides a comprehensive mocking system that allows developers to:
//! - Mock function calls (both namespaced and global)
//! - Validate function arguments
//! - Track call counts and history
//! - Execute side effects (set variables, call functions, log, throw errors, delay)
//! - Verify mock expectations
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use dist_agent_lang::testing::{MockBuilder, MockRegistry};
//! use dist_agent_lang::runtime::values::Value;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a mock using the builder pattern
//!     let mock = MockBuilder::new("mint")
//!         .in_namespace("chain")
//!         .returns(Value::Int(12345))
//!         .expects_calls(1)
//!         .validates_args(|args| {
//!             if args.len() != 2 {
//!                 Err("Expected 2 arguments".to_string())
//!             } else {
//!                 Ok(())
//!             }
//!         })
//!         .logs("Mock chain::mint called")
//!         .build();
//!
//!     // Register the mock
//!     let mut registry = MockRegistry::new();
//!     registry.register(mock);
//!
//!     // Call the mock
//!     let result = registry.call_mock("mint", Some("chain"), &[
//!         Value::String("0x123".to_string()),
//!         Value::Int(100)
//!     ])?;
//!
//!     // Verify expectations
//!     registry.verify_all()?;
//! #     Ok(())
//! # }
//! ```
//!
//! # Integration with Runtime
//!
//! Runtime integration is complete. Mocks registered via `MockRuntime` or `Runtime::set_mock_registry()`
//! will automatically intercept function calls during execution.

use crate::runtime::values::Value;
use crate::runtime::Runtime;
use std::collections::HashMap;

/// Mock function definition
pub struct MockFunction {
    pub name: String,
    pub namespace: Option<String>,
    pub return_value: Option<Value>,
    pub side_effects: Vec<MockSideEffect>,
    pub call_count: usize,
    pub expected_calls: Option<usize>,
    /// Validator function for argument validation (not Clone-safe, so stored as trait object)
    /// Note: Cloning will lose the validator
    pub arguments_validator: Option<Box<dyn Fn(&[Value]) -> Result<(), String> + Send + Sync>>,
    /// Captured arguments from all calls (for inspection)
    pub call_history: Vec<Vec<Value>>,
}

impl Clone for MockFunction {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            return_value: self.return_value.clone(),
            side_effects: self.side_effects.clone(),
            call_count: self.call_count,
            expected_calls: self.expected_calls,
            arguments_validator: None, // Validator cannot be cloned
            call_history: self.call_history.clone(),
        }
    }
}

impl MockFunction {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: None,
            return_value: None,
            side_effects: Vec::new(),
            call_count: 0,
            expected_calls: None,
            arguments_validator: None,
            call_history: Vec::new(),
        }
    }

    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    pub fn returns(mut self, value: Value) -> Self {
        self.return_value = Some(value);
        self
    }

    pub fn with_side_effect(mut self, side_effect: MockSideEffect) -> Self {
        self.side_effects.push(side_effect);
        self
    }

    pub fn expects_calls(mut self, count: usize) -> Self {
        self.expected_calls = Some(count);
        self
    }

    pub fn validates_arguments<F>(mut self, validator: F) -> Self
    where
        F: Fn(&[Value]) -> Result<(), String> + Send + Sync + 'static,
    {
        self.arguments_validator = Some(Box::new(validator));
        self
    }

    /// Get the last call's arguments
    pub fn last_call_args(&self) -> Option<&[Value]> {
        self.call_history.last().map(|v| v.as_slice())
    }

    /// Get all call arguments
    pub fn all_call_args(&self) -> &[Vec<Value>] {
        &self.call_history
    }

    pub fn call(&mut self, args: &[Value]) -> Result<Value, String> {
        // Validate arguments if validator is set
        if let Some(ref validator) = self.arguments_validator {
            validator(args).map_err(|e| format!("Argument validation failed: {}", e))?;
        }

        // Record call
        self.call_count += 1;
        self.call_history.push(args.to_vec());

        // Execute side effects (may mutate runtime state)
        for side_effect in &self.side_effects {
            side_effect.execute(args)?;
        }

        // Return mock value
        Ok(self.return_value.clone().unwrap_or(Value::Null))
    }

    pub fn verify(&self) -> Result<(), String> {
        if let Some(expected) = self.expected_calls {
            if self.call_count != expected {
                return Err(format!(
                    "Mock function '{}' was called {} times, expected {} times",
                    self.name, self.call_count, expected
                ));
            }
        }
        Ok(())
    }
}

/// Side effects that can be triggered by mock functions
#[derive(Debug, Clone)]
pub enum MockSideEffect {
    /// Set a variable in the runtime scope (requires runtime reference)
    SetVariable(String, Value),
    /// Call another function in the runtime (requires runtime reference)
    CallFunction(String, Vec<Value>),
    /// Log a message to stdout
    LogMessage(String),
    /// Throw an error (stops execution)
    ThrowError(String),
    /// Delay execution (useful for testing timeouts/async behavior)
    Delay(std::time::Duration),
}

impl MockSideEffect {
    /// Execute the side effect. For SetVariable and CallFunction, a runtime reference
    /// must be provided via execute_with_runtime() instead.
    pub fn execute(&self, _args: &[Value]) -> Result<(), String> {
        match self {
            MockSideEffect::SetVariable(var_name, _) => {
                // Note: This requires runtime access - use execute_with_runtime() instead
                eprintln!("[MOCK WARNING] SetVariable side effect for '{}' requires runtime access. Use execute_with_runtime() or MockRuntime.", var_name);
                Ok(())
            }
            MockSideEffect::CallFunction(func_name, _) => {
                // Note: This requires runtime access - use execute_with_runtime() instead
                eprintln!("[MOCK WARNING] CallFunction side effect for '{}' requires runtime access. Use execute_with_runtime() or MockRuntime.", func_name);
                Ok(())
            }
            MockSideEffect::LogMessage(message) => {
                println!("[MOCK] {}", message);
                Ok(())
            }
            MockSideEffect::ThrowError(error) => Err(error.clone()),
            MockSideEffect::Delay(duration) => {
                std::thread::sleep(*duration);
                Ok(())
            }
        }
    }

    /// Execute side effect with runtime access (for SetVariable and CallFunction)
    pub fn execute_with_runtime(
        &self,
        args: &[Value],
        runtime: &mut Runtime,
    ) -> Result<(), String> {
        match self {
            MockSideEffect::SetVariable(var_name, value) => {
                runtime.scope.set(var_name.clone(), value.clone());
                Ok(())
            }
            MockSideEffect::CallFunction(func_name, call_args) => {
                // Try to call the function - if it fails, return error
                runtime
                    .call_function(func_name, call_args)
                    .map_err(|e| format!("Mock side effect CallFunction failed: {}", e))?;
                Ok(())
            }
            _ => self.execute(args),
        }
    }
}

/// Mock registry for managing multiple mocks
pub struct MockRegistry {
    pub mocks: HashMap<String, MockFunction>,
    pub enabled: bool,
}

impl MockRegistry {
    pub fn new() -> Self {
        Self {
            mocks: HashMap::new(),
            enabled: true,
        }
    }

    pub fn register(&mut self, mock: MockFunction) {
        let key = self.get_mock_key(&mock.name, mock.namespace.as_deref());
        self.mocks.insert(key, mock);
    }

    pub fn get_mock(&mut self, name: &str, namespace: Option<&str>) -> Option<&mut MockFunction> {
        let key = self.get_mock_key(name, namespace);
        self.mocks.get_mut(&key)
    }

    pub fn call_mock(
        &mut self,
        name: &str,
        namespace: Option<&str>,
        args: &[Value],
    ) -> Result<Value, String> {
        if !self.enabled {
            return Err("Mock registry is disabled".to_string());
        }

        if let Some(mock) = self.get_mock(name, namespace) {
            mock.call(args)
        } else {
            Err(format!(
                "No mock found for '{}{}'",
                namespace.map(|ns| format!("{}::", ns)).unwrap_or_default(),
                name
            ))
        }
    }

    /// Call mock with runtime access (for side effects that need runtime)
    pub fn call_mock_with_runtime(
        &mut self,
        name: &str,
        namespace: Option<&str>,
        args: &[Value],
        runtime: &mut Runtime,
    ) -> Result<Value, String> {
        if !self.enabled {
            return Err("Mock registry is disabled".to_string());
        }

        let key = self.get_mock_key(name, namespace);
        if let Some(mock) = self.mocks.get_mut(&key) {
            // Validate arguments
            if let Some(ref validator) = mock.arguments_validator {
                validator(args).map_err(|e| format!("Argument validation failed: {}", e))?;
            }

            // Record call
            mock.call_count += 1;
            mock.call_history.push(args.to_vec());

            // Execute side effects with runtime access
            for side_effect in &mock.side_effects {
                side_effect.execute_with_runtime(args, runtime)?;
            }

            // Return mock value
            Ok(mock.return_value.clone().unwrap_or(Value::Null))
        } else {
            Err(format!(
                "No mock found for '{}{}'",
                namespace.map(|ns| format!("{}::", ns)).unwrap_or_default(),
                name
            ))
        }
    }

    pub fn verify_all(&self) -> Result<(), String> {
        for (key, mock) in &self.mocks {
            mock.verify()
                .map_err(|e| format!("Mock '{}': {}", key, e))?;
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.mocks.clear();
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn get_mock_key(&self, name: &str, namespace: Option<&str>) -> String {
        match namespace {
            Some(ns) => format!("{}::{}", ns, name),
            None => name.to_string(),
        }
    }

    /// Check if a mock exists for the given function name and namespace
    pub fn has_mock(&self, name: &str, namespace: Option<&str>) -> bool {
        let key = self.get_mock_key(name, namespace);
        self.mocks.contains_key(&key)
    }

    /// Get call count for a specific mock
    pub fn get_call_count(&self, name: &str, namespace: Option<&str>) -> Option<usize> {
        let key = self.get_mock_key(name, namespace);
        self.mocks.get(&key).map(|m| m.call_count)
    }

    /// Reset call counts for all mocks (useful for test teardown)
    pub fn reset_call_counts(&mut self) {
        for mock in self.mocks.values_mut() {
            mock.call_count = 0;
            mock.call_history.clear();
        }
    }
}

/// Mock builder for fluent API
pub struct MockBuilder {
    name: String,
    namespace: Option<String>,
    return_value: Option<Value>,
    side_effects: Vec<MockSideEffect>,
    expected_calls: Option<usize>,
    arguments_validator: Option<Box<dyn Fn(&[Value]) -> Result<(), String> + Send + Sync>>,
}

impl MockBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: None,
            return_value: None,
            side_effects: Vec::new(),
            expected_calls: None,
            arguments_validator: None,
        }
    }

    pub fn in_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    pub fn returns(mut self, value: Value) -> Self {
        self.return_value = Some(value);
        self
    }

    pub fn logs(mut self, message: &str) -> Self {
        self.side_effects
            .push(MockSideEffect::LogMessage(message.to_string()));
        self
    }

    pub fn throws(mut self, error: &str) -> Self {
        self.side_effects
            .push(MockSideEffect::ThrowError(error.to_string()));
        self
    }

    pub fn delays(mut self, duration: std::time::Duration) -> Self {
        self.side_effects.push(MockSideEffect::Delay(duration));
        self
    }

    pub fn expects_calls(mut self, count: usize) -> Self {
        self.expected_calls = Some(count);
        self
    }

    pub fn validates_args<F>(mut self, validator: F) -> Self
    where
        F: Fn(&[Value]) -> Result<(), String> + Send + Sync + 'static,
    {
        self.arguments_validator = Some(Box::new(validator));
        self
    }

    pub fn build(self) -> MockFunction {
        MockFunction {
            name: self.name,
            namespace: self.namespace,
            return_value: self.return_value,
            side_effects: self.side_effects,
            call_count: 0,
            expected_calls: self.expected_calls,
            // The validator already has Send + Sync bounds from validates_args
            arguments_validator: self.arguments_validator,
            call_history: Vec::new(),
        }
    }
}

/// Mock runtime that can intercept function calls
pub struct MockRuntime {
    pub runtime: Runtime,
    pub mock_registry: MockRegistry,
}

impl MockRuntime {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
            mock_registry: MockRegistry::new(),
        }
    }

    pub fn with_mock(mut self, mock: MockFunction) -> Self {
        self.mock_registry.register(mock);
        self
    }

    /// Execute DAL source code with mock interception enabled.
    /// Mocks registered via with_mock() will automatically intercept function calls
    /// during execution thanks to Runtime integration.
    pub fn execute_with_mocks(&mut self, source_code: &str) -> Result<Value, String> {
        use crate::lexer::Lexer;
        use crate::parser::Parser;

        // Set mock registry on Runtime before execution (Runtime will use it for interception)
        self.mock_registry.enable();
        // Move our registry to Runtime (Runtime will own it during execution)
        let registry = std::mem::take(&mut self.mock_registry);
        self.runtime.set_mock_registry(registry);

        // Parse the source code
        let tokens = Lexer::new(source_code)
            .tokenize()
            .map_err(|e| format!("Lexer error: {}", e))?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| format!("Parser error: {}", e))?;

        // Execute - Runtime will automatically check for mocks in call_function/call_namespace_function
        let result = self
            .runtime
            .execute_program(program, None)
            .map_err(|e| format!("Runtime error: {}", e));

        // Get the registry back from Runtime (with updated call counts from execution)
        if let Some(rt_registry) = self.runtime.take_mock_registry() {
            self.mock_registry = rt_registry;
        }

        // Return result
        match result {
            Ok(_) => Ok(self.runtime.stack.last().cloned().unwrap_or(Value::Null)),
            Err(e) => Err(e),
        }
    }

    pub fn verify_mocks(&self) -> Result<(), String> {
        self.mock_registry.verify_all()
    }
}

/// Convenience functions for creating common mocks
pub mod mock_helpers {
    use super::*;

    pub fn mock_chain_mint() -> MockBuilder {
        MockBuilder::new("mint")
            .in_namespace("chain")
            .returns(Value::Int(12345))
            .logs("Mock chain::mint called")
    }

    pub fn mock_oracle_fetch() -> MockBuilder {
        MockBuilder::new("fetch")
            .in_namespace("oracle")
            .returns(Value::String("mock_price_data".to_string()))
            .logs("Mock oracle::fetch called")
    }

    pub fn mock_service_call() -> MockBuilder {
        MockBuilder::new("call")
            .in_namespace("service")
            .returns(Value::String("mock_service_response".to_string()))
            .logs("Mock service::call called")
    }

    pub fn mock_auth_session() -> MockBuilder {
        MockBuilder::new("session")
            .in_namespace("auth")
            .returns(Value::String("mock_session_id".to_string()))
            .logs("Mock auth::session called")
    }

    pub fn mock_crypto_hash() -> MockBuilder {
        MockBuilder::new("hash")
            .in_namespace("crypto")
            .returns(Value::String("mock_hash_value".to_string()))
            .logs("Mock crypto::hash called")
    }
}

/// Test utilities for working with mocks
pub trait MockTestUtils {
    /// Enable mocks (returns mutable reference to registry for chaining)
    fn setup_mocks(&mut self) -> &mut MockRegistry;
    fn teardown_mocks(&mut self);
    fn assert_mock_called(&self, name: &str, namespace: Option<&str>, expected_calls: usize);
    fn assert_mock_not_called(&self, name: &str, namespace: Option<&str>);
}

impl MockTestUtils for MockRuntime {
    fn setup_mocks(&mut self) -> &mut MockRegistry {
        self.mock_registry.enable();
        &mut self.mock_registry
    }

    fn teardown_mocks(&mut self) {
        self.mock_registry.clear();
        self.mock_registry.disable();
    }

    fn assert_mock_called(&self, name: &str, namespace: Option<&str>, expected_calls: usize) {
        let key = self.mock_registry.get_mock_key(name, namespace);
        if let Some(mock) = self.mock_registry.mocks.get(&key) {
            assert_eq!(
                mock.call_count, expected_calls,
                "Mock '{}' was called {} times, expected {}",
                key, mock.call_count, expected_calls
            );
        } else {
            panic!("Mock '{}' not found", key);
        }
    }

    fn assert_mock_not_called(&self, name: &str, namespace: Option<&str>) {
        let key = self.mock_registry.get_mock_key(name, namespace);
        if let Some(mock) = self.mock_registry.mocks.get(&key) {
            assert_eq!(
                mock.call_count, 0,
                "Mock '{}' was called {} times, expected 0",
                key, mock.call_count
            );
        }
    }
}

impl Default for MockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MockRuntime {
    fn default() -> Self {
        Self::new()
    }
}

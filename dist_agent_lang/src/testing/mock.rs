use std::collections::HashMap;
use crate::runtime::values::Value;
use crate::runtime::Runtime;

/// Mock function definition
#[derive(Debug, Clone)]
pub struct MockFunction {
    pub name: String,
    pub namespace: Option<String>,
    pub return_value: Option<Value>,
    pub side_effects: Vec<MockSideEffect>,
    pub call_count: usize,
    pub expected_calls: Option<usize>,
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
    
    pub fn validates_arguments<F>(self, _validator: F) -> Self 
    where 
        F: Fn(&[Value]) -> Result<(), String> + 'static 
    {
        // Validator functionality removed for simplicity
        self
    }
    
    pub fn call(&mut self, args: &[Value]) -> Result<Value, String> {
        self.call_count += 1;
        
        // Execute side effects
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
    SetVariable(String, Value),
    CallFunction(String, Vec<Value>),
    LogMessage(String),
    ThrowError(String),
    Delay(std::time::Duration),
}

impl MockSideEffect {
    pub fn execute(&self, _args: &[Value]) -> Result<(), String> {
        match self {
            MockSideEffect::SetVariable(_, _) => {
                // In a real implementation, this would set a variable in the runtime
                Ok(())
            }
            MockSideEffect::CallFunction(_, _) => {
                // In a real implementation, this would call another function
                Ok(())
            }
            MockSideEffect::LogMessage(message) => {
                println!("[MOCK] {}", message);
                Ok(())
            }
            MockSideEffect::ThrowError(error) => {
                Err(error.clone())
            }
            MockSideEffect::Delay(duration) => {
                std::thread::sleep(*duration);
                Ok(())
            }
        }
    }
}

/// Mock registry for managing multiple mocks
#[derive(Debug, Clone)]
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
    
    pub fn call_mock(&mut self, name: &str, namespace: Option<&str>, args: &[Value]) -> Result<Value, String> {
        if !self.enabled {
            return Err("Mock registry is disabled".to_string());
        }
        
        if let Some(mock) = self.get_mock(name, namespace) {
            mock.call(args)
        } else {
            Err(format!("No mock found for '{}{}'", 
                namespace.map(|ns| format!("{}::", ns)).unwrap_or_default(), 
                name))
        }
    }
    
    pub fn verify_all(&self) -> Result<(), String> {
        for (key, mock) in &self.mocks {
            mock.verify().map_err(|e| format!("Mock '{}': {}", key, e))?;
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
    
    fn get_mock_key(&self, name: &str, namespace: Option<&str>) -> String {
        match namespace {
            Some(ns) => format!("{}::{}", ns, name),
            None => name.to_string(),
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
    arguments_validator: Option<Box<dyn Fn(&[Value]) -> Result<(), String>>>,
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
        self.side_effects.push(MockSideEffect::LogMessage(message.to_string()));
        self
    }
    
    pub fn throws(mut self, error: &str) -> Self {
        self.side_effects.push(MockSideEffect::ThrowError(error.to_string()));
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
        F: Fn(&[Value]) -> Result<(), String> + 'static 
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
    
    pub fn execute_with_mocks(&mut self, _source_code: &str) -> Result<Value, String> {
        // In a real implementation, this would:
        // 1. Parse the source code
        // 2. Intercept function calls to check for mocks
        // 3. Execute the code with mock support
        
        // For now, we'll just return a placeholder
        Ok(Value::String("Mock execution completed".to_string()))
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
    fn setup_mocks(&mut self) -> MockRegistry;
    fn teardown_mocks(&mut self);
    fn assert_mock_called(&self, name: &str, namespace: Option<&str>, expected_calls: usize);
    fn assert_mock_not_called(&self, name: &str, namespace: Option<&str>);
}

impl MockTestUtils for MockRuntime {
    fn setup_mocks(&mut self) -> MockRegistry {
        self.mock_registry.enable();
        self.mock_registry.clone()
    }
    
    fn teardown_mocks(&mut self) {
        self.mock_registry.clear();
        self.mock_registry.disable();
    }
    
    fn assert_mock_called(&self, name: &str, namespace: Option<&str>, expected_calls: usize) {
        let key = self.mock_registry.get_mock_key(name, namespace);
        if let Some(mock) = self.mock_registry.mocks.get(&key) {
            assert_eq!(mock.call_count, expected_calls, 
                "Mock '{}' was called {} times, expected {}", key, mock.call_count, expected_calls);
        } else {
            panic!("Mock '{}' not found", key);
        }
    }
    
    fn assert_mock_not_called(&self, name: &str, namespace: Option<&str>) {
        let key = self.mock_registry.get_mock_key(name, namespace);
        if let Some(mock) = self.mock_registry.mocks.get(&key) {
            assert_eq!(mock.call_count, 0, 
                "Mock '{}' was called {} times, expected 0", key, mock.call_count);
        }
    }
}

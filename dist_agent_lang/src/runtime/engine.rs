use std::collections::HashMap;
use crate::runtime::values::Value;
use crate::runtime::scope::Scope;
use crate::runtime::functions::{Function, RuntimeError};
use crate::runtime::reentrancy::ReentrancyGuard;
use crate::runtime::safe_math::SafeMath;
use crate::runtime::state_isolation::StateIsolationManager;
use crate::stdlib::cross_chain_security::CrossChainSecurityManager;
use crate::runtime::advanced_security::AdvancedSecurityManager;
use crate::parser::ast::{Program, ServiceStatement};

pub struct Runtime {
    pub stack: Vec<Value>,
    pub scope: Scope,
    pub functions: HashMap<String, Function>,
    pub call_stack: Vec<CallFrame>,
    pub services: HashMap<String, ServiceInstance>, // NEW: Service instances
    pub current_service: Option<ServiceContext>, // NEW: Current service context for trust validation
    pub reentrancy_guard: ReentrancyGuard, // NEW: Re-entrancy protection
    pub state_manager: StateIsolationManager, // NEW: State isolation manager
    pub cross_chain_manager: CrossChainSecurityManager, // NEW: Cross-chain security manager
    pub advanced_security: AdvancedSecurityManager, // NEW: Advanced security features
    execution_start: Option<std::time::Instant>, // NEW: Track execution start time for timeout
}

// NEW: Service instance structure
#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub name: String,
    pub fields: HashMap<String, Value>,
    pub methods: Vec<crate::parser::ast::FunctionStatement>,
    pub events: Vec<crate::parser::ast::EventDeclaration>,
}

// NEW: Service context for trust validation
#[derive(Debug, Clone)]
pub struct ServiceContext {
    pub name: String,
    pub trust_model: String, // "centralized", "hybrid", "decentralized"
    pub has_admin_privileges: bool,
    pub has_cloudadmin_privileges: bool,
    pub has_web_privileges: bool,
    pub has_ai_privileges: bool,
    pub has_chain_privileges: bool,
    pub attributes: Vec<String>, // @trust, @web, @ai, @chain, etc.
}

#[derive(Debug)]
pub struct CallFrame {
    pub scope: Scope,
}

impl Runtime {
    pub fn new() -> Self {
        let mut runtime = Self {
            stack: Vec::with_capacity(64), // Pre-allocate stack space
            scope: Scope::new(),
            functions: HashMap::with_capacity(16), // Pre-allocate for built-ins
            call_stack: Vec::with_capacity(8), // Pre-allocate call stack
            services: HashMap::with_capacity(8), // NEW: Pre-allocate for services
            current_service: None, // NEW: Initialize current service context
            reentrancy_guard: ReentrancyGuard::new(), // NEW: Re-entrancy protection
            state_manager: StateIsolationManager::new(), // NEW: State isolation manager
            cross_chain_manager: CrossChainSecurityManager::new(), // NEW: Cross-chain security manager
            advanced_security: AdvancedSecurityManager::new(), // NEW: Advanced security features
            execution_start: None, // NEW: Initialize execution start time
        };

        // Register built-in functions
        runtime.register_builtins();
        runtime
    }

    pub fn with_capacities(stack_cap: usize, func_cap: usize, call_cap: usize) -> Self {
        let mut runtime = Self {
            stack: Vec::with_capacity(stack_cap),
            scope: Scope::new(),
            functions: HashMap::with_capacity(func_cap),
            call_stack: Vec::with_capacity(call_cap),
            services: HashMap::with_capacity(8), // NEW: Pre-allocate for services
            current_service: None, // NEW: Initialize current service context
            reentrancy_guard: ReentrancyGuard::new(), // NEW: Re-entrancy protection
            state_manager: StateIsolationManager::new(), // NEW: State isolation manager
            cross_chain_manager: CrossChainSecurityManager::new(), // NEW: Cross-chain security manager
            advanced_security: AdvancedSecurityManager::new(), // NEW: Advanced security features
            execution_start: None, // NEW: Initialize execution start time
        };
        
        // Register built-in functions
        runtime.register_builtins();
        runtime
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Result<Value, RuntimeError> {
        self.stack.pop().ok_or_else(RuntimeError::stack_underflow)
    }

    pub fn peek(&self) -> Result<&Value, RuntimeError> {
        self.stack.last().ok_or_else(RuntimeError::stack_underflow)
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.scope.set(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Result<Value, RuntimeError> {
        self.scope.get(name).ok_or_else(|| {
            RuntimeError::VariableNotFound(name.to_string())
        })
    }

    // Helper methods for value conversion
    pub fn value_to_string(&self, value: &Value) -> Result<String, RuntimeError> {
        match value {
            Value::String(s) => Ok(s.clone()),
            Value::Int(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Null => Ok("null".to_string()),
            _ => Err(RuntimeError::General("Cannot convert value to string".to_string())),
        }
    }

    pub fn value_to_int(&self, value: &Value) -> Result<i64, RuntimeError> {
        match value {
            Value::Int(i) => Ok(*i),
            Value::Float(f) => Ok(*f as i64),
            Value::String(s) => s.parse::<i64>().map_err(|_| 
                RuntimeError::General("Cannot convert string to int".to_string())
            ),
            Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(RuntimeError::General("Cannot convert value to int".to_string())),
        }
    }

    pub fn execute_program(&mut self, program: Program) -> Result<Option<Value>, RuntimeError> {
        use std::time::{Instant, Duration};
        
        const MAX_EXECUTION_TIME: Duration = Duration::from_secs(10);
        let start_time = Instant::now();
        self.execution_start = Some(start_time);
        
        // Phase 4: Check for MEV attacks before execution
        self.advanced_security.analyze_transaction_for_mev(&format!("{:?}", program))?;
        
        let mut result = None;
        
        for statement in program.statements {
            // Check timeout before each statement
            if let Some(start) = self.execution_start {
                if start.elapsed() > MAX_EXECUTION_TIME {
                    self.execution_start = None;
                    return Err(RuntimeError::ExecutionTimeout);
                }
            }
            
            match self.execute_statement(&statement) {
                Ok(value) => {
                    result = Some(value);
                }
                Err(e) => {
                    self.execution_start = None;
                    return Err(e);
                }
            }
        }
        
        self.execution_start = None;
        Ok(result)
    }

    pub fn register_function(&mut self, function: Function) {
        self.functions.insert(function.name.clone(), function);
    }
    
    /// Check if a function is considered sensitive and requires time-lock protection
    fn is_sensitive_function(&self, name: &str) -> bool {
        // Define sensitive functions that require time-lock protection
        let sensitive_functions = [
            "transfer", "withdraw", "mint", "burn", "approve",
            "admin_transfer", "emergency_stop", "upgrade_contract",
            "change_owner", "set_permissions", "bridge_transfer"
        ];
        
        sensitive_functions.iter().any(|&sensitive| name.contains(sensitive))
    }

    pub fn call_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        // Phase 4: Check time-lock restrictions for sensitive functions
        if self.is_sensitive_function(name) {
            self.advanced_security.check_timelock(name)?;
        }
        
        // Handle special built-in functions for array/map access
        if name == "__index__" {
            // Array/map access: __index__(container, key)
            if args.len() != 2 {
                return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
            }
            let container = &args[0];
            let key = &args[1];
            
            match container {
                Value::Map(ref map) => {
                    let key_str = match key {
                        Value::String(s) => s.clone(),
                        Value::Int(i) => i.to_string(),
                        _ => return Err(RuntimeError::General(format!(
                            "Map key must be string or int, got: {}", key.type_name()
                        ))),
                    };
                    return Ok(map.get(&key_str).cloned().unwrap_or(Value::Null));
                }
                Value::Array(ref arr) => {
                    let index = match key {
                        Value::Int(i) => *i as usize,
                        _ => return Err(RuntimeError::General(format!(
                            "Array index must be int, got: {}", key.type_name()
                        ))),
                    };
                    return Ok(arr.get(index).cloned().unwrap_or(Value::Null));
                }
                _ => return Err(RuntimeError::General(format!(
                    "Cannot index value of type: {}", container.type_name()
                ))),
            }
        }
        
        if name == "__index_assign__" {
            // Array/map assignment: __index_assign__(container, key, value)
            // This is called for: self.field[key] = value or arr[index] = value
            if args.len() != 3 {
                return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() });
            }
            let container = &args[0];
            let key = &args[1];
            let value = args[2].clone();
            
            // Check if we're in a service method context (self is in scope)
            if let Ok(Value::String(ref instance_id)) = self.get_variable("self") {
                // We're in a service method - the container is the result of evaluating self.field
                // We need to find which field this map came from and update it
                if let Value::Map(ref map) = container {
                    if let Some(instance) = self.services.get_mut(instance_id) {
                        // Find the field that contains this map
                        // Since maps are cloned when accessed, we can't use reference equality
                        // Instead, we'll update all map fields - this works if there's only one map
                        // being modified, which is the common case
                        let key_str = match key {
                            Value::String(s) => s.clone(),
                            Value::Int(i) => i.to_string(),
                            _ => return Err(RuntimeError::General(format!(
                                "Map key must be string or int, got: {}", key.type_name()
                            ))),
                        };
                        
                        // Try to find a matching map field by checking if the keys overlap
                        // This is a heuristic - in practice, we'd want to track the field name
                        for (_field_name, field_value) in instance.fields.iter_mut() {
                            if let Value::Map(ref field_map) = field_value {
                                // Check if this field's map has overlapping keys with the container map
                                // This is a simple heuristic to identify the correct field
                                if map.keys().any(|k| field_map.contains_key(k)) || 
                                   (map.is_empty() && field_map.is_empty()) {
                                    // This is likely the field we want to update
                                    if let Value::Map(ref mut field_map_mut) = field_value {
                                        field_map_mut.insert(key_str.clone(), value.clone());
                                        return Ok(value);
                                    }
                                }
                            }
                        }
                        
                        // Fallback: if no match found, update the first map field
                        // This handles the case where the map is empty or newly created
                        for (_field_name, field_value) in instance.fields.iter_mut() {
                            if let Value::Map(ref mut field_map) = field_value {
                                field_map.insert(key_str.clone(), value.clone());
                                return Ok(value);
                            }
                        }
                    }
                }
            }
            
            // Handle direct map/array variables (not service instance fields)
            match container {
                Value::Map(_) => {
                    // For now, we can't update a map that's not in a variable or service instance
                    return Err(RuntimeError::General(
                        "Map assignment requires the map to be stored in a variable or service instance field".to_string()
                    ));
                }
                Value::Array(_) => {
                    return Err(RuntimeError::General(
                        "Array assignment not yet fully implemented".to_string()
                    ));
                }
                _ => return Err(RuntimeError::General(format!(
                    "Cannot assign to index of type: {}", container.type_name()
                ))),
            }
        }
        
        // Handle namespace calls (e.g., oracle::fetch)
        if name.contains("::") {
            return self.call_namespace_function(name, args);
        }
        
        // Handle instance method calls (e.g., nft.initialize)
        if name.contains(".") {
            let parts: Vec<&str> = name.split(".").collect();
            if parts.len() == 2 {
                let instance_var = parts[0];
                let method_name = parts[1];
                
                // Get the instance ID from the variable
                let instance_id = match self.get_variable(instance_var) {
                    Ok(Value::String(id)) => id,
                    Ok(other) => return Err(RuntimeError::General(format!(
                        "Variable '{}' is not a service instance (got: {})", 
                        instance_var, other.type_name()
                    ))),
                    Err(_) => return Err(RuntimeError::General(format!(
                        "Variable '{}' not found", instance_var
                    ))),
                };
                
                // Find and clone the method (we need to avoid multiple mutable borrows)
                let method = {
                    let instance = self.services.get(&instance_id)
                        .ok_or_else(|| RuntimeError::General(format!(
                            "Service instance '{}' not found", instance_id
                        )))?;
                    instance.methods.iter()
                        .find(|m| m.name == method_name)
                        .ok_or_else(|| RuntimeError::General(format!(
                            "Method '{}' not found on service instance '{}'", 
                            method_name, instance_id
                        )))?
                        .clone()
                };
                
                // Execute the method - we'll get mutable access inside execute_service_method
                return self.execute_service_method(&instance_id, method_name, &method, args);
            }
        }
        
        let function = self.functions.get(name)
            .ok_or_else(|| RuntimeError::function_not_found(name.to_string()))?;
        
        // Create call frame
        let call_frame = CallFrame {
            scope: self.scope.clone(),
        };
        self.call_stack.push(call_frame);
        
        // Call the function
        let result = function.call(args, &mut self.scope);
        
        // Restore scope from call frame
        if let Some(frame) = self.call_stack.pop() {
            self.scope = frame.scope;
        }
        
        result
    }

    fn call_namespace_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        let parts: Vec<&str> = name.split("::").collect();
        if parts.len() != 2 {
            return Err(RuntimeError::General(format!("Invalid namespace call: {}", name)));
        }
        
        let namespace = parts[0];
        let function_name = parts[1];
        
        // Check if namespace is a registered service name
        if self.services.contains_key(namespace) {
            return self.call_service_instance_method(namespace, function_name, args);
        }
        
        match namespace {
            "oracle" => self.call_oracle_function(function_name, args),
            "service" => self.call_service_function(function_name, args),
            "sync" => self.call_sync_function(function_name, args),
            "cap" => self.call_cap_function(function_name, args),
            "chain" => self.call_chain_function(function_name, args),
            "auth" => self.call_auth_function(function_name, args),
            "log" => self.call_log_function(function_name, args),
            "crypto" => self.call_crypto_function(function_name, args),
            "kyc" => self.call_kyc_function(function_name, args),
            "aml" => self.call_aml_function(function_name, args),
            "web" => self.call_web_function(function_name, args),
            "database" => self.call_database_function(function_name, args),
            "agent" => self.call_agent_function(function_name, args),
            "ai" => self.call_ai_function(function_name, args),
            "desktop" => self.call_desktop_function(function_name, args),
            "mobile" => self.call_mobile_function(function_name, args),
            "iot" => self.call_iot_function(function_name, args),
            "admin" => self.call_admin_function(function_name, args),
            "cloudadmin" => self.call_cloudadmin_function(function_name, args),
            _ => {
                // Check if namespace is a registered service name (e.g., TestNFT::new())
                if self.services.contains_key(namespace) {
                    self.call_service_instance_method(namespace, function_name, args)
                } else {
                    Err(RuntimeError::General(format!("Unknown namespace: {}", namespace)))
                }
            }
        }
    }

    // Handle method calls on service instances (e.g., TestNFT::new(), TestNFT::someMethod())
    fn call_service_instance_method(&mut self, service_name: &str, method_name: &str, _args: &[Value]) -> Result<Value, RuntimeError> {
        if method_name == "new" {
            // Create a new instance of the service
            let service_template = self.services.get(service_name)
                .ok_or_else(|| RuntimeError::General(format!("Service {} not found", service_name)))?;
            
            // Create a new instance with copied fields
            let new_instance = ServiceInstance {
                name: service_template.name.clone(),
                fields: service_template.fields.clone(),
                methods: service_template.methods.clone(),
                events: service_template.events.clone(),
            };
            
            // Store the new instance with a unique identifier
            let instance_id = format!("{}_instance_{}", service_name, self.services.len());
            self.services.insert(instance_id.clone(), new_instance);
            
            // Return the instance identifier
            Ok(Value::String(instance_id))
        } else {
            // Call a method on the service (this would need more implementation)
            Err(RuntimeError::General(format!("Service method calls not yet implemented: {}.{}()", service_name, method_name)))
        }
    }

    fn call_oracle_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "create_query" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::String(format!("query_{}", args[0])))
            }
            "fetch" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                // Simulate fetching data
                Ok(Value::Int(42000)) // Simulated price
            }
            "stream" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String(format!("stream_{}", args[0])))
            }
            _ => Err(RuntimeError::function_not_found(format!("oracle::{}", name))),
        }
    }

    fn call_service_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "new" => {
                // service::new("ServiceName") - create a new instance of a service
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                let service_name = match &args[0] {
                    Value::String(name) => name.clone(),
                    _ => return Err(RuntimeError::General("service::new() expects a string service name".to_string())),
                };
                
                // Get the service template
                let service_template = self.services.get(&service_name)
                    .ok_or_else(|| RuntimeError::General(format!("Service '{}' not found. Available services: {:?}", 
                        service_name, self.services.keys().collect::<Vec<_>>())))?;
                
                // Create a new instance with copied fields
                let new_instance = ServiceInstance {
                    name: service_template.name.clone(),
                    fields: service_template.fields.clone(),
                    methods: service_template.methods.clone(),
                    events: service_template.events.clone(),
                };
                
                // Store the new instance with a unique identifier
                let instance_id = format!("{}_instance_{}", service_name, self.services.len());
                self.services.insert(instance_id.clone(), new_instance);
                
                // Return the instance identifier
                Ok(Value::String(instance_id))
            }
            "create_ai_service" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::String(format!("ai_service_{}", args[0])))
            }
            "ai" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String("AI response: This is a simulated AI response".to_string()))
            }
            "create_service_call" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String(format!("service_call_{}_{}", args[0], args[1])))
            }
            "call" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::String("Service call completed successfully".to_string()))
            }
            "create_webhook" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String(format!("webhook_config_{}_{}", args[0], args[1])))
            }
            "webhook" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String("Webhook delivered successfully".to_string()))
            }
            _ => Err(RuntimeError::function_not_found(format!("service::{}", name))),
        }
    }

    fn call_sync_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "create_sync_target" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String(format!("sync_target_{}_{}", args[0], args[1])))
            }
            "push" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::Bool(true))
            }
            "pull" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String("Data pulled successfully".to_string()))
            }
            "create_sync_filters" => {
                Ok(Value::String("sync_filters".to_string()))
            }
            _ => Err(RuntimeError::function_not_found(format!("sync::{}", name))),
        }
    }

    fn call_cap_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "create_principal" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String(format!("principal_{}_{}", args[0], args[1])))
            }
            "create_capability_request" => {
                if args.len() != 3 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() });
                }
                Ok(Value::String(format!("cap_request_{}_{}_{}", args[0], args[1], args[2])))
            }
            "check" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::Bool(true))
            }
            _ => Err(RuntimeError::function_not_found(format!("cap::{}", name))),
        }
    }

    fn call_chain_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        // Validate chain access based on current trust context
        if !self.validate_chain_trust() {
            return Err(RuntimeError::PermissionDenied("Chain access denied".to_string()));
        }

        match name {
            "deploy" => {
                if args.len() != 3 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() });
                }
                let chain_id = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(RuntimeError::TypeError { expected: "int".to_string(), got: args[0].type_name().to_string() }),
                };
                let contract_name = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err(RuntimeError::TypeError { expected: "string".to_string(), got: args[1].type_name().to_string() }),
                };
                let constructor_args = HashMap::new(); // Simplified for now
                let address = crate::stdlib::chain::deploy(chain_id, contract_name, constructor_args);
                Ok(Value::String(address))
            }
            "estimate_gas" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let chain_id = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(RuntimeError::TypeError { expected: "int".to_string(), got: args[0].type_name().to_string() }),
                };
                let operation = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err(RuntimeError::TypeError { expected: "string".to_string(), got: args[1].type_name().to_string() }),
                };
                let gas = crate::stdlib::chain::estimate_gas(chain_id, operation);
                Ok(Value::Int(gas))
            }
            "get_gas_price" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                let chain_id = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(RuntimeError::TypeError { expected: "int".to_string(), got: args[0].type_name().to_string() }),
                };
                let gas_price = crate::stdlib::chain::get_gas_price(chain_id);
                Ok(Value::Float(gas_price))
            }
            "get_block_timestamp" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                let chain_id = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(RuntimeError::TypeError { expected: "int".to_string(), got: args[0].type_name().to_string() }),
                };
                let timestamp = crate::stdlib::chain::get_block_timestamp(chain_id);
                Ok(Value::Int(timestamp))
            }
            "get_transaction_status" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let chain_id = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(RuntimeError::TypeError { expected: "int".to_string(), got: args[0].type_name().to_string() }),
                };
                let tx_hash = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err(RuntimeError::TypeError { expected: "string".to_string(), got: args[1].type_name().to_string() }),
                };
                let status = crate::stdlib::chain::get_transaction_status(chain_id, tx_hash);
                Ok(Value::String(status))
            }
            "get_balance" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let chain_id = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(RuntimeError::TypeError { expected: "int".to_string(), got: args[0].type_name().to_string() }),
                };
                let address = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err(RuntimeError::TypeError { expected: "string".to_string(), got: args[1].type_name().to_string() }),
                };
                let balance = crate::stdlib::chain::get_balance(chain_id, address);
                Ok(Value::Int(balance))
            }
            "call" => {
                if args.len() != 4 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() });
                }
                let chain_id = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(RuntimeError::TypeError { expected: "int".to_string(), got: args[0].type_name().to_string() }),
                };
                let contract_address = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err(RuntimeError::TypeError { expected: "string".to_string(), got: args[1].type_name().to_string() }),
                };
                let function_name = match &args[2] {
                    Value::String(s) => s.clone(),
                    _ => return Err(RuntimeError::TypeError { expected: "string".to_string(), got: args[2].type_name().to_string() }),
                };
                let args_map = HashMap::new(); // Simplified for now
                let result = crate::stdlib::chain::call(chain_id, contract_address, function_name, args_map);
                Ok(Value::String(result))
            }
            "mint" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let name = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(RuntimeError::TypeError { expected: "string".to_string(), got: args[0].type_name().to_string() }),
                };
                let metadata = HashMap::new(); // Simplified for now
                let asset_id = crate::stdlib::chain::mint(name, metadata);
                Ok(Value::Int(asset_id))
            }
            "update" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let asset_id = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(RuntimeError::TypeError { expected: "int".to_string(), got: args[0].type_name().to_string() }),
                };
                let updates = HashMap::new(); // Simplified for now
                let success = crate::stdlib::chain::update(asset_id, updates);
                Ok(Value::Bool(success))
            }
            "get" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                let asset_id = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(RuntimeError::TypeError { expected: "int".to_string(), got: args[0].type_name().to_string() }),
                };
                let asset_info = crate::stdlib::chain::get(asset_id);
                Ok(Value::String(format!("{:?}", asset_info)))
            }
            "exists" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                let asset_id = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(RuntimeError::TypeError { expected: "int".to_string(), got: args[0].type_name().to_string() }),
                };
                let exists = crate::stdlib::chain::exists(asset_id);
                Ok(Value::Bool(exists))
            }
            _ => Err(RuntimeError::function_not_found(format!("chain::{}", name))),
        }
    }

    fn call_auth_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "session" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let user_id = self.value_to_string(&args[0])?;
                let roles = match &args[1] {
                    Value::Array(arr) => {
                        let mut role_vec = Vec::new();
                        for role in arr {
                            role_vec.push(self.value_to_string(role)?);
                        }
                        role_vec
                    }
                    _ => return Err(RuntimeError::TypeError { expected: "array".to_string(), got: args[1].type_name().to_string() }),
                };
                
                let session = crate::stdlib::auth::session(user_id, roles);
                
                // Store session in runtime scope for later use
                let session_id = session.id.clone();
                self.scope.set(session_id.clone(), Value::Struct("session".to_string(), {
                    let mut fields = std::collections::HashMap::new();
                    fields.insert("id".to_string(), Value::String(session.id));
                    fields.insert("user_id".to_string(), Value::String(session.user_id));
                    fields.insert("roles".to_string(), Value::Array(session.roles.into_iter().map(Value::String).collect()));
                    fields.insert("permissions".to_string(), Value::Array(session.permissions.into_iter().map(Value::String).collect()));
                    fields.insert("created_at".to_string(), Value::Int(session.created_at));
                    fields.insert("expires_at".to_string(), Value::Int(session.expires_at));
                    fields
                }));
                
                Ok(Value::String(session_id))
            }
            "has_role" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let session_id = self.value_to_string(&args[0])?;
                let role = self.value_to_string(&args[1])?;
                
                // Get session from scope
                let session_value = self.scope.get(&session_id)
                    .ok_or_else(|| RuntimeError::VariableNotFound(session_id.clone()))?;
                
                let session = match session_value {
                    Value::Struct(_, fields) => {
                        let user_id = match fields.get("user_id") {
                            Some(Value::String(s)) => s.clone(),
                            _ => return Err(RuntimeError::General("Invalid session structure".to_string())),
                        };
                        let roles = match fields.get("roles") {
                            Some(Value::Array(arr)) => {
                                let mut role_vec = Vec::new();
                                for role in arr {
                                    role_vec.push(self.value_to_string(role)?);
                                }
                                role_vec
                            }
                            _ => return Err(RuntimeError::General("Invalid session structure".to_string())),
                        };
                        let permissions = match fields.get("permissions") {
                            Some(Value::Array(arr)) => {
                                let mut perm_vec = Vec::new();
                                for perm in arr {
                                    perm_vec.push(self.value_to_string(perm)?);
                                }
                                perm_vec
                            }
                            _ => return Err(RuntimeError::General("Invalid session structure".to_string())),
                        };
                        let created_at = match fields.get("created_at") {
                            Some(Value::Int(i)) => *i,
                            _ => return Err(RuntimeError::General("Invalid session structure".to_string())),
                        };
                        let expires_at = match fields.get("expires_at") {
                            Some(Value::Int(i)) => *i,
                            _ => return Err(RuntimeError::General("Invalid session structure".to_string())),
                        };
                        
                        crate::stdlib::auth::Session {
                            id: session_id,
                            user_id,
                            roles,
                            permissions,
                            created_at,
                            expires_at,
                        }
                    }
                    _ => return Err(RuntimeError::TypeError { expected: "session".to_string(), got: session_value.type_name().to_string() }),
                };
                
                let has_role = crate::stdlib::auth::has_role(&session, &role);
                Ok(Value::Bool(has_role))
            }
            "has_permission" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let session_id = self.value_to_string(&args[0])?;
                let permission = self.value_to_string(&args[1])?;
                
                // Get session from scope
                let session_value = self.scope.get(&session_id)
                    .ok_or_else(|| RuntimeError::VariableNotFound(session_id.clone()))?;
                
                let session = match session_value {
                    Value::Struct(_, fields) => {
                        let user_id = match fields.get("user_id") {
                            Some(Value::String(s)) => s.clone(),
                            _ => return Err(RuntimeError::General("Invalid session structure".to_string())),
                        };
                        let roles = match fields.get("roles") {
                            Some(Value::Array(arr)) => {
                                let mut role_vec = Vec::new();
                                for role in arr {
                                    role_vec.push(self.value_to_string(role)?);
                                }
                                role_vec
                            }
                            _ => return Err(RuntimeError::General("Invalid session structure".to_string())),
                        };
                        let permissions = match fields.get("permissions") {
                            Some(Value::Array(arr)) => {
                                let mut perm_vec = Vec::new();
                                for perm in arr {
                                    perm_vec.push(self.value_to_string(perm)?);
                                }
                                perm_vec
                            }
                            _ => return Err(RuntimeError::General("Invalid session structure".to_string())),
                        };
                        let created_at = match fields.get("created_at") {
                            Some(Value::Int(i)) => *i,
                            _ => return Err(RuntimeError::General("Invalid session structure".to_string())),
                        };
                        let expires_at = match fields.get("expires_at") {
                            Some(Value::Int(i)) => *i,
                            _ => return Err(RuntimeError::General("Invalid session structure".to_string())),
                        };
                        
                        crate::stdlib::auth::Session {
                            id: session_id,
                            user_id,
                            roles,
                            permissions,
                            created_at,
                            expires_at,
                        }
                    }
                    _ => return Err(RuntimeError::TypeError { expected: "session".to_string(), got: session_value.type_name().to_string() }),
                };
                
                let has_permission = crate::stdlib::auth::has_permission(&session, &permission);
                Ok(Value::Bool(has_permission))
            }
            "validate_credentials" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let username = self.value_to_string(&args[0])?;
                let password = self.value_to_string(&args[1])?;
                
                let user_id = crate::stdlib::auth::validate_credentials(&username, &password);
                match user_id {
                    Some(id) => Ok(Value::String(id)),
                    None => Ok(Value::Null),
                }
            }
            "create_role" => {
                if args.len() != 3 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() });
                }
                let name = self.value_to_string(&args[0])?;
                let permissions = match &args[1] {
                    Value::Array(arr) => {
                        let mut perm_vec = Vec::new();
                        for perm in arr {
                            perm_vec.push(self.value_to_string(perm)?);
                        }
                        perm_vec
                    }
                    _ => return Err(RuntimeError::TypeError { expected: "array".to_string(), got: args[1].type_name().to_string() }),
                };
                let description = self.value_to_string(&args[2])?;
                
                let role = crate::stdlib::auth::create_role(name, permissions, description);
                
                // Store role in runtime scope
                let role_name = role.name.clone();
                self.scope.set(role_name.clone(), Value::Struct("role".to_string(), {
                    let mut fields = std::collections::HashMap::new();
                    fields.insert("name".to_string(), Value::String(role.name));
                    fields.insert("permissions".to_string(), Value::Array(role.permissions.into_iter().map(Value::String).collect()));
                    fields.insert("description".to_string(), Value::String(role.description));
                    fields
                }));
                
                Ok(Value::String(role_name))
            }
            "get_role" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                let role_name = self.value_to_string(&args[0])?;
                
                let role = crate::stdlib::auth::get_role(&role_name);
                match role {
                    Some(r) => {
                        let role_name = r.name.clone();
                        self.scope.set(role_name.clone(), Value::Struct("role".to_string(), {
                            let mut fields = std::collections::HashMap::new();
                            fields.insert("name".to_string(), Value::String(r.name));
                            fields.insert("permissions".to_string(), Value::Array(r.permissions.into_iter().map(Value::String).collect()));
                            fields.insert("description".to_string(), Value::String(r.description));
                            fields
                        }));
                        
                        Ok(Value::String(role_name))
                    }
                    None => Ok(Value::Null),
                }
            }
            _ => Err(RuntimeError::function_not_found(format!("auth::{}", name))),
        }
    }

    fn call_log_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "info" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                println!("[INFO] {}: {}", args[0], args[1]);
                Ok(Value::Null)
            }
            "audit" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                println!("[AUDIT] {}: {}", args[0], args[1]);
                Ok(Value::Null)
            }
            _ => Err(RuntimeError::function_not_found(format!("log::{}", name))),
        }
    }

    fn call_crypto_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "hash" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                // Simulate hash generation
                Ok(Value::String("hash_1234567890abcdef".to_string()))
            }
            "sign" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String("signature_abcdef123456".to_string()))
            }
            "verify" => {
                if args.len() != 3 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() });
                }
                Ok(Value::Bool(true))
            }
            _ => Err(RuntimeError::function_not_found(format!("crypto::{}", name))),
        }
    }

    pub fn execute(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut last_result = Value::Null;
        
        for statement in &program.statements {
            match self.execute_statement(statement) {
                Ok(value) => last_result = value,
                Err(e) => return Err(e),
            }
        }
        
        Ok(last_result)
    }

    fn execute_statement(&mut self, statement: &crate::parser::ast::Statement) -> Result<Value, RuntimeError> {
        // Note: Timeout checking is done in execute_program() before each statement
        
        // Check timeout (using a thread-local or passed-in start time)
        // For now, we'll check in execute_program, but this could be enhanced
        match statement {
            crate::parser::ast::Statement::Let(let_stmt) => {
                let evaluated_value = self.evaluate_expression(&let_stmt.value)?;
                
                // Phase 4: Apply formal verification to variable assignments
                if let Err(e) = self.advanced_security.verify_assignment(&let_stmt.name, &evaluated_value) {
                    return Err(e);
                }
                
                self.set_variable(let_stmt.name.clone(), evaluated_value.clone());
                Ok(evaluated_value)
            }
            crate::parser::ast::Statement::Return(return_stmt) => {
                if let Some(expr) = &return_stmt.value {
                    self.evaluate_expression(expr)
                } else {
                    Ok(Value::Null)
                }
            }
            crate::parser::ast::Statement::Expression(expression) => {
                self.evaluate_expression(expression)
            }
            crate::parser::ast::Statement::Block(block_stmt) => {
                let mut last_result = Value::Null;
                for stmt in &block_stmt.statements {
                    match self.execute_statement(stmt) {
                        Ok(value) => last_result = value,
                        Err(e) => return Err(e),
                    }
                }
                Ok(last_result)
            }
            crate::parser::ast::Statement::Function(_func_stmt) => {
                // For now, we'll skip function registration during execution
                // In a real implementation, we'd need to handle this differently
                // to avoid lifetime issues with closures
                Ok(Value::Null)
            }
            crate::parser::ast::Statement::Service(service_stmt) => {
                self.execute_service_statement(service_stmt)
            }
            crate::parser::ast::Statement::If(if_stmt) => {
                let condition = self.evaluate_expression(&if_stmt.condition)?;
                
                if self.is_truthy(&condition) {
                    self.execute_statement(&crate::parser::ast::Statement::Block(if_stmt.consequence.clone()))
                } else if let Some(alternative) = &if_stmt.alternative {
                    self.execute_statement(&crate::parser::ast::Statement::Block(alternative.clone()))
                } else {
                    Ok(Value::Null)
                }
            }
            crate::parser::ast::Statement::While(while_stmt) => {
                let mut last_result = Value::Null;
                loop {
                    let condition = self.evaluate_expression(&while_stmt.condition)?;
                    if !self.is_truthy(&condition) {
                        break;
                    }
                    for stmt in &while_stmt.body.statements {
                        match self.execute_statement(stmt) {
                            Ok(value) => last_result = value,
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(last_result)
            }
            crate::parser::ast::Statement::Try(try_stmt) => {
                // Execute try block
                match self.execute_statement(&crate::parser::ast::Statement::Block(try_stmt.try_block.clone())) {
                    Ok(result) => {
                        // If try block succeeds, execute finally block if present
                        if let Some(finally_block) = &try_stmt.finally_block {
                            let _ = self.execute_statement(&crate::parser::ast::Statement::Block(finally_block.clone()));
                        }
                        Ok(result)
                    }
                    Err(error) => {
                        // Try to find a matching catch block
                        for catch_block in &try_stmt.catch_blocks {
                            // For now, we'll catch all errors
                            // In a more sophisticated implementation, we'd check error types
                            let mut catch_scope = self.scope.clone();
                            
                            // Bind error variable if specified
                            if let Some(error_var) = &catch_block.error_variable {
                                catch_scope.set(error_var.clone(), Value::String(format!("{:?}", error)));
                            }
                            
                            let mut catch_runtime = Runtime {
                                services: HashMap::new(),
                                stack: Vec::new(),
                                scope: catch_scope,
                                functions: HashMap::new(),
                                call_stack: Vec::new(),
                                current_service: None,
                                reentrancy_guard: ReentrancyGuard::new(),
                                state_manager: StateIsolationManager::new(),
                                cross_chain_manager: CrossChainSecurityManager::new(),
                                advanced_security: AdvancedSecurityManager::new(),
                                execution_start: None,
                            };
                            
                            match catch_runtime.execute_statement(&crate::parser::ast::Statement::Block(catch_block.body.clone())) {
                                Ok(result) => {
                                    // Execute finally block if present
                                    if let Some(finally_block) = &try_stmt.finally_block {
                                        let _ = self.execute_statement(&crate::parser::ast::Statement::Block(finally_block.clone()));
                                    }
                                    return Ok(result);
                                }
                                Err(_) => continue, // Try next catch block
                            }
                        }
                        
                        // If no catch block handled the error, execute finally and re-throw
                        if let Some(finally_block) = &try_stmt.finally_block {
                            let _ = self.execute_statement(&crate::parser::ast::Statement::Block(finally_block.clone()));
                        }
                        
                        Err(error)
                    }
                }
            }
            crate::parser::ast::Statement::Spawn(spawn_stmt) => {
                self.execute_spawn_statement(spawn_stmt)
            }
            crate::parser::ast::Statement::Agent(agent_stmt) => {
                self.execute_agent_statement(agent_stmt)
            }
            crate::parser::ast::Statement::Message(msg_stmt) => {
                // Evaluate message data
                let mut data = HashMap::new();
                for (key, expr) in &msg_stmt.data {
                    data.insert(key.clone(), self.evaluate_expression(expr)?);
                }
                
                // For now, just return the message data as a string
                Ok(Value::String(format!("Message to {}: {:?}", msg_stmt.recipient, data)))
            }
            crate::parser::ast::Statement::Event(event_stmt) => {
                // Evaluate event data
                let mut data = HashMap::new();
                for (key, expr) in &event_stmt.data {
                    data.insert(key.clone(), self.evaluate_expression(expr)?);
                }
                
                // For now, just return the event data as a string
                Ok(Value::String(format!("Event {}: {:?}", event_stmt.event_name, data)))
            }
            crate::parser::ast::Statement::ForIn(for_in_stmt) => {
                let iterable = self.evaluate_expression(&for_in_stmt.iterable)?;
                let items: Vec<crate::runtime::values::Value> = match &iterable {
                    crate::runtime::values::Value::List(list) => list.clone(),
                    crate::runtime::values::Value::Array(arr) => arr.clone(),
                    crate::runtime::values::Value::Map(map) => {
                        map.keys().cloned().map(crate::runtime::values::Value::String).collect()
                    }
                    other => {
                        return Err(RuntimeError::General(format!(
                            "for-in requires list, array, or map; got {}",
                            other.type_name()
                        )));
                    }
                };
                let mut last_result = crate::runtime::values::Value::Null;
                for item in items {
                    self.set_variable(for_in_stmt.variable.clone(), item);
                    for stmt in &for_in_stmt.body.statements {
                        match self.execute_statement(stmt) {
                            Ok(value) => last_result = value,
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(last_result)
            }
        }
    }

    fn evaluate_expression(&mut self, expression: &crate::parser::ast::Expression) -> Result<Value, RuntimeError> {
        match expression {
            crate::parser::ast::Expression::Literal(literal) => {
                match literal {
                    crate::lexer::tokens::Literal::Int(n) => Ok(Value::Int(*n)),
                    crate::lexer::tokens::Literal::Float(f) => Ok(Value::Float(*f)),
                    crate::lexer::tokens::Literal::String(s) => Ok(Value::String(s.clone())),
                    crate::lexer::tokens::Literal::Bool(b) => Ok(Value::Bool(*b)),
                    crate::lexer::tokens::Literal::Null => Ok(Value::Null),
                }
            }
            crate::parser::ast::Expression::Identifier(name) => {
                // Check if this is 'self' - if so, return the instance ID
                if name == "self" {
                    let self_id = self.get_variable("self")?;
                    if let Value::String(instance_id) = self_id {
                        // Return a reference to the instance (we'll handle field access separately)
                        Ok(Value::String(instance_id))
                    } else {
                        Ok(self_id)
                    }
                } else {
                    self.get_variable(name)
                }
            }
            crate::parser::ast::Expression::BinaryOp(left, operator, right) => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                
                match operator {
                    crate::lexer::tokens::Operator::Plus => self.add_values(left_val, right_val),
                    crate::lexer::tokens::Operator::Minus => self.subtract_values(left_val, right_val),
                    crate::lexer::tokens::Operator::Star => self.multiply_values(left_val, right_val),
                    crate::lexer::tokens::Operator::Slash => self.divide_values(left_val, right_val),
                    crate::lexer::tokens::Operator::Percent => self.modulo_values(left_val, right_val),
                    crate::lexer::tokens::Operator::Equal => Ok(Value::Bool(left_val == right_val)),
                    crate::lexer::tokens::Operator::NotEqual => Ok(Value::Bool(left_val != right_val)),
                    crate::lexer::tokens::Operator::Less => self.compare_values(left_val, right_val, "<"),
                    crate::lexer::tokens::Operator::LessEqual => self.compare_values(left_val, right_val, "<="),
                    crate::lexer::tokens::Operator::Greater => self.compare_values(left_val, right_val, ">"),
                    crate::lexer::tokens::Operator::GreaterEqual => self.compare_values(left_val, right_val, ">="),
                    crate::lexer::tokens::Operator::And => self.logical_and(left_val, right_val),
                    crate::lexer::tokens::Operator::Or => self.logical_or(left_val, right_val),
                    _ => Err(RuntimeError::UnsupportedOperation(format!("{:?}", operator)))
                }
            }
            crate::parser::ast::Expression::UnaryOp(operator, operand) => {
                let operand_val = self.evaluate_expression(operand)?;
                
                match operator {
                    crate::lexer::tokens::Operator::Minus => self.negate_value(operand_val),
                    crate::lexer::tokens::Operator::Not => self.logical_not(operand_val),
                    _ => Err(RuntimeError::UnsupportedOperation(format!("{:?}", operator)))
                }
            }
            crate::parser::ast::Expression::Assignment(name, value) => {
                let evaluated_value = self.evaluate_expression(value)?;
                self.set_variable(name.clone(), evaluated_value.clone());
                Ok(evaluated_value)
            }
            crate::parser::ast::Expression::FieldAssignment(object_expr, field_name, value_expr) => {
                // Evaluate the value to assign
                let value = self.evaluate_expression(value_expr)?;

                // Handle 'self.field = value' for service instances
                match object_expr.as_ref() {
                    crate::parser::ast::Expression::Identifier(var_name) if var_name == "self" => {
                        // Get the instance ID from 'self'
                        let self_id = self.get_variable("self")?;
                        if let Value::String(instance_id) = self_id {
                            // Update the field in the service instance
                            if let Some(instance) = self.services.get_mut(&instance_id) {
                                instance.fields.insert(field_name.clone(), value.clone());
                                Ok(value)
                            } else {
                                Err(RuntimeError::General(format!(
                                    "Service instance '{}' not found", instance_id
                                )))
                            }
                        } else {
                            Err(RuntimeError::General(format!(
                                "'self' is not a service instance reference"
                            )))
                        }
                    }
                    crate::parser::ast::Expression::Identifier(var_name) => {
                        // Get the current object from variables
                        match self.get_variable(var_name) {
                            Ok(mut current_value) => {
                                // Update the field in the object
                                if current_value.struct_set_field(field_name.clone(), value.clone()) {
                                    // Store the updated object back to the variable
                                    self.set_variable(var_name.clone(), current_value);
                                    Ok(value)
                                } else {
                                    Err(RuntimeError::General(format!(
                                        "Cannot assign to field '{}' on variable '{}' of type '{}'",
                                        field_name, var_name, match self.get_variable(var_name) {
                                            Ok(v) => v.type_name().to_string(),
                                            Err(_) => "unknown".to_string()
                                        }
                                    )))
                                }
                            }
                            Err(_) => Err(RuntimeError::General(format!(
                                "Variable '{}' not found for field assignment",
                                var_name
                            ))),
                        }
                    }
                    _ => Err(RuntimeError::General(format!(
                        "Field assignment only supported for variables, got: {:?}",
                        object_expr
                    ))),
                }
            }
            crate::parser::ast::Expression::FunctionCall(call) => {
                let mut args = Vec::new();
                for arg in &call.arguments {
                    args.push(self.evaluate_expression(arg)?);
                }
                self.call_function(&call.name, &args)
            }
            crate::parser::ast::Expression::Await(expr) => {
                // For now, just evaluate the expression normally
                // In a real implementation, this would handle async/await
                self.evaluate_expression(expr)
            }
            crate::parser::ast::Expression::Spawn(expr) => {
                // For now, evaluate the expression (e.g. spawn worker_process(i) runs the call)
                // In a real implementation, this would run in background and return a handle
                self.evaluate_expression(expr)
            }
            crate::parser::ast::Expression::Throw(expr) => {
                let error_value = self.evaluate_expression(expr)?;
                Err(RuntimeError::General(format!("Thrown error: {}", error_value)))
            }
            crate::parser::ast::Expression::FieldAccess(object_expr, field_name) => {
                // Evaluate the object expression to get the object
                let object_value = self.evaluate_expression(object_expr)?;

                // Handle 'self.field' access for service instances
                if let Value::String(ref instance_id) = object_value {
                    if let Some(instance) = self.services.get(instance_id) {
                        // Access field from service instance
                        return instance.fields.get(field_name)
                            .cloned()
                            .ok_or_else(|| RuntimeError::General(format!(
                                "Field '{}' not found on service instance '{}'", 
                                field_name, instance_id
                            )));
                    }
                }

                // Get the field value directly from the object
                match object_value {
                    Value::Struct(_, ref fields) => {
                        fields.get(field_name)
                            .cloned()
                            .ok_or_else(|| RuntimeError::General(format!(
                                "Field '{}' not found on struct", field_name
                            )))
                    }
                    Value::Map(ref map) => {
                        map.get(field_name)
                            .cloned()
                            .ok_or_else(|| RuntimeError::General(format!(
                                "Field '{}' not found in map", field_name
                            )))
                    }
                    _ => Err(RuntimeError::General(format!(
                        "Cannot access field '{}' on value of type '{}'",
                        field_name, object_value.type_name()
                    ))),
                }
            }
            crate::parser::ast::Expression::ObjectLiteral(properties) => {
                let mut object_value = HashMap::new();
                for (key, expr) in properties {
                    let value = self.evaluate_expression(expr)?;
                    object_value.insert(key.clone(), value);
                }
                Ok(Value::Map(object_value))
            }
            crate::parser::ast::Expression::ArrayLiteral(elements) => {
                let mut array_value = Vec::new();
                for expr in elements {
                    let value = self.evaluate_expression(expr)?;
                    array_value.push(value);
                }
                Ok(Value::Array(array_value))
            }
            crate::parser::ast::Expression::ArrowFunction { .. } => {
                // Arrow functions are passed as callbacks; for now return a placeholder.
                // Full implementation would capture env and be invokable by .then() etc.
                Ok(Value::Null)
            }
        }
    }

    fn add_values(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (&left, &right) {
            (Value::String(a), Value::String(b)) => Ok(Value::String(a.clone() + b)),
            (Value::String(a), other) => Ok(Value::String(format!("{}{}", a, other))),
            (other, Value::String(b)) => Ok(Value::String(format!("{}{}", other, b))),
            _ => SafeMath::add(&left, &right)
        }
    }

    fn subtract_values(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        SafeMath::subtract(&left, &right)
    }

    fn multiply_values(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        SafeMath::multiply(&left, &right)
    }

    fn divide_values(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        SafeMath::divide(&left, &right)
    }

    fn modulo_values(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        SafeMath::modulo(&left, &right)
    }

    fn compare_values(&self, left: Value, right: Value, op: &str) -> Result<Value, RuntimeError> {
        use std::cmp::Ordering;
        
        let ordering = match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            _ => return Err(RuntimeError::General("Cannot compare these value types".to_string())),
        };
        
        let result = match op {
            "<" => matches!(ordering, Ordering::Less),
            "<=" => matches!(ordering, Ordering::Less | Ordering::Equal),
            ">" => matches!(ordering, Ordering::Greater),
            ">=" => matches!(ordering, Ordering::Greater | Ordering::Equal),
            _ => return Err(RuntimeError::General(format!("Unknown comparison operator: {}", op))),
        };
        
        Ok(Value::Bool(result))
    }

    fn logical_and(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(self.is_truthy(&left) && self.is_truthy(&right)))
    }

    fn logical_or(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(self.is_truthy(&left) || self.is_truthy(&right)))
    }

    fn negate_value(&self, value: Value) -> Result<Value, RuntimeError> {
        match value {
            Value::Int(n) => Ok(Value::Int(-n)),
            _ => Err(RuntimeError::TypeMismatch("negation".to_string()))
        }
    }

    fn logical_not(&self, value: Value) -> Result<Value, RuntimeError> {
        Ok(Value::Bool(!self.is_truthy(&value)))
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
            Value::Result(ok_val, _err_val) => self.is_truthy(ok_val),
            Value::Option(opt_val) => {
                if let Some(val) = opt_val { 
                    self.is_truthy(val) 
                } else { 
                    false 
                }
            },
            Value::List(list) => !list.is_empty(),
            Value::Map(map) => !map.is_empty(),
            Value::Set(set) => !set.is_empty(),
            Value::Struct(_, _) => true,
            Value::Array(arr) => !arr.is_empty(),
        }
    }

    fn register_builtins(&mut self) {
        // Built-in print function
        let print_fn = Function::new(
            "print".to_string(),
            vec!["value".to_string()],
            |args, _| {
                if let Some(value) = args.first() {
                    println!("{}", value);
                    Ok(Value::Null)
                } else {
                    Err(RuntimeError::General("print: no arguments".to_string()))
                }
            }
        );
        self.register_function(print_fn);

        // Built-in add function
        let add_fn = Function::new(
            "add".to_string(),
            vec!["a".to_string(), "b".to_string()],
            |args, _| {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                
                match (&args[0], &args[1]) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                    _ => Err(RuntimeError::TypeError {
                        expected: "int, int".to_string(),
                        got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                    }),
                }
            }
        );
        self.register_function(add_fn);

        // Built-in len function
        let len_fn = Function::new(
            "len".to_string(),
            vec!["value".to_string()],
            |args, _| {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                
                match &args[0] {
                    Value::String(s) => Ok(Value::Int(s.len() as i64)),
                    _ => Err(RuntimeError::TypeError {
                        expected: "string".to_string(),
                        got: args[0].type_name().to_string(),
                    }),
                }
            }
        );
        self.register_function(len_fn);

        // Built-in type function
        let type_fn = Function::new(
            "type".to_string(),
            vec!["value".to_string()],
            |args, _| {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                
                Ok(Value::String(args[0].type_name().to_string()))
            }
        );
        self.register_function(type_fn);

        // Built-in to_string function
        let to_string_fn = Function::new(
            "to_string".to_string(),
            vec!["value".to_string()],
            |args, _| {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                
                Ok(Value::String(args[0].to_string()))
            }
        );
        self.register_function(to_string_fn);

        // Built-in to_int function
        let to_int_fn = Function::new(
            "to_int".to_string(),
            vec!["value".to_string()],
            |args, _| {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                
                match &args[0] {
                    Value::Int(n) => Ok(Value::Int(*n)),
                    Value::String(s) => {
                        s.parse::<i64>()
                            .map(Value::Int)
                            .map_err(|_| RuntimeError::TypeError {
                                expected: "int or string representing int".to_string(),
                                got: "string".to_string(),
                            })
                    }
                    _ => Err(RuntimeError::TypeError {
                        expected: "int or string".to_string(),
                        got: args[0].type_name().to_string(),
                    }),
                }
            }
        );
        self.register_function(to_int_fn);

        // Built-in to_bool function
        let to_bool_fn = Function::new(
            "to_bool".to_string(),
            vec!["value".to_string()],
            |args, _| {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                
                let is_truthy = match &args[0] {
                    Value::Bool(b) => *b,
                    Value::Int(n) => *n != 0,
                    Value::Float(f) => *f != 0.0,
                    Value::String(s) => !s.is_empty(),
                    Value::Null => false,
                    Value::Result(ok_val, _err_val) => {
                        // For Result type, check if Ok value is truthy
                        match ok_val.as_ref() {
                            Value::Bool(b) => *b,
                            Value::Int(n) => *n != 0,
                            Value::Float(f) => *f != 0.0,
                            Value::String(s) => !s.is_empty(),
                            Value::Null => false,
                            _ => true, // Default for complex types
                        }
                    },
                    Value::Option(opt_val) => {
                        if let Some(val) = opt_val { 
                            match val.as_ref() {
                                Value::Bool(b) => *b,
                                Value::Int(n) => *n != 0,
                                Value::Float(f) => *f != 0.0,
                                Value::String(s) => !s.is_empty(),
                                Value::Null => false,
                                _ => true, // Default for complex types
                            }
                        } else { 
                            false 
                        }
                    },
                    Value::List(list) => !list.is_empty(),
                    Value::Map(map) => !map.is_empty(),
                    Value::Set(set) => !set.is_empty(),
                    Value::Struct(_, _) => true,
                    Value::Array(arr) => !arr.is_empty(),
                };
                
                Ok(Value::Bool(is_truthy))
            }
        );
        self.register_function(to_bool_fn);
    }

    fn call_kyc_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "verify_identity" => {
                if args.len() != 4 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() });
                }
                // Simulate KYC verification
                Ok(Value::String(format!("kyc_verified_{}", args[1])))
            }
            "get_verification_status" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::String("verified".to_string()))
            }
            "revoke_verification" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::Bool(true))
            }
            "get_provider_info" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::String("SecureKYC Inc.".to_string()))
            }
            "list_providers" => {
                Ok(Value::String("securekyc,veriff".to_string()))
            }
            "get_verification_levels" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::String("basic,enhanced,premium".to_string()))
            }
            "validate_document" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::Bool(true))
            }
            "check_identity_match" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::Bool(true))
            }
            "get_compliance_report" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::String("Compliance report generated".to_string()))
            }
            _ => Err(RuntimeError::function_not_found(format!("kyc::{}", name))),
        }
    }

    fn call_aml_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "perform_check" => {
                if args.len() != 4 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() });
                }
                // Simulate AML check
                Ok(Value::String(format!("aml_check_{}", args[1])))
            }
            "get_check_status" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::String("passed".to_string()))
            }
            "get_provider_info" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::String("Chainalysis".to_string()))
            }
            "list_providers" => {
                Ok(Value::String("chainalysis,elliptic".to_string()))
            }
            "get_check_types" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                Ok(Value::String("sanctions,pep,adverse_media,risk_assessment".to_string()))
            }
            "screen_transaction" => {
                if args.len() != 4 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() });
                }
                Ok(Value::String("approved".to_string()))
            }
            "monitor_address" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String("monitoring_active".to_string()))
            }
            "get_risk_assessment" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String("low_risk".to_string()))
            }
            "check_sanctions_list" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                Ok(Value::String("clear".to_string()))
            }
            _ => Err(RuntimeError::function_not_found(format!("aml::{}", name))),
        }
    }

    fn call_web_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        // Validate web access based on current trust context
        if !self.validate_web_trust() {
            return Err(RuntimeError::PermissionDenied("Web access denied".to_string()));
        }

        match name {
            // === HTTP SERVER FUNCTIONS ===
            "create_server" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let port = self.value_to_int(&args[0])?;
                Ok(Value::String(format!("enhanced_server_{}", port)))
            }
            "add_route" => {
                if args.len() != 4 { return Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() }); }
                let method = self.value_to_string(&args[1])?;
                let path = self.value_to_string(&args[2])?;
                let handler = self.value_to_string(&args[3])?;
                Ok(Value::String(format!("route_{}_{}_{}", method, path, handler)))
            }
            "add_middleware" => {
                if args.len() != 4 { return Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() }); }
                let name = self.value_to_string(&args[1])?;
                let handler = self.value_to_string(&args[2])?;
                let priority = self.value_to_int(&args[3])?;
                Ok(Value::String(format!("middleware_{}_{}_{}", name, handler, priority)))
            }
            "configure_cors" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let enabled = if let Value::Bool(b) = &args[1] { *b } else { false };
                Ok(Value::String(format!("cors_configured_{}", enabled)))
            }
            "serve_static_files" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let path = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("static_served_{}", path)))
            }
            "start_server" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("server_started".to_string()))
            }
            
            // === HTTP CLIENT FUNCTIONS ===
            "create_client" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let base_url = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("enhanced_client_{}", base_url)))
            }
            "get_request" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let url = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("response_{}", url)))
            }
            "post_request" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let url = self.value_to_string(&args[0])?;
                let _data = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("posted_{}", url)))
            }
            
            // === FRONTEND FRAMEWORK FUNCTIONS ===
            "create_html_page" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let title = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("html_page_{}", title)))
            }
            "add_css_file" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let css_path = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("css_added_{}", css_path)))
            }
            "add_js_file" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let js_path = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("js_added_{}", js_path)))
            }
            "create_element" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let tag = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("element_{}", tag)))
            }
            "add_attribute" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let key = self.value_to_string(&args[1])?;
                let value = self.value_to_string(&args[2])?;
                Ok(Value::String(format!("attr_{}_{}", key, value)))
            }
            "add_event_handler" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let event = self.value_to_string(&args[1])?;
                let handler = self.value_to_string(&args[2])?;
                Ok(Value::String(format!("event_{}_{}", event, handler)))
            }
            "append_child" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("child_appended".to_string()))
            }
            "render_html_page" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("<!DOCTYPE html><html>...</html>".to_string()))
            }
            "render_html_element" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let element = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("<{}>", element)))
            }
            "create_form" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let action = self.value_to_string(&args[0])?;
                let method = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("form_{}_{}", action, method)))
            }
            "create_input" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let input_type = self.value_to_string(&args[0])?;
                let name = self.value_to_string(&args[1])?;
                let placeholder = self.value_to_string(&args[2])?;
                Ok(Value::String(format!("input_{}_{}_{}", input_type, name, placeholder)))
            }
            "create_button" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let text = self.value_to_string(&args[0])?;
                let button_type = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("button_{}_{}", text, button_type)))
            }
            
            // === API FRAMEWORK FUNCTIONS ===
            "create_api_endpoint" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let path = self.value_to_string(&args[0])?;
                let method = self.value_to_string(&args[1])?;
                let handler = self.value_to_string(&args[2])?;
                Ok(Value::String(format!("api_endpoint_{}_{}_{}", path, method, handler)))
            }
            "add_auth_requirement" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let required = if let Value::Bool(b) = &args[1] { *b } else { false };
                Ok(Value::String(format!("auth_required_{}", required)))
            }
            "add_rate_limit" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let rpm = self.value_to_int(&args[1])?;
                let burst = self.value_to_int(&args[2])?;
                Ok(Value::String(format!("rate_limit_{}_{}", rpm, burst)))
            }
            "validate_json_request" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            
            // === WEBSOCKET FUNCTIONS ===
            "create_websocket_server" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let port = self.value_to_int(&args[0])?;
                Ok(Value::String(format!("websocket_server_{}", port)))
            }
            "add_websocket_connection" => {
                if args.len() >= 2 { 
                    let connection_id = self.value_to_string(&args[1])?;
                    Ok(Value::String(format!("connection_added_{}", connection_id)))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() })
                }
            }
            "join_room" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let connection_id = self.value_to_string(&args[1])?;
                let room_name = self.value_to_string(&args[2])?;
                Ok(Value::String(format!("joined_room_{}_{}", connection_id, room_name)))
            }
            "broadcast_to_room" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let _room_name = self.value_to_string(&args[1])?;
                let _message = self.value_to_string(&args[2])?;
                Ok(Value::Int(5)) // Simulated connection count
            }
            
            // === TEMPLATE ENGINE FUNCTIONS ===
            "create_template" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let name = self.value_to_string(&args[0])?;
                let content = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("template_{}_{}", name, content.len())))
            }
            "add_template_variable" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let key = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("variable_added_{}", key)))
            }
            "render_advanced_template" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("rendered_template_content".to_string()))
            }
            "render_template" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let template = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("rendered_{}", template)))
            }
            
            // === LEGACY FUNCTIONS (for backward compatibility) ===
            "create_html_element" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let tag = self.value_to_string(&args[0])?;
                let attributes = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("element_{}_{}", tag, attributes)))
            }
            "render_html" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let element = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("<{}>", element)))
            }
            "parse_url" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let url = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("parsed_{}", url)))
            }
            "json_response" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let data = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("json_{}", data)))
            }
            "html_response" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let html = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("html_{}", html)))
            }
            "error_response" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let status = self.value_to_int(&args[0])?;
                let message = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("error_{}_{}", status, message)))
            }
            _ => Err(RuntimeError::function_not_found(format!("web::{}", name))),
        }
    }

    fn call_database_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            "connect" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let connection_string = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("db_connected_{}", connection_string)))
            }
            "query" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let sql = self.value_to_string(&args[0])?;
                let _params = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("query_result_{}", sql)))
            }
            "transaction" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let operations = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("txn_{}", operations)))
            }
            "commit_transaction" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let _transaction = self.value_to_string(&args[0])?;
                Ok(Value::Bool(true))
            }
            "rollback_transaction" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let _transaction = self.value_to_string(&args[0])?;
                Ok(Value::Bool(true))
            }
            "create_table" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let _table_name = self.value_to_string(&args[0])?;
                let _schema = self.value_to_string(&args[1])?;
                Ok(Value::Bool(true))
            }
            "drop_table" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let _table_name = self.value_to_string(&args[0])?;
                Ok(Value::Bool(true))
            }
            "get_table_schema" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let table_name = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("schema_{}", table_name)))
            }
            "list_tables" => {
                if args.len() != 0 { return Err(RuntimeError::ArgumentCountMismatch { expected: 0, got: args.len() }); }
                Ok(Value::String("users,products,orders".to_string()))
            }
            "backup_database" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let _backup_path = self.value_to_string(&args[0])?;
                Ok(Value::Bool(true))
            }
            "restore_database" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let _backup_path = self.value_to_string(&args[0])?;
                Ok(Value::Bool(true))
            }
            "close_connection" => {
                if args.len() != 0 { return Err(RuntimeError::ArgumentCountMismatch { expected: 0, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "ping_database" => {
                if args.len() != 0 { return Err(RuntimeError::ArgumentCountMismatch { expected: 0, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "get_query_plan" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let sql = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("plan_{}", sql)))
            }

            // === PHASE 3: ADVANCED DATABASE FUNCTIONS ===

            // Connection Pool Functions
            "create_connection_pool" => {
                if args.len() != 4 { return Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() }); }
                let pool_name = self.value_to_string(&args[0])?;
                let _connection_string = self.value_to_string(&args[1])?;
                let max_connections = self.value_to_int(&args[2])?;
                let _min_connections = self.value_to_int(&args[3])?;
                Ok(Value::String(format!("pool_created_{}_{}", pool_name, max_connections)))
            }
            "get_connection_from_pool" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("connection_from_pool".to_string()))
            }
            "return_connection_to_pool" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("connection_returned".to_string()))
            }

            // Query Builder Functions
            "create_query_builder" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let table_name = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("query_builder_{}", table_name)))
            }
            "qb_select" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("qb_select_applied".to_string()))
            }
            "qb_where" => {
                if args.len() != 4 { return Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() }); }
                Ok(Value::String("qb_where_applied".to_string()))
            }
            "qb_join" => {
                if args.len() != 5 { return Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() }); }
                Ok(Value::String("qb_join_applied".to_string()))
            }
            "qb_order_by" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                Ok(Value::String("qb_order_by_applied".to_string()))
            }
            "qb_limit" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("qb_limit_applied".to_string()))
            }
            "qb_offset" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("qb_offset_applied".to_string()))
            }
            "qb_build_sql" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("SELECT * FROM table WHERE condition".to_string()))
            }
            "qb_execute" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("qb_execute_result".to_string()))
            }

            // Migration Functions
            "create_migration_manager" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let migrations_table = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("migration_manager_{}", migrations_table)))
            }
            "create_migration" => {
                if args.len() != 4 { return Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() }); }
                let version = self.value_to_string(&args[0])?;
                let name = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("migration_{}_{}", version, name)))
            }
            "apply_migration" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "rollback_migration" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // Caching Functions
            "create_cache" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("cache_created".to_string()))
            }
            "cache_set" => {
                if args.len() >= 3 {
                    let key = self.value_to_string(&args[1])?;
                    Ok(Value::String(format!("cached_{}", key)))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() })
                }
            }
            "cache_get" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let key = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("cache_hit_{}", key)))
            }
            "cache_delete" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let key = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("cache_deleted_{}", key)))
            }
            "cache_clear" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Int(42))
            }

            // File System Functions
            "read_file" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let path = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("file_content_{}", path)))
            }
            "write_file" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let path = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("file_written_{}", path)))
            }
            "delete_file" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let path = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("file_deleted_{}", path)))
            }
            "list_directory" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let path = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("directory_listed_{}", path)))
            }
            "create_directory" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let path = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("directory_created_{}", path)))
            }
            "file_exists" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "get_file_info" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let path = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("file_info_{}", path)))
            }

            // Data Validation Functions
            "create_validation_rule" => {
                if args.len() != 4 { return Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() }); }
                let field = self.value_to_string(&args[0])?;
                let rule_type = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("validation_rule_{}_{}", field, rule_type)))
            }
            "validate_data" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // Enhanced Backup/Restore Functions
            "create_backup" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("backup_created".to_string()))
            }
            "restore_from_backup" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // Performance Monitoring Functions
            "get_database_metrics" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("database_metrics".to_string()))
            }
            "log_query_stats" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                Ok(Value::String("query_stats_logged".to_string()))
            }

            _ => Err(RuntimeError::function_not_found(format!("database::{}", name))),
        }
    }

    fn call_ai_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        // Validate AI access based on current trust context
        if !self.validate_ai_trust() {
            return Err(RuntimeError::PermissionDenied("AI access denied".to_string()));
        }

        match name {
            // === PHASE 4: AI AGENT FUNCTIONS ===

            // Agent Lifecycle Management
            "spawn_agent" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }

                // Parse agent configuration from the argument
                let config_value = &args[0];
                let agent_config = self.parse_agent_config(config_value)?;

                match crate::stdlib::agent::spawn(agent_config) {
                    Ok(agent_context) => {
                        // Store agent context in runtime scope
                        let agent_id = agent_context.agent_id.clone();
                        self.scope.set(
                            agent_id.clone(),
                            Value::Struct("agent_context".to_string(), {
                                let mut fields = std::collections::HashMap::new();
                                fields.insert("agent_id".to_string(), Value::String(agent_id.clone()));
                                fields.insert("status".to_string(), Value::String(agent_context.status.to_string()));
                                fields.insert("agent_type".to_string(), Value::String(agent_context.config.agent_type.to_string()));
                                fields
                            })
                        );
                        Ok(Value::String(agent_id))
                    }
                    Err(e) => Err(RuntimeError::General(e))
                }
            }
            "terminate_agent" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let _agent_id = self.value_to_string(&args[0])?;
                // Mock termination - in real implementation this would terminate the actual agent
                Ok(Value::Bool(true))
            }
            "get_agent_status" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let _agent_id = self.value_to_string(&args[0])?;
                // Mock status - in real implementation this would check actual agent status
                Ok(Value::String("idle".to_string()))
            }

            // Message Passing System
            "send_message" => {
                if args.len() != 5 { return Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() }); }
                let sender_id = self.value_to_string(&args[0])?;
                let receiver_id = self.value_to_string(&args[1])?;
                let message_type = self.value_to_string(&args[2])?;
                let content = args[3].clone();
                let message_id = format!("msg_{}", generate_id());

                let message = crate::stdlib::agent::create_agent_message(
                    message_id,
                    sender_id.clone(),
                    receiver_id.clone(),
                    message_type,
                    content
                );

                match crate::stdlib::agent::communicate(&sender_id, &receiver_id, message) {
                    Ok(_) => Ok(Value::String(format!("message_sent_{}_{}", sender_id, receiver_id))),
                    Err(e) => Err(RuntimeError::General(e))
                }
            }
            "receive_message" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let _agent_id = self.value_to_string(&args[0])?;
                // Mock message reception - in real implementation this would check message queue
                Ok(Value::String("message_received".to_string()))
            }
            "process_message_queue" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let _agent_id = self.value_to_string(&args[0])?;
                // Mock queue processing - in real implementation this would process actual queue
                Ok(Value::String("messages_processed".to_string()))
            }
            "process_message" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let _agent_id = self.value_to_string(&args[0])?;
                let _message_id = self.value_to_string(&args[1])?;
                // Mock message processing - in real implementation this would process actual message
                Ok(Value::String("message_processed".to_string()))
            }

            // Task Management
            "create_task" => {
                if args.len() >= 3 {
                    let agent_id = self.value_to_string(&args[0])?;
                    let task_type = self.value_to_string(&args[1])?;
                    let description = self.value_to_string(&args[2])?;
                    let priority = if args.len() > 3 {
                        self.value_to_string(&args[3])?
                    } else {
                        "medium".to_string()
                    };

                    let task_id = format!("task_{}", generate_id());
                    let task = crate::stdlib::agent::create_agent_task(
                        task_id.clone(),
                        description.clone(),
                        &priority
                    );

                    match task {
                        Some(_task_obj) => {
                            // Mock task assignment - in real implementation this would assign to actual agent
                            Ok(Value::String(format!("task_created_{}_{}", agent_id, task_type)))
                        }
                        None => Err(RuntimeError::General("Failed to create task".to_string()))
                    }
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() })
                }
            }
            "create_task_from_message" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let _agent_id = self.value_to_string(&args[0])?;
                let _message_id = self.value_to_string(&args[1])?;
                // Mock task creation from message - in real implementation this would create task from actual message
                Ok(Value::String("task_from_message".to_string()))
            }
            "execute_task" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let agent_id = self.value_to_string(&args[0])?;
                let task_id = self.value_to_string(&args[1])?;
                // Mock task execution - in real implementation this would execute actual task
                Ok(Value::String(format!("task_executed_{}_{}", agent_id, task_id)))
            }

            // AI Processing Functions
            "analyze_text" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let text = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("text_analyzed_{}", text.len())))
            }
            "analyze_image" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("image_analyzed".to_string()))
            }
            "generate_text" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let prompt = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("generated_response_for_{}", prompt)))
            }
            "train_model" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("model_trained".to_string()))
            }
            "predict" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("prediction_made".to_string()))
            }

            // Agent Coordination
            "create_coordinator" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let coordinator_id = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("coordinator_created_{}", coordinator_id)))
            }
            "add_agent_to_coordinator" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("agent_added_to_coordinator".to_string()))
            }
            "create_workflow" => {
                if args.len() >= 2 {
                    let coordinator_id = self.value_to_string(&args[0])?;
                    let workflow_name = self.value_to_string(&args[1])?;
                    Ok(Value::String(format!("workflow_created_{}_{}", coordinator_id, workflow_name)))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() })
                }
            }
            "execute_workflow" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let coordinator_id = self.value_to_string(&args[0])?;
                let workflow_id = self.value_to_string(&args[1])?;
                Ok(Value::String(format!("workflow_executed_{}_{}", coordinator_id, workflow_id)))
            }

            // Agent State Management
            "save_agent_state" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "load_agent_state" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("agent_state_loaded".to_string()))
            }

            // Agent Communication Protocols
            "create_communication_protocol" => {
                if args.len() >= 3 {
                    let name = self.value_to_string(&args[0])?;
                    Ok(Value::String(format!("protocol_created_{}", name)))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() })
                }
            }
            "validate_message_protocol" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // Performance Monitoring
            "get_agent_metrics" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("agent_metrics".to_string()))
            }
            "get_coordinator_metrics" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("coordinator_metrics".to_string()))
            }

            _ => Err(RuntimeError::function_not_found(format!("ai::{}", name))),
        }
    }

    fn call_agent_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            // === AGENT LIFECYCLE MANAGEMENT ===

            "spawn" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let config_value = &args[0];
                let agent_config = self.parse_agent_config(config_value)?;

                match crate::stdlib::agent::spawn(agent_config) {
                    Ok(agent_context) => {
                        let agent_id = agent_context.agent_id.clone();
                        self.scope.set(
                            agent_id.clone(),
                            Value::Struct("agent_context".to_string(), {
                                let mut fields = std::collections::HashMap::new();
                                fields.insert("agent_id".to_string(), Value::String(agent_id.clone()));
                                fields.insert("status".to_string(), Value::String(agent_context.status.to_string()));
                                fields.insert("agent_type".to_string(), Value::String(agent_context.config.agent_type.to_string()));
                                fields
                            })
                        );
                        Ok(Value::String(agent_id))
                    }
                    Err(e) => Err(RuntimeError::General(e))
                }
            }
            "terminate" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let _agent_id = self.value_to_string(&args[0])?;
                // Mock termination - in real implementation this would terminate actual agent
                Ok(Value::Bool(true))
            }
            "get_status" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let _agent_id = self.value_to_string(&args[0])?;
                // Mock status - in real implementation this would check actual agent status
                Ok(Value::String("idle".to_string()))
            }

            // === AGENT COORDINATION ===

            "coordinate" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let agent_id = self.value_to_string(&args[0])?;
                let task_description = self.value_to_string(&args[1])?;
                let coordination_type = self.value_to_string(&args[2])?;

                // Create a mock task for coordination
                let task = crate::stdlib::agent::create_agent_task(
                    format!("task_{}", generate_id()),
                    task_description.clone(),
                    "medium"
                );

                match task {
                    Some(task_obj) => {
                        match crate::stdlib::agent::coordinate(&agent_id, task_obj, &coordination_type) {
                            Ok(_) => Ok(Value::String(format!("coordinated_{}_{}", agent_id, coordination_type))),
                            Err(e) => Err(RuntimeError::General(e))
                        }
                    }
                    None => Err(RuntimeError::General("Failed to create coordination task".to_string()))
                }
            }

            // === AGENT COMMUNICATION ===

            "communicate" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                let sender_id = self.value_to_string(&args[0])?;
                let receiver_id = self.value_to_string(&args[1])?;
                let message_content = args[2].clone();

                let message = crate::stdlib::agent::create_agent_message(
                    format!("msg_{}", generate_id()),
                    sender_id.clone(),
                    receiver_id.clone(),
                    "direct_communication".to_string(),
                    message_content
                );

                match crate::stdlib::agent::communicate(&sender_id, &receiver_id, message) {
                    Ok(_) => Ok(Value::String(format!("message_sent_{}_{}", sender_id, receiver_id))),
                    Err(e) => Err(RuntimeError::General(e))
                }
            }

            // === AGENT EVOLUTION ===

            "evolve" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let agent_id = self.value_to_string(&args[0])?;
                // Parse evolution data from second argument
                let evolution_data = std::collections::HashMap::new(); // Mock data

                match crate::stdlib::agent::evolve(&agent_id, evolution_data) {
                    Ok(_) => Ok(Value::String(format!("agent_evolved_{}", agent_id))),
                    Err(e) => Err(RuntimeError::General(e))
                }
            }

            // === AGENT CAPABILITY VALIDATION ===

            "validate_capabilities" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                let agent_type = self.value_to_string(&args[0])?;
                // Parse required capabilities from second argument
                let required_capabilities = vec!["basic_processing".to_string()]; // Mock

                match crate::stdlib::agent::validate_capabilities(&agent_type, required_capabilities) {
                    Ok(_) => Ok(Value::Bool(true)),
                    Err(e) => Err(RuntimeError::General(e))
                }
            }

            // === AGENT CONFIGURATION ===

            "create_config" => {
                if args.len() >= 2 {
                    let name = self.value_to_string(&args[0])?;
                    let agent_type_str = self.value_to_string(&args[1])?;

                    let config = crate::stdlib::agent::create_agent_config(name, &agent_type_str, "default".to_string());
                    match config {
                        Some(_) => Ok(Value::String("config_created".to_string())),
                        None => Err(RuntimeError::General("Failed to create agent config".to_string()))
                    }
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() })
                }
            }

            "create_task" => {
                if args.len() >= 2 {
                    let task_id = format!("task_{}", generate_id());
                    let description = self.value_to_string(&args[0])?;
                    let priority = if args.len() > 1 {
                        self.value_to_string(&args[1])?
                    } else {
                        "medium".to_string()
                    };

                    let task = crate::stdlib::agent::create_agent_task(
                        task_id.clone(),
                        description.clone(),
                        &priority
                    );

                    match task {
                        Some(_) => Ok(Value::String(task_id)),
                        None => Err(RuntimeError::General("Failed to create task".to_string()))
                    }
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() })
                }
            }

            "create_message" => {
                if args.len() >= 4 {
                    let message_id = format!("msg_{}", generate_id());
                    let sender_id = self.value_to_string(&args[0])?;
                    let receiver_id = self.value_to_string(&args[1])?;
                    let message_type = self.value_to_string(&args[2])?;
                    let content = args[3].clone();

                    let _message = crate::stdlib::agent::create_agent_message(
                        message_id.clone(),
                        sender_id,
                        receiver_id,
                        message_type,
                        content
                    );

                    Ok(Value::String(message_id))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() })
                }
            }

            _ => Err(RuntimeError::function_not_found(format!("agent::{}", name))),
        }
    }

    fn call_desktop_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            // === PHASE 5: DESKTOP GUI FUNCTIONS ===

            // Window Management
            "create_window" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("window_created".to_string()))
            }
            "show_window" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "hide_window" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "close_window" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "maximize_window" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "minimize_window" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "restore_window" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // UI Component Creation
            "create_button" => {
                if args.len() >= 5 {
                    let text = self.value_to_string(&args[0])?;
                    Ok(Value::String(format!("button_created_{}", text)))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_label" => {
                if args.len() >= 5 {
                    let text = self.value_to_string(&args[0])?;
                    Ok(Value::String(format!("label_created_{}", text)))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_text_field" => {
                if args.len() >= 5 {
                    Ok(Value::String("text_field_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_text_area" => {
                if args.len() >= 5 {
                    Ok(Value::String("text_area_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_checkbox" => {
                if args.len() >= 5 {
                    Ok(Value::String("checkbox_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_combobox" => {
                if args.len() >= 5 {
                    Ok(Value::String("combobox_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_listbox" => {
                if args.len() >= 5 {
                    Ok(Value::String("listbox_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_table" => {
                if args.len() >= 5 {
                    Ok(Value::String("table_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_menu_bar" => {
                Ok(Value::String("menu_bar_created".to_string()))
            }
            "create_toolbar" => {
                if args.len() >= 5 {
                    Ok(Value::String("toolbar_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_status_bar" => {
                if args.len() >= 5 {
                    Ok(Value::String("status_bar_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_tab_view" => {
                if args.len() >= 5 {
                    Ok(Value::String("tab_view_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_progress_bar" => {
                if args.len() >= 5 {
                    Ok(Value::String("progress_bar_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_image_view" => {
                if args.len() >= 5 {
                    Ok(Value::String("image_view_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }

            // Component Management
            "add_component_to_window" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "remove_component_from_window" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // Event Handling
            "add_event_handler" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "remove_event_handler" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "trigger_event" => {
                if args.len() >= 3 {
                    Ok(Value::String("event_triggered".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() })
                }
            }

            // Dialogs and System Integration
            "show_file_dialog" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("file_dialog_shown".to_string()))
            }
            "show_save_dialog" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("save_dialog_shown".to_string()))
            }
            "show_message_dialog" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("message_dialog_shown".to_string()))
            }
            "create_system_tray_icon" => {
                if args.len() >= 2 {
                    Ok(Value::String("system_tray_icon_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() })
                }
            }
            "show_notification" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // Theming and Styling
            "create_theme" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let name = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("theme_created_{}", name)))
            }
            "apply_theme_to_window" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "apply_theme_to_component" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("theme_applied".to_string()))
            }

            // Application Lifecycle
            "run_event_loop" => {
                Ok(Value::String("event_loop_started".to_string()))
            }
            "exit_application" => {
                Ok(Value::String("application_exited".to_string()))
            }

            _ => Err(RuntimeError::function_not_found(format!("desktop::{}", name))),
        }
    }

    fn call_mobile_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            // === PHASE 5: MOBILE FUNCTIONS ===

            // Application Management
            "create_app" => {
                if args.len() != 3 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() });
                }
                let name = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("app_created_{}", name)))
            }
            "add_screen_to_app" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "set_root_screen" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "push_screen" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "pop_screen" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("screen_popped".to_string()))
            }

            // Screen Management
            "create_screen" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let title = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("screen_created_{}", title)))
            }
            "add_component_to_screen" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // UI Component Creation
            "create_mobile_label" => {
                if args.len() >= 5 {
                    let text = self.value_to_string(&args[0])?;
                    Ok(Value::String(format!("mobile_label_created_{}", text)))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_mobile_button" => {
                if args.len() >= 5 {
                    let title = self.value_to_string(&args[0])?;
                    Ok(Value::String(format!("mobile_button_created_{}", title)))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_mobile_text_field" => {
                if args.len() >= 5 {
                    let placeholder = self.value_to_string(&args[0])?;
                    Ok(Value::String(format!("mobile_text_field_created_{}", placeholder)))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_mobile_image_view" => {
                if args.len() >= 5 {
                    Ok(Value::String("mobile_image_view_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_mobile_list_view" => {
                if args.len() >= 5 {
                    Ok(Value::String("mobile_list_view_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_mobile_map_view" => {
                if args.len() >= 5 {
                    Ok(Value::String("mobile_map_view_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }
            "create_mobile_web_view" => {
                if args.len() >= 5 {
                    Ok(Value::String("mobile_web_view_created".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 5, got: args.len() })
                }
            }

            // Device Hardware Integration
            "get_camera" => {
                Ok(Value::String("camera_accessed".to_string()))
            }
            "capture_photo" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("photo_captured".to_string()))
            }
            "get_gps_location" => {
                Ok(Value::String("gps_location_retrieved".to_string()))
            }
            "get_accelerometer_data" => {
                Ok(Value::String("accelerometer_data_retrieved".to_string()))
            }
            "get_gyroscope_data" => {
                Ok(Value::String("gyroscope_data_retrieved".to_string()))
            }

            // Notifications
            "send_push_notification" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "schedule_local_notification" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // App Permissions
            "request_permission" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                let permission = self.value_to_string(&args[0])?;
                Ok(Value::String(format!("permission_requested_{}", permission)))
            }
            "check_permission_status" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("permission_granted".to_string()))
            }

            // Mobile Wallet Integration
            "create_mobile_wallet" => {
                Ok(Value::String("mobile_wallet_created".to_string()))
            }
            "scan_qr_code" => {
                Ok(Value::String("qr_code_scanned".to_string()))
            }
            "perform_nfc_scan" => {
                Ok(Value::String("nfc_scan_performed".to_string()))
            }

            // App Store Integration
            "check_for_updates" => {
                Ok(Value::Bool(false))
            }
            "rate_app" => {
                if args.len() >= 1 {
                    Ok(Value::Bool(true))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() })
                }
            }

            // App Lifecycle
            "run_mobile_app" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("mobile_app_started".to_string()))
            }
            "terminate_mobile_app" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("mobile_app_terminated".to_string()))
            }

            _ => Err(RuntimeError::function_not_found(format!("mobile::{}", name))),
        }
    }

    fn execute_spawn_statement(&mut self, spawn_stmt: &crate::parser::ast::SpawnStatement) -> Result<Value, RuntimeError> {
        // Check if this is an AI agent spawn
        if let Some(agent_type) = &spawn_stmt.agent_type {
            if agent_type == "ai" {
                return self.execute_ai_agent_spawn(spawn_stmt);
            }
        }

        // For non-AI spawns, create a new execution context
        crate::stdlib::log::info("Executing spawn statement", {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_name".to_string(), Value::String(spawn_stmt.agent_name.clone()));
            data.insert("agent_type".to_string(), Value::String(spawn_stmt.agent_type.as_ref().unwrap_or(&"generic".to_string()).clone()));
            data.insert("message".to_string(), Value::String("Executing spawn statement".to_string()));
            data
        }, Some("runtime"));

        // Create new scope for the spawned agent
        let parent_scope = self.scope.clone();
        self.scope = Scope::new();

        // Execute the spawn block in new scope
        let result = self.execute_statement(&crate::parser::ast::Statement::Block(spawn_stmt.body.clone()));

        // Restore parent scope
        self.scope = parent_scope;

        result
    }

    fn execute_agent_statement(&mut self, agent_stmt: &crate::parser::ast::AgentStatement) -> Result<Value, RuntimeError> {
        // Handle different agent types
        match &agent_stmt.agent_type {
            crate::parser::ast::AgentType::AI => {
                self.execute_ai_agent_declaration(agent_stmt)
            }
            crate::parser::ast::AgentType::System => {
                self.execute_system_agent_declaration(agent_stmt)
            }
            crate::parser::ast::AgentType::Worker => {
                self.execute_worker_agent_declaration(agent_stmt)
            }
            crate::parser::ast::AgentType::Custom(custom_type) => {
                self.execute_custom_agent_declaration(agent_stmt, custom_type)
            }
        }
    }

    fn execute_ai_agent_spawn(&mut self, spawn_stmt: &crate::parser::ast::SpawnStatement) -> Result<Value, RuntimeError> {
        // Create AI agent configuration from spawn parameters
        let mut agent_config = crate::stdlib::ai::AgentConfig {
            agent_id: spawn_stmt.agent_name.clone(),
            name: spawn_stmt.agent_name.clone(),
            role: "spawned_agent".to_string(),
            capabilities: vec!["execution".to_string()],
            memory_size: 1000,
            max_concurrent_tasks: 5,
            trust_level: "hybrid".to_string(),
            communication_protocols: vec!["async".to_string()],
            ai_models: vec!["default".to_string()],
        };

        // Override config with spawn configuration if provided
        if let Some(config) = &spawn_stmt.config {
            for (key, expr) in config {
                let value = self.evaluate_expression(expr)?;
                match key.as_str() {
                    "role" => {
                        if let Value::String(role) = value {
                            agent_config.role = role;
                        }
                    }
                    "capabilities" => {
                        if let Value::List(capabilities) = value {
                            agent_config.capabilities = capabilities.into_iter()
                                .filter_map(|v| match v {
                                    Value::String(s) => Some(s),
                                    _ => None,
                                })
                                .collect();
                        }
                    }
                    "memory_size" => {
                        if let Value::Int(size) = value {
                            agent_config.memory_size = size as i64;
                        }
                    }
                    "max_concurrent_tasks" => {
                        if let Value::Int(max) = value {
                            agent_config.max_concurrent_tasks = max as i64;
                        }
                    }
                    "trust_level" => {
                        if let Value::String(level) = value {
                            agent_config.trust_level = level;
                        }
                    }
                    "ai_models" => {
                        if let Value::List(models) = value {
                            agent_config.ai_models = models.into_iter()
                                .filter_map(|v| match v {
                                    Value::String(s) => Some(s),
                                    _ => None,
                                })
                                .collect();
                        }
                    }
                    _ => {} // Ignore unknown config keys
                }
            }
        }

        // Spawn the AI agent
        match crate::stdlib::ai::spawn_agent(agent_config) {
            Ok(agent) => {
                // Store agent in current scope
                self.scope.set(spawn_stmt.agent_name.clone(), Value::String(format!("agent_{}", agent.id)));

                crate::stdlib::log::info("AI agent spawned successfully", {
                    let mut data = std::collections::HashMap::new();
                    data.insert("agent_id".to_string(), Value::String(agent.id.clone()));
                    data.insert("agent_name".to_string(), Value::String(spawn_stmt.agent_name.clone()));
                    data.insert("message".to_string(), Value::String("AI agent spawned successfully".to_string()));
                    data
                }, Some("ai"));

                // Execute agent body in new scope
                let parent_scope = self.scope.clone();
                self.scope = Scope::new();
                self.scope.set("agent".to_string(), Value::String(format!("agent_{}", agent.id)));

                let result = self.execute_statement(&crate::parser::ast::Statement::Block(spawn_stmt.body.clone()));

                // Restore parent scope
                self.scope = parent_scope;

                result
            }
            Err(err) => {
                Err(RuntimeError::General(format!("Failed to spawn AI agent: {}", err)))
            }
        }
    }

    fn execute_ai_agent_declaration(&mut self, agent_stmt: &crate::parser::ast::AgentStatement) -> Result<Value, RuntimeError> {
        // Create AI agent configuration from agent declaration
        let mut agent_config = crate::stdlib::ai::AgentConfig {
            agent_id: agent_stmt.name.clone(),
            name: agent_stmt.name.clone(),
            role: "declared_agent".to_string(),
            capabilities: agent_stmt.capabilities.clone(),
            memory_size: 1000,
            max_concurrent_tasks: 5,
            trust_level: "hybrid".to_string(),
            communication_protocols: vec!["async".to_string()],
            ai_models: vec!["default".to_string()],
        };

        // Override config with agent configuration
        for (key, expr) in &agent_stmt.config {
            let value = self.evaluate_expression(expr)?;
            match key.as_str() {
                "role" => {
                    if let Value::String(role) = value {
                        agent_config.role = role;
                    }
                }
                "memory_size" => {
                    if let Value::Int(size) = value {
                        agent_config.memory_size = size as i64;
                    }
                }
                "max_concurrent_tasks" => {
                    if let Value::Int(max) = value {
                        agent_config.max_concurrent_tasks = max as i64;
                    }
                }
                "trust_level" => {
                    if let Value::String(level) = value {
                        agent_config.trust_level = level;
                    }
                }
                "communication_protocols" => {
                    if let Value::List(protocols) = value {
                        agent_config.communication_protocols = protocols.into_iter()
                            .filter_map(|v| match v {
                                Value::String(s) => Some(s),
                                _ => None,
                            })
                            .collect();
                    }
                }
                "ai_models" => {
                    if let Value::List(models) = value {
                        agent_config.ai_models = models.into_iter()
                            .filter_map(|v| match v {
                                Value::String(s) => Some(s),
                                _ => None,
                            })
                            .collect();
                    }
                }
                _ => {} // Ignore unknown config keys
            }
        }

        // Spawn the AI agent
        match crate::stdlib::ai::spawn_agent(agent_config) {
            Ok(agent) => {
                // Store agent in current scope
                self.scope.set(agent_stmt.name.clone(), Value::String(format!("agent_{}", agent.id)));

                crate::stdlib::log::info("AI agent declared and spawned", {
                    let mut data = std::collections::HashMap::new();
                    data.insert("agent_id".to_string(), Value::String(agent.id.clone()));
                    data.insert("agent_name".to_string(), Value::String(agent_stmt.name.clone()));
                    data.insert("capabilities".to_string(), Value::String(agent_stmt.capabilities.join(", ")));
                    data.insert("message".to_string(), Value::String("AI agent declared and spawned".to_string()));
                    data
                }, Some("ai"));

                // Execute agent body in new scope
                let parent_scope = self.scope.clone();
                self.scope = Scope::new();
                self.scope.set("agent".to_string(), Value::String(format!("agent_{}", agent.id)));
                self.scope.set("agent_config".to_string(), Value::String(format!("config_{}", agent.id)));

                let result = self.execute_statement(&crate::parser::ast::Statement::Block(agent_stmt.body.clone()));

                // Restore parent scope
                self.scope = parent_scope;

                result
            }
            Err(err) => {
                Err(RuntimeError::General(format!("Failed to declare AI agent: {}", err)))
            }
        }
    }

    fn execute_system_agent_declaration(&mut self, agent_stmt: &crate::parser::ast::AgentStatement) -> Result<Value, RuntimeError> {
        crate::stdlib::log::info("Executing system agent declaration", {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_name".to_string(), Value::String(agent_stmt.name.clone()));
            data.insert("agent_type".to_string(), Value::String("system".to_string()));
            data.insert("message".to_string(), Value::String("Executing system agent declaration".to_string()));
            data
        }, Some("runtime"));

        // System agents run in isolated scopes
        let parent_scope = self.scope.clone();
        self.scope = Scope::new();
        self.scope.set("agent_type".to_string(), Value::String("system".to_string()));

        let result = self.execute_statement(&crate::parser::ast::Statement::Block(agent_stmt.body.clone()));

        // Restore parent scope
        self.scope = parent_scope;

        result
    }

    fn execute_worker_agent_declaration(&mut self, agent_stmt: &crate::parser::ast::AgentStatement) -> Result<Value, RuntimeError> {
        crate::stdlib::log::info("Executing worker agent declaration", {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_name".to_string(), Value::String(agent_stmt.name.clone()));
            data.insert("agent_type".to_string(), Value::String("worker".to_string()));
            data.insert("message".to_string(), Value::String("Executing worker agent declaration".to_string()));
            data
        }, Some("runtime"));

        // Worker agents have access to parent scope but run in separate context
        self.scope.set("agent_type".to_string(), Value::String("worker".to_string()));

        self.execute_statement(&crate::parser::ast::Statement::Block(agent_stmt.body.clone()))
    }

    fn execute_custom_agent_declaration(&mut self, agent_stmt: &crate::parser::ast::AgentStatement, custom_type: &str) -> Result<Value, RuntimeError> {
        crate::stdlib::log::info("runtime", {
            let mut data = std::collections::HashMap::new();
            data.insert("agent_name".to_string(), Value::String(agent_stmt.name.clone()));
            data.insert("agent_type".to_string(), Value::String(format!("custom:{}", custom_type)));
            data.insert("message".to_string(), Value::String("Executing custom agent declaration".to_string()));
            data
        }, Some("runtime"));

        // Custom agents can define their own execution model
        self.scope.set("agent_type".to_string(), Value::String(format!("custom:{}", custom_type)));
        self.scope.set("custom_type".to_string(), Value::String(custom_type.to_string()));

        self.execute_statement(&crate::parser::ast::Statement::Block(agent_stmt.body.clone()))
    }

    fn call_iot_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        match name {
            // === PHASE 6: IOT & EDGE COMPUTING FUNCTIONS ===

            // Device Management
            "register_device" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("device_registered".to_string()))
            }
            "connect_device" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("device_connected".to_string()))
            }
            "disconnect_device" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "get_device_status" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("online".to_string()))
            }
            "update_device_firmware" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // Sensor Management
            "add_sensor_to_device" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("sensor_added".to_string()))
            }
            "read_sensor_data" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("sensor_data_read".to_string()))
            }
            "calibrate_sensor" => {
                if args.len() >= 2 {
                    Ok(Value::Bool(true))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() })
                }
            }

            // Actuator Control
            "add_actuator_to_device" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("actuator_added".to_string()))
            }
            "send_actuator_command" => {
                if args.len() >= 3 {
                    Ok(Value::String("command_sent".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() })
                }
            }

            // Edge Computing
            "create_edge_node" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("edge_node_created".to_string()))
            }
            "process_data_at_edge" => {
                if args.len() >= 3 {
                    Ok(Value::String("data_processed".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() })
                }
            }
            "cache_data_at_edge" => {
                if args.len() >= 4 {
                    Ok(Value::Bool(true))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 4, got: args.len() })
                }
            }
            "get_cached_data_from_edge" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("cached_data".to_string()))
            }

            // Data Streaming
            "create_data_stream" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("data_stream_created".to_string()))
            }
            "add_filter_to_stream" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "add_processor_to_stream" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "add_sink_to_stream" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // Protocol Support
            "configure_protocol" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "publish_message" => {
                if args.len() != 3 { return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "subscribe_to_topic" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // Security Functions
            "authenticate_device" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "encrypt_device_data" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("encrypted_data".to_string()))
            }
            "verify_device_certificate" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }

            // Cloud Integration
            "sync_device_data_to_cloud" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Bool(true))
            }
            "get_device_data_from_cloud" => {
                if args.len() >= 1 {
                    Ok(Value::String("cloud_data".to_string()))
                } else {
                    Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() })
                }
            }

            // Anomaly Detection
            "detect_sensor_anomalies" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("anomalies_detected".to_string()))
            }
            "predict_device_failure" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::Float(0.15))
            }

            // Power Management
            "monitor_power_consumption" => {
                if args.len() != 1 { return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() }); }
                Ok(Value::String("power_status".to_string()))
            }
            "optimize_power_usage" => {
                if args.len() != 2 { return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() }); }
                Ok(Value::String("power_optimized".to_string()))
            }

            _ => Err(RuntimeError::function_not_found(format!("iot::{}", name))),
        }
    }

    pub fn execute_expression(&mut self, tokens: &[crate::lexer::tokens::Token]) -> Result<Value, RuntimeError> {
        // For now, implement a simple expression evaluator
        // This will be expanded in the parser phase
        
        if tokens.is_empty() {
            return Err(RuntimeError::General("Empty expression".to_string()));
        }
        
        // Simple literal evaluation for now
        match &tokens[0] {
            crate::lexer::tokens::Token::Literal(literal) => {
                match literal {
                    crate::lexer::tokens::Literal::Int(i) => Ok(Value::Int(*i)),
                    crate::lexer::tokens::Literal::Float(f) => Ok(Value::Float(*f)),
                    crate::lexer::tokens::Literal::String(s) => Ok(Value::String(s.clone())),
                    crate::lexer::tokens::Literal::Bool(b) => Ok(Value::Bool(*b)),
                    crate::lexer::tokens::Literal::Null => Ok(Value::Null),
                }
            }
            crate::lexer::tokens::Token::Identifier(name) => {
                self.get_variable(name)
            }
            _ => Err(RuntimeError::General("Unsupported expression".to_string())),
        }
    }

    // NEW: Service execution method
    fn execute_service_statement(&mut self, service_stmt: &ServiceStatement) -> Result<Value, RuntimeError> {
        // Create service instance
        let mut service_instance = ServiceInstance {
            name: service_stmt.name.clone(),
            fields: HashMap::new(),
            methods: service_stmt.methods.clone(),
            events: service_stmt.events.clone(),
        };
        
        // Initialize fields
        for field in &service_stmt.fields {
            let initial_value = if let Some(ref value) = field.initial_value {
                self.evaluate_expression(value)?
            } else {
                self.get_default_value(&field.field_type)?
            };
            
            service_instance.fields.insert(field.name.clone(), initial_value);
        }
        
        // Store service in runtime
        self.services.insert(service_stmt.name.clone(), service_instance);
        
        // Set service reference in current scope
        self.set_variable(format!("service_{}", service_stmt.name), Value::String(format!("service_{}", service_stmt.name)));
        
        Ok(Value::String(format!("service_{}", service_stmt.name)))
    }
    
    // Helper method to get default values for field types
    fn get_default_value(&self, field_type: &str) -> Result<Value, RuntimeError> {
        match field_type {
            "int" => Ok(Value::Int(0)),
            "string" => Ok(Value::String("".to_string())),
            "bool" => Ok(Value::Bool(false)),
            "float" => Ok(Value::Float(0.0)),
            _ => Ok(Value::Null),
        }
    }

    // Execute a service method with instance context
    fn execute_service_method(
        &mut self,
        instance_id: &str,
        _method_name: &str,
        method: &crate::parser::ast::FunctionStatement,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        // Validate argument count
        if args.len() != method.parameters.len() {
            return Err(RuntimeError::ArgumentCountMismatch {
                expected: method.parameters.len(),
                got: args.len(),
            });
        }

        // Save current scope
        let saved_scope = self.scope.clone();
        let call_frame = CallFrame {
            scope: saved_scope.clone(),
        };
        self.call_stack.push(call_frame);

        // Create new scope for method execution
        self.scope = Scope::new();

        // Bind method parameters
        for (param, arg_value) in method.parameters.iter().zip(args.iter()) {
            self.scope.set(param.name.clone(), arg_value.clone());
        }

        // Set up 'self' to reference the instance ID
        self.scope.set("self".to_string(), Value::String(instance_id.to_string()));

        // Execute method body
        let mut result = Value::Null;
        for stmt in &method.body.statements {
            match self.execute_statement(stmt) {
                Ok(value) => {
                    // Check if this is a return statement
                    if let crate::parser::ast::Statement::Return(_) = stmt {
                        result = value;
                        break;
                    }
                }
                Err(e) => {
                    // Restore scope on error
                    if let Some(frame) = self.call_stack.pop() {
                        self.scope = frame.scope;
                    }
                    return Err(e);
                }
            }
        }

        // Restore scope
        if let Some(frame) = self.call_stack.pop() {
            self.scope = frame.scope;
        }

        Ok(result)
    }

    // Helper method to parse agent configuration from Value
    fn parse_agent_config(&self, value: &Value) -> Result<crate::stdlib::agent::AgentConfig, RuntimeError> {
        match value {
            Value::Struct(_, fields) => {
                let name = if let Some(Value::String(name)) = fields.get("name") {
                    name.clone()
                } else {
                    "default_agent".to_string()
                };

                let agent_type_str = if let Some(Value::String(agent_type)) = fields.get("type") {
                    agent_type.clone()
                } else {
                    "ai".to_string()
                };

                let agent_type = crate::stdlib::agent::AgentType::from_string(&agent_type_str)
                    .ok_or_else(|| RuntimeError::General(format!("Invalid agent type: {}", agent_type_str)))?;

                let mut config = crate::stdlib::agent::AgentConfig::new(name, agent_type);

                // Parse role if provided
                if let Some(Value::String(role)) = fields.get("role") {
                    config = config.with_role(role.clone());
                }

                // Parse capabilities if provided
                if let Some(Value::List(capabilities)) = fields.get("capabilities") {
                    let capability_strings: Vec<String> = capabilities
                        .iter()
                        .filter_map(|cap| {
                            if let Value::String(cap_str) = cap {
                                Some(cap_str.clone())
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !capability_strings.is_empty() {
                        config = config.with_capabilities(capability_strings);
                    }
                }

                // Parse trust level if provided
                if let Some(Value::String(trust_level)) = fields.get("trust_level") {
                    config = config.with_trust_level(trust_level.clone());
                }

                // Parse metadata if provided
                if let Some(Value::Struct(_, metadata)) = fields.get("metadata") {
                    config = config.with_metadata(metadata.clone());
                }

                Ok(config)
            }
            _ => Err(RuntimeError::General("Agent config must be a struct".to_string()))
        }
    }

    fn call_admin_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        // Validate admin access based on current trust context
        if !self.validate_admin_access() {
            return Err(RuntimeError::PermissionDenied("Admin access denied".to_string()));
        }

        match name {
            "kill" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let process_id = self.value_to_string(&args[0])?;
                let reason = self.value_to_string(&args[1])?;
                
                match crate::stdlib::admin::kill(&process_id, &reason) {
                    Ok(result) => Ok(Value::Bool(result)),
                    Err(e) => Err(RuntimeError::General(e)),
                }
            }
            "get_process_info" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 1, got: args.len() });
                }
                let process_id = self.value_to_string(&args[0])?;
                
                match crate::stdlib::admin::get_process_info(&process_id) {
                    Ok(info) => {
                        let mut data = std::collections::HashMap::new();
                        data.insert("process_id".to_string(), Value::String(info.process_id));
                        data.insert("name".to_string(), Value::String(info.name));
                        data.insert("status".to_string(), Value::String(info.status));
                        data.insert("start_time".to_string(), Value::Int(info.start_time));
                        Ok(Value::Map(data))
                    }
                    Err(e) => Err(RuntimeError::General(e)),
                }
            }
            "list_processes" => {
                if args.len() != 0 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 0, got: args.len() });
                }
                
                let processes = crate::stdlib::admin::list_processes();
                let mut result = Vec::new();
                
                for process in processes {
                    let mut data = std::collections::HashMap::new();
                    data.insert("process_id".to_string(), Value::String(process.process_id));
                    data.insert("name".to_string(), Value::String(process.name));
                    data.insert("status".to_string(), Value::String(process.status));
                    data.insert("start_time".to_string(), Value::Int(process.start_time));
                    result.push(Value::Map(data));
                }
                
                Ok(Value::Array(result))
            }
            _ => Err(RuntimeError::function_not_found(format!("admin::{}", name))),
        }
    }

    fn call_cloudadmin_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        // Validate cloudadmin access based on current trust context
        if !self.validate_cloudadmin_access() {
            return Err(RuntimeError::PermissionDenied("CloudAdmin access denied".to_string()));
        }

        match name {
            "authorize" => {
                if args.len() != 3 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 3, got: args.len() });
                }
                let admin_id = self.value_to_string(&args[0])?;
                let operation = self.value_to_string(&args[1])?;
                let resource = self.value_to_string(&args[2])?;
                
                let result = crate::stdlib::cloudadmin::authorize(&admin_id, &operation, &resource);
                Ok(Value::Bool(result))
            }
            "validate_hybrid_trust" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let admin_trust = self.value_to_string(&args[0])?;
                let user_trust = self.value_to_string(&args[1])?;
                
                let result = crate::stdlib::cloudadmin::validate_hybrid_trust(&admin_trust, &user_trust);
                Ok(Value::Bool(result))
            }
            "bridge_trusts" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArgumentCountMismatch { expected: 2, got: args.len() });
                }
                let centralized_trust = self.value_to_string(&args[0])?;
                let decentralized_trust = self.value_to_string(&args[1])?;
                
                let result = crate::stdlib::cloudadmin::bridge_trusts(&centralized_trust, &decentralized_trust);
                Ok(Value::Bool(result))
            }
            _ => Err(RuntimeError::function_not_found(format!("cloudadmin::{}", name))),
        }
    }

    /// Validate admin access based on current trust context
    fn validate_admin_access(&self) -> bool {
        // Check if current context has admin privileges
        // This would typically check against the current service's trust model
        if let Some(current_service) = &self.current_service {
            // For centralized trust, admin access is always allowed
            if current_service.trust_model == "centralized" {
                return true;
            }
            
            // For hybrid trust, check if admin operations are permitted
            if current_service.trust_model == "hybrid" {
                // In hybrid mode, admin operations require special validation
                return current_service.has_admin_privileges;
            }
            
            // For decentralized trust, admin operations are restricted
            if current_service.trust_model == "decentralized" {
                return false;
            }
        }
        
        // Default: no admin access
        false
    }

    /// Validate cloudadmin access based on current trust context
    fn validate_cloudadmin_access(&self) -> bool {
        // Check if current context has cloudadmin privileges
        if let Some(current_service) = &self.current_service {
            // For centralized trust, cloudadmin access is always allowed
            if current_service.trust_model == "centralized" {
                return true;
            }
            
            // For hybrid trust, cloudadmin operations require special validation
            if current_service.trust_model == "hybrid" {
                return current_service.has_cloudadmin_privileges;
            }
            
            // For decentralized trust, cloudadmin operations are restricted
            if current_service.trust_model == "decentralized" {
                return false;
            }
        }
        
        // Default: no cloudadmin access
        false
    }

    /// Validate trust model for web operations
    fn validate_web_trust(&self) -> bool {
        if let Some(current_service) = &self.current_service {
            // Web operations require appropriate trust model
            match current_service.trust_model.as_str() {
                "centralized" => true,  // Centralized allows all web operations
                "hybrid" => current_service.has_web_privileges,  // Hybrid requires web privileges
                "decentralized" => current_service.has_web_privileges,  // Decentralized requires explicit web privileges
                _ => false,
            }
        } else {
            false
        }
    }

    /// Validate trust model for AI operations
    fn validate_ai_trust(&self) -> bool {
        if let Some(current_service) = &self.current_service {
            // AI operations require appropriate trust model
            match current_service.trust_model.as_str() {
                "centralized" => true,  // Centralized allows all AI operations
                "hybrid" => current_service.has_ai_privileges,  // Hybrid requires AI privileges
                "decentralized" => current_service.has_ai_privileges,  // Decentralized requires explicit AI privileges
                _ => false,
            }
        } else {
            false
        }
    }

    /// Validate trust model for blockchain operations
    fn validate_chain_trust(&self) -> bool {
        if let Some(current_service) = &self.current_service {
            // Blockchain operations require appropriate trust model
            match current_service.trust_model.as_str() {
                "centralized" => true,  // Centralized allows all chain operations
                "hybrid" => current_service.has_chain_privileges,  // Hybrid requires chain privileges
                "decentralized" => current_service.has_chain_privileges,  // Decentralized requires explicit chain privileges
                _ => false,
            }
        } else {
            false
        }
    }

    /// Set the current service context for trust validation
    pub fn set_current_service(&mut self, service_name: String, attributes: Vec<String>) {
        // Parse attributes to determine trust model and privileges
        let mut trust_model = "decentralized".to_string(); // Default
        let mut has_admin_privileges = false;
        let mut has_cloudadmin_privileges = false;
        let mut has_web_privileges = false;
        let mut has_ai_privileges = false;
        let mut has_chain_privileges = false;

        for attr in &attributes {
            if attr.starts_with("@trust(") {
                // Extract trust model from @trust("model")
                if let Some(model) = attr.strip_prefix("@trust(\"").and_then(|s| s.strip_suffix("\"")) {
                    trust_model = model.to_string();
                }
            } else if attr == "@web" {
                has_web_privileges = true;
            } else if attr == "@ai" {
                has_ai_privileges = true;
            } else if attr.starts_with("@chain(") {
                has_chain_privileges = true;
            }
        }

        // Set admin/cloudadmin privileges based on trust model
        match trust_model.as_str() {
            "centralized" => {
                has_admin_privileges = true;
                has_cloudadmin_privileges = true;
            }
            "hybrid" => {
                // In hybrid mode, check if admin/cloudadmin attributes are present
                has_admin_privileges = attributes.iter().any(|a| a == "@admin");
                has_cloudadmin_privileges = attributes.iter().any(|a| a == "@cloudadmin");
            }
            "decentralized" => {
                // In decentralized mode, admin/cloudadmin operations are restricted
                has_admin_privileges = false;
                has_cloudadmin_privileges = false;
            }
            _ => {}
        }

        self.current_service = Some(ServiceContext {
            name: service_name,
            trust_model,
            has_admin_privileges,
            has_cloudadmin_privileges,
            has_web_privileges,
            has_ai_privileges,
            has_chain_privileges,
            attributes,
        });
    }

    /// Clear the current service context
    pub fn clear_current_service(&mut self) {
        self.current_service = None;
    }
}

// Helper function to generate unique IDs
fn generate_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

// Use the proper RuntimeError variants
impl RuntimeError {

}

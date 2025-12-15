use crate::runtime::values::Value;
use crate::runtime::scope::Scope;

pub type FunctionBody = Box<dyn Fn(&[Value], &mut Scope) -> Result<Value, RuntimeError>>;

pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: FunctionBody,
}

impl Function {
    pub fn new<F>(name: String, parameters: Vec<String>, body: F) -> Self 
    where
        F: Fn(&[Value], &mut Scope) -> Result<Value, RuntimeError> + 'static,
    {
        Self {
            name,
            parameters,
            body: Box::new(body),
        }
    }

    pub fn call(&self, args: &[Value], scope: &mut Scope) -> Result<Value, RuntimeError> {
        // Check argument count
        if args.len() != self.parameters.len() {
            return Err(RuntimeError::ArgumentCountMismatch {
                expected: self.parameters.len(),
                got: args.len(),
            });
        }

        // Create new scope for function execution
        let mut function_scope = Scope::new_child(scope.clone());
        
        // Bind parameters to arguments
        for (param, arg) in self.parameters.iter().zip(args.iter()) {
            function_scope.set(param.clone(), arg.clone());
        }

        // Execute function body
        (self.body)(args, &mut function_scope)
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        // For now, we'll create a simple clone that can't be called
        // In a real implementation, you'd need to handle the function body cloning
        Self {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            body: Box::new(|_, _| Err(RuntimeError::FunctionNotClonable)),
        }
    }
}



#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Argument count mismatch: expected {expected}, got {got}")]
    ArgumentCountMismatch { expected: usize, got: usize },
    
    #[error("Function not clonable")]
    FunctionNotClonable,
    
    #[error("Variable '{0}' not found")]
    VariableNotFound(String),
    
    #[error("Type error: expected {expected}, got {got}")]
    TypeError { expected: String, got: String },
    
    #[error("Stack underflow")]
    StackUnderflow,
    
    #[error("Function '{0}' not found")]
    FunctionNotFound(String),
    
    #[error("Runtime error: {0}")]
    General(String),
    
    #[error("Type mismatch in {0}")]
    TypeMismatch(String),
    
    #[error("Division by zero")]
    DivisionByZero,
    
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Re-entrancy detected: {0}")]
    ReentrancyDetected(String),
    
    #[error("Integer overflow")]
    IntegerOverflow,
    
    #[error("Integer underflow")]
    IntegerUnderflow,
    
    #[error("Read-only violation")]
    ReadOnlyViolation,
    
    #[error("Access denied")]
    AccessDenied,
}

impl RuntimeError {
    pub fn stack_underflow() -> Self {
        RuntimeError::StackUnderflow
    }
    
    pub fn function_not_found(name: String) -> Self {
        RuntimeError::FunctionNotFound(name)
    }
}



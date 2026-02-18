use crate::runtime::scope::Scope;
use crate::runtime::values::Value;

pub type FunctionBody = Box<dyn Fn(&[Value], &mut Scope) -> Result<Value, RuntimeError>>;

pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    /// None for cloned built-ins; caller must pass body_override when invoking (e.g. engine resolves by name).
    pub body: Option<FunctionBody>,
}

impl Function {
    pub fn new<F>(name: String, parameters: Vec<String>, body: F) -> Self
    where
        F: Fn(&[Value], &mut Scope) -> Result<Value, RuntimeError> + 'static,
    {
        Self {
            name,
            parameters,
            body: Some(Box::new(body)),
        }
    }

    /// Reference to the body, if present (not a clone).
    pub fn body_ref(&self) -> Option<&FunctionBody> {
        self.body.as_ref()
    }

    /// Call with optional body override so cloned functions can be invoked when the engine resolves by name.
    pub fn call(
        &self,
        args: &[Value],
        scope: &mut Scope,
        body_override: Option<&FunctionBody>,
    ) -> Result<Value, RuntimeError> {
        if args.len() != self.parameters.len() {
            return Err(RuntimeError::ArgumentCountMismatch {
                expected: self.parameters.len(),
                got: args.len(),
            });
        }

        let body = body_override
            .or(self.body.as_ref())
            .ok_or(RuntimeError::FunctionNotClonable)?;

        let mut function_scope = Scope::new_child(scope.clone());
        for (param, arg) in self.parameters.iter().zip(args.iter()) {
            function_scope.set(param.clone(), arg.clone());
        }

        (body)(args, &mut function_scope)
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            body: None,
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

    #[error("Execution timeout: program exceeded maximum execution time")]
    ExecutionTimeout,
}

impl RuntimeError {
    pub fn stack_underflow() -> Self {
        RuntimeError::StackUnderflow
    }

    pub fn function_not_found(name: String) -> Self {
        RuntimeError::FunctionNotFound(name)
    }
}

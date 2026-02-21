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

/// Source location for error reporting (line/column; optional file path).
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub file_path: Option<String>,
}

/// One frame in the call stack (function name and optional line).
#[derive(Debug, Clone)]
pub struct CallFrameInfo {
    pub function_name: String,
    pub line: Option<usize>,
}

/// Runtime error with optional location, call stack, and suggestions (P5 "did you mean").
#[derive(Debug)]
pub struct RuntimeErrorWithContext {
    pub inner: RuntimeError,
    pub location: Option<SourceLocation>,
    pub call_stack: Vec<CallFrameInfo>,
    pub suggestions: Vec<String>,
}

impl RuntimeErrorWithContext {
    pub fn new(
        inner: RuntimeError,
        location: Option<SourceLocation>,
        call_stack: Vec<CallFrameInfo>,
    ) -> Self {
        Self {
            inner,
            location,
            call_stack,
            suggestions: Vec::new(),
        }
    }

    /// Wrap a plain RuntimeError with no location or call stack (e.g. from run_registered_tests).
    pub fn from_error(inner: RuntimeError) -> Self {
        Self {
            inner,
            location: None,
            call_stack: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn inner(&self) -> &RuntimeError {
        &self.inner
    }

    /// Format for display: message, then "  at line X, column Y" if location present, then call stack, then suggestions.
    pub fn format_display(
        &self,
        _source: Option<&str>,
        file_path: Option<&str>,
    ) -> String {
        let mut out = format!("{}\n", self.inner);
        if let Some(loc) = &self.location {
            let path = loc
                .file_path
                .as_deref()
                .or(file_path)
                .unwrap_or("");
            if path.is_empty() {
                out.push_str(&format!("  at line {}, column {}\n", loc.line, loc.column));
            } else {
                out.push_str(&format!(
                    "  at {}:{}:{}\n",
                    path, loc.line, loc.column
                ));
            }
        }
        if !self.call_stack.is_empty() {
            out.push_str("\nCall stack:\n");
            for (i, frame) in self.call_stack.iter().enumerate() {
                let line_info = frame
                    .line
                    .map(|l| format!(" (line {})", l))
                    .unwrap_or_default();
                out.push_str(&format!("  {}: {}{}\n", i, frame.function_name, line_info));
            }
        }
        if !self.suggestions.is_empty() {
            out.push_str("\nSuggestions:\n");
            for s in &self.suggestions {
                out.push_str(&format!("  did you mean '{}'?\n", s));
            }
        }
        out
    }
}

impl std::fmt::Display for RuntimeErrorWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for RuntimeErrorWithContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

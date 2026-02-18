use crate::runtime::values::Value;

/// Control flow result for loop statements (break/continue)
/// This replaces the string-based hack with a proper type-safe mechanism
#[derive(Debug, Clone)]
pub enum ControlFlow {
    /// Normal execution - continue processing
    Continue,
    /// Break out of the innermost loop, optionally with a return value
    Break(Option<Value>),
    /// Skip to next iteration of the innermost loop
    Next,
}

impl ControlFlow {
    /// Check if execution should continue normally
    pub fn is_continue(&self) -> bool {
        matches!(self, ControlFlow::Continue)
    }

    /// Check if execution should break
    pub fn is_break(&self) -> bool {
        matches!(self, ControlFlow::Break(_))
    }

    /// Check if execution should skip to next iteration
    pub fn is_next(&self) -> bool {
        matches!(self, ControlFlow::Next)
    }

    /// Extract the break value if this is a Break variant
    pub fn break_value(self) -> Option<Value> {
        match self {
            ControlFlow::Break(value) => value,
            _ => None,
        }
    }
}

/// Result type for statement execution that can signal control flow
pub type StatementResult = Result<StatementOutcome, crate::runtime::functions::RuntimeError>;

/// Outcome of executing a statement
#[derive(Debug, Clone)]
pub enum StatementOutcome {
    /// Normal execution with a value
    Value(Value),
    /// Control flow signal (break/continue)
    ControlFlow(ControlFlow),
}

impl StatementOutcome {
    /// Create a normal value outcome
    pub fn value(v: Value) -> Self {
        StatementOutcome::Value(v)
    }

    /// Create a break outcome
    pub fn break_with_value(v: Option<Value>) -> Self {
        StatementOutcome::ControlFlow(ControlFlow::Break(v))
    }

    /// Create a continue outcome
    pub fn next() -> Self {
        StatementOutcome::ControlFlow(ControlFlow::Next)
    }

    /// Extract the value if this is a Value outcome, otherwise return None
    pub fn as_value(&self) -> Option<&Value> {
        match self {
            StatementOutcome::Value(v) => Some(v),
            _ => None,
        }
    }

    /// Extract control flow if this is a ControlFlow outcome
    pub fn as_control_flow(&self) -> Option<&ControlFlow> {
        match self {
            StatementOutcome::ControlFlow(cf) => Some(cf),
            _ => None,
        }
    }

    /// Check if this is a control flow signal
    pub fn is_control_flow(&self) -> bool {
        matches!(self, StatementOutcome::ControlFlow(_))
    }
}

impl From<Value> for StatementOutcome {
    fn from(value: Value) -> Self {
        StatementOutcome::Value(value)
    }
}

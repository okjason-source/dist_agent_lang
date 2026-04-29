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

#[cfg(test)]
mod tests {
    use super::*;

    /// Pins `is_continue` / `is_break` / `is_next` / `break_value` (mutation: constant bool, `None`, deleted arm)
    #[test]
    fn control_flow_predicates_and_break_value() {
        let c = ControlFlow::Continue;
        assert!(c.is_continue());
        assert!(!c.is_break());
        assert!(!c.is_next());
        assert_eq!(c.break_value(), None);

        let b0 = ControlFlow::Break(None);
        assert!(!b0.is_continue());
        assert!(b0.is_break());
        assert!(!b0.is_next());
        assert_eq!(b0.clone().break_value(), None);

        let b1 = ControlFlow::Break(Some(Value::Int(7)));
        assert!(!b1.is_continue());
        assert!(b1.is_break());
        assert!(!b1.is_next());
        assert_eq!(b1.clone().break_value(), Some(Value::Int(7)));

        let n = ControlFlow::Next;
        assert!(!n.is_continue());
        assert!(!n.is_break());
        assert!(n.is_next());
        assert_eq!(n.break_value(), None);
    }

    /// Pins `as_value` / `as_control_flow` / `is_control_flow` (mutation: `None`, deleted arms, constant bool)
    #[test]
    fn statement_outcome_accessors() {
        let v = StatementOutcome::value(Value::String("x".to_string()));
        assert_eq!(v.as_value(), Some(&Value::String("x".to_string())));
        assert!(v.as_control_flow().is_none());
        assert!(!v.is_control_flow());

        let cf = StatementOutcome::ControlFlow(ControlFlow::Continue);
        assert_eq!(cf.as_value(), None);
        assert!(matches!(cf.as_control_flow(), Some(ControlFlow::Continue)));
        assert!(cf.is_control_flow());

        let br = StatementOutcome::break_with_value(Some(Value::Null));
        assert!(br.is_control_flow());
        assert_eq!(
            br.as_control_flow().unwrap().clone().break_value(),
            Some(Value::Null)
        );

        let nx = StatementOutcome::next();
        assert!(nx.is_control_flow());
        assert!(matches!(nx.as_control_flow(), Some(ControlFlow::Next)));
    }
}

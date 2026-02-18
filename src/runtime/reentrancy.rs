use crate::runtime::functions::RuntimeError;
/// Re-entrancy Protection System for DAL Runtime
/// Provides compile-time and runtime guards against re-entrancy attacks
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ReentrancyGuard {
    active_calls: Arc<Mutex<HashSet<String>>>,
    call_stack: Arc<Mutex<Vec<String>>>,
}

impl ReentrancyGuard {
    pub fn new() -> Self {
        Self {
            active_calls: Arc::new(Mutex::new(HashSet::new())),
            call_stack: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Enter a function call with re-entrancy protection
    pub fn enter(
        &self,
        function_name: &str,
        contract_address: Option<&str>,
    ) -> Result<ReentrancyToken, RuntimeError> {
        let call_key = match contract_address {
            Some(addr) => format!("{}::{}", addr, function_name),
            None => function_name.to_string(),
        };

        let mut active_calls = self
            .active_calls
            .lock()
            .map_err(|_| RuntimeError::General("Failed to acquire re-entrancy lock".to_string()))?;

        let mut call_stack = self
            .call_stack
            .lock()
            .map_err(|_| RuntimeError::General("Failed to acquire call stack lock".to_string()))?;

        // Check for re-entrancy
        if active_calls.contains(&call_key) {
            return Err(RuntimeError::ReentrancyDetected(format!(
                "Re-entrancy detected in function: {} (call stack: {:?})",
                call_key, *call_stack
            )));
        }

        // Add to active calls and call stack
        active_calls.insert(call_key.clone());
        call_stack.push(call_key.clone());

        Ok(ReentrancyToken {
            call_key,
            guard: self.clone(),
        })
    }

    /// Exit a function call
    fn exit(&self, call_key: &str) {
        if let (Ok(mut active_calls), Ok(mut call_stack)) =
            (self.active_calls.lock(), self.call_stack.lock())
        {
            active_calls.remove(call_key);
            call_stack.retain(|k| k != call_key);
        }
    }

    /// Get current call stack for debugging
    pub fn get_call_stack(&self) -> Vec<String> {
        self.call_stack
            .lock()
            .map(|stack| stack.clone())
            .unwrap_or_default()
    }

    /// Check if function is currently active
    pub fn is_active(&self, function_name: &str, contract_address: Option<&str>) -> bool {
        let call_key = match contract_address {
            Some(addr) => format!("{}::{}", addr, function_name),
            None => function_name.to_string(),
        };

        self.active_calls
            .lock()
            .map(|active| active.contains(&call_key))
            .unwrap_or(false)
    }
}

/// Token that represents an active function call
/// Automatically releases the re-entrancy guard when dropped
pub struct ReentrancyToken {
    call_key: String,
    guard: ReentrancyGuard,
}

impl Drop for ReentrancyToken {
    fn drop(&mut self) {
        self.guard.exit(&self.call_key);
    }
}

/// Macro for easy re-entrancy protection
#[macro_export]
macro_rules! nonreentrant {
    ($guard:expr, $function_name:expr, $contract:expr, $body:expr) => {{
        let _token = $guard.enter($function_name, $contract)?;
        $body
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_reentrancy_protection() {
        let guard = ReentrancyGuard::new();

        // First call should succeed
        let token1 = guard.enter("test_function", None);
        assert!(token1.is_ok());

        // Second call to same function should fail
        let token2 = guard.enter("test_function", None);
        assert!(token2.is_err());

        // After dropping first token, second call should succeed
        drop(token1);
        let token3 = guard.enter("test_function", None);
        assert!(token3.is_ok());
    }

    #[test]
    fn test_contract_specific_reentrancy() {
        let guard = ReentrancyGuard::new();

        // Same function in different contracts should be allowed
        let token1 = guard.enter("transfer", Some("0x123"));
        let token2 = guard.enter("transfer", Some("0x456"));

        assert!(token1.is_ok());
        assert!(token2.is_ok());

        // Same function in same contract should fail
        let token3 = guard.enter("transfer", Some("0x123"));
        assert!(token3.is_err());
    }

    #[test]
    fn test_call_stack_tracking() {
        let guard = ReentrancyGuard::new();

        let _token1 = guard.enter("function_a", None).unwrap();
        let _token2 = guard.enter("function_b", None).unwrap();

        let stack = guard.get_call_stack();
        assert_eq!(stack.len(), 2);
        assert!(stack.contains(&"function_a".to_string()));
        assert!(stack.contains(&"function_b".to_string()));
    }
}

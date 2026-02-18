// Security pattern conversion utilities

use super::parser::Contract;

/// Security pattern detector and converter
pub struct SecurityConverter;

impl SecurityConverter {
    pub fn new() -> Self {
        Self
    }

    /// Detect security patterns in contract
    pub fn detect_patterns(&self, contract: &Contract) -> Vec<String> {
        let mut patterns = Vec::new();

        // Check for reentrancy patterns
        if self.has_reentrancy_risk(contract) {
            patterns.push("@secure".to_string());
            patterns.push("@reentrancy_guard".to_string());
        }

        // Check for safe math patterns
        if self.uses_arithmetic(contract) {
            patterns.push("@safe_math".to_string());
        }

        // Check for access control
        if self.has_access_control(contract) {
            patterns.push("@admin".to_string());
        }

        patterns
    }

    /// Reentrancy risk: (public|external) + (payable|nonPayable) and body contains external call (.call, .transfer, .call{).
    /// Centralized here for converter and analyzer; DAL runtime has reentrancy protection so this guides attribute suggestions.
    pub fn has_reentrancy_risk(&self, contract: &Contract) -> bool {
        use super::parser::{Mutability, Visibility};
        contract.functions.iter().any(|f| {
            let visible = matches!(f.visibility, Visibility::Public | Visibility::External);
            let state_changing =
                matches!(f.mutability, Mutability::Payable | Mutability::NonPayable);
            let has_external_call = Self::body_has_external_call(f.body.as_deref().unwrap_or(""));
            visible && state_changing && has_external_call
        })
    }

    /// True if body contains .call(, .transfer(, or .call{ (external call patterns).
    fn body_has_external_call(body: &str) -> bool {
        body.contains(".call(") || body.contains(".call{") || body.contains(".transfer(")
    }

    /// Arithmetic usage: scan function bodies for +, -, *, /, ** (and legacy add/sub/mul/div in names).
    pub fn uses_arithmetic(&self, contract: &Contract) -> bool {
        if contract.functions.iter().any(|f| {
            f.name.contains("add")
                || f.name.contains("sub")
                || f.name.contains("mul")
                || f.name.contains("div")
        }) {
            return true;
        }
        for f in &contract.functions {
            if let Some(ref body) = f.body {
                if Self::body_has_arithmetic(body) {
                    return true;
                }
            }
        }
        for v in &contract.state_variables {
            if let Some(ref init) = v.initial_value {
                if Self::body_has_arithmetic(init) {
                    return true;
                }
            }
        }
        false
    }

    /// True if s contains arithmetic operators: +, -, *, /, ** (excluding ->, --, ++).
    fn body_has_arithmetic(s: &str) -> bool {
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if i + 1 < bytes.len() {
                match (bytes[i], bytes[i + 1]) {
                    (b'+', b'+') | (b'-', b'-') | (b'-', b'>') => {
                        i += 2;
                        continue;
                    }
                    (b'*', b'*') => return true,
                    _ => {}
                }
            }
            match bytes[i] {
                b'+' | b'-' | b'*' | b'/' => return true,
                _ => {}
            }
            i += 1;
        }
        false
    }

    fn has_access_control(&self, contract: &Contract) -> bool {
        // Check for owner/access control patterns
        contract.functions.iter().any(|f| {
            f.modifiers.iter().any(|m| {
                m.contains("onlyOwner") || m.contains("onlyAdmin") || m.contains("onlyRole")
            })
        })
    }
}

impl Default for SecurityConverter {
    fn default() -> Self {
        Self::new()
    }
}

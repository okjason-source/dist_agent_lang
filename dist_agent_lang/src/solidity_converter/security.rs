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
    
    fn has_reentrancy_risk(&self, contract: &Contract) -> bool {
        // Simple heuristic: external payable functions
        contract.functions.iter().any(|f| {
            matches!(f.visibility, super::parser::Visibility::Public | super::parser::Visibility::External) &&
            matches!(f.mutability, super::parser::Mutability::Payable)
        })
    }
    
    fn uses_arithmetic(&self, contract: &Contract) -> bool {
        // Check if contract uses arithmetic operations
        // This is a simplified check
        contract.functions.iter().any(|f| {
            f.name.contains("add") || 
            f.name.contains("sub") || 
            f.name.contains("mul") ||
            f.name.contains("div")
        })
    }
    
    fn has_access_control(&self, contract: &Contract) -> bool {
        // Check for owner/access control patterns
        contract.functions.iter().any(|f| {
            f.modifiers.iter().any(|m| 
                m.contains("onlyOwner") || 
                m.contains("onlyAdmin") ||
                m.contains("onlyRole")
            )
        })
    }
}

impl Default for SecurityConverter {
    fn default() -> Self {
        Self::new()
    }
}


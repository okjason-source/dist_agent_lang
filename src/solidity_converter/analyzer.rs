// Conversion Compatibility Analyzer

use super::parser::{Contract, Function, SolidityAST, Visibility};
use super::types::TypeMapper;

/// Analysis Report
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub compatibility_score: f64,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub suggestions: Vec<String>,
    pub unsupported_features: Vec<String>,
    /// Import paths / library identifiers found in the source (e.g. "openzeppelin/...", "@openzeppelin/...").
    pub used_libraries: Vec<String>,
}

/// Conversion Analyzer
pub struct ConversionAnalyzer {
    type_mapper: TypeMapper,
}

impl ConversionAnalyzer {
    pub fn new() -> Self {
        Self {
            type_mapper: TypeMapper::new(),
        }
    }

    /// Analyze Solidity AST for conversion compatibility
    pub fn analyze(&self, ast: SolidityAST) -> Result<AnalysisReport, String> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut suggestions = Vec::new();
        let mut unsupported_features = Vec::new();

        // Check imports for OpenZeppelin / known libraries
        let has_openzeppelin_import = ast.imports.iter().any(|s| {
            let lower = s.to_lowercase();
            lower.contains("openzeppelin") || lower.contains("@openzeppelin")
        });
        if has_openzeppelin_import {
            suggestions.push("File imports OpenZeppelin - DAL has built-in reentrancy protection; modifiers can be mapped to @secure".to_string());
        }

        for contract in &ast.contracts {
            self.analyze_contract(
                contract,
                &mut warnings,
                &mut errors,
                &mut suggestions,
                &mut unsupported_features,
            );
        }

        // Calculate compatibility score
        let compatibility_score =
            self.calculate_compatibility_score(&warnings, &errors, &unsupported_features);

        let used_libraries = ast.imports.clone();

        Ok(AnalysisReport {
            compatibility_score,
            warnings,
            errors,
            suggestions,
            unsupported_features,
            used_libraries,
        })
    }

    fn analyze_contract(
        &self,
        contract: &Contract,
        warnings: &mut Vec<String>,
        errors: &mut Vec<String>,
        suggestions: &mut Vec<String>,
        unsupported_features: &mut Vec<String>,
    ) {
        // Check contract kind
        match contract.kind {
            super::parser::ContractKind::Interface => {
                warnings.push(format!(
                    "Interface '{}' - will be converted to service",
                    contract.name
                ));
            }
            super::parser::ContractKind::Abstract => {
                warnings.push(format!(
                    "Abstract contract '{}' - abstract features may need manual conversion",
                    contract.name
                ));
            }
            super::parser::ContractKind::Library => {
                unsupported_features.push(format!(
                    "Library '{}' - libraries not directly supported",
                    contract.name
                ));
            }
            _ => {}
        }

        // Check inheritance
        if !contract.inheritance.is_empty() {
            warnings.push(format!(
                "Contract '{}' uses inheritance - may need manual review",
                contract.name
            ));
        }

        // Analyze functions
        for func in &contract.functions {
            self.analyze_function(func, warnings, errors, suggestions);
        }

        // Check for common patterns (reentrancy: centralized in security module)
        if super::security::SecurityConverter::new().has_reentrancy_risk(contract) {
            suggestions.push(format!(
                "Contract '{}' should use @secure attribute for reentrancy protection",
                contract.name
            ));
        }

        // Check for OpenZeppelin patterns
        if self.uses_openzeppelin(contract) {
            suggestions.push(format!(
                "Contract '{}' uses OpenZeppelin - consider using DAL built-in security features",
                contract.name
            ));
        }
    }

    fn analyze_function(
        &self,
        func: &Function,
        warnings: &mut Vec<String>,
        _errors: &mut Vec<String>,
        suggestions: &mut Vec<String>,
    ) {
        // Check visibility
        if matches!(func.visibility, Visibility::External) {
            warnings.push(format!(
                "Function '{}' is external - will be converted to @public",
                func.name
            ));
        }

        // Check mutability
        if matches!(func.mutability, super::parser::Mutability::Payable) {
            suggestions.push(format!(
                "Function '{}' is payable - ensure proper handling in DAL",
                func.name
            ));
        }

        // Check modifiers
        if !func.modifiers.is_empty() {
            warnings.push(format!(
                "Function '{}' uses modifiers - may need manual conversion",
                func.name
            ));
        }

        // Check return types
        for ret in &func.returns {
            if !self.type_mapper.is_supported(&ret.param_type) {
                warnings.push(format!(
                    "Function '{}' returns unsupported type '{}'",
                    func.name, ret.param_type
                ));
            }
        }

        // Check parameters
        for param in &func.parameters {
            if !self.type_mapper.is_supported(&param.param_type) {
                warnings.push(format!(
                    "Function '{}' parameter '{}' has unsupported type '{}'",
                    func.name, param.name, param.param_type
                ));
            }
        }
    }

    fn uses_openzeppelin(&self, contract: &Contract) -> bool {
        // Check modifiers for OpenZeppelin patterns; imports are checked in analyze() via ast.imports
        contract.functions.iter().any(|f| {
            f.modifiers.iter().any(|m| {
                m.contains("onlyOwner") || m.contains("nonReentrant") || m.contains("whenNotPaused")
            })
        })
    }

    fn calculate_compatibility_score(
        &self,
        warnings: &[String],
        errors: &[String],
        unsupported: &[String],
    ) -> f64 {
        let mut score = 100.0;

        // Deduct for errors
        score -= (errors.len() as f64) * 20.0;

        // Deduct for unsupported features
        score -= (unsupported.len() as f64) * 15.0;

        // Deduct for warnings
        score -= (warnings.len() as f64) * 5.0;

        // Ensure score is between 0 and 100
        score.max(0.0).min(100.0)
    }
}

impl Default for ConversionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

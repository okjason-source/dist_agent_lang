// Solidity to DAL AST Converter

use super::parser::{SolidityAST, Contract, Function, StateVariable, Event, Visibility, Mutability};
use super::types::TypeMapper;

/// DAL AST structures
#[derive(Debug, Clone)]
pub struct DALAST {
    pub services: Vec<Service>,
}

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub attributes: Vec<String>,
    pub fields: Vec<Field>,
    pub functions: Vec<DALFunction>,
    pub events: Vec<DALEvent>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DALFunction {
    pub name: String,
    pub parameters: Vec<DALParameter>,
    pub return_type: Option<String>,
    pub attributes: Vec<String>,
    pub body: Option<String>,
    /// Optional comment (e.g. "Multiple returns collapsed to first; original returns (T1, T2)").
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DALParameter {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Clone)]
pub struct DALEvent {
    pub name: String,
    pub parameters: Vec<DALParameter>,
}

/// Solidity to DAL Converter
pub struct SolidityConverter {
    type_mapper: TypeMapper,
}

impl SolidityConverter {
    pub fn new() -> Self {
        Self {
            type_mapper: TypeMapper::new(),
        }
    }
    
    /// Convert Solidity AST to DAL AST (nested contracts become separate services with Parent_Nested names).
    pub fn convert(&self, solidity_ast: SolidityAST) -> Result<DALAST, String> {
        let mut services = Vec::new();
        for contract in solidity_ast.contracts {
            services.extend(self.convert_contract_and_nested(contract, None)?);
        }
        Ok(DALAST { services })
    }

    /// Convert a contract and its nested contracts to a flat list of services (nested: "Parent_Child").
    fn convert_contract_and_nested(&self, contract: Contract, name_prefix: Option<String>) -> Result<Vec<Service>, String> {
        let name = match &name_prefix {
            None => contract.name.clone(),
            Some(p) => format!("{}_{}", p, contract.name),
        };
        let service = self.convert_contract(&contract, Some(name))?;
        let mut out = vec![service];
        let prefix = name_prefix.map(|p| format!("{}_{}", p, contract.name)).unwrap_or(contract.name.clone());
        for nested in contract.nested_contracts {
            out.extend(self.convert_contract_and_nested(nested, Some(prefix.clone()))?);
        }
        Ok(out)
    }

    fn convert_contract(&self, contract: &Contract, name_override: Option<String>) -> Result<Service, String> {
        let mut attributes = Vec::new();
        
        // Add trust model (default to hybrid for migrated contracts)
        attributes.push("@trust(\"hybrid\")".to_string());
        
        // Add blockchain attribute (default to ethereum)
        attributes.push("@chain(\"ethereum\")".to_string());
        
        // Add security attributes based on contract analysis (centralized in security module)
        if super::security::SecurityConverter::new().has_reentrancy_risk(&contract) {
            attributes.push("@secure".to_string());
        }
        
        // Convert fields
        let fields = contract.state_variables.iter()
            .map(|v| self.convert_state_variable(v))
            .collect::<Result<Vec<_>, _>>()?;
        
        // Convert functions
        let functions = contract.functions.iter()
            .map(|f| self.convert_function(f))
            .collect::<Result<Vec<_>, _>>()?;
        
        // Convert events
        let events = contract.events.iter()
            .map(|e| self.convert_event(e))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(Service {
            name: name_override.unwrap_or_else(|| contract.name.clone()),
            attributes,
            fields,
            functions,
            events,
        })
    }
    
    fn convert_state_variable(&self, var: &StateVariable) -> Result<Field, String> {
        let dal_type = self.type_mapper.convert_type(&var.var_type);
        
        Ok(Field {
            name: var.name.clone(),
            field_type: dal_type,
            initial_value: var.initial_value.clone(),
        })
    }
    
    fn convert_function(&self, func: &Function) -> Result<DALFunction, String> {
        let mut attributes = Vec::new();
        
        // Convert visibility
        match func.visibility {
            Visibility::Public => attributes.push("@public".to_string()),
            Visibility::External => attributes.push("@public".to_string()), // DAL doesn't distinguish
            Visibility::Private => attributes.push("@private".to_string()),
            Visibility::Internal => {}, // Default in DAL
        }
        
        // Convert mutability
        match func.mutability {
            Mutability::View => attributes.push("@view".to_string()),
            Mutability::Pure => attributes.push("@pure".to_string()),
            Mutability::Payable => {
                attributes.push("// @payable (value handling in DAL may differ)".to_string());
            }
            Mutability::NonPayable => {},
        }
        
        // Convert parameters
        let parameters = func.parameters.iter()
            .map(|p| DALParameter {
                name: p.name.clone(),
                param_type: self.type_mapper.convert_type(&p.param_type),
            })
            .collect();
        
        // Convert return type; multiple returns collapsed to first, with comment
        let (return_type, comment) = if func.returns.is_empty() {
            (None, None)
        } else if func.returns.len() == 1 {
            (Some(self.type_mapper.convert_type(&func.returns[0].param_type)), None)
        } else {
            let first = self.type_mapper.convert_type(&func.returns[0].param_type);
            let original = func.returns.iter()
                .map(|p| self.type_mapper.convert_type(&p.param_type))
                .collect::<Vec<_>>()
                .join(", ");
            (
                Some(first),
                Some(format!("Multiple returns collapsed to first; original returns ({})", original)),
            )
        };

        Ok(DALFunction {
            name: func.name.clone(),
            parameters,
            return_type,
            attributes,
            body: func.body.clone(),
            comment,
        })
    }
    
    fn convert_event(&self, event: &Event) -> Result<DALEvent, String> {
        let parameters = event.parameters.iter()
            .map(|p| DALParameter {
                name: p.name.clone(),
                param_type: self.type_mapper.convert_type(&p.param_type),
            })
            .collect();
        
        Ok(DALEvent {
            name: event.name.clone(),
            parameters,
        })
    }
    
}

impl Default for SolidityConverter {
    fn default() -> Self {
        Self::new()
    }
}


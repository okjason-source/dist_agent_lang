// Solidity Parser - Parses Solidity source code into AST

use std::collections::HashMap;

/// Solidity AST structures
#[derive(Debug, Clone)]
pub struct SolidityAST {
    pub contracts: Vec<Contract>,
    pub pragma: Option<String>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Contract {
    pub name: String,
    pub kind: ContractKind,
    pub state_variables: Vec<StateVariable>,
    pub functions: Vec<Function>,
    pub events: Vec<Event>,
    pub modifiers: Vec<Modifier>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub inheritance: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractKind {
    Contract,
    Interface,
    Abstract,
    Library,
}

#[derive(Debug, Clone)]
pub struct StateVariable {
    pub name: String,
    pub var_type: String,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub returns: Vec<Parameter>,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub modifiers: Vec<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    External,
    Internal,
    Private,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mutability {
    View,
    Pure,
    Payable,
    NonPayable,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub anonymous: bool,
}

#[derive(Debug, Clone)]
pub struct Modifier {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub values: Vec<String>,
}

/// Solidity Parser
pub struct SolidityParser {
    // Parser state
}

impl SolidityParser {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Parse Solidity source code
    pub fn parse(&self, source: &str) -> Result<SolidityAST, String> {
        // Extract pragma
        let pragma = self.extract_pragma(source);
        
        // Extract imports
        let imports = self.extract_imports(source);
        
        // Extract contracts
        let contracts = self.extract_contracts(source)?;
        
        Ok(SolidityAST {
            contracts,
            pragma,
            imports,
        })
    }
    
    fn extract_pragma(&self, source: &str) -> Option<String> {
        for line in source.lines() {
            let line = line.trim();
            if line.starts_with("pragma solidity") {
                return Some(line.to_string());
            }
        }
        None
    }
    
    fn extract_imports(&self, source: &str) -> Vec<String> {
        let mut imports = Vec::new();
        for line in source.lines() {
            let line = line.trim();
            if line.starts_with("import") {
                imports.push(line.to_string());
            }
        }
        imports
    }
    
    fn extract_contracts(&self, source: &str) -> Result<Vec<Contract>, String> {
        let mut contracts = Vec::new();
        let mut in_contract = false;
        let mut contract_start = 0;
        let mut brace_count = 0;
        let mut current_contract: Option<Contract> = None;
        
        let lines: Vec<&str> = source.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Detect contract start
            if trimmed.starts_with("contract ") || 
               trimmed.starts_with("interface ") ||
               trimmed.starts_with("abstract contract ") ||
               trimmed.starts_with("library ") {
                if current_contract.is_some() {
                    return Err("Nested contracts not yet supported".to_string());
                }
                
                let (name, kind) = self.parse_contract_declaration(trimmed)?;
                contract_start = i;
                in_contract = true;
                brace_count = 0;
                
                current_contract = Some(Contract {
                    name,
                    kind,
                    state_variables: Vec::new(),
                    functions: Vec::new(),
                    events: Vec::new(),
                    modifiers: Vec::new(),
                    structs: Vec::new(),
                    enums: Vec::new(),
                    inheritance: Vec::new(),
                });
            }
            
            if in_contract {
                // Count braces
                brace_count += trimmed.matches('{').count() as i32;
                brace_count -= trimmed.matches('}').count() as i32;
                
                // Parse contract content
                if let Some(ref mut contract) = current_contract {
                    self.parse_contract_line(trimmed, contract)?;
                }
                
                // Contract end
                if brace_count == 0 && i > contract_start {
                    if let Some(contract) = current_contract.take() {
                        contracts.push(contract);
                    }
                    in_contract = false;
                }
            }
        }
        
        // Handle unclosed contract
        if let Some(contract) = current_contract {
            contracts.push(contract);
        }
        
        Ok(contracts)
    }
    
    fn parse_contract_declaration(&self, line: &str) -> Result<(String, ContractKind), String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        let (kind, name_idx) = if line.starts_with("interface ") {
            (ContractKind::Interface, 1)
        } else if line.starts_with("abstract contract ") {
            (ContractKind::Abstract, 2)
        } else if line.starts_with("library ") {
            (ContractKind::Library, 1)
        } else {
            (ContractKind::Contract, 1)
        };
        
        if parts.len() <= name_idx {
            return Err("Invalid contract declaration".to_string());
        }
        
        let name = parts[name_idx].trim_end_matches('{').to_string();
        
        Ok((name, kind))
    }
    
    fn parse_contract_line(&self, line: &str, contract: &mut Contract) -> Result<(), String> {
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
            return Ok(());
        }
        
        // Parse function
        if trimmed.contains("function ") {
            if let Ok(func) = self.parse_function(line) {
                contract.functions.push(func);
            }
        }
        
        // Parse event
        if trimmed.starts_with("event ") {
            if let Ok(event) = self.parse_event(line) {
                contract.events.push(event);
            }
        }
        
        // Parse modifier
        if trimmed.starts_with("modifier ") {
            if let Ok(modifier) = self.parse_modifier(line) {
                contract.modifiers.push(modifier);
            }
        }
        
        // Parse state variable (simplified - variables without function keyword)
        if !trimmed.starts_with("function ") && 
           !trimmed.starts_with("event ") &&
           !trimmed.starts_with("modifier ") &&
           !trimmed.starts_with("struct ") &&
           !trimmed.starts_with("enum ") &&
           !trimmed.starts_with("constructor") &&
           trimmed.contains(' ') &&
           !trimmed.starts_with("//") {
            if let Ok(var) = self.parse_state_variable(line) {
                contract.state_variables.push(var);
            }
        }
        
        Ok(())
    }
    
    fn parse_function(&self, line: &str) -> Result<Function, String> {
        // Simplified function parsing
        // This is a basic implementation - can be enhanced
        
        let mut func = Function {
            name: String::new(),
            parameters: Vec::new(),
            returns: Vec::new(),
            visibility: Visibility::Public,
            mutability: Mutability::NonPayable,
            modifiers: Vec::new(),
            body: None,
        };
        
        // Extract function name
        if let Some(start) = line.find("function ") {
            let after_function = &line[start + 9..];
            if let Some(name_end) = after_function.find('(') {
                func.name = after_function[..name_end].trim().to_string();
            }
        }
        
        // Extract visibility
        if line.contains(" public") {
            func.visibility = Visibility::Public;
        } else if line.contains(" external") {
            func.visibility = Visibility::External;
        } else if line.contains(" internal") {
            func.visibility = Visibility::Internal;
        } else if line.contains(" private") {
            func.visibility = Visibility::Private;
        }
        
        // Extract mutability
        if line.contains(" view") {
            func.mutability = Mutability::View;
        } else if line.contains(" pure") {
            func.mutability = Mutability::Pure;
        } else if line.contains(" payable") {
            func.mutability = Mutability::Payable;
        }
        
        // Extract parameters (simplified)
        if let Some(params_start) = line.find('(') {
            if let Some(params_end) = line.find(')') {
                let params_str = &line[params_start + 1..params_end];
                func.parameters = self.parse_parameters(params_str)?;
            }
        }
        
        // Extract returns
        if let Some(returns_start) = line.find("returns(") {
            if let Some(returns_end) = line[returns_start..].find(')') {
                let returns_str = &line[returns_start + 8..returns_start + returns_end];
                func.returns = self.parse_parameters(returns_str)?;
            }
        }
        
        Ok(func)
    }
    
    fn parse_parameters(&self, params_str: &str) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();
        
        if params_str.trim().is_empty() {
            return Ok(params);
        }
        
        // Split by comma
        for param_str in params_str.split(',') {
            let parts: Vec<&str> = param_str.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                params.push(Parameter {
                    param_type: parts[0].to_string(),
                    name: parts[1].to_string(),
                });
            } else if parts.len() == 1 {
                // Type only, no name
                params.push(Parameter {
                    param_type: parts[0].to_string(),
                    name: format!("param{}", params.len()),
                });
            }
        }
        
        Ok(params)
    }
    
    fn parse_event(&self, line: &str) -> Result<Event, String> {
        let mut event = Event {
            name: String::new(),
            parameters: Vec::new(),
            anonymous: line.contains("anonymous"),
        };
        
        if let Some(start) = line.find("event ") {
            let after_event = &line[start + 6..];
            if let Some(name_end) = after_event.find('(') {
                event.name = after_event[..name_end].trim().to_string();
            }
            
            if let Some(params_start) = line.find('(') {
                if let Some(params_end) = line.find(')') {
                    let params_str = &line[params_start + 1..params_end];
                    event.parameters = self.parse_parameters(params_str)?;
                }
            }
        }
        
        Ok(event)
    }
    
    fn parse_modifier(&self, line: &str) -> Result<Modifier, String> {
        let mut modifier = Modifier {
            name: String::new(),
            parameters: Vec::new(),
            body: None,
        };
        
        if let Some(start) = line.find("modifier ") {
            let after_modifier = &line[start + 9..];
            if let Some(name_end) = after_modifier.find('(') {
                modifier.name = after_modifier[..name_end].trim().to_string();
            }
            
            if let Some(params_start) = line.find('(') {
                if let Some(params_end) = line.find(')') {
                    let params_str = &line[params_start + 1..params_end];
                    modifier.parameters = self.parse_parameters(params_str)?;
                }
            }
        }
        
        Ok(modifier)
    }
    
    fn parse_state_variable(&self, line: &str) -> Result<StateVariable, String> {
        // Simplified state variable parsing
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() < 2 {
            return Err("Invalid state variable".to_string());
        }
        
        let var_type = parts[0].to_string();
        let name = parts[1].trim_end_matches(';').to_string();
        
        let visibility = if line.contains(" public") {
            Visibility::Public
        } else if line.contains(" internal") {
            Visibility::Internal
        } else if line.contains(" private") {
            Visibility::Private
        } else {
            Visibility::Internal // Default
        };
        
        let mutability = if line.contains(" constant") || line.contains(" immutable") {
            Mutability::View
        } else {
            Mutability::NonPayable
        };
        
        Ok(StateVariable {
            name,
            var_type,
            visibility,
            mutability,
            initial_value: None,
        })
    }
}


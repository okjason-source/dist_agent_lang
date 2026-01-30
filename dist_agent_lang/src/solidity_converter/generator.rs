// DAL Code Generator - Generates DAL source code from DAL AST

use super::converter::{DALAST, Service, Field, DALFunction, DALParameter, DALEvent};

/// DAL Code Generator
pub struct DALGenerator {
    #[allow(dead_code)]
    indent_level: usize,
}

impl DALGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }
    
    /// Generate DAL source code from AST
    pub fn generate(&self, ast: DALAST) -> Result<String, String> {
        let mut code = String::new();
        
        for service in ast.services {
            code.push_str(&self.generate_service(service)?);
            code.push_str("\n\n");
        }
        
        Ok(code)
    }
    
    fn generate_service(&self, service: Service) -> Result<String, String> {
        let mut code = String::new();
        
        // Generate attributes
        for attr in &service.attributes {
            code.push_str(attr);
            code.push('\n');
        }
        
        // Generate service declaration
        code.push_str(&format!("service {} {{\n", service.name));
        
        // Generate fields
        if !service.fields.is_empty() {
            code.push_str("\n    // State variables\n");
            for field in &service.fields {
                code.push_str(&self.generate_field(field)?);
            }
        }
        
        // Generate functions
        if !service.functions.is_empty() {
            code.push_str("\n    // Functions\n");
            for func in &service.functions {
                code.push_str(&self.generate_function(func)?);
            }
        }
        
        // Generate events
        if !service.events.is_empty() {
            code.push_str("\n    // Events\n");
            for event in &service.events {
                code.push_str(&self.generate_event(event)?);
            }
        }
        
        code.push_str("}\n");
        
        Ok(code)
    }
    
    fn generate_field(&self, field: &Field) -> Result<String, String> {
        let mut code = String::new();
        
        code.push_str(&format!("    {}: {} = ", field.name, field.field_type));
        
        if let Some(ref init) = field.initial_value {
            code.push_str(init);
        } else {
            // Default values based on type
            code.push_str(&self.default_value_for_type(&field.field_type));
        }
        
        code.push_str(",\n");
        
        Ok(code)
    }
    
    fn default_value_for_type(&self, dal_type: &str) -> String {
        match dal_type {
            "int" => "0".to_string(),
            "bool" => "false".to_string(),
            "string" => "\"\"".to_string(),
            "vector<u8>" => "[]".to_string(),
            _ if dal_type.starts_with("vector<") => "[]".to_string(),
            _ if dal_type.starts_with("map<") => "{}".to_string(),
            _ => "null".to_string(),
        }
    }
    
    fn generate_function(&self, func: &DALFunction) -> Result<String, String> {
        let mut code = String::new();
        
        // Generate attributes
        for attr in &func.attributes {
            code.push_str(&format!("    {}\n", attr));
        }
        
        // Generate function signature
        code.push_str(&format!("    fn {}({})", func.name, self.generate_parameters(&func.parameters)?));
        
        // Generate return type
        if let Some(ref return_type) = func.return_type {
            code.push_str(&format!(" -> {}", return_type));
        }
        
        code.push_str(" {\n");
        
        // Generate function body
        if let Some(ref body) = func.body {
            // Try to convert Solidity body to DAL (simplified)
            code.push_str(&format!("        // TODO: Convert function body from Solidity\n"));
            code.push_str(&format!("        // Original: {}\n", body));
        } else {
            code.push_str("        // Function implementation\n");
        }
        
        code.push_str("    }\n\n");
        
        Ok(code)
    }
    
    fn generate_parameters(&self, params: &[DALParameter]) -> Result<String, String> {
        if params.is_empty() {
            return Ok(String::new());
        }
        
        let param_strings: Vec<String> = params.iter()
            .map(|p| format!("{}: {}", p.name, p.param_type))
            .collect();
        
        Ok(param_strings.join(", "))
    }
    
    fn generate_event(&self, event: &DALEvent) -> Result<String, String> {
        let mut code = String::new();
        
        code.push_str(&format!("    event {}({});\n", 
            event.name,
            self.generate_parameters(&event.parameters)?));
        
        Ok(code)
    }
}

impl Default for DALGenerator {
    fn default() -> Self {
        Self::new()
    }
}


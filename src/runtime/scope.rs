use crate::runtime::values::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Scope {
    pub variables: HashMap<String, Value>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::with_capacity(16), // Pre-allocate for common use cases
            parent: None,
        }
    }

    pub fn new_child(parent: Scope) -> Self {
        Self {
            variables: HashMap::with_capacity(8), // Child scopes typically have fewer variables
            parent: Some(Box::new(parent)),
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        // First check current scope
        if let Some(value) = self.variables.get(name) {
            return Some(value.clone());
        }

        // Then check parent scope
        if let Some(parent) = &self.parent {
            return parent.get(name);
        }

        None
    }

    /// Collect all variable names visible in this scope (current + parent chain). For "did you mean" suggestions.
    pub fn keys(&self) -> Vec<String> {
        let mut out: Vec<String> = self.variables.keys().cloned().collect();
        if let Some(parent) = &self.parent {
            out.extend(parent.keys());
        }
        out
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

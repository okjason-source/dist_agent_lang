
/// Capability ABI - Interface for capability-based access control
/// 
/// This provides a namespace-based approach to capability operations:
/// - cap::create(resource, permissions) - Create new capability
/// - cap::grant(capability, principal) - Grant capability to principal
/// - cap::check(capability, operation) - Check if operation is allowed

#[derive(Debug, Clone)]
pub struct Capability {
    pub id: String,
    pub resource: String,
    pub permissions: Vec<String>,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Principal {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CapabilityRequest {
    pub resource: String,
    pub operation: String,
    pub principal_id: String,
}

impl Capability {
    pub fn new(id: String, resource: String, permissions: Vec<String>) -> Self {
        Self {
            id,
            resource,
            permissions,
            expires_at: None,
        }
    }
    
    pub fn with_expiry(mut self, expires_at: i64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < 1756744707 // Mock current time
        } else {
            false
        }
    }
}

impl Principal {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            capabilities: Vec::new(),
        }
    }
    
    pub fn with_capability(mut self, capability_id: String) -> Self {
        self.capabilities.push(capability_id);
        self
    }
    
    pub fn has_capability(&self, capability_id: &str) -> bool {
        self.capabilities.contains(&capability_id.to_string())
    }
}

/// Create new capability
pub fn create(resource: &str, permissions: Vec<&str>) -> Result<Capability, String> {
    // Mock implementation - in real system this would create capability objects
    if resource.is_empty() {
        return Err("Resource cannot be empty".to_string());
    }
    
    if permissions.is_empty() {
        return Err("At least one permission is required".to_string());
    }
    
    let capability_id = format!("cap_{}_{}", resource.replace("/", "_"), 1756744707);
    let permissions_vec: Vec<String> = permissions.iter().map(|&s| s.to_string()).collect();
    
    Ok(Capability::new(capability_id, resource.to_string(), permissions_vec))
}

/// Grant capability to principal
pub fn grant(capability: &Capability, principal: &mut Principal) -> Result<bool, String> {
    // Mock implementation - in real system this would update capability registry
    if capability.is_expired() {
        return Err("Cannot grant expired capability".to_string());
    }
    
    if principal.has_capability(&capability.id) {
        return Err("Principal already has this capability".to_string());
    }
    
    *principal = principal.clone().with_capability(capability.id.clone());
    Ok(true)
}

/// Check if operation is allowed
pub fn check(request: CapabilityRequest) -> Result<bool, String> {
    // Mock implementation - in real system this would check capability registry
    match request.resource.as_str() {
        "user_data" => {
            if request.operation == "read" {
                Ok(true)
            } else if request.operation == "write" {
                Ok(false) // Write access requires admin capability
            } else {
                Err("Unknown operation".to_string())
            }
        }
        "system_config" => {
            if request.operation == "read" {
                Ok(true) // System config readable by all
            } else {
                Ok(false) // System config modification requires superadmin capability
            }
        }
        _ => Err("Unknown resource".to_string())
    }
}

/// Create a new principal
pub fn create_principal(id: String, name: String) -> Principal {
    Principal::new(id, name)
}

/// Create a capability request
pub fn create_capability_request(resource: String, operation: String, principal_id: String) -> CapabilityRequest {
    CapabilityRequest {
        resource,
        operation,
        principal_id,
    }
}

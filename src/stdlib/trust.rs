use crate::runtime::values::Value;
use crate::stdlib::key::{self, create_capability_request};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::env;

/// Trust ABI - Interface for trust and admin authorization
///
/// Real-world behavior:
/// - **authorize**: First checks the key registry (key::check); then an in-memory admin
///   registry (optionally loaded from env ADMIN_IDS, ADMIN_LEVEL_<id>); then built-in rules.
/// - **enforce_policy**: Evaluates policy by admin level; optional env POLICY_<name>_LEVEL overrides.
/// - **validate_hybrid_trust** / **bridge_trusts**: Semantic checks for hybrid trust models.

#[derive(Debug, Clone, PartialEq)]
pub enum AdminLevel {
    SuperAdmin,
    Admin,
    Moderator,
    User,
}

impl AdminLevel {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "superadmin" => Some(AdminLevel::SuperAdmin),
            "admin" => Some(AdminLevel::Admin),
            "moderator" => Some(AdminLevel::Moderator),
            "user" => Some(AdminLevel::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdminContext {
    pub admin_id: String,
    pub level: AdminLevel,
    pub permissions: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

impl AdminContext {
    pub fn new(admin_id: String, level: AdminLevel) -> Self {
        Self {
            admin_id,
            level,
            permissions: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone)]
pub struct AdminPolicy {
    pub name: String,
    pub rules: Vec<String>,
    pub admin_level: AdminLevel,
}

/// In-memory admin registry: admin_id -> (level, permissions). Can be populated by
/// register_admin() or optionally from env (ADMIN_IDS, ADMIN_LEVEL_<id>).
struct AdminRegistry {
    admins: HashMap<String, (AdminLevel, Vec<String>)>,
}

fn get_admin_registry() -> std::sync::MutexGuard<'static, AdminRegistry> {
    static REG: OnceLock<Mutex<AdminRegistry>> = OnceLock::new();
    let reg = REG.get_or_init(|| Mutex::new(AdminRegistry { admins: HashMap::new() }));
    let mut guard = reg.lock().unwrap();
    if guard.admins.is_empty() {
        let mut to_insert = HashMap::new();
        if let Ok(ids) = env::var("ADMIN_IDS") {
            for id in ids.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                let level_str = env::var(format!("ADMIN_LEVEL_{}", id.replace('.', "_")))
                    .unwrap_or_else(|_| "user".to_string());
                if let Some(level) = AdminLevel::from_string(&level_str) {
                    to_insert.insert(id.to_string(), (level, vec![]));
                }
            }
        }
        drop(guard);
        guard = reg.lock().unwrap();
        for (k, v) in to_insert {
            guard.admins.insert(k, v);
        }
    }
    guard
}

/// Register an admin in the in-memory registry (e.g. from config or DB). When ADMIN_IDS is set,
/// the registry is populated from env on first use instead.
pub fn register_admin(admin_id: String, level: AdminLevel, permissions: Vec<String>) {
    let mut reg = get_admin_registry();
    reg.admins.insert(admin_id, (level, permissions));
}

/// Remove an admin from the registry (revoke all access).
pub fn remove_admin(admin_id: &str) -> bool {
    let mut reg = get_admin_registry();
    reg.admins.remove(admin_id).is_some()
}

/// Get admin level and permissions for a user (for CLI roles listing).
pub fn get_admin_info(admin_id: &str) -> Option<(AdminLevel, Vec<String>)> {
    let reg = get_admin_registry();
    reg.admins.get(admin_id).cloned()
}

/// Authorize admin operation: 1) key::check(principal=admin_id, resource, operation),
/// 2) admin registry (or env ADMIN_IDS/ADMIN_LEVEL_*), 3) built-in rules.
/// Phase 2: All admin authorization decisions are logged for audit purposes.
pub fn authorize(admin_id: &str, operation: &str, resource: &str) -> bool {
    let req = create_capability_request(resource.to_string(), operation.to_string(), admin_id.to_string());
    
    let (allowed, reason) = if let Ok(true) = key::check(req) {
        (true, "key_registry_grant".to_string())
    } else if let Some((level, perms)) = get_admin_registry().admins.get(admin_id) {
        match operation {
            "read" => {
                (true, format!("admin_registry:{}:read", format!("{:?}", level)))
            }
            "write" => {
                let result = matches!(level, AdminLevel::Admin | AdminLevel::SuperAdmin) || perms.contains(&"write".to_string());
                (result, format!("admin_registry:{}:write:{}", format!("{:?}", level), if result { "granted" } else { "denied" }))
            }
            "delete" => {
                let result = *level == AdminLevel::SuperAdmin || perms.contains(&"delete".to_string());
                (result, format!("admin_registry:{}:delete:{}", format!("{:?}", level), if result { "granted" } else { "denied" }))
            }
            _ => {
                let result = perms.contains(&operation.to_string());
                (result, format!("admin_registry:{}:{}:{}", format!("{:?}", level), operation, if result { "granted" } else { "denied" }))
            }
        }
    } else {
        match operation {
            "read" => {
                (true, "builtin_rule:read_allowed".to_string())
            }
            "write" => {
                let result = admin_id == "admin" || admin_id == "superadmin";
                (result, format!("builtin_rule:write:{}", if result { "granted" } else { "denied" }))
            }
            "delete" => {
                let result = admin_id == "superadmin";
                (result, format!("builtin_rule:delete:{}", if result { "granted" } else { "denied" }))
            }
            _ => {
                (false, "builtin_rule:default_deny".to_string())
            }
        }
    };
    
    // Phase 2: Audit log admin authorization decision
    crate::stdlib::log::audit("admin_authorization", {
        let mut data = std::collections::HashMap::new();
        data.insert("admin_id".to_string(), crate::runtime::values::Value::String(admin_id.to_string()));
        data.insert("operation".to_string(), crate::runtime::values::Value::String(operation.to_string()));
        data.insert("resource".to_string(), crate::runtime::values::Value::String(resource.to_string()));
        data.insert("result".to_string(), crate::runtime::values::Value::String(if allowed { "allowed".to_string() } else { "denied".to_string() }));
        data.insert("reason".to_string(), crate::runtime::values::Value::String(reason.clone()));
        data
    }, Some("trust"));
    
    allowed
}

fn policy_min_level(policy_name: &str) -> Option<AdminLevel> {
    let key = format!("POLICY_{}_LEVEL", policy_name.to_uppercase().replace('-', "_"));
    env::var(key).ok().and_then(|s| AdminLevel::from_string(s.trim()))
}

/// True if level is at least min_level in hierarchy User < Moderator < Admin < SuperAdmin.
fn level_at_least(level: &AdminLevel, min_level: &AdminLevel) -> bool {
    use AdminLevel::*;
    matches!(
        (min_level, level),
        (User, _) |
        (Moderator, Moderator | Admin | SuperAdmin) |
        (Admin, Admin | SuperAdmin) |
        (SuperAdmin, SuperAdmin)
    )
}

/// Enforce admin policy: optional env POLICY_<name>_LEVEL sets minimum level; else built-in rules.
pub fn enforce_policy(policy_name: &str, context: AdminContext) -> Result<bool, String> {
    if let Some(min_level) = policy_min_level(policy_name) {
        return Ok(level_at_least(&context.level, &min_level));
    }
    match policy_name {
        "strict" => Ok(context.level == AdminLevel::SuperAdmin),
        "moderate" => Ok(context.level == AdminLevel::Admin || context.level == AdminLevel::SuperAdmin),
        "permissive" => Ok(true),
        _ => Err(format!("Unknown policy: {}", policy_name)),
    }
}

/// Validate hybrid trust between admin and user
pub fn validate_hybrid_trust(admin_trust: &str, user_trust: &str) -> bool {
    // Hybrid trust requires both admin and user trust to be valid
    admin_trust == "valid" && user_trust == "valid"
}

/// Bridge centralized admin trust with decentralized user trust
pub fn bridge_trusts(centralized_trust: &str, decentralized_trust: &str) -> bool {
    // Bridge requires both trust models to be compatible
    centralized_trust == "admin" && decentralized_trust == "user"
}

/// Create a new admin context
pub fn create_admin_context(admin_id: String, level: &str) -> Option<AdminContext> {
    let admin_level = AdminLevel::from_string(level)?;
    Some(AdminContext::new(admin_id, admin_level))
}

/// CloudAdmin ABI - Interface for CloudAdmin operations
///
/// Authorization and policy are delegated to the shared trust policy engine (trust::),
/// so admin permissions and policy rules are unified across trust and cloudadmin.
///
/// - cloudadmin::authorize(admin_id, operation, resource) - Authorize admin operation
/// - cloudadmin::enforce_policy(policy_name, context) - Enforce admin policy
/// - cloudadmin::validate_hybrid_trust(admin_trust, user_trust) - Validate hybrid trust
/// - cloudadmin::bridge_trusts(centralized_trust, decentralized_trust) - Bridge trust models

pub use crate::stdlib::trust::{AdminContext, AdminLevel, AdminPolicy, create_admin_context};

/// Authorize admin operation (delegates to trust policy engine: key registry, admin registry, env).
pub fn authorize(admin_id: &str, operation: &str, resource: &str) -> bool {
    crate::stdlib::trust::authorize(admin_id, operation, resource)
}

/// Enforce admin policy (delegates to trust policy engine).
pub fn enforce_policy(policy_name: &str, context: AdminContext) -> Result<bool, String> {
    crate::stdlib::trust::enforce_policy(policy_name, context)
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

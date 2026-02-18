/// Auth namespace for authentication and authorization
/// Provides session management and role-based access control
///
/// This module delegates to secure_auth (SecureUserStore) for production:
/// - session(), validate_credentials(), validate_token() use the store when initialized.
/// - Roles can be loaded from env (AUTH_ROLES_JSON or AUTH_ROLE_*); bridge validators from AUTH_BRIDGE_VALIDATORS.

use crate::stdlib::secure_auth::{SecureUserStore, SecureSession, RateLimiter};
use crate::stdlib::cross_chain_security::CrossChainSecurityManager;
use std::sync::{Mutex, OnceLock};
use std::env;

// Global secure user store
static USER_STORE: Mutex<Option<SecureUserStore>> = Mutex::new(None);

// Rate limiter for login/validate_credentials (per-key, e.g. username).
// Configurable via AUTH_RATE_LIMIT_REQUESTS, AUTH_RATE_LIMIT_WINDOW_SEC (read on first use).
fn login_rate_limiter() -> std::sync::MutexGuard<'static, RateLimiter> {
    static REG: OnceLock<Mutex<RateLimiter>> = OnceLock::new();
    REG.get_or_init(|| {
        let max = env::var("AUTH_RATE_LIMIT_REQUESTS").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
        let window = env::var("AUTH_RATE_LIMIT_WINDOW_SEC").ok().and_then(|s| s.parse().ok()).unwrap_or(60);
        Mutex::new(RateLimiter::new(max, window))
    });
    REG.get().unwrap().lock().unwrap()
}

// Global cross-chain security manager
static CROSS_CHAIN_MANAGER: Mutex<Option<CrossChainSecurityManager>> = Mutex::new(None);

/// Session information structure
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub created_at: i64,
    pub expires_at: i64,
}

/// Role information structure
#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<String>,
    pub description: String,
}

/// Initialize the secure authentication system
pub fn init_auth_system() {
    let mut store = USER_STORE.lock().unwrap();
    if store.is_none() {
        *store = Some(SecureUserStore::new());
    }
    
    // Initialize cross-chain security manager
    let mut cc_manager = CROSS_CHAIN_MANAGER.lock().unwrap();
    if cc_manager.is_none() {
        *cc_manager = Some(CrossChainSecurityManager::new());
    }
}

/// Create a new user account with secure password hashing
pub fn create_user(username: String, password: String, email: String, roles: Vec<String>) -> Result<String, String> {
    init_auth_system();
    let mut store = USER_STORE.lock().unwrap();
    if let Some(ref mut user_store) = *store {
        user_store.create_user(username, password, email, roles)
    } else {
        Err("Authentication system not initialized".to_string())
    }
}

/// Authenticate user and create secure session
/// Phase 2: All authentication attempts are logged for audit purposes.
pub fn authenticate(username: String, password: String) -> Result<Session, String> {
    init_auth_system();
    let mut store = USER_STORE.lock().unwrap();
    if let Some(ref mut user_store) = *store {
        match user_store.authenticate(username.clone(), password, None, None) {
            Ok(secure_session) => {
                // Phase 2: Audit log successful authentication
                crate::stdlib::log::audit("authentication_success", {
                    let mut data = std::collections::HashMap::new();
                    data.insert("username".to_string(), crate::runtime::values::Value::String(username.clone()));
                    data.insert("user_id".to_string(), crate::runtime::values::Value::String(secure_session.user_id.clone()));
                    data.insert("session_id".to_string(), crate::runtime::values::Value::String(secure_session.id.clone()));
                    data.insert("roles".to_string(), crate::runtime::values::Value::List(secure_session.roles.iter().map(|r| crate::runtime::values::Value::String(r.clone())).collect()));
                    data
                }, Some("auth"));
                
                // Convert SecureSession to legacy Session format for compatibility
                Ok(Session {
                    id: secure_session.id,
                    user_id: secure_session.user_id,
                    roles: secure_session.roles,
                    permissions: secure_session.permissions,
                    created_at: secure_session.created_at as i64,
                    expires_at: secure_session.expires_at as i64,
                })
            }
            Err(e) => {
                // Phase 2: Audit log failed authentication
                crate::stdlib::log::audit("authentication_failure", {
                    let mut data = std::collections::HashMap::new();
                    data.insert("username".to_string(), crate::runtime::values::Value::String(username.clone()));
                    data.insert("reason".to_string(), crate::runtime::values::Value::String(e.clone()));
                    data
                }, Some("auth"));
                Err(e)
            }
        }
    } else {
        let err = "Authentication system not initialized".to_string();
        // Phase 2: Audit log initialization failure
        crate::stdlib::log::audit("authentication_error", {
            let mut data = std::collections::HashMap::new();
            data.insert("username".to_string(), crate::runtime::values::Value::String(username));
            data.insert("reason".to_string(), crate::runtime::values::Value::String(err.clone()));
            data
        }, Some("auth"));
        Err(err)
    }
}

/// Create a new session for a user.
/// When the secure user store is initialized and contains the user, delegates to secure_auth
/// (create_session_for_user) and returns a real session stored in the store.
/// Otherwise builds a local Session (backward-compatible).
pub fn session(user_id: String, roles: Vec<String>) -> Session {
    init_auth_system();
    let mut store = USER_STORE.lock().unwrap();
    if let Some(ref mut user_store) = *store {
        if let Ok(secure_session) = user_store.create_session_for_user(&user_id) {
            return session_from_secure(&secure_session);
        }
    }
    // Fallback: build session locally (e.g. store not populated or user not in store)
    let session_id = format!("sess_{}_{}", user_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let expires_at = now + (24 * 60 * 60);
    let mut permissions = Vec::new();
    for role in &roles {
        if let Some(role_def) = get_role(role) {
            permissions.extend(role_def.permissions);
        } else {
            permissions.push("read".to_string());
        }
    }
    permissions.sort();
    permissions.dedup();
    Session {
        id: session_id,
        user_id,
        roles,
        permissions,
        created_at: now,
        expires_at,
    }
}

/// Convert SecureSession to legacy Session.
fn session_from_secure(s: &SecureSession) -> Session {
    Session {
        id: s.id.clone(),
        user_id: s.user_id.clone(),
        roles: s.roles.clone(),
        permissions: s.permissions.clone(),
        created_at: s.created_at as i64,
        expires_at: s.expires_at as i64,
    }
}

/// Validate a token (JWT or session token) and return a Session if valid.
/// When JWT_SECRET is set and token looks like a JWT (contains '.'), decodes JWT and builds Session from claims.
/// Otherwise looks up session by opaque token in the secure user store.
pub fn validate_token(token: &str) -> Option<Session> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    // JWT: three base64 parts separated by '.'
    if token.matches('.').count() == 2 {
        if let Ok(secret) = env::var("JWT_SECRET") {
            if !secret.is_empty() {
                if let Some(session) = validate_jwt_token(token, &secret) {
                    return Some(session);
                }
            }
        }
    }
    // Session token: look up in store
    init_auth_system();
    let store = USER_STORE.lock().unwrap();
    if let Some(ref user_store) = *store {
        if let Some(secure_session) = user_store.get_session_by_token(token) {
            return Some(session_from_secure(&secure_session));
        }
    }
    None
}

/// JWT claims for auth token validation (minimal; matches common JWT shape).
#[derive(serde::Deserialize)]
struct AuthJwtClaims {
    sub: String,
    exp: u64,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    permissions: Vec<String>,
}

fn validate_jwt_token(token: &str, secret: &str) -> Option<Session> {
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
    let key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_required_spec_claims(&["exp"]); // Reject tokens with malformed/missing exp (CVE-2026-25537)
    let token_data = decode::<AuthJwtClaims>(token, &key, &validation).ok()?;
    let claims = token_data.claims;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now > claims.exp {
        return None;
    }
    let mut permissions = claims.permissions;
    if permissions.is_empty() {
        for role in &claims.roles {
            if let Some(role_def) = get_role(role) {
                permissions.extend(role_def.permissions);
            } else {
                permissions.push("read".to_string());
            }
        }
        permissions.sort();
        permissions.dedup();
    }
    Some(Session {
        id: format!("jwt_{}", claims.sub),
        user_id: claims.sub,
        roles: claims.roles,
        permissions,
        created_at: now as i64,
        expires_at: claims.exp as i64,
    })
}

/// Check if a session is valid
/// 
/// # Arguments
/// * `session` - The session to validate
/// 
/// # Returns
/// * `bool` - True if session is valid, false otherwise
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::auth;
/// let session = auth::session("user123".to_string(), vec![]);
/// let is_valid = auth::is_valid_session(&session);
/// ```
pub fn is_valid_session(session: &Session) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    now < session.expires_at
}

/// Check if a session has a specific permission
/// 
/// # Arguments
/// * `session` - The session to check
/// * `permission` - The permission to check for
/// 
/// # Returns
/// * `bool` - True if session has permission, false otherwise
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::auth;
/// let session = auth::session("user123".to_string(), vec![]);
/// let can_write = auth::has_permission(&session, "write");
/// ```
pub fn has_permission(session: &Session, permission: &str) -> bool {
    if !is_valid_session(session) {
        return false;
    }
    
    session.permissions.contains(&permission.to_string())
}

/// Check if a session has a specific role
/// 
/// # Arguments
/// * `session` - The session to check
/// * `role` - The role to check for
/// 
/// # Returns
/// * `bool` - True if session has role, false otherwise
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::auth;
/// let session = auth::session("user123".to_string(), vec!["admin".to_string()]);
/// let is_admin = auth::has_role(&session, "admin");
/// ```
pub fn has_role(session: &Session, role: &str) -> bool {
    if !is_valid_session(session) {
        return false;
    }
    
    session.roles.contains(&role.to_string())
}

/// Create a new role
/// 
/// # Arguments
/// * `name` - The name of the role
/// * `permissions` - List of permissions for the role
/// * `description` - Description of the role
/// 
/// # Returns
/// * `Role` - The created role object
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::auth;
/// let role = auth::create_role("editor".to_string(), vec!["read".to_string(), "write".to_string()], "Can edit content".to_string());
/// ```
pub fn create_role(name: String, permissions: Vec<String>, description: String) -> Role {
    Role {
        name,
        permissions,
        description,
    }
}

/// Get role information.
/// Loads from env when set: AUTH_ROLES_JSON (JSON array of {name, permissions[], description}),
/// or AUTH_ROLE_<name>_PERMISSIONS (comma-separated) and AUTH_ROLE_<name>_DESCRIPTION.
/// Falls back to built-in admin/moderator/user when not configured.
pub fn get_role(role_name: &str) -> Option<Role> {
    // Env: single role AUTH_ROLE_<name>_PERMISSIONS and AUTH_ROLE_<name>_DESCRIPTION
    let env_key = role_name.to_uppercase().replace('-', "_");
    if let Ok(perms) = env::var(format!("AUTH_ROLE_{}_PERMISSIONS", env_key)) {
        let permissions: Vec<String> = perms.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        let description = env::var(format!("AUTH_ROLE_{}_DESCRIPTION", env_key)).unwrap_or_else(|_| format!("Role {}", role_name));
        return Some(Role {
            name: role_name.to_string(),
            permissions: if permissions.is_empty() { vec!["read".to_string()] } else { permissions },
            description,
        });
    }
    // Env: AUTH_ROLES_JSON array of {name, permissions, description}
    if let Ok(json) = env::var("AUTH_ROLES_JSON") {
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&json) {
            for r in arr {
                if let (Some(n), Some(p)) = (r.get("name").and_then(|v| v.as_str()), r.get("permissions")) {
                    if n == role_name {
                        let permissions: Vec<String> = p.as_array()
                            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default();
                        let description = r.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        return Some(Role { name: role_name.to_string(), permissions, description });
                    }
                }
            }
        }
    }
    // Fallback: built-in roles
    match role_name {
        "admin" => Some(Role {
            name: "admin".to_string(),
            permissions: vec!["read".to_string(), "write".to_string(), "delete".to_string(), "admin".to_string()],
            description: "Full system access".to_string(),
        }),
        "moderator" => Some(Role {
            name: "moderator".to_string(),
            permissions: vec!["read".to_string(), "write".to_string(), "moderate".to_string()],
            description: "Can moderate content".to_string(),
        }),
        "user" => Some(Role {
            name: "user".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            description: "Standard user access".to_string(),
        }),
        _ => None,
    }
}

/// Validate user credentials.
/// When the secure user store is initialized, checks against the store (PBKDF2-verified)
/// and applies rate limiting (AUTH_RATE_LIMIT_*). Otherwise falls back to a minimal mock for tests.
pub fn validate_credentials(username: &str, password: &str) -> Option<String> {
    init_auth_system();
    let key = format!("cred:{}", username);
    if !login_rate_limiter().is_allowed(&key) {
        return None;
    }
    let mut store = USER_STORE.lock().unwrap();
    if let Some(ref mut user_store) = *store {
        return user_store.try_validate_credentials(username, password);
    }
    // Fallback for tests when store not populated
    if username == "admin" && password == "admin123" {
        Some("admin_001".to_string())
    } else if username == "user" && password == "user123" {
        Some("user_001".to_string())
    } else {
        None
    }
}

/// Validate cross-chain bridge operation
pub fn validate_bridge_operation(
    source_chain: String,
    target_chain: String,
    amount: u64,
    user_session: &Session
) -> Result<String, String> {
    use crate::stdlib::cross_chain_security::{CrossChainOperation, CrossChainOperationType};
    
    init_auth_system();
    let mut cc_manager = CROSS_CHAIN_MANAGER.lock().unwrap();
    
    if let Some(ref mut manager) = *cc_manager {
        // Check if user has cross-chain permissions
        if !user_session.permissions.contains(&"cross_chain_transfer".to_string()) {
            return Err("User lacks cross-chain transfer permissions".to_string());
        }
        
        // Parse chain IDs (simple parsing for demo)
        let source_chain_id = source_chain.parse::<i64>().unwrap_or(1);
        let target_chain_id = target_chain.parse::<i64>().unwrap_or(2);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Create a cross-chain operation
        let operation = CrossChainOperation {
            operation_id: format!("bridge_{}_{}", source_chain, target_chain),
            source_chain: source_chain_id,
            target_chain: target_chain_id,
            operation_type: CrossChainOperationType::Transfer {
                from: user_session.user_id.clone(),
                to: "bridge_escrow".to_string(),
                amount,
            },
            data: vec![],
            signatures: vec![],
            status: crate::stdlib::cross_chain_security::OperationStatus::Pending,
            created_at: now,
            timeout: now + 3600, // 1 hour timeout
        };
        
        // Validate the cross-chain operation
        match manager.validate_cross_chain_operation(operation) {
            Ok(operation_id) => Ok(operation_id),
            Err(e) => Err(format!("Bridge validation failed: {:?}", e))
        }
    } else {
        Err("Cross-chain manager not initialized".to_string())
    }
}

/// Register a new bridge configuration
pub fn register_bridge(
    source_chain_id: i64,
    target_chain_id: i64,
    bridge_contract: String,
    admin_session: &Session
) -> Result<String, String> {
    init_auth_system();
    let mut cc_manager = CROSS_CHAIN_MANAGER.lock().unwrap();
    
    if let Some(ref mut manager) = *cc_manager {
        // Check admin permissions
        if !admin_session.permissions.contains(&"admin".to_string()) {
            return Err("Admin permissions required for bridge registration".to_string());
        }
        
        let validators: Vec<String> = env::var("AUTH_BRIDGE_VALIDATORS")
            .map(|s| s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect())
            .unwrap_or_default();
        match manager.create_bridge(
            source_chain_id,
            target_chain_id,
            bridge_contract,
            validators,
            2,      // min_signatures
            1000000, // max_amount
            10000,   // security_deposit
        ) {
            Ok(bridge_id) => Ok(bridge_id),
            Err(e) => Err(format!("Bridge registration failed: {:?}", e))
        }
    } else {
        Err("Cross-chain manager not initialized".to_string())
    }
}

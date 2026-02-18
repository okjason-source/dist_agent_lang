use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, OnceLock};

/// Key module — capability-based access control (keys to resources).
///
/// - key::create(resource, permissions) — Create and register a capability (unique ID).
/// - key::grant(capability, principal) — Grant capability to principal.
/// - key::check(request) — Check if operation is allowed (registry + optional built-in fallbacks).
/// - key::revoke(capability_id, principal_id) — Revoke one grant.
/// - key::revoke_all(principal_id) — Revoke all grants for a principal.
/// - key::list_for_principal(principal_id) — List capabilities granted to a principal.
///
/// Principal ID in requests typically comes from auth context (e.g. auth::current_principal() or session).

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Capability {
    pub id: String,
    pub resource: String,
    pub permissions: Vec<String>,
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub created_at: Option<i64>,
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
            created_at: Some(current_time_secs()),
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
            expires_at < current_time_secs()
        } else {
            false
        }
    }
}

/// Current Unix timestamp in seconds (for expiry checks).
fn current_time_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// In-memory key registry: created keys and principal grants.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct KeyRegistry {
    capabilities: HashMap<String, Capability>,
    principal_grants: HashMap<String, Vec<String>>,
}

/// Store backend for persistence. Default from env: DAL_KEY_STORE=memory|file|sqlite, DAL_KEY_STORE_PATH for file path.
pub trait KeyStore: Send + Sync {
    fn load(&self) -> Result<KeyRegistry, String>;
    fn save(&self, reg: &KeyRegistry) -> Result<(), String>;
}

/// In-memory store (default). No persistence.
pub struct MemoryStore;

impl KeyStore for MemoryStore {
    fn load(&self) -> Result<KeyRegistry, String> {
        Ok(KeyRegistry::default())
    }
    fn save(&self, _reg: &KeyRegistry) -> Result<(), String> {
        Ok(())
    }
}

/// File-backed store: JSON at path from DAL_KEY_STORE_PATH or ~/.dal/key_registry.json.
pub struct FileStore {
    path: std::path::PathBuf,
}

impl FileStore {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self { path }
    }
}

impl KeyStore for FileStore {
    fn load(&self) -> Result<KeyRegistry, String> {
        let data = match std::fs::read_to_string(&self.path) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(KeyRegistry::default()),
            Err(e) => return Err(e.to_string()),
        };
        serde_json::from_str(&data).map_err(|e| e.to_string())
    }
    fn save(&self, reg: &KeyRegistry) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(reg).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, data).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "sqlite-storage")]
mod sqlite_store {
    use super::{Capability, KeyRegistry, KeyStore};
    use std::collections::HashMap;

    pub struct SqliteStore {
        path: std::path::PathBuf,
    }

    impl SqliteStore {
        pub fn new(path: std::path::PathBuf) -> Self {
            Self { path }
        }
    }

    impl KeyStore for SqliteStore {
        fn load(&self) -> Result<KeyRegistry, String> {
            let conn = rusqlite::Connection::open(&self.path).map_err(|e| e.to_string())?;
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS capabilities (
                    id TEXT PRIMARY KEY,
                    resource TEXT NOT NULL,
                    permissions TEXT NOT NULL,
                    expires_at INTEGER,
                    created_at INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS principal_grants (
                    principal_id TEXT NOT NULL,
                    capability_id TEXT NOT NULL,
                    granted_at INTEGER NOT NULL,
                    PRIMARY KEY (principal_id, capability_id),
                    FOREIGN KEY (capability_id) REFERENCES capabilities(id)
                );",
            )
            .map_err(|e| e.to_string())?;
            let mut stmt = conn.prepare("SELECT id, resource, permissions, expires_at, created_at FROM capabilities").map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            }).map_err(|e| e.to_string())?;
            let mut capabilities = HashMap::new();
            for row in rows {
                let (id, resource, perms_json, expires_at, created_at) = row.map_err(|e| e.to_string())?;
                let permissions: Vec<String> = serde_json::from_str(&perms_json).unwrap_or_default();
                let cap = Capability {
                    id: id.clone(),
                    resource,
                    permissions,
                    expires_at,
                    created_at: Some(created_at),
                };
                capabilities.insert(id, cap);
            }
            let mut stmt = conn.prepare("SELECT principal_id, capability_id FROM principal_grants").map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))).map_err(|e| e.to_string())?;
            let mut principal_grants: HashMap<String, Vec<String>> = HashMap::new();
            for row in rows {
                let (principal_id, capability_id) = row.map_err(|e| e.to_string())?;
                principal_grants.entry(principal_id).or_default().push(capability_id);
            }
            Ok(KeyRegistry { capabilities, principal_grants })
        }
        fn save(&self, reg: &KeyRegistry) -> Result<(), String> {
            let conn = rusqlite::Connection::open(&self.path).map_err(|e| e.to_string())?;
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS capabilities (
                    id TEXT PRIMARY KEY,
                    resource TEXT NOT NULL,
                    permissions TEXT NOT NULL,
                    expires_at INTEGER,
                    created_at INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS principal_grants (
                    principal_id TEXT NOT NULL,
                    capability_id TEXT NOT NULL,
                    granted_at INTEGER NOT NULL,
                    PRIMARY KEY (principal_id, capability_id),
                    FOREIGN KEY (capability_id) REFERENCES capabilities(id)
                );",
            )
            .map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM principal_grants", []).map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM capabilities", []).map_err(|e| e.to_string())?;
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
            for cap in reg.capabilities.values() {
                let perms = serde_json::to_string(&cap.permissions).map_err(|e| e.to_string())?;
                let created = cap.created_at.unwrap_or(now);
                conn.execute(
                    "INSERT OR REPLACE INTO capabilities (id, resource, permissions, expires_at, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![cap.id, cap.resource, perms, cap.expires_at, created],
                )
                .map_err(|e| e.to_string())?;
            }
            for (principal_id, cap_ids) in &reg.principal_grants {
                for cap_id in cap_ids {
                    conn.execute(
                        "INSERT INTO principal_grants (principal_id, capability_id, granted_at) VALUES (?1, ?2, ?3)",
                        rusqlite::params![principal_id, cap_id, now],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
            Ok(())
        }
    }
}

#[cfg(feature = "sqlite-storage")]
use sqlite_store::SqliteStore;

fn from_env() -> Box<dyn KeyStore> {
    let store = std::env::var("DAL_KEY_STORE").unwrap_or_else(|_| "memory".to_string());
    let path = std::env::var("DAL_KEY_STORE_PATH").ok();
    match store.to_lowercase().as_str() {
        "file" => {
            let p = path
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    std::path::PathBuf::from(home).join(".dal").join("key_registry.json")
                });
            Box::new(FileStore::new(p))
        }
        "sqlite" => {
            #[cfg(feature = "sqlite-storage")]
            {
                let p = path
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| {
                        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                        std::path::PathBuf::from(home).join(".dal").join("key_registry.db")
                    });
                Box::new(SqliteStore::new(p))
            }
            #[cfg(not(feature = "sqlite-storage"))]
            {
                eprintln!("warning: DAL_KEY_STORE=sqlite requires 'sqlite-storage' feature, using memory");
                Box::new(MemoryStore)
            }
        }
        _ => Box::new(MemoryStore),
    }
}

/// Guard that persists the registry to the store when dropped.
struct KeyRegistryGuard<'a> {
    inner: std::sync::MutexGuard<'a, KeyRegistry>,
    store: Option<&'static dyn KeyStore>,
}

impl Deref for KeyRegistryGuard<'_> {
    type Target = KeyRegistry;
    fn deref(&self) -> &KeyRegistry {
        &self.inner
    }
}

impl DerefMut for KeyRegistryGuard<'_> {
    fn deref_mut(&mut self) -> &mut KeyRegistry {
        &mut self.inner
    }
}

impl Drop for KeyRegistryGuard<'_> {
    fn drop(&mut self) {
        if let Some(store) = self.store {
            let _ = store.save(&*self.inner);
        }
    }
}

fn get_registry() -> KeyRegistryGuard<'static> {
    static STORE: OnceLock<Box<dyn KeyStore>> = OnceLock::new();
    static REG: OnceLock<Mutex<KeyRegistry>> = OnceLock::new();
    let store = STORE.get_or_init(from_env);
    let reg = REG.get_or_init(|| {
        let data = store.load().unwrap_or_else(|_| KeyRegistry::default());
        Mutex::new(data)
    });
    KeyRegistryGuard {
        inner: reg.lock().unwrap(),
        store: Some(store.as_ref()),
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

/// Create new key (capability) and register it in the registry.
/// ID format: `key_{resource_slug}_{uuid}` for uniqueness.
pub fn create(resource: &str, permissions: Vec<&str>) -> Result<Capability, String> {
    if resource.is_empty() {
        return Err("Resource cannot be empty".to_string());
    }
    if permissions.is_empty() {
        return Err("At least one permission is required".to_string());
    }

    let slug = resource.replace('/', "_");
    let id = format!("key_{}_{}", slug, uuid::Uuid::new_v4().simple());
    let permissions_vec: Vec<String> = permissions.iter().map(|&s| s.to_string()).collect();
    let capability = Capability::new(id.clone(), resource.to_string(), permissions_vec);

    let mut reg = get_registry();
    reg.capabilities.insert(id, capability.clone());
    Ok(capability)
}

/// Grant key to principal and update registry.
/// Capability must already be registered (e.g. via create()); duplicate grants are rejected.
pub fn grant(capability: &Capability, principal: &mut Principal) -> Result<bool, String> {
    if capability.is_expired() {
        return Err("Cannot grant expired capability".to_string());
    }
    if principal.has_capability(&capability.id) {
        return Err("Principal already has this capability".to_string());
    }

    let mut reg = get_registry();
    reg.capabilities.insert(capability.id.clone(), capability.clone());
    reg.principal_grants
        .entry(principal.id.clone())
        .or_default()
        .push(capability.id.clone());
    *principal = principal.clone().with_capability(capability.id.clone());
    Ok(true)
}

/// Revoke one capability grant from a principal.
pub fn revoke(capability_id: &str, principal_id: &str) -> Result<bool, String> {
    let mut reg = get_registry();
    let Some(grants) = reg.principal_grants.get_mut(principal_id) else {
        return Ok(false);
    };
    if let Some(pos) = grants.iter().position(|id| id == capability_id) {
        grants.remove(pos);
        if grants.is_empty() {
            reg.principal_grants.remove(principal_id);
        }
        return Ok(true);
    }
    Ok(false)
}

/// Revoke all capability grants for a principal. Returns the number revoked.
pub fn revoke_all(principal_id: &str) -> Result<usize, String> {
    let mut reg = get_registry();
    let n = reg
        .principal_grants
        .remove(principal_id)
        .map(|v| v.len())
        .unwrap_or(0);
    Ok(n)
}

/// List capabilities granted to a principal (for debugging / introspection).
pub fn list_for_principal(principal_id: &str) -> Vec<Capability> {
    let reg = get_registry();
    let Some(cap_ids) = reg.principal_grants.get(principal_id) else {
        return Vec::new();
    };
    cap_ids
        .iter()
        .filter_map(|id| reg.capabilities.get(id).cloned())
        .filter(|c| !c.is_expired())
        .collect()
}

/// Check if operation is allowed by looking up key registry for the principal.
/// When env `DAL_KEY_STRICT` is set (e.g. "true", "1"), built-in fallbacks are disabled.
/// Phase 2: All access control decisions are logged for audit purposes.
pub fn check(request: CapabilityRequest) -> Result<bool, String> {
    let reg = get_registry();
    let mut allowed = false;
    let mut reason = "no_matching_capability".to_string();
    
    let cap_ids = reg.principal_grants.get(&request.principal_id);
    if let Some(cap_ids) = cap_ids {
        for cap_id in cap_ids {
            if let Some(cap) = reg.capabilities.get(cap_id) {
                if cap.is_expired() {
                    continue;
                }
                if cap.resource == request.resource && cap.has_permission(&request.operation) {
                    allowed = true;
                    reason = format!("capability_granted:{}", cap_id);
                    break;
                }
            }
        }
    }
    
    if !allowed && strict_mode() {
        // Phase 2: Audit log access denial
        crate::stdlib::log::audit("access_control_check", {
            let mut data = HashMap::new();
            data.insert("principal_id".to_string(), crate::runtime::values::Value::String(request.principal_id.clone()));
            data.insert("resource".to_string(), crate::runtime::values::Value::String(request.resource.clone()));
            data.insert("operation".to_string(), crate::runtime::values::Value::String(request.operation.clone()));
            data.insert("result".to_string(), crate::runtime::values::Value::String("denied".to_string()));
            data.insert("reason".to_string(), crate::runtime::values::Value::String(reason.clone()));
            data.insert("strict_mode".to_string(), crate::runtime::values::Value::Bool(true));
            data
        }, Some("key"));
        return Ok(false);
    }
    
    if !allowed {
        // Fallback: built-in rules when no matching key in registry
        let (result, reason_str) = match request.resource.as_str() {
            "user_data" => {
                if request.operation == "read" {
                    (true, "builtin_rule:user_data_read".to_string())
                } else if request.operation == "write" {
                    (false, "builtin_rule:user_data_write_denied".to_string())
                } else {
                    return Err("Unknown operation".to_string());
                }
            }
            "system_config" => {
                if request.operation == "read" {
                    (true, "builtin_rule:system_config_read".to_string())
                } else {
                    (false, "builtin_rule:system_config_write_denied".to_string())
                }
            }
            _ => {
                (false, "builtin_rule:default_deny".to_string())
            }
        };
        
        // Phase 2: Audit log access decision
        crate::stdlib::log::audit("access_control_check", {
            let mut data = HashMap::new();
            data.insert("principal_id".to_string(), crate::runtime::values::Value::String(request.principal_id.clone()));
            data.insert("resource".to_string(), crate::runtime::values::Value::String(request.resource.clone()));
            data.insert("operation".to_string(), crate::runtime::values::Value::String(request.operation.clone()));
            data.insert("result".to_string(), crate::runtime::values::Value::String(if result { "allowed".to_string() } else { "denied".to_string() }));
            data.insert("reason".to_string(), crate::runtime::values::Value::String(reason_str.clone()));
            data.insert("strict_mode".to_string(), crate::runtime::values::Value::Bool(false));
            data
        }, Some("key"));
        
        return Ok(result);
    }
    
    // Phase 2: Audit log access grant
    crate::stdlib::log::audit("access_control_check", {
        let mut data = HashMap::new();
        data.insert("principal_id".to_string(), crate::runtime::values::Value::String(request.principal_id.clone()));
        data.insert("resource".to_string(), crate::runtime::values::Value::String(request.resource.clone()));
        data.insert("operation".to_string(), crate::runtime::values::Value::String(request.operation.clone()));
        data.insert("result".to_string(), crate::runtime::values::Value::String("allowed".to_string()));
        data.insert("reason".to_string(), crate::runtime::values::Value::String(reason.clone()));
        data.insert("strict_mode".to_string(), crate::runtime::values::Value::Bool(strict_mode()));
        data
    }, Some("key"));
    
    Ok(true)
}

fn strict_mode() -> bool {
    std::env::var("DAL_KEY_STRICT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        .unwrap_or(false)
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

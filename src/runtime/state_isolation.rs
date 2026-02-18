/// State Isolation System for DAL Runtime
/// Provides secure contract state isolation and access control

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::runtime::values::Value;
use crate::runtime::functions::RuntimeError;

/// Isolated contract state container
#[derive(Debug, Clone)]
pub struct IsolatedContractState {
    contract_address: String,
    state: Arc<RwLock<HashMap<String, Value>>>,
    metadata: StateMetadata,
    access_control: AccessControl,
}

#[derive(Debug, Clone)]
pub struct StateMetadata {
    pub contract_name: String,
    pub owner: String,
    pub created_at: u64,
    pub last_modified: u64,
    pub read_only: bool,
    pub gas_limit: u64,
    pub state_version: u64,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct AccessControl {
    pub allowed_callers: Vec<String>,
    pub required_permissions: Vec<String>,
    pub trust_level: String,
    pub admin_only_operations: Vec<String>,
}

impl IsolatedContractState {
    /// Create a new isolated contract state
    pub fn new(
        contract_address: String,
        contract_name: String,
        owner: String,
        trust_level: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            contract_address: contract_address.clone(),
            state: Arc::new(RwLock::new(HashMap::new())),
            metadata: StateMetadata {
                contract_name,
                owner,
                created_at: now,
                last_modified: now,
                read_only: false,
                gas_limit: 100_000,
                state_version: 1,
                checksum: String::new(),
            },
            access_control: AccessControl {
                allowed_callers: vec![contract_address],
                required_permissions: Vec::new(),
                trust_level,
                admin_only_operations: vec![
                    "set_read_only".to_string(),
                    "transfer_ownership".to_string(),
                    "upgrade_contract".to_string(),
                ],
            },
        }
    }

    /// Read a value from the isolated state
    pub fn read_value(&self, key: &str, caller: &str, permissions: &[String]) -> Result<Value, RuntimeError> {
        // Validate access
        self.validate_access(caller, "read", permissions)?;

        let state = self.state.read().map_err(|_| {
            RuntimeError::General("Failed to acquire read lock on contract state".to_string())
        })?;

        Ok(state.get(key).cloned().unwrap_or(Value::Null))
    }

    /// Write a value to the isolated state
    pub fn write_value(&mut self, key: &str, value: Value, caller: &str, permissions: &[String]) -> Result<(), RuntimeError> {
        // Validate access
        self.validate_access(caller, "write", permissions)?;

        // Check if contract is read-only
        if self.metadata.read_only {
            return Err(RuntimeError::ReadOnlyViolation);
        }

        // Check gas limit (simplified)
        if key.len() + self.estimate_value_size(&value) > self.metadata.gas_limit as usize {
            return Err(RuntimeError::General("Gas limit exceeded for state operation".to_string()));
        }

        let mut state = self.state.write().map_err(|_| {
            RuntimeError::General("Failed to acquire write lock on contract state".to_string())
        })?;

        // Update state
        state.insert(key.to_string(), value);

        // Update metadata
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.metadata.last_modified = now;
        self.metadata.state_version += 1;
        self.metadata.checksum = self.calculate_state_checksum(&state);

        Ok(())
    }

    /// Delete a value from the isolated state
    pub fn delete_value(&mut self, key: &str, caller: &str, permissions: &[String]) -> Result<bool, RuntimeError> {
        // Validate access
        self.validate_access(caller, "delete", permissions)?;

        // Check if contract is read-only
        if self.metadata.read_only {
            return Err(RuntimeError::ReadOnlyViolation);
        }

        let mut state = self.state.write().map_err(|_| {
            RuntimeError::General("Failed to acquire write lock on contract state".to_string())
        })?;

        let existed = state.remove(key).is_some();

        if existed {
            // Update metadata
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            self.metadata.last_modified = now;
            self.metadata.state_version += 1;
            self.metadata.checksum = self.calculate_state_checksum(&state);
        }

        Ok(existed)
    }

    /// Get all keys in the state (admin only)
    pub fn get_all_keys(&self, caller: &str, permissions: &[String]) -> Result<Vec<String>, RuntimeError> {
        // Validate admin access
        self.validate_admin_access(caller, permissions)?;

        let state = self.state.read().map_err(|_| {
            RuntimeError::General("Failed to acquire read lock on contract state".to_string())
        })?;

        Ok(state.keys().cloned().collect())
    }

    /// Set contract to read-only mode (admin only)
    pub fn set_read_only(&mut self, read_only: bool, caller: &str, permissions: &[String]) -> Result<(), RuntimeError> {
        // Validate admin access
        self.validate_admin_access(caller, permissions)?;

        self.metadata.read_only = read_only;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.metadata.last_modified = now;

        Ok(())
    }

    /// Transfer ownership (admin only)
    pub fn transfer_ownership(&mut self, new_owner: String, caller: &str, permissions: &[String]) -> Result<(), RuntimeError> {
        // Validate admin access
        self.validate_admin_access(caller, permissions)?;

        self.metadata.owner = new_owner;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.metadata.last_modified = now;

        Ok(())
    }

    /// Add allowed caller
    pub fn add_allowed_caller(&mut self, caller_address: String, caller: &str, permissions: &[String]) -> Result<(), RuntimeError> {
        // Validate admin access
        self.validate_admin_access(caller, permissions)?;

        if !self.access_control.allowed_callers.contains(&caller_address) {
            self.access_control.allowed_callers.push(caller_address);
        }

        Ok(())
    }

    /// Remove allowed caller
    pub fn remove_allowed_caller(&mut self, caller_address: &str, caller: &str, permissions: &[String]) -> Result<(), RuntimeError> {
        // Validate admin access
        self.validate_admin_access(caller, permissions)?;

        self.access_control.allowed_callers.retain(|addr| addr != caller_address);

        Ok(())
    }

    /// Validate access for basic operations
    /// Phase 2: All state isolation access control decisions are logged for audit purposes.
    fn validate_access(&self, caller: &str, operation: &str, permissions: &[String]) -> Result<(), RuntimeError> {
        // Check if caller is in allowed list
        if !self.access_control.allowed_callers.contains(&caller.to_string()) {
            // Phase 2: Audit log access denial
            crate::stdlib::log::audit("state_isolation_access_denied", {
                let mut data = std::collections::HashMap::new();
                data.insert("contract_address".to_string(), Value::String(self.contract_address.clone()));
                data.insert("contract_name".to_string(), Value::String(self.metadata.contract_name.clone()));
                data.insert("caller".to_string(), Value::String(caller.to_string()));
                data.insert("operation".to_string(), Value::String(operation.to_string()));
                data.insert("reason".to_string(), Value::String("caller_not_in_allowed_list".to_string()));
                data
            }, Some("state_isolation"));
            return Err(RuntimeError::AccessDenied);
        }

        // Check operation-specific permissions
        let result = match operation {
            "read" => {
                // Read is generally allowed for authorized callers
                Ok(())
            }
            "write" | "delete" => {
                // Write/delete requires write permission
                if !permissions.contains(&"write".to_string()) && !permissions.contains(&"admin".to_string()) {
                    // Phase 2: Audit log permission denial
                    crate::stdlib::log::audit("state_isolation_access_denied", {
                        let mut data = std::collections::HashMap::new();
                        data.insert("contract_address".to_string(), Value::String(self.contract_address.clone()));
                        data.insert("contract_name".to_string(), Value::String(self.metadata.contract_name.clone()));
                        data.insert("caller".to_string(), Value::String(caller.to_string()));
                        data.insert("operation".to_string(), Value::String(operation.to_string()));
                        data.insert("reason".to_string(), Value::String("insufficient_permissions".to_string()));
                        data.insert("required_permissions".to_string(), Value::String("write or admin".to_string()));
                        data.insert("caller_permissions".to_string(), Value::List(permissions.iter().map(|p| Value::String(p.clone())).collect()));
                        data
                    }, Some("state_isolation"));
                    return Err(RuntimeError::PermissionDenied("Write permission required".to_string()));
                }
                Ok(())
            }
            _ => {
                let err = Err(RuntimeError::UnsupportedOperation(format!("Unknown operation: {}", operation)));
                // Phase 2: Audit log unsupported operation
                crate::stdlib::log::audit("state_isolation_access_denied", {
                    let mut data = std::collections::HashMap::new();
                    data.insert("contract_address".to_string(), Value::String(self.contract_address.clone()));
                    data.insert("contract_name".to_string(), Value::String(self.metadata.contract_name.clone()));
                    data.insert("caller".to_string(), Value::String(caller.to_string()));
                    data.insert("operation".to_string(), Value::String(operation.to_string()));
                    data.insert("reason".to_string(), Value::String("unsupported_operation".to_string()));
                    data
                }, Some("state_isolation"));
                return err;
            }
        };
        
        // Phase 2: Audit log successful access
        if result.is_ok() {
            crate::stdlib::log::audit("state_isolation_access_allowed", {
                let mut data = std::collections::HashMap::new();
                data.insert("contract_address".to_string(), Value::String(self.contract_address.clone()));
                data.insert("contract_name".to_string(), Value::String(self.metadata.contract_name.clone()));
                data.insert("caller".to_string(), Value::String(caller.to_string()));
                data.insert("operation".to_string(), Value::String(operation.to_string()));
                data
            }, Some("state_isolation"));
        }
        
        result
    }

    /// Validate admin access for privileged operations
    /// Phase 2: All admin access control decisions are logged for audit purposes.
    fn validate_admin_access(&self, caller: &str, permissions: &[String]) -> Result<(), RuntimeError> {
        // Check if caller is owner
        if caller == self.metadata.owner {
            // Phase 2: Audit log owner access
            crate::stdlib::log::audit("state_isolation_admin_access_allowed", {
                let mut data = std::collections::HashMap::new();
                data.insert("contract_address".to_string(), Value::String(self.contract_address.clone()));
                data.insert("contract_name".to_string(), Value::String(self.metadata.contract_name.clone()));
                data.insert("caller".to_string(), Value::String(caller.to_string()));
                data.insert("reason".to_string(), Value::String("owner_access".to_string()));
                data
            }, Some("state_isolation"));
            return Ok(());
        }

        // Check if caller has admin permission
        if permissions.contains(&"admin".to_string()) || permissions.contains(&"contract_admin".to_string()) {
            // Phase 2: Audit log admin permission access
            crate::stdlib::log::audit("state_isolation_admin_access_allowed", {
                let mut data = std::collections::HashMap::new();
                data.insert("contract_address".to_string(), Value::String(self.contract_address.clone()));
                data.insert("contract_name".to_string(), Value::String(self.metadata.contract_name.clone()));
                data.insert("caller".to_string(), Value::String(caller.to_string()));
                data.insert("reason".to_string(), Value::String("admin_permission".to_string()));
                data.insert("caller_permissions".to_string(), Value::List(permissions.iter().map(|p| Value::String(p.clone())).collect()));
                data
            }, Some("state_isolation"));
            return Ok(());
        }

        // Phase 2: Audit log admin access denial
        crate::stdlib::log::audit("state_isolation_admin_access_denied", {
            let mut data = std::collections::HashMap::new();
            data.insert("contract_address".to_string(), Value::String(self.contract_address.clone()));
            data.insert("contract_name".to_string(), Value::String(self.metadata.contract_name.clone()));
            data.insert("caller".to_string(), Value::String(caller.to_string()));
            data.insert("owner".to_string(), Value::String(self.metadata.owner.clone()));
            data.insert("reason".to_string(), Value::String("insufficient_admin_permissions".to_string()));
            data.insert("caller_permissions".to_string(), Value::List(permissions.iter().map(|p| Value::String(p.clone())).collect()));
            data
        }, Some("state_isolation"));

        Err(RuntimeError::PermissionDenied("Admin permission required".to_string()))
    }

    /// Estimate the serialized size of a value (for gas calculation)
    fn estimate_value_size(&self, value: &Value) -> usize {
        match value {
            Value::Null => 4,
            Value::Bool(_) => 4,
            Value::Int(_) => 8,
            Value::Float(_) => 8,
            Value::String(s) => s.len(),
            Value::List(arr) => arr.iter().map(|v| self.estimate_value_size(v)).sum::<usize>() + (arr.len() * 4),
            Value::Map(obj) => obj.iter()
                .map(|(k, v)| k.len() + self.estimate_value_size(v))
                .sum::<usize>() + (obj.len() * 8),
            Value::Array(arr) => arr.iter().map(|v| self.estimate_value_size(v)).sum::<usize>() + (arr.len() * 4),
            Value::Result(ok, err) => self.estimate_value_size(ok) + self.estimate_value_size(err) + 8,
            Value::Option(Some(v)) => self.estimate_value_size(v) + 4,
            Value::Option(None) => 4,
            Value::Set(s) => s.iter().map(|v| v.len()).sum::<usize>() + (s.len() * 4),
            Value::Struct(name, fields) => name.len() + fields.iter()
                .map(|(k, v)| k.len() + self.estimate_value_size(v))
                .sum::<usize>() + (fields.len() * 8),
            Value::Closure(id) => id.len() + 8,
        }
    }

    /// Calculate checksum of current state
    fn calculate_state_checksum(&self, state: &HashMap<String, Value>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Sort keys for deterministic hashing
        let mut sorted_keys: Vec<_> = state.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            key.hash(&mut hasher);
            if let Some(value) = state.get(key) {
                self.hash_value(value, &mut hasher);
            }
        }

        format!("{:x}", hasher.finish())
    }

    /// Hash a value recursively
    fn hash_value(&self, value: &Value, hasher: &mut std::collections::hash_map::DefaultHasher) {
        use std::hash::Hash;

        match value {
            Value::Null => 0u8.hash(hasher),
            Value::Bool(b) => b.hash(hasher),
            Value::Int(i) => i.hash(hasher),
            Value::Float(f) => f.to_bits().hash(hasher),
            Value::String(s) => s.hash(hasher),
            Value::List(arr) => {
                arr.len().hash(hasher);
                for item in arr {
                    self.hash_value(item, hasher);
                }
            }
            Value::Map(obj) => {
                obj.len().hash(hasher);
                let mut sorted_keys: Vec<_> = obj.keys().collect();
                sorted_keys.sort();
                for key in sorted_keys {
                    key.hash(hasher);
                    if let Some(val) = obj.get(key) {
                        self.hash_value(val, hasher);
                    }
                }
            }
            Value::Array(arr) => {
                arr.len().hash(hasher);
                for item in arr {
                    self.hash_value(item, hasher);
                }
            }
            Value::Result(ok, err) => {
                self.hash_value(ok, hasher);
                self.hash_value(err, hasher);
            }
            Value::Option(Some(v)) => {
                1u8.hash(hasher);
                self.hash_value(v, hasher);
            }
            Value::Option(None) => {
                0u8.hash(hasher);
            }
            Value::Set(s) => {
                s.len().hash(hasher);
                let mut sorted_items: Vec<_> = s.iter().collect();
                sorted_items.sort();
                for item in sorted_items {
                    item.hash(hasher);
                }
            }
            Value::Struct(name, fields) => {
                name.hash(hasher);
                fields.len().hash(hasher);
                let mut sorted_keys: Vec<_> = fields.keys().collect();
                sorted_keys.sort();
                for key in sorted_keys {
                    key.hash(hasher);
                    if let Some(val) = fields.get(key) {
                        self.hash_value(val, hasher);
                    }
                }
            }
            Value::Closure(id) => id.hash(hasher),
        }
    }

    /// Get state metadata
    pub fn get_metadata(&self) -> &StateMetadata {
        &self.metadata
    }

    /// Get access control info
    pub fn get_access_control(&self) -> &AccessControl {
        &self.access_control
    }

    /// Create a state snapshot for backup/rollback
    pub fn create_snapshot(&self) -> Result<StateSnapshot, RuntimeError> {
        let state = self.state.read().map_err(|_| {
            RuntimeError::General("Failed to acquire read lock for snapshot".to_string())
        })?;

        Ok(StateSnapshot {
            contract_address: self.contract_address.clone(),
            state_data: state.clone(),
            metadata: self.metadata.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Restore from a state snapshot (admin only)
    pub fn restore_from_snapshot(&mut self, snapshot: StateSnapshot, caller: &str, permissions: &[String]) -> Result<(), RuntimeError> {
        // Validate admin access
        self.validate_admin_access(caller, permissions)?;

        // Verify snapshot is for this contract
        if snapshot.contract_address != self.contract_address {
            return Err(RuntimeError::General("Snapshot contract address mismatch".to_string()));
        }

        let mut state = self.state.write().map_err(|_| {
            RuntimeError::General("Failed to acquire write lock for restore".to_string())
        })?;

        // Restore state
        *state = snapshot.state_data;

        // Update metadata
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.metadata.last_modified = now;
        self.metadata.state_version += 1;
        self.metadata.checksum = self.calculate_state_checksum(&state);

        Ok(())
    }
}

/// State snapshot for backup and rollback functionality
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub contract_address: String,
    pub state_data: HashMap<String, Value>,
    pub metadata: StateMetadata,
    pub timestamp: u64,
}

/// Global state isolation manager
pub struct StateIsolationManager {
    contracts: HashMap<String, IsolatedContractState>,
    snapshots: HashMap<String, Vec<StateSnapshot>>, // contract_address -> snapshots
    max_snapshots_per_contract: usize,
}

impl StateIsolationManager {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            snapshots: HashMap::new(),
            max_snapshots_per_contract: 10,
        }
    }

    /// Create a new isolated contract
    pub fn create_contract(
        &mut self,
        contract_address: String,
        contract_name: String,
        owner: String,
        trust_level: String,
    ) -> Result<(), RuntimeError> {
        if self.contracts.contains_key(&contract_address) {
            return Err(RuntimeError::General("Contract already exists".to_string()));
        }

        let contract_state = IsolatedContractState::new(
            contract_address.clone(),
            contract_name,
            owner,
            trust_level,
        );

        self.contracts.insert(contract_address, contract_state);
        Ok(())
    }

    /// Get mutable reference to contract state
    pub fn get_contract_mut(&mut self, contract_address: &str) -> Option<&mut IsolatedContractState> {
        self.contracts.get_mut(contract_address)
    }

    /// Get reference to contract state
    pub fn get_contract(&self, contract_address: &str) -> Option<&IsolatedContractState> {
        self.contracts.get(contract_address)
    }

    /// Create snapshot of contract state
    pub fn create_snapshot(&mut self, contract_address: &str) -> Result<(), RuntimeError> {
        let contract = self.contracts.get(contract_address)
            .ok_or_else(|| RuntimeError::General("Contract not found".to_string()))?;

        let snapshot = contract.create_snapshot()?;

        let snapshots = self.snapshots.entry(contract_address.to_string()).or_insert(Vec::new());
        snapshots.push(snapshot);

        // Keep only the most recent snapshots
        if snapshots.len() > self.max_snapshots_per_contract {
            snapshots.remove(0);
        }

        Ok(())
    }

    /// List available snapshots for a contract
    pub fn get_snapshots(&self, contract_address: &str) -> Vec<&StateSnapshot> {
        self.snapshots.get(contract_address)
            .map(|snapshots| snapshots.iter().collect())
            .unwrap_or_default()
    }

    /// Restore contract from snapshot
    pub fn restore_from_snapshot(
        &mut self,
        contract_address: &str,
        snapshot_timestamp: u64,
        caller: &str,
        permissions: &[String],
    ) -> Result<(), RuntimeError> {
        let snapshots = self.snapshots.get(contract_address)
            .ok_or_else(|| RuntimeError::General("No snapshots found for contract".to_string()))?;

        let snapshot = snapshots.iter()
            .find(|s| s.timestamp == snapshot_timestamp)
            .ok_or_else(|| RuntimeError::General("Snapshot not found".to_string()))?
            .clone();

        let contract = self.contracts.get_mut(contract_address)
            .ok_or_else(|| RuntimeError::General("Contract not found".to_string()))?;

        contract.restore_from_snapshot(snapshot, caller, permissions)?;

        Ok(())
    }

    /// Remove contract and all its snapshots (admin only)
    pub fn remove_contract(&mut self, contract_address: &str, caller: &str, permissions: &[String]) -> Result<(), RuntimeError> {
        if let Some(contract) = self.contracts.get(contract_address) {
            contract.validate_admin_access(caller, permissions)?;
        }

        self.contracts.remove(contract_address);
        self.snapshots.remove(contract_address);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolated_state_creation() {
        let state = IsolatedContractState::new(
            "0x123".to_string(),
            "TestContract".to_string(),
            "owner123".to_string(),
            "decentralized".to_string(),
        );

        assert_eq!(state.contract_address, "0x123");
        assert_eq!(state.metadata.contract_name, "TestContract");
        assert_eq!(state.metadata.owner, "owner123");
        assert_eq!(state.access_control.trust_level, "decentralized");
    }

    #[test]
    fn test_state_read_write() {
        let mut state = IsolatedContractState::new(
            "0x123".to_string(),
            "TestContract".to_string(),
            "owner123".to_string(),
            "decentralized".to_string(),
        );

        let permissions = vec!["write".to_string()];

        // Write a value
        assert!(state.write_value("key1", Value::Int(42), "0x123", &permissions).is_ok());

        // Read the value
        let result = state.read_value("key1", "0x123", &permissions).unwrap();
        assert_eq!(result, Value::Int(42));

        // Read non-existent key
        let result = state.read_value("nonexistent", "0x123", &permissions).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_access_control() {
        let mut state = IsolatedContractState::new(
            "0x123".to_string(),
            "TestContract".to_string(),
            "owner123".to_string(),
            "decentralized".to_string(),
        );

        let permissions = vec!["write".to_string()];

        // Authorized caller should succeed
        assert!(state.write_value("key1", Value::Int(42), "0x123", &permissions).is_ok());

        // Unauthorized caller should fail
        assert!(state.write_value("key2", Value::Int(43), "0x456", &permissions).is_err());
    }

    #[test]
    fn test_read_only_mode() {
        let mut state = IsolatedContractState::new(
            "0x123".to_string(),
            "TestContract".to_string(),
            "owner123".to_string(),
            "decentralized".to_string(),
        );

        let admin_permissions = vec!["admin".to_string()];
        let write_permissions = vec!["write".to_string()];

        // Set to read-only mode
        assert!(state.set_read_only(true, "owner123", &admin_permissions).is_ok());

        // Write should fail in read-only mode
        assert!(state.write_value("key1", Value::Int(42), "0x123", &write_permissions).is_err());

        // Read should still work
        assert!(state.read_value("key1", "0x123", &write_permissions).is_ok());
    }

    #[test]
    fn test_state_snapshots() {
        let mut state = IsolatedContractState::new(
            "0x123".to_string(),
            "TestContract".to_string(),
            "owner123".to_string(),
            "decentralized".to_string(),
        );

        let permissions = vec!["write".to_string()];
        let admin_permissions = vec!["admin".to_string()];

        // Write some initial state
        assert!(state.write_value("key1", Value::Int(42), "0x123", &permissions).is_ok());

        // Create snapshot
        let snapshot = state.create_snapshot().unwrap();
        assert_eq!(snapshot.contract_address, "0x123");

        // Modify state
        assert!(state.write_value("key1", Value::Int(100), "0x123", &permissions).is_ok());
        assert_eq!(state.read_value("key1", "0x123", &permissions).unwrap(), Value::Int(100));

        // Restore from snapshot
        assert!(state.restore_from_snapshot(snapshot, "owner123", &admin_permissions).is_ok());
        assert_eq!(state.read_value("key1", "0x123", &permissions).unwrap(), Value::Int(42));
    }
}

//! Fleet: named set of agents, optionally created from a mold (N instances).
//! Off-chain only. Storage: in-memory; when base path is provided, also persisted to `.dal/fleets.json`.
//! See COMPREHENSIVE_AGENT_AND_MOLD_PLANS.md §5.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// A fleet is a named set of agent IDs. Optionally created from a mold (one mold + N instances).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fleet {
    pub name: String,
    /// If created from mold, the mold source (path or ipfs://cid).
    pub mold_source: Option<String>,
    /// Agent IDs in this fleet (order preserved).
    pub member_ids: Vec<String>,
}

static FLEETS: OnceLock<Mutex<HashMap<String, Fleet>>> = OnceLock::new();

fn fleets() -> std::sync::MutexGuard<'static, HashMap<String, Fleet>> {
    FLEETS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
}

fn path_for_fleets(base: &Path) -> PathBuf {
    base.join(".dal").join("fleets.json")
}

fn load_from_file(base: &Path) -> HashMap<String, Fleet> {
    let p = path_for_fleets(base);
    if let Ok(data) = fs::read_to_string(&p) {
        if let Ok(map) = serde_json::from_str::<HashMap<String, Fleet>>(&data) {
            return map;
        }
    }
    HashMap::new()
}

fn save_to_file(base: &Path, map: &HashMap<String, Fleet>) -> Result<(), String> {
    let p = path_for_fleets(base);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create .dal: {}", e))?;
    }
    let data = serde_json::to_string_pretty(map).map_err(|e| format!("serialize fleets: {}", e))?;
    fs::write(&p, data).map_err(|e| format!("write {}: {}", p.display(), e))?;
    Ok(())
}

fn ensure_loaded(base: Option<&Path>) {
    if let Some(b) = base {
        let loaded = load_from_file(b);
        let mut f = fleets();
        *f = loaded;
    }
}

fn save_if_base(base: Option<&Path>) -> Result<(), String> {
    if let Some(b) = base {
        let f = fleets();
        save_to_file(b, &f)?;
    }
    Ok(())
}

/// Create an empty fleet with the given name. Fails if name already exists.
/// If base is Some, fleet data is persisted to base/.dal/fleets.json.
pub fn create(name: &str, base: Option<&Path>) -> Result<(), String> {
    ensure_loaded(base);
    let name = name.to_string();
    if name.is_empty() {
        return Err("fleet name cannot be empty".to_string());
    }
    let mut f = fleets();
    if f.contains_key(&name) {
        return Err(format!("fleet '{}' already exists", name));
    }
    f.insert(
        name.clone(),
        Fleet {
            name,
            mold_source: None,
            member_ids: Vec::new(),
        },
    );
    drop(f);
    save_if_base(base)
}

/// Create a fleet by spawning N agents from a mold. Same principal/mold/fallback rules as single create.
/// Each agent is named `<fleet_name>_0`, `<fleet_name>_1`, ...; params apply to each.
/// Fleet is persisted to base/.dal/fleets.json.
pub fn create_from_mold(
    name: &str,
    mold_source: &str,
    count: u32,
    base: &Path,
    params: Option<&HashMap<String, String>>,
) -> Result<(), String> {
    if count == 0 {
        return Err("fleet from mold requires count >= 1".to_string());
    }
    if count > 1000 {
        return Err("fleet from mold count capped at 1000".to_string());
    }
    ensure_loaded(Some(base));
    let name = name.to_string();
    let f = fleets();
    if f.contains_key(&name) {
        return Err(format!("fleet '{}' already exists", name));
    }
    drop(f);

    let mut member_ids = Vec::with_capacity(count as usize);
    for i in 0..count {
        let agent_name = format!("{}_{}", name, i);
        let ctx = crate::mold::create_from_mold_source(
            mold_source,
            base,
            Some(agent_name.as_str()),
            params,
        )
        .map_err(|e| format!("spawn agent {} from mold: {}", i, e))?;
        member_ids.push(ctx.agent_id);
    }

    let mut f = fleets();
    f.insert(
        name.clone(),
        Fleet {
            name,
            mold_source: Some(mold_source.to_string()),
            member_ids,
        },
    );
    drop(f);
    save_if_base(Some(base))
}

/// List fleet names. If base is Some, loads from base/.dal/fleets.json first.
pub fn list(base: Option<&Path>) -> Vec<String> {
    ensure_loaded(base);
    let f = fleets();
    let mut names: Vec<String> = f.keys().cloned().collect();
    names.sort();
    names
}

/// Get fleet by name, if it exists. If base is Some, loads from file first.
pub fn show(name: &str, base: Option<&Path>) -> Option<Fleet> {
    ensure_loaded(base);
    let f = fleets();
    f.get(name).cloned()
}

/// Remove a fleet (agents remain in runtime; only the grouping is removed). Returns true if removed.
/// If base is Some, persists after remove.
pub fn delete(name: &str, base: Option<&Path>) -> Result<bool, String> {
    ensure_loaded(base);
    let mut f = fleets();
    let removed = f.remove(name).is_some();
    drop(f);
    if removed {
        save_if_base(base)?;
    }
    Ok(removed)
}

/// Resize fleet to N members. If base is Some, persists after change.
/// - Scale down (N < current): truncates member list to N (agent contexts are not removed).
/// - Scale up (N > current): only allowed when fleet has mold_source; spawns (N - current) new agents from the same mold and appends to members.
/// - Empty fleets (no mold_source) cannot be scaled up.
pub fn scale(name: &str, n: u32, base: Option<&Path>) -> Result<(), String> {
    if n > 1000 {
        return Err("fleet scale count capped at 1000".to_string());
    }
    ensure_loaded(base);
    let base_path = base.ok_or("scale requires a base path for persistence")?;

    let mut f = fleets();
    let fleet = f.get(name).cloned().ok_or_else(|| format!("fleet '{}' not found", name))?;
    let current = fleet.member_ids.len() as u32;

    if n == current {
        drop(f);
        return Ok(());
    }

    if n < current {
        let mut fleet = fleet;
        fleet.member_ids.truncate(n as usize);
        f.insert(name.to_string(), fleet);
        drop(f);
        return save_if_base(Some(base_path));
    }

    // n > current: scale up
    let mold_source = fleet
        .mold_source
        .as_deref()
        .ok_or_else(|| format!("fleet '{}' has no mold; cannot scale up (create from mold first)", name))?;

    let mut member_ids = fleet.member_ids;
    drop(f);

    for i in current..n {
        let agent_name = format!("{}_{}", name, i);
        let ctx = crate::mold::create_from_mold_source(
            mold_source,
            base_path,
            Some(agent_name.as_str()),
            None,
        )
        .map_err(|e| format!("spawn agent {} from mold: {}", i, e))?;
        member_ids.push(ctx.agent_id);
    }

    let mut f = fleets();
    if let Some(fleet) = f.get_mut(name) {
        fleet.member_ids = member_ids;
    }
    drop(f);
    save_if_base(Some(base_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_list_empty_fleet() {
        let name = format!("test_fleet_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        assert!(create(&name, None).is_ok());
        let names = list(None);
        assert!(names.contains(&name));
        assert!(show(&name, None).is_some());
        assert_eq!(show(&name, None).unwrap().member_ids.len(), 0);
        let _ = delete(&name, None);
    }

    #[test]
    fn create_duplicate_fails() {
        let name = format!("dup_fleet_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        assert!(create(&name, None).is_ok());
        assert!(create(&name, None).is_err());
        let _ = delete(&name, None);
    }

    #[test]
    fn scale_requires_base() {
        let name = format!("scale_no_base_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        assert!(create(&name, None).is_ok());
        assert!(scale(&name, 0, None).is_err());
        let _ = delete(&name, None);
    }
}

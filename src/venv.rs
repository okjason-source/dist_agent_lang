//! First-class venv: named execution environment with root, deps, and security profile.
//! See docs/VENV_FIRST_CLASS_DESIGN.md.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Security profile for a venv (strict = allow-list stdlib only; relaxed = current behavior).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VenvProfile {
    Strict,
    Relaxed,
}

impl VenvProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            VenvProfile::Strict => "strict",
            VenvProfile::Relaxed => "relaxed",
        }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "strict" => Some(VenvProfile::Strict),
            "relaxed" => Some(VenvProfile::Relaxed),
            _ => None,
        }
    }
}

impl std::str::FromStr for VenvProfile {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        VenvProfile::from_str(s).ok_or(())
    }
}

/// Persisted venv record (name -> root + profile).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VenvRecord {
    pub root: String,
    pub profile: String,
}

/// In-memory venv with resolved root path and profile.
#[derive(Debug, Clone)]
pub struct Venv {
    pub name: String,
    pub root: PathBuf,
    pub profile: VenvProfile,
}

fn path_for_venvs(base: &Path) -> PathBuf {
    base.join(".dal").join("venvs.json")
}

/// Load venvs from a registry file. Returns empty map if file missing or invalid.
pub fn load_venvs(registry_path: &Path) -> HashMap<String, VenvRecord> {
    if let Ok(data) = fs::read_to_string(registry_path) {
        if let Ok(map) = serde_json::from_str::<HashMap<String, VenvRecord>>(&data) {
            return map;
        }
    }
    HashMap::new()
}

/// Save venvs to a registry file. Creates .dal dir if needed.
pub fn save_venvs(registry_path: &Path, venvs: &HashMap<String, VenvRecord>) -> Result<(), String> {
    if let Some(parent) = registry_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create .dal: {}", e))?;
    }
    let data =
        serde_json::to_string_pretty(venvs).map_err(|e| format!("serialize venvs: {}", e))?;
    fs::write(registry_path, data)
        .map_err(|e| format!("write {}: {}", registry_path.display(), e))?;
    Ok(())
}

/// Project-local registry path: base/.dal/venvs.json.
pub fn project_registry_path(base: &Path) -> PathBuf {
    path_for_venvs(base)
}

/// Global registry path: DAL_VENV_REGISTRY env, or dirs::config_dir()/dal/venvs.json.
pub fn global_registry_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("DAL_VENV_REGISTRY") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    dirs::config_dir().map(|d| d.join("dal").join("venvs.json"))
}

/// Resolve venv by name: project-local first, then global. Returns (Venv, path to registry used).
pub fn resolve_venv(name: &str, project_base: &Path) -> Result<(Venv, PathBuf), String> {
    let proj_path = project_registry_path(project_base);
    let mut venvs = load_venvs(&proj_path);
    if let Some(rec) = venvs.remove(name) {
        let profile = VenvProfile::from_str(&rec.profile).unwrap_or(VenvProfile::Relaxed);
        let root = PathBuf::from(&rec.root);
        if !root.exists() {
            return Err(format!(
                "venv '{}' root does not exist: {}",
                name,
                root.display()
            ));
        }
        return Ok((
            Venv {
                name: name.to_string(),
                root,
                profile,
            },
            proj_path,
        ));
    }
    if let Some(global_path) = global_registry_path() {
        let mut venvs = load_venvs(&global_path);
        if let Some(rec) = venvs.remove(name) {
            let profile = VenvProfile::from_str(&rec.profile).unwrap_or(VenvProfile::Relaxed);
            let root = PathBuf::from(&rec.root);
            if !root.exists() {
                return Err(format!(
                    "venv '{}' root does not exist: {}",
                    name,
                    root.display()
                ));
            }
            return Ok((
                Venv {
                    name: name.to_string(),
                    root,
                    profile,
                },
                global_path,
            ));
        }
    }
    Err(format!("venv '{}' not found", name))
}

/// Create a venv and save to registry. Default profile is Relaxed.
/// If use_global is true, use global registry; else use project_local (base/.dal/venvs.json).
pub fn create_venv(
    name: &str,
    root: &Path,
    profile: VenvProfile,
    registry_path: &Path,
) -> Result<(), String> {
    if name.is_empty() {
        return Err("venv name cannot be empty".to_string());
    }
    let root = root
        .canonicalize()
        .map_err(|e| format!("root {}: {}", root.display(), e))?;
    if !root.is_dir() {
        return Err(format!("root is not a directory: {}", root.display()));
    }
    let mut venvs = load_venvs(registry_path);
    if venvs.contains_key(name) {
        return Err(format!("venv '{}' already exists", name));
    }
    venvs.insert(
        name.to_string(),
        VenvRecord {
            root: root.to_string_lossy().to_string(),
            profile: profile.as_str().to_string(),
        },
    );
    save_venvs(registry_path, &venvs)
}

/// List venv names (and root + profile) from project-local then global.
pub fn list_venvs(project_base: &Path) -> Vec<(String, VenvRecord)> {
    let mut out: Vec<(String, VenvRecord)> = Vec::new();
    let proj_path = project_registry_path(project_base);
    let proj = load_venvs(&proj_path);
    for (k, v) in proj {
        out.push((k, v));
    }
    if let Some(global_path) = global_registry_path() {
        if global_path != proj_path {
            for (k, v) in load_venvs(&global_path) {
                if !out.iter().any(|(n, _)| n == &k) {
                    out.push((k, v));
                }
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Show one venv by name (project then global). Returns (record, registry_path).
pub fn show_venv(name: &str, project_base: &Path) -> Result<(VenvRecord, PathBuf), String> {
    let proj_path = project_registry_path(project_base);
    let venvs = load_venvs(&proj_path);
    if let Some(rec) = venvs.get(name) {
        return Ok((rec.clone(), proj_path));
    }
    if let Some(global_path) = global_registry_path() {
        let venvs = load_venvs(&global_path);
        if let Some(rec) = venvs.get(name) {
            return Ok((rec.clone(), global_path));
        }
    }
    Err(format!("venv '{}' not found", name))
}

/// Delete a venv from the registry where it was found (project or global).
pub fn delete_venv(name: &str, project_base: &Path) -> Result<bool, String> {
    let proj_path = project_registry_path(project_base);
    let mut venvs = load_venvs(&proj_path);
    if venvs.remove(name).is_some() {
        save_venvs(&proj_path, &venvs)?;
        return Ok(true);
    }
    if let Some(global_path) = global_registry_path() {
        let mut venvs = load_venvs(&global_path);
        if venvs.remove(name).is_some() {
            save_venvs(&global_path, &venvs)?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// Namespaces allowed when profile is Strict (no sh, no service; chain, crypto, log, config, etc.).
pub const STRICT_ALLOWED_NAMESPACES: &[&str] = &[
    "chain", "crypto", "log", "config", "key", "auth", "evolve", "sync", "json", "test",
];

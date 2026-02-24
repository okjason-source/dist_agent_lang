//! M3: dal.toml and lockfile — parse [dependencies], resolve path deps, read/write dal.lock.
//! Path-only resolution first; no registry fetch.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest not found: {0}")]
    NotFound(PathBuf),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("invalid dependency spec for '{name}': {message}")]
    InvalidDependency { name: String, message: String },
}

/// Dependency spec from dal.toml: version string or path.
#[derive(Debug, Clone)]
pub enum DependencySpec {
    Version(String),
    Path(PathBuf),
}

/// Parsed [dependencies] section: package name -> spec.
pub type DependenciesMap = HashMap<String, DependencySpec>;

/// Resolved dependencies: package name -> absolute package root directory.
pub type ResolvedDeps = HashMap<String, PathBuf>;

/// Parse dal.toml and return [dependencies] (path or version).
pub fn parse_dependencies(manifest_path: &Path) -> Result<DependenciesMap, ManifestError> {
    let content = std::fs::read_to_string(manifest_path)
        .map_err(|_| ManifestError::NotFound(manifest_path.to_path_buf()))?;
    let table: toml::Table = toml::from_str(&content)?;
    let deps_table = match table.get("dependencies") {
        Some(toml::Value::Table(t)) => t,
        None => return Ok(HashMap::new()),
        _ => return Err(ManifestError::InvalidDependency {
            name: "dependencies".to_string(),
            message: "must be a table".to_string(),
        }),
    };
    let project_root = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let mut out = HashMap::new();
    for (name, val) in deps_table {
        let spec = match val {
            toml::Value::String(s) => DependencySpec::Version(s.clone()),
            toml::Value::Table(t) => {
                if let Some(toml::Value::String(p)) = t.get("path") {
                    let path = project_root.join(p);
                    DependencySpec::Path(path)
                } else {
                    return Err(ManifestError::InvalidDependency {
                        name: name.clone(),
                        message: "inline table must have 'path' for M3 (path-only)".to_string(),
                    });
                }
            }
            _ => {
                return Err(ManifestError::InvalidDependency {
                    name: name.clone(),
                    message: "dependency must be string (version) or table { path = \"...\" }".to_string(),
                });
            }
        };
        out.insert(name.clone(), spec);
    }
    Ok(out)
}

/// Resolve path dependencies to canonical absolute paths. Version-only deps are skipped (no registry).
pub fn resolve_dependencies(manifest_path: &Path) -> Result<ResolvedDeps, ManifestError> {
    let deps = parse_dependencies(manifest_path)?;
    let mut resolved = HashMap::new();
    for (name, spec) in deps {
        match spec {
            DependencySpec::Path(p) => {
                let canonical = p.canonicalize().map_err(|e| ManifestError::InvalidDependency {
                    name: name.clone(),
                    message: format!("path {}: {}", p.display(), e),
                })?;
                if !canonical.is_dir() {
                    return Err(ManifestError::InvalidDependency {
                        name,
                        message: format!("{} is not a directory", canonical.display()),
                    });
                }
                resolved.insert(name, canonical);
            }
            DependencySpec::Version(_) => {
                // M3: no registry; skip or optional: leave unresolved and warn
            }
        }
    }
    Ok(resolved)
}

/// Lockfile format: [dependencies] name = "absolute_path"
const LOCKFILE_SECTION: &str = "[dependencies]\n";

/// Write dal.lock next to manifest_path with resolved paths.
pub fn write_lockfile(manifest_path: &Path, resolved: &ResolvedDeps) -> Result<(), ManifestError> {
    let lock_path = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("dal.lock");
    let mut content = String::from(LOCKFILE_SECTION);
    for (name, path) in resolved {
        let path_str = path.to_string_lossy();
        content.push_str(&format!("{} = \"{}\"\n", name, path_str.replace('\\', "/")));
    }
    std::fs::write(lock_path, content)?;
    Ok(())
}

/// Read dal.lock and return name -> path. If lockfile missing, resolve from manifest and return.
pub fn load_resolved_deps(manifest_path: &Path) -> Result<ResolvedDeps, ManifestError> {
    let project_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let lock_path = project_dir.join("dal.lock");
    if lock_path.exists() {
        let content = std::fs::read_to_string(&lock_path)?;
        let table: toml::Table = toml::from_str(&content)?;
        let deps = match table.get("dependencies") {
            Some(toml::Value::Table(t)) => t,
            _ => return Ok(HashMap::new()),
        };
        let mut out = HashMap::new();
        for (name, val) in deps {
            if let toml::Value::String(p) = val {
                let path = PathBuf::from(p);
                let abs = if path.is_absolute() {
                    path
                } else {
                    project_dir.join(p)
                };
                out.insert(name.clone(), abs);
            }
        }
        return Ok(out);
    }
    // No lockfile: resolve from manifest (path deps only)
    resolve_dependencies(manifest_path)
}

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
    #[error("registry: {0}")]
    Registry(String),
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

/// Version metadata for lockfile: name -> (version, source). Only for version deps.
pub type LockfileVersionMeta = HashMap<String, (String, String)>;

/// Package identity from [package] section (for publish).
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

/// Parse [package] name and version from dal.toml.
pub fn parse_package_info(manifest_path: &Path) -> Result<PackageInfo, ManifestError> {
    let content = std::fs::read_to_string(manifest_path)
        .map_err(|_| ManifestError::NotFound(manifest_path.to_path_buf()))?;
    let table: toml::Table = toml::from_str(&content)?;
    let package = match table.get("package") {
        Some(toml::Value::Table(t)) => t,
        _ => {
            return Err(ManifestError::InvalidDependency {
                name: "package".to_string(),
                message: "missing [package] section".to_string(),
            })
        }
    };
    let name = package
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ManifestError::InvalidDependency {
            name: "package.name".to_string(),
            message: "missing or invalid".to_string(),
        })?
        .to_string();
    let version = package
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ManifestError::InvalidDependency {
            name: "package.version".to_string(),
            message: "missing or invalid".to_string(),
        })?
        .to_string();
    Ok(PackageInfo { name, version })
}

/// Parse dal.toml and return [dependencies] (path or version).
pub fn parse_dependencies(manifest_path: &Path) -> Result<DependenciesMap, ManifestError> {
    let content = std::fs::read_to_string(manifest_path)
        .map_err(|_| ManifestError::NotFound(manifest_path.to_path_buf()))?;
    let table: toml::Table = toml::from_str(&content)?;
    let deps_table = match table.get("dependencies") {
        Some(toml::Value::Table(t)) => t,
        None => return Ok(HashMap::new()),
        _ => {
            return Err(ManifestError::InvalidDependency {
                name: "dependencies".to_string(),
                message: "must be a table".to_string(),
            })
        }
    };
    let project_root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
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
                    message: "dependency must be string (version) or table { path = \"...\" }"
                        .to_string(),
                });
            }
        };
        out.insert(name.clone(), spec);
    }
    Ok(out)
}

/// Resolve dependencies: path deps canonicalized; version deps fetched from registry and cached.
/// Returns (resolved deps, version metadata for lockfile).
pub fn resolve_dependencies(
    manifest_path: &Path,
) -> Result<(ResolvedDeps, LockfileVersionMeta), ManifestError> {
    let deps = parse_dependencies(manifest_path)?;
    let mut resolved = HashMap::new();
    let mut version_meta = HashMap::new();
    for (name, spec) in deps {
        match spec {
            DependencySpec::Path(p) => {
                let canonical = p
                    .canonicalize()
                    .map_err(|e| ManifestError::InvalidDependency {
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
            DependencySpec::Version(version_request) => {
                let (path, version, source) =
                    crate::registry::resolve_and_fetch_with_meta(&name, &version_request)
                        .map_err(|e| ManifestError::Registry(e.to_string()))?;
                version_meta.insert(name.clone(), (version, source));
                resolved.insert(name, path);
            }
        }
    }
    Ok((resolved, version_meta))
}

/// Lockfile format: [dependencies] name = "absolute_path"
const LOCKFILE_SECTION: &str = "[dependencies]\n";

/// Write dal.lock next to manifest_path with resolved paths and optional version metadata.
pub fn write_lockfile(
    manifest_path: &Path,
    resolved: &ResolvedDeps,
    version_meta: &LockfileVersionMeta,
) -> Result<(), ManifestError> {
    let lock_path = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("dal.lock");
    let quote_key = |k: &str| -> String {
        if k.chars().any(|c| c == '@' || c == '/' || c == '.') {
            format!("\"{}\"", k.replace('\\', "\\\\").replace('"', "\\\""))
        } else {
            k.to_string()
        }
    };
    let mut content = String::from(LOCKFILE_SECTION);
    for (name, path) in resolved {
        let path_str = path.to_string_lossy();
        content.push_str(&format!(
            "{} = \"{}\"\n",
            quote_key(name),
            path_str.replace('\\', "/")
        ));
    }
    if !version_meta.is_empty() {
        content.push_str("\n[metadata]\n");
        for (name, (version, source)) in version_meta {
            let v_esc = version.replace('\\', "\\\\").replace('"', "\\\"");
            let s_esc = source.replace('\\', "\\\\").replace('"', "\\\"");
            content.push_str(&format!(
                "{} = {{ version = \"{}\", source = \"{}\" }}\n",
                quote_key(name),
                v_esc,
                s_esc
            ));
        }
    }
    std::fs::write(lock_path, content)?;
    Ok(())
}

/// Read dal.lock and return name -> path. If lockfile missing, resolve from manifest and return.
/// When a cached path is missing, re-fetches from [metadata] source if available.
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
        let metadata = table.get("metadata").and_then(|v| v.as_table());
        let mut out = HashMap::new();
        for (name, val) in deps {
            if let toml::Value::String(p) = val {
                let path = PathBuf::from(p);
                let abs = if path.is_absolute() {
                    path
                } else {
                    project_dir.join(p)
                };
                if abs.exists() {
                    out.insert(name.clone(), abs);
                } else if let Some(meta) = metadata.and_then(|t| t.get(name)).and_then(|v| v.as_table()) {
                    let version = meta
                        .get("version")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ManifestError::InvalidDependency {
                            name: name.clone(),
                            message: "[metadata] entry missing 'version'".to_string(),
                        })?;
                    let source = meta
                        .get("source")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ManifestError::InvalidDependency {
                            name: name.clone(),
                            message: "[metadata] entry missing 'source'".to_string(),
                        })?;
                    let path = crate::registry::fetch_and_cache(name, version, source)
                        .map_err(|e| ManifestError::Registry(e.to_string()))?;
                    out.insert(name.clone(), path);
                } else {
                    return Err(ManifestError::InvalidDependency {
                        name: name.clone(),
                        message: format!(
                            "cached path {} does not exist and no [metadata] to re-fetch",
                            abs.display()
                        ),
                    });
                }
            }
        }
        return Ok(out);
    }
    // No lockfile: resolve from manifest
    let (resolved, _) = resolve_dependencies(manifest_path)?;
    Ok(resolved)
}

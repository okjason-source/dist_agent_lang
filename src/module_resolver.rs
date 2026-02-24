//! Module resolution (M2/M3): resolve import paths to stdlib, relative files, or packages.
//!
//! Resolution order: stdlib first, then relative path, then package (from dal.toml/lockfile).
//! Detects import cycles when resolving relative files.

use crate::manifest::ResolvedDeps;
use crate::parser::ast::{ImportStatement, Program, Statement};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Known stdlib namespaces (must match engine's `call_namespace_function` and stdlib modules).
const KNOWN_STDLIB: &[&str] = &[
    "add_sol",
    "admin",
    "agent",
    "ai",
    "aml",
    "auth",
    "chain",
    "cloudadmin",
    "config",
    "cross_chain_security",
    "crypto",
    "crypto_signatures",
    "database",
    "desktop",
    "iot",
    "key",
    "kyc",
    "log",
    "mobile",
    "mold",
    "oracle",
    "secure_auth",
    "service",
    "sync",
    "test",
    "trust",
    "web",
];

/// Result of resolving a single import path.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedImport {
    /// Built-in stdlib namespace (e.g. stdlib::chain → "chain").
    Stdlib(String),
    /// Resolved path to a DAL file (absolute or relative to cwd).
    RelativeFile(PathBuf),
    /// Package dependency (name and resolved root path). M3 will fill this.
    Package { name: String, path: PathBuf },
}

/// Errors produced during module resolution.
#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("unknown module: {0}")]
    UnknownModule(String),

    #[error("file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("import cycle detected: {0}")]
    CycleDetected(String),

    #[error("relative import requires an entry path")]
    RelativeWithoutEntryPath,

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("package resolution not available (M3): {0}")]
    PackageNotAvailable(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error in {path}: {message}")]
    ParseError { path: PathBuf, message: String },
}

/// Resolved import with the original AST node (for later binding).
#[derive(Debug, Clone)]
pub struct ResolvedImportEntry {
    pub import: ImportStatement,
    pub resolved: ResolvedImport,
}

/// Resolves import paths for a program. M2: stdlib + relative paths. M3: + package from lockfile.
#[derive(Debug, Default)]
pub struct ModuleResolver {
    /// Optional root directory for resolving relative paths and dal.toml (project root).
    pub root_dir: Option<PathBuf>,
    /// M3: Resolved package name -> package root directory (from dal.lock or dal.toml).
    pub dependencies: Option<ResolvedDeps>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_root_dir(mut self, root: PathBuf) -> Self {
        self.root_dir = Some(root);
        self
    }

    /// M3: Use resolved dependencies (package name -> package root path) for package imports.
    pub fn with_dependencies(mut self, deps: ResolvedDeps) -> Self {
        self.dependencies = Some(deps);
        self
    }

    /// Resolve a single import path. Resolution order: stdlib first, then relative, then package.
    pub fn resolve(
        &self,
        import_path: &str,
        current_file_dir: Option<&Path>,
    ) -> Result<ResolvedImport, ResolveError> {
        let path = import_path.trim();

        // 1. Stdlib: "stdlib::name" or identifier path starting with stdlib::
        if path.starts_with("stdlib::") {
            let name = path.strip_prefix("stdlib::").unwrap().trim();
            if name.is_empty() {
                return Err(ResolveError::UnknownModule(import_path.to_string()));
            }
            let namespace = name.split("::").next().unwrap_or(name);
            if KNOWN_STDLIB.contains(&namespace) {
                return Ok(ResolvedImport::Stdlib(namespace.to_string()));
            }
            return Err(ResolveError::UnknownModule(format!(
                "unknown stdlib namespace '{}' (known: {})",
                namespace,
                KNOWN_STDLIB[..KNOWN_STDLIB.len().min(5)].join(", ")
            )));
        }

        // 2. Relative path: "./..." or "../..."
        if path.starts_with("./") || path.starts_with("../") {
            let base = current_file_dir
                .or_else(|| self.root_dir.as_deref())
                .ok_or(ResolveError::RelativeWithoutEntryPath)?;
            let joined = base.join(path);
            let canonical = joined
                .canonicalize()
                .map_err(|_| ResolveError::FileNotFound(joined.clone()))?;
            if !canonical.is_file() {
                return Err(ResolveError::FileNotFound(canonical));
            }
            return Ok(ResolvedImport::RelativeFile(canonical));
        }

        // 3. String that looks like a path
        if path.contains('/') || path.ends_with(".dal") {
            let base = current_file_dir
                .or_else(|| self.root_dir.as_deref())
                .ok_or(ResolveError::RelativeWithoutEntryPath)?;
            let joined = base.join(path);
            let canonical = joined
                .canonicalize()
                .map_err(|_| ResolveError::FileNotFound(joined))?;
            if !canonical.is_file() {
                return Err(ResolveError::FileNotFound(canonical));
            }
            return Ok(ResolvedImport::RelativeFile(canonical));
        }

        // 4. Package name (M3): look up in dependencies
        if let Some(ref deps) = self.dependencies {
            if let Some(package_root) = deps.get(path) {
                return Ok(ResolvedImport::Package {
                    name: path.to_string(),
                    path: package_root.clone(),
                });
            }
        }
        Err(ResolveError::PackageNotAvailable(format!(
            "package '{}' not in [dependencies]; run `dal install` or add it to dal.toml",
            path
        )))
    }

    /// Collect and resolve all imports in a program. Does not follow relative files (no cycle check yet).
    pub fn resolve_program_imports(
        &self,
        program: &Program,
        entry_path: Option<&Path>,
    ) -> Result<Vec<ResolvedImportEntry>, ResolveError> {
        let current_dir = entry_path.and_then(|p| p.parent());
        let mut out = Vec::new();
        for stmt in &program.statements {
            if let Statement::Import(import) = stmt {
                let resolved = self.resolve(&import.path, current_dir)?;
                out.push(ResolvedImportEntry {
                    import: import.clone(),
                    resolved,
                });
            }
        }
        Ok(out)
    }

    /// Resolve program and recursively resolve relative imports with cycle detection.
    /// Returns a flat list of all resolved imports (entry file first, then dependencies in order).
    pub fn resolve_program_with_cycles(
        &self,
        program: &Program,
        entry_path: Option<&Path>,
        parse_fn: impl Fn(&str) -> Result<Program, String>,
    ) -> Result<Vec<ResolvedImportEntry>, ResolveError> {
        let mut stack: Vec<PathBuf> = vec![];
        let mut result = Vec::new();
        self.resolve_program_recursive(program, entry_path, &mut stack, &mut result, &parse_fn)?;
        Ok(result)
    }

    fn resolve_program_recursive(
        &self,
        program: &Program,
        current_file: Option<&Path>,
        stack: &mut Vec<PathBuf>,
        result: &mut Vec<ResolvedImportEntry>,
        parse_fn: &impl Fn(&str) -> Result<Program, String>,
    ) -> Result<(), ResolveError> {
        let current_dir = current_file.and_then(|p| p.parent());
        for stmt in &program.statements {
            if let Statement::Import(import) = stmt {
                let resolved = self.resolve(&import.path, current_dir)?;
                match &resolved {
                    ResolvedImport::RelativeFile(path) => {
                        let canonical = path.clone();
                        if stack.contains(&canonical) {
                            let chain: Vec<String> = stack
                                .iter()
                                .chain(std::iter::once(&canonical))
                                .map(|p| p.display().to_string())
                                .collect();
                            return Err(ResolveError::CycleDetected(chain.join(" -> ")));
                        }
                        let source = std::fs::read_to_string(&canonical)
                            .map_err(|_| ResolveError::FileNotFound(canonical.clone()))?;
                        let dep_program =
                            parse_fn(&source).map_err(|msg| ResolveError::ParseError {
                                path: canonical.clone(),
                                message: msg,
                            })?;
                        stack.push(canonical.clone());
                        self.resolve_program_recursive(
                            &dep_program,
                            Some(&canonical),
                            stack,
                            result,
                            parse_fn,
                        )?;
                        stack.pop();
                    }
                    ResolvedImport::Package {
                        path: package_root, ..
                    } => {
                        let entry_path = package_entry_path(package_root).ok_or_else(|| {
                            ResolveError::FileNotFound(package_root.join("main.dal"))
                        })?;
                        if stack.contains(&entry_path) {
                            let chain: Vec<String> = stack
                                .iter()
                                .chain(std::iter::once(&entry_path))
                                .map(|p| p.display().to_string())
                                .collect();
                            return Err(ResolveError::CycleDetected(chain.join(" -> ")));
                        }
                        let source = std::fs::read_to_string(&entry_path)
                            .map_err(|_| ResolveError::FileNotFound(entry_path.clone()))?;
                        let dep_program =
                            parse_fn(&source).map_err(|msg| ResolveError::ParseError {
                                path: entry_path.clone(),
                                message: msg,
                            })?;
                        stack.push(entry_path.clone());
                        self.resolve_program_recursive(
                            &dep_program,
                            Some(&entry_path),
                            stack,
                            result,
                            parse_fn,
                        )?;
                        stack.pop();
                    }
                    ResolvedImport::Stdlib(_) => {}
                }
                result.push(ResolvedImportEntry {
                    import: import.clone(),
                    resolved,
                });
            }
        }
        Ok(())
    }
}

/// M3: Return path to package entry file (main.dal or lib.dal) if present.
fn package_entry_path(package_root: &PathBuf) -> Option<PathBuf> {
    let main = package_root.join("main.dal");
    let lib = package_root.join("lib.dal");
    if main.exists() {
        Some(main)
    } else if lib.exists() {
        Some(lib)
    } else {
        None
    }
}

/// Convenience: resolve all imports in a program (no recursive cycle check).
pub fn resolve_imports(
    program: &Program,
    entry_path: Option<&Path>,
) -> Result<Vec<ResolvedImportEntry>, ResolveError> {
    ModuleResolver::new().resolve_program_imports(program, entry_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_stdlib() {
        let r = ModuleResolver::new();
        let out = r.resolve("stdlib::chain", None).unwrap();
        assert!(matches!(out, ResolvedImport::Stdlib(s) if s == "chain"));
        let out = r.resolve("stdlib::ai", None).unwrap();
        assert!(matches!(out, ResolvedImport::Stdlib(s) if s == "ai"));
    }

    #[test]
    fn test_resolve_stdlib_unknown() {
        let r = ModuleResolver::new();
        let e = r.resolve("stdlib::nonexistent", None).unwrap_err();
        assert!(matches!(e, ResolveError::UnknownModule(_)));
    }

    #[test]
    fn test_resolve_relative_without_entry() {
        let r = ModuleResolver::new();
        let e = r.resolve("./foo.dal", None).unwrap_err();
        assert!(matches!(e, ResolveError::RelativeWithoutEntryPath));
    }

    #[test]
    fn test_resolve_package_not_available() {
        let r = ModuleResolver::new();
        let e = r.resolve("some_package", None).unwrap_err();
        assert!(matches!(e, ResolveError::PackageNotAvailable(_)));
    }
}

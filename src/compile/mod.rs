//! CT1/CT2/CT3/CT4: Compiler pipeline — driver, CompileBackend; blockchain, wasm, native = real codegen.
//! Wire: module resolution so dal build works with imports and multi-file projects.
//! See docs/development/implementation/COMPILE_TARGET_IMPLEMENTATION_PLAN.md.

mod blockchain;
mod native;
mod wasm;

use crate::lexer::tokens::CompilationTarget;
use crate::parser::ast::{Program, ServiceStatement, Statement};
use crate::module_resolver::{ModuleResolver, ResolvedImport};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of a compile run (stub: manifest of what would be compiled).
#[derive(Debug, Clone)]
pub struct CompileArtifacts {
    pub target: String,
    pub service_names: Vec<String>,
    /// Artifact paths (.stub for stub backend; .bin/.abi for blockchain, etc.).
    pub artifact_paths: Vec<PathBuf>,
    /// True if backend did not emit real codegen (CT1 stub).
    pub stub: bool,
}

/// Options for a single compile invocation.
#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub entry_path: PathBuf,
    pub target: CompilationTarget,
    pub output_dir: PathBuf,
}

/// Compile driver error.
#[derive(Debug)]
pub enum CompileError {
    Parse(String),
    Io(std::io::Error),
    NoBackend { target: String },
    CompilerNotFound { target: String, hint: String },
    Backend(String),
    /// Import resolution failed (cycle, missing file, etc.)
    Resolve(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Parse(s) => write!(f, "Parse error: {}", s),
            CompileError::Io(e) => write!(f, "I/O error: {}", e),
            CompileError::NoBackend { target } => {
                write!(f, "No compile backend registered for target '{}'", target)
            }
            CompileError::CompilerNotFound { target, hint } => {
                write!(f, "Target compiler not found for '{}'. {}", target, hint)
            }
            CompileError::Backend(s) => write!(f, "Backend error: {}", s),
            CompileError::Resolve(s) => write!(f, "Import resolution failed: {}", s),
        }
    }
}

impl std::error::Error for CompileError {}

/// Backend trait: one implementation per target (blockchain, wasm, native, etc.).
pub trait CompileBackend: Send + Sync {
    /// Compile the selected services. CT1: stub returns manifest only.
    fn compile(
        &self,
        program: &Program,
        services: &[&ServiceStatement],
        opts: &CompileOptions,
    ) -> Result<CompileArtifacts, CompileError>;
}

/// Stub backend: no codegen; returns a manifest of service names and checks compiler availability.
struct StubBackend {
    target: CompilationTarget,
    check_cmd: &'static str,
    install_hint: &'static str,
}

impl CompileBackend for StubBackend {
    fn compile(
        &self,
        _program: &Program,
        services: &[&ServiceStatement],
        opts: &CompileOptions,
    ) -> Result<CompileArtifacts, CompileError> {
        if !check_compiler_available(self.check_cmd) {
            return Err(CompileError::CompilerNotFound {
                target: self.target.to_string(),
                hint: self.install_hint.to_string(),
            });
        }
        let service_names: Vec<String> = services.iter().map(|s| s.name.clone()).collect();
        Ok(CompileArtifacts {
            target: self.target.to_string(),
            service_names: service_names.clone(),
            artifact_paths: service_names
                .iter()
                .map(|n| opts.output_dir.join(format!("{}.stub", n)))
                .collect(),
            stub: true,
        })
    }
}

fn check_compiler_available(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns the backend for the given target (blockchain, wasm, native = real codegen; mobile/edge = stub).
pub fn get_backend(target: &CompilationTarget) -> Option<Box<dyn CompileBackend>> {
    let b: Box<dyn CompileBackend> = match *target {
        CompilationTarget::Blockchain => Box::new(blockchain::BlockchainBackend),
        CompilationTarget::WebAssembly => Box::new(wasm::WasmBackend),
        CompilationTarget::Native => Box::new(native::NativeBackend),
        CompilationTarget::Mobile | CompilationTarget::Edge => Box::new(StubBackend {
            target: target.clone(),
            check_cmd: "rustc",
            install_hint: "Install Rust: https://rustup.rs (mobile/edge toolchains TBD)",
        }),
    };
    Some(b)
}

/// Select services from program that have compilation_target == given target.
pub fn select_services_for_target<'a>(
    program: &'a Program,
    target: &CompilationTarget,
) -> Vec<&'a ServiceStatement> {
    program
        .statements
        .iter()
        .filter_map(|s| {
            if let Statement::Service(svc) = s {
                if let Some(ref info) = svc.compilation_target {
                    if info.target == *target {
                        return Some(svc);
                    }
                }
            }
            None
        })
        .collect()
}

/// Package entry file (main.dal or lib.dal) for resolution.
fn package_entry_path(package_root: &Path) -> Option<PathBuf> {
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

/// Run the compiler driver: parse entry, resolve imports (if any), merge programs, select services, call backend.
pub fn run_compile(
    entry_path: PathBuf,
    target: CompilationTarget,
    output_dir: PathBuf,
    source: &str,
) -> Result<CompileArtifacts, CompileError> {
    let program = crate::parse_source(source).map_err(|e| CompileError::Parse(e.to_string()))?;

    let program = if program.statements.iter().any(|s| matches!(s, Statement::Import(_))) {
        let entry_dir = entry_path.parent().unwrap_or_else(|| Path::new("."));
        let mut resolver = ModuleResolver::new().with_root_dir(entry_dir.to_path_buf());
        let manifest_path = entry_dir.join("dal.toml");
        if manifest_path.exists() {
            if let Ok(deps) = crate::manifest::load_resolved_deps(&manifest_path) {
                resolver = resolver.with_dependencies(deps);
            }
        }
        let parse_fn = |s: &str| crate::parse_source(s).map_err(|e| e.to_string());
        let resolved = resolver
            .resolve_program_with_cycles(&program, Some(entry_path.as_path()), parse_fn)
            .map_err(|e| CompileError::Resolve(e.to_string()))?;

        let mut merged = program;
        for entry in &resolved {
            let dep_path = match &entry.resolved {
                ResolvedImport::RelativeFile(p) => p.clone(),
                ResolvedImport::Package { path, .. } => {
                    match package_entry_path(path) {
                        Some(p) => p,
                        None => continue,
                    }
                }
                ResolvedImport::Stdlib(_) => continue,
            };
            let dep_source = std::fs::read_to_string(&dep_path).map_err(CompileError::Io)?;
            let dep_program = crate::parse_source(&dep_source)
                .map_err(|e| CompileError::Parse(format!("{}: {}", dep_path.display(), e)))?;
            let n = dep_program.statements.len();
            merged.statements.extend(dep_program.statements);
            merged.statement_spans.extend((0..n).map(|_| None));
        }
        merged
    } else {
        program
    };

    let services = select_services_for_target(&program, &target);

    let backend = get_backend(&target).ok_or_else(|| CompileError::NoBackend {
        target: target.to_string(),
    })?;

    let opts = CompileOptions {
        entry_path: entry_path.clone(),
        target,
        output_dir: output_dir.clone(),
    };

    let artifacts = backend.compile(&program, &services, &opts)?;

    std::fs::create_dir_all(&output_dir).map_err(CompileError::Io)?;
    let manifest_path = output_dir.join("compile-manifest.json");
    let manifest = serde_json::json!({
        "target": artifacts.target,
        "services": artifacts.service_names,
        "stub": artifacts.stub
    });
    std::fs::write(manifest_path, manifest.to_string()).map_err(CompileError::Io)?;

    Ok(artifacts)
}

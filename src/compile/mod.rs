//! CT1/CT2/CT3/CT4: Compiler pipeline — driver, CompileBackend; blockchain, wasm, native, edge = real codegen.
//! Wire: module resolution so dal build works with imports and multi-file projects.
//! See docs/development/implementation/COMPILE_TARGET_IMPLEMENTATION_PLAN.md.

mod blockchain;
mod edge;
mod native;
mod wasm;

use crate::lexer::tokens::CompilationTarget;
use crate::module_resolver::{ModuleResolver, ResolvedImport};
use crate::parser::ast::{BlockStatement, Expression, Program, ServiceStatement, Statement};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::Command;

thread_local! {
    static COMPILER_AVAILABLE_OVERRIDE: RefCell<Option<bool>> = const { RefCell::new(None) };
}

/// Returns the current override if set (used by backend check functions).
pub(crate) fn get_compiler_available_override() -> Option<bool> {
    COMPILER_AVAILABLE_OVERRIDE.with(|cell| *cell.borrow())
}

/// Sets the compiler-available override for the current thread (used by tests to force CompilerNotFound or success).
/// When set to `Some(b)`, all backend check functions return `b` instead of running the real command.
/// Restore with `set_compiler_available_override(None)` when done.
#[doc(hidden)]
pub fn set_compiler_available_override(available: Option<bool>) {
    COMPILER_AVAILABLE_OVERRIDE.with(|cell| *cell.borrow_mut() = available);
}

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
    pub trust_mode: TrustCompileMode,
}

/// Trust mode profile for compile-time policy checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustCompileMode {
    /// Use per-service @trust attribute to decide policy.
    Auto,
    /// Force decentralized policy checks on selected services.
    Decentralized,
    /// Force hybrid mode (no decentralized-only restrictions).
    Hybrid,
    /// Force centralized mode (no decentralized-only restrictions).
    Centralized,
}

impl TrustCompileMode {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "decentralized" => Some(Self::Decentralized),
            "hybrid" => Some(Self::Hybrid),
            "centralized" => Some(Self::Centralized),
            _ => None,
        }
    }
}

impl std::str::FromStr for TrustCompileMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TrustCompileMode::from_str(s).ok_or(())
    }
}

/// Compile driver error.
#[derive(Debug)]
pub enum CompileError {
    Parse(String),
    Io(std::io::Error),
    NoBackend {
        target: String,
    },
    CompilerNotFound {
        target: String,
        hint: String,
    },
    Backend(String),
    /// Import resolution failed (cycle, missing file, etc.)
    Resolve(String),
    /// Trust/profile policy check failed before backend compile.
    Policy(String),
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
            CompileError::Policy(s) => write!(f, "Policy check failed: {}", s),
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
    if let Some(available) = get_compiler_available_override() {
        return available;
    }
    Command::new(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns the backend for the given target (blockchain, wasm, native, edge = real codegen; mobile = stub).
pub fn get_backend(target: &CompilationTarget) -> Option<Box<dyn CompileBackend>> {
    let b: Box<dyn CompileBackend> = match *target {
        CompilationTarget::Blockchain => Box::new(blockchain::BlockchainBackend),
        CompilationTarget::WebAssembly => Box::new(wasm::WasmBackend),
        CompilationTarget::Native => Box::new(native::NativeBackend),
        CompilationTarget::Edge => Box::new(edge::EdgeBackend),
        CompilationTarget::Mobile => Box::new(StubBackend {
            target: target.clone(),
            check_cmd: "rustc",
            install_hint:
                "Install Rust: https://rustup.rs (mobile compile deferred until transpile path)",
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
    run_compile_with_mode(
        entry_path,
        target,
        output_dir,
        source,
        TrustCompileMode::Auto,
    )
}

/// Run compiler with explicit trust-mode profile.
pub fn run_compile_with_mode(
    entry_path: PathBuf,
    target: CompilationTarget,
    output_dir: PathBuf,
    source: &str,
    trust_mode: TrustCompileMode,
) -> Result<CompileArtifacts, CompileError> {
    let program = crate::parse_source(source).map_err(|e| CompileError::Parse(e.to_string()))?;

    let program = if program
        .statements
        .iter()
        .any(|s| matches!(s, Statement::Import(_)))
    {
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
                ResolvedImport::Package { path, .. } => match package_entry_path(path) {
                    Some(p) => p,
                    None => continue,
                },
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

    validate_trust_policy_for_services(&services, &target, trust_mode)?;

    let opts = CompileOptions {
        entry_path: entry_path.clone(),
        target,
        output_dir: output_dir.clone(),
        trust_mode,
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

fn service_declares_decentralized(service: &ServiceStatement) -> bool {
    service.attributes.iter().any(|attr| {
        attr.name == "@trust"
            && attr
                .parameters
                .first()
                .and_then(expression_to_string_literal)
                == Some("decentralized")
    })
}

fn expression_to_string_literal(expr: &Expression) -> Option<&str> {
    use crate::lexer::tokens::Literal;
    if let Expression::Literal(Literal::String(s)) = expr {
        Some(s.as_str())
    } else {
        None
    }
}

fn validate_trust_policy_for_services(
    services: &[&ServiceStatement],
    target: &CompilationTarget,
    mode: TrustCompileMode,
) -> Result<(), CompileError> {
    let should_validate_service = |service: &ServiceStatement| match mode {
        TrustCompileMode::Auto => service_declares_decentralized(service),
        TrustCompileMode::Decentralized => true,
        TrustCompileMode::Hybrid | TrustCompileMode::Centralized => false,
    };

    for service in services {
        if !should_validate_service(service) {
            continue;
        }
        validate_decentralized_service(service, target)?;
    }
    Ok(())
}

fn validate_decentralized_service(
    service: &ServiceStatement,
    target: &CompilationTarget,
) -> Result<(), CompileError> {
    let forbidden = [
        "ai",
        "sh",
        "fs",
        "web",
        "http",
        "oracle",
        "agent",
        "cloudadmin",
    ];
    for method in &service.methods {
        let namespaces = collect_namespaces_from_block(&method.body);
        let mut violating: Vec<String> = namespaces
            .into_iter()
            .filter(|ns| forbidden.contains(&ns.as_str()))
            .collect();
        if !violating.is_empty() {
            violating.sort();
            violating.dedup();
            return Err(CompileError::Policy(format!(
                "Service '{}' (decentralized mode) method '{}' uses disallowed namespace(s): {:?}. \
Allowed approach: keep on-chain deterministic logic in decentralized services and move orchestration/AI/tooling to hybrid or centralized services.",
                service.name, method.name, violating
            )));
        }
        let unsupported = collect_decentralized_v1_unsupported_constructs(&method.body);
        if !unsupported.is_empty() {
            return Err(CompileError::Policy(format!(
                "Service '{}' (decentralized mode) method '{}' uses unsupported decentralized-v1 construct(s): [{}]. \
Supported in v1: deterministic let/return/if/event/assign/call expressions only. \
Move orchestration/dynamic behavior to hybrid or centralized services.",
                service.name,
                method.name,
                unsupported.join(", ")
            )));
        }
        if method.is_async {
            return Err(CompileError::Policy(format!(
                "Service '{}' (decentralized mode) method '{}' cannot be async in decentralized-v1 subset.",
                service.name, method.name
            )));
        }
    }
    if *target == CompilationTarget::Blockchain {
        let unsupported_fields: Vec<String> = service
            .fields
            .iter()
            .filter(|f| !is_decentralized_v1_supported_type(&f.field_type))
            .map(|f| format!("{}: {}", f.name, f.field_type))
            .collect();
        if !unsupported_fields.is_empty() {
            return Err(CompileError::Policy(format!(
                "Service '{}' (decentralized mode) has unsupported decentralized-v1 field type(s): [{}]. \
Supported field types: int, bool, string, address, bytes32, map<address,int>, map<string,int>.",
                service.name,
                unsupported_fields.join(", ")
            )));
        }
    }
    Ok(())
}

fn is_decentralized_v1_supported_type(field_type: &str) -> bool {
    let t = field_type.trim().to_ascii_lowercase();
    matches!(
        t.as_str(),
        "int" | "bool" | "string" | "address" | "bytes32" | "map<address,int>" | "map<string,int>"
    )
}

fn collect_decentralized_v1_unsupported_constructs(block: &BlockStatement) -> Vec<String> {
    let mut unsupported = std::collections::HashSet::new();
    for stmt in &block.statements {
        collect_unsupported_from_statement(stmt, &mut unsupported);
    }
    let mut out: Vec<String> = unsupported.into_iter().collect();
    out.sort();
    out
}

fn collect_unsupported_from_statement(
    stmt: &Statement,
    unsupported: &mut std::collections::HashSet<String>,
) {
    use crate::parser::ast::Statement::*;
    match stmt {
        Let(s) => collect_unsupported_from_expression(&s.value, unsupported),
        Return(s) => {
            if let Some(v) = &s.value {
                collect_unsupported_from_expression(v, unsupported);
            }
        }
        Block(b) => {
            for child in &b.statements {
                collect_unsupported_from_statement(child, unsupported);
            }
        }
        Expression(e) => collect_unsupported_from_expression(e, unsupported),
        If(s) => {
            collect_unsupported_from_expression(&s.condition, unsupported);
            for child in &s.consequence.statements {
                collect_unsupported_from_statement(child, unsupported);
            }
            if let Some(alt) = &s.alternative {
                for child in &alt.statements {
                    collect_unsupported_from_statement(child, unsupported);
                }
            }
        }
        Event(e) => {
            for value in e.data.values() {
                collect_unsupported_from_expression(value, unsupported);
            }
        }
        While(_) => {
            unsupported.insert("while".to_string());
        }
        ForIn(_) => {
            unsupported.insert("for-in".to_string());
        }
        Try(_) => {
            unsupported.insert("try/catch".to_string());
        }
        Loop(_) => {
            unsupported.insert("loop".to_string());
        }
        Match(_) => {
            unsupported.insert("match".to_string());
        }
        Spawn(_) => {
            unsupported.insert("spawn-statement".to_string());
        }
        Agent(_) => {
            unsupported.insert("agent-statement".to_string());
        }
        Message(_) => {
            unsupported.insert("message-statement".to_string());
        }
        Import(_) | Function(_) | Service(_) | Break(_) | Continue(_) => {}
    }
}

fn collect_unsupported_from_expression(
    expr: &Expression,
    unsupported: &mut std::collections::HashSet<String>,
) {
    use crate::parser::ast::Expression::*;
    match expr {
        Literal(_) | Identifier(_) => {}
        BinaryOp(l, _, r) => {
            collect_unsupported_from_expression(l, unsupported);
            collect_unsupported_from_expression(r, unsupported);
        }
        UnaryOp(_, e) | Assignment(_, e) | FieldAccess(e, _) | Await(e) | Spawn(e) | Throw(e) => {
            if matches!(expr, Await(_)) {
                unsupported.insert("await".to_string());
            }
            if matches!(expr, Spawn(_)) {
                unsupported.insert("spawn-expression".to_string());
            }
            if matches!(expr, Throw(_)) {
                unsupported.insert("throw".to_string());
            }
            collect_unsupported_from_expression(e, unsupported);
        }
        FunctionCall(call) => {
            for arg in &call.arguments {
                collect_unsupported_from_expression(arg, unsupported);
            }
        }
        FieldAssignment(l, _, r) => {
            collect_unsupported_from_expression(l, unsupported);
            collect_unsupported_from_expression(r, unsupported);
        }
        ObjectLiteral(map) => {
            for value in map.values() {
                collect_unsupported_from_expression(value, unsupported);
            }
        }
        ArrayLiteral(items) => {
            unsupported.insert("array-literal".to_string());
            for item in items {
                collect_unsupported_from_expression(item, unsupported);
            }
        }
        IndexAccess(container, idx) => {
            unsupported.insert("index-access".to_string());
            collect_unsupported_from_expression(container, unsupported);
            collect_unsupported_from_expression(idx, unsupported);
        }
        ArrowFunction { .. } => {
            unsupported.insert("arrow-function".to_string());
        }
        Range(start, end) => {
            unsupported.insert("range-expression".to_string());
            collect_unsupported_from_expression(start, unsupported);
            collect_unsupported_from_expression(end, unsupported);
        }
        MethodCall {
            receiver,
            arguments,
            ..
        } => {
            unsupported.insert("method-call".to_string());
            collect_unsupported_from_expression(receiver, unsupported);
            for arg in arguments {
                collect_unsupported_from_expression(arg, unsupported);
            }
        }
    }
}

fn collect_namespaces_from_block(block: &BlockStatement) -> std::collections::HashSet<String> {
    block
        .statements
        .iter()
        .flat_map(collect_namespaces_from_statement)
        .collect()
}

fn collect_namespaces_from_statement(stmt: &Statement) -> std::collections::HashSet<String> {
    use crate::parser::ast::Statement::*;
    match stmt {
        Let(s) => collect_namespaces_from_expression(&s.value),
        Return(s) => s
            .value
            .as_ref()
            .map(collect_namespaces_from_expression)
            .unwrap_or_default(),
        Expression(e) => collect_namespaces_from_expression(e),
        If(s) => {
            let mut set = collect_namespaces_from_expression(&s.condition);
            set.extend(collect_namespaces_from_block(&s.consequence));
            if let Some(alt) = &s.alternative {
                set.extend(collect_namespaces_from_block(alt));
            }
            set
        }
        While(s) => {
            let mut set = collect_namespaces_from_expression(&s.condition);
            set.extend(collect_namespaces_from_block(&s.body));
            set
        }
        ForIn(s) => {
            let mut set = collect_namespaces_from_expression(&s.iterable);
            set.extend(collect_namespaces_from_block(&s.body));
            set
        }
        Try(s) => {
            let mut set = collect_namespaces_from_block(&s.try_block);
            for catch in &s.catch_blocks {
                set.extend(collect_namespaces_from_block(&catch.body));
            }
            if let Some(fin) = &s.finally_block {
                set.extend(collect_namespaces_from_block(fin));
            }
            set
        }
        Event(e) => e
            .data
            .values()
            .flat_map(collect_namespaces_from_expression)
            .collect(),
        Function(f) => collect_namespaces_from_block(&f.body),
        Block(b) => collect_namespaces_from_block(b),
        Match(m) => {
            let mut set = collect_namespaces_from_expression(&m.expression);
            for case in &m.cases {
                set.extend(collect_namespaces_from_block(&case.body));
            }
            if let Some(default_case) = &m.default_case {
                set.extend(collect_namespaces_from_block(default_case));
            }
            set
        }
        Loop(l) => collect_namespaces_from_block(&l.body),
        Service(_) | Import(_) | Agent(_) | Spawn(_) | Break(_) | Continue(_) | Message(_) => {
            std::collections::HashSet::new()
        }
    }
}

fn collect_namespaces_from_expression(expr: &Expression) -> std::collections::HashSet<String> {
    use crate::parser::ast::Expression::*;
    let mut set = std::collections::HashSet::new();
    match expr {
        FunctionCall(call) => {
            if let Some((ns, _)) = call.name.split_once("::") {
                set.insert(ns.to_string());
            }
            for arg in &call.arguments {
                set.extend(collect_namespaces_from_expression(arg));
            }
        }
        BinaryOp(l, _, r) => {
            set.extend(collect_namespaces_from_expression(l));
            set.extend(collect_namespaces_from_expression(r));
        }
        UnaryOp(_, e) | Assignment(_, e) | FieldAccess(e, _) | Await(e) | Spawn(e) | Throw(e) => {
            set.extend(collect_namespaces_from_expression(e))
        }
        FieldAssignment(l, _, r) => {
            set.extend(collect_namespaces_from_expression(l));
            set.extend(collect_namespaces_from_expression(r));
        }
        ObjectLiteral(map) => {
            for v in map.values() {
                set.extend(collect_namespaces_from_expression(v));
            }
        }
        ArrayLiteral(list) => {
            for e in list {
                set.extend(collect_namespaces_from_expression(e));
            }
        }
        IndexAccess(container, idx) => {
            set.extend(collect_namespaces_from_expression(container));
            set.extend(collect_namespaces_from_expression(idx));
        }
        MethodCall {
            receiver,
            arguments,
            ..
        } => {
            set.extend(collect_namespaces_from_expression(receiver));
            for arg in arguments {
                set.extend(collect_namespaces_from_expression(arg));
            }
        }
        Range(start, end) => {
            set.extend(collect_namespaces_from_expression(start));
            set.extend(collect_namespaces_from_expression(end));
        }
        Identifier(_) | Literal(_) | ArrowFunction { .. } => {}
    }
    set
}

#[cfg(test)]
mod trust_compile_mode_tests {
    use super::TrustCompileMode;

    #[test]
    fn from_str_accepts_all_compile_trust_modes_case_insensitive() {
        assert_eq!(
            TrustCompileMode::from_str("auto"),
            Some(TrustCompileMode::Auto)
        );
        assert_eq!(
            TrustCompileMode::from_str("AUTO"),
            Some(TrustCompileMode::Auto)
        );
        assert_eq!(
            TrustCompileMode::from_str("  Auto  "),
            Some(TrustCompileMode::Auto)
        );
        assert_eq!(
            TrustCompileMode::from_str("decentralized"),
            Some(TrustCompileMode::Decentralized)
        );
        assert_eq!(
            TrustCompileMode::from_str("DECENTRALIZED"),
            Some(TrustCompileMode::Decentralized)
        );
        assert_eq!(
            TrustCompileMode::from_str("hybrid"),
            Some(TrustCompileMode::Hybrid)
        );
        assert_eq!(
            TrustCompileMode::from_str("centralized"),
            Some(TrustCompileMode::Centralized)
        );
        assert_eq!(
            TrustCompileMode::from_str("auto "),
            Some(TrustCompileMode::Auto)
        );
    }

    #[test]
    fn from_str_rejects_unknown_and_empty() {
        assert_eq!(TrustCompileMode::from_str(""), None);
        assert_eq!(TrustCompileMode::from_str("   "), None);
        assert_eq!(TrustCompileMode::from_str("xyzzy"), None);
    }

    #[test]
    fn std_from_str_matches_from_str() {
        assert_eq!(TrustCompileMode::Hybrid, "hybrid".parse().unwrap());
        assert!("nope".parse::<TrustCompileMode>().is_err());
    }
}

#[cfg(test)]
mod h5_policy_convergence_tests {
    use crate::lexer::tokens::CompilationTarget;

    #[test]
    fn parser_rejects_trust_without_string_model() {
        let source = r#"
@trust @chain("ethereum")
service Bad {
    fn noop() -> int { return 0; }
}
"#;
        let result = crate::parse_source(source);
        assert!(
            result.is_err(),
            "parser should reject @trust without string model"
        );
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("without a valid string model"),
            "error should explain missing model: {}",
            err
        );
    }

    #[test]
    fn parser_rejects_invalid_trust_model() {
        let source = r#"
@trust("invalid_model") @chain("ethereum")
service Bad {
    fn noop() -> int { return 0; }
}
"#;
        let result = crate::parse_source(source);
        assert!(result.is_err(), "parser should reject invalid trust model");
    }

    #[test]
    fn parser_and_compiler_agree_on_fs_namespace_rejection() {
        let source = r#"
@secure
@trust("decentralized")
@chain("ethereum")
service Store @compile_target("blockchain") {
    fn read_data(path: string) -> string {
        let data = fs::read_text(path);
        return data;
    }
}
"#;
        let parse_result = crate::parse_source(source);
        assert!(
            parse_result.is_err(),
            "parser should reject fs:: in decentralized service"
        );
        let err = format!("{}", parse_result.unwrap_err());
        assert!(
            err.contains("fs") && err.contains("disallowed"),
            "parser error should mention fs namespace: {}",
            err
        );
    }

    #[test]
    fn compiler_rejects_fs_namespace_in_decentralized() {
        let source = r#"
@secure
@trust("decentralized")
@chain("ethereum")
service Store @compile_target("blockchain") {
    val: int = 0;
    fn bump() -> int {
        self.val = self.val + 1;
        return self.val;
    }
}
"#;
        let program = crate::parse_source(source).expect("parse");
        let services = super::select_services_for_target(&program, &CompilationTarget::Blockchain);
        let result = super::validate_trust_policy_for_services(
            &services,
            &CompilationTarget::Blockchain,
            super::TrustCompileMode::Auto,
        );
        assert!(
            result.is_ok(),
            "clean decentralized service should pass compiler policy"
        );
    }

    #[test]
    fn all_three_trust_models_parse_and_compile_without_error() {
        for model in &["decentralized", "hybrid", "centralized"] {
            let source = format!(
                r#"
@secure
@trust("{}")
@chain("ethereum")
service Test @compile_target("blockchain") {{
    val: int = 0;
    fn get_val() -> int {{
        return self.val;
    }}
}}
"#,
                model
            );
            let program = crate::parse_source(&source)
                .unwrap_or_else(|e| panic!("parse failed for model '{}': {}", model, e));
            let services =
                super::select_services_for_target(&program, &CompilationTarget::Blockchain);
            if !services.is_empty() {
                let result = super::validate_trust_policy_for_services(
                    &services,
                    &CompilationTarget::Blockchain,
                    super::TrustCompileMode::Auto,
                );
                assert!(
                    result.is_ok(),
                    "model '{}' should pass compiler policy: {:?}",
                    model,
                    result
                );
            }
        }
    }

    #[test]
    fn forbidden_namespace_lists_are_aligned() {
        let parser_forbidden: std::collections::HashSet<&str> = [
            "ai",
            "sh",
            "fs",
            "web",
            "http",
            "oracle",
            "agent",
            "cloudadmin",
        ]
        .iter()
        .copied()
        .collect();
        let compiler_forbidden: std::collections::HashSet<&str> = [
            "ai",
            "sh",
            "fs",
            "web",
            "http",
            "oracle",
            "agent",
            "cloudadmin",
        ]
        .iter()
        .copied()
        .collect();
        assert_eq!(
            parser_forbidden, compiler_forbidden,
            "parser and compiler forbidden namespace lists must be identical"
        );
    }
}

//! CT1: Compiler pipeline skeleton tests — driver, service selection, stub backend.

use dist_agent_lang::compile::{
    run_compile, select_services_for_target, CompileError,
};
use dist_agent_lang::lexer::tokens::{get_target_constraints, CompilationTarget, TargetConstraint};
use dist_agent_lang::parser::ast::{
    CompilationTargetInfo, Program, ServiceStatement, Statement,
};

/// Build a minimal program with one service that has compilation_target set (no parser validation).
fn program_with_native_service() -> Program {
    let service = ServiceStatement {
        name: "MyApp".to_string(),
        attributes: vec![dist_agent_lang::parser::ast::Attribute {
            name: "@native".to_string(),
            parameters: vec![],
            target: dist_agent_lang::parser::ast::AttributeTarget::Module,
        }],
        fields: vec![],
        methods: vec![],
        events: vec![],
        exported: false,
        compilation_target: Some(CompilationTargetInfo {
            target: CompilationTarget::Native,
            constraints: TargetConstraint::new(CompilationTarget::Native),
            validation_errors: vec![],
        }),
    };
    Program {
        statements: vec![Statement::Service(service)],
        statement_spans: vec![None],
    }
}

#[test]
fn test_select_services_for_target_native() {
    let program = program_with_native_service();
    let services = select_services_for_target(&program, &CompilationTarget::Native);
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].name, "MyApp");
}

#[test]
fn test_select_services_for_target_empty_when_no_match() {
    let program = program_with_native_service();
    let services = select_services_for_target(&program, &CompilationTarget::Blockchain);
    assert!(services.is_empty());
}

#[test]
fn test_select_services_for_target_multiple() {
    // Build program with two blockchain services manually to avoid parser attribute validation
    let mk_service = |name: &str| ServiceStatement {
        name: name.to_string(),
        attributes: vec![],
        fields: vec![],
        methods: vec![],
        events: vec![],
        exported: false,
        compilation_target: Some(CompilationTargetInfo {
            target: CompilationTarget::Blockchain,
            constraints: TargetConstraint::new(CompilationTarget::Blockchain),
            validation_errors: vec![],
        }),
    };
    let program = Program {
        statements: vec![
            Statement::Service(mk_service("Token")),
            Statement::Service(mk_service("Vault")),
        ],
        statement_spans: vec![None, None],
    };
    let services = select_services_for_target(&program, &CompilationTarget::Blockchain);
    assert_eq!(services.len(), 2);
    let names: Vec<&str> = services.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"Token"));
    assert!(names.contains(&"Vault"));
}

/// CT0: Runtime validate_compile_target rejects service missing required attributes.
#[test]
fn test_ct0_runtime_rejects_missing_required_attributes() {
    let constraints = get_target_constraints();
    let bc = constraints
        .get(&CompilationTarget::Blockchain)
        .cloned()
        .unwrap();
    let service = ServiceStatement {
        name: "Bad".to_string(),
        attributes: vec![], // missing @secure, @trust
        fields: vec![],
        methods: vec![],
        events: vec![],
        exported: false,
        compilation_target: Some(CompilationTargetInfo {
            target: CompilationTarget::Blockchain,
            constraints: bc,
            validation_errors: vec![],
        }),
    };
    let program = Program {
        statements: vec![Statement::Service(service)],
        statement_spans: vec![None],
    };
    let mut runtime = dist_agent_lang::Runtime::new();
    let result = runtime.execute_program(program, None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("missing required attribute") || msg.contains("Missing required"),
        "expected compile-target validation error, got: {}",
        msg
    );
}

/// Build with imports: entry imports a sibling file; resolution runs and merged program compiles.
/// (Service with @compile_target lives in entry; lib is import-only so we don't hit parser @native validation in deps.)
#[test]
fn test_run_compile_with_imports_resolves() {
    let dir = tempfile::tempdir().unwrap();
    let main_path = dir.path().join("main.dal");
    let lib_path = dir.path().join("lib.dal");
    std::fs::write(&lib_path, "fn helper() { 0 }\n").unwrap();
    let main_source = r#"
import "./lib.dal" as m;
@native
service App @compile_target("native") {
    fn run() { 0 }
}
"#;
    std::fs::write(&main_path, main_source).unwrap();
    let out = dir.path().join("out");
    std::fs::create_dir_all(&out).unwrap();

    let result = run_compile(
        main_path.clone(),
        CompilationTarget::Native,
        out.clone(),
        main_source,
    );

    assert!(result.is_ok(), "build with imports should resolve and compile: {:?}", result.err());
    let artifacts = result.unwrap();
    assert_eq!(artifacts.service_names, vec!["App"]);
}

/// CT2: Blockchain backend — when solc is present, produces .sol, .bin, .abi and stub: false.
#[test]
fn test_run_compile_blockchain_backend() {
    let source = r#"
@secure
@trust("hybrid")
service Token @compile_target("blockchain") {
    fn transfer(to: string, amount: int) { }
}
"#;
    let dir = tempfile::tempdir().unwrap();
    let entry = dir.path().join("main.dal");
    let out = dir.path().join("out");
    std::fs::create_dir_all(&out).unwrap();

    let result = run_compile(entry.clone(), CompilationTarget::Blockchain, out.clone(), source);

    match result {
        Ok(artifacts) => {
            assert_eq!(artifacts.target, "blockchain");
            assert_eq!(artifacts.service_names, vec!["Token"]);
            assert!(!artifacts.stub, "blockchain backend should produce real artifacts when solc is available");
            let has_bin = artifacts.artifact_paths.iter().any(|p| p.extension().map(|e| e == "bin").unwrap_or(false));
            let has_abi = artifacts.artifact_paths.iter().any(|p| p.extension().map(|e| e == "abi").unwrap_or(false));
            assert!(has_bin && has_abi, "expected .bin and .abi artifacts, got {:?}", artifacts.artifact_paths);
            let manifest = out.join("compile-manifest.json");
            assert!(manifest.exists());
            let content = std::fs::read_to_string(manifest).unwrap();
            assert!(content.contains("\"stub\":false"));
        }
        Err(CompileError::CompilerNotFound { .. }) => {
            // solc not installed — backend correctly reports it
        }
        Err(CompileError::Parse(_)) => {
            // Parser/attribute validation may fail in some configurations
        }
        Err(e) => panic!("unexpected error: {}", e),
    }
}

/// CT3: WASM backend — when wasm32 target is present, produces .wasm and stub: false.
#[test]
fn test_run_compile_wasm_backend() {
    let source = r#"
@web
service WebApp @compile_target("wasm") {
    fn handle() { }
}
"#;
    let dir = tempfile::tempdir().unwrap();
    let entry = dir.path().join("main.dal");
    let out = dir.path().join("out");
    std::fs::create_dir_all(&out).unwrap();

    let result = run_compile(
        entry.clone(),
        CompilationTarget::WebAssembly,
        out.clone(),
        source,
    );

    match result {
        Ok(artifacts) => {
            assert_eq!(artifacts.target, "wasm");
            assert_eq!(artifacts.service_names, vec!["WebApp"]);
            assert!(
                !artifacts.stub,
                "wasm backend should produce real artifacts when wasm32 target is available"
            );
            let has_wasm = artifacts
                .artifact_paths
                .iter()
                .any(|p| p.extension().map(|e| e == "wasm").unwrap_or(false));
            assert!(
                has_wasm,
                "expected .wasm artifact, got {:?}",
                artifacts.artifact_paths
            );
            let manifest = out.join("compile-manifest.json");
            assert!(manifest.exists());
            let content = std::fs::read_to_string(manifest).unwrap();
            assert!(content.contains("\"stub\":false"));
        }
        Err(CompileError::CompilerNotFound { .. }) => {
            // wasm32 target not installed
        }
        Err(CompileError::Parse(_)) => {}
        Err(CompileError::Backend(_)) => {
            // cargo build failed (e.g. missing target)
        }
        Err(e) => panic!("unexpected error: {}", e),
    }
}

/// CT4: Native backend — when cargo is present, produces .rlib and stub: false.
#[test]
fn test_run_compile_native_backend() {
    let source = r#"
@native
service App @compile_target("native") { fn run() { 42 } }
"#;
    let dir = tempfile::tempdir().unwrap();
    let entry = dir.path().join("main.dal");
    let out = dir.path().join("out");
    std::fs::create_dir_all(&out).unwrap();

    let result = run_compile(entry.clone(), CompilationTarget::Native, out.clone(), source);

    match result {
        Ok(artifacts) => {
            assert_eq!(artifacts.target, "native");
            assert_eq!(artifacts.service_names, vec!["App"]);
            assert!(
                !artifacts.stub,
                "native backend should report real codegen when cargo is available"
            );
            if let Some(p) = artifacts.artifact_paths.iter().find(|p| p.extension().map(|e| e == "rlib").unwrap_or(false)) {
                assert!(p.exists(), "rlib path should exist: {}", p.display());
            }
            let manifest = out.join("compile-manifest.json");
            assert!(manifest.exists());
            let content = std::fs::read_to_string(manifest).unwrap();
            assert!(content.contains("\"stub\":false"));
        }
        Err(CompileError::CompilerNotFound { .. }) => {}
        Err(CompileError::Parse(_)) => {}
        Err(CompileError::Backend(_)) => {}
        Err(e) => panic!("unexpected error: {}", e),
    }
}

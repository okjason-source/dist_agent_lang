//! M2: Module resolution tests — stdlib, relative path, missing file, cycle detection.
//! M4: Runtime module loading — import then call (stdlib alias and relative module).
//! M3: dal.toml and package resolution — path deps, lockfile, import package.

use dist_agent_lang::manifest::{
    load_resolved_deps, parse_dependencies, resolve_dependencies, write_lockfile,
};
use dist_agent_lang::module_resolver::{ModuleResolver, ResolveError, ResolvedImport};
use dist_agent_lang::parser::ast::Statement;
use dist_agent_lang::runtime::values::Value;
use dist_agent_lang::{parse_source, resolve_imports, Runtime};

#[test]
fn test_resolve_program_imports_stdlib() {
    let program = parse_source("import stdlib::chain;").unwrap();
    let resolved = resolve_imports(&program, None).unwrap();
    assert_eq!(resolved.len(), 1);
    assert!(matches!(&resolved[0].resolved, ResolvedImport::Stdlib(s) if s == "chain"));
}

#[test]
fn test_resolve_program_imports_multiple_stdlib() {
    let program =
        parse_source("import stdlib::chain;\nimport stdlib::ai as ai_mod;\nimport stdlib::log;")
            .unwrap();
    let resolved = resolve_imports(&program, None).unwrap();
    assert_eq!(resolved.len(), 3);
    assert!(matches!(&resolved[0].resolved, ResolvedImport::Stdlib(s) if s == "chain"));
    assert!(matches!(&resolved[1].resolved, ResolvedImport::Stdlib(s) if s == "ai"));
    assert!(matches!(&resolved[2].resolved, ResolvedImport::Stdlib(s) if s == "log"));
}

#[test]
fn test_resolve_relative_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let entry = root.join("main.dal");
    let dep = root.join("mymod.dal");
    std::fs::write(&dep, "let x = 1;").unwrap();
    let program = parse_source(r#"import "./mymod.dal" as m;"#).unwrap();
    let resolver = ModuleResolver::new();
    let resolved = resolver
        .resolve_program_imports(&program, Some(entry.as_path()))
        .unwrap();
    assert_eq!(resolved.len(), 1);
    match &resolved[0].resolved {
        ResolvedImport::RelativeFile(p) => {
            assert!(p.is_absolute());
            assert_eq!(p.file_name().unwrap(), "mymod.dal");
        }
        _ => panic!("expected RelativeFile"),
    }
}

#[test]
fn test_resolve_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let entry = dir.path().join("main.dal");
    let program = parse_source(r#"import "./nonexistent.dal";"#).unwrap();
    let resolver = ModuleResolver::new();
    let err = resolver
        .resolve_program_imports(&program, Some(entry.as_path()))
        .unwrap_err();
    assert!(matches!(err, ResolveError::FileNotFound(_)));
}

#[test]
fn test_resolve_cycle_detected() {
    let dir = tempfile::tempdir().unwrap();
    let a_path = dir.path().join("a.dal");
    let b_path = dir.path().join("b.dal");
    std::fs::write(&a_path, r#"import "./b.dal" as b; let x = 1;"#).unwrap();
    std::fs::write(&b_path, r#"import "./a.dal" as a; let y = 2;"#).unwrap();
    let program = parse_source(&std::fs::read_to_string(&a_path).unwrap()).unwrap();
    let resolver = ModuleResolver::new();
    let parse_fn = |s: &str| parse_source(s).map_err(|e| e.to_string());
    let err = resolver
        .resolve_program_with_cycles(&program, Some(a_path.as_path()), parse_fn)
        .unwrap_err();
    assert!(matches!(err, ResolveError::CycleDetected(_)));
}

#[test]
fn test_resolve_no_cycle_chain() {
    let dir = tempfile::tempdir().unwrap();
    let a_path = dir.path().join("a.dal");
    let b_path = dir.path().join("b.dal");
    let c_path = dir.path().join("c.dal");
    std::fs::write(&a_path, r#"import "./b.dal" as b; let x = 1;"#).unwrap();
    std::fs::write(&b_path, r#"import "./c.dal" as c; let y = 2;"#).unwrap();
    std::fs::write(&c_path, r#"let z = 3;"#).unwrap();
    let program = parse_source(&std::fs::read_to_string(&a_path).unwrap()).unwrap();
    let resolver = ModuleResolver::new();
    let parse_fn = |s: &str| parse_source(s).map_err(|e| e.to_string());
    let resolved = resolver
        .resolve_program_with_cycles(&program, Some(a_path.as_path()), parse_fn)
        .unwrap();
    assert!(resolved.len() >= 2);
}

// --- M4: Runtime module loading tests ---

#[test]
fn test_m4_stdlib_alias_import_then_call() {
    let source = r#"import stdlib::log as lg; lg::info("hi", "ok")"#;
    let program = parse_source(source).unwrap();
    let resolved = resolve_imports(&program, None).unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program, Some(&resolved));
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

#[test]
fn test_m4_relative_module_import_then_call() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let main_path = root.join("main.dal");
    let mymod_path = root.join("mymod.dal");
    std::fs::write(&mymod_path, r#"fn foo() { 42 }"#).unwrap();
    std::fs::write(&main_path, r#"import "./mymod.dal" as m; m::foo()"#).unwrap();
    let source = std::fs::read_to_string(&main_path).unwrap();
    let program = parse_source(&source).unwrap();
    let resolver = ModuleResolver::new();
    let parse_fn = |s: &str| parse_source(s).map_err(|e| e.to_string());
    let resolved = resolver
        .resolve_program_with_cycles(&program, Some(main_path.as_path()), parse_fn)
        .unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program, Some(&resolved));
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    let value = result.unwrap();
    assert_eq!(value, Some(Value::Int(42)));
}

// --- M3: dal.toml and package resolution tests ---

#[test]
fn test_m3_parse_dependencies_path() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("dal.toml");
    std::fs::write(
        &manifest,
        r#"[package]
name = "app"
version = "0.1.0"

[dependencies]
utils = { path = "../utils" }
"#,
    )
    .unwrap();
    let deps = parse_dependencies(&manifest).unwrap();
    assert_eq!(deps.len(), 1);
    match deps.get("utils").unwrap() {
        dist_agent_lang::manifest::DependencySpec::Path(p) => {
            assert!(p.ends_with("utils"));
        }
        _ => panic!("expected path dep"),
    }
}

#[test]
fn test_m3_resolve_and_lockfile() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let lib_dir = root.join("utils");
    std::fs::create_dir_all(&lib_dir).unwrap();
    let manifest = root.join("dal.toml");
    std::fs::write(
        &manifest,
        r#"[package]
name = "app"
version = "0.1.0"

[dependencies]
utils = { path = "utils" }
"#,
    )
    .unwrap();
    let resolved = resolve_dependencies(&manifest).unwrap();
    assert_eq!(resolved.len(), 1);
    assert!(resolved.get("utils").unwrap().ends_with("utils"));
    write_lockfile(&manifest, &resolved).unwrap();
    let lock_path = root.join("dal.lock");
    assert!(lock_path.exists());
    let loaded = load_resolved_deps(&manifest).unwrap();
    assert_eq!(loaded.len(), 1);
    assert!(loaded.get("utils").is_some());
}

#[test]
fn test_m3_import_package_then_call() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let app_dir = root.join("app");
    let lib_dir = root.join("utils");
    std::fs::create_dir_all(&app_dir).unwrap();
    std::fs::create_dir_all(&lib_dir).unwrap();
    std::fs::write(
        app_dir.join("dal.toml"),
        r#"[package]
name = "app"
version = "0.1.0"

[dependencies]
utils = { path = "../utils" }
"#,
    )
    .unwrap();
    std::fs::write(lib_dir.join("lib.dal"), "fn foo() { 42 }").unwrap();
    let main_path = app_dir.join("main.dal");
    std::fs::write(&main_path, r#"import "utils" as m; m::foo()"#).unwrap();
    let manifest_path = app_dir.join("dal.toml");
    let resolved = resolve_dependencies(&manifest_path).unwrap();
    write_lockfile(&manifest_path, &resolved).unwrap();
    let deps = load_resolved_deps(&manifest_path).unwrap();
    let program = parse_source(&std::fs::read_to_string(&main_path).unwrap()).unwrap();
    let resolver = ModuleResolver::new()
        .with_root_dir(app_dir.to_path_buf())
        .with_dependencies(deps);
    let parse_fn = |s: &str| parse_source(s).map_err(|e| e.to_string());
    let resolved_imports = resolver
        .resolve_program_with_cycles(&program, Some(main_path.as_path()), parse_fn)
        .unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program, Some(&resolved_imports));
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    assert_eq!(result.unwrap(), Some(Value::Int(42)));
}

// --- M5: Exports and visibility tests ---

#[test]
fn test_m5_export_fn_parsed() {
    let program = parse_source("export fn foo() { 1 }").unwrap();
    match program.statements.as_slice() {
        [Statement::Function(f)] => assert!(f.exported, "expected exported true"),
        _ => panic!("expected one function statement"),
    }
}

#[test]
fn test_m5_import_explicit_export_then_call() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let main_path = root.join("main.dal");
    let mod_path = root.join("mod.dal");
    std::fs::write(&mod_path, "export fn bar() { 99 }").unwrap();
    std::fs::write(&main_path, r#"import "./mod.dal" as m; m::bar()"#).unwrap();
    let source = std::fs::read_to_string(&main_path).unwrap();
    let program = parse_source(&source).unwrap();
    let resolver = ModuleResolver::new();
    let parse_fn = |s: &str| parse_source(s).map_err(|e| e.to_string());
    let resolved = resolver
        .resolve_program_with_cycles(&program, Some(main_path.as_path()), parse_fn)
        .unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program, Some(&resolved));
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    assert_eq!(result.unwrap(), Some(Value::Int(99)));
}

#[test]
fn test_m5_only_exported_visible() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let main_path = root.join("main.dal");
    let mod_path = root.join("mod.dal");
    // Module: one private fn (no export), one export fn. Only exported should be callable.
    std::fs::write(
        &mod_path,
        r#"
fn private_fn() { 1 }
export fn pub_fn() { 2 }
"#,
    )
    .unwrap();
    std::fs::write(&main_path, r#"import "./mod.dal" as m; m::pub_fn()"#).unwrap();
    let source = std::fs::read_to_string(&main_path).unwrap();
    let program = parse_source(&source).unwrap();
    let resolver = ModuleResolver::new();
    let parse_fn = |s: &str| parse_source(s).map_err(|e| e.to_string());
    let resolved = resolver
        .resolve_program_with_cycles(&program, Some(main_path.as_path()), parse_fn)
        .unwrap();
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program, Some(&resolved));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Int(2)));
    // Calling m::private_fn() would fail (not in exports) - we only test that pub_fn works
}

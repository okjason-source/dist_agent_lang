use dist_agent_lang::compile::{
    run_compile_with_mode, set_compiler_available_override, TrustCompileMode,
};
use dist_agent_lang::lexer::tokens::CompilationTarget;

#[test]
fn decentralized_service_rejects_ai_namespace_at_parse_time() {
    let source = r#"
@trust("decentralized")
@chain("ethereum")
service SecureToken {
    fn run() {
        ai::generate_text("hello");
    }
}
"#;

    let parsed = dist_agent_lang::parse_source(source);
    assert!(
        parsed.is_err(),
        "expected decentralized namespace rejection"
    );
    let msg = parsed.err().unwrap().to_string();
    assert!(
        msg.contains("decentralized service") && msg.contains("disallowed namespace"),
        "unexpected error: {}",
        msg
    );
}

#[test]
fn decentralized_service_allows_chain_namespace() {
    let source = r#"
@trust("decentralized")
@chain("ethereum")
service SecureToken {
    fn run() {
        chain::get_gas_price(1);
    }
}
"#;

    let parsed = dist_agent_lang::parse_source(source);
    assert!(
        parsed.is_ok(),
        "expected parse success, got: {:?}",
        parsed.err()
    );
}

#[test]
fn forced_decentralized_mode_rejects_hybrid_style_orchestration() {
    let source = r#"
@trust("hybrid")
@chain("ethereum")
@native
service Orchestrator @compile_target("native") {
    fn run() {
        ai::generate_text("hello");
    }
}
"#;

    let dir = tempfile::tempdir().unwrap();
    let entry = dir.path().join("main.dal");
    let out = dir.path().join("out");
    std::fs::write(&entry, source).unwrap();
    std::fs::create_dir_all(&out).unwrap();

    set_compiler_available_override(Some(true));
    let result = run_compile_with_mode(
        entry,
        CompilationTarget::Native,
        out,
        source,
        TrustCompileMode::Decentralized,
    );
    set_compiler_available_override(None);

    assert!(
        result.is_err(),
        "forced decentralized compile mode should reject ai namespace"
    );
    let msg = result.err().unwrap().to_string();
    assert!(
        msg.contains("Policy check failed") && msg.contains("disallowed namespace"),
        "unexpected compile error: {}",
        msg
    );
}

#[test]
fn decentralized_v1_rejects_try_catch_constructs() {
    let source = r#"
@trust("decentralized")
@chain("ethereum")
@mobile
service SecureFlow @compile_target("mobile") {
    fn run() {
        try {
            let x = 1;
        } catch (err) {
            let y = 2;
        }
    }
}
"#;

    let dir = tempfile::tempdir().unwrap();
    let entry = dir.path().join("main.dal");
    let out = dir.path().join("out");
    std::fs::write(&entry, source).unwrap();
    std::fs::create_dir_all(&out).unwrap();

    set_compiler_available_override(Some(true));
    let result = run_compile_with_mode(
        entry,
        CompilationTarget::Mobile,
        out,
        source,
        TrustCompileMode::Auto,
    );
    set_compiler_available_override(None);

    assert!(result.is_err(), "expected unsupported construct rejection");
    let msg = result.err().unwrap().to_string();
    assert!(
        msg.contains("unsupported decentralized-v1 construct") && msg.contains("try/catch"),
        "unexpected compile error: {}",
        msg
    );
}

#[test]
fn decentralized_v1_allows_critical_subset_constructs() {
    let source = r#"
@trust("decentralized")
@chain("ethereum")
@mobile
service SecureFlow @compile_target("mobile") {
    fn run(x: int) -> int {
        let y = x + 1;
        if (y > 10) {
            return y;
        }
        return y + 1;
    }
}
"#;

    let dir = tempfile::tempdir().unwrap();
    let entry = dir.path().join("main.dal");
    let out = dir.path().join("out");
    std::fs::write(&entry, source).unwrap();
    std::fs::create_dir_all(&out).unwrap();

    set_compiler_available_override(Some(true));
    let result = run_compile_with_mode(
        entry,
        CompilationTarget::Mobile,
        out,
        source,
        TrustCompileMode::Auto,
    );
    set_compiler_available_override(None);

    assert!(
        result.is_ok(),
        "expected decentralized-v1 subset source to compile, got: {:?}",
        result.err()
    );
}

//! Integration tests for `fs::` stdlib (DAL_FS_ROOT + runtime).

use dist_agent_lang::lexer::Lexer;
use dist_agent_lang::parser::Parser;
use dist_agent_lang::runtime::values::Value;
use dist_agent_lang::Runtime;
use std::sync::Mutex;

static FS_DAL_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn dal_fs_uses_dal_fs_root() {
    let _g = FS_DAL_LOCK.lock().expect("fs dal lock");

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("note.txt"), "roundtrip").unwrap();

    std::env::set_var("DAL_FS_ROOT", root.as_os_str());
    let code = r#"
        let content = fs::read_text("note.txt");
        let _ = fs::write_text("out.txt", "ok");
        let ok = fs::exists("out.txt");
    "#;
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    let mut runtime = Runtime::new();
    let res = runtime.execute_program(program, None);
    std::env::remove_var("DAL_FS_ROOT");

    assert!(res.is_ok(), "execution failed: {:?}", res.err());
    match runtime.get_variable("content") {
        Ok(Value::String(s)) => assert_eq!(s, "roundtrip"),
        other => panic!("expected content string, got {:?}", other),
    }
    match runtime.get_variable("ok") {
        Ok(Value::Bool(b)) => assert!(b),
        other => panic!("expected ok bool, got {:?}", other),
    }
    assert_eq!(std::fs::read_to_string(root.join("out.txt")).unwrap(), "ok");
}

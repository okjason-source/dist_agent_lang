//! Integration tests for mold CLI: list, show, create, and agent create --mold.
//! Runs the dal binary in a temp dir; asserts exit code and output.
//! Covers §4.5 (COMPREHENSIVE_AGENT_AND_MOLD_PLANS.md).

use std::fs;
use std::process::Command;

fn dal() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dal"))
}

#[test]
fn mold_list_in_empty_dir_succeeds() {
    let temp = tempfile::tempdir().unwrap();
    let out = dal()
        .args(["agent", "mold", "list"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success(), "mold list should succeed: {:?}", out);
}

#[test]
fn mold_create_writes_dal_file() {
    let temp = tempfile::tempdir().unwrap();
    let out = dal()
        .args(["agent", "mold", "create", "test_mold"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "mold create should succeed: {:?}",
        out
    );
    let path = temp.path().join("test_mold.mold.dal");
    assert!(path.exists(), "test_mold.mold.dal should exist");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("mold ") && content.contains("test_mold"));
    assert!(content.contains("agent") && content.contains("type AI"));
}

#[test]
fn mold_list_after_create_shows_file() {
    let temp = tempfile::tempdir().unwrap();
    let _ = dal()
        .args(["agent", "mold", "create", "listed_mold"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let out = dal()
        .args(["agent", "mold", "list"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("listed_mold") || s.contains(".mold.dal"));
}

#[test]
fn mold_show_prints_mold_details() {
    let temp = tempfile::tempdir().unwrap();
    let mold_dal = r#"
mold "show_mold" "1.0"
agent
  type AI
  role "Test role"
"#;
    fs::write(temp.path().join("show_mold.mold.dal"), mold_dal).unwrap();
    let out = dal()
        .args(["agent", "mold", "show", "show_mold"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("show_mold"));
    assert!(s.contains("1.0") || s.contains("version"));
    assert!(s.contains("AI") || s.contains("type"));
}

#[test]
fn agent_create_from_mold_succeeds() {
    let temp = tempfile::tempdir().unwrap();
    let mold_dal = r#"
mold "create_agent_mold" "1.0"
agent
  type Worker
  role "Test"
"#;
    fs::write(temp.path().join("create_agent_mold.mold.dal"), mold_dal).unwrap();
    // Use -- so clap passes --mold through to agent create (avoids top-level flag parse)
    let out = dal()
        .args([
            "agent",
            "create",
            "--",
            "--mold",
            "create_agent_mold",
            "my_agent",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "agent create --mold should succeed: {:?}",
        out
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Agent created") || s.contains("✅"));
}

#[test]
fn agent_create_from_mold_with_param_succeeds() {
    let temp = tempfile::tempdir().unwrap();
    let mold_dal = r#"
mold "param_mold" "1.0"
agent
  type Worker
  role "Env: {{env}}"
"#;
    fs::write(temp.path().join("param_mold.mold.dal"), mold_dal).unwrap();
    // Use -- so clap passes --mold/--param through to agent create
    let out = dal()
        .args([
            "agent",
            "create",
            "--",
            "--mold",
            "param_mold",
            "param_agent",
            "--param",
            "env=production",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Agent created") || s.contains("✅"));
}

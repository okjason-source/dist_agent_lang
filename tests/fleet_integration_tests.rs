//! Integration tests for fleet CLI: create (empty and from-mold), list, show, scale, delete.
//! Covers COMPREHENSIVE_AGENT_AND_MOLD_PLANS.md §5.

use std::fs;
use std::process::Command;

fn dal() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dal"))
}

#[test]
fn fleet_create_empty_and_list() {
    let temp = tempfile::tempdir().unwrap();
    let out = dal()
        .args(["agent", "fleet", "create", "empty_fleet"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "fleet create should succeed: {:?}",
        out
    );
    let out = dal()
        .args(["agent", "fleet", "list"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("empty_fleet"));
}

#[test]
fn fleet_show_and_delete() {
    let temp = tempfile::tempdir().unwrap();
    let _ = dal()
        .args(["agent", "fleet", "create", "show_delete_fleet"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let out = dal()
        .args(["agent", "fleet", "show", "show_delete_fleet"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("show_delete_fleet"));
    assert!(s.contains("Members: 0"));
    let out = dal()
        .args(["agent", "fleet", "delete", "show_delete_fleet"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn fleet_create_from_mold() {
    let temp = tempfile::tempdir().unwrap();
    let mold_dal = r#"
mold "fleet_mold" "1.0"
agent
  type Worker
  role "Fleet worker"
"#;
    fs::write(temp.path().join("fleet_mold.mold.dal"), mold_dal).unwrap();
    let out = dal()
        .args([
            "agent",
            "fleet",
            "create",
            "from_mold_fleet",
            "--from-mold",
            "fleet_mold",
            "--count",
            "2",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "fleet create --from-mold should succeed: {:?}",
        out
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("2 agents") && s.contains("from mold"));
    let out = dal()
        .args(["agent", "fleet", "show", "from_mold_fleet"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("from_mold_fleet"));
    assert!(s.contains("From mold: fleet_mold"));
    assert!(s.contains("Members: 2"));
}

#[test]
fn fleet_scale() {
    let temp = tempfile::tempdir().unwrap();
    let mold_dal = r#"
mold "scale_mold" "1.0"
agent
  type Worker
  role "Scale worker"
"#;
    fs::write(temp.path().join("scale_mold.mold.dal"), mold_dal).unwrap();
    let _ = dal()
        .args([
            "agent",
            "fleet",
            "create",
            "scale_fleet",
            "--from-mold",
            "scale_mold",
            "--count",
            "2",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    // Scale down to 1
    let out = dal()
        .args(["agent", "fleet", "scale", "scale_fleet", "1"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success(), "scale down: {:?}", out);
    let out = dal()
        .args(["agent", "fleet", "show", "scale_fleet"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Members: 1"));
    // Scale up to 3
    let out = dal()
        .args(["agent", "fleet", "scale", "scale_fleet", "3"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success(), "scale up: {:?}", out);
    let out = dal()
        .args(["agent", "fleet", "show", "scale_fleet"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Members: 3"));
}

//! Integration tests for fleet CLI: create (empty and from-mold), list, show, scale, delete,
//! deploy, run (task dispatch via coordinate), health, export.
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

#[test]
fn fleet_deploy_run_dispatches_coordinate() {
    let temp = tempfile::tempdir().unwrap();
    let mold_dal = r#"
mold "deploy_run_mold" "1.0"
agent
  type Worker
  role "Deploy run worker"
"#;
    fs::write(temp.path().join("drm.mold.dal"), mold_dal).unwrap();

    let out = dal()
        .args([
            "agent",
            "fleet",
            "create",
            "deploy_run_fleet",
            "--from-mold",
            "drm",
            "--count",
            "2",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "fleet create: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = dal()
        .args([
            "agent",
            "fleet",
            "deploy",
            "deploy_run_fleet",
            "Process queued items",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "fleet deploy: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = dal()
        .args(["agent", "fleet", "run", "deploy_run_fleet"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "fleet run: {:?} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("dispatched to 2 member"),
        "expected dispatch line, got: {}",
        stdout
    );
}

#[test]
fn fleet_health_shows_deployed_task() {
    let temp = tempfile::tempdir().unwrap();
    let mold_dal = r#"
mold "health_mold" "1.0"
agent
  type Worker
  role "Health mold"
"#;
    fs::write(temp.path().join("hm.mold.dal"), mold_dal).unwrap();
    let _ = dal()
        .args([
            "agent",
            "fleet",
            "create",
            "health_fleet",
            "--from-mold",
            "hm",
            "--count",
            "1",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let out = dal()
        .args(["agent", "fleet", "deploy", "health_fleet", "watchdog tick"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = dal()
        .args(["agent", "fleet", "health", "health_fleet"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("watchdog tick"), "health output: {}", s);
    assert!(s.contains("Status:"), "health output: {}", s);
}

#[test]
fn fleet_export_k8s_contains_agent_env() {
    let temp = tempfile::tempdir().unwrap();
    let mold_dal = r#"
mold "export_mold" "1.0"
agent
  type Worker
  role "Export worker"
"#;
    fs::write(temp.path().join("em.mold.dal"), mold_dal).unwrap();
    let _ = dal()
        .args([
            "agent",
            "fleet",
            "create",
            "export_fleet",
            "--from-mold",
            "em",
            "--count",
            "1",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let out = dal()
        .args([
            "agent",
            "fleet",
            "export",
            "export_fleet",
            "--format",
            "k8s",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(out.status.success(), "export: {:?}", out);
    let yaml = String::from_utf8_lossy(&out.stdout);
    assert!(yaml.contains("kind: Job"), "expected Job YAML: {}", yaml);
    assert!(yaml.contains("DAL_AGENT_ID"), "expected env ref: {}", yaml);
}

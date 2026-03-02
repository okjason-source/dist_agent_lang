//! Integration tests for cross-component CLI (bond, pipe, invoke).
//! Runs the dal binary and asserts on exit code and output.
//! Per CROSS_COMPONENT_IMPLEMENTATION_PLAN: at least one test per bond flow,
//! per invoke workflow, and 2–3 pipe scenarios.

use std::process::Command;

fn dal() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dal"))
}

#[test]
fn test_bond_auth_to_web_dry_run() {
    let out = dal()
        .args([
            "bond",
            "auth-to-web",
            "mytoken",
            "https://example.com",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("dry-run") || s.contains("example.com"));
}

#[test]
fn test_bond_oracle_to_chain_dry_run() {
    let out = dal()
        .args([
            "bond",
            "oracle-to-chain",
            "https://api.example.com",
            "btc",
            "1",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("dry-run") || s.contains("oracle"));
}

#[test]
fn test_bond_chain_to_sync_dry_run() {
    let out = dal()
        .args([
            "bond",
            "chain-to-sync",
            "1",
            "0xabc",
            "https://sync.example.com",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_bond_iot_to_db_dry_run() {
    let out = dal()
        .args([
            "bond",
            "iot-to-db",
            "dev_001",
            "sqlite://:memory:",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_bond_iot_to_web_dry_run() {
    let out = dal()
        .args([
            "bond",
            "iot-to-web",
            "dev_001",
            "https://api.example.com",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_bond_db_to_sync_dry_run() {
    let out = dal()
        .args([
            "bond",
            "db-to-sync",
            "sqlite://:memory:",
            "https://sync.example.com",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_bond_sync_to_db_dry_run() {
    let out = dal()
        .args([
            "bond",
            "sync-to-db",
            "https://sync.example.com",
            "sqlite://:memory:",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_bond_ai_to_service_dry_run() {
    let out = dal()
        .args([
            "bond",
            "ai-to-service",
            "analyze",
            "https://api.example.com",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_bond_service_to_chain_dry_run() {
    let out = dal()
        .args([
            "bond",
            "service-to-chain",
            "https://api.example.com",
            "1",
            "0xaddr",
            "deploy",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_bond_log_to_sync_dry_run() {
    let out = dal()
        .args([
            "bond",
            "log-to-sync",
            "https://sync.example.com",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_bond_unknown_flow_fails() {
    let out = dal()
        .args(["bond", "unknown-flow", "--dry-run"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let s = String::from_utf8_lossy(&out.stderr);
    assert!(s.contains("unknown"));
}

#[test]
fn test_pipe_dry_run() {
    let out = dal()
        .args([
            "pipe",
            "oracle",
            "fetch",
            "x",
            "btc",
            "->",
            "chain",
            "estimate",
            "1",
            "deploy",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("step") || s.contains("dry-run"));
}

#[test]
fn test_pipe_chain_log_dry_run() {
    let out = dal()
        .args([
            "pipe",
            "chain",
            "gas-price",
            "1",
            "->",
            "log",
            "info",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_invoke_price_to_deploy_dry_run() {
    let out = dal()
        .args([
            "invoke",
            "price-to-deploy",
            "https://api.example.com",
            "1",
            "MyContract",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_invoke_iot_ingest_dry_run() {
    let out = dal()
        .args([
            "invoke",
            "iot-ingest",
            "dev_001",
            "sqlite://:memory:",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_invoke_ai_audit_dry_run() {
    let out = dal()
        .args([
            "invoke",
            "ai-audit",
            "sqlite://:memory:",
            "SELECT 1",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_invoke_compliance_check_dry_run() {
    let out = dal()
        .args([
            "invoke",
            "compliance-check",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn test_invoke_unknown_workflow_fails() {
    let out = dal()
        .args(["invoke", "unknown-wf", "--dry-run"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

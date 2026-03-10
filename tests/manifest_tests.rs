//! Manifest and lockfile tests — mutation targets: add_dependencies_if_missing, write_lockfile.
//! See docs/MUTATION_ANALYSIS.md.

use std::collections::HashMap;

use dist_agent_lang::manifest::{
    add_dependencies_if_missing, write_lockfile, LockfileVersionMeta, ResolvedDeps,
};

#[test]
fn add_dependencies_if_missing_adds_one_returns_one() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("dal.toml");
    std::fs::write(
        &manifest_path,
        r#"
[package]
name = "pkg"
version = "1.0.0"

[dependencies]
"#,
    )
    .unwrap();
    let added = add_dependencies_if_missing(&manifest_path, &["foo".to_string()], "1.0.0").unwrap();
    assert_eq!(added, 1, "adding one new dep must return 1");
    let content = std::fs::read_to_string(&manifest_path).unwrap();
    assert!(content.contains("foo") && content.contains("1.0.0"));
}

#[test]
fn add_dependencies_if_missing_idempotent_returns_zero() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("dal.toml");
    std::fs::write(
        &manifest_path,
        r#"
[package]
name = "pkg"
version = "1.0.0"

[dependencies]
foo = "1.0.0"
"#,
    )
    .unwrap();
    let added = add_dependencies_if_missing(&manifest_path, &["foo".to_string()], "1.0.0").unwrap();
    assert_eq!(added, 0, "adding existing dep must return 0");
}

#[test]
fn add_dependencies_if_missing_adds_two_returns_two() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("dal.toml");
    std::fs::write(
        &manifest_path,
        r#"
[package]
name = "pkg"
version = "1.0.0"

[dependencies]
"#,
    )
    .unwrap();
    let added = add_dependencies_if_missing(
        &manifest_path,
        &["bar".to_string(), "baz".to_string()],
        "2.0.0",
    )
    .unwrap();
    assert_eq!(added, 2, "adding two new deps must return 2");
    let content = std::fs::read_to_string(&manifest_path).unwrap();
    assert!(content.contains("bar") && content.contains("baz"));
}

#[test]
fn write_lockfile_quotes_keys_with_special_chars() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("dal.toml");
    std::fs::write(
        &manifest_path,
        "[package]\nname = \"x\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    let mut resolved: ResolvedDeps = HashMap::new();
    resolved.insert("pkg@scope/name".to_string(), dir.path().to_path_buf());
    let version_meta: LockfileVersionMeta = HashMap::new();
    write_lockfile(&manifest_path, &resolved, &version_meta).unwrap();
    let lock_path = dir.path().join("dal.lock");
    let content = std::fs::read_to_string(&lock_path).unwrap();
    assert!(
        content.contains("pkg@scope/name") || content.contains("\"pkg@scope/name\""),
        "lockfile must contain the key (quoted if needed): {}",
        content
    );
    assert!(content.contains("[dependencies]"));
}

#[test]
fn write_lockfile_includes_metadata_section_when_non_empty() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("dal.toml");
    std::fs::write(
        &manifest_path,
        "[package]\nname = \"x\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    let mut resolved: ResolvedDeps = HashMap::new();
    resolved.insert("mypkg".to_string(), dir.path().to_path_buf());
    let mut version_meta: LockfileVersionMeta = HashMap::new();
    version_meta.insert(
        "mypkg".to_string(),
        ("1.0.0".to_string(), "registry".to_string()),
    );
    write_lockfile(&manifest_path, &resolved, &version_meta).unwrap();
    let lock_path = dir.path().join("dal.lock");
    let content = std::fs::read_to_string(&lock_path).unwrap();
    assert!(
        content.contains("[metadata]"),
        "lockfile must have [metadata] when version_meta non-empty: {}",
        content
    );
    assert!(content.contains("1.0.0") && content.contains("registry"));
}

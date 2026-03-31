//! Integration tests for `dist_agent_lang::registry_paths` (dal-registry name/path safety).

use dist_agent_lang::registry_paths::{
    name_to_storage_dir, path_under_storage, sanitize_name, sanitize_version,
};
use std::fs;

#[test]
fn sanitize_name_accepts_scoped_and_plain() {
    assert!(sanitize_name("lodash"));
    assert!(sanitize_name("@scope/pkg"));
    assert!(sanitize_name("pkg@1.0.0"));
    assert!(sanitize_name("a-b_c.d"));
}

#[test]
fn sanitize_name_rejects_empty_traversal_and_bad_chars() {
    assert!(!sanitize_name(""));
    assert!(!sanitize_name("a/../b"));
    assert!(!sanitize_name("pkg "));
    assert!(!sanitize_name("pkg\n"));
    assert!(!sanitize_name("pkg:foo"));
}

#[test]
fn sanitize_version_accepts_semver_like() {
    assert!(sanitize_version("1.0.0"));
    assert!(sanitize_version("1.0.0-rc.1"));
    assert!(sanitize_version("0_a"));
}

#[test]
fn sanitize_version_rejects_traversal_and_space() {
    assert!(!sanitize_version(""));
    assert!(!sanitize_version("1../2"));
    assert!(!sanitize_version("1 0"));
}

#[test]
fn name_to_storage_dir_replaces_scopes() {
    assert_eq!(name_to_storage_dir("@dal/testing"), "_at_dal_slash_testing");
    assert_eq!(name_to_storage_dir("plain"), "plain");
}

#[test]
fn path_under_storage_allows_descendant() {
    let tmp = tempfile::tempdir().unwrap();
    let storage = tmp.path();
    fs::create_dir_all(storage.join("packages").join("pkg").join("versions")).unwrap();
    let deep = storage
        .join("packages")
        .join("pkg")
        .join("versions")
        .join("1.0.0.tgz");
    assert!(
        path_under_storage(&deep, storage),
        "path under storage should be allowed"
    );
}

#[test]
fn path_under_storage_rejects_outside_path() {
    let tmp = tempfile::tempdir().unwrap();
    let storage = tmp.path();
    fs::create_dir_all(storage.join("packages")).unwrap();
    let outside = tempfile::tempdir().unwrap();
    let evil = outside.path().join("evil.tgz");
    fs::write(&evil, b"x").unwrap();
    assert!(
        !path_under_storage(&evil, storage),
        "path outside storage must be rejected"
    );
}

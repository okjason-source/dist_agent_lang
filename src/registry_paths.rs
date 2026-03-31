//! Path and name validation for the `dal-registry` HTTP server (`src/bin/registry_server.rs`).
//! Kept in the library so unit tests and mutation testing can target this logic without the binary.

use std::path::Path;

/// `true` if `s` is a safe package name segment for storage keys (no traversal, allowed charset).
pub fn sanitize_name(s: &str) -> bool {
    !s.is_empty()
        && !s.contains("..")
        && s.chars().all(|c| {
            c.is_alphanumeric() || c == '/' || c == '@' || c == '-' || c == '_' || c == '.'
        })
}

/// `true` if `s` is a safe version string for filenames (semver-ish, no traversal).
pub fn sanitize_version(s: &str) -> bool {
    !s.is_empty()
        && !s.contains("..")
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
}

/// Verify `path` resolves under `storage` (prevents path traversal). Handles non-existent paths
/// by walking up to the first canonicalized ancestor.
pub fn path_under_storage(path: &Path, storage: &Path) -> bool {
    let storage_canon = match storage.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };
    if let Ok(path_canon) = path.canonicalize() {
        return path_canon.starts_with(&storage_canon);
    }
    let mut current = path;
    while let Some(parent) = current.parent() {
        if let Ok(parent_canon) = parent.canonicalize() {
            return parent_canon.starts_with(&storage_canon);
        }
        current = parent;
    }
    false
}

/// Map package name to a single filesystem directory segment (e.g. `@dal/testing` → `_at_dal_slash_testing`).
pub fn name_to_storage_dir(name: &str) -> String {
    name.replace('@', "_at_").replace('/', "_slash_")
}

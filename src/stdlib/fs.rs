//! Host filesystem access from DAL (`fs::read_text`, `fs::write_text`, etc.).
//!
//! Paths are **relative** to a root directory: process current directory by default, or
//! `DAL_FS_ROOT` when set (canonicalized when possible). `..`, absolute paths, and Windows
//! UNC prefixes are rejected (same jail as agent `read_file` / `write_file`).

use std::path::{Path, PathBuf};

/// Max bytes `fs::read_text` will load (defense against huge files).
pub const MAX_READ_BYTES: usize = 8 * 1024 * 1024;

/// Resolve path relative to root; reject path traversal. Returns Err if path escapes root.
pub fn resolve_path_under_root(root: &Path, path: &str) -> Result<PathBuf, String> {
    let path = path.trim();
    if path.is_empty() {
        return Ok(root.to_path_buf());
    }
    if path.contains("..") {
        return Err("Path traversal (..) not allowed".to_string());
    }
    if path.starts_with('/') || (path.len() >= 2 && path.get(..2) == Some("\\\\")) {
        return Err("Absolute paths not allowed".to_string());
    }
    let root_canonical = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => root.to_path_buf(),
    };
    let joined = root_canonical.join(path);
    if joined.exists() {
        let canonical = joined.canonicalize().map_err(|e| e.to_string())?;
        if !canonical.starts_with(&root_canonical) {
            return Err("Path escapes working directory".to_string());
        }
        Ok(canonical)
    } else {
        if !joined.starts_with(&root_canonical) {
            return Err("Path escapes working directory".to_string());
        }
        Ok(joined)
    }
}

/// Root for `fs::*`: `DAL_FS_ROOT` or current working directory.
pub fn filesystem_root() -> PathBuf {
    if let Ok(raw) = std::env::var("DAL_FS_ROOT") {
        let raw = raw.trim();
        if !raw.is_empty() {
            let p = Path::new(raw);
            return p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Read entire file as UTF-8 string. Enforces [`MAX_READ_BYTES`].
pub fn read_text(root: &Path, rel_path: &str) -> Result<String, String> {
    let p = resolve_path_under_root(root, rel_path)?;
    if !p.is_file() {
        return Err("not a file".to_string());
    }
    let meta = std::fs::metadata(&p).map_err(|e| e.to_string())?;
    let len = meta.len() as usize;
    if len > MAX_READ_BYTES {
        return Err(format!(
            "file too large ({} bytes; max {})",
            len, MAX_READ_BYTES
        ));
    }
    std::fs::read_to_string(&p).map_err(|e| e.to_string())
}

/// Write UTF-8 text; creates parent directories. Returns byte length written.
pub fn write_text(root: &Path, rel_path: &str, contents: &str) -> Result<usize, String> {
    let p = resolve_path_under_root(root, rel_path)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&p, contents.as_bytes()).map_err(|e| e.to_string())?;
    Ok(contents.len())
}

/// Append UTF-8 text; creates parent directories and file if missing.
pub fn append_text(root: &Path, rel_path: &str, contents: &str) -> Result<usize, String> {
    let p = resolve_path_under_root(root, rel_path)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
        .map_err(|e| e.to_string())?;
    let n = contents.as_bytes().len();
    f.write_all(contents.as_bytes()).map_err(|e| e.to_string())?;
    Ok(n)
}

/// Whether a file or directory exists under root.
pub fn exists(root: &Path, rel_path: &str) -> Result<bool, String> {
    let p = resolve_path_under_root(root, rel_path)?;
    Ok(p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolve_rejects_dotdot() {
        let tmp = tempfile::tempdir().unwrap();
        let r = tmp.path();
        assert!(resolve_path_under_root(r, "../etc/passwd").is_err());
    }

    #[test]
    fn read_write_roundtrip_under_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_text(root, "sub/hello.txt", "dal").unwrap();
        let s = read_text(root, "sub/hello.txt").unwrap();
        assert_eq!(s, "dal");
    }

    #[test]
    fn append_preserves_prior() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::write(root.join("a"), "x").unwrap();
        append_text(root, "a", "y").unwrap();
        assert_eq!(fs::read_to_string(root.join("a")).unwrap(), "xy");
    }

    #[test]
    fn exists_false_for_missing() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!exists(tmp.path(), "nope.txt").unwrap());
    }
}

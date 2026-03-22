//! Package registry client (R1/R2/R4): fetch package tarball from HTTP URL or IPFS, unpack, cache; publish.
//!
//! See [PACKAGE_REGISTRY_PLAN.md](../docs/development/PACKAGE_REGISTRY_PLAN.md).

use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Resolved source for a package version: URL and/or IPFS CID.
#[derive(Debug, Clone)]
pub struct ResolvedSource {
    pub url: Option<String>,
    pub ipfs_cid: Option<String>,
}

impl ResolvedSource {
    /// Prefer IPFS if DAL_PREFER_IPFS=1 and ipfs_cid present; else prefer url.
    pub fn fetch_source(&self) -> Option<String> {
        let prefer_ipfs = std::env::var("DAL_PREFER_IPFS").as_deref() == Ok("1");
        if prefer_ipfs {
            self.ipfs_cid
                .as_ref()
                .map(|c| format!("ipfs://{}", c))
                .or_else(|| self.url.clone())
        } else {
            self.url
                .clone()
                .or_else(|| self.ipfs_cid.as_ref().map(|c| format!("ipfs://{}", c)))
        }
    }
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("http-interface feature required for registry fetch")]
    HttpInterfaceRequired,

    #[error("fetch failed: {0}")]
    Fetch(String),

    #[error("unpack failed: {0}")]
    Unpack(String),

    #[error("invalid source: {0}")]
    InvalidSource(String),

    #[error("index fetch failed: {0}")]
    IndexFetch(String),

    #[error("version not found: {0}")]
    VersionNotFound(String),

    #[error("publish failed: {0}")]
    Publish(String),
}

fn registry_url() -> String {
    std::env::var("DAL_REGISTRY_URL")
        .unwrap_or_else(|_| "https://registry.dal-lang.org".to_string())
}

/// Packages cache directory: ~/.dal/packages or DAL_PACKAGES_CACHE.
fn packages_cache_dir() -> PathBuf {
    if let Ok(p) = std::env::var("DAL_PACKAGES_CACHE") {
        return PathBuf::from(p);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".dal")
        .join("packages")
}

/// Resolve (name, version_request) to ResolvedSource via registry index.
/// Version request: exact match ("1.0.0") or prefix ("1.0" -> latest 1.0.x).
#[cfg(feature = "http-interface")]
pub fn resolve_version(
    name: &str,
    version_request: &str,
) -> Result<(String, ResolvedSource), RegistryError> {
    let base = registry_url().trim_end_matches('/').to_string();
    let encoded_name = urlencoding::encode(name);
    let url = format!("{}/v1/packages/{}", base, encoded_name);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| RegistryError::IndexFetch(e.to_string()))?;
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| RegistryError::IndexFetch(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(RegistryError::IndexFetch(format!(
            "{}: {}",
            resp.status(),
            resp.text().unwrap_or_default()
        )));
    }
    let json: serde_json::Value = resp
        .json()
        .map_err(|e| RegistryError::IndexFetch(e.to_string()))?;

    let versions = json
        .get("versions")
        .and_then(|v| v.as_object())
        .ok_or_else(|| RegistryError::IndexFetch("missing versions in index".to_string()))?;

    // Resolve version: exact match, or prefix match (1.0 -> latest 1.0.x)
    let version = if versions.contains_key(version_request) {
        version_request.to_string()
    } else {
        let prefix = format!("{}.", version_request.trim_end_matches('.'));
        let matching: Vec<_> = versions
            .keys()
            .filter(|v| v.starts_with(&prefix) || v.starts_with(version_request))
            .collect();
        matching
            .into_iter()
            .max_by(|a, b| {
                // Simple version sort: compare as strings (1.0.0 < 1.0.1)
                a.as_str().cmp(b.as_str())
            })
            .cloned()
            .ok_or_else(|| {
                RegistryError::VersionNotFound(format!("{}@{}", name, version_request))
            })?
    };

    let ver_obj = versions
        .get(&version)
        .and_then(|v| v.as_object())
        .ok_or_else(|| RegistryError::VersionNotFound(format!("{}@{}", name, version)))?;

    let url = ver_obj
        .get("url")
        .and_then(|v| v.as_str())
        .map(String::from);
    let ipfs_cid = ver_obj
        .get("ipfs_cid")
        .or_else(|| ver_obj.get("ipfsCid"))
        .and_then(|v| v.as_str())
        .map(String::from);

    if url.is_none() && ipfs_cid.is_none() {
        return Err(RegistryError::IndexFetch(format!(
            "version {} has no url or ipfs_cid",
            version
        )));
    }

    Ok((version, ResolvedSource { url, ipfs_cid }))
}

#[cfg(not(feature = "http-interface"))]
pub fn resolve_version(
    _name: &str,
    _version_request: &str,
) -> Result<(String, ResolvedSource), RegistryError> {
    Err(RegistryError::HttpInterfaceRequired)
}

/// Fetch package and cache at ~/.dal/packages/{name}/{version}/. Returns package root path.
/// Skips fetch if cache already has dal.toml.
pub fn fetch_and_cache(name: &str, version: &str, source: &str) -> Result<PathBuf, RegistryError> {
    let cache_dir = packages_cache_dir();
    let pkg_dir = cache_dir.join(name).join(version);
    let dal_toml = pkg_dir.join("dal.toml");

    if dal_toml.exists() {
        return Ok(find_package_root(&pkg_dir).unwrap_or(pkg_dir));
    }

    eprintln!("   Fetching {}@{}...", name, version);
    let unpacked = fetch_package(source)?;
    let root = find_package_root(&unpacked).unwrap_or(unpacked.clone());

    std::fs::create_dir_all(&pkg_dir).map_err(|e| RegistryError::Unpack(e.to_string()))?;
    copy_dir_all(&root, &pkg_dir).map_err(|e| RegistryError::Unpack(e.to_string()))?;

    Ok(find_package_root(&pkg_dir).unwrap_or(pkg_dir))
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

/// Resolve version from index, fetch, cache. Returns (path, version, source) for lockfile.
pub fn resolve_and_fetch_with_meta(
    name: &str,
    version_request: &str,
) -> Result<(PathBuf, String, String), RegistryError> {
    let (version, resolved) = resolve_version(name, version_request)?;
    let source = resolved
        .fetch_source()
        .ok_or_else(|| RegistryError::VersionNotFound(format!("{}@{}", name, version_request)))?;
    let path = fetch_and_cache(name, &version, &source)?;
    Ok((path, version, source))
}

/// Resolve version from index, fetch, cache. Returns package root path.
pub fn resolve_and_fetch(name: &str, version_request: &str) -> Result<PathBuf, RegistryError> {
    let (path, _, _) = resolve_and_fetch_with_meta(name, version_request)?;
    Ok(path)
}

/// Directories to skip when packing a package (publish).
const PACK_IGNORE_DIRS: &[&str] = &[".git", "target", "node_modules", ".dal"];

fn collect_package_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if PACK_IGNORE_DIRS.contains(&name) {
                continue;
            }
            collect_package_files(&path, out)?;
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let is_dal = ext == "dal" || path.file_name().map(|n| n == "dal.toml").unwrap_or(false);
            if is_dal || name == "dal.toml" {
                out.push(path);
            }
        }
    }
    Ok(())
}

/// Pack project at manifest_path into a gzipped tarball. Includes dal.toml and all *.dal under project root.
/// Tarball has a single top-level directory {name}-{version}/.
pub fn pack_package(
    manifest_path: &Path,
    name: &str,
    version: &str,
) -> Result<Vec<u8>, RegistryError> {
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let mut files = Vec::new();
    collect_package_files(root, &mut files).map_err(|e| RegistryError::Unpack(e.to_string()))?;
    if files.is_empty() {
        return Err(RegistryError::Publish(
            "no dal.toml or *.dal files found".to_string(),
        ));
    }
    let prefix = format!("{}-{}/", name, version);
    let mut buf = Vec::new();
    {
        let enc = flate2::write::GzEncoder::new(&mut buf, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);
        for path in &files {
            let rel = path.strip_prefix(root).unwrap_or(path);
            let arc_name = format!("{}{}", prefix, rel.to_string_lossy().replace('\\', "/"));
            if path.is_file() {
                let mut f =
                    std::fs::File::open(path).map_err(|e| RegistryError::Unpack(e.to_string()))?;
                tar.append_file(&arc_name, &mut f)
                    .map_err(|e| RegistryError::Unpack(e.to_string()))?;
            }
        }
        tar.finish()
            .map_err(|e| RegistryError::Unpack(e.to_string()))?;
    }
    Ok(buf)
}

/// Publish the package at manifest_path to the registry. Requires DAL_REGISTRY_TOKEN. Uses http-interface.
#[cfg(feature = "http-interface")]
pub fn publish_package(manifest_path: &Path) -> Result<(), RegistryError> {
    let info = crate::manifest::parse_package_info(manifest_path)
        .map_err(|e| RegistryError::Publish(e.to_string()))?;
    let tarball = pack_package(manifest_path, &info.name, &info.version)?;
    let token = std::env::var("DAL_REGISTRY_TOKEN")
        .map_err(|_| RegistryError::Publish("DAL_REGISTRY_TOKEN not set".to_string()))?;
    let base = registry_url().trim_end_matches('/').to_string();
    let encoded_name = urlencoding::encode(&info.name);
    let url = format!(
        "{}/v1/packages/{}/versions/{}",
        base, encoded_name, info.version
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| RegistryError::Publish(e.to_string()))?;
    let resp = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/gzip")
        .body(tarball)
        .send()
        .map_err(|e| RegistryError::Publish(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(RegistryError::Publish(format!("{}: {}", status, body)));
    }
    Ok(())
}

#[cfg(not(feature = "http-interface"))]
pub fn publish_package(_manifest_path: &Path) -> Result<(), RegistryError> {
    Err(RegistryError::HttpInterfaceRequired)
}

/// Fetch a package from `source` (HTTP/HTTPS URL or `ipfs://<cid>`), unpack to a temp dir, return the unpacked path.
///
/// The unpacked directory should contain a package root (e.g. with `dal.toml`). Tarballs may have a
/// single top-level directory (e.g. `package-1.0.0/`) or files at root; caller should locate `dal.toml`.
///
/// Requires `http-interface` feature for network fetch.
pub fn fetch_package(source: &str) -> Result<PathBuf, RegistryError> {
    let bytes = fetch_bytes(source)?;
    unpack_tarball(&bytes)
}

/// Fetch raw bytes from source. HTTP/HTTPS URL or `ipfs://<cid>`.
#[cfg(feature = "http-interface")]
fn fetch_bytes(source: &str) -> Result<Vec<u8>, RegistryError> {
    let source = source.trim();
    if source.starts_with("ipfs://") {
        let cid = source.trim_start_matches("ipfs://").trim();
        if cid.is_empty() {
            return Err(RegistryError::InvalidSource(
                "ipfs:// requires a CID".to_string(),
            ));
        }
        crate::mold::ipfs::download_bytes_from_ipfs(cid)
            .map_err(|e| RegistryError::Fetch(format!("IPFS: {}", e)))
    } else if source.starts_with("http://") || source.starts_with("https://") {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| RegistryError::Fetch(format!("HTTP client: {}", e)))?;
        let resp = client
            .get(source)
            .send()
            .map_err(|e| RegistryError::Fetch(format!("HTTP request: {}", e)))?;
        if !resp.status().is_success() {
            return Err(RegistryError::Fetch(format!(
                "HTTP {}: {}",
                resp.status(),
                resp.text().unwrap_or_default()
            )));
        }
        let bytes = resp
            .bytes()
            .map_err(|e| RegistryError::Fetch(format!("HTTP body: {}", e)))?;
        Ok(bytes.to_vec())
    } else {
        Err(RegistryError::InvalidSource(
            "source must be http(s):// URL or ipfs://<cid>".to_string(),
        ))
    }
}

#[cfg(not(feature = "http-interface"))]
fn fetch_bytes(_source: &str) -> Result<Vec<u8>, RegistryError> {
    Err(RegistryError::HttpInterfaceRequired)
}

/// Unpack tar.gz (or .tgz) bytes into a temp directory. Returns the temp dir path.
fn unpack_tarball(bytes: &[u8]) -> Result<PathBuf, RegistryError> {
    let reader = BufReader::new(Cursor::new(bytes));
    let decoder = flate2::read::GzDecoder::new(reader);
    let mut archive = tar::Archive::new(decoder);

    let temp_dir = tempfile::tempdir().map_err(|e| RegistryError::Unpack(e.to_string()))?;
    let dest = temp_dir.path().to_path_buf();

    archive
        .unpack(&dest)
        .map_err(|e| RegistryError::Unpack(e.to_string()))?;

    // Keep the temp dir alive (caller owns the path; it will be cleaned on process exit or when caller is done)
    let _ = temp_dir.keep();

    Ok(dest)
}

/// Find package root (directory containing dal.toml) within an unpacked path.
/// Tarballs often have a single top-level dir (e.g. package-1.0.0/); this locates dal.toml.
pub fn find_package_root(unpacked: &Path) -> Option<PathBuf> {
    if unpacked.join("dal.toml").exists() {
        return Some(unpacked.to_path_buf());
    }
    if let Ok(entries) = std::fs::read_dir(unpacked) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("dal.toml").exists() {
                return Some(path);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_source() {
        let err = fetch_package("not-a-valid-source").unwrap_err();
        assert!(matches!(err, RegistryError::InvalidSource(_)));
    }

    #[test]
    fn test_find_package_root_at_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(root.join("dal.toml"), "[package]\nname=\"test\"").unwrap();
        assert_eq!(find_package_root(root), Some(root.to_path_buf()));
    }

    #[test]
    fn test_find_package_root_nested() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("pkg-1.0");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("dal.toml"), "[package]\nname=\"test\"").unwrap();
        assert_eq!(find_package_root(dir.path()), Some(nested));
    }

    #[test]
    fn test_resolved_source_prefer_url() {
        let src = ResolvedSource {
            url: Some("https://example.com/pkg.tgz".to_string()),
            ipfs_cid: Some("QmX".to_string()),
        };
        // Without DAL_PREFER_IPFS, prefer url
        assert_eq!(
            src.fetch_source().as_deref(),
            Some("https://example.com/pkg.tgz")
        );
    }

    #[test]
    fn test_resolved_source_url_only() {
        let src = ResolvedSource {
            url: Some("https://example.com/pkg.tgz".to_string()),
            ipfs_cid: None,
        };
        assert_eq!(
            src.fetch_source().as_deref(),
            Some("https://example.com/pkg.tgz")
        );
    }

    #[test]
    fn test_resolved_source_ipfs_fallback() {
        let src = ResolvedSource {
            url: None,
            ipfs_cid: Some("QmX".to_string()),
        };
        assert_eq!(src.fetch_source().as_deref(), Some("ipfs://QmX"));
    }

    // Mutation testing: registry_url and packages_cache_dir (see docs/MUTATION_ANALYSIS.md).
    #[test]
    fn test_registry_url_default() {
        std::env::remove_var("DAL_REGISTRY_URL");
        let u = registry_url();
        assert!(
            u.contains("registry") && u.contains("dal"),
            "default registry URL should contain registry and dal: {}",
            u
        );
    }

    #[test]
    fn test_registry_url_from_env() {
        std::env::set_var("DAL_REGISTRY_URL", "https://custom.example.com/registry");
        let u = registry_url();
        std::env::remove_var("DAL_REGISTRY_URL");
        assert_eq!(u, "https://custom.example.com/registry");
    }

    #[test]
    fn test_packages_cache_dir_from_env() {
        let custom = std::path::PathBuf::from("/tmp/dal-packages-cache");
        std::env::set_var("DAL_PACKAGES_CACHE", &custom);
        let p = packages_cache_dir();
        std::env::remove_var("DAL_PACKAGES_CACHE");
        assert_eq!(p, custom);
    }

    #[test]
    fn test_packages_cache_dir_default_contains_dal_packages() {
        std::env::remove_var("DAL_PACKAGES_CACHE");
        let p = packages_cache_dir();
        let s = p.to_string_lossy();
        assert!(
            s.contains(".dal") && s.contains("packages"),
            "default cache dir should contain .dal and packages: {}",
            s
        );
    }
}

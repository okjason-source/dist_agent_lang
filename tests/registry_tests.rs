//! Registry and ResolvedSource tests — catch mutations in registry_url, packages_cache_dir, fetch_source.

use dist_agent_lang::registry::ResolvedSource;

/// Catches: replace fetch_source return with None/empty; replace == with != in DAL_PREFER_IPFS.
#[test]
fn test_resolved_source_fetch_source_prefers_url_when_no_prefer_ipfs() {
    let r = ResolvedSource {
        url: Some("https://registry.example.com/tarball.tgz".to_string()),
        ipfs_cid: Some("QmXXX".to_string()),
    };
    let out = r.fetch_source();
    assert_eq!(
        out.as_deref(),
        Some("https://registry.example.com/tarball.tgz"),
        "without DAL_PREFER_IPFS, url should be preferred"
    );
}

/// Catches: replace fetch_source logic; url-only returns url.
#[test]
fn test_resolved_source_fetch_source_url_only() {
    let r = ResolvedSource {
        url: Some("https://x.org/p.tgz".to_string()),
        ipfs_cid: None,
    };
    assert_eq!(r.fetch_source(), Some("https://x.org/p.tgz".to_string()));
}

/// Catches: ipfs_cid branch; ipfs-only returns ipfs:// URL.
#[test]
fn test_resolved_source_fetch_source_ipfs_only() {
    let r = ResolvedSource {
        url: None,
        ipfs_cid: Some("QmYYY".to_string()),
    };
    assert_eq!(r.fetch_source(), Some("ipfs://QmYYY".to_string()));
}

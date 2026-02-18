// IPFS upload/download for mold files. Requires http-interface (reqwest).

use std::path::Path;

/// Default IPFS API base when DAL_IPFS_API is not set (local node).
pub const DEFAULT_IPFS_API: &str = "http://127.0.0.1:5001";

fn api_base() -> String {
    std::env::var("DAL_IPFS_API").unwrap_or_else(|_| DEFAULT_IPFS_API.to_string())
}

/// Upload a file to IPFS via the HTTP API (POST /api/v0/add). Returns the CID (Hash).
pub fn upload_file(api_base: &str, path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read file: {}", e))?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mold.json");
    let part = reqwest::blocking::multipart::Part::bytes(bytes)
        .file_name(name.to_string())
        .mime_str("application/json")
        .map_err(|e| format!("multipart: {}", e))?;
    let form = reqwest::blocking::multipart::Form::new().part("file", part);
    let url = format!("{}/api/v0/add", api_base.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("http client: {}", e))?;
    let resp = client
        .post(&url)
        .multipart(form)
        .send()
        .map_err(|e| format!("IPFS add request: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("IPFS add failed {}: {}", status, body));
    }
    let json: serde_json::Value = resp.json().map_err(|e| format!("IPFS add response: {}", e))?;
    let hash = json
        .get("Hash")
        .and_then(|v: &serde_json::Value| v.as_str())
        .ok_or_else(|| "IPFS response missing Hash".to_string())?;
    Ok(hash.to_string())
}

/// Download content from IPFS by CID (POST /api/v0/cat?arg=<cid>).
pub fn cat(api_base: &str, cid: &str) -> Result<String, String> {
    let url = format!(
        "{}/api/v0/cat?arg={}",
        api_base.trim_end_matches('/'),
        cid
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("http client: {}", e))?;
    let resp = client
        .post(&url)
        .send()
        .map_err(|e| format!("IPFS cat request: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("IPFS cat failed {}: {}", status, body));
    }
    let body = resp.text().map_err(|e| format!("IPFS cat body: {}", e))?;
    Ok(body)
}

/// Upload mold file at path; uses DAL_IPFS_API or localhost:5001.
pub fn upload_mold_to_ipfs(path: &Path) -> Result<String, String> {
    let base = api_base();
    upload_file(&base, path)
}

/// Download mold content from IPFS by CID.
pub fn download_mold_from_ipfs(cid: &str) -> Result<String, String> {
    let base = api_base();
    cat(&base, cid)
}

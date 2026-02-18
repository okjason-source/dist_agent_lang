// Mold: config, parser, create_from_mold, list_local, scaffold, IPFS (optional).

pub mod config;
pub mod parser;

#[cfg(feature = "http-interface")]
pub mod ipfs;
#[cfg(feature = "web3")]
pub mod web3;

pub use config::MoldConfig;
pub use parser::{load_mold_from_path, parse_mold_content};

#[cfg(feature = "http-interface")]
pub use ipfs::{download_mold_from_ipfs, upload_mold_to_ipfs};
#[cfg(feature = "web3")]
pub use web3::{get_mold_info, mint_mold, mold_id_by_ipfs_hash, use_mold, MoldInfo};

/// Use on-chain mold (pay + cap check), then load from IPFS and spawn. Returns agent_id.
/// Requires web3 feature. Env: DAL_PRIVATE_KEY, DAL_MOLD_REGISTRY_ADDRESS, DAL_RPC_URL.
#[cfg(feature = "web3")]
pub fn use_mold_and_spawn(
    mold_id: u64,
    base: &Path,
    name_override: Option<&str>,
) -> Result<crate::stdlib::agent::AgentContext, String> {
    use crate::mold::web3;
    let info = web3::get_mold_info(mold_id)?;
    if !info.active {
        return Err(format!("Mold {} is not active", mold_id));
    }
    web3::use_mold(mold_id, info.mint_fee)?;
    let ipfs_source = format!("ipfs://{}", info.ipfs_hash.trim_start_matches("ipfs://"));
    create_from_mold_source(&ipfs_source, base, name_override)
}

use crate::stdlib::agent::{spawn, AgentConfig, AgentType, LifecycleHooks};
use crate::runtime::values::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Build AgentConfig from a MoldConfig. Name override: if `name_override` is Some, use it as agent name; else use mold.name.
pub fn mold_config_to_agent_config(
    mold: &MoldConfig,
    name_override: Option<&str>,
) -> Result<AgentConfig, String> {
    let name = name_override
        .map(String::from)
        .unwrap_or_else(|| mold.name.clone());
    let agent_type_str = mold.agent.agent_type.trim().to_lowercase();
    let agent_type = AgentType::from_string(&agent_type_str)
        .ok_or_else(|| format!("unknown agent type in mold: {}", mold.agent.agent_type))?;

    let max_memory = MoldConfig::memory_limit_to_max_memory(&mold.agent.memory_limit);
    let mut metadata: HashMap<String, Value> = mold
        .metadata
        .iter()
        .map(|(k, v)| (k.clone(), json_value_to_runtime_value(v)))
        .collect();
    metadata.insert("mold_name".to_string(), Value::String(mold.name.clone()));
    metadata.insert("mold_version".to_string(), Value::String(mold.version.clone()));

    let lifecycle = mold.lifecycle.as_ref().map(|l| LifecycleHooks {
        on_create: l.on_create.clone(),
        on_message: l.on_message.clone(),
        on_evolve: l.on_evolve.clone(),
        on_destroy: l.on_destroy.clone(),
    });

    Ok(AgentConfig::new(name, agent_type)
        .with_role(mold.agent.role.clone())
        .with_capabilities(mold.agent.capabilities.clone())
        .with_trust_level(mold.agent.trust_level.clone())
        .with_max_memory(max_memory)
        .with_learning_enabled(mold.agent.learning)
        .with_communication_enabled(mold.agent.communication)
        .with_coordination_enabled(mold.agent.coordination)
        .with_metadata(metadata)
        .with_lifecycle(lifecycle))
}

fn json_value_to_runtime_value(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => Value::List(
            arr.iter().map(json_value_to_runtime_value).collect(),
        ),
        serde_json::Value::Object(obj) => Value::Map(
            obj.iter()
                .map(|(k, v)| (k.clone(), json_value_to_runtime_value(v)))
                .collect(),
        ),
    }
}

/// Load mold from path, convert to AgentConfig, and spawn. Name override optional.
pub fn create_from_mold_path(
    path: &Path,
    name_override: Option<&str>,
) -> Result<crate::stdlib::agent::AgentContext, String> {
    let mold = load_mold_from_path(path)?;
    let config = mold_config_to_agent_config(&mold, name_override)?;
    spawn(config)
}

/// Load mold from a source: local path/name (resolved with `base`) or `ipfs://<cid>` (requires http-interface).
pub fn load_mold_from_source(source: &str, base: &Path) -> Result<MoldConfig, String> {
    let source = source.trim();
    if source.starts_with("ipfs://") {
        #[cfg(feature = "http-interface")]
        {
            let cid = source.trim_start_matches("ipfs://").trim_end_matches('/');
            let content = download_mold_from_ipfs(cid)?;
            parse_mold_content(&content)
        }
        #[cfg(not(feature = "http-interface"))]
        {
            let _ = base;
            Err("IPFS sources require building with http-interface feature (default)".to_string())
        }
    } else {
        let path = resolve_mold_path(base, source)?;
        load_mold_from_path(&path)
    }
}

/// Create agent from mold source (path/name or ipfs://cid). Name override optional.
pub fn create_from_mold_source(
    source: &str,
    base: &Path,
    name_override: Option<&str>,
) -> Result<crate::stdlib::agent::AgentContext, String> {
    let mold = load_mold_from_source(source, base)?;
    let config = mold_config_to_agent_config(&mold, name_override)?;
    spawn(config)
}

/// Discover local mold files under `base`: base, base/mold, base/mold/samples.
/// Returns paths for files named `*.mold.json` or `*.mold.dal`.
pub fn list_local_paths(base: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let dirs = [
        base.to_path_buf(),
        base.join("mold"),
        base.join("mold").join("samples"),
    ];
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_file() {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name.ends_with(".mold.json") || name.ends_with(".mold.dal") {
                        out.push(p);
                    }
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Resolve path or name to a mold file path. If `path_or_name` is an existing path, return it; else try list_local and match by stem or mold name.
pub fn resolve_mold_path(base: &Path, path_or_name: &str) -> Result<PathBuf, String> {
    let p = Path::new(path_or_name);
    if p.is_absolute() && p.exists() {
        return Ok(p.to_path_buf());
    }
    let relative = base.join(path_or_name);
    if relative.exists() {
        return Ok(relative);
    }
    // Try with .mold.json suffix
    let with_json = base.join(format!("{}.mold.json", path_or_name));
    if with_json.exists() {
        return Ok(with_json);
    }
    let with_dal = base.join(format!("{}.mold.dal", path_or_name));
    if with_dal.exists() {
        return Ok(with_dal);
    }
    let list = list_local_paths(base);
    for path in &list {
        if path.file_stem().and_then(|s| s.to_str()).map(|s| s.trim_end_matches(".mold") == path_or_name).unwrap_or(false) {
            return Ok(path.clone());
        }
        if let Ok(m) = load_mold_from_path(path) {
            if m.name == path_or_name {
                return Ok(path.clone());
            }
        }
    }
    Err(format!("mold not found: {} (tried path, {}.mold.json, {}.mold.dal, and local list)", path_or_name, path_or_name, path_or_name))
}

/// Scaffold a new mold file at `out_path` with minimal JSON (strict fields only).
pub fn scaffold_mold(name: &str, out_path: &Path) -> Result<(), String> {
    let content = format!(
        r#"{{
  "name": "{}",
  "version": "1.0",
  "agent": {{
    "type": "AI"
  }}
}}
"#,
        name
    );
    std::fs::write(out_path, content).map_err(|e| format!("write {}: {}", out_path.display(), e))
}

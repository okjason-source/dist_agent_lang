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
    create_from_mold_source(&ipfs_source, base, name_override, None)
}

use crate::runtime::values::Value;
use crate::stdlib::agent::{spawn, AgentConfig, AgentType, LifecycleHooks};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Principal-owned values (process config). Trust and context path are never taken from the mold.
/// See COMPREHENSIVE_AGENT_AND_MOLD_PLANS.md §3.
#[derive(Debug, Clone, Default)]
pub struct PrincipalOverrides {
    /// Shell/trust level from agent.toml [agent.sh] or DAL_AGENT_SHELL_TRUST. Never use mold's trust.
    pub trust_level: String,
}

/// Load principal overrides from process (agent.toml, dal.toml, env). Used when merging mold config.
pub fn load_principal_overrides() -> PrincipalOverrides {
    let sh_config = crate::stdlib::sh::load_sh_config();
    PrincipalOverrides {
        trust_level: sh_config.trust.to_string(),
    }
}

/// Substitute `{{key}}` in `s` with `params.get(key)`. Keys are case-sensitive.
fn substitute_params(s: &str, params: &HashMap<String, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in params {
        let placeholder = format!("{{{{{}}}}}", k);
        out = out.replace(&placeholder, v);
    }
    out
}

/// Build AgentConfig from a MoldConfig with optional principal overrides and create-time params.
/// Principal owns trust: when `principal` is Some, trust_level is taken from principal only (never from mold).
/// When `params` is Some, each k=v is merged into metadata and `{{key}}` in role/capabilities is substituted.
/// Context path is always from process (evolve::get_path()); it is not stored in AgentConfig.
/// Name override: if `name_override` is Some, use it as agent name; else use mold.name.
pub fn mold_config_to_agent_config(
    mold: &MoldConfig,
    name_override: Option<&str>,
    principal: Option<&PrincipalOverrides>,
    params: Option<&HashMap<String, String>>,
) -> Result<AgentConfig, String> {
    let name = name_override
        .map(String::from)
        .unwrap_or_else(|| mold.name.clone());
    let agent_type_str = mold.agent.agent_type.trim().to_lowercase();
    let agent_type = AgentType::from_string(&agent_type_str)
        .ok_or_else(|| format!("unknown agent type in mold: {}", mold.agent.agent_type))?;

    let trust_level = principal
        .map(|p| p.trust_level.as_str())
        .unwrap_or("sandboxed")
        .to_string();

    let max_memory = MoldConfig::memory_limit_to_max_memory(&mold.agent.memory_limit);
    let mut metadata: HashMap<String, Value> = mold
        .metadata
        .iter()
        .map(|(k, v)| (k.clone(), json_value_to_runtime_value(v)))
        .collect();
    metadata.insert("mold_name".to_string(), Value::String(mold.name.clone()));
    metadata.insert(
        "mold_version".to_string(),
        Value::String(mold.version.clone()),
    );
    if let Some(p) = params {
        for (k, v) in p {
            metadata.insert(k.clone(), Value::String(v.clone()));
        }
    }

    let role = params
        .map(|p| substitute_params(&mold.agent.role, p))
        .unwrap_or_else(|| mold.agent.role.clone());
    let capabilities: Vec<String> = params
        .map(|p| {
            mold.agent
                .capabilities
                .iter()
                .map(|c| substitute_params(c, p))
                .collect()
        })
        .unwrap_or_else(|| mold.agent.capabilities.clone());

    let lifecycle = mold.lifecycle.as_ref().map(|l| LifecycleHooks {
        on_create: l.on_create.clone(),
        on_message: l.on_message.clone(),
        on_evolve: l.on_evolve.clone(),
        on_destroy: l.on_destroy.clone(),
    });

    let skills: Vec<String> = if mold.agent.skills.is_empty() {
        crate::skills::DEFAULT_LEARNING_PATH_SKILLS
            .iter()
            .map(|s| (*s).to_string())
            .collect()
    } else {
        mold.agent.skills.clone()
    };

    Ok(AgentConfig::new(name, agent_type)
        .with_role(role)
        .with_capabilities(capabilities)
        .with_skills(skills)
        .with_trust_level(trust_level)
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
        serde_json::Value::Array(arr) => {
            Value::List(arr.iter().map(json_value_to_runtime_value).collect())
        }
        serde_json::Value::Object(obj) => Value::Map(
            obj.iter()
                .map(|(k, v)| (k.clone(), json_value_to_runtime_value(v)))
                .collect(),
        ),
    }
}

/// Load mold from path, convert to AgentConfig (with principal overrides for trust), and spawn.
/// Evolve/context path is always from process (agent.toml / DAL_AGENT_CONTEXT_PATH).
/// Create-time params are merged into metadata and substituted in role/capabilities ({{key}}).
pub fn create_from_mold_path(
    path: &Path,
    name_override: Option<&str>,
    params: Option<&HashMap<String, String>>,
) -> Result<crate::stdlib::agent::AgentContext, String> {
    let mold = load_mold_from_path(path)?;
    let principal = load_principal_overrides();
    let config = mold_config_to_agent_config(&mold, name_override, Some(&principal), params)?;
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
/// Principal (agent.toml / env) supplies trust; context path is from process.
/// Create-time params are merged into metadata and substituted in role/capabilities ({{key}}).
pub fn create_from_mold_source(
    source: &str,
    base: &Path,
    name_override: Option<&str>,
    params: Option<&HashMap<String, String>>,
) -> Result<crate::stdlib::agent::AgentContext, String> {
    let mold = load_mold_from_source(source, base)?;
    let principal = load_principal_overrides();
    let config = mold_config_to_agent_config(&mold, name_override, Some(&principal), params)?;
    spawn(config)
}

/// Discover local mold files under `base`: base, base/mold, base/mold/samples.
/// Returns paths for files named `*.mold.dal` (canonical) or `*.mold.json` (legacy). .mold.dal listed first per stem.
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
                    if name.ends_with(".mold.dal") || name.ends_with(".mold.json") {
                        out.push(p);
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| {
        let a_dal = a.to_string_lossy().ends_with(".mold.dal");
        let b_dal = b.to_string_lossy().ends_with(".mold.dal");
        match (a_dal, b_dal) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        }
    });
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
    // Prefer .mold.dal (canonical), then .mold.json (legacy)
    let with_dal = base.join(format!("{}.mold.dal", path_or_name));
    if with_dal.exists() {
        return Ok(with_dal);
    }
    let with_json = base.join(format!("{}.mold.json", path_or_name));
    if with_json.exists() {
        return Ok(with_json);
    }
    let list = list_local_paths(base);
    for path in &list {
        if path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.trim_end_matches(".mold") == path_or_name)
            .unwrap_or(false)
        {
            return Ok(path.clone());
        }
        if let Ok(m) = load_mold_from_path(path) {
            if m.name == path_or_name {
                return Ok(path.clone());
            }
        }
    }
    Err(format!(
        "mold not found: {} (tried path, {}.mold.dal, {}.mold.json, and local list)",
        path_or_name, path_or_name, path_or_name
    ))
}

/// Scaffold a new .mold.dal file at `out_path` (canonical format).
/// Includes default learning-path skills (development, creative, office, home); see docs/DAL_CEO_APP_PLAN.md.
pub fn scaffold_mold(name: &str, out_path: &Path) -> Result<(), String> {
    let content = format!(
        r#"mold "{}" "1.0"
agent
  type AI
  role ""
  capabilities
  skills "development" "creative" "office" "home"
  memory_limit "256MB"
  learning true
  communication true
  coordination true
"#,
        name
    );
    std::fs::write(out_path, content).map_err(|e| format!("write {}: {}", out_path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn principal_trust_overrides_mold_trust() {
        let mold = parse_mold_content(
            r#"{"name":"t","version":"1","agent":{"type":"AI","trustLevel":"trusted","role":"x"}}"#,
        )
        .expect("valid json");
        assert_eq!(mold.agent.trust_level, "trusted");

        let principal = PrincipalOverrides {
            trust_level: "sandboxed".to_string(),
        };
        let config =
            mold_config_to_agent_config(&mold, None, Some(&principal), None).expect("merge ok");
        assert_eq!(
            config.trust_level, "sandboxed",
            "principal trust must override mold"
        );
    }

    #[test]
    fn merge_without_principal_uses_default_trust() {
        let mold = parse_mold_content(
            r#"{"name":"t","version":"1","agent":{"type":"AI","trustLevel":"trusted"}}"#,
        )
        .expect("valid json");
        let config = mold_config_to_agent_config(&mold, None, None, None).expect("merge ok");
        assert_eq!(
            config.trust_level, "sandboxed",
            "no principal => default sandboxed"
        );
    }

    #[test]
    fn create_time_params_merged_and_substituted() {
        let mold = parse_mold_content(
            r#"{"name":"t","version":"1","agent":{"type":"AI","role":"Hello {{who}}","capabilities":["{{cap}}"]}}"#,
        )
        .expect("valid json");
        let principal = PrincipalOverrides {
            trust_level: "sandboxed".to_string(),
        };
        let mut params = HashMap::new();
        params.insert("who".to_string(), "World".to_string());
        params.insert("cap".to_string(), "read".to_string());
        params.insert("extra".to_string(), "in_metadata".to_string());
        let config = mold_config_to_agent_config(&mold, None, Some(&principal), Some(&params))
            .expect("merge ok");
        assert_eq!(config.role, "Hello World");
        assert_eq!(config.capabilities, &["read"]);
        match config.metadata.get("extra") {
            Some(Value::String(s)) => assert_eq!(s, "in_metadata"),
            _ => panic!("expected metadata.extra to be string"),
        }
    }
}

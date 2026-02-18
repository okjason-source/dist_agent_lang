// Mold stdlib — Expose mold load/spawn to DAL code.
//
// Aligns with mold/STDLIB_PROPOSAL.md and mold/DEFINITION.md.
// - mold::load(source) — Load mold from path or name, return config map
// - mold::spawn_from(source, name_override?) — Load mold + spawn agent, return agent_id
// - mold::list() — List local mold paths
// - mold::get_info(mold_id) — Query on-chain mold info (requires web3)
// - mold::use_mold(mold_id, name_override?) — Pay + spawn from on-chain mold (requires web3)

use crate::mold::{create_from_mold_source, load_mold_from_source, MoldConfig};
use crate::runtime::values::Value;
use std::collections::HashMap;
use std::path::Path;

/// Load mold from source (path, name, or ipfs://cid). Returns config as map for DAL.
/// Base is typically std::env::current_dir() or script directory.
pub fn load(source: &str, base: &Path) -> Result<Value, String> {
    let mold = load_mold_from_source(source, base)?;
    Ok(mold_config_to_value(&mold))
}

/// Create agent from mold source. Returns agent_id. Name override optional.
pub fn spawn_from(
    source: &str,
    base: &Path,
    name_override: Option<&str>,
) -> Result<String, String> {
    let ctx = create_from_mold_source(source, base, name_override)?;
    Ok(ctx.agent_id)
}

/// List local mold file paths (base, mold/, mold/samples). Returns list of path strings.
pub fn list(base: &Path) -> Value {
    let paths = crate::mold::list_local_paths(base);
    Value::List(
        paths
            .into_iter()
            .map(|p| Value::String(p.display().to_string()))
            .collect(),
    )
}

/// Get on-chain mold info. Requires web3 feature.
#[cfg(feature = "web3")]
pub fn get_info(mold_id: u64) -> Result<Value, String> {
    let info = crate::mold::get_mold_info(mold_id)?;
    Ok(mold_info_to_value(&info))
}

#[cfg(not(feature = "web3"))]
pub fn get_info(_mold_id: u64) -> Result<Value, String> {
    Err("mold::get_info requires web3 feature (cargo build --features web3)".to_string())
}

/// Use on-chain mold (pay + cap check), then spawn. Returns agent_id. Requires web3 feature.
#[cfg(feature = "web3")]
pub fn use_mold(
    mold_id: u64,
    base: &Path,
    name_override: Option<&str>,
) -> Result<String, String> {
    let ctx = crate::mold::use_mold_and_spawn(mold_id, base, name_override)?;
    Ok(ctx.agent_id)
}

#[cfg(not(feature = "web3"))]
pub fn use_mold(_mold_id: u64, _base: &Path, _name_override: Option<&str>) -> Result<String, String> {
    Err("mold::use_mold requires web3 feature (cargo build --features web3)".to_string())
}

/// Convert MoldInfo to Value::Map for DAL.
#[cfg(feature = "web3")]
fn mold_info_to_value(info: &crate::mold::MoldInfo) -> Value {
    let mut map = HashMap::new();
    map.insert("creator".to_string(), Value::String(format!("{:?}", info.creator)));
    map.insert("ipfs_hash".to_string(), Value::String(info.ipfs_hash.clone()));
    map.insert("mint_fee".to_string(), Value::Int(info.mint_fee as i64));
    map.insert("mint_count".to_string(), Value::Int(info.mint_count as i64));
    map.insert("max_use_count".to_string(), Value::Int(info.max_use_count as i64));
    map.insert("active".to_string(), Value::Bool(info.active));
    map.insert("created_at".to_string(), Value::Int(info.created_at as i64));
    map.insert("updated_at".to_string(), Value::Int(info.updated_at as i64));
    Value::Map(map)
}

/// Convert MoldConfig to Value::Map for DAL.
fn mold_config_to_value(mold: &MoldConfig) -> Value {
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String(mold.name.clone()));
    map.insert("version".to_string(), Value::String(mold.version.clone()));

    let mut agent_map = HashMap::new();
    agent_map.insert("type".to_string(), Value::String(mold.agent.agent_type.clone()));
    agent_map.insert("role".to_string(), Value::String(mold.agent.role.clone()));
    agent_map.insert(
        "capabilities".to_string(),
        Value::List(
            mold.agent
                .capabilities
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        ),
    );
    agent_map.insert(
        "trust_level".to_string(),
        Value::String(mold.agent.trust_level.clone()),
    );
    agent_map.insert(
        "learning".to_string(),
        Value::Bool(mold.agent.learning),
    );
    agent_map.insert(
        "communication".to_string(),
        Value::Bool(mold.agent.communication),
    );
    agent_map.insert(
        "coordination".to_string(),
        Value::Bool(mold.agent.coordination),
    );
    agent_map.insert(
        "memory_limit".to_string(),
        Value::String(mold.agent.memory_limit.clone()),
    );
    map.insert("agent".to_string(), Value::Map(agent_map));

    if !mold.parameters.is_empty() {
        let params: HashMap<String, Value> = mold
            .parameters
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        map.insert("parameters".to_string(), Value::Map(params));
    }

    if !mold.dependencies.is_empty() {
        map.insert(
            "dependencies".to_string(),
            Value::List(
                mold.dependencies
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }

    Value::Map(map)
}

// Mold config: in-memory representation of a .mold.dal (or JSON) mold.
// Maps to mold/DEFINITION.md §3.

use serde::Deserialize;
use std::collections::HashMap;

/// Lifecycle hooks: DAL code strings executed at agent events.
/// See mold/DEFINITION.md §3 lifecycle block.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MoldLifecycle {
    pub on_create: Option<String>,
    pub on_message: Option<String>,
    pub on_evolve: Option<String>,
    pub on_destroy: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MoldConfig {
    pub name: String,
    pub version: String,
    pub agent: MoldAgentBlock,
    pub parameters: HashMap<String, String>,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub lifecycle: Option<MoldLifecycle>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MoldAgentBlock {
    #[serde(rename = "type")]
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub trust_level: String,
    pub learning: bool,
    pub communication: bool,
    pub coordination: bool,
    pub memory_limit: String,
    pub role: String,
}

impl MoldConfig {
    /// Memory limit string (e.g. "2GB") to approximate max_memory in bytes/pages.
    pub fn memory_limit_to_max_memory(s: &str) -> usize {
        let s = s.trim().to_uppercase();
        if s.is_empty() {
            return 1000;
        }
        let (num_part, unit) = if s.ends_with("GB") {
            (s.trim_end_matches("GB"), 1024 * 1024 * 1024)
        } else if s.ends_with("MB") {
            (s.trim_end_matches("MB"), 1024 * 1024)
        } else if s.ends_with("KB") {
            (s.trim_end_matches("KB"), 1024)
        } else {
            (s.as_str(), 1)
        };
        let n: usize = num_part.trim().parse().unwrap_or(2);
        (n * unit).min(usize::MAX) / 4096 // rough page count for max_memory
    }
}

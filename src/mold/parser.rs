// Parse mold config from file content. Supports JSON format (e.g. .mold.dal as JSON).

use crate::mold::config::MoldConfig;
use std::path::Path;

/// Parse mold config from string. Expects JSON object with name, version, agent { }, etc.
pub fn parse_mold_content(content: &str) -> Result<MoldConfig, String> {
    let content = content.trim();
    if content.is_empty() {
        return Err("empty mold content".to_string());
    }
    if !content.starts_with('{') {
        return Err("mold config must be JSON object (start with '{')".to_string());
    }
    serde_json::from_str(content).map_err(|e| format!("invalid mold JSON: {}", e))
}

/// Load and parse mold from a file path.
pub fn load_mold_from_path(path: &Path) -> Result<MoldConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("read mold file {}: {}", path.display(), e))?;
    parse_mold_content(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_mold() {
        let json = r#"{"name":"Test","version":"1.0","agent":{"type":"AI","capabilities":["read"],"trustLevel":"standard"}}"#;
        let mold = parse_mold_content(json).unwrap();
        assert_eq!(mold.name, "Test");
        assert_eq!(mold.version, "1.0");
        assert_eq!(mold.agent.agent_type, "AI");
        assert_eq!(mold.agent.capabilities, &["read"]);
    }
}

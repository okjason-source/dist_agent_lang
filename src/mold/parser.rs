// Parse mold config from file content. Canonical format: .mold.dal (DAL block syntax).
// Legacy: JSON (content starting with '{') is still accepted.

use crate::mold::config::{MoldAgentBlock, MoldConfig, MoldLifecycle};
use std::path::Path;

/// Parse mold config from string. If content starts with '{', parse as JSON (legacy).
/// Otherwise parse as .mold.dal (canonical) block syntax.
pub fn parse_mold_content(content: &str) -> Result<MoldConfig, String> {
    let content = content.trim();
    if content.is_empty() {
        return Err("empty mold content".to_string());
    }
    if content.starts_with('{') {
        return serde_json::from_str(content).map_err(|e| format!("invalid mold JSON: {}", e));
    }
    parse_mold_dal(content)
}

/// Parse .mold.dal block syntax. Example:
///   mold "name" "1.0"
///   agent
///     type AI
///     role "Assistant"
///     capabilities "read" "write"
///     memory_limit "256MB"
///   lifecycle
///     on_create "evolve::append_log(...)"
fn parse_mold_dal(content: &str) -> Result<MoldConfig, String> {
    let lines: Vec<&str> = content
        .lines()
        .map(|l| {
            let l = l.trim();
            if let Some(i) = l.find("//") {
                l[..i].trim()
            } else {
                l
            }
        })
        .filter(|l| !l.is_empty())
        .collect();

    let name;
    let mut version = "1.0".to_string();
    let mut agent = MoldAgentBlock::default();
    let mut lifecycle: Option<MoldLifecycle> = None;

    let mut i = 0;
    if i >= lines.len() {
        return Err("mold: expected 'mold \"name\" \"version\"'".to_string());
    }
    let mold_line = lines[i];
    if !mold_line.starts_with("mold ") && !mold_line.eq_ignore_ascii_case("mold") {
        return Err(format!(
            "mold: expected 'mold \"name\" \"version\"', got '{}'",
            mold_line
        ));
    }
    let rest = mold_line["mold".len()..].trim();
    let (n, v) = parse_two_tokens(rest)?;
    name = n;
    if !v.is_empty() {
        version = v;
    }
    if name.is_empty() {
        return Err("mold: name is required".to_string());
    }
    i += 1;

    if i >= lines.len() || !lines[i].eq_ignore_ascii_case("agent") {
        return Err("mold: expected 'agent' block".to_string());
    }
    i += 1;

    while i < lines.len() {
        let line = lines[i];
        if line.eq_ignore_ascii_case("lifecycle") {
            i += 1;
            lifecycle = Some(parse_lifecycle(&lines, &mut i)?);
            break;
        }
        if line.starts_with("type ") {
            agent.agent_type = line["type".len()..].trim().to_string();
        } else if line.starts_with("role ") {
            agent.role = parse_quoted_rest(line, "role")?;
        } else if line.starts_with("capabilities ") {
            agent.capabilities = parse_quoted_list(line, "capabilities")?;
        } else if line.starts_with("memory_limit ") {
            agent.memory_limit = parse_quoted_rest(line, "memory_limit")?.trim().to_string();
        } else if line.eq_ignore_ascii_case("learning true") {
            agent.learning = true;
        } else if line.eq_ignore_ascii_case("learning false") {
            agent.learning = false;
        } else if line.eq_ignore_ascii_case("communication true") {
            agent.communication = true;
        } else if line.eq_ignore_ascii_case("communication false") {
            agent.communication = false;
        } else if line.eq_ignore_ascii_case("coordination true") {
            agent.coordination = true;
        } else if line.eq_ignore_ascii_case("coordination false") {
            agent.coordination = false;
        } else if line.starts_with("trust_level ") {
            agent.trust_level = line["trust_level".len()..].trim().to_string();
        }
        i += 1;
    }

    if agent.agent_type.is_empty() {
        agent.agent_type = "AI".to_string();
    }

    Ok(MoldConfig {
        name,
        version,
        agent,
        parameters: std::collections::HashMap::new(),
        dependencies: Vec::new(),
        metadata: std::collections::HashMap::new(),
        lifecycle,
    })
}

fn parse_two_tokens(rest: &str) -> Result<(String, String), String> {
    let rest = rest.trim();
    let (first, rest) = next_token(rest)?;
    let (second, _) = next_token(rest)?;
    Ok((first, second))
}

fn next_token(s: &str) -> Result<(String, &str), String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok((String::new(), s));
    }
    if s.starts_with('"') {
        let end = find_string_end(s)?;
        Ok((unescape_string(&s[1..end])?, s[end + 1..].trim()))
    } else {
        let end = s.find(char::is_whitespace).unwrap_or(s.len());
        Ok((s[..end].to_string(), s[end..].trim()))
    }
}

fn find_string_end(s: &str) -> Result<usize, String> {
    let mut i = 1;
    let bytes = s.as_bytes();
    while i < s.len() {
        if bytes[i] == b'\\' && i + 1 < s.len() {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Ok(i);
        }
        i += 1;
    }
    Err("unterminated quoted string".to_string())
}

fn unescape_string(s: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut i = 0;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'n' => out.push('\n'),
                b't' => out.push('\t'),
                b'r' => out.push('\r'),
                b'"' => out.push('"'),
                b'\\' => out.push('\\'),
                _ => out.push(bytes[i + 1] as char),
            }
            i += 2;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    Ok(out)
}

fn parse_quoted_rest(line: &str, key: &str) -> Result<String, String> {
    let rest = line[key.len()..].trim();
    if rest.is_empty() {
        return Ok(String::new());
    }
    if rest.starts_with('"') {
        let end = find_string_end(rest)?;
        unescape_string(&rest[1..end])
    } else {
        Ok(rest.to_string())
    }
}

fn parse_quoted_list(line: &str, key: &str) -> Result<Vec<String>, String> {
    let mut rest = line[key.len()..].trim();
    let mut list = Vec::new();
    while !rest.is_empty() {
        if rest.starts_with('"') {
            let end = find_string_end(rest)?;
            list.push(unescape_string(&rest[1..end])?);
            rest = rest[end + 1..].trim();
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            list.push(rest[..end].to_string());
            rest = rest[end..].trim();
        }
    }
    Ok(list)
}

fn parse_lifecycle(lines: &[&str], i: &mut usize) -> Result<MoldLifecycle, String> {
    let mut on_create = None;
    let mut on_message = None;
    let mut on_evolve = None;
    let mut on_destroy = None;
    while *i < lines.len() {
        let line = lines[*i];
        if line.starts_with("on_create ") {
            on_create = Some(parse_quoted_rest(line, "on_create")?);
        } else if line.starts_with("on_message ") {
            on_message = Some(parse_quoted_rest(line, "on_message")?);
        } else if line.starts_with("on_evolve ") {
            on_evolve = Some(parse_quoted_rest(line, "on_evolve")?);
        } else if line.starts_with("on_destroy ") {
            on_destroy = Some(parse_quoted_rest(line, "on_destroy")?);
        } else if line.starts_with("on_") {
            *i += 1;
            continue;
        } else {
            break;
        }
        *i += 1;
    }
    Ok(MoldLifecycle {
        on_create,
        on_message,
        on_evolve,
        on_destroy,
    })
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
    fn parse_minimal_mold_json() {
        let json = r#"{"name":"Test","version":"1.0","agent":{"type":"AI","capabilities":["read"],"trustLevel":"standard"}}"#;
        let mold = parse_mold_content(json).unwrap();
        assert_eq!(mold.name, "Test");
        assert_eq!(mold.version, "1.0");
        assert_eq!(mold.agent.agent_type, "AI");
        assert_eq!(mold.agent.capabilities, &["read"]);
    }

    #[test]
    fn parse_minimal_mold_dal() {
        let dal = r#"
mold "Test" "1.0"
agent
  type AI
  capabilities "read"
  trust_level standard
"#;
        let mold = parse_mold_content(dal).unwrap();
        assert_eq!(mold.name, "Test");
        assert_eq!(mold.version, "1.0");
        assert_eq!(mold.agent.agent_type, "AI");
        assert_eq!(mold.agent.capabilities, &["read"]);
        assert_eq!(mold.agent.trust_level, "standard");
    }

    #[test]
    fn parse_mold_dal_with_lifecycle() {
        let dal = r#"
mold "Runner" "2.0"
agent
  type Worker
  role "Run tasks"
  memory_limit "256MB"
lifecycle
  on_create "evolve::append_log(agent_id, \"created\")"
  on_evolve "evolve::append_summary(agent_id, evolution_data)"
"#;
        let mold = parse_mold_content(dal).unwrap();
        assert_eq!(mold.name, "Runner");
        assert_eq!(mold.version, "2.0");
        assert_eq!(mold.agent.agent_type, "Worker");
        assert_eq!(mold.agent.role, "Run tasks");
        assert_eq!(mold.agent.memory_limit, "256MB");
        let lc = mold.lifecycle.as_ref().unwrap();
        assert!(lc.on_create.as_deref().unwrap_or("").contains("append_log"));
        assert!(lc
            .on_evolve
            .as_deref()
            .unwrap_or("")
            .contains("append_summary"));
    }
}

//! Evolve: dynamic markdown file for conversation history and action log (Phase 3).
//! Path from DAL_AGENT_CONTEXT_PATH or [agent] context_path in agent.toml / dal.toml; default ./agent_context.md.

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

const DEFAULT_CONTEXT_PATH: &str = "./agent_context.md";
const DEFAULT_AGENT_NAME: &str = "Agent";

fn get_context_path() -> PathBuf {
    if let Ok(p) = env::var("DAL_AGENT_CONTEXT_PATH") {
        return PathBuf::from(p);
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for name in &["agent.toml", "dal.toml"] {
        let path = cwd.join(name);
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(table) = content.parse::<toml::Table>() {
                if let Some(ctx_path) = table
                    .get("agent")
                    .and_then(|v| v.as_table())
                    .and_then(|t| t.get("context_path"))
                    .and_then(|v| v.as_str())
                {
                    let mut buf = PathBuf::from(ctx_path);
                    if !buf.is_absolute() {
                        buf = cwd.join(buf);
                    }
                    return buf;
                }
            }
        }
    }
    cwd.join(DEFAULT_CONTEXT_PATH)
}

fn ensure_header(path: &std::path::Path, agent_name: &str) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let now = chrono::Utc::now().to_rfc3339();
    let header = format!(
        "# Agent context — {}\nUpdated: {}\n\n## Conversation\n\n",
        agent_name, now
    );
    std::fs::write(path, header)
}

fn ensure_action_log_section(path: &std::path::Path) -> std::io::Result<()> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    if content.contains("## Action log") {
        return Ok(());
    }
    let table_header = "\n## Action log\n\n| Time | Action | Detail | Result |\n|------|--------|--------|--------|\n";
    let mut f = OpenOptions::new().append(true).open(path)?;
    f.write_all(table_header.as_bytes())?;
    Ok(())
}

/// Load the current context file content. Creates file with header if missing.
pub fn load(agent_name: Option<&str>) -> Result<String, String> {
    let path = get_context_path();
    let name = agent_name.unwrap_or(DEFAULT_AGENT_NAME);
    ensure_header(&path, name).map_err(|e| e.to_string())?;
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

/// Append a conversation turn (user message + agent response) to the context file.
pub fn append_conversation(
    user_message: &str,
    agent_response: &str,
    agent_name: Option<&str>,
) -> Result<(), String> {
    let path = get_context_path();
    let name = agent_name.unwrap_or(DEFAULT_AGENT_NAME);
    ensure_header(&path, name).map_err(|e| e.to_string())?;

    let now = chrono::Utc::now();
    let ts = now.format("%Y-%m-%dT%H:%M");
    let block = format!(
        "\n### {}\n**User:** {}\n\n**Agent:** {}\n\n",
        ts,
        user_message.replace("\n", "\n  "),
        agent_response.replace("\n", "\n  ")
    );

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    f.write_all(block.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

/// Append an action log entry (e.g. sh::run, command, result).
pub fn append_log(action: &str, detail: &str, result: &str) -> Result<(), String> {
    let path = get_context_path();
    ensure_header(&path, DEFAULT_AGENT_NAME).map_err(|e| e.to_string())?;
    ensure_action_log_section(&path).map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().format("%H:%M:%S");
    let line = format!("| {} | {} | {} | {} |\n", now, action, detail, result);

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    f.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

/// Return the configured context file path (for display or inclusion).
pub fn get_path() -> String {
    get_context_path().display().to_string()
}

/// Load recent context: last `max_lines` lines of the file, or full content if max_lines <= 0.
/// Use for inclusion in agent context without loading the entire history.
pub fn load_recent(agent_name: Option<&str>, max_lines: i64) -> Result<String, String> {
    let full = load(agent_name)?;
    if max_lines <= 0 {
        return Ok(full);
    }
    let lines: Vec<&str> = full.lines().collect();
    let n = max_lines as usize;
    let start = if lines.len() <= n { 0 } else { lines.len() - n };
    Ok(lines[start..].join("\n"))
}

/// Trim the context file to keep only the last `keep_tail_lines` lines of content (after the header).
/// Preserves the "# Agent context" and "## Conversation" header, then keeps the last keep_tail_lines lines.
pub fn trim_retention(keep_tail_lines: i64) -> Result<(), String> {
    let path = get_context_path();
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    if keep_tail_lines <= 0 {
        return Ok(());
    }
    let lines: Vec<&str> = content.lines().collect();
    let header_end = lines
        .iter()
        .position(|&l| l.trim() == "## Conversation")
        .map(|i| i + 2)
        .unwrap_or(0);
    let body_lines = lines.len().saturating_sub(header_end);
    let keep = keep_tail_lines as usize;
    if body_lines <= keep {
        return Ok(());
    }
    let drop = body_lines - keep;
    let new_body_start = header_end + drop;
    let new_content = [
        lines[..header_end].join("\n"),
        lines[new_body_start..].join("\n"),
    ]
    .join("\n");
    std::fs::write(&path, new_content).map_err(|e| e.to_string())
}

/// Append a summary section (e.g. periodic or on-demand session summary).
pub fn append_summary(summary_text: &str, title: Option<&str>) -> Result<(), String> {
    let path = get_context_path();
    ensure_header(&path, DEFAULT_AGENT_NAME).map_err(|e| e.to_string())?;

    let now = chrono::Utc::now();
    let ts = now.format("%Y-%m-%d %H:%M");
    let heading = title
        .map(|t| format!("## Summary — {}\n\n", t))
        .unwrap_or_else(|| "## Summary\n\n".to_string());
    let block = format!("{}\n{}\n\n{}\n\n", heading, ts, summary_text);

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    f.write_all(block.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

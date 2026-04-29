//! Evolve: dynamic markdown file for conversation history and action log (Phase 3).
//! Path from DAL_AGENT_CONTEXT_PATH or \[agent\] context_path in agent.toml / dal.toml; default ./evolve.md.

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

const DEFAULT_CONTEXT_PATH: &str = "./evolve.md";
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
    let table_header = "\n## Action log\n\n| Time | Action | Detail | Result | Get Task (ms) | Do Task (ms) | Total (ms) |\n|------|--------|--------|--------|----------------|--------------|------------|\n";
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

/// Max length of user/agent content written to the conversation log (log injection / DoS).
const MAX_CONVERSATION_LOG_LEN: usize = 32_768;

/// Sanitize content for safe inclusion in the conversation log (prevents log injection).
pub fn sanitize_for_conversation(s: &str) -> String {
    let truncated = if s.len() > MAX_CONVERSATION_LOG_LEN {
        &s[..MAX_CONVERSATION_LOG_LEN]
    } else {
        s
    };
    let with_newlines = truncated.replace("\n", "\n  ");
    // Prevent injection of our own **User:** / **Agent:** structure
    with_newlines
        .replace("**User:**", "[User]:")
        .replace("**Agent:**", "[Agent]:")
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
    let user_safe = sanitize_for_conversation(user_message);
    let agent_safe = sanitize_for_conversation(agent_response);
    let block = format!(
        "\n### {}\n**User:** {}\n\n**Agent:** {}\n\n",
        ts, user_safe, agent_safe
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
    append_log_with_timing(action, detail, result, None, None)
}

/// Append an action log entry with task timing fields.
///
/// `get_task_ms` should represent the time spent choosing/acquiring a task.
/// `do_task_ms` should represent the time spent executing the task.
pub fn append_log_timed(
    action: &str,
    detail: &str,
    result: &str,
    get_task_ms: i64,
    do_task_ms: i64,
) -> Result<(), String> {
    append_log_with_timing(action, detail, result, Some(get_task_ms), Some(do_task_ms))
}

fn append_log_with_timing(
    action: &str,
    detail: &str,
    result: &str,
    get_task_ms: Option<i64>,
    do_task_ms: Option<i64>,
) -> Result<(), String> {
    let path = get_context_path();
    ensure_header(&path, DEFAULT_AGENT_NAME).map_err(|e| e.to_string())?;
    ensure_action_log_section(&path).map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().format("%H:%M:%S");
    let get_ms = get_task_ms
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    let do_ms = do_task_ms
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    let total_ms = match (get_task_ms, do_task_ms) {
        (Some(get), Some(exec)) => (get.saturating_add(exec)).to_string(),
        _ => "-".to_string(),
    };
    let line = format!(
        "| {} | {} | {} | {} | {} | {} | {} |\n",
        now, action, detail, result, get_ms, do_ms, total_ms
    );

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
///
/// **Note:** Line-based cuts often split markdown tables and mix unrelated sections. Prefer
/// [`load_recent_for_prompt`] for LLM prompts.
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

/// Extract body of a level-2 section `## {name}` until the next `## ` heading or EOF.
fn extract_section_body(full: &str, name: &str) -> Option<String> {
    let header = format!("## {}", name);
    let start = full.find(&header)?;
    let rest = full.get(start + header.len()..)?;
    let rest = rest.trim_start_matches(['\n', '\r']);
    let end = rest
        .find("\n## ")
        .or_else(|| rest.find("\r\n## "))
        .unwrap_or(rest.len());
    let body = rest.get(..end)?.trim();
    if body.is_empty() {
        None
    } else {
        Some(body.to_string())
    }
}

/// Split `## Conversation` body into turns (blocks starting with a `### ` heading line).
fn split_conversation_turns(body: &str) -> Vec<String> {
    let lines: Vec<&str> = body.lines().collect();
    let heading_idx: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| {
            let t = l.trim_start();
            t.starts_with("### ") && !t.starts_with("####")
        })
        .map(|(i, _)| i)
        .collect();
    if heading_idx.is_empty() {
        let t = body.trim();
        return if t.is_empty() {
            Vec::new()
        } else {
            vec![t.to_string()]
        };
    }
    let mut out = Vec::new();
    for w in heading_idx.windows(2) {
        out.push(lines[w[0]..w[1]].join("\n"));
    }
    out.push(lines[*heading_idx.last().unwrap()..].join("\n"));
    out
}

/// Best-effort extract of the `### <timestamp>` marker from a conversation turn.
fn turn_marker(turn: &str) -> Option<String> {
    turn.lines()
        .find_map(|line| line.trim_start().strip_prefix("### "))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Detect if conversation turns appear in reverse-chronological order (newest first).
fn is_newest_first_turn_order(turns: &[String]) -> bool {
    if turns.len() < 2 {
        return false;
    }
    let first = match turn_marker(&turns[0]) {
        Some(v) => v,
        None => return false,
    };
    let last = match turn_marker(&turns[turns.len() - 1]) {
        Some(v) => v,
        None => return false,
    };
    // Turn markers are ISO-like in this file; lexical compare is sufficient.
    first > last
}

/// Best-effort: include the most recently appended `## Summary` / `## Summary — …` block.
fn extract_last_summary_block(full: &str, max_chars: usize) -> Option<String> {
    let lines: Vec<&str> = full.lines().collect();
    let mut last_start_line: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("## Summary") {
            last_start_line = Some(i);
        }
    }
    let sline = last_start_line?;
    let mut end_line = lines.len();
    for j in (sline + 1)..lines.len() {
        let t = lines[j].trim();
        if t.starts_with("## ") && !t.starts_with("###") {
            end_line = j;
            break;
        }
    }
    let block = lines[sline..end_line].join("\n");
    let block = block.trim();
    if block.is_empty() {
        return None;
    }
    if block.len() > max_chars {
        let mut cut = max_chars;
        while cut > 0 && !block.is_char_boundary(cut) {
            cut -= 1;
        }
        Some(format!(
            "{}…\n_(summary truncated for prompt)_",
            &block[..cut]
        ))
    } else {
        Some(block.to_string())
    }
}

/// Prompt-oriented evolve load: **working memory** bullets, last **N conversation turns**, optional
/// **summary** tail — **excludes** the action-log table by default (noisy for reasoning).
///
/// `max_conversation_turns`: keep only the last this many `###` turns from `## Conversation`.
/// `max_chars`: hard cap on returned string (keeps the tail / most recent content).
pub fn load_recent_for_prompt(
    agent_name: Option<&str>,
    max_conversation_turns: usize,
    max_chars: usize,
) -> Result<String, String> {
    let full = load(agent_name)?;
    let max_turns = max_conversation_turns.max(1);
    let max_chars = max_chars.max(512);

    let wm = extract_section_body(&full, "Working memory");
    let conv_body = extract_section_body(&full, "Conversation").unwrap_or_default();
    let mut turns = split_conversation_turns(&conv_body);
    let newest_first = is_newest_first_turn_order(&turns);
    if turns.len() > max_turns {
        if newest_first {
            turns.truncate(max_turns);
        } else {
            turns = turns.split_off(turns.len() - max_turns);
        }
    }
    if newest_first {
        // Keep output chronological for readability in prompts.
        turns.reverse();
    }
    let conv_out = turns.join("\n\n");
    let summary = extract_last_summary_block(&full, 2400);

    let mut parts: Vec<String> = Vec::new();
    parts.push(
        "_Evolve: structured view (working memory + recent turns + optional summary). \
         Full `evolve.md` may still contain action logs on disk._\n\n"
            .to_string(),
    );
    if let Some(ref w) = wm {
        let w = w.trim();
        if !w.is_empty() {
            parts.push("## Working memory\n\n".to_string());
            parts.push(format!("{}\n\n", w));
            parts.push("---\n\n".to_string());
        }
    }
    if !conv_out.is_empty() {
        parts.push("## Recent conversation\n\n".to_string());
        parts.push(conv_out);
        parts.push("\n\n---\n\n".to_string());
    }
    if let Some(s) = summary {
        parts.push("## Latest summary (from file)\n\n".to_string());
        parts.push(s);
        parts.push("\n\n".to_string());
    }
    let mut out = parts.join("");
    if out.len() > max_chars {
        let skip = out.len() - max_chars;
        let mut start = skip;
        while start < out.len() && !out.is_char_boundary(start) {
            start += 1;
        }
        out = format!(
            "… _(truncated {} chars from older content)_\n\n{}",
            skip,
            &out[start..]
        );
    }
    Ok(out)
}

/// Append one bullet under `## Working memory` (created before `## Conversation` if missing).
/// Use for **decisions, paths, open threads** — short lines (sanitized, capped) that actually evolve context.
pub fn append_working_memory_line(line: &str) -> Result<(), String> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(());
    }
    let mut safe = sanitize_for_conversation(line);
    if safe.len() > 512 {
        safe.truncate(512);
        safe.push_str("…");
    }
    let safe = safe.replace('\n', " ");
    let bullet = format!("- {}\n", safe);

    let path = get_context_path();
    ensure_header(&path, DEFAULT_AGENT_NAME).map_err(|e| e.to_string())?;
    let mut content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    const WM: &str = "## Working memory";
    if content.contains(WM) {
        if let Some(i) = content.find(WM) {
            let after = i + WM.len();
            let tail = &content[after..];
            let end_rel = tail
                .find("\n## ")
                .or_else(|| tail.find("\r\n## "))
                .unwrap_or(tail.len());
            content.insert_str(after + end_rel, &bullet);
        }
    } else if let Some(i) = content.find("## Conversation") {
        let block = format!("{}\n\n{}", WM, bullet);
        content.insert_str(i, &block);
    } else {
        content.push_str(&format!("\n{}\n\n{}", WM, bullet));
    }

    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_ctx_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn split_conversation_turns_three_blocks() {
        let body = r###"
### 2026-01-01T10:00
**User:** hi

**Agent:** hello

### 2026-01-01T10:01
**User:** bye

**Agent:** ok
"###;
        let t = split_conversation_turns(body);
        assert_eq!(t.len(), 2, "{:?}", t);
        assert!(t[0].contains("hi"));
        assert!(t[1].contains("bye"));
    }

    #[test]
    fn extract_section_body_finds_conversation() {
        let full = r#"# Agent context — X
## Working memory

- note a

## Conversation

### 2026-01-01T10:00
**User:** u

**Agent:** a

## Action log

| x |
"#;
        let c = extract_section_body(full, "Conversation").expect("conv");
        assert!(c.contains("**User:** u"));
        let wm = extract_section_body(full, "Working memory").expect("wm");
        assert!(wm.contains("note a"));
    }

    #[test]
    fn load_recent_for_prompt_skips_action_log() {
        use std::env;
        use std::fs;
        let _g = env_ctx_lock().lock().unwrap();

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("evolve_test.md");
        let prev_ctx = env::var("DAL_AGENT_CONTEXT_PATH").ok();
        env::set_var("DAL_AGENT_CONTEXT_PATH", path.as_os_str());
        let md = r#"# Agent context — Agent
Updated: test

## Conversation

### 2026-01-01T10:00
**User:** scaffold where?

**Agent:** pick 1/2/3

## Action log

| t | a | b | c | d | e | f |
| x | y | z | noisy | table | here | ... |

"#;
        fs::write(&path, md).unwrap();
        let out = load_recent_for_prompt(None, 5, 20_000).expect("ok");
        assert!(out.contains("scaffold"));
        assert!(out.contains("pick 1/2/3"));
        assert!(!out.contains("noisy"), "{}", out);
        match prev_ctx {
            Some(p) => env::set_var("DAL_AGENT_CONTEXT_PATH", p),
            None => env::remove_var("DAL_AGENT_CONTEXT_PATH"),
        }
    }

    #[test]
    fn load_recent_for_prompt_supports_newest_first_turn_order() {
        use std::env;
        use std::fs;
        let _g = env_ctx_lock().lock().unwrap();

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("evolve_test_newest_first.md");
        let prev_ctx = env::var("DAL_AGENT_CONTEXT_PATH").ok();
        env::set_var("DAL_AGENT_CONTEXT_PATH", path.as_os_str());
        let md = r#"# Agent context — Agent
Updated: test

## Conversation

### 2026-01-01T10:03
**User:** newest

**Agent:** newest-reply

### 2026-01-01T10:02
**User:** middle

**Agent:** middle-reply

### 2026-01-01T10:01
**User:** oldest

**Agent:** oldest-reply
"#;
        fs::write(&path, md).unwrap();
        let out = load_recent_for_prompt(None, 2, 20_000).expect("ok");
        // Should keep the newest two turns, but render them chronologically.
        assert!(out.contains("middle"), "{}", out);
        assert!(out.contains("newest"), "{}", out);
        assert!(!out.contains("oldest"), "{}", out);
        let mid_pos = out.find("middle").unwrap();
        let new_pos = out.find("newest").unwrap();
        assert!(mid_pos < new_pos, "expected chronological output: {}", out);
        match prev_ctx {
            Some(p) => env::set_var("DAL_AGENT_CONTEXT_PATH", p),
            None => env::remove_var("DAL_AGENT_CONTEXT_PATH"),
        }
    }
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

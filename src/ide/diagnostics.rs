//! DAL diagnostics, completion, and hover for IDE HTTP API.
//! Full LSP (tower-lsp) uses similar logic when lsp feature is enabled.

use crate::lexer::Lexer;
use crate::parser::ast::Statement;
use crate::parser::Parser;
use serde::{Deserialize, Serialize};

/// A single diagnostic (error or warning).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub message: String,
    pub severity: String, // "error" | "warning"
}

/// Run lexer and parser on source, return diagnostics.
pub fn diagnostics_from_source(source: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let lexer = Lexer::new(source);
    match lexer.tokenize_with_positions_immutable() {
        Err(e) => {
            let (line, col) = e.line_column().unwrap_or((1, 1));
            diags.push(Diagnostic {
                line: line as u32,
                column: col as u32,
                end_line: line as u32,
                end_column: (col + 1) as u32,
                message: e.to_string(),
                severity: "error".to_string(),
            });
            return diags;
        }
        Ok(tokens_with_pos) => {
            let mut parser = Parser::new_with_positions(tokens_with_pos);
            if let Err(e) = parser.parse() {
                let line = e.line_number().unwrap_or(1);
                let col = e.column_number().unwrap_or(1);
                diags.push(Diagnostic {
                    line: line as u32,
                    column: col as u32,
                    end_line: line as u32,
                    end_column: (col + 1) as u32,
                    message: e.to_string(),
                    severity: "error".to_string(),
                });
            }
        }
    }

    diags
}

/// Get word at 0-based line and character.
fn word_at_position(source: &str, line_0: u32, char_0: u32) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let line_idx = line_0 as usize;
    let line = lines.get(line_idx)?;
    let chars: Vec<char> = line.chars().collect();
    let char_idx = char_0 as usize;
    if char_idx > chars.len() {
        return None;
    }
    fn is_word_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
    let start = chars[..char_idx]
        .iter()
        .rposition(|&c| is_word_char(c))
        .map(|i| i + 1)
        .unwrap_or(0);
    let end = chars[char_idx..]
        .iter()
        .position(|&c| !is_word_char(c))
        .map(|i| char_idx + i)
        .unwrap_or(chars.len());
    if start >= end {
        return None;
    }
    Some(chars[start..end].iter().collect())
}

fn keyword_doc(word: &str) -> Option<&'static str> {
    Some(match word {
        "fn" => "Function declaration. `fn name(params) -> return_type { ... }`",
        "let" => "Bind a value to a variable.",
        "return" => "Return a value from a function.",
        "if" | "else" => "Conditional branch.",
        "while" | "for" | "loop" => "Loop construct.",
        "service" => "Service declaration. `service Name { fields; fn methods() {} }`",
        "struct" => "Struct type definition.",
        "import" => "Import a module. `import \"path\";` or `import \"path\" as alias;`",
        "match" | "case" | "default" => "Pattern matching.",
        "try" | "catch" | "throw" => "Error handling.",
        "spawn" | "agent" => "Agent / concurrent execution.",
        "true" | "false" => "Boolean literal.",
        _ => return None,
    })
}

fn stdlib_doc(module: &str) -> Option<&'static str> {
    Some(match module {
        "chain" => "Blockchain operations: deploy, call, balance, gas, etc.",
        "ai" | "assist" => "AI/LLM: generate_text, spawn_agent, send_message, etc.",
        "log" => "Logging: info, error, warning, audit.",
        "auth" => "Authentication and authorization.",
        "config" => "Configuration and environment.",
        "db" => "Database operations.",
        "crypto" => "Cryptography: hash, sign, verify, keygen.",
        "oracle" => "Oracle and external data.",
        "agent" => "Agent coordination and communication.",
        _ => return None,
    })
}

fn collect_symbols_from_source(source: &str) -> Vec<(String, String)> {
    let mut symbols = Vec::new();
    let lexer = Lexer::new(source);
    let tokens_with_pos = match lexer.tokenize_with_positions_immutable() {
        Ok(t) => t,
        Err(_) => return symbols,
    };
    let mut parser = Parser::new_with_positions(tokens_with_pos);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(_) => return symbols,
    };
    for stmt in &program.statements {
        match stmt {
            Statement::Service(s) => {
                symbols.push((s.name.clone(), format!("service {}", s.name)));
                for f in &s.fields {
                    symbols.push((
                        f.name.clone(),
                        format!("field {}: {}", f.name, f.field_type),
                    ));
                }
                for m in &s.methods {
                    let params: Vec<String> = m.parameters.iter().map(|p| p.name.clone()).collect();
                    let sig = format!("fn {}({})", m.name, params.join(", "));
                    symbols.push((m.name.clone(), sig));
                }
            }
            Statement::Function(f) => {
                let params: Vec<String> = f.parameters.iter().map(|p| p.name.clone()).collect();
                let sig = format!("fn {}({})", f.name, params.join(", "));
                symbols.push((f.name.clone(), sig));
            }
            _ => {}
        }
    }
    symbols
}

/// Hover content for a word at position.
pub fn hover_at_position(source: &str, line_0: u32, char_0: u32) -> Option<String> {
    let word = word_at_position(source, line_0, char_0)?;
    let word = word.trim();
    if word.is_empty() {
        return None;
    }
    if let Some(doc) = keyword_doc(word) {
        return Some(format!("**{}**\n\n{}", word, doc));
    }
    if let Some(doc) = stdlib_doc(word) {
        return Some(format!("**{}** (stdlib)\n\n{}", word, doc));
    }
    for (name, detail) in collect_symbols_from_source(source) {
        if name == word {
            return Some(format!("**{}**\n\n`{}`", name, detail));
        }
    }
    None
}

/// Completion item for HTTP API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: String, // "keyword" | "module" | "function"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Completion at position (prefix from source).
pub fn completion_at_position(source: &str, line_0: u32, char_0: u32) -> Vec<CompletionItem> {
    let prefix = word_at_position(source, line_0, char_0)
        .unwrap_or_default()
        .trim()
        .to_string();

    let mut items = Vec::new();

    let keywords = [
        "fn", "let", "return", "if", "else", "while", "for", "in", "loop", "break", "continue",
        "service", "struct", "import", "match", "case", "default", "try", "catch", "throw",
        "spawn", "agent", "true", "false", "impl", "pub", "self", "Ok", "Err", "Some", "None",
    ];
    for kw in keywords {
        if prefix.is_empty() || kw.starts_with(&prefix) {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: "keyword".to_string(),
                detail: keyword_doc(kw).map(String::from),
            });
        }
    }

    let modules = [
        "chain", "ai", "assist", "log", "auth", "config", "db", "crypto", "oracle", "agent",
    ];
    for m in modules {
        if prefix.is_empty() || m.starts_with(&prefix) {
            items.push(CompletionItem {
                label: m.to_string(),
                kind: "module".to_string(),
                detail: stdlib_doc(m).map(String::from),
            });
        }
    }

    for (name, detail) in collect_symbols_from_source(source) {
        if prefix.is_empty() || name.starts_with(&prefix) {
            items.push(CompletionItem {
                label: name,
                kind: "function".to_string(),
                detail: Some(detail),
            });
        }
    }

    items
}

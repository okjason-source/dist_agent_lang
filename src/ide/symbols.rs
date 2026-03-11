//! DAL document symbols and references for IDE (Functions & Invocations view).
//! Used by POST /api/lsp/document_symbols and POST /api/lsp/references.

use crate::lexer::Lexer;
use crate::parser::ast::Statement;
use crate::parser::Parser;
use serde::{Deserialize, Serialize};

/// One symbol (function, service, or method) with location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: String, // "function" | "service" | "method"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub line: u32,
    pub column: u32,
}

/// A reference (call site): line and column where the symbol is called.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceLocation {
    pub line: u32,
    pub column: u32,
}

/// Parse source and return document symbols with locations (from AST + statement_spans).
pub fn document_symbols_from_source(source: &str) -> Vec<DocumentSymbol> {
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
    for (i, stmt) in program.statements.iter().enumerate() {
        let span = program
            .statement_spans
            .get(i)
            .and_then(|s| *s)
            .unwrap_or(crate::parser::ast::Span { line: 1, column: 1 });
        let line = span.line as u32;
        let col = span.column as u32;
        match stmt {
            Statement::Function(f) => {
                let sig = format!(
                    "fn {}({})",
                    f.name,
                    f.parameters
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                symbols.push(DocumentSymbol {
                    name: f.name.clone(),
                    kind: "function".to_string(),
                    signature: Some(sig),
                    line,
                    column: col + 4, // after "fn "
                });
            }
            Statement::Service(s) => {
                symbols.push(DocumentSymbol {
                    name: s.name.clone(),
                    kind: "service".to_string(),
                    signature: Some(format!("service {}", s.name)),
                    line,
                    column: col + 8, // after "service "
                });
                for m in &s.methods {
                    let sig = format!(
                        "fn {}({})",
                        m.name,
                        m.parameters
                            .iter()
                            .map(|p| p.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    symbols.push(DocumentSymbol {
                        name: m.name.clone(),
                        kind: "method".to_string(),
                        signature: Some(sig),
                        line,
                        column: col + 8,
                    });
                }
            }
            _ => {}
        }
    }
    symbols
}

/// Find call sites of `name` in source (text-based: identifier followed by `(`).
pub fn references_in_source(source: &str, name: &str) -> Vec<ReferenceLocation> {
    let mut refs = Vec::new();
    if name.is_empty() {
        return refs;
    }
    for (line_0, line) in source.lines().enumerate() {
        let mut search_start = 0;
        while let Some(idx) = line[search_start..].find(name) {
            let pos = search_start + idx;
            let after = line[pos + name.len()..].trim_start();
            let word_bound_before = pos == 0
                || !line[pos - 1..pos]
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_alphanumeric() || c == '_');
            let word_bound_after = after.is_empty()
                || !after
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_alphanumeric() || c == '_');
            if word_bound_before && word_bound_after && after.starts_with('(') {
                let column_0 = line[..pos].chars().count();
                refs.push(ReferenceLocation {
                    line: (line_0 + 1) as u32,
                    column: (column_0 + 1) as u32,
                });
            }
            search_start = pos + 1;
        }
    }
    refs
}

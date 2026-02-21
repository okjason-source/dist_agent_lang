//! Central error formatting for production-grade diagnostics.
//!
//! Used by CLI and other entry points so lexer, parser, and runtime errors
//! are shown with source snippet, caret, and suggestions in a consistent way.
//! See [ERROR_HANDLING_AND_WARNINGS_PLAN.md](../docs/development/ERROR_HANDLING_AND_WARNINGS_PLAN.md).

use crate::lexer::LexerError;
use crate::parser::error::{ErrorContext, ParserError};
use crate::parser::ParseWarning;
use crate::runtime::functions::RuntimeErrorWithContext;

/// Format a lexer error for display: message, optional file path, and source line with caret.
pub fn format_lexer_error(e: &LexerError, file_path: Option<&str>, source: Option<&str>) -> String {
    let mut out = String::new();
    if let Some(path) = file_path {
        if !path.is_empty() {
            out.push_str(&format!("  --> {}\n", path));
        }
    }
    out.push_str(&format!("{}\n", e));
    if let (Some(src), Some((line, col))) = (source, e.line_column()) {
        let lines: Vec<&str> = src.lines().collect();
        if line > 0 && line <= lines.len() {
            let line_content = lines[line - 1];
            out.push_str(&format!("  --> Line {}: {}\n", line, line_content));
            let pad = " ".repeat(if col >= 1 { col - 1 } else { 0 });
            out.push_str(&format!("      {}^\n", pad));
        }
    }
    out
}

/// Attach file path, source, and generated suggestions to a parser error, then format with source.
/// Use whenever you have the source and path (e.g. CLI run_dal_file).
pub fn format_parser_error(
    e: &ParserError,
    file_path: Option<&str>,
    source: Option<&str>,
) -> String {
    let suggestions = e.generate_suggestions();
    let mut ctx = ErrorContext::new().with_suggestions(suggestions);
    if let Some(p) = file_path {
        if !p.is_empty() {
            ctx = ctx.with_file_path(p.to_string());
        }
    }
    if let Some(s) = source {
        if !s.is_empty() {
            ctx = ctx.with_source_code(s.to_string());
        }
    }
    let e = e.clone().with_context(ctx);
    e.format_with_source()
}

/// Format a runtime error for display (message, location, call stack).
pub fn format_runtime_error(
    e: &RuntimeErrorWithContext,
    file_path: Option<&str>,
    _source: Option<&str>,
) -> String {
    e.format_display(_source, file_path)
}

/// Format parse warnings for display (e.g. unused variable).
pub fn format_parse_warnings(
    warnings: &[ParseWarning],
    file_path: Option<&str>,
    source: Option<&str>,
) -> String {
    if warnings.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    if let Some(path) = file_path {
        if !path.is_empty() {
            out.push_str(&format!("  --> {}\n", path));
        }
    }
    out.push_str(&format!("⚠️  Warnings ({}):\n", warnings.len()));
    for w in warnings {
        if w.line > 0 {
            out.push_str(&format!("  --> Line {}: {}\n", w.line, w.message));
            if let Some(src) = source {
                let lines: Vec<&str> = src.lines().collect();
                if w.line <= lines.len() {
                    out.push_str(&format!("      {}\n", lines[w.line - 1].trim_end()));
                }
            }
        } else {
            out.push_str(&format!("  --> {}\n", w.message));
        }
    }
    out
}

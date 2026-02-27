// Test that parser, lexer, runtime, and warning formatting include file path / structure.
// These tests catch mutations in reporting.rs (format_lexer_error, format_parser_error,
// format_runtime_error, format_parse_warnings).
use dist_agent_lang::lexer::tokens::{Punctuation, Token};
use dist_agent_lang::lexer::LexerError;
use dist_agent_lang::parser::error::ParserError;
use dist_agent_lang::parser::ParseWarning;
use dist_agent_lang::reporting::{
    format_lexer_error, format_parse_warnings, format_parser_error, format_runtime_error,
};
use dist_agent_lang::runtime::{
    CallFrameInfo, RuntimeError, RuntimeErrorWithContext, SourceLocation,
};

#[test]
fn test_format_parser_error_includes_file_path_and_source_line() {
    let err = ParserError::unexpected_token(
        &Token::Punctuation(Punctuation::Comma),
        &["Punctuation(Semicolon)"],
        11,
        11,
    );
    // Source must have at least 11 lines so line 11 is valid
    let mut source = String::new();
    for i in 1..=11 {
        if i == 11 {
            source.push_str("    users: map<string, any>,\n");
        } else {
            source.push_str(&format!("    line {};\n", i));
        }
    }
    let out = format_parser_error(&err, Some("examples/test.dal"), Some(&source));

    assert!(
        out.contains("  --> "),
        "output should include file path or source line marker; got:\n{}",
        out
    );
    assert!(
        out.contains("examples/test.dal"),
        "output should include file path; got:\n{}",
        out
    );
    assert!(
        out.contains("Line 11:"),
        "output should include source line; got:\n{}",
        out
    );
    assert!(
        out.contains("map<string, any>"),
        "output should include offending source line; got:\n{}",
        out
    );
    assert!(
        out.contains("^"),
        "output should include caret; got:\n{}",
        out
    );
}

#[test]
fn test_format_lexer_error_includes_file_path_line_and_caret() {
    let err = LexerError::UnexpectedCharacter('@', 2, 5);
    let source = "line one\nline two with @ here\nline three";
    let out = format_lexer_error(&err, Some("test.dal"), Some(source));

    assert!(
        out.contains("  --> "),
        "output should include marker; got:\n{}",
        out
    );
    assert!(
        out.contains("test.dal"),
        "output should include file path; got:\n{}",
        out
    );
    assert!(
        out.contains("Line 2:") || out.contains("line two"),
        "output should include line number or line content; got:\n{}",
        out
    );
    assert!(
        out.contains("^"),
        "output should include caret; got:\n{}",
        out
    );
    assert!(
        out.contains("@"),
        "output should include the unexpected character; got:\n{}",
        out
    );
}

#[test]
fn test_format_runtime_error_includes_message_and_location() {
    let err = RuntimeErrorWithContext::new(
        RuntimeError::VariableNotFound("x".to_string()),
        Some(SourceLocation {
            line: 5,
            column: 2,
            file_path: Some("script.dal".to_string()),
        }),
        vec![CallFrameInfo {
            function_name: "main".to_string(),
            line: Some(5),
        }],
    );
    let out = format_runtime_error(&err, Some("script.dal"), None);

    assert!(
        out.contains("Variable") && out.contains("not found"),
        "output should include error message; got:\n{}",
        out
    );
    assert!(
        out.contains("at ") && (out.contains("5") || out.contains("script.dal")),
        "output should include location; got:\n{}",
        out
    );
    assert!(
        out.contains("Call stack:") && out.contains("main"),
        "output should include call stack; got:\n{}",
        out
    );
}

#[test]
fn test_format_parse_warnings_includes_file_path_and_warning_content() {
    let warnings = vec![
        ParseWarning {
            message: "unused variable 'y'".to_string(),
            line: 3,
        },
        ParseWarning {
            message: "unused variable 'z'".to_string(),
            line: 7,
        },
    ];
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    let out = format_parse_warnings(&warnings, Some("app.dal"), Some(source));

    assert!(
        out.contains("  --> "),
        "output should include marker; got:\n{}",
        out
    );
    assert!(
        out.contains("app.dal"),
        "output should include file path; got:\n{}",
        out
    );
    assert!(
        out.contains("Warnings") && out.contains("2"),
        "output should include warning count; got:\n{}",
        out
    );
    assert!(
        out.contains("unused variable") && out.contains("y") && out.contains("z"),
        "output should include warning messages; got:\n{}",
        out
    );
    assert!(
        out.contains("Line 3") || out.contains("line 3") || out.contains("3"),
        "output should include line number; got:\n{}",
        out
    );
}

#[test]
fn test_format_parse_warnings_empty_returns_empty_string() {
    let out = format_parse_warnings(&[], Some("x.dal"), Some("let a = 1;"));
    assert!(
        out.is_empty(),
        "empty warnings should produce empty string; got: {:?}",
        out
    );
}

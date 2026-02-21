// Test that parser error formatting includes file path, source line, and caret.
use dist_agent_lang::lexer::tokens::{Punctuation, Token};
use dist_agent_lang::parser::error::ParserError;
use dist_agent_lang::reporting::format_parser_error;

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

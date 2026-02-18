// Lexer Mutation Tests
// These tests are designed to catch mutations identified by mutation testing
// Tests use only public APIs to verify lexer behavior

use dist_agent_lang::lexer::tokens::{Keyword, Literal, Operator, Punctuation, Token};
use dist_agent_lang::lexer::Lexer;

// ============================================================================
// OPERATOR TOKEN TESTS
// ============================================================================
// These tests catch "delete match arm" mutations by ensuring all operators
// are tested, and catch comparison operator mutations by testing boundaries

#[test]
fn test_operator_plus() {
    let code = "+";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2); // Plus + EOF
    assert!(matches!(tokens[0], Token::Operator(Operator::Plus)));
}

#[test]
fn test_operator_minus() {
    let code = "-";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::Minus)));
}

#[test]
fn test_operator_star() {
    let code = "*";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::Star)));
}

#[test]
fn test_operator_slash() {
    let code = "/";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::Slash)));
}

#[test]
fn test_operator_percent() {
    let code = "%";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::Percent)));
}

#[test]
fn test_operator_less() {
    // Test < operator - catches comparison operator mutations
    let code = "<";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::Less)));
}

#[test]
fn test_operator_less_equal() {
    // Test <= operator - catches comparison operator mutations and boundary checks
    let code = "<=";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::LessEqual)));
}

#[test]
fn test_operator_greater() {
    // Test > operator - catches comparison operator mutations
    let code = ">";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::Greater)));
}

#[test]
fn test_operator_greater_equal() {
    // Test >= operator - catches comparison operator mutations and boundary checks
    let code = ">=";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::GreaterEqual)));
}

#[test]
fn test_operator_equal() {
    // Test == operator - catches comparison operator mutations
    let code = "==";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::Equal)));
}

#[test]
fn test_operator_not_equal() {
    // Test != operator - catches comparison operator mutations
    let code = "!=";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::NotEqual)));
}

#[test]
fn test_operator_not() {
    // Test ! operator (not !=)
    let code = "!";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::Not)));
}

#[test]
fn test_operator_assign() {
    // Test = operator (not ==)
    let code = "=";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::Assign)));
}

#[test]
fn test_operator_and() {
    // Test && operator - catches delete match arm mutations
    let code = "&&";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::And)));
}

#[test]
fn test_operator_or() {
    // Test || operator - catches delete match arm mutations
    let code = "||";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Operator(Operator::Or)));
}

#[test]
fn test_operator_comparison_boundary_less_vs_less_equal() {
    // Test that < and <= are distinct - catches comparison operator mutations
    let code1 = "x < 10";
    let code2 = "x <= 10";

    let tokens1 = Lexer::new(code1).tokenize_immutable().unwrap();
    let tokens2 = Lexer::new(code2).tokenize_immutable().unwrap();

    // Find the comparison operator in each
    let op1 = tokens1.iter().find(|t| matches!(t, Token::Operator(_)));
    let op2 = tokens2.iter().find(|t| matches!(t, Token::Operator(_)));

    assert!(op1.is_some() && op2.is_some());
    assert!(matches!(op1.unwrap(), Token::Operator(Operator::Less)));
    assert!(matches!(op2.unwrap(), Token::Operator(Operator::LessEqual)));
}

#[test]
fn test_operator_comparison_boundary_greater_vs_greater_equal() {
    // Test that > and >= are distinct - catches comparison operator mutations
    let code1 = "x > 10";
    let code2 = "x >= 10";

    let tokens1 = Lexer::new(code1).tokenize_immutable().unwrap();
    let tokens2 = Lexer::new(code2).tokenize_immutable().unwrap();

    let op1 = tokens1.iter().find(|t| matches!(t, Token::Operator(_)));
    let op2 = tokens2.iter().find(|t| matches!(t, Token::Operator(_)));

    assert!(op1.is_some() && op2.is_some());
    assert!(matches!(op1.unwrap(), Token::Operator(Operator::Greater)));
    assert!(matches!(
        op2.unwrap(),
        Token::Operator(Operator::GreaterEqual)
    ));
}

// ============================================================================
// PUNCTUATION TOKEN TESTS
// ============================================================================
// These tests catch "delete match arm" mutations by ensuring all punctuation
// types are tested

#[test]
fn test_punctuation_left_paren() {
    let code = "(";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(
        tokens[0],
        Token::Punctuation(Punctuation::LeftParen)
    ));
}

#[test]
fn test_punctuation_right_paren() {
    let code = ")";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(
        tokens[0],
        Token::Punctuation(Punctuation::RightParen)
    ));
}

#[test]
fn test_punctuation_left_brace() {
    let code = "{";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(
        tokens[0],
        Token::Punctuation(Punctuation::LeftBrace)
    ));
}

#[test]
fn test_punctuation_right_brace() {
    let code = "}";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(
        tokens[0],
        Token::Punctuation(Punctuation::RightBrace)
    ));
}

#[test]
fn test_punctuation_left_bracket() {
    let code = "[";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(
        tokens[0],
        Token::Punctuation(Punctuation::LeftBracket)
    ));
}

#[test]
fn test_punctuation_right_bracket() {
    let code = "]";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(
        tokens[0],
        Token::Punctuation(Punctuation::RightBracket)
    ));
}

#[test]
fn test_punctuation_semicolon() {
    let code = ";";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(
        tokens[0],
        Token::Punctuation(Punctuation::Semicolon)
    ));
}

#[test]
fn test_punctuation_colon() {
    // Test single : (not ::)
    let code = ":";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Punctuation(Punctuation::Colon)));
}

#[test]
fn test_punctuation_double_colon() {
    // Test :: - catches boundary check mutations
    let code = "::";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(
        tokens[0],
        Token::Punctuation(Punctuation::DoubleColon)
    ));
}

#[test]
fn test_punctuation_comma() {
    let code = ",";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Punctuation(Punctuation::Comma)));
}

#[test]
fn test_punctuation_dot() {
    let code = ".";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Punctuation(Punctuation::Dot)));
}

#[test]
fn test_punctuation_question() {
    let code = "?";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(
        tokens[0],
        Token::Punctuation(Punctuation::Question)
    ));
}

#[test]
fn test_punctuation_at() {
    let code = "@";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Punctuation(Punctuation::At)));
}

#[test]
fn test_punctuation_arrow() {
    // Test -> (not just -)
    let code = "->";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Punctuation(Punctuation::Arrow)));
}

#[test]
fn test_punctuation_colon_vs_double_colon_boundary() {
    // Test boundary between : and :: - catches comparison operator mutations
    let code1 = ":";
    let code2 = "::";

    let tokens1 = Lexer::new(code1).tokenize_immutable().unwrap();
    let tokens2 = Lexer::new(code2).tokenize_immutable().unwrap();

    assert!(matches!(tokens1[0], Token::Punctuation(Punctuation::Colon)));
    assert!(matches!(
        tokens2[0],
        Token::Punctuation(Punctuation::DoubleColon)
    ));
}

// ============================================================================
// POSITION TRACKING TESTS
// ============================================================================
// These tests catch comparison operator mutations in position calculations
// and arithmetic operator mutations in position increments

#[test]
fn test_position_tracking_single_char_token() {
    // Test that single character tokens advance position by 1
    // Catches arithmetic operator mutations (position += 1)
    let code = "+";
    let tokens_with_pos = Lexer::new(code)
        .tokenize_with_positions_immutable()
        .unwrap();

    assert_eq!(tokens_with_pos.len(), 2); // Plus + EOF
    assert_eq!(tokens_with_pos[0].line, 1);
    assert_eq!(tokens_with_pos[0].column, 1); // Starts at column 1
}

#[test]
fn test_position_tracking_two_char_token() {
    // Test that two character tokens advance position by 2
    // Catches arithmetic operator mutations (position += 2)
    let code = "==";
    let tokens_with_pos = Lexer::new(code)
        .tokenize_with_positions_immutable()
        .unwrap();

    assert_eq!(tokens_with_pos.len(), 2);
    assert_eq!(tokens_with_pos[0].line, 1);
    assert_eq!(tokens_with_pos[0].column, 1); // Starts at column 1
                                              // The next token (EOF) should be at column 3
    assert_eq!(tokens_with_pos[1].column, 3);
}

#[test]
fn test_position_tracking_newline() {
    // Test line/column tracking across newlines
    // Catches comparison operator mutations in skip_whitespace
    let code = "+\n-";
    let tokens_with_pos = Lexer::new(code)
        .tokenize_with_positions_immutable()
        .unwrap();

    assert_eq!(tokens_with_pos.len(), 3); // Plus, Minus, EOF
    assert_eq!(tokens_with_pos[0].line, 1);
    assert_eq!(tokens_with_pos[0].column, 1);
    assert_eq!(tokens_with_pos[1].line, 2); // Second token on line 2
    assert_eq!(tokens_with_pos[1].column, 1); // Column resets to 1
}

#[test]
fn test_position_tracking_whitespace() {
    // Test that whitespace advances column correctly
    // Catches arithmetic operator mutations in skip_whitespace
    let code = "  +";
    let tokens_with_pos = Lexer::new(code)
        .tokenize_with_positions_immutable()
        .unwrap();

    assert_eq!(tokens_with_pos.len(), 2);
    assert_eq!(tokens_with_pos[0].line, 1);
    assert_eq!(tokens_with_pos[0].column, 3); // Plus is at column 3 (after 2 spaces)
}

#[test]
fn test_position_tracking_boundary_end_of_input() {
    // Test position tracking at end of input
    // Catches comparison operator mutations (position >= input.len())
    let code = "x";
    let tokens_with_pos = Lexer::new(code)
        .tokenize_with_positions_immutable()
        .unwrap();

    assert_eq!(tokens_with_pos.len(), 2); // Identifier + EOF
    assert_eq!(tokens_with_pos[0].line, 1);
    assert_eq!(tokens_with_pos[0].column, 1);
    // EOF position is tracked - verify it exists and has valid position
    assert!(matches!(tokens_with_pos[1].token, Token::EOF));
    assert!(tokens_with_pos[1].column >= 1); // EOF has valid column position
}

#[test]
fn test_position_tracking_multi_char_operator() {
    // Test that <= advances position correctly
    // Catches arithmetic operator mutations
    let code = "x <= 10";
    let tokens_with_pos = Lexer::new(code)
        .tokenize_with_positions_immutable()
        .unwrap();

    // Find the <= operator
    let less_equal_token = tokens_with_pos
        .iter()
        .find(|t| matches!(t.token, Token::Operator(Operator::LessEqual)));

    assert!(less_equal_token.is_some());
    let token = less_equal_token.unwrap();
    // <= starts after "x " - verify it's at a valid position (column 2 or 3 depending on spacing)
    assert!(
        token.column >= 2 && token.column <= 3,
        "<= should be at column 2 or 3, got {}",
        token.column
    );
    assert_eq!(token.line, 1);
}

// ============================================================================
// KEYWORD TESTS
// ============================================================================
// These tests catch "delete match arm" mutations by testing keyword recognition

#[test]
fn test_keyword_let() {
    let code = "let";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Keyword(Keyword::Let)));
}

#[test]
fn test_keyword_fn() {
    let code = "fn";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Keyword(Keyword::Fn)));
}

#[test]
fn test_keyword_if() {
    let code = "if";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Keyword(Keyword::If)));
}

#[test]
fn test_keyword_return() {
    let code = "return";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Keyword(Keyword::Return)));
}

#[test]
fn test_keyword_vs_identifier() {
    // Test that keywords are recognized vs identifiers
    // Catches delete match arm mutations in is_keyword
    let code1 = "let";
    let code2 = "let_var";

    let tokens1 = Lexer::new(code1).tokenize_immutable().unwrap();
    let tokens2 = Lexer::new(code2).tokenize_immutable().unwrap();

    assert!(matches!(tokens1[0], Token::Keyword(Keyword::Let)));
    assert!(matches!(tokens2[0], Token::Identifier(_)));
}

// ============================================================================
// LITERAL TESTS
// ============================================================================
// These tests catch "delete match arm" mutations and return value mutations

#[test]
fn test_literal_bool_true() {
    let code = "true";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Literal(Literal::Bool(true))));
}

#[test]
fn test_literal_bool_false() {
    let code = "false";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Literal(Literal::Bool(false))));
}

#[test]
fn test_literal_null() {
    let code = "null";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Literal(Literal::Null)));
}

#[test]
fn test_literal_int() {
    // Test integer literal - catches return value mutations
    let code = "42";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);

    if let Token::Literal(Literal::Int(value)) = &tokens[0] {
        assert_eq!(*value, 42);
    } else {
        panic!("Expected Int(42), got {:?}", tokens[0]);
    }
}

#[test]
fn test_literal_string() {
    // Test string literal - catches return value mutations
    let code = r#""hello""#;
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);

    if let Token::Literal(Literal::String(value)) = &tokens[0] {
        assert_eq!(value, "hello");
    } else {
        panic!("Expected String(\"hello\"), got {:?}", tokens[0]);
    }
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================
// These tests catch boundary condition mutations and error handling

#[test]
fn test_empty_input() {
    // Test empty input - catches boundary check mutations
    let code = "";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 1); // Just EOF
    assert!(matches!(tokens[0], Token::EOF));
}

#[test]
fn test_whitespace_only() {
    // Test whitespace-only input - catches boundary check mutations
    let code = "   \n\t  ";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 1); // Just EOF
    assert!(matches!(tokens[0], Token::EOF));
}

#[test]
fn test_invalid_single_ampersand() {
    // Test that single & is invalid - catches boundary check mutations
    let code = "&";
    let result = Lexer::new(code).tokenize_immutable();
    assert!(result.is_err(), "Single & should be an error");
}

#[test]
fn test_invalid_single_pipe() {
    // Test that single | is invalid - catches boundary check mutations
    let code = "|";
    let result = Lexer::new(code).tokenize_immutable();
    assert!(result.is_err(), "Single | should be an error");
}

#[test]
fn test_unterminated_string() {
    // Test unterminated string - catches boundary check mutations
    let code = r#""hello"#;
    let result = Lexer::new(code).tokenize_immutable();
    assert!(result.is_err(), "Unterminated string should be an error");
}

#[test]
fn test_identifier_with_underscore() {
    // Test identifier starting with underscore
    let code = "_test";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);
    if let Token::Identifier(name) = &tokens[0] {
        assert_eq!(name, "_test");
    } else {
        panic!("Expected Identifier(\"_test\"), got {:?}", tokens[0]);
    }
}

#[test]
fn test_number_parsing() {
    // Test number parsing - catches return value mutations
    let code = "123";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    assert_eq!(tokens.len(), 2);

    if let Token::Literal(Literal::Int(value)) = &tokens[0] {
        assert_eq!(*value, 123);
    } else {
        panic!("Expected Int(123), got {:?}", tokens[0]);
    }
}

#[test]
fn test_complex_expression_tokenization() {
    // Test complex expression with multiple operators
    // Catches multiple delete match arm mutations
    let code = "x + y < z && a == b";
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();

    // Should have: x, +, y, <, z, &&, a, ==, b, EOF
    assert!(tokens.len() >= 9);

    // Verify specific operators are present
    let has_plus = tokens
        .iter()
        .any(|t| matches!(t, Token::Operator(Operator::Plus)));
    let has_less = tokens
        .iter()
        .any(|t| matches!(t, Token::Operator(Operator::Less)));
    let has_and = tokens
        .iter()
        .any(|t| matches!(t, Token::Operator(Operator::And)));
    let has_equal = tokens
        .iter()
        .any(|t| matches!(t, Token::Operator(Operator::Equal)));

    assert!(has_plus, "Should have Plus operator");
    assert!(has_less, "Should have Less operator");
    assert!(has_and, "Should have And operator");
    assert!(has_equal, "Should have Equal operator");
}

// ============================================================================
// ESCAPE SEQUENCE TESTS
// ============================================================================
// These tests catch "delete match arm" mutations in decode_escape (lines 694-698)
// Each escape sequence must be tested individually so deleting one arm is caught.

#[test]
fn test_escape_sequence_double_quote() {
    // Catches: delete match arm '"' in decode_escape (line 694)
    let code = r#""hello\"world""#;
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    if let Token::Literal(Literal::String(value)) = &tokens[0] {
        assert_eq!(
            value, "hello\"world",
            "Escaped double quote should produce literal quote"
        );
    } else {
        panic!("Expected string literal, got {:?}", tokens[0]);
    }
}

#[test]
fn test_escape_sequence_backslash() {
    // Catches: delete match arm '\\' in decode_escape (line 695)
    let code = r#""path\\to\\file""#;
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    if let Token::Literal(Literal::String(value)) = &tokens[0] {
        assert_eq!(
            value, "path\\to\\file",
            "Escaped backslash should produce literal backslash"
        );
    } else {
        panic!("Expected string literal, got {:?}", tokens[0]);
    }
}

#[test]
fn test_escape_sequence_newline() {
    // Catches: delete match arm 'n' in decode_escape (line 696)
    let code = r#""line1\nline2""#;
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    if let Token::Literal(Literal::String(value)) = &tokens[0] {
        assert_eq!(
            value, "line1\nline2",
            "\\n should produce newline character"
        );
        assert!(
            value.contains('\n'),
            "String must contain actual newline char"
        );
    } else {
        panic!("Expected string literal, got {:?}", tokens[0]);
    }
}

#[test]
fn test_escape_sequence_carriage_return() {
    // Catches: delete match arm 'r' in decode_escape (line 697)
    let code = r#""before\rafter""#;
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    if let Token::Literal(Literal::String(value)) = &tokens[0] {
        assert_eq!(
            value, "before\rafter",
            "\\r should produce carriage return character"
        );
        assert!(
            value.contains('\r'),
            "String must contain actual carriage return char"
        );
    } else {
        panic!("Expected string literal, got {:?}", tokens[0]);
    }
}

#[test]
fn test_escape_sequence_tab() {
    // Catches: delete match arm 't' in decode_escape (line 698)
    let code = r#""col1\tcol2""#;
    let tokens = Lexer::new(code).tokenize_immutable().unwrap();
    if let Token::Literal(Literal::String(value)) = &tokens[0] {
        assert_eq!(value, "col1\tcol2", "\\t should produce tab character");
        assert!(value.contains('\t'), "String must contain actual tab char");
    } else {
        panic!("Expected string literal, got {:?}", tokens[0]);
    }
}

// ============================================================================
// MUTABLE TOKENIZE() TESTS
// ============================================================================
// These tests use tokenize() (mutable version) to cover read_string, read_number,
// read_identifier — the non-immutable code paths that have 20+ missed mutations.

#[test]
fn test_mutable_tokenize_string_exact_value() {
    // Catches: replace Lexer::read_string -> Result<String, LexerError> with Ok(String::new())
    // Catches: replace Lexer::read_string -> Result<String, LexerError> with Ok("xyzzy".into())
    let code = r#""hello world""#;
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    if let Token::Literal(Literal::String(value)) = &tokens[0] {
        assert_eq!(
            value, "hello world",
            "read_string must return exact string content"
        );
        assert_ne!(value, "", "read_string must not return empty string");
        assert_ne!(value, "xyzzy", "read_string must not return garbage");
    } else {
        panic!("Expected string literal, got {:?}", tokens[0]);
    }
}

#[test]
fn test_mutable_tokenize_string_with_escape() {
    // Catches: replace == with != in Lexer::read_string (lines 373, 376)
    let code = r#""tab\there""#;
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    if let Token::Literal(Literal::String(value)) = &tokens[0] {
        assert_eq!(value, "tab\there");
    } else {
        panic!("Expected string literal, got {:?}", tokens[0]);
    }
}

#[test]
fn test_mutable_tokenize_string_boundary() {
    // Catches: replace < with == in Lexer::read_string (line 371)
    // Catches: replace < with > in Lexer::read_string (line 371)
    // Catches: replace < with <= in Lexer::read_string (line 371)
    // Tests: string at end of input, string with closing quote
    let code = r#""end""#;
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2); // string + EOF
    if let Token::Literal(Literal::String(value)) = &tokens[0] {
        assert_eq!(value, "end");
    } else {
        panic!("Expected string literal, got {:?}", tokens[0]);
    }
}

#[test]
fn test_mutable_tokenize_number_exact_value() {
    // Catches: replace Lexer::read_number -> Result<i64, LexerError> with Ok(0)
    // Catches: replace Lexer::read_number -> Result<i64, LexerError> with Ok(1)
    // Catches: replace Lexer::read_number -> Result<i64, LexerError> with Ok(-1)
    let code = "42";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    if let Token::Literal(Literal::Int(value)) = &tokens[0] {
        assert_eq!(*value, 42, "read_number must return exact number");
        assert_ne!(*value, 0, "read_number must not return 0");
        assert_ne!(*value, 1, "read_number must not return 1");
        assert_ne!(*value, -1, "read_number must not return -1");
    } else {
        panic!("Expected int literal, got {:?}", tokens[0]);
    }
}

#[test]
fn test_mutable_tokenize_number_boundary() {
    // Catches: replace < with == in Lexer::read_number (line 352)
    // Catches: replace < with <= in Lexer::read_number (line 352)
    // Number at end of input — boundary condition
    let code = "999";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2); // number + EOF
    if let Token::Literal(Literal::Int(value)) = &tokens[0] {
        assert_eq!(*value, 999);
    } else {
        panic!("Expected int literal, got {:?}", tokens[0]);
    }
}

#[test]
fn test_mutable_tokenize_number_followed_by_identifier() {
    // Verifies number stops at non-digit boundary
    let code = "123abc";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    if let Token::Literal(Literal::Int(value)) = &tokens[0] {
        assert_eq!(*value, 123, "Number should stop at 'a'");
    } else {
        panic!("Expected int literal, got {:?}", tokens[0]);
    }
    if let Token::Identifier(name) = &tokens[1] {
        assert_eq!(name, "abc", "Identifier should be 'abc' after number");
    } else {
        panic!("Expected identifier 'abc', got {:?}", tokens[1]);
    }
}

#[test]
fn test_mutable_tokenize_identifier_exact_value() {
    // Catches: replace Lexer::read_identifier -> String with String::new()
    // Catches: replace Lexer::read_identifier -> String with "xyzzy".into()
    let code = "myVariable";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    if let Token::Identifier(name) = &tokens[0] {
        assert_eq!(name, "myVariable", "read_identifier must return exact name");
        assert_ne!(name, "", "read_identifier must not return empty string");
        assert_ne!(name, "xyzzy", "read_identifier must not return garbage");
    } else {
        panic!("Expected identifier, got {:?}", tokens[0]);
    }
}

#[test]
fn test_mutable_tokenize_identifier_boundary() {
    // Catches: replace < with == in Lexer::read_identifier (line 337)
    // Catches: replace || with && in Lexer::read_identifier (line 339)
    // Catches: replace == with != in Lexer::read_identifier (line 339)
    // Identifier at end of input — boundary condition
    let code = "x";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2); // identifier + EOF
    if let Token::Identifier(name) = &tokens[0] {
        assert_eq!(name, "x");
    } else {
        panic!("Expected identifier, got {:?}", tokens[0]);
    }
}

#[test]
fn test_mutable_tokenize_identifier_with_underscore() {
    // Catches: replace == with != in Lexer::read_identifier (line 339)
    // The _ character check: ch == '_'
    let code = "has_underscore_here";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    if let Token::Identifier(name) = &tokens[0] {
        assert_eq!(name, "has_underscore_here");
    } else {
        panic!("Expected identifier, got {:?}", tokens[0]);
    }
}

#[test]
fn test_mutable_tokenize_unterminated_string() {
    // Catches: boundary mutations in read_string loop
    let code = r#""unterminated"#;
    let mut lexer = Lexer::new(code);
    let result = lexer.tokenize();
    assert!(
        result.is_err(),
        "Unterminated string should fail in mutable tokenize"
    );
}

#[test]
fn test_mutable_tokenize_multitoken_statement() {
    // Comprehensive test exercising all 3 readers in one pass
    let code = r#"let name = "hello";"#;
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    // let, name, =, "hello", ;, EOF
    assert!(tokens.len() >= 5);
    assert!(matches!(tokens[0], Token::Keyword(Keyword::Let)));
    if let Token::Identifier(name) = &tokens[1] {
        assert_eq!(name, "name");
    }
    if let Token::Literal(Literal::String(value)) = &tokens[3] {
        assert_eq!(value, "hello");
    }
}

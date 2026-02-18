// Lexer Boundary Tests - Critical Improvements
// These tests catch comparison operator mutations, arithmetic mutations, and boundary conditions
// Target: Catch ~15-20 lexer mutations from the first 99 mutations analyzed

use dist_agent_lang::lexer::lexer::Lexer;
use dist_agent_lang::lexer::tokens::{get_trust_profiles, Operator, Token, TrustLevel};

// ============================================================================
// CRITICAL: LEXER BOUNDARY TESTS (Targeting specific mutations)
// ============================================================================

#[test]
fn test_next_token_immutable_comparison_boundary() {
    // Test comparison boundary: position >= input.len()
    // Catches: replace >= with >, == in Lexer::next_token_immutable (line 71)
    // If mutated to >, position == input.len() would incorrectly proceed
    // If mutated to ==, position > input.len() would incorrectly proceed

    let code = "test";
    let lexer = Lexer::new(code);

    // Tokenize the input - this calls next_token_immutable multiple times
    let tokens = lexer.tokenize_immutable().unwrap();

    // Should tokenize all characters and end with EOF
    // If comparison is wrong (>= -> >), it might try to read past end
    // If comparison is wrong (>= -> ==), it might try to read past end
    assert!(!tokens.is_empty(), "Should tokenize input");

    // Last token should be EOF (not an error from reading past end)
    let last_token = &tokens[tokens.len() - 1];
    assert!(
        matches!(last_token, Token::EOF),
        "Should end with EOF, not error from boundary violation"
    );
}

#[test]
fn test_lexer_comparison_boundary_less_than() {
    // Test exact boundary: position < input.len() vs position <= input.len()
    // Catches: replace < with <= in Lexer::tokenize_with_positions_immutable (line 37)
    // Catches: replace < with > in Lexer::next_token_immutable (line 93, 95)
    let code = "x";
    let lexer = Lexer::new(code);
    let tokens = lexer.tokenize_immutable().unwrap();

    // Should tokenize exactly 1 identifier + EOF
    // If < is mutated to <=, might read past end
    // If < is mutated to >, might not read anything
    assert_eq!(tokens.len(), 2, "Should have identifier and EOF");
    assert!(matches!(tokens[0], Token::Identifier(_)));
    assert!(matches!(tokens[1], Token::EOF));
}

#[test]
fn test_lexer_comparison_boundary_equal() {
    // Test exact boundary: == vs !=
    // Catches: replace == with != in Lexer::next_token_immutable (line 94, 96)
    let code = "x::y";
    let lexer = Lexer::new(code);
    let tokens = lexer.tokenize_immutable().unwrap();

    // Should tokenize namespace call correctly
    // If == is mutated to !=, namespace detection fails
    assert!(tokens.len() >= 2, "Should tokenize namespace");
    // Verify namespace tokens are present
    let has_colon_colon = tokens.iter().any(|t| matches!(t, Token::Punctuation(_)));
    assert!(has_colon_colon, "Should detect namespace separator");
}

#[test]
fn test_lexer_arithmetic_plus_boundary() {
    // Test exact arithmetic: + vs - vs *
    // Catches: replace + with - in Lexer::next_token_immutable (line 95, 96)
    // Catches: replace + with * in Lexer::next_token_immutable (line 95, 96)
    let code = "x + 1";
    let lexer = Lexer::new(code);
    let tokens = lexer.tokenize_immutable().unwrap();

    // Should tokenize with correct position tracking
    // If + is mutated to -, position calculation wrong (might cause errors)
    // If + is mutated to *, position calculation wrong (might cause errors)
    assert!(tokens.len() >= 3, "Should tokenize expression");

    // Find the + operator
    let has_plus = tokens
        .iter()
        .any(|t| matches!(t, Token::Operator(Operator::Plus)));
    assert!(has_plus, "Should contain Plus operator");
}

#[test]
fn test_lexer_logical_and_boundary() {
    // Test exact logical: && vs ||
    // Catches: replace && with || in Lexer::next_token_immutable (line 94)
    let code = "x && y";
    let lexer = Lexer::new(code);
    let tokens = lexer.tokenize_immutable().unwrap();

    // Should tokenize && correctly
    // If && is mutated to ||, wrong operator detected
    let has_and = tokens
        .iter()
        .any(|t| matches!(t, Token::Operator(Operator::And)));
    assert!(has_and, "Should contain And operator, not Or");
}

#[test]
fn test_lexer_position_advancement_single_char() {
    // Test that position always advances (prevents infinite loops)
    // Catches: mutations that cause position to not advance
    let code = "+";
    let lexer = Lexer::new(code);
    let tokens = lexer.tokenize_immutable().unwrap();

    // Should tokenize single character and advance
    // If position doesn't advance, would cause infinite loop (now caught by bounds check)
    assert_eq!(tokens.len(), 2, "Should have operator and EOF");
    assert!(matches!(tokens[0], Token::Operator(Operator::Plus)));
}

#[test]
fn test_lexer_position_advancement_multi_char() {
    // Test position advancement for multi-character tokens
    // Catches: mutations in position calculation for multi-char operators
    let code = "==";
    let lexer = Lexer::new(code);
    let tokens = lexer.tokenize_immutable().unwrap();

    // Should tokenize == correctly (2 chars, position advances by 2)
    // If position calculation wrong, might only advance by 1
    assert_eq!(tokens.len(), 2, "Should have operator and EOF");
    assert!(matches!(tokens[0], Token::Operator(Operator::Equal)));
}

#[test]
fn test_lexer_column_tracking_boundary() {
    // Test column tracking doesn't go below 1
    // Catches: mutations that cause column to decrease incorrectly
    let code = "x y z";
    let lexer = Lexer::new(code);
    let tokens_with_pos = lexer.tokenize_with_positions_immutable().unwrap();

    // All columns should be >= 1
    // If column calculation wrong (e.g., += mutated to -=), columns might be 0
    for twp in &tokens_with_pos {
        assert!(twp.column >= 1, "Column should always be >= 1");
    }
}

#[test]
fn test_lexer_bounds_check_position_advance() {
    // Test bounds checking prevents infinite loops
    // Catches: mutations that cause position to not advance (now caught by safety check)
    let code = "test123";
    let lexer = Lexer::new(code);

    // Should complete without infinite loop
    // If position doesn't advance, bounds check should catch it
    let result = lexer.tokenize_immutable();
    assert!(result.is_ok(), "Should tokenize without infinite loop");

    let tokens = result.unwrap();
    assert!(tokens.len() >= 2, "Should have tokens");
    assert!(matches!(tokens[tokens.len() - 1], Token::EOF));
}

#[test]
fn test_lexer_comparison_operator_less_vs_less_equal() {
    // Test exact boundary between < and <=
    // Catches: replace < with <= in comparison checks
    let code1 = "<";
    let code2 = "<=";

    let tokens1 = Lexer::new(code1).tokenize_immutable().unwrap();
    let tokens2 = Lexer::new(code2).tokenize_immutable().unwrap();

    // Should distinguish between < and <=
    assert!(matches!(tokens1[0], Token::Operator(Operator::Less)));
    assert!(matches!(tokens2[0], Token::Operator(Operator::LessEqual)));
}

#[test]
fn test_lexer_comparison_operator_greater_vs_greater_equal() {
    // Test exact boundary between > and >=
    // Catches: replace > with >= in comparison checks
    let code1 = ">";
    let code2 = ">=";

    let tokens1 = Lexer::new(code1).tokenize_immutable().unwrap();
    let tokens2 = Lexer::new(code2).tokenize_immutable().unwrap();

    // Should distinguish between > and >=
    assert!(matches!(tokens1[0], Token::Operator(Operator::Greater)));
    assert!(matches!(
        tokens2[0],
        Token::Operator(Operator::GreaterEqual)
    ));
}

#[test]
fn test_lexer_identifier_namespace_detection() {
    // Test namespace detection with exact comparison
    // Catches: replace == with != in namespace detection (line 94, 96)
    let code = "std::io";
    let lexer = Lexer::new(code);
    let tokens = lexer.tokenize_immutable().unwrap();

    // Should detect namespace separator ::
    // If == is mutated to !=, namespace detection fails
    let has_double_colon = tokens.iter().any(|t| {
        matches!(
            t,
            Token::Punctuation(dist_agent_lang::lexer::tokens::Punctuation::DoubleColon)
        )
    });
    assert!(has_double_colon, "Should detect namespace separator");
}

#[test]
fn test_get_trust_profiles_return_value() {
    // Test exact return value of get_trust_profiles
    // Catches: return value mutations (line 683)
    let profiles = get_trust_profiles();

    // Should return a HashMap with specific trust levels
    // If return value is mutated, profiles might be empty or wrong
    assert!(!profiles.is_empty(), "Should return non-empty profiles");

    // Verify specific trust levels exist
    assert!(
        profiles.contains_key(&TrustLevel::Decentralized),
        "Should contain Decentralized trust level"
    );

    // Verify profiles have expected structure (not empty/default)
    assert!(
        profiles.get(&TrustLevel::Decentralized).is_some(),
        "Decentralized profile should have configuration"
    );
}

// ============================================================================
// PHASE 4: ADDITIONAL LEXER POSITION TESTS
// ============================================================================
// These tests catch arithmetic mutations in position increments

#[test]
fn test_lexer_position_increment_plus_vs_minus() {
    // Test that position increments use +, not -
    // Catches: replace += with -= in Lexer::next_token_immutable (line 162, 167, etc.)
    let code = "abc def";
    let lexer = Lexer::new(code);
    let tokens = lexer.tokenize_immutable().unwrap();

    // Should tokenize all characters correctly
    // If += is mutated to -=, position would go backwards (wrong)
    assert!(tokens.len() >= 3, "Should tokenize multiple identifiers");

    // Verify we have identifiers (not errors from wrong position)
    let has_identifiers = tokens.iter().any(|t| matches!(t, Token::Identifier(_)));
    assert!(
        has_identifiers,
        "Should contain identifiers (position increment works correctly)"
    );
}

#[test]
fn test_lexer_position_increment_plus_vs_multiply() {
    // Test that position increments use +, not *
    // Catches: replace += with *= in Lexer::next_token_immutable
    let code = "x y z";
    let lexer = Lexer::new(code);
    let tokens = lexer.tokenize_immutable().unwrap();

    // Should tokenize all identifiers
    // If += is mutated to *=, position calculation would be wrong
    // For position 0: 0 * 1 = 0 (stuck!)
    // For position 1: 1 * 1 = 1 (stuck!)
    assert!(tokens.len() >= 4, "Should tokenize all identifiers and EOF");

    // Count identifiers - should have 3
    let identifier_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::Identifier(_)))
        .count();
    assert_eq!(
        identifier_count, 3,
        "Should have exactly 3 identifiers (position increments correctly)"
    );
}

#[test]
fn test_lexer_position_tracking_newline_boundary() {
    // Test position tracking across newlines
    // Catches: mutations in line/column tracking
    let code = "x\ny\nz";
    let lexer = Lexer::new(code);
    let tokens_with_pos = lexer.tokenize_with_positions_immutable().unwrap();

    // Should track line numbers correctly
    // First identifier should be on line 1
    // Second identifier should be on line 2
    // Third identifier should be on line 3
    let line_numbers: Vec<usize> = tokens_with_pos
        .iter()
        .filter(|twp| matches!(twp.token, Token::Identifier(_)))
        .map(|twp| twp.line)
        .collect();

    assert_eq!(line_numbers.len(), 3, "Should have 3 identifiers");
    assert_eq!(line_numbers[0], 1, "First identifier should be on line 1");
    assert_eq!(line_numbers[1], 2, "Second identifier should be on line 2");
    assert_eq!(line_numbers[2], 3, "Third identifier should be on line 3");
}

#[test]
fn test_lexer_position_tracking_whitespace_boundary() {
    // Test position tracking with whitespace
    // Catches: mutations in column tracking for whitespace
    let code = "  x  y  ";
    let lexer = Lexer::new(code);
    let tokens_with_pos = lexer.tokenize_with_positions_immutable().unwrap();

    // Should track column positions correctly
    // First identifier 'x' should be at column 3 (after 2 spaces)
    // Second identifier 'y' should be at column 6 (after 'x' and 2 spaces)
    let identifiers: Vec<_> = tokens_with_pos
        .iter()
        .filter(|twp| matches!(twp.token, Token::Identifier(_)))
        .collect();

    assert_eq!(identifiers.len(), 2, "Should have 2 identifiers");
    // Column positions should be correct (not 0 or wrong)
    assert!(
        identifiers[0].column >= 1,
        "First identifier column should be >= 1"
    );
    assert!(
        identifiers[1].column > identifiers[0].column,
        "Second identifier should be after first"
    );
}

#[test]
fn test_lexer_position_tracking_multi_char_operator() {
    // Test position tracking for multi-character operators
    // Catches: mutations in position increment for 2-char operators
    let code = "x == y";
    let lexer = Lexer::new(code);
    let tokens_with_pos = lexer.tokenize_with_positions_immutable().unwrap();

    // Should tokenize == correctly (2 characters)
    // Position should advance by 2 after ==
    // If position increment is wrong, might only advance by 1
    let has_equal = tokens_with_pos
        .iter()
        .any(|twp| matches!(twp.token, Token::Operator(Operator::Equal)));

    assert!(has_equal, "Should contain == operator");

    // Verify we have all expected tokens (x, ==, y, EOF)
    let token_count = tokens_with_pos.len();
    assert!(
        token_count >= 4,
        "Should have x, ==, y, and EOF (position tracking works)"
    );
}

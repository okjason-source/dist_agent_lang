// Lexer Tokens Mutation Tests
// These tests are designed to catch mutations in enum from_string/to_string methods
// and related token functionality

use dist_agent_lang::lexer::tokens::{
    BlockchainNetwork, CompilationTarget, InterfaceLanguage, Token, TokenStruct, TokenType,
    TrustLevel,
};

// ============================================================================
// COMPILATION TARGET TESTS
// ============================================================================
// These tests catch delete match arm and return value mutations

#[test]
fn test_compilation_target_from_string_blockchain() {
    // Test "blockchain" - catches delete match arm mutations
    let result = CompilationTarget::from_string("blockchain");
    assert!(result.is_some(), "Should parse 'blockchain'");
    assert!(matches!(result.unwrap(), CompilationTarget::Blockchain));
}

#[test]
fn test_compilation_target_from_string_wasm() {
    // Test "wasm" - catches delete match arm mutations
    let result = CompilationTarget::from_string("wasm");
    assert!(result.is_some(), "Should parse 'wasm'");
    assert!(matches!(result.unwrap(), CompilationTarget::WebAssembly));
}

#[test]
fn test_compilation_target_from_string_webassembly() {
    // Test "webassembly" - catches delete match arm mutations
    let result = CompilationTarget::from_string("webassembly");
    assert!(result.is_some(), "Should parse 'webassembly'");
    assert!(matches!(result.unwrap(), CompilationTarget::WebAssembly));
}

#[test]
fn test_compilation_target_from_string_native() {
    // Test "native" - catches delete match arm mutations
    let result = CompilationTarget::from_string("native");
    assert!(result.is_some(), "Should parse 'native'");
    assert!(matches!(result.unwrap(), CompilationTarget::Native));
}

#[test]
fn test_compilation_target_from_string_mobile() {
    // Test "mobile" - catches delete match arm mutations
    let result = CompilationTarget::from_string("mobile");
    assert!(result.is_some(), "Should parse 'mobile'");
    assert!(matches!(result.unwrap(), CompilationTarget::Mobile));
}

#[test]
fn test_compilation_target_from_string_edge() {
    // Test "edge" - catches delete match arm mutations
    let result = CompilationTarget::from_string("edge");
    assert!(result.is_some(), "Should parse 'edge'");
    assert!(matches!(result.unwrap(), CompilationTarget::Edge));
}

#[test]
fn test_compilation_target_from_string_invalid() {
    // Test invalid string - should return None
    // Catches mutations that return Some instead of None
    let result = CompilationTarget::from_string("invalid");
    assert!(result.is_none(), "Should return None for invalid string");
}

#[test]
fn test_compilation_target_to_string_blockchain() {
    // Test to_string for Blockchain - catches return value mutations
    let target = CompilationTarget::Blockchain;
    let result = target.to_string();
    assert_eq!(
        result, "blockchain",
        "Should return 'blockchain', not empty string or 'xyzzy'"
    );
}

#[test]
fn test_compilation_target_to_string_wasm() {
    // Test to_string for WebAssembly - catches return value mutations
    let target = CompilationTarget::WebAssembly;
    let result = target.to_string();
    assert_eq!(
        result, "wasm",
        "Should return 'wasm', not empty string or 'xyzzy'"
    );
}

#[test]
fn test_compilation_target_to_string_native() {
    // Test to_string for Native - catches return value mutations
    let target = CompilationTarget::Native;
    let result = target.to_string();
    assert_eq!(
        result, "native",
        "Should return 'native', not empty string or 'xyzzy'"
    );
}

#[test]
fn test_compilation_target_to_string_mobile() {
    // Test to_string for Mobile - catches return value mutations
    let target = CompilationTarget::Mobile;
    let result = target.to_string();
    assert_eq!(
        result, "mobile",
        "Should return 'mobile', not empty string or 'xyzzy'"
    );
}

#[test]
fn test_compilation_target_to_string_edge() {
    // Test to_string for Edge - catches return value mutations
    let target = CompilationTarget::Edge;
    let result = target.to_string();
    assert_eq!(
        result, "edge",
        "Should return 'edge', not empty string or 'xyzzy'"
    );
}

// ============================================================================
// TRUST LEVEL TESTS
// ============================================================================

#[test]
fn test_trust_level_from_string_decentralized() {
    // Test "decentralized" - catches delete match arm mutations
    let result = TrustLevel::from_string("decentralized");
    assert!(result.is_some(), "Should parse 'decentralized'");
    assert!(matches!(result.unwrap(), TrustLevel::Decentralized));
}

#[test]
fn test_trust_level_from_string_hybrid() {
    // Test "hybrid" - catches delete match arm mutations
    let result = TrustLevel::from_string("hybrid");
    assert!(result.is_some(), "Should parse 'hybrid'");
    assert!(matches!(result.unwrap(), TrustLevel::Hybrid));
}

#[test]
fn test_trust_level_from_string_centralized() {
    // Test "centralized" - catches delete match arm mutations
    let result = TrustLevel::from_string("centralized");
    assert!(result.is_some(), "Should parse 'centralized'");
    assert!(matches!(result.unwrap(), TrustLevel::Centralized));
}

#[test]
fn test_trust_level_from_string_invalid() {
    // Test invalid string - should return None
    let result = TrustLevel::from_string("invalid");
    assert!(result.is_none(), "Should return None for invalid string");
}

#[test]
fn test_trust_level_to_string_decentralized() {
    // Test to_string for Decentralized - catches return value mutations
    let level = TrustLevel::Decentralized;
    let result = level.to_string();
    assert_eq!(
        result, "decentralized",
        "Should return 'decentralized', not empty string or 'xyzzy'"
    );
}

#[test]
fn test_trust_level_to_string_hybrid() {
    // Test to_string for Hybrid - catches return value mutations
    let level = TrustLevel::Hybrid;
    let result = level.to_string();
    assert_eq!(
        result, "hybrid",
        "Should return 'hybrid', not empty string or 'xyzzy'"
    );
}

#[test]
fn test_trust_level_to_string_centralized() {
    // Test to_string for Centralized - catches return value mutations
    let level = TrustLevel::Centralized;
    let result = level.to_string();
    assert_eq!(
        result, "centralized",
        "Should return 'centralized', not empty string or 'xyzzy'"
    );
}

// ============================================================================
// BLOCKCHAIN NETWORK TESTS
// ============================================================================

#[test]
fn test_blockchain_network_from_string_ethereum() {
    // Test "ethereum" - catches delete match arm mutations
    let result = BlockchainNetwork::from_string("ethereum");
    assert!(result.is_some(), "Should parse 'ethereum'");
    assert!(matches!(result.unwrap(), BlockchainNetwork::Ethereum));
}

#[test]
fn test_blockchain_network_from_string_polygon() {
    // Test "polygon" - catches delete match arm mutations
    let result = BlockchainNetwork::from_string("polygon");
    assert!(result.is_some(), "Should parse 'polygon'");
    assert!(matches!(result.unwrap(), BlockchainNetwork::Polygon));
}

#[test]
fn test_blockchain_network_from_string_binance() {
    // Test "binance" - catches delete match arm mutations
    let result = BlockchainNetwork::from_string("binance");
    assert!(result.is_some(), "Should parse 'binance'");
    assert!(matches!(result.unwrap(), BlockchainNetwork::Binance));
}

#[test]
fn test_blockchain_network_from_string_solana() {
    // Test "solana" - catches delete match arm mutations
    let result = BlockchainNetwork::from_string("solana");
    assert!(result.is_some(), "Should parse 'solana'");
    assert!(matches!(result.unwrap(), BlockchainNetwork::Solana));
}

#[test]
fn test_blockchain_network_from_string_avalanche() {
    // Test "avalanche" - catches delete match arm mutations
    let result = BlockchainNetwork::from_string("avalanche");
    assert!(result.is_some(), "Should parse 'avalanche'");
    assert!(matches!(result.unwrap(), BlockchainNetwork::Avalanche));
}

#[test]
fn test_blockchain_network_from_string_arbitrum() {
    // Test "arbitrum" - catches delete match arm mutations
    let result = BlockchainNetwork::from_string("arbitrum");
    assert!(result.is_some(), "Should parse 'arbitrum'");
    assert!(matches!(result.unwrap(), BlockchainNetwork::Arbitrum));
}

#[test]
fn test_blockchain_network_from_string_optimism() {
    // Test "optimism" - catches delete match arm mutations
    let result = BlockchainNetwork::from_string("optimism");
    assert!(result.is_some(), "Should parse 'optimism'");
    assert!(matches!(result.unwrap(), BlockchainNetwork::Optimism));
}

#[test]
fn test_blockchain_network_from_string_custom() {
    // Test custom network - should return Custom variant
    let result = BlockchainNetwork::from_string("custom_network");
    assert!(result.is_some(), "Should parse custom network");
    if let Some(BlockchainNetwork::Custom(name)) = result {
        assert_eq!(name, "custom_network");
    } else {
        panic!("Should return Custom variant");
    }
}

#[test]
fn test_blockchain_network_to_string_ethereum() {
    // Test to_string for Ethereum - catches return value mutations
    let network = BlockchainNetwork::Ethereum;
    let result = network.to_string();
    assert_eq!(
        result, "ethereum",
        "Should return 'ethereum', not empty string or 'xyzzy'"
    );
}

#[test]
fn test_blockchain_network_to_string_solana() {
    // Test to_string for Solana - catches return value mutations
    let network = BlockchainNetwork::Solana;
    let result = network.to_string();
    assert_eq!(
        result, "solana",
        "Should return 'solana', not empty string or 'xyzzy'"
    );
}

#[test]
fn test_blockchain_network_is_evm_compatible_ethereum() {
    // Test is_evm_compatible for Ethereum - catches return value mutations
    let network = BlockchainNetwork::Ethereum;
    let result = network.is_evm_compatible();
    assert!(result, "Ethereum should be EVM compatible, not false");
}

#[test]
fn test_blockchain_network_is_evm_compatible_solana() {
    // Test is_evm_compatible for Solana - should be false
    let network = BlockchainNetwork::Solana;
    let result = network.is_evm_compatible();
    assert!(!result, "Solana should not be EVM compatible");
}

#[test]
fn test_blockchain_network_is_solana_compatible_solana() {
    // Test is_solana_compatible for Solana - catches return value mutations
    let network = BlockchainNetwork::Solana;
    let result = network.is_solana_compatible();
    assert!(result, "Solana should be Solana compatible, not false");
}

#[test]
fn test_blockchain_network_is_solana_compatible_ethereum() {
    // Test is_solana_compatible for Ethereum - should be false
    let network = BlockchainNetwork::Ethereum;
    let result = network.is_solana_compatible();
    assert!(!result, "Ethereum should not be Solana compatible");
}

// ============================================================================
// INTERFACE LANGUAGE TESTS
// ============================================================================

#[test]
fn test_interface_language_from_string_typescript() {
    // Test "typescript" - catches delete match arm mutations
    let result = InterfaceLanguage::from_string("typescript");
    assert!(result.is_some(), "Should parse 'typescript'");
    assert!(matches!(result.unwrap(), InterfaceLanguage::TypeScript));
}

#[test]
fn test_interface_language_from_string_javascript() {
    // Test "javascript" - catches delete match arm mutations
    let result = InterfaceLanguage::from_string("javascript");
    assert!(result.is_some(), "Should parse 'javascript'");
    assert!(matches!(result.unwrap(), InterfaceLanguage::JavaScript));
}

#[test]
fn test_interface_language_from_string_python() {
    // Test "python" - catches delete match arm mutations
    let result = InterfaceLanguage::from_string("python");
    assert!(result.is_some(), "Should parse 'python'");
    assert!(matches!(result.unwrap(), InterfaceLanguage::Python));
}

#[test]
fn test_interface_language_from_string_rust() {
    // Test "rust" - catches delete match arm mutations
    let result = InterfaceLanguage::from_string("rust");
    assert!(result.is_some(), "Should parse 'rust'");
    assert!(matches!(result.unwrap(), InterfaceLanguage::Rust));
}

#[test]
fn test_interface_language_from_string_java() {
    // Test "java" - catches delete match arm mutations
    let result = InterfaceLanguage::from_string("java");
    assert!(result.is_some(), "Should parse 'java'");
    assert!(matches!(result.unwrap(), InterfaceLanguage::Java));
}

#[test]
fn test_interface_language_from_string_go() {
    // Test "go" - catches delete match arm mutations
    let result = InterfaceLanguage::from_string("go");
    assert!(result.is_some(), "Should parse 'go'");
    assert!(matches!(result.unwrap(), InterfaceLanguage::Go));
}

#[test]
fn test_interface_language_from_string_custom() {
    // Test custom language - should return Custom variant
    let result = InterfaceLanguage::from_string("custom_lang");
    assert!(result.is_some(), "Should parse custom language");
    if let Some(InterfaceLanguage::Custom(name)) = result {
        assert_eq!(name, "custom_lang");
    } else {
        panic!("Should return Custom variant");
    }
}

#[test]
fn test_interface_language_to_string_typescript() {
    // Test to_string for TypeScript - catches return value mutations
    let lang = InterfaceLanguage::TypeScript;
    let result = lang.to_string();
    assert_eq!(
        result, "typescript",
        "Should return 'typescript', not empty string or 'xyzzy'"
    );
}

#[test]
fn test_interface_language_to_string_javascript() {
    // Test to_string for JavaScript - catches return value mutations
    let lang = InterfaceLanguage::JavaScript;
    let result = lang.to_string();
    assert_eq!(
        result, "javascript",
        "Should return 'javascript', not empty string or 'xyzzy'"
    );
}

// ============================================================================
// DISPLAY TRAIT TESTS
// ============================================================================
// These tests catch mutations in Display trait implementations

#[test]
fn test_token_type_display() {
    // Test Display for TokenType - catches return value mutations
    let token_type = TokenType::Number("42".to_string());
    let result = format!("{}", token_type);
    // Should format correctly, not return empty or default
    assert!(!result.is_empty(), "Display should not return empty string");
    assert!(result.contains("42"), "Display should include the number");
}

#[test]
fn test_token_struct_display() {
    // Test Display for TokenStruct - catches return value mutations
    let token_struct = TokenStruct {
        token_type: TokenType::String("test".to_string()),
        line: 1,
        column: 5,
        lexeme: "test".to_string(),
    };
    let result = format!("{}", token_struct);
    // Should format correctly with line and column
    assert!(!result.is_empty(), "Display should not return empty string");
    assert!(result.contains("1"), "Display should include line number");
    assert!(result.contains("5"), "Display should include column number");
}

#[test]
fn test_token_display_keyword() {
    // Test Display for Token::Keyword - catches return value mutations
    use dist_agent_lang::lexer::tokens::Keyword;
    let token = Token::Keyword(Keyword::Let);
    let result = format!("{}", token);
    // Should format correctly
    assert!(!result.is_empty(), "Display should not return empty string");
    assert!(
        result.to_lowercase().contains("keyword"),
        "Display should include 'keyword'"
    );
}

#[test]
fn test_token_display_operator() {
    // Test Display for Token::Operator - catches return value mutations
    use dist_agent_lang::lexer::tokens::Operator;
    let token = Token::Operator(Operator::Plus);
    let result = format!("{}", token);
    // Should format correctly
    assert!(!result.is_empty(), "Display should not return empty string");
    assert!(
        result.to_lowercase().contains("operator"),
        "Display should include 'operator'"
    );
}

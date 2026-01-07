#![no_main]

use libfuzzer_sys::fuzz_target;
use dist_agent_lang::lexer::Lexer;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(input) = std::str::from_utf8(data) {
        // Test lexer with arbitrary input
        let lexer = Lexer::new(input);
        let _ = lexer.tokenize_immutable();
        // We don't care about errors - we just want to ensure it doesn't panic
    }
});


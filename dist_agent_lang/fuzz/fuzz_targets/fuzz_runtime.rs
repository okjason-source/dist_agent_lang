#![no_main]

use libfuzzer_sys::fuzz_target;
use dist_agent_lang::lexer::Lexer;
use dist_agent_lang::parser::Parser;
use dist_agent_lang::runtime::Runtime;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(input) = std::str::from_utf8(data) {
        // Tokenize
        let lexer = Lexer::new(input);
        if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
            // Parse
            let mut parser = Parser::new_with_positions(tokens_with_pos);
            if let Ok(program) = parser.parse() {
                // Try to execute
                let mut runtime = Runtime::new();
                let _ = runtime.execute_program(program);
                // We don't care about errors - we just want to ensure it doesn't panic
            }
        }
    }
});


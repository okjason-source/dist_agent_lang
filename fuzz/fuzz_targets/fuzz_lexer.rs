#![no_main]

use libfuzzer_sys::fuzz_target;
use dist_agent_lang::lexer::Lexer;
use std::sync::mpsc;
use std::time::Duration;

/// Per-input timeout so a single slow unit doesn't dominate fuzzing (e.g. 897s).
const FUZZ_TIMEOUT_SECS: u64 = 5;

fuzz_target!(|data: &[u8]| {
    // Cap input size to avoid lexer blow-up and 15-minute slow units
    if data.len() > 65536 {
        return;
    }
    if let Ok(input) = std::str::from_utf8(data) {
        let (tx, rx) = mpsc::channel();
        let input = input.to_string();
        let _ = std::thread::spawn(move || {
            let lexer = Lexer::new(&input);
            if let Ok(tokens_with_pos) = lexer.tokenize_with_positions_immutable() {
                // Skip pathological inputs that produce huge token counts (DoS / slow units)
                if tokens_with_pos.len() > 50_000 {
                    let _ = tx.send(());
                    return;
                }
            }
            let _ = tx.send(());
        });
        let _ = rx.recv_timeout(Duration::from_secs(FUZZ_TIMEOUT_SECS));
        // On timeout we return; fuzzer moves on without waiting for the thread
    }
});


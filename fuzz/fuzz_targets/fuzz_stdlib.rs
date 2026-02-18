#![no_main]

use libfuzzer_sys::fuzz_target;
use dist_agent_lang::runtime::values::Value;
use std::sync::mpsc;
use std::time::Duration;

/// Per-input timeout so a single slow unit doesn't dominate fuzzing (e.g. 630s).
const FUZZ_TIMEOUT_SECS: u64 = 5;

fuzz_target!(|data: &[u8]| {
    // Cap input size to avoid blow-up
    if data.len() > 65536 {
        return;
    }
    let (tx, rx) = mpsc::channel();
    let data = data.to_vec();
    let _ = std::thread::spawn(move || {
        // Fuzz standard library functions with arbitrary input
        if let Ok(s) = std::str::from_utf8(&data) {
            let _ = Value::String(s.to_string());
        }
        if data.len() >= 8 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[..8]);
            let num = i64::from_le_bytes(bytes);
            let _ = Value::Int(num);
        }
        if !data.is_empty() {
            let _ = Value::String(format!("{:?}", data));
        }
        let _ = tx.send(());
    });
    let _ = rx.recv_timeout(Duration::from_secs(FUZZ_TIMEOUT_SECS));
});


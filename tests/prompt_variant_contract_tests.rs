use std::fs;
use std::path::Path;

fn resolve_coo_server_path() -> Option<std::path::PathBuf> {
    let candidates = [
        Path::new("COO/server.dal").to_path_buf(),
        Path::new("../COO/server.dal").to_path_buf(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("COO/server.dal"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../COO/server.dal"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

#[test]
fn coo_response_envelopes_include_prompt_variant() {
    let Some(server_path) = resolve_coo_server_path() else {
        return;
    };

    let content = fs::read_to_string(server_path).expect("read COO/server.dal");

    assert!(
        content.contains("\"prompt_variant\": prompt_variant"),
        "expected /api/message response envelope to include prompt_variant",
    );
    assert!(
        content.contains("out[\"prompt_variant\"] = prompt_variant;"),
        "expected /api/task response envelope to include prompt_variant",
    );
    assert!(
        content.contains("\"prompt_variant\": ideas_variant"),
        "expected /api/ideas/generate response envelope to include prompt_variant",
    );
}

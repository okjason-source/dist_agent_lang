//! RAG MVP lexical retrieval tests.

use dist_agent_lang::rag_retrieval::{bm25_scores, should_attempt_rag, tokenize, RagChunk};
use dist_agent_lang::stdlib::rag::prompt_section_prefix;
use std::sync::{Mutex, OnceLock};

fn dal_rag_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn tokenize_splits_words() {
    let t = tokenize("Hello, DAL 2.0!");
    assert!(t.contains(&"hello".to_string()));
    assert!(t.contains(&"dal".to_string()));
}

#[test]
fn should_attempt_rag_respects_env_and_body() {
    let _guard = dal_rag_env_lock()
        .lock()
        .expect("DAL_RAG env mutex poisoned");
    let previous = std::env::var("DAL_RAG").ok();
    std::env::remove_var("DAL_RAG");

    assert!(!should_attempt_rag(None));
    assert!(should_attempt_rag(Some(true)));
    assert!(!should_attempt_rag(Some(false)));

    std::env::set_var("DAL_RAG", "1");
    assert!(should_attempt_rag(None));
    assert!(!should_attempt_rag(Some(false)));

    match previous {
        Some(v) => std::env::set_var("DAL_RAG", v),
        None => std::env::remove_var("DAL_RAG"),
    }
}

#[test]
fn prompt_section_prefix_empty_when_rag_forced_off() {
    let _guard = dal_rag_env_lock()
        .lock()
        .expect("DAL_RAG env mutex poisoned");
    let previous = std::env::var("DAL_RAG").ok();
    std::env::set_var("DAL_RAG", "1");
    let s = prompt_section_prefix("any query text", Some(false));
    assert!(s.is_empty());
    match previous {
        Some(v) => std::env::set_var("DAL_RAG", v),
        None => std::env::remove_var("DAL_RAG"),
    }
}

#[test]
fn bm25_prefers_matching_chunk() {
    let chunks = vec![
        RagChunk {
            id: "a".into(),
            path: "a.md".into(),
            start_line: None,
            text: "the quick brown fox".into(),
        },
        RagChunk {
            id: "b".into(),
            path: "b.md".into(),
            start_line: None,
            text: "unrelated text without query terms".into(),
        },
    ];
    let scores = bm25_scores("quick fox", &chunks);
    assert!(scores[0] > scores[1]);
}

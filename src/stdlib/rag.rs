//! DAL stdlib: lexical RAG prompt augmentation (CEO / `dal serve` apps).
//! See `docs/development/RAG_MVP_SPEC.md` and `crate::rag_retrieval`.

use crate::agent_context_schema::ContextBlock;

/// Text for one `## Context` section, or empty if RAG disabled / no index / no hits.
pub fn prompt_block(query: &str, include_rag: Option<bool>) -> String {
    let blocks = crate::rag_retrieval::rag_context_blocks_for_query(query, include_rag);
    blocks
        .into_iter()
        .find(|b| b.source == "rag")
        .map(|ContextBlock { content, .. }| content)
        .unwrap_or_default()
}

/// Full `## Context (retrieved documentation)` block with trailing `---`, or empty when there is nothing to inject.
/// Used by `workflow::run_steps` and matches CEO `enrich_prompt` / agent RAG wrapping.
pub fn prompt_section_prefix(query: &str, include_rag: Option<bool>) -> String {
    let inner = prompt_block(query, include_rag);
    if inner.is_empty() {
        return String::new();
    }
    format!("## Context (retrieved documentation)\n\n{inner}\n\n---\n\n")
}

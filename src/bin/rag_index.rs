//! Build RAG lexical index (`chunks.jsonl` + `manifest.json`) under `.dal/rag`.
//! Run from repository root: `cargo run --bin rag-index`

fn main() {
    let cwd = std::env::current_dir().expect("cwd");
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    let docs = cwd.join("docs");
    if docs.is_dir() {
        roots.push(docs);
    }
    let ceo_docs = cwd.join("CEO/docs");
    if ceo_docs.is_dir() {
        roots.push(ceo_docs);
    }
    if roots.is_empty() {
        eprintln!("rag-index: no docs/ or CEO/docs/ found from {}", cwd.display());
        std::process::exit(1);
    }
    let out = cwd.join(".dal/rag");
    match dist_agent_lang::rag_retrieval::write_index(&roots, &out) {
        Ok((files, chunks)) => {
            println!(
                "rag-index: {} markdown file(s) -> {} chunk(s) -> {}",
                files,
                chunks,
                out.display()
            );
        }
        Err(e) => {
            eprintln!("rag-index: {}", e);
            std::process::exit(1);
        }
    }
}

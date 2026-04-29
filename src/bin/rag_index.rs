//! Build RAG lexical index (`chunks.jsonl` + `manifest.json`) under `.dal/rag`.
//! Run from repository root: `cargo run --bin rag-index`

fn main() {
    let cwd = std::env::current_dir().expect("cwd");
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    let doc_candidates = [
        cwd.join("docs"),
        cwd.join("COO/docs"),
        cwd.join("../COO/docs"),
    ];
    for candidate in doc_candidates {
        if candidate.is_dir() && !roots.contains(&candidate) {
            roots.push(candidate);
        }
    }
    if roots.is_empty() {
        eprintln!(
            "rag-index: no docs/ or COO/docs/ (or ../COO/docs) found from {}",
            cwd.display()
        );
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

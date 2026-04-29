//! Lexical RAG MVP (BM25) over pre-built `chunks.jsonl`.
//! See docs/development/RAG_MVP_SPEC.md.

use crate::agent_context_schema::ContextBlock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// One chunk line in `chunks.jsonl`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagChunk {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub start_line: Option<u32>,
    pub text: String,
}

static WARNED_MISSING_INDEX: OnceLock<()> = OnceLock::new();
/// Cached successful load: `(resolved_chunks_path, chunks)`.
static CACHED: Mutex<Option<(String, Vec<RagChunk>)>> = Mutex::new(None);

fn index_dir_from_env() -> PathBuf {
    std::env::var("DAL_RAG_INDEX_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".dal/rag"))
}

fn chunks_path() -> PathBuf {
    index_dir_from_env().join("chunks.jsonl")
}

fn top_k_from_env() -> usize {
    std::env::var("DAL_RAG_TOP_K")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|n: &usize| *n > 0 && *n <= 50)
        .unwrap_or(5)
}

/// Tokenize for lexical scoring (lowercase alphanumeric runs).
pub fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() {
            cur.push(c.to_ascii_lowercase());
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn term_freqs(text: &str) -> HashMap<String, u32> {
    let mut m = HashMap::new();
    for t in tokenize(text) {
        *m.entry(t).or_insert(0) += 1;
    }
    m
}

/// Load chunks from disk; cache by resolved path string.
fn load_chunks_cached() -> Option<Vec<RagChunk>> {
    let path = chunks_path();
    if !path.exists() {
        WARNED_MISSING_INDEX.get_or_init(|| {
            log::warn!(
                target: "dal_rag",
                "RAG index not found at {} (set DAL_RAG_INDEX_DIR or run `cargo run --bin rag-index`)",
                path.display()
            );
        });
        return None;
    }
    let key = path.to_string_lossy().to_string();
    let mut guard = CACHED.lock().ok()?;
    if let Some((k, v)) = guard.as_ref() {
        if k == &key {
            return Some(v.clone());
        }
    }
    let file = File::open(&path).ok()?;
    let mut chunks = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(c) = serde_json::from_str::<RagChunk>(&line) {
            if !c.text.trim().is_empty() {
                chunks.push(c);
            }
        }
    }
    log::info!(
        target: "dal_rag",
        "Loaded {} RAG chunks from {}",
        chunks.len(),
        path.display()
    );
    *guard = Some((key, chunks.clone()));
    Some(chunks)
}

/// BM25 scoring (Okapi), corpus = all chunks.
pub fn bm25_scores(query: &str, chunks: &[RagChunk]) -> Vec<f64> {
    let n = chunks.len() as f64;
    if n < 1.0 {
        return vec![0.0; chunks.len()];
    }
    let mut df: HashMap<String, u32> = HashMap::new();
    let mut doc_lens = Vec::with_capacity(chunks.len());
    let mut tfs: Vec<HashMap<String, u32>> = Vec::with_capacity(chunks.len());

    for c in chunks {
        let tf = term_freqs(&c.text);
        for term in tf.keys() {
            *df.entry(term.clone()).or_insert(0) += 1;
        }
        doc_lens.push(c.text.chars().count() as f64);
        tfs.push(tf);
    }

    let avgdl: f64 = doc_lens.iter().sum::<f64>() / n.max(1.0);
    let k1 = 1.2_f64;
    let b = 0.75_f64;

    let qterms = tokenize(query);
    if qterms.is_empty() {
        return vec![0.0; chunks.len()];
    }

    let mut scores = vec![0.0_f64; chunks.len()];
    for (i, tf) in tfs.iter().enumerate() {
        let dl = doc_lens[i];
        for qt in &qterms {
            let Some(f) = tf.get(qt).copied() else {
                continue;
            };
            let dfi = *df.get(qt).unwrap_or(&1) as f64;
            let idf = ((n - dfi + 0.5) / (dfi + 0.5) + 1.0).ln();
            let denom = f as f64 + k1 * (1.0 - b + b * dl / avgdl);
            let num = f as f64 * (k1 + 1.0);
            scores[i] += idf * (num / denom.max(1e-9));
        }
    }
    scores
}

/// Whether this request should attempt RAG (`include_rag` + `DAL_RAG`).
pub fn should_attempt_rag(include_rag: Option<bool>) -> bool {
    match include_rag {
        Some(false) => false,
        Some(true) => true,
        None => std::env::var("DAL_RAG")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
    }
}

/// Build zero or one `ContextBlock` with source `rag`.
pub fn rag_context_blocks_for_query(query: &str, include_rag: Option<bool>) -> Vec<ContextBlock> {
    if !should_attempt_rag(include_rag) {
        return Vec::new();
    }
    let Some(chunks) = load_chunks_cached() else {
        return Vec::new();
    };
    if chunks.is_empty() {
        return Vec::new();
    }
    let k = top_k_from_env();
    let scores = bm25_scores(query, &chunks);
    let mut order: Vec<usize> = (0..chunks.len()).collect();
    order.sort_by(|a, b| {
        scores[*b]
            .partial_cmp(&scores[*a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut picked = Vec::new();
    for idx in order.into_iter().take(k) {
        if scores[idx] > 0.0 {
            picked.push(chunks[idx].clone());
        }
    }
    if picked.is_empty() {
        return Vec::new();
    }

    let mut body = String::from(
        "The following excerpts are retrieved from the project documentation (RAG). They may be truncated. Prefer them over general knowledge when they apply.\n\n",
    );
    for c in picked {
        body.push('[');
        body.push_str(&c.path);
        body.push_str("]\n");
        let excerpt: String = c.text.chars().take(2800).collect();
        body.push_str(&excerpt);
        body.push_str("\n\n");
    }
    vec![ContextBlock {
        source: "rag".to_string(),
        content: body.trim_end().to_string(),
    }]
}

// --- Indexer (used by `dal rag-index` binary) ---

fn skip_dir(name: &std::ffi::OsStr) -> bool {
    name == "target"
        || name == "node_modules"
        || name == ".git"
        || name == "mutants.out"
        || name == "mutants.out.old"
}

/// Collect `*.md` paths under `root`, skipping ignored dirs.
pub fn collect_markdown_files(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if skip_dir(&name) {
            continue;
        }
        if path.is_dir() {
            collect_markdown_files(&path, out)?;
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            out.push(path);
        }
    }
    Ok(())
}

/// Split `text` into chunks (char budget ~3000, overlap ~200).
pub fn chunk_markdown_text(rel_path: &str, text: &str, id_prefix: &str) -> Vec<RagChunk> {
    const MAX_CHARS: usize = 3000;
    const OVERLAP: usize = 200;
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut i = 0_usize;
    let mut part = 0_u32;
    while i < chars.len() {
        let end = (i + MAX_CHARS).min(chars.len());
        let chunk_s: String = chars[i..end].iter().collect();
        if !chunk_s.trim().is_empty() {
            part += 1;
            out.push(RagChunk {
                id: format!("{}:{}:{}", id_prefix, rel_path, part),
                path: rel_path.to_string(),
                start_line: None,
                text: chunk_s,
            });
        }
        if end >= chars.len() {
            break;
        }
        i = end.saturating_sub(OVERLAP);
        if i >= chars.len() {
            break;
        }
    }
    out
}

/// Write `chunks.jsonl` and `manifest.json` under `out_dir`.
pub fn write_index(roots: &[PathBuf], out_dir: &Path) -> Result<(usize, usize), String> {
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    for root in roots {
        if root.is_dir() {
            collect_markdown_files(root, &mut paths).map_err(|e| e.to_string())?;
        }
    }
    paths.sort();
    paths.dedup();
    let n_files = paths.len();

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut all_chunks = Vec::new();
    for p in paths {
        let rel = p
            .strip_prefix(&cwd)
            .unwrap_or(&p)
            .to_string_lossy()
            .to_string();
        let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {}", rel, e))?;
        let id_prefix = rel.replace('/', "_");
        let mut cs = chunk_markdown_text(&rel, &text, &id_prefix);
        all_chunks.append(&mut cs);
    }

    let chunks_path = out_dir.join("chunks.jsonl");
    let mut f = std::fs::File::create(&chunks_path).map_err(|e| e.to_string())?;
    use std::io::Write;
    for c in &all_chunks {
        let line = serde_json::to_string(c).map_err(|e| e.to_string())?;
        f.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
        f.write_all(b"\n").map_err(|e| e.to_string())?;
    }

    let manifest = serde_json::json!({
        "version": 1,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "roots": roots.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
        "chunking": { "max_chars": 3000, "overlap": 200, "engine": "bm25" },
    });
    std::fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .map_err(|e| e.to_string())?;

    Ok((n_files, all_chunks.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;

    static RAG_TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn bm25_scores_match_reference_values_for_simple_corpus() {
        let _lock = RAG_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let chunks = vec![
            RagChunk {
                id: "c1".into(),
                path: "a.md".into(),
                start_line: None,
                text: "alpha alpha beta".into(),
            },
            RagChunk {
                id: "c2".into(),
                path: "b.md".into(),
                start_line: None,
                text: "alpha gamma delta epsilon zeta eta theta iota kappa lambda mu".into(),
            },
            RagChunk {
                id: "c3".into(),
                path: "c.md".into(),
                start_line: None,
                text: "omega".into(),
            },
        ];
        let scores = bm25_scores("alpha", &chunks);
        assert_eq!(scores.len(), 3);
        assert!(
            approx_eq(scores[0], 0.7315673400856836, 1e-12),
            "{scores:?}"
        );
        assert!(
            approx_eq(scores[1], 0.3125272934608578, 1e-12),
            "{scores:?}"
        );
        assert!(approx_eq(scores[2], 0.0, 1e-12), "{scores:?}");
        assert!(scores[0] > scores[1] && scores[1] > scores[2], "{scores:?}");
    }

    #[test]
    fn rag_context_blocks_filters_zero_score_results() {
        let _lock = RAG_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let idx = dir.path().join("chunks.jsonl");
        let mut f = std::fs::File::create(&idx).unwrap();
        let line = serde_json::to_string(&RagChunk {
            id: "x1".into(),
            path: "docs/one.md".into(),
            start_line: None,
            text: "alpha beta".into(),
        })
        .unwrap();
        f.write_all(line.as_bytes()).unwrap();
        f.write_all(b"\n").unwrap();

        std::env::set_var(
            "DAL_RAG_INDEX_DIR",
            dir.path().to_string_lossy().to_string(),
        );
        std::env::set_var("DAL_RAG_TOP_K", "5");

        let none = rag_context_blocks_for_query("query_term_not_present", Some(true));
        assert!(none.is_empty(), "expected no zero-score picks: {none:?}");

        let some = rag_context_blocks_for_query("alpha", Some(true));
        assert_eq!(some.len(), 1);
        assert_eq!(some[0].source, "rag");
        assert!(some[0].content.contains("docs/one.md"));
        assert!(some[0].content.contains("alpha beta"));
    }

    #[test]
    fn rag_context_blocks_honor_explicit_disable_even_with_index_present() {
        let _lock = RAG_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let idx = dir.path().join("chunks.jsonl");
        let mut f = std::fs::File::create(&idx).unwrap();
        let line = serde_json::to_string(&RagChunk {
            id: "x1".into(),
            path: "docs/one.md".into(),
            start_line: None,
            text: "alpha beta".into(),
        })
        .unwrap();
        f.write_all(line.as_bytes()).unwrap();
        f.write_all(b"\n").unwrap();

        std::env::set_var(
            "DAL_RAG_INDEX_DIR",
            dir.path().to_string_lossy().to_string(),
        );
        let out = rag_context_blocks_for_query("alpha", Some(false));
        assert!(out.is_empty(), "include_rag=false must bypass RAG: {out:?}");
    }

    fn count_path_headers(content: &str) -> usize {
        content
            .lines()
            .filter(|l| l.starts_with('[') && l.contains(']'))
            .count()
    }

    #[test]
    fn rag_context_respects_dal_rag_top_k_cap() {
        let _lock = RAG_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let idx = dir.path().join("chunks.jsonl");
        let mut f = std::fs::File::create(&idx).unwrap();
        for i in 0..4 {
            let line = serde_json::to_string(&RagChunk {
                id: format!("id{i}"),
                path: format!("docs/p{i}.md"),
                start_line: None,
                text: "sharedkw sharedkw sharedkw extra words here".into(),
            })
            .unwrap();
            f.write_all(line.as_bytes()).unwrap();
            f.write_all(b"\n").unwrap();
        }
        std::env::set_var(
            "DAL_RAG_INDEX_DIR",
            dir.path().to_string_lossy().to_string(),
        );
        std::env::set_var("DAL_RAG_TOP_K", "2");
        let blocks = rag_context_blocks_for_query("sharedkw", Some(true));
        assert_eq!(blocks.len(), 1);
        let n = count_path_headers(&blocks[0].content);
        assert_eq!(
            n, 2,
            "TOP_K=2 must pick two chunks, not {}",
            blocks[0].content
        );

        std::env::set_var("DAL_RAG_TOP_K", "4");
        let blocks4 = rag_context_blocks_for_query("sharedkw", Some(true));
        assert_eq!(count_path_headers(&blocks4[0].content), 4);
    }

    #[test]
    fn load_chunks_cache_hit_survives_index_file_replace() {
        let _lock = RAG_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let idx = dir.path().join("chunks.jsonl");
        {
            let mut f = std::fs::File::create(&idx).unwrap();
            let line = serde_json::to_string(&RagChunk {
                id: "v1".into(),
                path: "docs/old.md".into(),
                start_line: None,
                text: "keepme_stable_token_xyz".into(),
            })
            .unwrap();
            f.write_all(line.as_bytes()).unwrap();
            f.write_all(b"\n").unwrap();
        }
        std::env::set_var(
            "DAL_RAG_INDEX_DIR",
            dir.path().to_string_lossy().to_string(),
        );
        std::env::set_var("DAL_RAG_TOP_K", "5");

        let first = rag_context_blocks_for_query("keepme_stable_token_xyz", Some(true));
        assert_eq!(first.len(), 1);
        assert!(first[0].content.contains("keepme_stable_token_xyz"));

        let mut f = std::fs::File::create(&idx).unwrap();
        let line = serde_json::to_string(&RagChunk {
            id: "v2".into(),
            path: "docs/new.md".into(),
            start_line: None,
            text: "only_new_world".into(),
        })
        .unwrap();
        f.write_all(line.as_bytes()).unwrap();
        f.write_all(b"\n").unwrap();

        let second = rag_context_blocks_for_query("keepme_stable_token_xyz", Some(true));
        assert_eq!(second.len(), 1);
        assert!(
            second[0].content.contains("keepme_stable_token_xyz"),
            "cached chunks must still serve first index after disk replace: {:?}",
            second[0].content
        );
        assert!(
            !second[0].content.contains("only_new_world"),
            "must not reload from disk while cache key matches"
        );
    }

    #[test]
    fn collect_markdown_files_skips_target_and_vcs_dirs() {
        let _lock = RAG_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(root.join("keep.md"), "# ok").unwrap();
        std::fs::create_dir_all(root.join("target")).unwrap();
        std::fs::write(root.join("target").join("hide.md"), "# no").unwrap();
        std::fs::create_dir_all(root.join("node_modules").join("pkg")).unwrap();
        std::fs::write(root.join("node_modules").join("pkg").join("x.md"), "# no").unwrap();
        std::fs::create_dir_all(root.join(".git").join("hooks")).unwrap();
        std::fs::write(root.join(".git").join("hooks").join("y.md"), "# no").unwrap();
        std::fs::create_dir_all(root.join("mutants.out")).unwrap();
        std::fs::write(root.join("mutants.out").join("z.md"), "# no").unwrap();

        let mut out = Vec::new();
        collect_markdown_files(root, &mut out).unwrap();
        assert_eq!(out.len(), 1);
        assert!(out[0].ends_with("keep.md"));
    }

    #[test]
    fn collect_markdown_files_non_dir_root_is_noop() {
        let _lock = RAG_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("notadir.md");
        std::fs::write(&file, "# x").unwrap();
        let mut out = vec![PathBuf::from("/should/not/appear")];
        collect_markdown_files(&file, &mut out).unwrap();
        assert_eq!(out.len(), 1);
        assert!(out[0].to_string_lossy().contains("should"));
    }

    #[test]
    fn chunk_markdown_text_splits_long_input() {
        let _lock = RAG_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let body = "word ".repeat(800);
        let chunks = chunk_markdown_text("long.md", &body, "pfx");
        assert!(
            chunks.len() >= 2,
            "3500+ chars should yield multiple chunks, got {}",
            chunks.len()
        );
        assert!(chunks.iter().all(|c| c.text.len() <= 3000));
        assert!(chunks[0].id.contains("long.md"));
    }

    #[test]
    fn write_index_returns_file_and_chunk_counts() {
        let _lock = RAG_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docroot");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("a.md"), "# A\nhello").unwrap();
        std::fs::write(docs.join("b.md"), "# B\nworld").unwrap();
        let out_dir = dir.path().join("idx");
        let (n_files, n_chunks) = write_index(&[docs.clone()], &out_dir).unwrap();
        assert_eq!(n_files, 2);
        assert!(
            n_chunks >= 2,
            "two non-empty files must produce at least two chunks, got {}",
            n_chunks
        );
        assert!(out_dir.join("chunks.jsonl").exists());
        assert!(out_dir.join("manifest.json").exists());
        let raw = std::fs::read_to_string(out_dir.join("chunks.jsonl")).unwrap();
        assert_eq!(raw.lines().filter(|l| !l.is_empty()).count(), n_chunks);
    }

    #[test]
    fn top_k_env_invalid_falls_back_to_default_max_five() {
        let _lock = RAG_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let idx = dir.path().join("chunks.jsonl");
        let mut f = std::fs::File::create(&idx).unwrap();
        for i in 0..6 {
            let line = serde_json::to_string(&RagChunk {
                id: format!("id{i}"),
                path: format!("q{i}.md"),
                start_line: None,
                text: "uniqmarker uniqmarker phrase".into(),
            })
            .unwrap();
            f.write_all(line.as_bytes()).unwrap();
            f.write_all(b"\n").unwrap();
        }
        std::env::set_var(
            "DAL_RAG_INDEX_DIR",
            dir.path().to_string_lossy().to_string(),
        );
        std::env::set_var("DAL_RAG_TOP_K", "99");
        let blocks = rag_context_blocks_for_query("uniqmarker", Some(true));
        assert_eq!(blocks.len(), 1);
        let n = count_path_headers(&blocks[0].content);
        assert_eq!(
            n, 5,
            "TOP_K>50 must clamp via filter to default behavior (max 5 picks): {}",
            blocks[0].content
        );
    }
}

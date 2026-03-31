//! Integration tests for `dal_summary` (agent context extraction). Complements unit tests in
//! `src/dal_summary.rs` by exercising the public API from an external crate.

use dist_agent_lang::dal_summary::{summary_from_source, to_context_string, DalSummary};

#[test]
fn summary_end_to_end_agent_imports_and_context_string() {
    let source = r#"
import stdlib::log;
import "./peer.dal" as peer;

agent Worker : ai {} with [ "dal", "http" ] { }

service Api {
    fn ping() { }
}
"#;
    let summary = summary_from_source(source).expect("parse");
    assert_eq!(summary.imports, vec!["stdlib::log", "./peer.dal"]);
    assert_eq!(summary.capabilities, vec!["dal", "http"]);
    assert_eq!(summary.services.len(), 1);
    assert_eq!(summary.services[0].name, "Api");
    assert_eq!(summary.services[0].methods, vec!["ping"]);

    let ctx = to_context_string(&summary);
    assert!(ctx.contains("Imports") && ctx.contains("stdlib::log"));
    assert!(ctx.contains("Capabilities") && ctx.contains("dal"));
    assert!(ctx.contains("Api") && ctx.contains("ping"));
}

#[test]
fn to_context_string_skips_empty_sections() {
    let summary = DalSummary::default();
    assert_eq!(to_context_string(&summary), "");
}

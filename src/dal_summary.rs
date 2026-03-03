//! DAL summary for agent context (P3). Produces a serializable summary from parsed DAL
//! (services, top-level functions, imports, capabilities) for inclusion in the prompt.

use crate::parser::ast::{Program, Statement};
use std::path::Path;

/// One service in the summary: name and method names.
#[derive(Debug, Clone, Default)]
pub struct ServiceSummary {
    pub name: String,
    pub methods: Vec<String>,
}

/// Summary of a DAL program for the agent: project root, entry file, services, functions, imports, capabilities.
#[derive(Debug, Clone, Default)]
pub struct DalSummary {
    /// Project root path (e.g. from cwd or manifest).
    pub project_root: Option<String>,
    /// Entry file path (e.g. main.dal or the file that was summarized).
    pub entry_file: Option<String>,
    /// Services with their method names.
    pub services: Vec<ServiceSummary>,
    /// Top-level function names (export fn / fn at program level).
    pub top_level_functions: Vec<String>,
    /// Import paths (e.g. "stdlib::chain", "./mymod.dal").
    pub imports: Vec<String>,
    /// Capabilities from agent blocks in the AST.
    pub capabilities: Vec<String>,
}

/// Build a DalSummary from a parsed program (single file). Does not resolve imports.
pub fn summary_from_program(program: &Program) -> DalSummary {
    let mut summary = DalSummary::default();
    for stmt in &program.statements {
        match stmt {
            Statement::Service(s) => {
                let methods = s.methods.iter().map(|m| m.name.clone()).collect();
                summary.services.push(ServiceSummary {
                    name: s.name.clone(),
                    methods,
                });
            }
            Statement::Function(f) => {
                summary.top_level_functions.push(f.name.clone());
            }
            Statement::Import(i) => {
                summary.imports.push(i.path.clone());
            }
            Statement::Agent(a) => {
                summary.capabilities.extend(a.capabilities.clone());
            }
            _ => {}
        }
    }
    summary
}

/// Build a DalSummary from source code. Returns parse error as Err.
pub fn summary_from_source(source: &str) -> Result<DalSummary, String> {
    let program = crate::parse_source(source).map_err(|e| e.to_string())?;
    Ok(summary_from_program(&program))
}

/// Build a DalSummary from a file path. Reads the file and parses it.
pub fn summary_from_path(path: &Path) -> Result<DalSummary, String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("Read {}: {}", path.display(), e))?;
    let mut summary = summary_from_source(&source)?;
    summary.entry_file = Some(path.display().to_string());
    if let Some(parent) = path.parent() {
        summary.project_root = Some(parent.display().to_string());
    }
    Ok(summary)
}

/// Render the summary as markdown for inclusion in the agent context (## Context / dal_summary block).
pub fn to_context_string(summary: &DalSummary) -> String {
    let mut out = String::new();
    if let Some(ref root) = summary.project_root {
        out.push_str(&format!("Project root: {}\n", root));
    }
    if let Some(ref entry) = summary.entry_file {
        out.push_str(&format!("Entry file: {}\n", entry));
    }
    if !summary.services.is_empty() {
        out.push_str("\nServices:\n");
        for s in &summary.services {
            out.push_str(&format!(
                "- {} (methods: {})\n",
                s.name,
                s.methods.join(", ")
            ));
        }
    }
    if !summary.top_level_functions.is_empty() {
        out.push_str("\nTop-level functions: ");
        out.push_str(&summary.top_level_functions.join(", "));
        out.push_str("\n");
    }
    if !summary.imports.is_empty() {
        out.push_str("\nImports: ");
        out.push_str(&summary.imports.join(", "));
        out.push_str("\n");
    }
    if !summary.capabilities.is_empty() {
        out.push_str("\nCapabilities (from agent blocks): ");
        out.push_str(&summary.capabilities.join(", "));
        out.push_str("\n");
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_from_empty_source() {
        let summary = summary_from_source("").unwrap();
        assert!(summary.services.is_empty());
        assert!(summary.top_level_functions.is_empty());
        assert!(summary.imports.is_empty());
    }

    #[test]
    fn summary_extracts_service_and_fn() {
        let source = r#"
import stdlib::chain;
fn main() { }
service Foo {
    fn bar() { }
    fn baz() { }
}
"#;
        let summary = summary_from_source(source).unwrap();
        assert_eq!(summary.imports, vec!["stdlib::chain"]);
        assert_eq!(summary.top_level_functions, vec!["main"]);
        assert_eq!(summary.services.len(), 1);
        assert_eq!(summary.services[0].name, "Foo");
        assert_eq!(summary.services[0].methods, vec!["bar", "baz"]);
    }

    #[test]
    fn to_context_string_includes_services_and_fns() {
        let mut summary = DalSummary::default();
        summary.entry_file = Some("main.dal".to_string());
        summary.services.push(ServiceSummary {
            name: "MyService".to_string(),
            methods: vec!["run".to_string()],
        });
        summary.top_level_functions = vec!["main".to_string()];
        let s = to_context_string(&summary);
        assert!(s.contains("Entry file: main.dal"));
        assert!(s.contains("MyService"));
        assert!(s.contains("run"));
        assert!(s.contains("main"));
    }
}

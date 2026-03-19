//! Orchestration: discover DAL projects, run configs, scripts, workflows.
//! Implements the contract from IDE_DAL_ORCHESTRATION.md.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Best-effort guard that a path is contained by root.
/// For existing paths, compares canonical paths to account for symlinks.
/// For non-existing paths, falls back to lexical prefix check.
fn path_is_within_root(root: &Path, path: &Path) -> bool {
    if let (Ok(canon_root), Ok(canon_path)) = (root.canonicalize(), path.canonicalize()) {
        return canon_path.starts_with(&canon_root);
    }
    path.starts_with(root)
}

/// Project info returned by orchestration API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub r#type: String,
    pub root: String,
    pub manifest_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Run config: run script, serve, or test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    pub id: String,
    pub label: String,
    pub r#type: RunConfigType,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunConfigType {
    Run,
    Serve,
    Test,
    Workflow,
    Cargo,
}

/// Script entry (path + label).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptEntry {
    pub path: String,
    pub label: String,
}

/// Workflow definition (optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<String>>,
}

/// Agent/evolve context (optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvolveInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evolve_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<usize>,
}

/// Full orchestration response for a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResponse {
    pub projects: Vec<ProjectInfo>,
    pub run_configs: Vec<RunConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Vec<ScriptEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflows: Option<Vec<WorkflowInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_evolve: Option<AgentEvolveInfo>,
}

/// Discover DAL projects and run configs in a workspace.
/// Uses canonical form of workspace_root for path operations to avoid path traversal.
pub fn discover_workspace(workspace_root: &Path) -> OrchestrationResponse {
    let workspace_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    let workspace_root = workspace_root.as_path();

    let mut projects = Vec::new();
    let mut run_configs = Vec::new();
    let mut scripts = Vec::new();
    let mut workflows = None;
    let mut agent_evolve = None;

    // Find dal.toml at root and in subfolders
    let manifest_paths = find_manifests(workspace_root);

    for manifest_path in manifest_paths {
        let project_root = manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        let project_name = crate::manifest::parse_package_info(&manifest_path)
            .ok()
            .map(|p| p.name);

        let root_str = project_root.to_string_lossy().to_string();
        let manifest_str = manifest_path.to_string_lossy().to_string();

        projects.push(ProjectInfo {
            r#type: "dal".to_string(),
            root: root_str.clone(),
            manifest_path: manifest_str.clone(),
            name: project_name.clone(),
        });

        // Run configs: common entrypoints
        let entrypoints = discover_entrypoints(&project_root);
        for (path, label, is_serve) in entrypoints {
            let rel = path
                .strip_prefix(&project_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            let id = format!("{}:{}", root_str, rel);
            run_configs.push(RunConfig {
                id: id.clone(),
                label: label.clone(),
                r#type: if is_serve {
                    RunConfigType::Serve
                } else {
                    RunConfigType::Run
                },
                command: "dal".to_string(),
                args: Some(if is_serve {
                    vec![
                        "serve".to_string(),
                        rel.clone(),
                        "--port".to_string(),
                        "4040".to_string(),
                    ]
                } else {
                    vec!["run".to_string(), rel.clone()]
                }),
                cwd: Some(root_str.clone()),
            });
            scripts.push(ScriptEntry {
                path: path.to_string_lossy().to_string(),
                label,
            });
        }

        // Test config: one "Run tests" per project (runs `dal test`)
        run_configs.push(RunConfig {
            id: format!("{}:__tests__", root_str),
            label: "Run tests".to_string(),
            r#type: RunConfigType::Test,
            command: "dal".to_string(),
            args: Some(vec!["test".to_string()]),
            cwd: Some(root_str.clone()),
        });

        // Agent/evolve: check agent.toml
        let agent_toml = project_root.join("agent.toml");
        if agent_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&agent_toml) {
                let evolve_path = parse_agent_context_path(&content, &project_root);
                let context_len = evolve_path
                    .as_ref()
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .map(|s| s.len());
                agent_evolve = Some(AgentEvolveInfo {
                    evolve_path,
                    context_length: context_len,
                });
            }
        }

        // Workflows: check for workflows.dal and add run config per workflow
        let workflows_dal = project_root.join("workflows.dal");
        if workflows_dal.exists() {
            if let Some(wfs) = parse_workflows_from_dal(&workflows_dal) {
                workflows = Some(wfs.clone());
                for wf in wfs {
                    run_configs.push(RunConfig {
                        id: format!("workflow:{}:{}", root_str, wf.id),
                        label: format!("Workflow: {}", wf.name),
                        r#type: RunConfigType::Workflow,
                        command: "dal".to_string(),
                        args: Some(vec!["run".to_string(), "workflows.dal".to_string()]),
                        cwd: Some(root_str.clone()),
                    });
                }
            }
        }
    }

    // Cargo projects: discover Cargo.toml at workspace root and in direct subdirs
    let cargo_roots = find_cargo_roots(workspace_root);
    for cargo_root in cargo_roots {
        let root_str = cargo_root.to_string_lossy().to_string();
        for (suffix, label, args) in [
            ("run", "Cargo: run", vec!["run".to_string()]),
            ("test", "Cargo: test", vec!["test".to_string()]),
            ("build", "Cargo: build", vec!["build".to_string()]),
        ] {
            run_configs.push(RunConfig {
                id: format!("cargo:{}:{}", root_str, suffix),
                label: label.to_string(),
                r#type: RunConfigType::Cargo,
                command: "cargo".to_string(),
                args: Some(args),
                cwd: Some(root_str.clone()),
            });
        }
    }

    OrchestrationResponse {
        projects,
        run_configs,
        scripts: Some(scripts),
        workflows,
        agent_evolve,
    }
}

/// Find directories that contain Cargo.toml (workspace root + immediate subdirs).
fn find_cargo_roots(workspace_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let root_cargo = workspace_root.join("Cargo.toml");
    if path_is_within_root(workspace_root, &root_cargo) && root_cargo.exists() {
        out.push(workspace_root.to_path_buf());
    }
    if path_is_within_root(workspace_root, workspace_root) {
        if let Ok(entries) = std::fs::read_dir(workspace_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let manifest = path.join("Cargo.toml");
                    if path_is_within_root(workspace_root, &manifest) && manifest.exists() {
                        out.push(path);
                    }
                }
            }
        }
    }
    out
}

/// Find manifest paths recursively (root + immediate subdirs).
fn find_manifests(workspace_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let root_manifest = workspace_root.join("dal.toml");
    if path_is_within_root(workspace_root, &root_manifest) && root_manifest.exists() {
        out.push(root_manifest);
    }
    if path_is_within_root(workspace_root, workspace_root) {
        if let Ok(entries) = std::fs::read_dir(workspace_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let manifest = path.join("dal.toml");
                    if path_is_within_root(workspace_root, &manifest) && manifest.exists() {
                        out.push(manifest);
                    }
                }
            }
        }
    }
    out
}

/// Discover entrypoint .dal files.
fn discover_entrypoints(project_root: &Path) -> Vec<(PathBuf, String, bool)> {
    let mut out = Vec::new();
    let candidates = [
        ("server.dal", "Serve server.dal", true),
        ("main.dal", "Run main.dal", false),
        ("agent.dal", "Run agent.dal", false),
    ];
    for (name, label, is_serve) in candidates {
        let p = project_root.join(name);
        if p.exists() {
            out.push((p, label.to_string(), is_serve));
        }
    }
    // Also scan for .dal files in root (excluding already discovered)
    let seen: HashSet<&str> = candidates.iter().map(|(n, _, _)| *n).collect();
    if let Ok(entries) = std::fs::read_dir(project_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".dal") && !seen.contains(name) {
                        let label = format!("Run {}", name);
                        out.push((path.clone(), label, false));
                    }
                }
            }
        }
    }
    out
}

/// Parse context_path from agent.toml [agent] section.
fn parse_agent_context_path(content: &str, project_root: &Path) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("context_path") {
            if let Some(v) = line.split('=').nth(1) {
                let path = v.trim().trim_matches('"').trim();
                if !path.is_empty() {
                    let resolved = project_root.join(path);
                    return Some(resolved.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

/// Parse workflow definitions from workflows.dal (simple heuristic).
fn parse_workflows_from_dal(path: &Path) -> Option<Vec<WorkflowInfo>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut workflows = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("// workflow:") || line.starts_with("# workflow:") {
            let rest = line
                .trim_start_matches("// workflow:")
                .trim_start_matches("# workflow:")
                .trim();
            if let Some((name, desc)) = rest.split_once(' ') {
                workflows.push(WorkflowInfo {
                    id: name.to_string().to_lowercase().replace(' ', "_"),
                    name: name.to_string(),
                    description: Some(desc.to_string()),
                    steps: None,
                });
            } else if !rest.is_empty() {
                workflows.push(WorkflowInfo {
                    id: rest.to_lowercase().replace(' ', "_"),
                    name: rest.to_string(),
                    description: None,
                    steps: None,
                });
            }
        }
    }
    if workflows.is_empty() {
        None
    } else {
        Some(workflows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_workspace_empty() {
        let tmp = std::env::temp_dir().join("dal_ide_test_empty");
        let _ = std::fs::create_dir_all(&tmp);
        let resp = discover_workspace(&tmp);
        assert!(resp.projects.is_empty());
        assert!(resp.run_configs.is_empty());
    }
}

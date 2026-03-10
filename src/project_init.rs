//! Project init: run `dal init` logic from code (CLI and agent dal_init tool).
//! Templates: general (main.dal), chain (chain.dal), iot (iot.dal), agent (agent.dal + evolve).

use std::path::Path;

/// Run init for the given template in `dir`. Additive; does not overwrite existing files.
/// Returns a short success message or an error.
/// Templates: "general" | "dal" (main.dal), "chain" (chain.dal), "iot" (iot.dal), "agent" (agent.dal + evolve).
pub fn run_init(template: &str, dir: &Path) -> Result<String, String> {
    let template = template.trim().to_lowercase();
    let template = template.as_str();
    match template {
        "general" | "dal" | "" => run_init_general(dir),
        "chain" => run_init_chain(dir),
        "iot" => run_init_iot(dir),
        "agent" => run_init_agent(dir),
        _ => Err(format!(
            "Unknown template '{}'. Use: general, chain, iot, agent (e.g. dal init --template chain)",
            template
        )),
    }
}

fn write_if_missing(dir: &Path, name: &str, contents: &str) -> std::io::Result<()> {
    let p = dir.join(name);
    if !p.exists() {
        std::fs::write(p, contents)?;
    }
    Ok(())
}

fn ensure_gitignore(dir: &Path) -> std::io::Result<()> {
    let p = dir.join(".gitignore");
    if p.exists() {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if !content.lines().any(|l| l.trim() == ".env") {
                let mut f = std::fs::OpenOptions::new().append(true).open(&p)?;
                use std::io::Write;
                writeln!(f, "\n# Local env (secrets)\n.env")?;
            }
        }
    } else {
        std::fs::write(p, "# Local env (do not commit)\n.env\n")?;
    }
    Ok(())
}

fn run_init_general(dir: &Path) -> Result<String, String> {
    if dir.join("dal.toml").exists() {
        return Err("Project already initialized (dal.toml exists)".to_string());
    }
    let dal_toml = r#"[package]
name = "my-project"
version = "0.1.0"
authors = []

[dependencies]
# Add dependencies here
"#;
    write_if_missing(dir, "dal.toml", dal_toml).map_err(|e| e.to_string())?;

    let main_dal = r#"// Main entry point

fn main() {
    print("Hello from dist_agent_lang!");
}

main();
"#;
    write_if_missing(dir, "main.dal", main_dal).map_err(|e| e.to_string())?;

    write_if_missing(
        dir,
        "README.md",
        "# My DAL Project\n\nA dist_agent_lang project.\n\nSet env as needed (e.g. from `.env`); see `.env.example`.\n",
    )
    .map_err(|e| e.to_string())?;

    write_if_missing(
        dir,
        ".env.example",
        r#"# DAL project env (copy to .env and set values; do not commit .env)
# Optional: for ai:: / assist
# OPENAI_API_KEY=
# ANTHROPIC_API_KEY=
"#,
    )
    .map_err(|e| e.to_string())?;

    write_if_missing(
        dir,
        ".env",
        "# Local env — set values and do not commit (add .env to .gitignore)\n",
    )
    .map_err(|e| e.to_string())?;

    ensure_gitignore(dir).map_err(|e| e.to_string())?;
    Ok("Initialized DAL project (main.dal, dal.toml).".to_string())
}

fn run_init_chain(dir: &Path) -> Result<String, String> {
    if dir.join("dal.toml").exists() {
        return Err("Project already initialized (dal.toml exists)".to_string());
    }
    let dal_toml = r#"[package]
name = "my-chain"
version = "0.1.0"
authors = []

[dependencies]
# Add dependencies here
"#;
    write_if_missing(dir, "dal.toml", dal_toml).map_err(|e| e.to_string())?;

    let chain_dal = r#"// Chain / smart contract entry point

fn main() {
    print("Chain project — add chain:: calls here.");
}

main();
"#;
    write_if_missing(dir, "chain.dal", chain_dal).map_err(|e| e.to_string())?;

    write_if_missing(
        dir,
        "README.md",
        "# Chain DAL Project\n\nBlockchain/smart contract entry: chain.dal.\n",
    )
    .map_err(|e| e.to_string())?;

    write_if_missing(dir, ".env.example", "# Chain project env\n").map_err(|e| e.to_string())?;
    write_if_missing(dir, ".env", "# Local env\n").map_err(|e| e.to_string())?;
    ensure_gitignore(dir).map_err(|e| e.to_string())?;
    Ok("Initialized chain project (chain.dal, dal.toml).".to_string())
}

fn run_init_iot(dir: &Path) -> Result<String, String> {
    if dir.join("dal.toml").exists() {
        return Err("Project already initialized (dal.toml exists)".to_string());
    }
    let dal_toml = r#"[package]
name = "my-iot"
version = "0.1.0"
authors = []

[dependencies]
# Add dependencies here
"#;
    write_if_missing(dir, "dal.toml", dal_toml).map_err(|e| e.to_string())?;

    let iot_dal = r#"// IoT / device entry point

fn main() {
    print("IoT project — add iot device logic here.");
}

main();
"#;
    write_if_missing(dir, "iot.dal", iot_dal).map_err(|e| e.to_string())?;

    write_if_missing(
        dir,
        "README.md",
        "# IoT DAL Project\n\nDevice entry: iot.dal.\n",
    )
    .map_err(|e| e.to_string())?;

    write_if_missing(dir, ".env.example", "# IoT project env\n").map_err(|e| e.to_string())?;
    write_if_missing(dir, ".env", "# Local env\n").map_err(|e| e.to_string())?;
    ensure_gitignore(dir).map_err(|e| e.to_string())?;
    Ok("Initialized IoT project (iot.dal, dal.toml).".to_string())
}

fn run_init_agent(dir: &Path) -> Result<String, String> {
    if !dir.join("dal.toml").exists() {
        let dal_toml = r#"[package]
name = "my-agent"
version = "0.1.0"
authors = []

[dependencies]
"#;
        std::fs::write(dir.join("dal.toml"), dal_toml).map_err(|e| e.to_string())?;
    }

    let agent_toml = r#"# Agent project config (see docs/guides/AGENT_SETUP_AND_USAGE.md)

[agent.sh]
trust = "sandboxed"

# Evolve: wired by default. To disable: comment out the line below.
[agent]
context_path = "./evolve.md"
"#;
    write_if_missing(dir, "agent.toml", agent_toml).map_err(|e| e.to_string())?;

    let agent_dal = r#"// Agent behavior entry (run with: dal run agent.dal or dal agent serve)
// Evolve: context_path in agent.toml. Use sh::run(cmd) for shell (respects [agent.sh] trust).

import stdlib::agent;

fn main() {
    let agent_id = agent::spawn({
        "name": "my-agent",
        "type": "worker",
        "role": "Agent serve"
    });
    agent::set_serve_agent(agent_id);
}

main();
"#;
    write_if_missing(dir, "agent.dal", agent_dal).map_err(|e| e.to_string())?;

    write_if_missing(
        dir,
        "README.md",
        "# Agent project\n\n- agent.dal, agent.toml, evolve.md\n- dal run agent.dal | dal agent serve\n",
    )
    .map_err(|e| e.to_string())?;

    if !dir.join("evolve.md").exists() {
        let header = "# Agent context\n\n## Conversation\n\n## Action log\n\n| Time | Action | Detail | Result |\n|------|--------|--------|--------|\n";
        std::fs::write(dir.join("evolve.md"), header).map_err(|e| e.to_string())?;
    }

    write_if_missing(
        dir,
        ".env.example",
        r#"# Agent project env
DAL_AGENT_SHELL_TRUST=sandboxed
DAL_AGENT_CONTEXT_PATH=./evolve.md
# OPENAI_API_KEY=
# ANTHROPIC_API_KEY=
"#,
    )
    .map_err(|e| e.to_string())?;

    write_if_missing(
        dir,
        ".env",
        "# Local env\nDAL_AGENT_SHELL_TRUST=sandboxed\nDAL_AGENT_CONTEXT_PATH=./evolve.md\n",
    )
    .map_err(|e| e.to_string())?;

    ensure_gitignore(dir).map_err(|e| e.to_string())?;
    Ok("Initialized agent project (agent.dal, agent.toml, evolve.md).".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_init_unknown_template_returns_err() {
        let dir = std::env::temp_dir().join("dal_init_unknown_test");
        let _ = std::fs::create_dir_all(&dir);
        let r = run_init("unknown", &dir);
        assert!(r.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_init_general_creates_dal_toml_and_main_dal() {
        let dir = std::env::temp_dir().join("dal_init_general_test");
        let _ = std::fs::create_dir_all(&dir);
        let r = run_init("general", &dir);
        assert!(r.is_ok());
        assert!(dir.join("dal.toml").exists());
        assert!(dir.join("main.dal").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_init_general_creates_gitignore_with_env() {
        let dir = tempfile::tempdir().unwrap();
        let r = run_init("general", dir.path());
        assert!(r.is_ok());
        let gitignore = dir.path().join(".gitignore");
        assert!(
            gitignore.exists(),
            "ensure_gitignore must create .gitignore"
        );
        let content = std::fs::read_to_string(&gitignore).unwrap();
        assert!(content.contains(".env"), ".gitignore must contain .env");
    }

    #[test]
    fn run_init_chain_creates_chain_dal_and_dal_toml() {
        let dir = tempfile::tempdir().unwrap();
        let r = run_init("chain", dir.path());
        assert!(r.is_ok(), "run_init chain: {:?}", r);
        assert!(dir.path().join("dal.toml").exists());
        assert!(dir.path().join("chain.dal").exists());
        let content = std::fs::read_to_string(dir.path().join("chain.dal")).unwrap();
        assert!(content.contains("Chain") && content.contains("chain::"));
    }

    #[test]
    fn run_init_iot_creates_iot_dal_and_dal_toml() {
        let dir = tempfile::tempdir().unwrap();
        let r = run_init("iot", dir.path());
        assert!(r.is_ok(), "run_init iot: {:?}", r);
        assert!(dir.path().join("dal.toml").exists());
        assert!(dir.path().join("iot.dal").exists());
        let content = std::fs::read_to_string(dir.path().join("iot.dal")).unwrap();
        assert!(content.contains("IoT") && content.contains("iot"));
    }

    #[test]
    fn run_init_agent_creates_agent_dal_evolve_and_agent_toml() {
        let dir = tempfile::tempdir().unwrap();
        let r = run_init("agent", dir.path());
        assert!(r.is_ok(), "run_init agent: {:?}", r);
        assert!(dir.path().join("agent.dal").exists());
        assert!(dir.path().join("agent.toml").exists());
        assert!(dir.path().join("evolve.md").exists());
        let content = std::fs::read_to_string(dir.path().join("agent.dal")).unwrap();
        assert!(content.contains("agent") && content.contains("spawn"));
    }
}

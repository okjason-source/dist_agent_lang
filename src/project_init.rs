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

    let playground_dal = r#"// Language playground: run with `dal run playground.dal`
// Use this file to learn core DAL syntax outside the agent serve flow.

import stdlib::crypto;
import stdlib::agent;

fn greet(name) {
    return "Hello, " + name + "!";
}

fn main() {
    let user = "builder";
    let numbers = [1, 2, 3];
    let profile = {"name": user, "active": true};

    print(greet(user));
    print(numbers);
    print(profile);

    // Tiny stdlib example: deterministic hash.
    let digest = crypto::hash("sha256", "hello dal");
    print("sha256(hello dal): " + digest);

    // Agent on-ramp (commented, optional):
    // let agent_id = agent::spawn({
    //     "name": "playground-agent",
    //     "type": "worker",
    //     "role": "Playground demo"
    // });
    // print("spawned agent id: " + agent_id);

    print("Next: edit this file, then run `dal run playground.dal` again.");
}

main();
"#;
    write_if_missing(dir, "playground.dal", playground_dal).map_err(|e| e.to_string())?;

    write_if_missing(
        dir,
        "README.md",
        "# Agent project\n\n## Minimum working flow\n\n1. Start the behavior script:\n   - `dal run agent.dal`\n2. Start HTTP serving:\n   - `dal agent serve`\n3. Verify health:\n   - `curl -s http://localhost:4040/status`\n4. Send a message:\n   - `curl -s -X POST http://localhost:4040/message -H 'Content-Type: application/json' -d '{\"sender_id\":\"user\",\"content\":\"Run pwd once and reply with the output\",\"policy\":\"tool_loop\"}'`\n\n## Files\n\n- `agent.dal` - minimal serve agent bootstrap\n- `agent.toml` - shell trust + evolve context path\n- `evolve.md` - conversation and action log context\n- `playground.dal` - language learning sandbox (`dal run playground.dal`)\n- `.env` / `.env.example` - runtime env defaults\n",
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
# Host-protocol agent defaults (completion-first with safety guards)
DAL_AGENT_POLICY_DEFAULT=auto
DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=1
DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=0
DAL_AGENT_GUARDS_STRICT_MODE=1
# DAL_AI_ENDPOINT=http://localhost:11434/api/generate
# OPENAI_API_KEY=
# ANTHROPIC_API_KEY=
"#,
    )
    .map_err(|e| e.to_string())?;

    write_if_missing(
        dir,
        ".env",
        "# Local env\nDAL_AGENT_SHELL_TRUST=sandboxed\nDAL_AGENT_CONTEXT_PATH=./evolve.md\nDAL_AGENT_POLICY_DEFAULT=auto\nDAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=1\nDAL_AGENT_ENABLE_LEGACY_TEXT_JSON=0\nDAL_AGENT_GUARDS_STRICT_MODE=1\n",
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
        assert!(dir.path().join("playground.dal").exists());
        let content = std::fs::read_to_string(dir.path().join("agent.dal")).unwrap();
        assert!(content.contains("agent") && content.contains("spawn"));
    }

    #[test]
    fn run_init_agent_readme_includes_minimum_working_flow() {
        let dir = tempfile::tempdir().unwrap();
        let r = run_init("agent", dir.path());
        assert!(r.is_ok(), "run_init agent: {:?}", r);
        let readme = std::fs::read_to_string(dir.path().join("README.md")).unwrap();
        assert!(readme.contains("Minimum working flow"));
        assert!(readme.contains("dal agent serve"));
        assert!(readme.contains("playground.dal"));
    }

    #[test]
    fn run_init_agent_playground_includes_stdlib_onramp() {
        let dir = tempfile::tempdir().unwrap();
        let r = run_init("agent", dir.path());
        assert!(r.is_ok(), "run_init agent: {:?}", r);
        let playground = std::fs::read_to_string(dir.path().join("playground.dal")).unwrap();
        assert!(playground.contains("import stdlib::crypto;"));
        assert!(playground.contains("import stdlib::agent;"));
        assert!(playground.contains("crypto::hash(\"sha256\", \"hello dal\")"));
        assert!(playground.contains("agent::spawn({"));
    }

    #[test]
    fn run_init_agent_env_includes_minimal_host_protocol_profile() {
        let dir = tempfile::tempdir().unwrap();
        let r = run_init("agent", dir.path());
        assert!(r.is_ok(), "run_init agent: {:?}", r);

        let env_example = std::fs::read_to_string(dir.path().join(".env.example")).unwrap();
        assert!(env_example.contains("DAL_AGENT_POLICY_DEFAULT=auto"));
        assert!(env_example.contains("DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=1"));
        assert!(env_example.contains("DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=0"));
        assert!(env_example.contains("DAL_AGENT_GUARDS_STRICT_MODE=1"));

        let env_local = std::fs::read_to_string(dir.path().join(".env")).unwrap();
        assert!(env_local.contains("DAL_AGENT_POLICY_DEFAULT=auto"));
        assert!(env_local.contains("DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=1"));
        assert!(env_local.contains("DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=0"));
        assert!(env_local.contains("DAL_AGENT_GUARDS_STRICT_MODE=1"));
    }
}

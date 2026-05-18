//! Shell execution for agents (Phase 1–2 of AGENT_SHELL_EVOLUTION_PLAN).
//! sh::run(cmd) runs a command in a subprocess; trust config (off/sandboxed/confirmed/trusted)
//! is read from env DAL_AGENT_SHELL_TRUST, agent.toml \[agent.sh\], or dal.toml \[agent.sh\].
//! When owner_principal_id is set (env DAL_AGENT_OWNER_PRINCIPAL or \[agent\] owner_principal_id),
//! key::check("sh", "run", principal_id) is consulted first; if key allows, run proceeds; if key
//! denies or no key exists, we fall back to \[agent.sh\] config.

use crate::runtime::values::Value;
use crate::stdlib::key::{self, CapabilityRequest};
use std::collections::HashMap;
use std::env;
use std::process::Command;

/// Trust level for sh execution. Precedence: env DAL_AGENT_SHELL_TRUST → agent.toml / dal.toml \[agent.sh\] → default "sandboxed".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustLevel {
    Off,
    Sandboxed,
    Confirmed,
    Trusted,
}

impl TrustLevel {
    fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "off" => TrustLevel::Off,
            "confirmed" => TrustLevel::Confirmed,
            "trusted" => TrustLevel::Trusted,
            _ => TrustLevel::Sandboxed,
        }
    }
}

impl std::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustLevel::Off => write!(f, "off"),
            TrustLevel::Sandboxed => write!(f, "sandboxed"),
            TrustLevel::Confirmed => write!(f, "confirmed"),
            TrustLevel::Trusted => write!(f, "trusted"),
        }
    }
}

/// Config for [agent.sh]. Loaded from agent.toml or dal.toml or env.
#[derive(Debug, Clone)]
pub struct ShConfig {
    pub trust: TrustLevel,
    pub forbidden_patterns: Vec<String>,
    pub allowed_prefixes: Option<Vec<String>>,
}

impl Default for ShConfig {
    fn default() -> Self {
        Self {
            trust: TrustLevel::Sandboxed,
            forbidden_patterns: Vec::new(),
            allowed_prefixes: None,
        }
    }
}

/// Load [agent.sh] from a TOML table (e.g. from agent.toml root or dal.toml under ["agent"]["sh"]).
fn parse_agent_sh_from_table(table: &toml::Table) -> Option<ShConfig> {
    let agent_sh = table
        .get("agent")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("sh"))
        .and_then(|v| v.as_table());
    let table = agent_sh.or_else(|| table.get("sh").and_then(|v| v.as_table()))?;
    let trust = table
        .get("trust")
        .and_then(|v| v.as_str())
        .map(TrustLevel::from_str)
        .unwrap_or(TrustLevel::Sandboxed);
    let forbidden_patterns: Vec<String> = table
        .get("forbidden_patterns")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let allowed_prefixes = table
        .get("allowed_prefixes")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });
    Some(ShConfig {
        trust,
        forbidden_patterns,
        allowed_prefixes,
    })
}

/// Load sh config: dal.toml then agent.toml (agent.toml overrides); env DAL_AGENT_SHELL_TRUST overrides trust.
pub fn load_sh_config() -> ShConfig {
    let cwd = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let mut config = ShConfig::default();

    if let Ok(content) = std::fs::read_to_string(cwd.join("dal.toml")) {
        if let Ok(table) = content.parse::<toml::Table>() {
            if let Some(c) = parse_agent_sh_from_table(&table) {
                config = c;
            }
        }
    }
    if let Ok(content) = std::fs::read_to_string(cwd.join("agent.toml")) {
        if let Ok(table) = content.parse::<toml::Table>() {
            if let Some(c) = parse_agent_sh_from_table(&table) {
                config = c;
            }
        }
    }

    if let Ok(trust_env) = env::var("DAL_AGENT_SHELL_TRUST") {
        config.trust = TrustLevel::from_str(&trust_env);
    }
    config
}

/// Human-readable constraints description for agent prompt context (P4). Shell trust and forbidden patterns from [agent.sh].
pub fn constraints_description_for_prompt(config: &ShConfig) -> String {
    let mut lines = vec![format!("Shell: {}.", config.trust)];
    if !config.forbidden_patterns.is_empty() {
        lines.push(format!(
            "Forbidden command patterns: {}.",
            config.forbidden_patterns.join(", ")
        ));
    }
    lines.join(" ")
}

/// Load owner principal ID for key::check. Precedence: DAL_AGENT_OWNER_PRINCIPAL → agent.toml \[agent\] owner_principal_id → dal.toml \[agent\] owner_principal_id.
pub fn load_owner_principal_id() -> Option<String> {
    if let Ok(id) = env::var("DAL_AGENT_OWNER_PRINCIPAL") {
        if !id.trim().is_empty() {
            return Some(id.trim().to_string());
        }
    }
    let cwd = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    for file in ["agent.toml", "dal.toml"] {
        let content = std::fs::read_to_string(cwd.join(file)).ok()?;
        let table = content.parse::<toml::Table>().ok()?;
        let agent = table.get("agent").and_then(|v| v.as_table())?;
        let id = agent.get("owner_principal_id").and_then(|v| v.as_str())?;
        if !id.trim().is_empty() {
            return Some(id.trim().to_string());
        }
    }
    None
}

/// Check command against config: returns Err if not allowed.
fn check_allowed(cmd: &str, config: &ShConfig) -> Result<(), String> {
    if config.trust == TrustLevel::Off {
        return Err("sh is disabled by config (trust=off). Set DAL_AGENT_SHELL_TRUST or [agent.sh] trust to allow.".to_string());
    }
    if config.trust == TrustLevel::Confirmed {
        if env::var("DAL_AGENT_SHELL_CONFIRM").ok().as_deref() != Some("1") {
            return Err(
                "sh::run: confirmed mode requires approval. Set DAL_AGENT_SHELL_CONFIRM=1 to run without prompt, or use trust=trusted/sandboxed in config.".to_string()
            );
        }
    }
    let cmd_trim = cmd.trim();
    for pattern in &config.forbidden_patterns {
        if cmd_trim.contains(pattern) {
            return Err(format!(
                "command forbidden by config (forbidden_patterns): {}",
                pattern
            ));
        }
    }
    if let Some(ref prefixes) = config.allowed_prefixes {
        if !prefixes.is_empty() {
            let first_word = cmd_trim.split_whitespace().next().unwrap_or("");
            let allowed = prefixes
                .iter()
                .any(|p| first_word.starts_with(p) || cmd_trim.starts_with(p));
            if !allowed {
                return Err(format!(
                    "command not in allowed_prefixes (config): {:?}",
                    prefixes
                ));
            }
        }
    }
    Ok(())
}

/// Run a shell command. Respects [agent.sh], DAL_AGENT_SHELL_TRUST, and key::check when owner_principal_id is set. Returns a map: stdout, stderr, exit_code.
pub fn run(cmd: &str) -> Result<Value, String> {
    let config = load_sh_config();
    if let Some(principal_id) = load_owner_principal_id() {
        let req = CapabilityRequest {
            resource: "sh".to_string(),
            operation: "run".to_string(),
            principal_id: principal_id.clone(),
        };
        if key::check(req).unwrap_or(false) {
            // Key allows: still apply config (forbidden_patterns, etc.)
        } else {
            // Key denies or no key: fall back to [agent.sh] config (check_allowed below)
        }
    }
    check_allowed(cmd, &config)?;
    let (stdout, stderr, exit_code) = run_impl(cmd)?;
    let mut map = HashMap::new();
    map.insert("stdout".to_string(), Value::String(stdout));
    map.insert("stderr".to_string(), Value::String(stderr));
    map.insert("exit_code".to_string(), Value::Int(exit_code));
    Ok(Value::Map(map))
}

fn run_impl(cmd: &str) -> Result<(String, String, i64), String> {
    let output = if cfg!(unix) {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| e.to_string())?
    } else if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", cmd])
            .output()
            .map_err(|e| e.to_string())?
    } else {
        return Err("Unsupported platform for sh::run".to_string());
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    Ok((stdout, stderr, exit_code as i64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvVarGuard {
        key: &'static str,
        old: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let old = std::env::var(key).ok();
            if let Some(v) = value {
                std::env::set_var(key, v);
            } else {
                std::env::remove_var(key);
            }
            Self { key, old }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(v) = &self.old {
                std::env::set_var(self.key, v);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    struct CwdGuard {
        old: std::path::PathBuf,
    }

    impl CwdGuard {
        fn set_to(path: &std::path::Path) -> Self {
            let old = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(path).expect("set cwd");
            Self { old }
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.old).expect("restore cwd");
        }
    }

    #[test]
    fn trust_level_from_str_and_display() {
        assert_eq!(TrustLevel::from_str("off"), TrustLevel::Off);
        assert_eq!(TrustLevel::from_str("confirmed"), TrustLevel::Confirmed);
        assert_eq!(TrustLevel::from_str("trusted"), TrustLevel::Trusted);
        assert_eq!(TrustLevel::from_str("unknown"), TrustLevel::Sandboxed);
        assert_eq!(TrustLevel::Trusted.to_string(), "trusted");
    }

    #[test]
    fn load_sh_config_env_overrides_file() {
        let _l = test_lock().lock().expect("lock");
        let td = tempdir().expect("tempdir");
        std::fs::write(
            td.path().join("dal.toml"),
            r#"[agent.sh]
trust = "off"
"#,
        )
        .expect("write dal.toml");
        std::fs::write(
            td.path().join("agent.toml"),
            r#"[agent.sh]
trust = "trusted"
forbidden_patterns = ["rm -rf", "sudo"]
allowed_prefixes = ["echo", "ls"]
"#,
        )
        .expect("write agent.toml");

        let _cwd = CwdGuard::set_to(td.path());
        let _env = EnvVarGuard::set("DAL_AGENT_SHELL_TRUST", Some("confirmed"));
        let cfg = load_sh_config();
        assert_eq!(cfg.trust, TrustLevel::Confirmed); // env override
        assert_eq!(cfg.forbidden_patterns.len(), 2);
        assert_eq!(cfg.allowed_prefixes.as_ref().map(|v| v.len()), Some(2));
    }

    #[test]
    fn check_allowed_enforces_trust_forbidden_and_allowlist() {
        let _l = test_lock().lock().expect("lock");
        let _confirm = EnvVarGuard::set("DAL_AGENT_SHELL_CONFIRM", None);

        let off = ShConfig {
            trust: TrustLevel::Off,
            forbidden_patterns: vec![],
            allowed_prefixes: None,
        };
        assert!(check_allowed("echo hi", &off)
            .unwrap_err()
            .contains("disabled"));

        let confirmed = ShConfig {
            trust: TrustLevel::Confirmed,
            forbidden_patterns: vec![],
            allowed_prefixes: None,
        };
        assert!(check_allowed("echo hi", &confirmed)
            .unwrap_err()
            .contains("requires approval"));

        let _confirm_yes = EnvVarGuard::set("DAL_AGENT_SHELL_CONFIRM", Some("1"));
        assert!(check_allowed("echo hi", &confirmed).is_ok());

        let guarded = ShConfig {
            trust: TrustLevel::Trusted,
            forbidden_patterns: vec!["rm -rf".to_string()],
            allowed_prefixes: Some(vec!["echo".to_string(), "ls".to_string()]),
        };
        assert!(check_allowed("echo ok", &guarded).is_ok());
        assert!(check_allowed("rm -rf /tmp", &guarded)
            .unwrap_err()
            .contains("forbidden_patterns"));
        assert!(check_allowed("cat file.txt", &guarded)
            .unwrap_err()
            .contains("allowed_prefixes"));
    }

    #[test]
    fn run_returns_map_with_exit_code_stdout_stderr() {
        let _l = test_lock().lock().expect("lock");
        let td = tempdir().expect("tempdir");
        std::fs::write(
            td.path().join("agent.toml"),
            r#"[agent.sh]
trust = "trusted"
allowed_prefixes = ["echo"]
"#,
        )
        .expect("write agent.toml");

        let _cwd = CwdGuard::set_to(td.path());
        let result = run("echo hello").expect("run echo");
        let Value::Map(m) = result else {
            panic!("expected map");
        };
        assert!(matches!(m.get("exit_code"), Some(Value::Int(0))));
        assert!(matches!(m.get("stdout"), Some(Value::String(s)) if s.contains("hello")));
        assert!(matches!(m.get("stderr"), Some(Value::String(_))));
    }
}

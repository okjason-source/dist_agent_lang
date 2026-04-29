//! Run backend: spawn dal run/serve, stream output.
//! Used by IDE UI and agent API.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;
use tokio::sync::{broadcast, oneshot};

/// Request to run a config by id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRequest {
    pub config_id: String,
}

/// Request to run a CLI command (agent API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCommandRequest {
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

/// Request to write a file (agent API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileRequest {
    pub path: String,
    pub contents: String,
    /// Optional workspace root (relative or absolute). If set, path is resolved relative to this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

/// Request to read a file (agent API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileRequest {
    pub path: String,
    /// Optional workspace root (relative or absolute). If set, path is resolved relative to this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

/// Response: job started, returns job_id for streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStartedResponse {
    pub job_id: String,
    pub config_id: String,
}

/// Response: command output (stdout + stderr combined).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunOutputResponse {
    pub job_id: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Resolve config_id to (command, args, cwd).
/// Format: "root_path:relative_script.dal" for run configs.
pub fn resolve_run_config(
    config_id: &str,
    run_configs: &[crate::ide::orchestration::RunConfig],
) -> Option<(String, Vec<String>, String)> {
    for cfg in run_configs {
        if cfg.id == config_id {
            let cmd = cfg.command.clone();
            let args = cfg.args.clone().unwrap_or_default();
            let cwd = cfg.cwd.clone().unwrap_or_else(|| ".".to_string());
            return Some((cmd, args, cwd));
        }
    }
    None
}

/// Run a command and return (stdout, stderr, exit_code).
pub fn run_command_blocking(
    cmd: &str,
    args: &[String],
    cwd: Option<&Path>,
) -> Result<(String, String, i32), String> {
    let mut child = Command::new(cmd)
        .args(args)
        .current_dir(cwd.unwrap_or(Path::new(".")))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout not captured".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "stderr not captured".to_string())?;

    let out = std::io::read_to_string(std::io::BufReader::new(stdout))
        .map_err(|e| format!("Failed to read stdout: {}", e))?;
    let err = std::io::read_to_string(std::io::BufReader::new(stderr))
        .map_err(|e| format!("Failed to read stderr: {}", e))?;

    let status = child.wait().map_err(|e| format!("Failed to wait: {}", e))?;
    let code = status.code().unwrap_or(-1);
    Ok((out, err, code))
}

/// Spawn a command and stream output. Returns (output_tx, kill_tx).
/// Caller stores these; when kill_tx is sent, the process is killed.
pub fn spawn_run_streaming(
    cmd: &str,
    args: &[String],
    cwd: Option<&Path>,
) -> Result<(broadcast::Sender<String>, oneshot::Sender<()>), String> {
    let mut child = AsyncCommand::new(cmd)
        .args(args)
        .current_dir(cwd.unwrap_or(Path::new(".")))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout not captured".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "stderr not captured".to_string())?;

    let (output_tx, _) = broadcast::channel::<String>(64);
    let (kill_tx, kill_rx) = oneshot::channel::<()>();

    let tx_out = output_tx.clone();
    let tx_err = output_tx.clone();
    let tx_done = output_tx.clone();

    tokio::spawn(async move {
        let h_stdout = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx_out.send(format!("{}\n", line));
            }
        });

        let h_stderr = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx_err.send(format!("[stderr] {}\n", line));
            }
        });

        let mut cancelled = false;
        let mut exit_code: i32 = 0;
        tokio::select! {
            _ = kill_rx => {
                cancelled = true;
                let _ = child.kill().await;
                if let Ok(status) = child.wait().await {
                    exit_code = status.code().unwrap_or(-1);
                }
            }
            _ = async {
                let _ = tokio::join!(h_stdout, h_stderr);
                if let Ok(status) = child.wait().await {
                    exit_code = status.code().unwrap_or(-1);
                }
            } => {}
        }

        if cancelled {
            let _ = tx_done.send("[CANCELLED]".to_string());
        } else if exit_code != 0 {
            let _ = tx_done.send(format!("[ERROR:{}]", exit_code));
        }

        // Send done marker
        let _ = tx_done.send("[DONE]".to_string());
    });

    Ok((output_tx, kill_tx))
}

/// Get the path to the dal binary (current executable).
pub fn dal_binary_path() -> Result<std::path::PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))
}

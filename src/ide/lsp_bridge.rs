//! WebSocket bridge for `dal lsp` stdio. Allows the browser to use full LSP (monaco-languageclient)
//! by connecting to GET /api/lsp/stream, which spawns `dal lsp` and forwards LSP messages.

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use tokio::sync::mpsc as tokio_mpsc;

/// Read one LSP message (Content-Length: n \r\n \r\n body) from reader.
fn read_lsp_message<R: Read>(reader: &mut BufReader<R>) -> Option<String> {
    let mut header = String::new();
    if reader.read_line(&mut header).ok()? == 0 {
        return None;
    }
    let mut content_length: Option<usize> = None;
    loop {
        let line = header.trim_end_matches("\r\n").trim_end_matches('\n');
        if line.is_empty() {
            break;
        }
        if let Some(stripped) = line.strip_prefix("Content-Length:") {
            if let Ok(n) = stripped.trim().parse::<usize>() {
                content_length = Some(n);
            }
        }
        header.clear();
        if reader.read_line(&mut header).ok()? == 0 {
            return None;
        }
    }
    let n = content_length?;
    let mut buf = vec![0u8; n];
    let mut read = 0;
    while read < n {
        let got = reader.get_mut().read(&mut buf[read..]).ok()?;
        if got == 0 {
            return None;
        }
        read += got;
    }
    String::from_utf8(buf).ok()
}

/// Write one LSP message to writer.
fn write_lsp_message<W: Write>(writer: &mut W, body: &str) -> std::io::Result<()> {
    let bytes = body.as_bytes();
    write!(writer, "Content-Length: {}\r\n\r\n", bytes.len())?;
    writer.write_all(bytes)?;
    writer.flush()?;
    Ok(())
}

/// Spawn `dal lsp` with workspace as cwd. Returns (child, stdout_reader) for bridging.
fn spawn_dal_lsp(workspace_root: &Path) -> Option<(Child, BufReader<std::process::ChildStdout>)> {
    let mut cmd = Command::new("dal");
    cmd.arg("lsp")
        .current_dir(workspace_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = cmd.spawn().ok()?;
    let stdout = child.stdout.take()?;
    let reader = BufReader::new(stdout);
    Some((child, reader))
}

/// Run the LSP bridge: spawn `dal lsp`, forward between WebSocket and process stdio.
/// WebSocket sends/receives raw JSON-RPC body (no Content-Length header); we add/strip header for stdio.
pub async fn run_lsp_bridge(socket: WebSocket, workspace_root: std::path::PathBuf) {
    if let Some((mut child, mut stdout_reader)) = spawn_dal_lsp(&workspace_root) {
        let mut stdin = match child.stdin.take() {
            Some(s) => s,
            None => return,
        };
        let (tx_out, mut rx_out) = tokio_mpsc::unbounded_channel::<String>();
        let (tx_in, rx_in) = mpsc::sync_channel::<String>(64);

        // Thread: read from LSP stdout, send body to async via tx_out
        thread::spawn(move || {
            while let Some(body) = read_lsp_message(&mut stdout_reader) {
                if tx_out.send(body).is_err() {
                    break;
                }
            }
        });

        // Thread: receive from async via rx_in, write to LSP stdin
        thread::spawn(move || {
            while let Ok(body) = rx_in.recv() {
                if write_lsp_message(&mut stdin, &body).is_err() {
                    break;
                }
            }
        });

        // Async: forward between WebSocket and channels
        let (mut ws_sender, mut ws_receiver) = socket.split();
        loop {
            tokio::select! {
                Some(body) = rx_out.recv() => {
                    if ws_sender.send(Message::Text(body)).await.is_err() {
                        break;
                    }
                }
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            let _ = tx_in.send(text);
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        _ => {}
                    }
                }
            }
        }
        let _ = child.kill();
    }
}

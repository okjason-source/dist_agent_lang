//! Second LSP client (e.g. rust-analyzer) over stdio for mixed-language IDE.
//! Used when the frontend requests diagnostics/hover/completion for non-DAL files (e.g. .rs).

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Convert a path to a file:// URI for LSP.
pub fn path_to_file_uri(path: &std::path::Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let s = path.to_string_lossy();
    #[cfg(windows)]
    let s = format!("file:///{}", s.replace('\\', "/"));
    #[cfg(not(windows))]
    let s = format!("file://{}", s);
    s
}

/// LSP request to send to the worker thread.
pub enum LspRequest {
    Diagnostics {
        uri: String,
        text: String,
    },
    Hover {
        uri: String,
        text: String,
        line: u32,
        character: u32,
    },
    Completion {
        uri: String,
        text: String,
        line: u32,
        character: u32,
    },
}

/// LSP response from the worker thread.
pub enum LspResponse {
    Diagnostics(Vec<LspDiagnostic>),
    Hover(Option<String>),
    Completion(Vec<LspCompletionItem>),
}

#[derive(Debug, Clone)]
pub struct LspDiagnostic {
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct LspCompletionItem {
    pub label: String,
    pub kind: Option<u8>,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
}

/// Read one LSP message from stdout (Content-Length: n \r\n \r\n body).
fn read_lsp_message(reader: &mut BufReader<&mut ChildStdout>) -> Option<String> {
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

/// Write one LSP message to stdin.
fn write_lsp_message(stdin: &mut ChildStdin, body: &str) -> std::io::Result<()> {
    let bytes = body.as_bytes();
    write!(stdin, "Content-Length: {}\r\n\r\n", bytes.len())?;
    stdin.write_all(bytes)?;
    stdin.flush()?;
    Ok(())
}

/// Parse LSP JSON-RPC response by id; returns the result field or error.
fn parse_response(json: &str, id: u64) -> Option<Value> {
    let v: Value = serde_json::from_str(json).ok()?;
    if v.get("id")?.as_u64()? != id {
        return None;
    }
    if v.get("error").is_some() {
        return None;
    }
    v.get("result").cloned()
}

/// Run the LSP worker thread: owns the process and handles requests.
fn run_lsp_worker(
    mut child: Child,
    rx: mpsc::Receiver<(LspRequest, mpsc::Sender<Option<LspResponse>>)>,
    root_uri: String,
) {
    let mut stdin = child.stdin.take().expect("stdin");
    let mut stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(&mut stdout);

    let mut next_id: u64 = 1;

    let init_params = json!({
        "processId": std::process::id(),
        "rootUri": root_uri,
        "capabilities": {},
        "clientInfo": { "name": "dal-ide", "version": "0.1.0" }
    });
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": next_id,
        "method": "initialize",
        "params": init_params
    });
    next_id += 1;
    if write_lsp_message(&mut stdin, &init_req.to_string()).is_err() {
        return;
    }
    while let Some(m) = read_lsp_message(&mut reader) {
        let msg = m;
        if parse_response(&msg, next_id - 1).is_some() {
            break; // initialize response received
        }
        // might be a notification; ignore
    }

    // Send initialized notification
    let initialized = json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} });
    let _ = write_lsp_message(&mut stdin, &initialized.to_string());

    // Request loop
    while let Ok((req, reply_tx)) = rx.recv() {
        let response = handle_one_request(&mut stdin, &mut reader, &mut next_id, req);
        let _ = reply_tx.send(response);
    }
}

fn handle_one_request(
    stdin: &mut ChildStdin,
    reader: &mut BufReader<&mut ChildStdout>,
    next_id: &mut u64,
    req: LspRequest,
) -> Option<LspResponse> {
    let (method, uri, text, line, character) = match &req {
        LspRequest::Diagnostics { uri, text } => (
            "textDocument/diagnostic",
            uri.clone(),
            text.clone(),
            0u32,
            0u32,
        ),
        LspRequest::Hover {
            uri,
            text,
            line,
            character,
        } => (
            "textDocument/hover",
            uri.clone(),
            text.clone(),
            *line,
            *character,
        ),
        LspRequest::Completion {
            uri,
            text,
            line,
            character,
        } => (
            "textDocument/completion",
            uri.clone(),
            text.clone(),
            *line,
            *character,
        ),
    };

    // didOpen (sync document so server has current content)
    let did_open = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "rust",
                "version": 1,
                "text": text
            }
        }
    });
    if write_lsp_message(stdin, &did_open.to_string()).is_err() {
        return None;
    }

    let id = *next_id;
    *next_id = next_id.saturating_add(1);

    let method_params = match &req {
        LspRequest::Diagnostics { .. } => json!({ "textDocument": { "uri": uri } }),
        LspRequest::Hover { .. } | LspRequest::Completion { .. } => json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }),
    };

    let lsp_req = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": method_params
    });
    if write_lsp_message(stdin, &lsp_req.to_string()).is_err() {
        return None;
    }

    // Read until we get our response (skip notifications like publishDiagnostics)
    loop {
        let msg = read_lsp_message(reader)?;
        if let Some(result) = parse_response(&msg, id) {
            return match &req {
                LspRequest::Diagnostics { .. } => {
                    let items = result
                        .get("items")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|d| {
                                    let range = d.get("range")?;
                                    let start = range.get("start")?;
                                    let end = range.get("end")?;
                                    let sev =
                                        d.get("severity").and_then(Value::as_u64).unwrap_or(1);
                                    let severity =
                                        if sev == 2 { "warning" } else { "error" }.to_string();
                                    Some(LspDiagnostic {
                                        line: start.get("line")?.as_u64()? as u32,
                                        column: start.get("character")?.as_u64()? as u32,
                                        end_line: end.get("line")?.as_u64()? as u32,
                                        end_column: end.get("character")?.as_u64()? as u32,
                                        message: d
                                            .get("message")?
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                        severity,
                                    })
                                })
                                .collect::<Vec<LspDiagnostic>>()
                        })
                        .unwrap_or_default();
                    Some(LspResponse::Diagnostics(items))
                }
                LspRequest::Hover { .. } => {
                    let contents = result.get("contents").and_then(|c| {
                        if let Some(s) = c.as_str() {
                            return Some(s.to_string());
                        }
                        c.get("value").and_then(|v| v.as_str().map(String::from))
                    });
                    Some(LspResponse::Hover(contents))
                }
                LspRequest::Completion { .. } => {
                    let items = result
                        .get("items")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|item| {
                                    Some(LspCompletionItem {
                                        label: item
                                            .get("label")?
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                        kind: item
                                            .get("kind")
                                            .and_then(Value::as_u64)
                                            .map(|k| k as u8),
                                        detail: item
                                            .get("detail")
                                            .and_then(Value::as_str)
                                            .map(String::from),
                                        insert_text: item
                                            .get("insertText")
                                            .and_then(Value::as_str)
                                            .map(String::from),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    Some(LspResponse::Completion(items))
                }
            };
        }
        // Skip notifications (e.g. publishDiagnostics)
    }
}

/// Optional second LSP (e.g. rust-analyzer). Created on first use; runs in a dedicated thread.
pub struct SecondLsp {
    tx: mpsc::Sender<(LspRequest, mpsc::Sender<Option<LspResponse>>)>,
}

impl SecondLsp {
    /// Try to create a second LSP process (e.g. rust-analyzer) with the given workspace root.
    /// Returns None if the binary is not found or spawn fails.
    pub fn spawn(workspace_root: &std::path::Path) -> Option<Self> {
        let cwd = workspace_root.to_path_buf();
        let root_uri = path_to_file_uri(&cwd);
        let mut cmd = Command::new("rust-analyzer");
        cmd.current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let child = cmd.spawn().ok()?;
        let (req_tx, req_rx) = mpsc::channel();
        let _handle = thread::spawn(move || run_lsp_worker(child, req_rx, root_uri));
        Some(Self { tx: req_tx })
    }

    /// Request diagnostics for a document. Blocks until response or timeout.
    pub fn request_diagnostics(&self, uri: String, text: String) -> Option<Vec<LspDiagnostic>> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.tx
            .send((LspRequest::Diagnostics { uri, text }, reply_tx))
            .ok()?;
        match reply_rx.recv_timeout(Duration::from_secs(10)) {
            Ok(Some(LspResponse::Diagnostics(d))) => Some(d),
            _ => None,
        }
    }

    /// Request hover at position. Blocks until response or timeout.
    pub fn request_hover(
        &self,
        uri: String,
        text: String,
        line: u32,
        character: u32,
    ) -> Option<String> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.tx
            .send((
                LspRequest::Hover {
                    uri,
                    text,
                    line,
                    character,
                },
                reply_tx,
            ))
            .ok()?;
        match reply_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Some(LspResponse::Hover(s))) => s,
            _ => None,
        }
    }

    /// Request completion at position. Blocks until response or timeout.
    pub fn request_completion(
        &self,
        uri: String,
        text: String,
        line: u32,
        character: u32,
    ) -> Option<Vec<LspCompletionItem>> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.tx
            .send((
                LspRequest::Completion {
                    uri,
                    text,
                    line,
                    character,
                },
                reply_tx,
            ))
            .ok()?;
        match reply_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Some(LspResponse::Completion(items))) => Some(items),
            _ => None,
        }
    }
}

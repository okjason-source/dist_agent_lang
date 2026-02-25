//! LSP (Language Server Protocol) server for DAL — Phase 1–3: diagnostics, hover, completion, definition, signature help.
//! See docs/development/LSP_AND_AGENT_INTEGRATION_PLAN.md.

#![cfg(feature = "lsp")]

use dist_agent_lang::lexer::Lexer;
use dist_agent_lang::parser::ast::Statement;
use dist_agent_lang::parser::Parser;
use lsp_types::{
    CompletionItem, CompletionOptions, CompletionParams, CompletionResponse, Diagnostic,
    DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams,
    Location, OneOf, ParameterInformation, Position, Range, ServerCapabilities, SignatureHelp,
    SignatureHelpOptions, SignatureHelpParams, SignatureInformation, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, Url,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::{async_trait, Client, LanguageServer, LspService, Server};

#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
}

#[derive(Debug, Clone)]
struct DocumentState {
    text: String,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Convert 1-based line/column (DAL) to 0-based LSP Position.
    fn to_lsp_position(line: usize, column: usize) -> Position {
        Position {
            line: line.saturating_sub(1) as u32,
            character: column.saturating_sub(1) as u32,
        }
    }

    /// Build LSP diagnostics from source: run lexer and parser, map errors to Diagnostic.
    fn diagnostics_from_source(source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // Lexer
        let lexer = Lexer::new(source);
        match lexer.tokenize_with_positions_immutable() {
            Err(e) => {
                let (line, col) = e.line_column().unwrap_or((1, 1));
                diags.push(Diagnostic {
                    range: Range {
                        start: Self::to_lsp_position(line, col),
                        end: Self::to_lsp_position(line, col.saturating_add(1)),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("dal".to_string()),
                    message: e.to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                });
                return diags;
            }
            Ok(tokens_with_pos) => {
                // Parser
                let mut parser = Parser::new_with_positions(tokens_with_pos);
                if let Err(e) = parser.parse() {
                    let line = e.line_number().unwrap_or(1);
                    let col = e.column_number().unwrap_or(1);
                    diags.push(Diagnostic {
                        range: Range {
                            start: Self::to_lsp_position(line, col),
                            end: Self::to_lsp_position(line, col.saturating_add(1)),
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("dal".to_string()),
                        message: e.to_string(),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
        }

        diags
    }

    async fn publish_diagnostics_for_uri(&self, uri: Url, version: Option<i32>) {
        let text = {
            let docs = self.documents.lock().await;
            docs.get(&uri).map(|d| d.text.clone())
        };
        let diags = match text.as_deref() {
            Some(t) => Self::diagnostics_from_source(t),
            None => vec![],
        };
        self.client.publish_diagnostics(uri, diags, version).await;
    }

    /// Get the word (identifier or keyword) at 0-based line and character in source.
    fn word_at_position(source: &str, line_0: u32, char_0: u32) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = line_0 as usize;
        let line = lines.get(line_idx)?;
        let chars: Vec<char> = line.chars().collect();
        let char_idx = char_0 as usize;
        if char_idx > chars.len() {
            return None;
        }
        fn is_word_char(c: char) -> bool {
            c.is_ascii_alphanumeric() || c == '_'
        }
        let start = chars[..char_idx]
            .iter()
            .rposition(|&c| is_word_char(c))
            .map(|i| i + 1)
            .unwrap_or(0);
        let end = chars[char_idx..]
            .iter()
            .position(|&c| !is_word_char(c))
            .map(|i| char_idx + i)
            .unwrap_or(chars.len());
        if start >= end {
            return None;
        }
        Some(chars[start..end].iter().collect())
    }

    /// Short doc for a keyword (Phase 2: minimal set).
    fn keyword_doc(word: &str) -> Option<&'static str> {
        Some(match word {
            "fn" => "Function declaration. `fn name(params) -> return_type { ... }`",
            "let" => "Bind a value to a variable.",
            "return" => "Return a value from a function.",
            "if" | "else" => "Conditional branch.",
            "while" | "for" | "loop" => "Loop construct.",
            "service" => "Service declaration. `service Name { fields; fn methods() {} }`",
            "struct" => "Struct type definition.",
            "import" => "Import a module. `import \"path\";` or `import \"path\" as alias;`",
            "match" | "case" | "default" => "Pattern matching.",
            "try" | "catch" | "throw" => "Error handling.",
            "spawn" | "agent" => "Agent / concurrent execution.",
            "true" | "false" => "Boolean literal.",
            _ => return None,
        })
    }

    /// Short doc for a stdlib module name.
    fn stdlib_doc(module: &str) -> Option<&'static str> {
        Some(match module {
            "chain" => "Blockchain operations: deploy, call, balance, gas, etc.",
            "ai" => "AI/LLM: generate_text, spawn_agent, send_message, etc.",
            "log" => "Logging: info, error, warning, audit.",
            "auth" => "Authentication and authorization.",
            "config" => "Configuration and environment.",
            "db" => "Database operations.",
            "crypto" => "Cryptography: hash, sign, verify, keygen.",
            "oracle" => "Oracle and external data.",
            "agent" => "Agent coordination and communication.",
            _ => return None,
        })
    }

    /// Collect symbol names and details from parsed AST for completion/hover.
    fn collect_symbols_from_source(source: &str) -> Vec<(String, String)> {
        let mut symbols = Vec::new();
        let lexer = Lexer::new(source);
        let tokens_with_pos = match lexer.tokenize_with_positions_immutable() {
            Ok(t) => t,
            Err(_) => return symbols,
        };
        let mut parser = Parser::new_with_positions(tokens_with_pos);
        let program = match parser.parse() {
            Ok(p) => p,
            Err(_) => return symbols,
        };
        for stmt in &program.statements {
            match stmt {
                Statement::Service(s) => {
                    symbols.push((s.name.clone(), format!("service {}", s.name)));
                    for f in &s.fields {
                        symbols.push((
                            f.name.clone(),
                            format!("field {}: {}", f.name, f.field_type),
                        ));
                    }
                    for m in &s.methods {
                        let params: Vec<String> =
                            m.parameters.iter().map(|p| p.name.clone()).collect();
                        let sig = format!("fn {}({})", m.name, params.join(", "));
                        symbols.push((m.name.clone(), sig));
                    }
                }
                Statement::Function(f) => {
                    let params: Vec<String> = f.parameters.iter().map(|p| p.name.clone()).collect();
                    let sig = format!("fn {}({})", f.name, params.join(", "));
                    symbols.push((f.name.clone(), sig));
                }
                _ => {}
            }
        }
        symbols
    }

    /// Resolve hover content for a word (keyword, stdlib, or symbol from AST).
    fn hover_for_word(&self, source: &str, word: &str) -> Option<String> {
        if let Some(doc) = Self::keyword_doc(word) {
            return Some(format!("**{}**\n\n{}", word, doc));
        }
        if let Some(doc) = Self::stdlib_doc(word) {
            return Some(format!("**{}** (stdlib)\n\n{}", word, doc));
        }
        for (name, detail) in Self::collect_symbols_from_source(source) {
            if name == word {
                return Some(format!("**{}**\n\n`{}`", name, detail));
            }
        }
        None
    }

    /// Find the definition range (0-based) of a symbol in source. Returns None for keywords/stdlib.
    fn find_definition_range(source: &str, name: &str) -> Option<Range> {
        fn word_boundary_before(s: &str, i: usize) -> bool {
            i == 0
                || !s
                    .chars()
                    .nth(i.saturating_sub(1))
                    .map_or(false, |c| c.is_ascii_alphanumeric() || c == '_')
        }
        fn word_boundary_after(s: &str, i: usize, len: usize) -> bool {
            let end = i + len;
            let ch = s.chars().nth(end);
            ch.map_or(true, |c| !c.is_ascii_alphanumeric() && c != '_')
        }
        for (line_0, line) in source.lines().enumerate() {
            // service Name { or service Name \n
            if let Some(rest) = line.strip_prefix("service ") {
                let name_start = rest
                    .find(|c: char| c.is_ascii_alphabetic() || c == '_')
                    .unwrap_or(0);
                let name_end = rest[name_start..]
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .map_or(rest[name_start..].len(), |i| name_start + i);
                let def_name = &rest[name_start..name_end];
                if def_name == name && word_boundary_after(rest, name_start, name.len()) {
                    let char_0 = rest[..name_start].chars().count() as u32;
                    return Some(Range {
                        start: Position {
                            line: line_0 as u32,
                            character: char_0,
                        },
                        end: Position {
                            line: line_0 as u32,
                            character: char_0 + name.chars().count() as u32,
                        },
                    });
                }
            }
            // fn name( or fn name (
            if let Some(rest) = line.strip_prefix("fn ") {
                let name_start = rest
                    .find(|c: char| c.is_ascii_alphabetic() || c == '_')
                    .unwrap_or(0);
                let name_end = rest[name_start..]
                    .find(|c: char| c == '(' || c.is_whitespace())
                    .map_or(rest[name_start..].len(), |i| name_start + i);
                let def_name = &rest[name_start..name_end];
                if def_name == name {
                    let char_0 = 3u32 + rest[..name_start].chars().count() as u32;
                    return Some(Range {
                        start: Position {
                            line: line_0 as u32,
                            character: char_0,
                        },
                        end: Position {
                            line: line_0 as u32,
                            character: char_0 + name.chars().count() as u32,
                        },
                    });
                }
            }
            // field_name: type (word boundary before name)
            if let Some(idx) = line.find(name) {
                if word_boundary_before(line, idx) {
                    let after = &line[idx + name.len()..];
                    if after.starts_with(':')
                        && (idx == 0 || line[..idx].ends_with(' ') || line[..idx].ends_with('\t'))
                    {
                        let char_0 = line[..idx].chars().count() as u32;
                        return Some(Range {
                            start: Position {
                                line: line_0 as u32,
                                character: char_0,
                            },
                            end: Position {
                                line: line_0 as u32,
                                character: char_0 + name.chars().count() as u32,
                            },
                        });
                    }
                }
            }
        }
        None
    }

    /// Find the function call containing position and the active parameter index (0-based).
    fn function_call_at_position(source: &str, line_0: u32, char_0: u32) -> Option<(String, u32)> {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = line_0 as usize;
        let line = lines.get(line_idx)?;
        let char_idx = char_0 as usize;
        let prefix = line.chars().take(char_idx).collect::<String>();
        let suffix = line.chars().skip(char_idx).collect::<String>();
        // Find the last "name(" before cursor on this line; if cursor is inside ( ), count commas to get active param.
        let open = prefix.rfind('(')?;
        let before_open = prefix[..open].trim_end();
        let name_start =
            before_open.rfind(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == ':')?;
        let _name_end = before_open.len();
        let name_part = before_open[name_start..].trim_start_matches(':');
        let name_start_in_part = name_part
            .find(|c: char| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(0);
        let fn_name = name_part[name_start_in_part..]
            .split_whitespace()
            .next()?
            .to_string();
        let after_open = suffix.clone();
        let in_parens = prefix[open + 1..].to_string() + &after_open;
        let (before_close, _) = in_parens
            .split_once(')')
            .unwrap_or((in_parens.as_str(), ""));
        let active_param = before_close.split(',').count().saturating_sub(1) as u32;
        Some((fn_name, active_param))
    }

    /// Get signature label and parameter labels for a function (from AST or stdlib).
    fn signature_for_function(
        source: &str,
        name: &str,
    ) -> Option<(String, Vec<String>, Option<String>)> {
        for (sym_name, detail) in Self::collect_symbols_from_source(source) {
            if sym_name == name && detail.starts_with("fn ") {
                let label = detail.clone();
                let params = detail
                    .strip_prefix("fn ")
                    .and_then(|s| s.strip_suffix(")"))
                    .map(|s| s.split('(').nth(1).unwrap_or(""))
                    .unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                return Some((label, params, None));
            }
        }
        if Self::stdlib_doc(name).is_some() {
            return Some((format!("{}( … )", name), vec![], None));
        }
        None
    }
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _params: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: None,
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(lsp_types::ServerInfo {
                name: "dal".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(
                lsp_types::MessageType::INFO,
                "DAL language server initialized",
            )
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = Some(params.text_document.version);
        let text = params.text_document.text;
        {
            let mut docs = self.documents.lock().await;
            docs.insert(uri.clone(), DocumentState { text: text.clone() });
        }
        self.publish_diagnostics_for_uri(uri, version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = Some(params.text_document.version);
        // We requested TextDocumentSyncKind::FULL, so content_changes should have one full-doc item.
        let text = params
            .content_changes
            .into_iter()
            .last()
            .map(|c| c.text)
            .unwrap_or_default();
        {
            let mut docs = self.documents.lock().await;
            docs.insert(uri.clone(), DocumentState { text: text.clone() });
        }
        self.publish_diagnostics_for_uri(uri, version).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        {
            let mut docs = self.documents.lock().await;
            docs.remove(&uri);
        }
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let source = {
            let docs = self.documents.lock().await;
            docs.get(&uri).map(|d| d.text.clone())
        };
        let source = match source {
            Some(s) => s,
            None => return Ok(None),
        };
        let word = match Self::word_at_position(&source, pos.line, pos.character) {
            Some(w) if !w.is_empty() => w,
            _ => return Ok(None),
        };
        let content = match self.hover_for_word(&source, &word) {
            Some(c) => c,
            None => return Ok(None),
        };
        Ok(Some(Hover {
            contents: HoverContents::Scalar(lsp_types::MarkedString::String(content)),
            range: None,
        }))
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let source = {
            let docs = self.documents.lock().await;
            docs.get(&uri).map(|d| d.text.clone())
        };
        let source = source.unwrap_or_default();
        let prefix = Self::word_at_position(&source, pos.line, pos.character).unwrap_or_default();

        let mut items = Vec::new();

        // Keywords (Phase 2: main set)
        let keywords = [
            "fn", "let", "return", "if", "else", "while", "for", "in", "loop", "break", "continue",
            "service", "struct", "import", "match", "case", "default", "try", "catch", "throw",
            "spawn", "agent", "true", "false", "impl", "pub", "self", "Ok", "Err", "Some", "None",
        ];
        for kw in keywords {
            if prefix.is_empty() || kw.starts_with(&prefix) {
                items.push(CompletionItem {
                    label: kw.to_string(),
                    kind: Some(lsp_types::CompletionItemKind::KEYWORD),
                    detail: Self::keyword_doc(kw).map(String::from),
                    ..Default::default()
                });
            }
        }

        // Stdlib modules
        let modules = [
            "chain", "ai", "log", "auth", "config", "db", "crypto", "oracle", "agent",
        ];
        for m in modules {
            if prefix.is_empty() || m.starts_with(&prefix) {
                items.push(CompletionItem {
                    label: m.to_string(),
                    kind: Some(lsp_types::CompletionItemKind::MODULE),
                    detail: Self::stdlib_doc(m).map(String::from),
                    ..Default::default()
                });
            }
        }

        // Symbols from AST
        for (name, detail) in Self::collect_symbols_from_source(&source) {
            if prefix.is_empty() || name.starts_with(&prefix) {
                items.push(CompletionItem {
                    label: name.clone(),
                    kind: Some(lsp_types::CompletionItemKind::FUNCTION),
                    detail: Some(detail),
                    ..Default::default()
                });
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let source = {
            let docs = self.documents.lock().await;
            docs.get(&uri).map(|d| d.text.clone())
        };
        let source = match source {
            Some(s) => s,
            None => return Ok(None),
        };
        let word = match Self::word_at_position(&source, pos.line, pos.character) {
            Some(w) if !w.is_empty() => w,
            _ => return Ok(None),
        };
        if Self::keyword_doc(&word).is_some() || Self::stdlib_doc(&word).is_some() {
            return Ok(None);
        }
        let range = match Self::find_definition_range(&source, &word) {
            Some(r) => r,
            None => return Ok(None),
        };
        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri,
            range,
        })))
    }

    async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let source = {
            let docs = self.documents.lock().await;
            docs.get(&uri).map(|d| d.text.clone())
        };
        let source = source.unwrap_or_default();
        let (fn_name, active_param) =
            match Self::function_call_at_position(&source, pos.line, pos.character) {
                Some(x) => x,
                None => return Ok(None),
            };
        let (label, param_labels, doc) = match Self::signature_for_function(&source, &fn_name) {
            Some(x) => x,
            None => return Ok(None),
        };
        let params = if param_labels.is_empty() {
            None
        } else {
            Some(
                param_labels
                    .into_iter()
                    .map(|l| ParameterInformation {
                        label: lsp_types::ParameterLabel::Simple(l),
                        documentation: None,
                    })
                    .collect(),
            )
        };
        let sig = SignatureInformation {
            label,
            documentation: doc.map(lsp_types::Documentation::String),
            parameters: params,
            active_parameter: None,
        };
        Ok(Some(SignatureHelp {
            signatures: vec![sig],
            active_signature: Some(0),
            active_parameter: Some(active_param),
        }))
    }
}

/// Run the LSP server on stdio. Call from main with tokio::runtime block_on.
pub async fn run_lsp_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

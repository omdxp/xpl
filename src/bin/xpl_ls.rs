// src/bin/xpl_ls.rs

use tower_lsp::lsp_types::{
    CodeLens, CodeLensOptions, CodeLensParams, Command, CompletionItem, CompletionOptions,
    CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, Documentation, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverContents, HoverParams, InitializeParams, InitializeResult,
    InitializedParams, Location, MarkedString, MessageType, OneOf, ParameterInformation,
    ParameterLabel, Position, Range, ReferenceParams, ServerCapabilities, SignatureHelp,
    SignatureHelpOptions, SignatureHelpParams, SignatureInformation, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};
use xpl::XplError;
use xpl::parser;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}

struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    work_done_progress_options: Default::default(),
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                }),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(true),
                }),
                hover_provider: Some(tower_lsp::lsp_types::HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..ServerCapabilities::default()
            },
            server_info: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "xpl Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let path = uri.to_file_path().unwrap();
        let src = std::fs::read_to_string(&path).unwrap_or_default();
        let pos = params.text_document_position_params.position;
        let token = get_token_at(&src, pos.line as usize, pos.character as usize);
        if let Some(tok) = token {
            // search in current file
            for (i, line) in src.lines().enumerate() {
                if line.contains(&format!("<function name=\"{}\"", tok)) {
                    if let Some(col) = line.find(&tok) {
                        let loc = Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position::new(i as u32, col as u32),
                                end: Position::new(i as u32, (col + tok.len()) as u32),
                            },
                        };
                        return Ok(Some(GotoDefinitionResponse::Scalar(loc)));
                    }
                }
            }
            // search other .xpl files in the same directory
            if let Some(dir) = path.parent() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let pth = entry.path();
                        if pth.extension().and_then(|e| e.to_str()) == Some("xpl") {
                            let content = std::fs::read_to_string(&pth).unwrap_or_default();
                            for (j, l) in content.lines().enumerate() {
                                if l.contains(&format!("<function name=\"{}\"", tok)) {
                                    if let Some(start) = l.find(&tok) {
                                        let uri2 = Url::from_file_path(&pth).unwrap();
                                        let range = Range {
                                            start: Position::new(j as u32, start as u32),
                                            end: Position::new(
                                                j as u32,
                                                (start + tok.len()) as u32,
                                            ),
                                        };
                                        return Ok(Some(GotoDefinitionResponse::Scalar(
                                            Location { uri: uri2, range },
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    async fn references(
        &self,
        params: ReferenceParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let path = uri.to_file_path().unwrap();
        let src = std::fs::read_to_string(&path).unwrap_or_default();
        let pos = params.text_document_position.position;
        let token = get_token_at(&src, pos.line as usize, pos.character as usize);
        let mut locs = Vec::new();
        if let Some(tok) = token {
            for (i, line) in src.lines().enumerate() {
                if let Some(col) = line.find(&tok) {
                    locs.push(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position::new(i as u32, col as u32),
                            end: Position::new(i as u32, (col + tok.len()) as u32),
                        },
                    });
                }
            }
        }
        Ok(Some(locs))
    }

    async fn signature_help(
        &self,
        _params: SignatureHelpParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SignatureHelp>> {
        let params = _params;
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let path = uri.to_file_path().unwrap();
        let src = std::fs::read_to_string(&path).unwrap_or_default();
        let pos = params.text_document_position_params.position;
        let mut col = pos.character as usize;
        while col > 0
            && src
                .lines()
                .nth(pos.line as usize)
                .unwrap_or("")
                .chars()
                .nth(col - 1)
                .unwrap_or(' ')
                != '('
        {
            col -= 1;
        }
        if col > 0 {
            let token = get_token_at(&src, pos.line as usize, col - 1);
            if let Some(tok) = token {
                if let Ok(prog) = parser::parse_file(path.to_str().unwrap()) {
                    if let Some(f) = prog.functions.get(&tok) {
                        let sig_label = format!(
                            "{}({})",
                            f.name,
                            f.params
                                .iter()
                                .map(|p| p.name.clone())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        let parameters = f
                            .params
                            .iter()
                            .map(|p| ParameterInformation {
                                label: ParameterLabel::Simple(p.name.clone()),
                                documentation: p.description.clone().map(Documentation::String),
                            })
                            .collect();
                        let sign = SignatureInformation {
                            label: sig_label,
                            documentation: f.description.clone().map(Documentation::String),
                            parameters: Some(parameters),
                            active_parameter: Some(0),
                        };
                        return Ok(Some(SignatureHelp {
                            signatures: vec![sign],
                            active_signature: Some(0),
                            active_parameter: Some(0),
                        }));
                    }
                }
            }
        }
        Ok(None)
    }

    async fn code_lens(
        &self,
        _params: CodeLensParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<CodeLens>>> {
        // URI not needed here
        let mut lenses = Vec::new();
        lenses.push(CodeLens {
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            command: Some(Command {
                title: "Run".to_string(),
                command: "xpl.runFile".to_string(),
                arguments: None,
            }),
            data: None,
        });
        Ok(Some(lenses))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let path = uri.to_file_path().unwrap();
        let diagnostics = match xpl::run_file(path.to_str().unwrap()) {
            Ok(_) => Vec::new(),
            Err(e) => {
                let (line0, col0, msg) = match &e {
                    XplError::Semantic { line, col, msg, .. } => (
                        (*line).saturating_sub(1) as u32,
                        (*col).saturating_sub(1) as u32,
                        msg.clone(),
                    ),
                    XplError::Xml { .. } | XplError::Io { .. } => (0, 0, e.to_string()),
                };
                let range = Range {
                    start: Position::new(line0, col0),
                    end: Position::new(line0, col0 + 1),
                };
                vec![Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("xpl".to_string()),
                    message: msg,
                    tags: None,
                    related_information: None,
                    data: None,
                }]
            }
        };
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let path = uri.to_file_path().unwrap();
        let diagnostics = match xpl::run_file(path.to_str().unwrap()) {
            Ok(_) => Vec::new(),
            Err(e) => {
                let (line0, col0, msg) = match &e {
                    XplError::Semantic { line, col, msg, .. } => (
                        (*line).saturating_sub(1) as u32,
                        (*col).saturating_sub(1) as u32,
                        msg.clone(),
                    ),
                    _ => (0, 0, e.to_string()),
                };
                let range = Range {
                    start: Position::new(line0, col0),
                    end: Position::new(line0, col0 + 1),
                };
                vec![Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("xpl".to_string()),
                    message: msg,
                    tags: None,
                    related_information: None,
                    data: None,
                }]
            }
        };
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let path = uri.to_file_path().unwrap();
        let pos = params.text_document_position_params.position;
        let src = std::fs::read_to_string(&path).unwrap_or_default();
        let line = src.lines().nth(pos.line as usize).unwrap_or("");
        let col = pos.character as usize;
        let is_word = |c: char| c.is_alphanumeric() || c == '_';
        let mut start = col;
        while start > 0 && is_word(line.chars().nth(start - 1).unwrap_or(' ')) {
            start -= 1;
        }
        let mut end = col;
        while end < line.len() && is_word(line.chars().nth(end).unwrap_or(' ')) {
            end += 1;
        }
        let token = &line[start..end];
        if let Ok(prog) = parser::parse_file(path.to_str().unwrap()) {
            if let Some(func) = prog.functions.get(token) {
                let mut contents = String::new();
                if let Some(desc) = &prog.description {
                    contents.push_str(&format!("**Program**: {}\n\n", desc));
                }
                if let Some(fdesc) = &func.description {
                    contents.push_str(&format!("**Function** {}**:** {}\n\n", func.name, fdesc));
                } else {
                    contents.push_str(&format!("**Function** {}\n\n", func.name));
                }
                let params = func
                    .params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                contents.push_str(&format!("Signature: {}({})", func.name, params));
                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(contents)),
                    range: None,
                }));
            }

            for func in prog.functions.values() {
                for param in &func.params {
                    if param.name == token {
                        let mut text = format!("Parameter **{}**", param.name);
                        if let Some(pt) = &param.ptype {
                            text.push_str(&format!(": {}", pt));
                        }
                        if let Some(pd) = &param.description {
                            text.push_str(&format!("\n\n{}", pd));
                        }
                        return Ok(Some(Hover {
                            contents: HoverContents::Scalar(MarkedString::String(text)),
                            range: None,
                        }));
                    }
                }
            }

            if token == "program" {
                let mut text = String::new();
                if let Some(prog_desc) = &prog.description {
                    text.push_str(&format!("Program: {}\n\n", prog_desc));
                }
                text.push_str("Entry point: main()\n");
                text.push_str("Usage: xpl <file>.xpl");
                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(text)),
                    range: None,
                }));
            }
        }
        Ok(None)
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let path = uri.to_file_path().unwrap();
        let pos = params.text_document_position.position;
        let src = std::fs::read_to_string(&path).unwrap_or_default();
        let line = src.lines().nth(pos.line as usize).unwrap_or("");
        if line[..pos.character as usize].contains("include") {
            let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
            let mut items = Vec::new();
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.ends_with(".xpl") {
                        items.push(CompletionItem::new_simple(
                            fname.clone(),
                            "XPL include".to_string(),
                        ));
                    }
                }
            }
            return Ok(Some(CompletionResponse::Array(items)));
        }
        let items = parser::parse_file("examples/math.xpl")
            .map(|prog| {
                prog.functions
                    .keys()
                    .map(|name| {
                        CompletionItem::new_simple(name.clone(), "xpl function".to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Some(CompletionResponse::Array(items)))
    }
}

// helper to extract word at line,col
fn get_token_at(src: &str, line: usize, col: usize) -> Option<String> {
    let l = src.lines().nth(line)?;
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let mut start = col;
    while start > 0 && is_word(l.chars().nth(start - 1)?) {
        start -= 1;
    }
    let mut end = col;
    while end < l.len() && is_word(l.chars().nth(end)?) {
        end += 1;
    }
    Some(l[start..end].to_string())
}

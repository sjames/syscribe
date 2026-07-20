//! `syscribe lsp` — a Language Server Protocol server over stdio.
//!
//! v1 (`ADR-SYS-LSP-001`): diagnostics, go-to-definition, find-references, hover,
//! workspace/symbol — every capability maps onto structure `syscribe-model` already
//! computes. v2 (`ADR-SYS-LSP-002`): field-aware/enum completion, plus a
//! `WorkspaceEdit`-based rename scoped to stable ids (the server never writes to
//! disk for a rename). v3 (`ADR-SYS-LSP-003`): display-only `codeLens` counts, plus
//! `codeAction` quick-fixes for exactly `E310` and `W090` (the latter executed
//! server-side via `workspace/executeCommand`, since it's a real disk-write side
//! effect, not a client-applicable edit). No custom (non-LSP) methods anywhere.
//!
//! The store is disk-backed and full-reload only (REQ-TRS-LSP-007):
//! `textDocument/didChange` never triggers reparsing or diagnostics — only
//! `completion`/`hover`/`definition`/`references`/`prepareRename`/`rename`/`codeLens`/
//! `codeAction` read live off already-saved disk state.

mod actions;
mod completion;
mod rename;
mod store;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::{Error as LspError, Result as LspResult};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use syscribe_model::element::RawElement;
use syscribe_model::validator::{validate_with_config, Finding, Severity};

use crate::query::type_label;
use store::LspStore;

struct Backend {
    client: Client,
    store: Arc<RwLock<LspStore>>,
    open_docs: Arc<RwLock<HashSet<Url>>>,
}

// ── URI / position helpers ───────────────────────────────────────────────────

fn path_to_uri(path: &str) -> Option<Url> {
    Url::from_file_path(Path::new(path)).ok()
}

fn zero_range() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 0))
}

/// Best-effort diagnostic range (REQ-TRS-LSP-002): the extent of the file's YAML
/// frontmatter block. `Finding` carries only a file path, not a field/line location.
fn frontmatter_range(path: &Path) -> Range {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Range::new(Position::new(0, 0), Position::new(1, 0));
    };
    let mut lines = text.lines().enumerate();
    match lines.next() {
        Some((_, "---")) => {}
        _ => return Range::new(Position::new(0, 0), Position::new(1, 0)),
    }
    for (i, line) in lines {
        if line == "---" {
            return Range::new(Position::new(0, 0), Position::new((i + 1) as u32, 0));
        }
    }
    Range::new(Position::new(0, 0), Position::new(1, 0))
}

fn finding_to_diagnostic(f: &Finding) -> Diagnostic {
    Diagnostic {
        range: frontmatter_range(Path::new(&f.file)),
        severity: Some(match f.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
        }),
        code: Some(NumberOrString::String(f.code.to_string())),
        code_description: None,
        source: Some("syscribe".to_string()),
        message: f.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// True for characters that make up an id/qname token (`is_tok` in the informal
/// sense used throughout this module — alphanumerics, `_`, `:` for `::`-qnames, `-`
/// for stable ids).
fn is_tok_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '-'
}

/// Extract the identifier-like token at `character` in `line` (byte/char index —
/// model ids and qnames are ASCII, so LSP's UTF-16 `character` and a char count
/// coincide for the content this targets), along with its `[start, end)` char range.
fn token_at_with_range(line: &str, character: usize) -> Option<(String, usize, usize)> {
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let mut start = character.min(chars.len());
    if start == chars.len() || !is_tok_char(chars[start]) {
        if start > 0 && is_tok_char(chars[start - 1]) {
            start -= 1;
        } else {
            return None;
        }
    }
    let mut s = start;
    while s > 0 && is_tok_char(chars[s - 1]) {
        s -= 1;
    }
    let mut e = start;
    while e < chars.len() && is_tok_char(chars[e]) {
        e += 1;
    }
    // Trim stray leading/trailing colons (a bare YAML key's trailing `:`, e.g.
    // `supertype:`) from the token but keep the reported range untrimmed — the
    // caller (prepareRename) wants the exact on-screen span.
    let raw: String = chars[s..e].iter().collect();
    let tok = raw.trim_matches(':');
    if tok.is_empty() {
        None
    } else if tok.len() == raw.len() {
        Some((tok.to_string(), s, e))
    } else {
        let lead_trim = raw.len() - raw.trim_start_matches(':').len();
        Some((tok.to_string(), s + lead_trim, s + lead_trim + tok.len()))
    }
}

fn token_at(line: &str, character: usize) -> Option<String> {
    token_at_with_range(line, character).map(|(t, _, _)| t)
}

fn resolve_token_element<'a>(store: &'a LspStore, path: &Path, position: Position) -> Option<&'a RawElement> {
    let text = std::fs::read_to_string(path).ok()?;
    let line = text.lines().nth(position.line as usize)?;
    let token = token_at(line, position.character as usize)?;
    store.resolver.resolve_ref(&store.elements, &token)
}

/// Generic reverse-reference check (REQ-TRS-LSP-004): does `elem`'s frontmatter
/// contain `target`'s id or qname anywhere (scalar, sequence entry, or map key —
/// covers `traceBaselines:` keys)? Field-name-agnostic, so it needs no per-field-kind
/// enumeration of the many cross-reference field kinds (`supertype`, `derivedFrom`,
/// `verifies`, `satisfies`, allocation `from`/`to`, …), at the cost of a small false-
/// positive risk if free text happens to equal an id/qname exactly.
fn element_references(elem: &RawElement, target: &RawElement) -> bool {
    let mut needles: Vec<&str> = vec![target.qualified_name.as_str()];
    if let Some(id) = target.frontmatter.id.as_deref() {
        needles.push(id);
    }
    let Ok(val) = serde_yaml::to_value(&elem.frontmatter) else {
        return false;
    };
    value_contains_any(&val, &needles)
}

fn value_contains_any(v: &serde_yaml::Value, needles: &[&str]) -> bool {
    match v {
        serde_yaml::Value::String(s) => needles.contains(&s.as_str()),
        serde_yaml::Value::Sequence(seq) => seq.iter().any(|x| value_contains_any(x, needles)),
        serde_yaml::Value::Mapping(m) => m.iter().any(|(k, v)| {
            matches!(k, serde_yaml::Value::String(ks) if needles.contains(&ks.as_str())) || value_contains_any(v, needles)
        }),
        _ => false,
    }
}

fn hover_markdown(elem: &RawElement) -> String {
    let type_str = elem.frontmatter.element_type.as_ref().map(type_label).unwrap_or("?");
    let name = elem.frontmatter.name.clone().unwrap_or_else(|| elem.qualified_name.clone());
    let mut md = format!("**{name}**  \n`{type_str}`\n\nqname: `{}`", elem.qualified_name);
    if let Some(id) = &elem.frontmatter.id {
        md.push_str(&format!("\n\nid: `{id}`"));
    }
    if let Some(status) = &elem.frontmatter.status {
        md.push_str(&format!("\n\nstatus: `{status}`"));
    }
    md
}

impl Backend {
    async fn diagnostics_by_file(&self) -> std::collections::HashMap<String, Vec<Diagnostic>> {
        let store = self.store.read().await;
        let result = validate_with_config(&store.elements, &store.config);
        let mut by_file: std::collections::HashMap<String, Vec<Diagnostic>> = std::collections::HashMap::new();
        for f in &result.findings {
            by_file.entry(f.file.clone()).or_default().push(finding_to_diagnostic(f));
        }
        by_file
    }

    async fn publish_for_uri(&self, uri: &Url) {
        let Ok(path) = uri.to_file_path() else { return };
        let by_file = self.diagnostics_by_file().await;
        let key = path.to_string_lossy().to_string();
        let diags = by_file.get(&key).cloned().unwrap_or_default();
        self.client.publish_diagnostics(uri.clone(), diags, None).await;
    }

    async fn publish_for_all_open(&self) {
        let by_file = self.diagnostics_by_file().await;
        let open: Vec<Url> = self.open_docs.read().await.iter().cloned().collect();
        for uri in open {
            let Ok(path) = uri.to_file_path() else { continue };
            let key = path.to_string_lossy().to_string();
            let diags = by_file.get(&key).cloned().unwrap_or_default();
            self.client.publish_diagnostics(uri, diags, None).await;
        }
    }

    async fn reload(&self) {
        let mut store = self.store.write().await;
        if let Err(e) = store.reload() {
            drop(store);
            self.client.log_message(MessageType::ERROR, format!("model reload failed, keeping prior state: {e}")).await;
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "syscribe-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions::default()),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
                code_lens_provider: Some(CodeLensOptions { resolve_provider: None }),
                code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                    code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                    resolve_provider: None,
                    work_done_progress_options: Default::default(),
                })),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["syscribe.suspectAccept".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {}

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        self.open_docs.write().await.insert(uri.clone());
        self.publish_for_uri(&uri).await;
    }

    async fn did_change(&self, _params: DidChangeTextDocumentParams) {
        // v1 validates saved disk state only (REQ-TRS-LSP-002 / ADR-SYS-LSP-001's
        // full-reload, no-incremental-parsing decision) — unsaved buffer edits do
        // not trigger a reload or diagnostics republish.
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        self.reload().await;
        self.publish_for_all_open().await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.open_docs.write().await.remove(&uri);
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn did_change_watched_files(&self, _params: DidChangeWatchedFilesParams) {
        self.reload().await;
        self.publish_for_all_open().await;
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let Ok(path) = uri.to_file_path() else { return Ok(None) };
        let store = self.store.read().await;
        let Some(elem) = resolve_token_element(&store, &path, position) else { return Ok(None) };
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value: hover_markdown(elem) }),
            range: None,
        }))
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let Ok(path) = uri.to_file_path() else { return Ok(None) };
        let store = self.store.read().await;
        let Some(elem) = resolve_token_element(&store, &path, position) else { return Ok(None) };
        let Some(target_uri) = path_to_uri(&elem.file_path) else { return Ok(None) };
        Ok(Some(GotoDefinitionResponse::Scalar(Location { uri: target_uri, range: zero_range() })))
    }

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;
        let Ok(path) = uri.to_file_path() else { return Ok(None) };
        let store = self.store.read().await;
        let Some(target) = resolve_token_element(&store, &path, position) else { return Ok(None) };

        let mut locations: Vec<Location> = store
            .elements
            .iter()
            .filter(|elem| elem.qualified_name != target.qualified_name && element_references(elem, target))
            .filter_map(|elem| path_to_uri(&elem.file_path).map(|uri| Location { uri, range: zero_range() }))
            .collect();
        if include_declaration {
            if let Some(uri) = path_to_uri(&target.file_path) {
                locations.push(Location { uri, range: zero_range() });
            }
        }
        Ok(Some(locations))
    }

    async fn symbol(&self, params: WorkspaceSymbolParams) -> LspResult<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();
        let store = self.store.read().await;
        #[allow(deprecated)]
        let results: Vec<SymbolInformation> = store
            .elements
            .iter()
            .filter(|elem| {
                if query.is_empty() {
                    return true;
                }
                let name = elem.frontmatter.name.as_deref().unwrap_or_default();
                let id = elem.frontmatter.id.as_deref().unwrap_or_default();
                let haystack = format!("{name} {id} {}", elem.qualified_name).to_lowercase();
                haystack.contains(&query)
            })
            .take(200)
            .filter_map(|elem| {
                path_to_uri(&elem.file_path).map(|uri| SymbolInformation {
                    name: elem.frontmatter.name.clone().unwrap_or_else(|| elem.qualified_name.clone()),
                    kind: SymbolKind::OBJECT,
                    tags: None,
                    deprecated: None,
                    location: Location { uri, range: zero_range() },
                    container_name: None,
                })
            })
            .collect();
        Ok(Some(results))
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let Ok(path) = uri.to_file_path() else { return Ok(None) };
        let store = self.store.read().await;
        Ok(completion::candidates(&store, &path, position))
    }

    async fn prepare_rename(&self, params: TextDocumentPositionParams) -> LspResult<Option<PrepareRenameResponse>> {
        let Ok(path) = params.text_document.uri.to_file_path() else { return Ok(None) };
        let store = self.store.read().await;
        rename::prepare(&store, &path, params.position).map(Some).map_err(LspError::invalid_params)
    }

    async fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let Ok(path) = uri.to_file_path() else { return Ok(None) };
        let store = self.store.read().await;
        rename::compute(&store, &path, position, &params.new_name).map(Some).map_err(LspError::invalid_params)
    }

    async fn code_lens(&self, params: CodeLensParams) -> LspResult<Option<Vec<CodeLens>>> {
        let Ok(path) = params.text_document.uri.to_file_path() else { return Ok(None) };
        let store = self.store.read().await;
        Ok(Some(actions::lenses_for(&store, &path)))
    }

    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        let Ok(path) = params.text_document.uri.to_file_path() else { return Ok(None) };
        let store = self.store.read().await;
        Ok(Some(actions::actions_for(&store, &path, &params.range)))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> LspResult<Option<serde_json::Value>> {
        if params.command != "syscribe.suspectAccept" {
            return Err(LspError::method_not_found());
        }
        let source = params.arguments.first().and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let target = params.arguments.get(1).and_then(|v| v.as_str()).unwrap_or_default().to_string();

        let mut store = self.store.write().await;
        let result = actions::execute_suspect_accept(&store, &source, &target);
        match result {
            Ok(()) => {
                if let Err(e) = store.reload() {
                    self.client.log_message(MessageType::ERROR, format!("model reload failed, keeping prior state: {e}")).await;
                }
                drop(store);
                self.publish_for_all_open().await;
                Ok(Some(serde_json::json!({"accepted": true})))
            }
            Err(e) => Err(LspError::invalid_params(e)),
        }
    }
}

/// Start the `syscribe lsp` server: load the model once, then serve LSP requests
/// over stdio until the client sends `exit` and closes stdin.
pub fn cmd_lsp(model_root: &Path) -> anyhow::Result<()> {
    let model_root: PathBuf = model_root.to_path_buf();
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async move {
        let store = LspStore::load(&model_root)?;
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let (service, socket) = LspService::new(move |client| Backend {
            client,
            store: Arc::new(RwLock::new(store)),
            open_docs: Arc::new(RwLock::new(HashSet::new())),
        });
        Server::new(stdin, stdout, socket).serve(service).await;
        Ok::<(), anyhow::Error>(())
    })
}

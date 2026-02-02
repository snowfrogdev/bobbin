//! LSP server implementation using tower-lsp.

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use bobbin_syntax::{
    AriadneRenderer, BOOLEAN_LITERALS, CommandDeclaration, KEYWORDS, LineIndex, Renderer,
    VariableDeclaration, VariableKind, analyze, validate,
};

use crate::convert::to_lsp_diagnostics;

/// The Bobbin language server.
pub struct BobbinLanguageServer {
    /// LSP client for sending notifications (e.g., diagnostics).
    client: Client,
    /// Document store: URI -> source text.
    documents: RwLock<HashMap<Url, String>>,
    /// Position encoding to use (negotiated during initialize).
    /// true = UTF-16 (fallback), false = UTF-8 (preferred).
    use_utf16: RwLock<bool>,
}

impl BobbinLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
            use_utf16: RwLock::new(true), // Default to UTF-16 for compatibility
        }
    }

    /// Validate a document and publish diagnostics.
    async fn validate_document(&self, uri: Url, source: &str) {
        let diagnostics = validate(source);
        let use_utf16 = *self.use_utf16.read().unwrap();

        let lsp_diagnostics = if diagnostics.is_empty() {
            vec![]
        } else {
            let line_index = LineIndex::new(source);

            // Log beautiful ASCII-formatted errors to Output channel
            let filename = uri
                .path_segments()
                .and_then(|s| s.last())
                .unwrap_or("unknown.bobbin");
            let renderer = AriadneRenderer::without_colors();
            let rendered = renderer.render_all(&diagnostics, filename, source);
            self.client.log_message(MessageType::ERROR, rendered).await;

            to_lsp_diagnostics(&diagnostics, &line_index, use_utf16)
        };

        self.client
            .publish_diagnostics(uri, lsp_diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for BobbinLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Check client's preferred position encodings
        let use_utf16 = params
            .capabilities
            .general
            .as_ref()
            .and_then(|g| g.position_encodings.as_ref())
            .map(|encodings| {
                // Prefer UTF-8 if client supports it
                !encodings.iter().any(|e| *e == PositionEncodingKind::UTF8)
            })
            .unwrap_or(true); // Default to UTF-16 if not specified

        *self.use_utf16.write().unwrap() = use_utf16;

        let position_encoding = if use_utf16 {
            Some(PositionEncodingKind::UTF16)
        } else {
            Some(PositionEncodingKind::UTF8)
        };

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding,
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["{".to_string()]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "bobbin-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Bobbin language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        // Store the document
        self.documents
            .write()
            .unwrap()
            .insert(uri.clone(), text.clone());

        // Validate and publish diagnostics
        self.validate_document(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        // We're using FULL sync, so there's exactly one change with the full text
        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;

            // Update stored document
            self.documents
                .write()
                .unwrap()
                .insert(uri.clone(), text.clone());

            // Validate and publish diagnostics
            self.validate_document(uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove from document store
        self.documents.write().unwrap().remove(&uri);

        // Clear diagnostics for closed document
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get document source, returning None if not found or lock is poisoned
        let source = match self
            .documents
            .read()
            .ok()
            .and_then(|docs| docs.get(&uri).cloned())
        {
            Some(s) => s,
            None => return Ok(None),
        };

        let use_utf16 = self.use_utf16.read().map(|g| *g).unwrap_or(true);
        let line_index = LineIndex::new(&source);
        let offset = line_index.offset(position.line, position.character, use_utf16);

        let in_interpolation = is_inside_interpolation(&source, offset);
        let analysis = analyze(&source);

        let items = build_completion_items(&analysis.declarations, &analysis.commands, in_interpolation);
        Ok(Some(CompletionResponse::List(CompletionList {
            is_incomplete: false,
            items,
        })))
    }
}

/// Check if cursor is inside an interpolation `{...}`.
///
/// NOTE: This uses simple brace counting which doesn't handle escaped braces
/// or braces in string literals. This is acceptable for Bobbin's current syntax
/// which has no escape sequences or nested string contexts.
fn is_inside_interpolation(source: &str, offset: usize) -> bool {
    let before = &source[..offset.min(source.len())];
    let open_braces = before.matches('{').count();
    let close_braces = before.matches('}').count();
    open_braces > close_braces
}

/// Build completion items from declarations and context.
fn build_completion_items(
    decls: &[VariableDeclaration],
    commands: &[CommandDeclaration],
    in_interpolation: bool,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();

    // Variables (deduplicated by name)
    for decl in decls {
        if seen.insert(decl.name.clone()) {
            items.push(CompletionItem {
                label: decl.name.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some(match decl.kind {
                    VariableKind::Temp => "(temp)".to_string(),
                    VariableKind::Save => "(save)".to_string(),
                    VariableKind::Extern => "(extern)".to_string(),
                }),
                sort_text: Some(format!("0_{}", decl.name)), // Variables first
                ..Default::default()
            });
        }
    }

    // Commands only outside interpolation
    if !in_interpolation {
        for cmd in commands {
            if seen.insert(cmd.name.clone()) {
                let params_str = cmd.params.join(", ");
                let label = format!("{}({})", cmd.name, params_str);
                let insert_text = format!("{}($0)", cmd.name);
                items.push(CompletionItem {
                    label,
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some("(command)".to_string()),
                    insert_text: Some(insert_text),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    sort_text: Some(format!("0_{}", cmd.name)), // Same priority as variables
                    ..Default::default()
                });
            }
        }

        // Keywords
        for keyword in KEYWORDS {
            items.push(CompletionItem {
                label: (*keyword).to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                insert_text: Some(format!("{} ", keyword)),
                filter_text: Some((*keyword).to_string()), // Filter without trailing space
                sort_text: Some(format!("1_{}", keyword)), // Keywords after variables
                ..Default::default()
            });
        }
        // Boolean literals
        for literal in BOOLEAN_LITERALS {
            items.push(CompletionItem {
                label: (*literal).to_string(),
                kind: Some(CompletionItemKind::CONSTANT),
                sort_text: Some(format!("2_{}", literal)), // Literals last
                ..Default::default()
            });
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_inside_interpolation_open_brace() {
        assert!(is_inside_interpolation("Hello {", 7));
        assert!(is_inside_interpolation("Hello {x", 8));
        assert!(is_inside_interpolation("Hello {name}! Value: {", 22));
    }

    #[test]
    fn is_inside_interpolation_closed() {
        assert!(!is_inside_interpolation("Hello", 5));
        assert!(!is_inside_interpolation("Hello {x}", 9));
        assert!(!is_inside_interpolation("Hello {x} world", 15));
    }

    #[test]
    fn is_inside_interpolation_multiple_braces() {
        assert!(is_inside_interpolation("{a} {b} {", 9));
        assert!(!is_inside_interpolation("{a} {b}", 7));
    }

    #[test]
    fn build_completion_items_deduplicates() {
        use bobbin_syntax::Span;

        let decls = vec![
            VariableDeclaration {
                name: "x".to_string(),
                kind: VariableKind::Temp,
                span: Span { start: 0, end: 1 },
            },
            VariableDeclaration {
                name: "x".to_string(),
                kind: VariableKind::Temp,
                span: Span { start: 10, end: 11 },
            },
        ];

        let items = build_completion_items(&decls, &[], false);
        let x_items: Vec<_> = items.iter().filter(|i| i.label == "x").collect();
        assert_eq!(x_items.len(), 1);
    }

    #[test]
    fn build_completion_items_includes_keywords_outside_interpolation() {
        let items = build_completion_items(&[], &[], false);
        let keyword_items: Vec<_> = items
            .iter()
            .filter(|i| i.kind == Some(CompletionItemKind::KEYWORD))
            .collect();
        assert!(!keyword_items.is_empty());
        assert!(keyword_items.iter().any(|i| i.label == "save"));
    }

    #[test]
    fn build_completion_items_excludes_keywords_in_interpolation() {
        let items = build_completion_items(&[], &[], true);
        let keyword_items: Vec<_> = items
            .iter()
            .filter(|i| i.kind == Some(CompletionItemKind::KEYWORD))
            .collect();
        assert!(keyword_items.is_empty());
    }
}

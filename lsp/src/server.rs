//! LSP server implementation using tower-lsp.

use std::collections::HashMap;
use std::sync::RwLock;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use bobbin_syntax::{validate, LineIndex};

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
                !encodings
                    .iter()
                    .any(|e| *e == PositionEncodingKind::UTF8)
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
}

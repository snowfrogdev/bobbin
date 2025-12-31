//! Bobbin Language Server
//!
//! A Language Server Protocol implementation for the Bobbin narrative scripting language.
//! Provides diagnostics (error squiggles) for VS Code and other LSP-compatible editors.

use tower_lsp::{LspService, Server};

mod convert;
mod server;

use server::BobbinLanguageServer;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(BobbinLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

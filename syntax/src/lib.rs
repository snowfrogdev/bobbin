//! Bobbin syntax analysis crate.
//!
//! This crate provides the frontend components of the Bobbin language:
//! - Scanner (lexical analysis)
//! - Parser (syntax analysis)
//! - Resolver (semantic analysis)
//! - Diagnostic system (error reporting)
//!
//! It is used by both the runtime (for compilation) and the LSP/editor tooling
//! (for diagnostics without execution).

pub mod ast;
pub mod diagnostic;
pub mod parser;
pub mod resolver;
pub mod scanner;
pub mod token;

pub use ast::{Choice, ExternDeclData, Literal, NodeId, Script, Stmt, TextPart, VarBindingData};
pub use diagnostic::{
    AriadneRenderer, Diagnostic, DiagnosticContext, IntoDiagnostic, JaroWinklerMatcher, Label,
    LabelStyle, LineIndex, Matcher, Renderer, Severity, SourcePosition, Suggestion,
};
pub use parser::{ParseError, Parser};
pub use resolver::{Resolver, SemanticError, SymbolTable};
pub use scanner::{LexicalError, Scanner};
pub use token::{Span, Token, TokenKind};

/// Validate source code and return diagnostics without executing.
///
/// This is the main entry point for editor tooling. It runs the scanner,
/// parser, and resolver to collect all errors without compiling or running
/// the script.
///
/// # Example
///
/// ```
/// use bobbin_syntax::validate;
///
/// let source = "Hello {unknown}!";
/// let diagnostics = validate(source);
/// assert_eq!(diagnostics.len(), 1);
/// assert!(diagnostics[0].message.contains("undefined"));
/// ```
pub fn validate(source: &str) -> Vec<Diagnostic> {
    let tokens = Scanner::new(source).tokens();
    match Parser::new(tokens).parse() {
        Err(errors) => {
            let matcher = JaroWinklerMatcher::default();
            let ctx = DiagnosticContext::new(&[], &matcher);
            errors
                .into_iter()
                .map(|e| e.into_diagnostic(&ctx))
                .collect()
        }
        Ok(ast) => match Resolver::new(&ast).analyze() {
            Err((errors, known_variables)) => {
                let matcher = JaroWinklerMatcher::default();
                let ctx = DiagnosticContext::new(&known_variables, &matcher);
                errors
                    .into_iter()
                    .map(|e| e.into_diagnostic(&ctx))
                    .collect()
            }
            Ok(_) => vec![],
        },
    }
}

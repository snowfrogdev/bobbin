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
pub use resolver::{Resolver, SemanticError, SymbolTable, VariableDeclaration, VariableKind};
pub use scanner::{LexicalError, Scanner};
pub use token::{Span, Token, TokenKind};

/// Result of analyzing source code (for IDE tooling)
#[derive(Debug)]
pub struct AnalysisResult {
    /// Diagnostics (parse errors and semantic errors)
    pub diagnostics: Vec<Diagnostic>,
    /// All variable declarations found (available even with errors)
    pub declarations: Vec<VariableDeclaration>,
}

/// Analyze source code and return diagnostics and declarations.
///
/// This is the main entry point for IDE tooling. It runs the scanner,
/// parser, and resolver to collect all errors and variable declarations
/// without compiling or running the script.
///
/// Unlike `validate()`, this returns declarations even when there are
/// semantic errors, enabling autocomplete to work with partially valid code.
///
/// # Example
///
/// ```
/// use bobbin_syntax::analyze;
///
/// let source = "temp x = 1\nHello {x}!";
/// let result = analyze(source);
/// assert!(result.diagnostics.is_empty());
/// assert_eq!(result.declarations.len(), 1);
/// assert_eq!(result.declarations[0].name, "x");
/// ```
pub fn analyze(source: &str) -> AnalysisResult {
    let tokens = Scanner::new(source).tokens();
    match Parser::new(tokens).parse() {
        Err(errors) => {
            let matcher = JaroWinklerMatcher::default();
            let ctx = DiagnosticContext::new(&[], &matcher);
            AnalysisResult {
                diagnostics: errors.into_iter().map(|e| e.into_diagnostic(&ctx)).collect(),
                declarations: vec![], // No declarations on parse error (v1 simplification)
            }
        }
        Ok(ast) => {
            let (result, declarations, known_variables) = Resolver::new(&ast).analyze();
            match result {
                Err(errors) => {
                    let matcher = JaroWinklerMatcher::default();
                    let ctx = DiagnosticContext::new(&known_variables, &matcher);
                    AnalysisResult {
                        diagnostics: errors.into_iter().map(|e| e.into_diagnostic(&ctx)).collect(),
                        declarations, // Return declarations even with semantic errors
                    }
                }
                Ok(_) => AnalysisResult {
                    diagnostics: vec![],
                    declarations,
                },
            }
        }
    }
}

/// Validate source code and return diagnostics without executing.
///
/// This is a convenience wrapper around `analyze()` that returns only
/// diagnostics. Use `analyze()` if you also need variable declarations
/// for IDE features like autocomplete.
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
    analyze(source).diagnostics
}

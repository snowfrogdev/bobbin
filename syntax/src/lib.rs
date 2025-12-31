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
pub use token::{BOOLEAN_LITERALS, KEYWORDS, Span, Token, TokenKind};

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

    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => ast,
        Err(errors) => return make_parse_error_result(errors),
    };

    let (result, declarations, known_variables) = Resolver::new(&ast).analyze();

    match result {
        Ok(_) => AnalysisResult {
            diagnostics: vec![],
            declarations,
        },
        Err(errors) => make_semantic_error_result(errors, declarations, &known_variables),
    }
}

fn make_parse_error_result(errors: Vec<ParseError>) -> AnalysisResult {
    let matcher = JaroWinklerMatcher::default();
    let context = DiagnosticContext::new(&[], &matcher);
    AnalysisResult {
        diagnostics: errors
            .into_iter()
            .map(|e| e.into_diagnostic(&context))
            .collect(),
        declarations: vec![],
    }
}

fn make_semantic_error_result(
    errors: Vec<SemanticError>,
    declarations: Vec<VariableDeclaration>,
    known_variables: &[String],
) -> AnalysisResult {
    let matcher = JaroWinklerMatcher::default();
    let context = DiagnosticContext::new(known_variables, &matcher);
    AnalysisResult {
        diagnostics: errors
            .into_iter()
            .map(|e| e.into_diagnostic(&context))
            .collect(),
        declarations,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_valid_script() {
        let result = analyze("temp x = 1\nHello {x}!");
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.declarations.len(), 1);
        assert_eq!(result.declarations[0].name, "x");
    }

    #[test]
    fn analyze_undefined_variable() {
        let result = analyze("Hello {unknown}!");
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("undefined"));
    }

    #[test]
    fn analyze_returns_declarations_with_semantic_errors() {
        // Define a variable but also use an undefined one
        let result = analyze("temp x = 1\nHello {x} and {undefined}!");
        assert!(!result.diagnostics.is_empty());
        // Should still return the valid declaration
        assert_eq!(result.declarations.len(), 1);
        assert_eq!(result.declarations[0].name, "x");
    }

    #[test]
    fn analyze_parse_error_returns_no_declarations() {
        // Invalid syntax - unmatched brace
        let result = analyze("Hello {");
        assert!(!result.diagnostics.is_empty());
        assert!(result.declarations.is_empty());
    }
}

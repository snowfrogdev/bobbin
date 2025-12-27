//! Conversion traits for turning errors into diagnostics.
//!
//! The `IntoDiagnostic` trait provides a uniform way to convert different
//! error types into `Diagnostic` values for rendering.

use super::{Diagnostic, Matcher};

/// Context provided during diagnostic conversion.
///
/// This carries information needed to produce enhanced diagnostics,
/// such as the list of known variables for fuzzy matching.
pub struct DiagnosticContext<'a> {
    /// Known variable names for "did you mean?" suggestions.
    pub known_variables: &'a [String],
    /// The fuzzy matcher to use for suggestions.
    pub matcher: &'a dyn Matcher,
}

impl<'a> DiagnosticContext<'a> {
    /// Create a new context with the given variables and matcher.
    pub fn new(known_variables: &'a [String], matcher: &'a dyn Matcher) -> Self {
        Self {
            known_variables,
            matcher,
        }
    }

    /// Find a similar variable name for "did you mean?" suggestions.
    pub fn find_similar_variable(&self, name: &str) -> Option<&str> {
        self.matcher
            .best_match(name, self.known_variables)
            .map(|(s, _)| s)
    }
}

/// Trait for converting an error into a diagnostic.
///
/// Implemented by all error types in the pipeline (LexicalError, ParseError,
/// SemanticError, RuntimeError) to provide rich diagnostic output.
pub trait IntoDiagnostic {
    /// Convert this error into a diagnostic.
    ///
    /// The context provides information for enhanced diagnostics like
    /// "did you mean?" suggestions.
    fn into_diagnostic(self, ctx: &DiagnosticContext) -> Diagnostic;
}

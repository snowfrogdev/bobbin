//! Core diagnostic types for error reporting.
//!
//! These are pure data types with no rendering logic - rendering is handled
//! by the `Renderer` trait implementations.

use crate::token::Span;

/// A diagnostic message with source locations and optional suggestions.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The severity of this diagnostic.
    pub severity: Severity,
    /// The primary message describing the issue.
    pub message: String,
    /// Labeled spans in the source code.
    pub labels: Vec<Label>,
    /// Additional notes without source locations.
    pub notes: Vec<String>,
    /// Suggested fixes with replacement text.
    pub suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    /// Create a new error diagnostic with a primary label.
    pub fn error(message: impl Into<String>, span: Span, label: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            labels: vec![Label::primary(span, label)],
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Create a new warning diagnostic with a primary label.
    pub fn warning(message: impl Into<String>, span: Span, label: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            labels: vec![Label::primary(span, label)],
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Add a secondary label to this diagnostic.
    pub fn with_secondary(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label::secondary(span, message));
        self
    }

    /// Add a note to this diagnostic.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a suggestion with replacement text.
    pub fn with_suggestion(
        mut self,
        message: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
    ) -> Self {
        self.suggestions.push(Suggestion {
            message: message.into(),
            span,
            replacement: replacement.into(),
        });
        self
    }
}

/// The severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A fatal error that prevents compilation.
    Error,
    /// A warning that doesn't prevent compilation.
    Warning,
    /// An informational note.
    Note,
    /// A help message with suggestions.
    Help,
}

/// A labeled span in the source code.
#[derive(Debug, Clone)]
pub struct Label {
    /// The source span this label points to.
    pub span: Span,
    /// The message displayed with this label.
    pub message: String,
    /// The style of this label (primary or secondary).
    pub style: LabelStyle,
}

impl Label {
    /// Create a primary label (the main error location).
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            style: LabelStyle::Primary,
        }
    }

    /// Create a secondary label (supporting context).
    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            style: LabelStyle::Secondary,
        }
    }
}

/// The visual style of a label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    /// Primary label - the main error location (typically red).
    Primary,
    /// Secondary label - supporting context (typically blue).
    Secondary,
}

/// A suggested fix with replacement text.
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// A message describing the suggestion (e.g., "did you mean 'name'?").
    pub message: String,
    /// The span to replace.
    pub span: Span,
    /// The replacement text.
    pub replacement: String,
}

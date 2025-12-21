//! Diagnostic system for rich error reporting.
//!
//! This module provides infrastructure for producing Rust/Elm-quality error messages
//! with source snippets, multi-span labels, and auto-fix suggestions.
//!
//! # Architecture
//!
//! The diagnostic system follows the adapter pattern for flexibility:
//!
//! - [`Diagnostic`] - Pure data type representing an error/warning
//! - [`Renderer`] - Trait for rendering diagnostics (terminal, LSP, etc.)
//! - [`Matcher`] - Trait for fuzzy string matching ("did you mean?")
//!
//! External dependencies (ariadne, strsim) are wrapped behind traits,
//! allowing them to be swapped out if needed.

mod convert;
mod fuzzy;
mod render;
mod types;

pub use convert::{DiagnosticContext, IntoDiagnostic};
pub use fuzzy::{JaroWinklerMatcher, Matcher};
pub use render::{AriadneRenderer, Renderer};
pub use types::{Diagnostic, Label, LabelStyle, Severity, Suggestion};

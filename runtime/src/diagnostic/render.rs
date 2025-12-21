//! Renderer adapter for diagnostic output.
//!
//! The `Renderer` trait abstracts over different output formats (terminal, LSP, JSON).
//! This allows swapping rendering implementations without changing diagnostic logic.

use ariadne::{Color, Config, IndexType, Label as AriadneLabel, Report, ReportKind, Source};

use super::{Diagnostic, LabelStyle, Severity};

/// Trait for rendering diagnostics to a string.
///
/// This abstraction allows different rendering backends (terminal with colors,
/// plain text, LSP JSON) without modifying the core diagnostic types.
pub trait Renderer {
    /// Render a diagnostic to a string.
    ///
    /// # Arguments
    /// * `diagnostic` - The diagnostic to render
    /// * `source_id` - A name for the source (e.g., filename)
    /// * `source` - The source code text
    fn render(&self, diagnostic: &Diagnostic, source_id: &str, source: &str) -> String;

    /// Render multiple diagnostics to a string.
    fn render_all(&self, diagnostics: &[Diagnostic], source_id: &str, source: &str) -> String {
        diagnostics
            .iter()
            .map(|d| self.render(d, source_id, source))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Ariadne-based renderer for beautiful terminal output.
///
/// Produces colorized output with source snippets and underlines,
/// similar to Rust compiler errors.
#[derive(Debug, Default)]
pub struct AriadneRenderer {
    /// Whether to use colors in output.
    pub colors: bool,
}

impl AriadneRenderer {
    /// Create a new renderer with colors enabled.
    pub fn new() -> Self {
        Self { colors: true }
    }

    /// Create a new renderer without colors.
    pub fn without_colors() -> Self {
        Self { colors: false }
    }
}

impl Renderer for AriadneRenderer {
    fn render(&self, diagnostic: &Diagnostic, source_id: &str, source: &str) -> String {
        let kind = match diagnostic.severity {
            Severity::Error => ReportKind::Error,
            Severity::Warning => ReportKind::Warning,
            Severity::Note => ReportKind::Advice,
            Severity::Help => ReportKind::Advice,
        };

        // Start building the report with the first label's span as the primary location
        let offset = diagnostic.labels.first().map(|l| l.span.start).unwrap_or(0);

        let mut builder = Report::<(&str, std::ops::Range<usize>)>::build(kind, source_id, offset)
            .with_config(
                Config::default()
                    .with_color(self.colors)
                    .with_index_type(IndexType::Byte),
            )
            .with_message(&diagnostic.message);

        // Add labels
        for label in &diagnostic.labels {
            let color = match label.style {
                LabelStyle::Primary => Color::Red,
                LabelStyle::Secondary => Color::Blue,
            };

            let ariadne_label = AriadneLabel::new((source_id, label.span.start..label.span.end))
                .with_message(&label.message)
                .with_color(color);

            builder = builder.with_label(ariadne_label);
        }

        // Add notes
        for note in &diagnostic.notes {
            builder = builder.with_note(note);
        }

        // Add suggestions as help messages
        for suggestion in &diagnostic.suggestions {
            builder = builder.with_help(&suggestion.message);
        }

        let report = builder.finish();

        // Render to string
        let mut output = Vec::new();
        report
            .write((source_id, Source::from(source)), &mut output)
            .expect("write to Vec should not fail");

        String::from_utf8(output).expect("ariadne output should be valid UTF-8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Span;

    #[test]
    fn render_simple_error() {
        let diagnostic = Diagnostic::error(
            "undefined variable 'foo'",
            Span { start: 7, end: 10 },
            "not defined",
        );

        let renderer = AriadneRenderer::without_colors();
        let output = renderer.render(&diagnostic, "test.bobbin", "Hello, foo!");

        assert!(output.contains("undefined variable"));
        assert!(output.contains("not defined"));
    }

    #[test]
    fn render_with_suggestion() {
        let diagnostic = Diagnostic::error(
            "undefined variable 'naem'",
            Span { start: 7, end: 11 },
            "not defined",
        )
        .with_suggestion("did you mean 'name'?", Span { start: 7, end: 11 }, "name");

        let renderer = AriadneRenderer::without_colors();
        let output = renderer.render(&diagnostic, "test.bobbin", "Hello, naem!");

        assert!(output.contains("did you mean"));
    }

    #[test]
    fn render_with_secondary_label() {
        let diagnostic = Diagnostic::error(
            "variable 'x' shadows previous declaration",
            Span { start: 20, end: 21 },
            "shadows previous declaration",
        )
        .with_secondary(Span { start: 5, end: 6 }, "previously declared here");

        let renderer = AriadneRenderer::without_colors();
        let source = "temp x = 1\ntemp x = 2";
        let output = renderer.render(&diagnostic, "test.bobbin", source);

        assert!(output.contains("shadows"));
        assert!(output.contains("previously declared"));
    }

    #[test]
    fn render_multiline() {
        // Test that multiline source renders correctly
        let source = "line1\nline2\nerror here";

        // "here" starts at byte 18
        let span_start = source.find("here").unwrap();
        let span_end = span_start + 4;

        let diagnostic = Diagnostic::error(
            "test error",
            Span {
                start: span_start,
                end: span_end,
            },
            "error at 'here'",
        );

        let renderer = AriadneRenderer::without_colors();
        let output = renderer.render(&diagnostic, "test.bobbin", source);

        assert!(output.contains("test error"));
        assert!(output.contains("here"));
        assert!(output.contains("error at 'here'"));
    }
}

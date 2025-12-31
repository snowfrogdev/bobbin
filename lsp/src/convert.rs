//! Conversion utilities from Bobbin diagnostics to LSP types.

use bobbin_syntax::{Diagnostic, LineIndex, Severity};
use tower_lsp::lsp_types;

/// Convert Bobbin diagnostics to LSP diagnostics.
pub fn to_lsp_diagnostics(
    diagnostics: &[Diagnostic],
    line_index: &LineIndex,
    use_utf16: bool,
) -> Vec<lsp_types::Diagnostic> {
    diagnostics
        .iter()
        .map(|diag| to_lsp_diagnostic(diag, line_index, use_utf16))
        .collect()
}

/// Convert a single Bobbin diagnostic to an LSP diagnostic.
fn to_lsp_diagnostic(
    diag: &Diagnostic,
    line_index: &LineIndex,
    use_utf16: bool,
) -> lsp_types::Diagnostic {
    let range = diag
        .primary_label()
        .map(|label| {
            let start = line_index.to_lsp_position(label.span.start, use_utf16);
            let end = line_index.to_lsp_position(label.span.end, use_utf16);
            lsp_types::Range::new(
                lsp_types::Position::new(start.line, start.column),
                lsp_types::Position::new(end.line, end.column),
            )
        })
        .unwrap_or_else(|| {
            // No primary label - use start of file
            lsp_types::Range::new(
                lsp_types::Position::new(0, 0),
                lsp_types::Position::new(0, 0),
            )
        });

    // Collect related information from secondary labels
    let related_information = if diag.labels.len() > 1 {
        Some(
            diag.labels
                .iter()
                .filter(|l| l.style != bobbin_syntax::LabelStyle::Primary)
                .map(|label| {
                    let start = line_index.to_lsp_position(label.span.start, use_utf16);
                    let end = line_index.to_lsp_position(label.span.end, use_utf16);
                    lsp_types::DiagnosticRelatedInformation {
                        location: lsp_types::Location {
                            // We don't have the URI here, so we use a placeholder
                            // In practice, secondary labels are in the same file
                            uri: lsp_types::Url::parse("file:///").unwrap(),
                            range: lsp_types::Range::new(
                                lsp_types::Position::new(start.line, start.column),
                                lsp_types::Position::new(end.line, end.column),
                            ),
                        },
                        message: label.message.clone(),
                    }
                })
                .collect(),
        )
    } else {
        None
    };

    // Build complete message including notes and suggestions
    let mut message = diag.message.clone();
    for note in &diag.notes {
        message.push_str("\n\nNote: ");
        message.push_str(note);
    }
    for suggestion in &diag.suggestions {
        message.push_str("\n\nHelp: ");
        message.push_str(&suggestion.message);
    }

    lsp_types::Diagnostic {
        range,
        severity: Some(to_lsp_severity(diag.severity)),
        code: None,
        code_description: None,
        source: Some("bobbin".to_string()),
        message,
        related_information,
        tags: None,
        data: None,
    }
}

/// Convert Bobbin severity to LSP severity.
fn to_lsp_severity(severity: Severity) -> lsp_types::DiagnosticSeverity {
    match severity {
        Severity::Error => lsp_types::DiagnosticSeverity::ERROR,
        Severity::Warning => lsp_types::DiagnosticSeverity::WARNING,
        Severity::Note => lsp_types::DiagnosticSeverity::INFORMATION,
        Severity::Help => lsp_types::DiagnosticSeverity::HINT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_lsp_severity_mapping() {
        assert_eq!(
            to_lsp_severity(Severity::Error),
            lsp_types::DiagnosticSeverity::ERROR
        );
        assert_eq!(
            to_lsp_severity(Severity::Warning),
            lsp_types::DiagnosticSeverity::WARNING
        );
        assert_eq!(
            to_lsp_severity(Severity::Note),
            lsp_types::DiagnosticSeverity::INFORMATION
        );
        assert_eq!(
            to_lsp_severity(Severity::Help),
            lsp_types::DiagnosticSeverity::HINT
        );
    }

    #[test]
    fn to_lsp_diagnostic_no_primary_label() {
        let diag = Diagnostic {
            message: "test error".to_string(),
            severity: Severity::Error,
            labels: vec![],
            notes: vec![],
            suggestions: vec![],
        };
        let line_index = LineIndex::new("");

        let lsp_diag = to_lsp_diagnostic(&diag, &line_index, false);

        // Should fall back to position 0,0
        assert_eq!(lsp_diag.range.start.line, 0);
        assert_eq!(lsp_diag.range.start.character, 0);
        assert_eq!(lsp_diag.range.end.line, 0);
        assert_eq!(lsp_diag.range.end.character, 0);
    }

    #[test]
    fn to_lsp_diagnostic_includes_notes_and_suggestions() {
        use bobbin_syntax::Span;

        let diag = Diagnostic {
            message: "main error".to_string(),
            severity: Severity::Error,
            labels: vec![],
            notes: vec!["a note".to_string()],
            suggestions: vec![bobbin_syntax::Suggestion {
                message: "try this".to_string(),
                span: Span { start: 0, end: 1 },
                replacement: "fixed".to_string(),
            }],
        };
        let line_index = LineIndex::new("");

        let lsp_diag = to_lsp_diagnostic(&diag, &line_index, false);

        assert!(lsp_diag.message.contains("main error"));
        assert!(lsp_diag.message.contains("Note: a note"));
        assert!(lsp_diag.message.contains("Help: try this"));
    }
}

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

    lsp_types::Diagnostic {
        range,
        severity: Some(to_lsp_severity(diag.severity)),
        code: None,
        code_description: None,
        source: Some("bobbin".to_string()),
        message: diag.message.clone(),
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

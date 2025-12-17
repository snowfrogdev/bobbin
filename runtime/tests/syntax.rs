//! Syntax and lexical error tests.

mod support;

#[test]
fn errors_tabs() {
    support::run_error_test(&support::cases_dir().join("syntax/errors/tabs.bobbin"));
}

#[test]
fn errors_unclosed_interpolation() {
    support::run_error_test(&support::cases_dir().join("syntax/errors/unclosed_interpolation.bobbin"));
}

#[test]
fn errors_empty_interpolation() {
    support::run_error_test(&support::cases_dir().join("syntax/errors/empty_interpolation.bobbin"));
}

#[test]
fn errors_lone_closing_brace() {
    support::run_error_test(&support::cases_dir().join("syntax/errors/lone_closing_brace.bobbin"));
}

// =============================================================================
// Assignment Syntax Errors
// =============================================================================

#[test]
fn errors_set_missing_identifier() {
    support::run_error_test(&support::cases_dir().join("syntax/errors/set_missing_identifier.bobbin"));
}

#[test]
fn errors_set_missing_equals() {
    support::run_error_test(&support::cases_dir().join("syntax/errors/set_missing_equals.bobbin"));
}

#[test]
fn errors_set_missing_value() {
    support::run_error_test(&support::cases_dir().join("syntax/errors/set_missing_value.bobbin"));
}

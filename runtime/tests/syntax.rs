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

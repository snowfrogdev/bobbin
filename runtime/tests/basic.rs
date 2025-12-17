//! Basic dialogue tests - simple lines and empty handling.

mod support;

use bobbin_runtime::Runtime;

#[test]
fn simple_lines() {
    support::run_output_test(&support::cases_dir().join("basic/simple_lines.bobbin"));
}

#[test]
fn empty_lines() {
    support::run_output_test(&support::cases_dir().join("basic/empty_lines.bobbin"));
}

#[test]
fn empty_source() {
    // Special case: empty source produces empty output
    let runtime = Runtime::new("").unwrap();
    assert_eq!(runtime.current_line(), "");
    assert!(!runtime.has_more());
}

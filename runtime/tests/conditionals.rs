//! Conditional statement tests (if/elif/else).

mod support;

// =============================================================================
// Basic If Statement
// =============================================================================

#[test]
fn if_basic() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_basic.bobbin"));
}

#[test]
fn if_skipped() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_skipped.bobbin"));
}

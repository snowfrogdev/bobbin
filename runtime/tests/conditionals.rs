//! Tests for conditional statements (if/elseif/else).

mod support;

// ============================================
// Basic if/else Tests
// ============================================

#[test]
fn if_true() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_true.bobbin"));
}

#[test]
fn if_false() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_false.bobbin"));
}

#[test]
fn if_else_true() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_else_true.bobbin"));
}

#[test]
fn if_else_false() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_else_false.bobbin"));
}

// ============================================
// Elseif Chain Tests
// ============================================

#[test]
fn if_elseif_else() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_elseif_else.bobbin"));
}

#[test]
fn if_multiple_elseif() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_multiple_elseif.bobbin"));
}

#[test]
fn if_only() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_only.bobbin"));
}

#[test]
fn if_elseif_no_else() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_elseif_no_else.bobbin"));
}

// ============================================
// Variable and Expression Tests
// ============================================

#[test]
fn if_comparison() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_comparison.bobbin"));
}

#[test]
fn if_logical_and() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_logical_and.bobbin"));
}

#[test]
fn if_logical_or() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_logical_or.bobbin"));
}

#[test]
fn if_logical_not() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_logical_not.bobbin"));
}

#[test]
fn if_variable() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_variable.bobbin"));
}

#[test]
fn if_arithmetic() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_arithmetic.bobbin"));
}

#[test]
fn if_complex_expression() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_complex_expression.bobbin"));
}

// ============================================
// Nesting Tests
// ============================================

#[test]
fn if_nested() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_nested.bobbin"));
}

#[test]
fn if_with_set() {
    support::run_output_test(&support::cases_dir().join("conditionals/if_with_set.bobbin"));
}

#[test]
fn if_in_choice_select_a() {
    support::run_trace_test(
        &support::cases_dir().join("conditionals/if_in_choice.bobbin"),
        "select_a",
    );
}

#[test]
fn if_in_choice_select_b() {
    support::run_trace_test(
        &support::cases_dir().join("conditionals/if_in_choice.bobbin"),
        "select_b",
    );
}

#[test]
fn choice_in_if_first() {
    support::run_trace_test(
        &support::cases_dir().join("conditionals/choice_in_if.bobbin"),
        "first",
    );
}

#[test]
fn choice_in_if_second() {
    support::run_trace_test(
        &support::cases_dir().join("conditionals/choice_in_if.bobbin"),
        "second",
    );
}

// ============================================
// Error Tests
// ============================================

#[test]
fn errors_if_condition_number() {
    support::run_error_test(
        &support::cases_dir().join("conditionals/errors/if_condition_number.bobbin"),
    );
}

#[test]
fn errors_if_condition_string() {
    support::run_error_test(
        &support::cases_dir().join("conditionals/errors/if_condition_string.bobbin"),
    );
}

#[test]
fn errors_elseif_condition_number() {
    support::run_error_test(
        &support::cases_dir().join("conditionals/errors/elseif_condition_number.bobbin"),
    );
}

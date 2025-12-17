//! Variable and interpolation tests.

mod support;

// =============================================================================
// Basic Interpolation
// =============================================================================

#[test]
fn interpolation() {
    support::run_output_test(&support::cases_dir().join("variables/interpolation.bobbin"));
}

#[test]
fn multiple() {
    support::run_output_test(&support::cases_dir().join("variables/multiple.bobbin"));
}

#[test]
fn escaped_braces() {
    support::run_output_test(&support::cases_dir().join("variables/escaped_braces.bobbin"));
}

#[test]
fn multiple_uses() {
    support::run_output_test(&support::cases_dir().join("variables/multiple_uses.bobbin"));
}

// =============================================================================
// Type-specific Interpolation
// =============================================================================

#[test]
fn types_integer() {
    support::run_output_test(&support::cases_dir().join("variables/types/integer.bobbin"));
}

#[test]
fn types_float() {
    support::run_output_test(&support::cases_dir().join("variables/types/float.bobbin"));
}

#[test]
fn types_boolean() {
    support::run_output_test(&support::cases_dir().join("variables/types/boolean.bobbin"));
}

#[test]
fn types_negative() {
    support::run_output_test(&support::cases_dir().join("variables/types/negative.bobbin"));
}

#[test]
fn types_empty_string() {
    support::run_output_test(&support::cases_dir().join("variables/types/empty_string.bobbin"));
}

// =============================================================================
// Variables in Choices
// =============================================================================

#[test]
fn in_choices_choice_text_keep() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/choice_text.bobbin"),
        "keep",
    );
}

#[test]
fn in_choices_choice_text_drop() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/choice_text.bobbin"),
        "drop",
    );
}

#[test]
fn in_choices_outer_scope_enter() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/outer_scope.bobbin"),
        "enter_cave",
    );
}

#[test]
fn in_choices_outer_scope_stay() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/outer_scope.bobbin"),
        "stay_outside",
    );
}

#[test]
fn in_choices_nested_left() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/nested.bobbin"),
        "left",
    );
}

#[test]
fn in_choices_nested_right() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/nested.bobbin"),
        "right",
    );
}

#[test]
fn in_choices_sibling_reuse_a() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/sibling_reuse.bobbin"),
        "path_a",
    );
}

#[test]
fn in_choices_sibling_reuse_b() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/sibling_reuse.bobbin"),
        "path_b",
    );
}

// =============================================================================
// Semantic Errors
// =============================================================================

#[test]
fn errors_undefined() {
    support::run_error_test(&support::cases_dir().join("variables/errors/undefined.bobbin"));
}

#[test]
fn errors_shadowing() {
    support::run_error_test(&support::cases_dir().join("variables/errors/shadowing.bobbin"));
}

#[test]
fn errors_redeclaration() {
    support::run_error_test(&support::cases_dir().join("variables/errors/redeclaration.bobbin"));
}

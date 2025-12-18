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
// Assignment
// =============================================================================

#[test]
fn assignment() {
    support::run_output_test(&support::cases_dir().join("variables/assignment.bobbin"));
}

#[test]
fn assignment_multiple() {
    support::run_output_test(&support::cases_dir().join("variables/assignment_multiple.bobbin"));
}

#[test]
fn assignment_types() {
    support::run_output_test(&support::cases_dir().join("variables/assignment_types.bobbin"));
}

// =============================================================================
// Save Variables
// =============================================================================

#[test]
fn save_basic() {
    support::run_trace_test(
        &support::cases_dir().join("variables/save/basic.bobbin"),
        "basic",
    );
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
fn in_choices_outer_scope_enter_cave() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/outer_scope.bobbin"),
        "enter_cave",
    );
}

#[test]
fn in_choices_outer_scope_stay_outside() {
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
fn in_choices_sibling_reuse_path_a() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/sibling_reuse.bobbin"),
        "path_a",
    );
}

#[test]
fn in_choices_sibling_reuse_path_b() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/sibling_reuse.bobbin"),
        "path_b",
    );
}

#[test]
fn in_choices_outer_scope_assignment_cheer_up() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/outer_scope_assignment.bobbin"),
        "cheer_up",
    );
}

#[test]
fn in_choices_outer_scope_assignment_get_angry() {
    support::run_trace_test(
        &support::cases_dir().join("variables/in_choices/outer_scope_assignment.bobbin"),
        "get_angry",
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

#[test]
fn errors_assignment_undefined() {
    support::run_error_test(&support::cases_dir().join("variables/errors/assignment_undefined.bobbin"));
}

#[test]
fn errors_assignment_typo() {
    support::run_error_test(&support::cases_dir().join("variables/errors/assignment_typo.bobbin"));
}

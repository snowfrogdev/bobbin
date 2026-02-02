//! Command invocation tests.

mod support;

// =============================================================================
// Basic Command Invocation
// =============================================================================

#[test]
fn command_basic() {
    support::run_trace_test(
        &support::cases_dir().join("commands/basic.bobbin"),
        "basic",
    );
}

#[test]
fn command_multiple_args() {
    support::run_trace_test(
        &support::cases_dir().join("commands/multiple_args.bobbin"),
        "multiple_args",
    );
}

#[test]
fn command_no_args() {
    support::run_trace_test(
        &support::cases_dir().join("commands/no_args.bobbin"),
        "no_args",
    );
}

#[test]
fn command_in_choices_buy_potion() {
    support::run_trace_test(
        &support::cases_dir().join("commands/in_choices.bobbin"),
        "buy_potion",
    );
}

#[test]
fn command_in_choices_leave() {
    support::run_trace_test(
        &support::cases_dir().join("commands/in_choices.bobbin"),
        "leave",
    );
}

#[test]
fn command_with_expressions() {
    support::run_trace_test(
        &support::cases_dir().join("commands/with_expressions.bobbin"),
        "with_expressions",
    );
}

#[test]
fn command_variable_refs() {
    support::run_trace_test(
        &support::cases_dir().join("commands/variable_refs.bobbin"),
        "variable_refs",
    );
}

#[test]
fn command_negative_args() {
    support::run_trace_test(
        &support::cases_dir().join("commands/negative_args.bobbin"),
        "negative_args",
    );
}

#[test]
fn command_mixed_with_dialogue() {
    support::run_trace_test(
        &support::cases_dir().join("commands/mixed_with_dialogue.bobbin"),
        "mixed_with_dialogue",
    );
}

// =============================================================================
// Error Cases
// =============================================================================

#[test]
fn errors_undeclared_command() {
    support::run_error_test(
        &support::cases_dir().join("commands/errors/undeclared_command.bobbin"),
    );
}

#[test]
fn errors_command_shadows_variable() {
    support::run_error_test(
        &support::cases_dir().join("commands/errors/command_shadows_variable.bobbin"),
    );
}

#[test]
fn errors_variable_shadows_command() {
    support::run_error_test(
        &support::cases_dir().join("commands/errors/variable_shadows_command.bobbin"),
    );
}

#[test]
fn errors_wrong_arity_too_few() {
    support::run_error_test(
        &support::cases_dir().join("commands/errors/wrong_arity_too_few.bobbin"),
    );
}

#[test]
fn errors_wrong_arity_too_many() {
    support::run_error_test(
        &support::cases_dir().join("commands/errors/wrong_arity_too_many.bobbin"),
    );
}

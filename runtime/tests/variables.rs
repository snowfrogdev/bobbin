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
// Assignment with Expressions
// =============================================================================

#[test]
fn assignment_arithmetic_temp() {
    support::run_output_test(&support::cases_dir().join("variables/assignment_arithmetic_temp.bobbin"));
}

#[test]
fn assignment_arithmetic_save() {
    support::run_output_test(&support::cases_dir().join("variables/assignment_arithmetic_save.bobbin"));
}

#[test]
fn assignment_arithmetic_set() {
    support::run_output_test(&support::cases_dir().join("variables/assignment_arithmetic_set.bobbin"));
}

#[test]
fn assignment_comparison_result() {
    support::run_output_test(&support::cases_dir().join("variables/assignment_comparison_result.bobbin"));
}

#[test]
fn assignment_logical_result() {
    support::run_output_test(&support::cases_dir().join("variables/assignment_logical_result.bobbin"));
}

#[test]
fn assignment_variable_ref() {
    support::run_output_test(&support::cases_dir().join("variables/assignment_variable_ref.bobbin"));
}

#[test]
fn assignment_complex_nested() {
    support::run_output_test(&support::cases_dir().join("variables/assignment_complex_nested.bobbin"));
}

#[test]
fn assignment_precedence() {
    support::run_output_test(&support::cases_dir().join("variables/assignment_precedence.bobbin"));
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

#[test]
fn save_types() {
    support::run_trace_test(
        &support::cases_dir().join("variables/save/types.bobbin"),
        "basic",
    );
}

#[test]
fn save_multiple() {
    support::run_trace_test(
        &support::cases_dir().join("variables/save/multiple.bobbin"),
        "basic",
    );
}

#[test]
fn save_assignment() {
    support::run_trace_test(
        &support::cases_dir().join("variables/save/assignment.bobbin"),
        "basic",
    );
}

#[test]
fn save_in_choices_happy() {
    support::run_trace_test(
        &support::cases_dir().join("variables/save/in_choices.bobbin"),
        "happy",
    );
}

#[test]
fn save_in_choices_sad() {
    support::run_trace_test(
        &support::cases_dir().join("variables/save/in_choices.bobbin"),
        "sad",
    );
}

#[test]
fn save_mixed_with_temp() {
    support::run_trace_test(
        &support::cases_dir().join("variables/save/mixed_with_temp.bobbin"),
        "basic",
    );
}

// =============================================================================
// Save Variable Pre-population (Storage Persistence Pattern)
// =============================================================================
// These tests verify the core runtime supports pre-populated storage as described
// in ADR-0002/0004. This enables save game restoration to work correctly.

/// Test that pre-populated storage values are preserved by initialize_if_absent.
/// This verifies the core runtime supports the persistence pattern.
#[test]
fn save_variable_prepopulated_storage_preserved() {
    use bobbin_runtime::{HostState, Runtime, Value, VariableStorage};
    use std::sync::Arc;
    use support::{EmptyHostState, MemoryStorage};

    let source = r#"
save has_talked_once = 0

if has_talked_once < 1
    First time message
else
    Repeat message
"#;

    // Pre-populate storage BEFORE creating runtime (simulates restored save game)
    let storage = Arc::new(MemoryStorage::new());
    storage.set("has_talked_once", Value::Number(1.0));

    let runtime = Runtime::new(
        source,
        storage.clone() as Arc<dyn VariableStorage>,
        Arc::new(EmptyHostState) as Arc<dyn HostState>,
    )
    .expect("Runtime creation should succeed");

    // initialize_if_absent should NOT overwrite pre-populated value
    assert_eq!(
        storage.get("has_talked_once"),
        Some(Value::Number(1.0)),
        "Pre-populated value should be preserved by initialize_if_absent"
    );

    // Condition evaluated with pre-populated value (1 < 1 = false) -> else branch
    assert_eq!(
        runtime.current_line(),
        "Repeat message",
        "Should show repeat message because pre-populated value is 1"
    );
}

/// Test that fresh storage gets initialized to default value.
/// This is the "first run" case.
#[test]
fn save_variable_fresh_storage_initialized() {
    use bobbin_runtime::{HostState, Runtime, Value, VariableStorage};
    use std::sync::Arc;
    use support::{EmptyHostState, MemoryStorage};

    let source = r#"
save has_talked_once = 0

if has_talked_once < 1
    First time message
else
    Repeat message
"#;

    // Fresh storage (no pre-population)
    let storage = Arc::new(MemoryStorage::new());

    let runtime = Runtime::new(
        source,
        storage.clone() as Arc<dyn VariableStorage>,
        Arc::new(EmptyHostState) as Arc<dyn HostState>,
    )
    .expect("Runtime creation should succeed");

    // initialize_if_absent should set value to 0
    assert_eq!(
        storage.get("has_talked_once"),
        Some(Value::Number(0.0)),
        "Fresh storage should be initialized to default value"
    );

    // Condition evaluated with initial value (0 < 1 = true) -> if branch
    assert_eq!(
        runtime.current_line(),
        "First time message",
        "Should show first time message because initial value is 0"
    );
}

/// Test full persistence cycle: first run -> set variable -> second run with shared storage.
#[test]
fn save_variable_persistence_across_runtimes() {
    use bobbin_runtime::{HostState, Runtime, Value, VariableStorage};
    use std::sync::Arc;
    use support::{EmptyHostState, MemoryStorage};

    let source = r#"
save has_talked_once = 0

if has_talked_once < 1
    First time message
else
    Repeat message
set has_talked_once = 1
"#;

    // Shared storage persists across runtime instances
    let storage = Arc::new(MemoryStorage::new());
    let host: Arc<dyn HostState> = Arc::new(EmptyHostState);

    // === First runtime (first conversation) ===
    {
        let mut runtime = Runtime::new(
            source,
            storage.clone() as Arc<dyn VariableStorage>,
            Arc::clone(&host),
        )
        .expect("Runtime creation should succeed");

        assert_eq!(
            storage.get("has_talked_once"),
            Some(Value::Number(0.0)),
            "First run: should initialize to 0"
        );

        assert_eq!(
            runtime.current_line(),
            "First time message",
            "First run: should show first time message"
        );

        // Run to completion (executes set has_talked_once = 1)
        while runtime.has_more() {
            runtime.advance().expect("advance should succeed");
        }

        assert_eq!(
            storage.get("has_talked_once"),
            Some(Value::Number(1.0)),
            "First run: should be 1 after set statement"
        );
    }

    // === Second runtime (second conversation, same storage) ===
    {
        let runtime = Runtime::new(
            source,
            storage.clone() as Arc<dyn VariableStorage>,
            Arc::clone(&host),
        )
        .expect("Runtime creation should succeed");

        // initialize_if_absent should preserve the value from first run
        assert_eq!(
            storage.get("has_talked_once"),
            Some(Value::Number(1.0)),
            "Second run: should preserve value from first run"
        );

        // Condition now evaluates with persisted value (1 < 1 = false)
        assert_eq!(
            runtime.current_line(),
            "Repeat message",
            "Second run: should show repeat message"
        );
    }
}

// =============================================================================
// Extern Variables (Host State)
// =============================================================================

#[test]
fn extern_basic() {
    support::run_trace_test(
        &support::cases_dir().join("variables/extern/basic.bobbin"),
        "basic",
    );
}

#[test]
fn extern_types() {
    support::run_trace_test(
        &support::cases_dir().join("variables/extern/types.bobbin"),
        "basic",
    );
}

#[test]
fn extern_multiple() {
    support::run_trace_test(
        &support::cases_dir().join("variables/extern/multiple.bobbin"),
        "basic",
    );
}

#[test]
fn extern_in_choices_buy() {
    support::run_trace_test(
        &support::cases_dir().join("variables/extern/in_choices.bobbin"),
        "buy",
    );
}

#[test]
fn extern_in_choices_leave() {
    support::run_trace_test(
        &support::cases_dir().join("variables/extern/in_choices.bobbin"),
        "leave",
    );
}

#[test]
fn extern_mixed_with_temp_and_save() {
    support::run_trace_test(
        &support::cases_dir().join("variables/extern/mixed_with_temp_and_save.bobbin"),
        "basic",
    );
}

#[test]
fn extern_missing_at_runtime() {
    // Test that using a declared extern variable that the host doesn't provide
    // results in a runtime error (MissingExternVariable)
    use bobbin_runtime::{HostState, Runtime, RuntimeError, VariableStorage};
    use std::sync::Arc;
    use support::{EmptyHostState, MemoryStorage};

    let source = "extern player_health\n\nYou have {player_health} HP.\n";

    let storage: Arc<dyn VariableStorage> = Arc::new(MemoryStorage::new());
    let host: Arc<dyn HostState> = Arc::new(EmptyHostState); // Doesn't provide player_health

    // Runtime::new steps on creation, which will try to interpolate the extern variable
    let result = Runtime::new(source, storage, host);

    // Should fail with MissingExternVariable since EmptyHostState doesn't provide it
    match result {
        Err(bobbin_runtime::BobbinError::Runtime(RuntimeError::MissingExternVariable { name })) => {
            assert_eq!(name, "player_health");
        }
        Err(e) => panic!("Expected MissingExternVariable error, got: {:?}", e),
        Ok(_) => panic!("Expected MissingExternVariable error, but runtime succeeded"),
    }
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
// String Escape Sequences
// =============================================================================

#[test]
fn string_escape_newline() {
    support::run_output_test(&support::cases_dir().join("variables/string_escape_newline.bobbin"));
}

#[test]
fn string_escape_tab() {
    support::run_output_test(&support::cases_dir().join("variables/string_escape_tab.bobbin"));
}

#[test]
fn string_escape_quote() {
    support::run_output_test(&support::cases_dir().join("variables/string_escape_quote.bobbin"));
}

#[test]
fn string_escape_backslash() {
    support::run_output_test(&support::cases_dir().join("variables/string_escape_backslash.bobbin"));
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
    support::run_error_test(
        &support::cases_dir().join("variables/errors/assignment_undefined.bobbin"),
    );
}

#[test]
fn errors_assignment_typo() {
    support::run_error_test(&support::cases_dir().join("variables/errors/assignment_typo.bobbin"));
}

#[test]
fn errors_temp_shadows_save() {
    support::run_error_test(
        &support::cases_dir().join("variables/errors/temp_shadows_save.bobbin"),
    );
}

#[test]
fn errors_save_shadows_temp() {
    support::run_error_test(
        &support::cases_dir().join("variables/errors/save_shadows_temp.bobbin"),
    );
}

#[test]
fn errors_save_redeclaration() {
    support::run_error_test(
        &support::cases_dir().join("variables/errors/save_redeclaration.bobbin"),
    );
}

#[test]
fn errors_save_set_undefined() {
    support::run_error_test(
        &support::cases_dir().join("variables/errors/save_set_undefined.bobbin"),
    );
}

// =============================================================================
// Extern Semantic Errors
// =============================================================================

#[test]
fn errors_extern_assignment() {
    support::run_error_test(
        &support::cases_dir().join("variables/errors/extern_assignment.bobbin"),
    );
}

#[test]
fn errors_extern_redeclaration() {
    support::run_error_test(
        &support::cases_dir().join("variables/errors/extern_redeclaration.bobbin"),
    );
}

#[test]
fn errors_extern_shadows_temp() {
    support::run_error_test(
        &support::cases_dir().join("variables/errors/extern_shadows_temp.bobbin"),
    );
}

#[test]
fn errors_temp_shadows_extern() {
    support::run_error_test(
        &support::cases_dir().join("variables/errors/temp_shadows_extern.bobbin"),
    );
}

#[test]
fn errors_extern_shadows_save() {
    support::run_error_test(
        &support::cases_dir().join("variables/errors/extern_shadows_save.bobbin"),
    );
}

#[test]
fn errors_save_shadows_extern() {
    support::run_error_test(
        &support::cases_dir().join("variables/errors/save_shadows_extern.bobbin"),
    );
}

#[test]
fn errors_extern_undefined() {
    support::run_error_test(&support::cases_dir().join("variables/errors/extern_undefined.bobbin"));
}

// =============================================================================
// Assignment Expression Errors
// =============================================================================

#[test]
fn errors_assignment_forward_ref() {
    support::run_error_test(&support::cases_dir().join("variables/errors/assignment_forward_ref.bobbin"));
}

#[test]
fn errors_assignment_type_mismatch() {
    support::run_error_test(&support::cases_dir().join("variables/errors/assignment_type_mismatch.bobbin"));
}

#[test]
fn errors_assignment_undefined_var() {
    support::run_error_test(&support::cases_dir().join("variables/errors/assignment_undefined_var.bobbin"));
}

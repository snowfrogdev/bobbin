//! Choice and branching tests.

mod support;

// =============================================================================
// Basic Choices
// =============================================================================

#[test]
fn basic_select_first() {
    support::run_trace_test(
        &support::cases_dir().join("choices/basic.bobbin"),
        "select_first",
    );
}

#[test]
fn basic_select_second() {
    support::run_trace_test(
        &support::cases_dir().join("choices/basic.bobbin"),
        "select_second",
    );
}

// =============================================================================
// Choices with Content
// =============================================================================

#[test]
fn with_content_select_first() {
    support::run_trace_test(
        &support::cases_dir().join("choices/with_content.bobbin"),
        "select_first",
    );
}

#[test]
fn with_content_select_second() {
    support::run_trace_test(
        &support::cases_dir().join("choices/with_content.bobbin"),
        "select_second",
    );
}

// =============================================================================
// Empty Choices (no content, go to gather)
// =============================================================================

#[test]
fn empty_select_first() {
    support::run_trace_test(
        &support::cases_dir().join("choices/empty.bobbin"),
        "select_first",
    );
}

#[test]
fn empty_select_second() {
    support::run_trace_test(
        &support::cases_dir().join("choices/empty.bobbin"),
        "select_second",
    );
}

// =============================================================================
// Gather Points
// =============================================================================

#[test]
fn gather_door_a() {
    support::run_trace_test(
        &support::cases_dir().join("choices/gather.bobbin"),
        "door_a",
    );
}

#[test]
fn gather_door_b() {
    support::run_trace_test(
        &support::cases_dir().join("choices/gather.bobbin"),
        "door_b",
    );
}

// =============================================================================
// Sequential Choices
// =============================================================================

#[test]
fn sequential() {
    support::run_trace_test(
        &support::cases_dir().join("choices/sequential.bobbin"),
        "yes_blue",
    );
}

// =============================================================================
// Mixed (some with content, some without)
// =============================================================================

#[test]
fn mixed_talk() {
    support::run_trace_test(
        &support::cases_dir().join("choices/mixed.bobbin"),
        "talk",
    );
}

#[test]
fn mixed_leave() {
    support::run_trace_test(
        &support::cases_dir().join("choices/mixed.bobbin"),
        "leave",
    );
}

// =============================================================================
// Nested Choices
// =============================================================================

#[test]
fn nested_talk_to_alice() {
    support::run_trace_test(
        &support::cases_dir().join("choices/nested.bobbin"),
        "talk_to_alice",
    );
}

#[test]
fn nested_talk_to_bob() {
    support::run_trace_test(
        &support::cases_dir().join("choices/nested.bobbin"),
        "talk_to_bob",
    );
}

#[test]
fn nested_leave() {
    support::run_trace_test(
        &support::cases_dir().join("choices/nested.bobbin"),
        "leave",
    );
}

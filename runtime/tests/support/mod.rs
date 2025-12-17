//! Test support utilities for bobbin runtime e2e tests.
//!
//! This module provides infrastructure for running data-driven tests using
//! sidecar files that specify expected outputs.

use bobbin_runtime::Runtime;
use std::path::Path;

// =============================================================================
// Trace File Data Structures
// =============================================================================

/// A named execution path through a test case.
#[derive(Debug)]
pub struct TracePath {
    pub name: String,
    pub steps: Vec<Step>,
}

/// A single step in a trace execution.
#[derive(Debug)]
pub enum Step {
    Assert(Assertion),
    Action(Action),
}

/// An assertion to verify runtime state.
#[derive(Debug)]
pub enum Assertion {
    /// Assert current_line() equals the given text
    Line(String),
    /// Assert current_choices() equals the given list
    Choices(Vec<String>),
    /// Assert has_more() is false
    Done,
    /// Assert has_more() is true
    HasMore,
    /// Assert is_waiting_for_choice() is true
    WaitingForChoice,
}

/// An action to perform on the runtime.
#[derive(Debug)]
pub enum Action {
    /// Call advance()
    Advance,
    /// Call select_choice(index)
    SelectChoice(usize),
}

// =============================================================================
// Test Runner Functions
// =============================================================================

/// Run a linear output test (.out sidecar).
///
/// Executes the script and compares all output lines against the expected output.
pub fn run_output_test(case_path: &Path) {
    let source = std::fs::read_to_string(case_path)
        .unwrap_or_else(|e| panic!("Failed to read test case {}: {}", case_path.display(), e));

    let out_path = case_path.with_extension("out");
    let expected = std::fs::read_to_string(&out_path)
        .unwrap_or_else(|e| panic!("Failed to read expected output {}: {}", out_path.display(), e));

    let mut runtime = Runtime::new(&source)
        .unwrap_or_else(|e| panic!("Failed to create runtime: {}", e.format_with_source(&source)));

    let expected_lines: Vec<&str> = expected.lines().collect();
    let mut actual_lines = Vec::new();

    // Collect all output lines
    loop {
        let line = runtime.current_line();
        if !line.is_empty() || actual_lines.is_empty() {
            actual_lines.push(line.to_string());
        }
        if !runtime.has_more() {
            break;
        }
        runtime.advance();
    }

    // Compare
    assert_eq!(
        actual_lines.len(),
        expected_lines.len(),
        "Line count mismatch in {}\nExpected {} lines: {:?}\nActual {} lines: {:?}",
        case_path.display(),
        expected_lines.len(),
        expected_lines,
        actual_lines.len(),
        actual_lines
    );

    for (i, (actual, expected)) in actual_lines.iter().zip(expected_lines.iter()).enumerate() {
        assert_eq!(
            actual, expected,
            "Line {} mismatch in {}\nExpected: {:?}\nActual: {:?}",
            i + 1,
            case_path.display(),
            expected,
            actual
        );
    }
}

/// Run an interactive trace test (.trace sidecar).
///
/// Executes a specific named path through the test case.
pub fn run_trace_test(case_path: &Path, path_name: &str) {
    let source = std::fs::read_to_string(case_path)
        .unwrap_or_else(|e| panic!("Failed to read test case {}: {}", case_path.display(), e));

    let trace_path = case_path.with_extension("trace");
    let trace_content = std::fs::read_to_string(&trace_path)
        .unwrap_or_else(|e| panic!("Failed to read trace file {}: {}", trace_path.display(), e));

    let paths = parse_trace(&trace_content);
    let trace = paths
        .iter()
        .find(|p| p.name == path_name)
        .unwrap_or_else(|| {
            let available: Vec<_> = paths.iter().map(|p| &p.name).collect();
            panic!(
                "Path '{}' not found in {}. Available paths: {:?}",
                path_name,
                trace_path.display(),
                available
            )
        });

    let mut runtime = Runtime::new(&source)
        .unwrap_or_else(|e| panic!("Failed to create runtime: {}", e.format_with_source(&source)));

    for (step_idx, step) in trace.steps.iter().enumerate() {
        match step {
            Step::Assert(assertion) => {
                execute_assertion(&runtime, assertion, case_path, path_name, step_idx);
            }
            Step::Action(action) => {
                execute_action(&mut runtime, action, case_path, path_name, step_idx);
            }
        }
    }
}

/// Run an error test (.err sidecar).
///
/// Expects the runtime to fail with an error containing the specified substrings.
pub fn run_error_test(case_path: &Path) {
    let source = std::fs::read_to_string(case_path)
        .unwrap_or_else(|e| panic!("Failed to read test case {}: {}", case_path.display(), e));

    let err_path = case_path.with_extension("err");
    let expected = std::fs::read_to_string(&err_path)
        .unwrap_or_else(|e| panic!("Failed to read expected error {}: {}", err_path.display(), e));

    match Runtime::new(&source) {
        Ok(_) => {
            panic!(
                "Expected error in {} but script executed successfully",
                case_path.display()
            );
        }
        Err(err) => {
            let err_string = err.format_with_source(&source);
            let err_lower = err_string.to_lowercase();

            for expected_substring in expected.lines() {
                let expected_substring = expected_substring.trim();
                if expected_substring.is_empty() {
                    continue;
                }
                assert!(
                    err_lower.contains(&expected_substring.to_lowercase()),
                    "Error message missing expected substring in {}\nExpected to contain: {:?}\nActual error: {}",
                    case_path.display(),
                    expected_substring,
                    err_string
                );
            }
        }
    }
}

// =============================================================================
// Trace File Parsing
// =============================================================================

/// Parse a trace file into a list of named paths.
pub fn parse_trace(content: &str) -> Vec<TracePath> {
    let mut paths = Vec::new();
    let mut current_path: Option<TracePath> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Path delimiter
        if line.starts_with("--- path:") {
            // Save previous path if any
            if let Some(path) = current_path.take() {
                paths.push(path);
            }
            let name = line
                .strip_prefix("--- path:")
                .unwrap()
                .trim()
                .to_string();
            current_path = Some(TracePath {
                name,
                steps: Vec::new(),
            });
            continue;
        }

        // Must be inside a path
        let path = current_path.as_mut().expect("Step outside of path block");

        // Strip inline comments
        let line = if let Some(idx) = line.find("  #") {
            line[..idx].trim()
        } else {
            line
        };

        // Parse the step
        if let Some(step) = parse_step(line) {
            path.steps.push(step);
        }
    }

    // Don't forget the last path
    if let Some(path) = current_path {
        paths.push(path);
    }

    paths
}

fn parse_step(line: &str) -> Option<Step> {
    // Line assertion: > text
    if let Some(text) = line.strip_prefix("> ") {
        return Some(Step::Assert(Assertion::Line(text.to_string())));
    }
    if line == ">" {
        return Some(Step::Assert(Assertion::Line(String::new())));
    }

    // Choices assertion: ? A | B | C
    if let Some(choices_str) = line.strip_prefix("? ") {
        let choices: Vec<String> = choices_str
            .split(" | ")
            .map(|s| s.trim().to_string())
            .collect();
        return Some(Step::Assert(Assertion::Choices(choices)));
    }

    // State assertions: ! done, ! has_more, ! waiting_for_choice
    if let Some(state) = line.strip_prefix("! ") {
        let state = state.trim();
        return match state {
            "done" => Some(Step::Assert(Assertion::Done)),
            "has_more" => Some(Step::Assert(Assertion::HasMore)),
            "waiting_for_choice" => Some(Step::Assert(Assertion::WaitingForChoice)),
            _ => panic!("Unknown state assertion: {}", state),
        };
    }

    // Actions: [advance], [choice N]
    if line.starts_with('[') && line.ends_with(']') {
        let inner = &line[1..line.len() - 1];
        if inner == "advance" {
            return Some(Step::Action(Action::Advance));
        }
        if let Some(idx_str) = inner.strip_prefix("choice ") {
            let idx: usize = idx_str
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("Invalid choice index: {}", idx_str));
            return Some(Step::Action(Action::SelectChoice(idx)));
        }
        panic!("Unknown action: {}", inner);
    }

    panic!("Unparseable trace line: {}", line);
}

// =============================================================================
// Execution Helpers
// =============================================================================

fn execute_assertion(
    runtime: &Runtime,
    assertion: &Assertion,
    case_path: &Path,
    path_name: &str,
    step_idx: usize,
) {
    match assertion {
        Assertion::Line(expected) => {
            let actual = runtime.current_line();
            assert_eq!(
                actual, expected,
                "Line mismatch at step {} in {} (path: {})\nExpected: {:?}\nActual: {:?}",
                step_idx,
                case_path.display(),
                path_name,
                expected,
                actual
            );
        }
        Assertion::Choices(expected) => {
            let actual = runtime.current_choices();
            assert_eq!(
                actual, expected,
                "Choices mismatch at step {} in {} (path: {})\nExpected: {:?}\nActual: {:?}",
                step_idx,
                case_path.display(),
                path_name,
                expected,
                actual
            );
        }
        Assertion::Done => {
            assert!(
                !runtime.has_more(),
                "Expected done at step {} in {} (path: {}), but has_more() is true",
                step_idx,
                case_path.display(),
                path_name
            );
        }
        Assertion::HasMore => {
            assert!(
                runtime.has_more(),
                "Expected has_more at step {} in {} (path: {}), but has_more() is false",
                step_idx,
                case_path.display(),
                path_name
            );
        }
        Assertion::WaitingForChoice => {
            assert!(
                runtime.is_waiting_for_choice(),
                "Expected waiting_for_choice at step {} in {} (path: {}), but is_waiting_for_choice() is false",
                step_idx,
                case_path.display(),
                path_name
            );
        }
    }
}

fn execute_action(
    runtime: &mut Runtime,
    action: &Action,
    case_path: &Path,
    path_name: &str,
    step_idx: usize,
) {
    match action {
        Action::Advance => {
            runtime.advance();
        }
        Action::SelectChoice(idx) => {
            runtime.select_choice(*idx).unwrap_or_else(|e| {
                panic!(
                    "select_choice({}) failed at step {} in {} (path: {}): {:?}",
                    idx,
                    step_idx,
                    case_path.display(),
                    path_name,
                    e
                )
            });
        }
    }
}

// =============================================================================
// Path Resolution Helper
// =============================================================================

/// Get the path to the test cases directory.
pub fn cases_dir() -> std::path::PathBuf {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("tests").join("cases")
}

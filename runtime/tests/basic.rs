//! Basic dialogue tests - simple lines and empty handling.

mod support;

use bobbin_runtime::{HostState, Runtime, VariableStorage};
use std::sync::Arc;
use support::{EmptyHostState, MemoryStorage};

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
    let storage: Arc<dyn VariableStorage> = Arc::new(MemoryStorage::new());
    let host: Arc<dyn HostState> = Arc::new(EmptyHostState);
    let runtime = Runtime::new("", storage, host).unwrap();
    assert_eq!(runtime.current_line(), "");
    assert!(!runtime.has_more());
}

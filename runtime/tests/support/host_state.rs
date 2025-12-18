//! Mock implementation of HostState for testing.

use bobbin_runtime::{HostState, Value};
use std::collections::HashMap;

/// Mock implementation of HostState for testing.
///
/// Allows test code to configure host variable values that will be
/// returned when the runtime looks them up via `extern` declarations.
#[derive(Debug, Default)]
pub struct MockHostState {
    values: HashMap<String, Value>,
}

impl MockHostState {
    /// Create a new empty mock host state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a host variable value for testing.
    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        self.values.insert(name.into(), value);
    }
}

impl HostState for MockHostState {
    fn lookup(&self, name: &str) -> Option<Value> {
        self.values.get(name).cloned()
    }
}
